"""Execution engine for schema-2 TDI cgroup-v2 campaigns."""
import contextlib
import os
from pathlib import Path
import subprocess
import sys

import tdi_experiment_supervisor as durable
from tdi_linux_containment import CgroupV2Attempt, ContainmentError, spawn_reaper
from tdi_cgroup_delegation import require_inside_delegation
from tdi_linux_contract import (
    attempt_identity, execution_profile, journal_binding, recovery_marker,
    validate_plan,
)

TECHNICAL_FAILURES = set(durable.TECHNICAL_FAILURES) | {"ContainmentFailed"}


def _launcher_argv(group, worker_argv):
    return [sys.executable, str(Path(__file__).with_name("tdi_cgroup_exec.py").resolve()),
            "--cgroup-procs", str(group.path / "cgroup.procs"), "--", *worker_argv]


def _terminate_and_measure(group):
    evidence = group.metrics()
    if group.is_populated():
        group.kill()
    return evidence


def _recover_via_reaper(reaper, timeout=5.0):
    """Hand containment to the detached reaper without sending release.

    Closing the control pipe is the same signal used when the supervisor
    disappears unexpectedly.  The reaper must complete recovery before this
    supervisor records the containment failure and aborts the campaign.
    """
    with contextlib.suppress(OSError):
        os.close(reaper.control_fd)
    reaper.control_fd = -1
    try:
        code = reaper.process.wait(timeout=timeout)
    except subprocess.TimeoutExpired as error:
        reaper.process.kill()
        reaper.process.wait()
        raise ContainmentError("containment reaper did not complete recovery") from error
    if code != 0:
        raise ContainmentError(f"containment reaper recovery exited with {code}")


def _record_interrupted(journal, index, attempt_id, cgroup_parent, marker_path, reason):
    """Durably consume one interrupted attempt after reconciling containment."""
    journal.append({"kind": "Start", "index": index})
    interrupted = {
        "status": "Interrupted",
        "reason": reason,
        "attempt_id": attempt_id,
        "recovery": CgroupV2Attempt.reconcile_existing(cgroup_parent, attempt_id),
    }
    marker = recovery_marker(marker_path)
    if marker is not None:
        interrupted["reaper_marker"] = marker
    journal.append({"kind": "Finish", "index": index, "result": interrupted})


def run(plan, root, journal_path, cgroup_parent, cancelled=lambda: False,
        after_commit=lambda _: None, anchor_path=None, recovery_dir=None):
    """Run one schema-2 campaign with one isolated cgroup per durable attempt."""
    root = Path(root).resolve(strict=True)
    plan_id = validate_plan(plan, root)
    profile = execution_profile(plan)
    cgroup_parent = Path(cgroup_parent).resolve(strict=True)
    require_inside_delegation(cgroup_parent)
    recovery_dir = Path(recovery_dir) if recovery_dir else Path(str(journal_path) + ".recovery")
    recovery_dir.mkdir(parents=True, exist_ok=True)
    journal = durable.Journal(journal_path, journal_binding(plan), anchor_path=anchor_path)
    try:
        done, active, _ = journal.read()
        if active is not None:
            attempt_id = attempt_identity(plan_id, active)
            marker_path = recovery_dir / f"{attempt_id}.json"
            interrupted = {
                "status": "Interrupted",
                "reason": "previous supervisor ended before durable completion; not retried",
                "attempt_id": attempt_id,
                "recovery": CgroupV2Attempt.reconcile_existing(cgroup_parent, attempt_id),
            }
            marker = recovery_marker(marker_path)
            if marker is not None:
                interrupted["reaper_marker"] = marker
            journal.append({"kind": "Finish", "index": active, "result": interrupted})
            done, _, _ = journal.read()

        for index in plan["indices"][len(done):]:
            if cancelled():
                break
            validate_plan(plan, root)
            seed = index | (2**63 if plan["domain"] == "Validation" else 0)
            attempt_id = attempt_identity(plan_id, index)
            marker_path = recovery_dir / f"{attempt_id}.json"
            stale_path = cgroup_parent / f"tdi-{attempt_id}"

            # Recover attempts produced by older runners that could create a
            # cgroup/reaper before Start was durably appended.  The trial is
            # consumed as Interrupted rather than silently retried.
            if marker_path.exists() or stale_path.exists():
                _record_interrupted(
                    journal,
                    index,
                    attempt_id,
                    cgroup_parent,
                    marker_path,
                    "orphaned containment state existed before durable Start; recovered and not retried",
                )
                after_commit(index)
                continue

            # Publish discovery before creating any attempt-owned kernel state.
            # A crash after this point is therefore recoverable through the
            # ordinary active-attempt path above.
            journal.append({"kind": "Start", "index": index})
            group = CgroupV2Attempt(cgroup_parent, attempt_id, profile)
            reaper = None
            result = {"attempt_id": attempt_id, "backend": "linux-cgroup-v2"}
            cleanup_errors = []
            abort_after_finish = False
            try:
                group.create()
                reaper = spawn_reaper(group.path, marker_path)
                worker_argv = [str(durable.artifact(root, plan["argv"][0])), *plan["argv"][1:],
                               "--tdi-seed", str(seed), "--tdi-plan-id", plan_id]
                result.update(durable.supervise(_launcher_argv(group, worker_argv),
                                                plan["timeout_seconds"],
                                                plan["max_output_bytes"], cancelled))
                result["resource_evidence"] = _terminate_and_measure(group)
                if result["status"] == "Completed":
                    if result["stdout"] is None:
                        raise durable.ContractError("worker stdout is not valid UTF-8")
                    response = durable.strict_json(result["stdout"],
                                                   max_bytes=plan["max_output_bytes"])
                    if (not isinstance(response, dict) or response.get("seed") != seed
                            or type(response.get("seed")) is not int
                            or response.get("plan_id") != plan_id
                            or response.get("status") not in ("Evaluated", "Rejected")):
                        raise durable.ContractError("worker response binding or status mismatch")
                    result["response"] = response
                validate_plan(plan, root)
            except (durable.ContractError, OSError, subprocess.SubprocessError) as error:
                result.update(status="ContractRejected", error=str(error)[:4096])
            except ContainmentError as error:
                result.update(status="ContainmentFailed", error=str(error)[:4096])
            finally:
                try:
                    if group.path.exists():
                        if group.is_populated():
                            group.kill()
                        if "resource_evidence" not in result:
                            result["resource_evidence"] = group.metrics()
                        group.cleanup()
                except ContainmentError as error:
                    cleanup_errors.append(f"cgroup cleanup: {error}")

                if reaper is not None:
                    if cleanup_errors:
                        try:
                            _recover_via_reaper(reaper)
                        except ContainmentError as error:
                            cleanup_errors.append(f"reaper recovery: {error}")
                        abort_after_finish = True
                    else:
                        try:
                            reaper.release()
                        except ContainmentError as error:
                            cleanup_errors.append(f"reaper release: {error}")
                            abort_after_finish = True

                if cleanup_errors:
                    prior = result.get("status")
                    result.update(status="ContainmentFailed", containment_errors=cleanup_errors)
                    if prior and prior != "ContainmentFailed":
                        result["prior_status"] = prior

            journal.append({"kind": "Finish", "index": index, "result": result})
            after_commit(index)
            if abort_after_finish:
                raise ContainmentError(
                    "containment cleanup/reaper failure; campaign stopped after durable failure record"
                )
        return journal.read()[0]
    finally:
        journal.close()


def exit_code_for_records(records, stopped=False):
    if stopped:
        return durable.EXIT_CANCELLED
    return durable.EXIT_TRIAL_FAILURE if any(
        record.get("result", {}).get("status") in TECHNICAL_FAILURES for record in records
    ) else durable.EXIT_OK
