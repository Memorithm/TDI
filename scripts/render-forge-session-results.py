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
import tempfile

from tdi_optuna_benchmark import verify_report

PREFIX = "2026-09-18-forge-session"
SOURCE = "a4d74690f605acfe3733e163446adf85001ab494"
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
    args = parser.parse_args()
    output = args.directory
    reports = {name: read_report(output / filename) for name, filename in FILES.items()}
    for name in ("transport", "quality32", "quality64"):
        if reports[name]["manifest"]["tdi_source_commit"] != SOURCE:
            raise ValueError("incorrect executed qualification source")
    if trajectories(reports["quality32"]) != trajectories(reports["quality64"], 32):
        raise ValueError("cross-budget first-32 trajectories differ")
    if trajectories(reports["quality32"]) != trajectories(reports["preliminary32"]):
        raise ValueError("qualification corrections changed search trajectories")

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
    (output / f"{PREFIX}-results.md").write_text("\n".join(text), encoding="utf-8")
    (output / f"{PREFIX}-artifacts.json").write_text(json.dumps({
        "schema": 1, "qualified_runs": 3000, "qualified_evaluations": 142080,
        "transport_pairs_equal": 60, "cross_budget_prefixes_equal": 1440,
        "prequalification_trajectories_equal": 1440, "files": provenance,
    }, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"verified_reports": len(reports), "qualified_runs": 3000,
                      "qualified_evaluations": 142080, "transport_pairs_equal": 60,
                      "cross_budget_prefixes_equal": 1440, "prequalification_trajectories_equal": 1440}))


if __name__ == "__main__":
    main()
