"""Render the fixed 2026-09-18 Development comparison; never run an optimizer.

Read the complete compressed reports, verify their identities and raw scores,
and derive every table/curve from all declared cases. Requires matplotlib for
the figure; the benchmark verifier itself uses the Python standard library.
"""
from __future__ import annotations

import argparse
import gzip
import hashlib
import json
from pathlib import Path
import statistics
import struct
import tempfile
import zlib

from tdi_engine_store import identity
from tdi_optuna_benchmark import verify_report

PREFIX = "2026-09-18-forge-session"
SOURCE = "a4d74690f605acfe3733e163446adf85001ab494"
PREQUAL_SOURCE = "d7c0714267e4428f386f2661f31ab825fbe51552"
PNG_PIXEL_SHA256 = "5401011779ee509c29c50feda6e909fe7c3e88fc37e6f40c5e4f3ff2eebf86c1"
INCOMPLETE_EVENTS_SHA256 = "c388a2a9e2c6e36a3dc14ae5c3590f7e179f47a3b3ec77a15e7321c3338edeb2"
INCOMPLETE_MANIFEST_SHA256 = "e54a43e6b641f0059b0a7b22bad9da0bbd3f360a189c7e9a09af233be7bdf1cb"
INCOMPLETE_MANIFEST_ID = "7df0ea2e685540bbde92714a8adff42c4af5677c64393d55803c0e709d44b227"
FILES = {
    "transport": PREFIX + "-transport.json.gz",
    "quality32": PREFIX + "-quality32.json.gz",
    "quality64": PREFIX + "-quality64.json.gz",
    "preliminary32": PREFIX + "-prequalification32.json.gz",
}
LABELS = {
    "forge-random-session": "Forge random",
    "forge-tpe-session": "Forge TPE 10",
    "forge-gp-session": "Forge GP",
    "forge-tpe-early-session": "Forge TPE 4",
    "optuna-random": "Optuna random",
    "optuna-tpe": "Optuna independent",
    "optuna-tpe-multivariate": "Optuna multi 10",
    "optuna-tpe-multivariate-early": "Optuna multi 4",
}


def read_report(path):
    with gzip.open(path, "rb") as source, tempfile.NamedTemporaryFile() as temp:
        payload = source.read(64 * 1024 * 1024 + 1)
        if len(payload) > 64 * 1024 * 1024:
            raise ValueError("report exceeds verifier byte limit")
        temp.write(payload)
        temp.flush()
        return verify_report(temp.name)


def png_pixel_digest(path):
    """Hash the lossless PNG image payload while ignoring ancillary metadata."""
    payload = Path(path).read_bytes()
    if not payload.startswith(b"\x89PNG\r\n\x1a\n"):
        raise ValueError("published curve is not a PNG")
    offset = 8
    ihdr = None
    idat = []
    saw_iend = False
    while offset < len(payload):
        if offset + 12 > len(payload):
            raise ValueError("truncated PNG chunk")
        size = struct.unpack(">I", payload[offset:offset + 4])[0]
        kind = payload[offset + 4:offset + 8]
        end = offset + 12 + size
        if end > len(payload):
            raise ValueError("truncated PNG payload")
        data = payload[offset + 8:offset + 8 + size]
        expected_crc = struct.unpack(">I", payload[offset + 8 + size:end])[0]
        if zlib.crc32(kind + data) & 0xFFFFFFFF != expected_crc:
            raise ValueError("published curve has invalid PNG CRC")
        if kind == b"IHDR":
            if ihdr is not None or size != 13:
                raise ValueError("invalid PNG IHDR")
            ihdr = data
        elif kind == b"IDAT":
            idat.append(data)
        elif kind == b"IEND":
            saw_iend = True
            if end != len(payload):
                raise ValueError("trailing bytes after PNG IEND")
            break
        offset = end
    if ihdr is None or not idat or not saw_iend:
        raise ValueError("incomplete PNG structure")
    try:
        pixels = zlib.decompress(b"".join(idat))
    except zlib.error as error:
        raise ValueError("invalid PNG image stream") from error
    return hashlib.sha256(ihdr + pixels).hexdigest()


def verify_incomplete_evidence(directory, preliminary):
    """Validate the retained interrupted budget-64 evidence as incomplete data."""
    incomplete = preliminary.get("incomplete_budget64")
    if not isinstance(incomplete, dict) or incomplete.get("status") != "incomplete-no-report":
        raise ValueError("incorrect preliminary evidence classification")
    retained = incomplete.get("retained_files")
    if not isinstance(retained, list) or len(retained) != 2:
        raise ValueError("incomplete evidence must retain exactly manifest and events")
    rows = {row.get("source_name"): row for row in retained if isinstance(row, dict)}
    expected = {
        "events.jsonl": (PREFIX + "-incomplete64-events.jsonl.gz", INCOMPLETE_EVENTS_SHA256),
        "manifest.json": (PREFIX + "-incomplete64-manifest.json.gz", INCOMPLETE_MANIFEST_SHA256),
    }
    if set(rows) != set(expected):
        raise ValueError("unexpected incomplete evidence inventory")
    for source_name, (filename, digest) in expected.items():
        row = rows[source_name]
        if row.get("file") != filename or row.get("sha256") != digest:
            raise ValueError("incomplete evidence inventory is not pinned: " + source_name)
        raw = (directory / filename).read_bytes()
        if hashlib.sha256(raw).hexdigest() != digest or len(raw) != row.get("bytes"):
            raise ValueError("incomplete evidence bytes differ from pinned artifact: " + filename)

    manifest_path = directory / expected["manifest.json"][0]
    with gzip.open(manifest_path, "rb") as source:
        payload = source.read(1024 * 1024 + 1)
    if len(payload) > 1024 * 1024:
        raise ValueError("incomplete manifest exceeds verifier byte limit")
    try:
        manifest_record = json.loads(payload)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ValueError("invalid incomplete manifest JSON") from error
    if not isinstance(manifest_record, dict):
        raise ValueError("incomplete manifest must be an object")
    manifest_id = manifest_record.get("identity")
    manifest = {key: value for key, value in manifest_record.items() if key != "identity"}
    if manifest_id != INCOMPLETE_MANIFEST_ID or manifest_id != identity("tdi-optuna-manifest/v1", manifest):
        raise ValueError("incomplete manifest identity mismatch")
    if manifest.get("tdi_source_commit") != PREQUAL_SOURCE:
        raise ValueError("incomplete manifest has incorrect prequalification source")
    protocol = manifest.get("protocol")
    if not isinstance(protocol, dict):
        raise ValueError("incomplete manifest protocol missing")
    required_protocol = {
        "schema": 1,
        "kind": "tdi-optuna-session-comparison/v1",
        "profile": "session-budget64",
        "domain": "Development",
        "confirmatory": False,
        "evaluations_per_arm": 64,
        "failure_policy": "preserve-partial-records-stop-no-automatic-retry",
        "scientific_verdict": "not-assessed",
    }
    if any(protocol.get(key) != value for key, value in required_protocol.items()):
        raise ValueError("incomplete manifest is not the retained Development budget-64 protocol")
    tasks, seeds, arms = protocol.get("tasks"), protocol.get("seeds"), protocol.get("arms")
    if not all(isinstance(values, list) and values for values in (tasks, seeds, arms)):
        raise ValueError("incomplete manifest has empty run dimensions")
    if any(len(values) != len(set(values)) for values in (tasks, seeds, arms)):
        raise ValueError("incomplete manifest has duplicate run dimensions")
    budget = protocol["evaluations_per_arm"]
    allowed = set((task, seed, arm) for task in tasks for seed in seeds for arm in arms)

    events_path = directory / expected["events.jsonl"][0]
    started = set()
    current = None
    next_trial = 0
    event_count = 0
    decompressed_bytes = 0
    max_events = len(allowed) * (budget + 1)
    try:
        with gzip.open(events_path, "rb") as source:
            for raw_line in source:
                decompressed_bytes += len(raw_line)
                event_count += 1
                if decompressed_bytes > 64 * 1024 * 1024 or event_count > max_events:
                    raise ValueError("incomplete event stream exceeds verifier limits")
                try:
                    event = json.loads(raw_line)
                except (UnicodeDecodeError, json.JSONDecodeError) as error:
                    raise ValueError("invalid incomplete event JSON") from error
                if not isinstance(event, dict):
                    raise ValueError("incomplete event must be an object")
                kind = event.get("kind")
                key = (event.get("task"), event.get("seed"), event.get("arm"))
                if key not in allowed:
                    raise ValueError("incomplete event references undeclared run")
                if kind == "start":
                    if current is not None and next_trial != budget:
                        raise ValueError("new run starts after an incomplete non-final run")
                    if key in started:
                        raise ValueError("duplicate start in incomplete event stream")
                    started.add(key)
                    current, next_trial = key, 0
                elif kind == "trial":
                    if current != key or type(event.get("trial")) is not int or event["trial"] != next_trial:
                        raise ValueError("non-contiguous trial in incomplete event stream")
                    if next_trial >= budget:
                        raise ValueError("run exceeds declared incomplete-event budget")
                    parameters = event.get("parameters")
                    if not isinstance(parameters, dict) or set(parameters) != {"x", "y"}:
                        raise ValueError("invalid parameters in incomplete trial")
                    if parameters["x"] not in protocol.get("values", []) or parameters["y"] not in protocol.get("values", []):
                        raise ValueError("incomplete trial uses value outside declared domain")
                    if type(event.get("loss")) is not int or type(event.get("best_loss")) is not int:
                        raise ValueError("incomplete trial loss fields must be integers")
                    next_trial += 1
                else:
                    raise ValueError("unexpected event kind in incomplete evidence")
    except (OSError, EOFError) as error:
        raise ValueError("invalid incomplete gzip event stream") from error
    if not started or current is None or not (0 < next_trial < budget):
        raise ValueError("retained event stream does not end in a partial run")
    if len(started) >= len(allowed):
        raise ValueError("retained event stream is not an incomplete campaign")
    return {"started_runs": len(started), "events": event_count, "partial_trials": next_trial}


def verify_inventory(directory, reports):
    """Check stored bytes and all completed reports; partial logs stay partial."""
    inventory = json.loads((directory / f"{PREFIX}-artifacts.json").read_text())
    preliminary = json.loads((directory / f"{PREFIX}-prequalification.json").read_text())
    if preliminary.get("source") != PREQUAL_SOURCE:
        raise ValueError("prequalification inventory has incorrect source")
    known = {FILES[name]: report for name, report in reports.items()}
    records = [(row, "gzip_bytes", "gzip_sha256") for row in inventory["files"]]
    records += [(row, "bytes", "sha256") for row in preliminary["completed_reports"]]
    records += [(row, "bytes", "sha256") for row in preliminary["incomplete_budget64"]["retained_files"]]
    for row, size_key, hash_key in records:
        name = row["file"]
        if Path(name).name != name or not name.startswith(PREFIX + "-"):
            raise ValueError("invalid inventory path")
        raw = (directory / name).read_bytes()
        if len(raw) != row[size_key] or hashlib.sha256(raw).hexdigest() != row[hash_key]:
            raise ValueError("artifact bytes differ from inventory: " + name)
        if "report_identity" in row:
            if name not in known:
                known[name] = read_report(directory / name)
            if known[name]["identity"] != row["report_identity"]:
                raise ValueError("inventory report identity mismatch")
    if set(row["file"] for row in inventory["files"]) != set(FILES.values()):
        raise ValueError("incomplete primary inventory")
    for row in preliminary["completed_reports"]:
        if known[row["file"]]["manifest"].get("tdi_source_commit") != PREQUAL_SOURCE:
            raise ValueError("prequalification report has incorrect source: " + row["file"])
    if len(known) != 7:
        raise ValueError("incorrect completed-report inventory")
    incomplete_summary = verify_incomplete_evidence(directory, preliminary)
    return len(known), incomplete_summary


def trajectories(report, limit=None):
    return {(run["task"], run["seed"], run["arm"]): [
        (trial["parameters"], trial["loss"], trial["best_loss"])
        for trial in run["trials"][:limit]] for run in report["raw_runs"]}


def table(headers, rows):
    return "\n".join(["| " + " | ".join(headers) + " |",
                      "| " + " | ".join(["---"] * len(headers)) + " |"] +
                     ["| " + " | ".join(map(str, row)) + " |" for row in rows])


def metrics(report):
    return {(row["task"], row["arm"]): row for row in report["summary"]["aggregates"]}


def comparisons(report, left, right, metric):
    values = metrics(report)
    tasks = [task for task in report["manifest"]["protocol"]["tasks"]
             if task != "categorical-interaction"]
    pairs = [(values[task, left][metric], values[task, right][metric]) for task in tasks]
    return "/".join(str(sum(predicate(a, b) for a, b in pairs))
                    for predicate in (lambda a, b: a < b, lambda a, b: a == b, lambda a, b: a > b))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--directory", type=Path,
                        default=Path(__file__).resolve().parents[1] / "docs/engineering/benchmarks")
    parser.add_argument("--verify-only", action="store_true",
                        help="verify published evidence without rendering or writing files")
    args = parser.parse_args()
    output = args.directory
    reports = {name: read_report(output / filename) for name, filename in FILES.items()}
    expected_profiles = {"transport": "transport-development", "quality32": "session-development",
                         "quality64": "session-budget64", "preliminary32": "session-development"}
    for name, profile in expected_profiles.items():
        if reports[name]["manifest"]["protocol"]["profile"] != profile:
            raise ValueError("report has incorrect profile: " + name)
    for name in ("transport", "quality32", "quality64"):
        if reports[name]["manifest"]["tdi_source_commit"] != SOURCE:
            raise ValueError("incorrect executed qualification source")
    if trajectories(reports["quality32"]) != trajectories(reports["quality64"], 32):
        raise ValueError("cross-budget first-32 trajectories differ")
    if trajectories(reports["quality32"]) != trajectories(reports["preliminary32"]):
        raise ValueError("qualification corrections changed search trajectories")
    if args.verify_only:
        verified_count, incomplete_summary = verify_inventory(output, reports)
    else:
        verified_count, incomplete_summary = None, None

    transport = reports["transport"]
    transport_rows = []
    for strategy, label in (("forge-tpe", "TPE"), ("forge-gp", "GP")):
        times = [statistics.median(int(run["observed_adapter_ns"]) / 1e9
                                   for run in transport["raw_runs"] if run["arm"] == arm)
                 for arm in (strategy, strategy + "-session")]
        ratios = [row["replay_over_session"] for row in transport["summary"]["transport_pairs"]
                  if row["strategy"] == strategy]
        transport_rows.append([label, *(f"{value:.6f}" for value in times),
                               f"{statistics.median(ratios):.3f}", len(ratios)])
    text = ["# Forge sessions and early TPE — executed Development evidence, 2026-09-18", "",
            f"The persistent adapter is **{transport_rows[0][3]}× faster for TPE and {transport_rows[1][3]}× faster for GP** "
            "in the median of paired replay/session ratios on this host. All 60 paired "
            "parameter/loss/incumbent trajectories match exactly. This is a transport improvement, "
            "not an intrinsic speed comparison with Optuna.", "",
            "All three qualified campaigns completed: **3,000 runs and 142,080 evaluations**. "
            "Quality covers nine public categorical tasks, twenty fixed seeds, eight arms and "
            "budgets of 32 and 64 evaluations including the common baseline. "
            "Eight tasks discriminate search quality; categorical-interaction starts at its optimum "
            "and is retained but excluded from win/tie/loss counts.", "",
            "The early TPE is an opt-in, separately versioned candidate; these outcomes do not "
            "promote it to a default. No single policy is selected retrospectively for each task. "
            "Tables include every arm and every task, including unfavorable cases.", "",
            "## Paired transport", "",
            table(["Strategy", "Replay median seconds", "Session median seconds", "Median paired ratio", "Pairs"],
                  transport_rows), "",
            "Each case performs 32 evaluations. Replay also receives the shared pidfd completion "
            "improvement, so these ratios isolate the measured adapter transition to persistent "
            "sessions against an already improved replay path. Ratio-of-medians and median-of-ratios "
            "are different statistics; the reported speed factors use the latter.", "",
            "## Quality: task-level comparisons", "",
            "Cells are **lower / equal / higher** task medians (eight informative tasks); lower "
            "is better. These are descriptive counts, not statistical significance, independent "
            "task-population inference, or universal superiority. All alternatives were fixed "
            "before execution; the equally early Optuna comparator is retained.", ""]
    contrast_rows = []
    for budget in (32, 64):
        report = reports[f"quality{budget}"]
        for left in ("forge-tpe-session", "forge-gp-session", "forge-tpe-early-session"):
            for right in ("optuna-tpe-multivariate", "optuna-tpe-multivariate-early"):
                contrast_rows.append([budget, LABELS[left], LABELS[right],
                                      comparisons(report, left, right, "median_final_regret"),
                                      comparisons(report, left, right, "median_normalized_auc")])
    text += [table(["Budget", "Forge", "Comparator", "Final regret", "Incumbent AUC"], contrast_rows), "",
             "The early-start change improves some tasks and harms others. Its comparison with "
             "original Forge TPE is reported independently of the Optuna contrasts:", "",
             table(["Budget", "Early vs original: final regret", "Early vs original: AUC"], [
                 [budget, comparisons(reports[f"quality{budget}"], "forge-tpe-early-session", "forge-tpe-session", "median_final_regret"),
                  comparisons(reports[f"quality{budget}"], "forge-tpe-early-session", "forge-tpe-session", "median_normalized_auc")]
                 for budget in (32, 64)]), ""]
    for budget in (32, 64):
        report = reports[f"quality{budget}"]
        protocol = report["manifest"]["protocol"]
        arms = protocol["arms"]
        values = metrics(report)
        text += [f"## Budget {budget}: all task medians", "",
                 "Final regret is best observed loss minus the exact finite-grid minimum; zero is optimal.", "",
                 table(["Task"] + [LABELS[arm] for arm in arms], [
                     [task] + [f"{values[task, arm]['median_final_regret']:g}" for arm in arms]
                     for task in protocol["tasks"]]), "",
                 "Incumbent AUC is the mean regret across all evaluations, divided by the exact "
                 "grid loss range. It includes the common baseline and measures the complete "
                 "learning trajectory; it is not simply the final quality.", "",
                 table(["Task"] + [LABELS[arm] for arm in arms], [
                     [task] + [f"{values[task, arm]['median_normalized_auc']:.6f}" for arm in arms]
                     for task in protocol["tasks"]]), ""]
        times = []
        for arm in arms:
            chosen = [row for row in report["summary"]["runs"] if row["arm"] == arm]
            times.append([LABELS[arm], f"{statistics.median(int(row['observed_adapter_ns']) / 1e6 for row in chosen):.3f}",
                          f"{statistics.median(row['unique_points'] for row in chosen):g}",
                          sum(row["final_regret"] == 0 for row in chosen if row["task"] != "categorical-interaction")])
        text += ["Observed timing covers initialization, proposal, verification, objective, durable "
                 "writes and shutdown. Medians below pool all 180 task/seed runs per arm; optimum "
                 "counts exclude the noninformative task (160 cases).", "",
                 table(["Arm", "Median milliseconds/run", "Median unique points", "Optimum cases / 160"], times), ""]

    text += ["## Complete learning curves at budget 64", "",
             f"![Median normalized incumbent regret across twenty seeds]({PREFIX}-curves64.png)", "",
             "Each curve is the pointwise median over twenty runs; it is not a single run or a "
             "confidence interval. The vertical scale is symmetric-log with a linear region near "
             "zero. Every arm and task is plotted, including the nondiscriminating zero panel.", "",
             "## Reproducibility and preserved earlier evidence", "",
             f"Executed TDI source: `{SOURCE}`. Forge source: "
             "`83c8c572a692016fb786e661c47a249112b6cadd`, subsequently merged by "
             "[Forge #41](https://github.com/Memorithm/Forge/pull/41). SciRust GP source: "
             "`146575107005c24a47682dcaa08c4cd9464d1cc3`. Optuna: **5.0.0**. "
             "The full environment, dependency lock hash, Python identity, individual source hashes "
             "and observed Forge binary hash are embedded in each manifest. Declared source and "
             "observed binary hashes are not build attestations.", "",
             "The frozen protocol is [forge-session-comparison.md](../forge-session-comparison.md); "
             "implementation and qualification are in [TDI #503](https://github.com/Memorithm/TDI/pull/503). "
             "All **1,440** budget-64 trajectories have exactly the budget-32 trajectory as their "
             "first 32 trials. All **1,440** corrected-source budget-32 trajectories also match the "
             "initial candidate's completed campaign.", "",
             "The initial TDI candidate `d7c0714267e4428f386f2661f31ab825fbe51552` had completed "
             "smoke, transport and budget-32 campaigns before review found two robustness issues: "
             "high descriptor counts in pidfd waiting and incomplete recovery-provenance hashing. "
             "The initial budget-64 campaign lost its execution-server session and has **no complete "
             "report**. Its partial records were preserved. Qualification reran the entire fixed "
             "campaign matrix on the corrected source, without changing algorithms, tasks, seeds "
             "or budgets, and without using earlier outcomes to tune anything.", "",
             f"The [prequalification inventory]({PREFIX}-prequalification.json) identifies all retained completed reports "
             "and the incomplete campaign. The separately retained full preliminary archive contains "
             "all 775,635 files (164,918,128 compressed bytes), SHA-256 "
             "`c1dff5a97860a887e600710477f82039b81505f9c13e4d620166037c6886d20f`. "
             "The incomplete campaign is not included in completed-evaluation counts.", ""]
    provenance = []
    for name, filename in FILES.items():
        path = output / filename
        report = reports[name]
        provenance.append({"role": name, "file": filename, "gzip_sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                           "gzip_bytes": path.stat().st_size, "report_identity": report["identity"],
                           "source": report["manifest"]["tdi_source_commit"],
                           "runs": len(report["raw_runs"]),
                           "evaluations": sum(len(run["trials"]) for run in report["raw_runs"])})
    text += [table(["Report", "Runs", "Evaluations", "Report identity"], [
        [f"[{row['role']}]({row['file']})", row["runs"], row["evaluations"], f"`{row['report_identity']}`"]
        for row in provenance]), "",
        f"Compressed-file hashes and sizes: [{PREFIX}-artifacts.json]({PREFIX}-artifacts.json).", "",
        "To verify identities, independently recompute raw objective scores and summaries, check "
        "the 60 transport pairs and both 1,440-trajectory equivalences, and regenerate this "
        "document and figure:", "", "```bash", "python3 scripts/render-forge-session-results.py", "```", "",
        "For read-only verification without matplotlib or output writes, use "
        "`python3 scripts/render-forge-session-results.py --verify-only`. This also verifies "
        "the compressed-file inventories, all seven completed reports and preservation of the "
        "incomplete campaign as incomplete. A dedicated CI job runs this check.", "",
        "Python with matplotlib is needed only for rendering. Running this command does not "
        "run a new optimization campaign. The protocol document gives the separately pinned "
        "commands for actual execution.", "",
        "## Scope and remaining performance work", "",
        "Forge still has higher observed adapter cost than in-process Optuna in this comparison. "
        "Persistent sessions remove repeated process launch and full replay, but durable command "
        "journals, verification stages, filesystem operations and the cross-process bridge remain. "
        "These costs require separate profiling before another protocol-preserving optimization.", "",
        "Quality and anytime convergence are different objectives. Early feedback can lock onto "
        "a poor basin, as the retained plateau-gate outcomes illustrate. The categorical GP also "
        "does not dominate TPE. A next search-policy candidate needs a new, prospectively frozen "
        "comparison on a broader development population; it must retain the present losses.", "",
        "This is a two-dimensional, noiseless, finite categorical comparison on known public "
        "tasks. Forge does not repeat points; Optuna repeats remain charged. Forge enumerates "
        "admissible acquisition points whereas Optuna samples 24 candidates. Equal objective "
        "budgets do not imply equal acquisition computation. Shared-host timings and different "
        "durability architectures cannot establish intrinsic optimizer speed, power-loss "
        "durability, ML quality, scaling, GPU performance or universal superiority. No protected "
        "TDI series, confirmation boundary or default backend is changed.", ""]

    artifact_payload = {
        "schema": 1, "qualified_runs": 3000, "qualified_evaluations": 142080,
        "transport_pairs_equal": 60, "cross_budget_prefixes_equal": 1440,
        "prequalification_trajectories_equal": 1440, "files": provenance,
    }
    expected_markdown = "\n".join(text)
    if args.verify_only:
        if (output / f"{PREFIX}-results.md").read_text(encoding="utf-8") != expected_markdown:
            raise ValueError("published Markdown differs from verified reports")
        published_artifacts = json.loads((output / f"{PREFIX}-artifacts.json").read_text(encoding="utf-8"))
        if published_artifacts != artifact_payload:
            raise ValueError("published artifact summary differs from verified reports")
        if png_pixel_digest(output / f"{PREFIX}-curves64.png") != PNG_PIXEL_SHA256:
            raise ValueError("published curve pixels differ from qualified rendering")
        print(json.dumps({"verified_reports": verified_count, "qualified_runs": 3000,
                          "qualified_evaluations": 142080, "transport_pairs_equal": 60,
                          "cross_budget_prefixes_equal": 1440, "prequalification_trajectories_equal": 1440,
                          "published_outputs_verified": True, "incomplete_evidence_verified": incomplete_summary,
                          "incomplete_campaign_promoted": False}))
        return

    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    report = reports["quality64"]
    tasks = report["manifest"]["protocol"]["tasks"]
    arms = report["manifest"]["protocol"]["arms"]
    colors = ("#718096", "#176baf", "#00856a", "#6f2dbd", "#ad8b73", "#d98000", "#c3374f", "#ec6a98")
    fig, axes = plt.subplots(3, 3, figsize=(16, 12), sharex=True)
    for ax, task in zip(axes.flat, tasks):
        bounds = report["manifest"]["task_bounds"][task]
        scale = bounds["maximum"] - bounds["minimum"]
        for arm, color in zip(arms, colors):
            runs = [run for run in report["raw_runs"] if run["task"] == task and run["arm"] == arm]
            curve = [statistics.median((run["trials"][i]["best_loss"] - bounds["minimum"]) / scale
                                       for run in runs) for i in range(64)]
            ax.step(range(1, 65), curve, where="post", color=color, linewidth=1.7,
                    linestyle="--" if arm.startswith("optuna") else "-", label=LABELS[arm])
        ax.set_title(task + (" (baseline optimal)" if task == "categorical-interaction" else ""), fontsize=11)
        ax.set_yscale("symlog", linthresh=0.002)
        ax.set_ylim(-0.0001, 1)
        ax.set_xlim(1, 64)
        ax.grid(alpha=0.18)
    for ax in axes[:, 0]:
        ax.set_ylabel("Median normalized incumbent regret")
    for ax in axes[-1, :]:
        ax.set_xlabel("Evaluations, including baseline")
    fig.suptitle("Forge / Optuna: all fixed public tasks, 20 seeds, budget 64", fontsize=17)
    handles, labels = axes[0, 0].get_legend_handles_labels()
    fig.legend(handles, labels, loc="lower center", ncol=4, frameon=False, fontsize=11)
    fig.tight_layout(rect=(0, 0.065, 1, 0.96))
    fig.savefig(output / f"{PREFIX}-curves64.png", dpi=150)
    plt.close(fig)
    (output / f"{PREFIX}-results.md").write_text(expected_markdown, encoding="utf-8")
    (output / f"{PREFIX}-artifacts.json").write_text(json.dumps(artifact_payload, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"verified_reports": len(reports), "qualified_runs": 3000,
                      "qualified_evaluations": 142080, "transport_pairs_equal": 60,
                      "cross_budget_prefixes_equal": 1440, "prequalification_trajectories_equal": 1440}))


if __name__ == "__main__":
    main()
