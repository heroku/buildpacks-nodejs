#!/usr/bin/env python3
"""Analyse sharded nextest CI runs.

Usage:
    python3 analyze_shard_timings.py <run-id> [<run-id> ...]

For each run-id, downloads JUnit artifacts and fetches job metadata via the gh CLI,
then aggregates per-cell timings (median across runs) and emits a markdown report
to stdout.

Requires: gh CLI authenticated against the buildpacks-nodejs repository.
"""

from __future__ import annotations

import argparse
import json
import re
import statistics
import subprocess
import sys
import tempfile
import xml.etree.ElementTree as ET
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

CellKey = tuple[str, str, int, int]

JOB_NAME_RE = re.compile(
    r"Integration PoC \((?P<builder>builder:\d+), (?P<arch>amd64|arm64), "
    r"(?P<shard_index>\d+)/(?P<shard_total>\d+)\)"
)
TEST_STEP_NAME = "Run integration tests (sharded)"


# --- pure helpers (covered by tests) -------------------------------------------------

def parse_junit_xml(xml_text: str) -> list[tuple[str, float]]:
    """Extract (qualified_test_name, duration_seconds) from a nextest junit.xml."""
    root = ET.fromstring(xml_text)
    out: list[tuple[str, float]] = []
    for suite in root.findall("testsuite"):
        suite_name = suite.attrib.get("name", "")
        for case in suite.findall("testcase"):
            name = case.attrib["name"]
            time = float(case.attrib["time"])
            qualified = f"{suite_name}::{name}" if suite_name else name
            out.append((qualified, time))
    return out


def step_durations(job: dict[str, Any]) -> dict[str, float]:
    """Map step name -> duration in seconds, skipping steps without timestamps."""
    durations: dict[str, float] = {}
    for step in job.get("steps", []):
        start = step.get("started_at")
        end = step.get("completed_at")
        if not start or not end:
            continue
        s = datetime.fromisoformat(start.replace("Z", "+00:00"))
        e = datetime.fromisoformat(end.replace("Z", "+00:00"))
        durations[step["name"]] = (e - s).total_seconds()
    return durations


def imbalance(durations: list[float]) -> float:
    """Compute max/mean. Returns 0.0 for empty input, 1.0 for single-element input."""
    if not durations:
        return 0.0
    mean = sum(durations) / len(durations)
    if mean == 0:
        return 0.0
    return max(durations) / mean


def aggregate_runs(runs: list[dict[CellKey, dict[str, float]]]) -> dict[CellKey, dict[str, float]]:
    """Combine multiple runs by computing the median for each (cell, metric)."""
    all_keys: set[CellKey] = set()
    for run in runs:
        all_keys.update(run.keys())

    aggregated: dict[CellKey, dict[str, float]] = {}
    for key in all_keys:
        per_metric: dict[str, list[float]] = {}
        for run in runs:
            cell = run.get(key)
            if cell is None:
                continue
            for metric, value in cell.items():
                per_metric.setdefault(metric, []).append(value)
        aggregated[key] = {m: statistics.median(vs) for m, vs in per_metric.items()}
    return aggregated


# --- gh CLI wrappers -----------------------------------------------------------------

def gh_run_jobs(run_id: str) -> list[dict[str, Any]]:
    """Fetch the jobs for a workflow run via `gh run view --json jobs`."""
    result = subprocess.run(
        ["gh", "run", "view", run_id, "--json", "jobs"],
        check=True, capture_output=True, text=True,
    )
    return json.loads(result.stdout)["jobs"]


def gh_download_artifacts(run_id: str, dest: Path) -> None:
    """Download all artifacts for a run into dest/."""
    dest.mkdir(parents=True, exist_ok=True)
    subprocess.run(
        ["gh", "run", "download", run_id, "-D", str(dest)],
        check=True,
    )


# --- per-run extraction --------------------------------------------------------------

def parse_job_name(name: str) -> CellKey | None:
    m = JOB_NAME_RE.search(name)
    if not m:
        return None
    return (m["builder"], m["arch"], int(m["shard_total"]), int(m["shard_index"]))


def extract_cell_stats(job: dict[str, Any]) -> tuple[CellKey, dict[str, float]] | None:
    """Build a stats dict for one job (one shard cell)."""
    key = parse_job_name(job.get("name", ""))
    if key is None:
        return None
    durations = step_durations(job)
    test_dur = durations.get(TEST_STEP_NAME, 0.0)
    setup_dur = sum(d for name, d in durations.items() if name != TEST_STEP_NAME)
    queue_dur = 0.0
    if job.get("started_at") and job.get("created_at"):
        s = datetime.fromisoformat(job["started_at"].replace("Z", "+00:00"))
        c = datetime.fromisoformat(job["created_at"].replace("Z", "+00:00"))
        queue_dur = (s - c).total_seconds()
    return key, {
        "test_duration": test_dur,
        "setup_duration": setup_dur,
        "queue_duration": queue_dur,
    }


def collect_run(run_id: str) -> dict[CellKey, dict[str, float]]:
    """Collect per-cell stats from one workflow run."""
    print(f"  fetching jobs for run {run_id}…", file=sys.stderr)
    jobs = gh_run_jobs(run_id)
    out: dict[CellKey, dict[str, float]] = {}
    for job in jobs:
        result = extract_cell_stats(job)
        if result is None:
            continue
        key, stats = result
        out[key] = stats
    return out


# --- report formatting ---------------------------------------------------------------

def fmt_seconds(s: float) -> str:
    m, sec = divmod(int(s), 60)
    return f"{m}:{sec:02d}"


def report(aggregated: dict[CellKey, dict[str, float]]) -> str:
    """Render a markdown report with one table per (builder, arch) cell."""
    cells: dict[tuple[str, str], dict[int, list[tuple[int, dict[str, float]]]]] = {}
    for (builder, arch, total, index), stats in aggregated.items():
        cells.setdefault((builder, arch), {}).setdefault(total, []).append((index, stats))

    lines: list[str] = []
    for (builder, arch), totals in sorted(cells.items()):
        lines.append(f"### {builder} {arch}\n")
        lines.append("| Shards (N) | Critical path | Best shard | Worst shard | Imbalance | Setup |")
        lines.append("|---|---|---|---|---|---|")
        for total in sorted(totals.keys()):
            shards = sorted(totals[total], key=lambda p: p[0])
            test_durs = [s["test_duration"] for _, s in shards]
            setup_med = statistics.median(s["setup_duration"] for _, s in shards)
            crit = setup_med + max(test_durs)
            lines.append(
                f"| {total} | {fmt_seconds(crit)} | {fmt_seconds(min(test_durs))} | "
                f"{fmt_seconds(max(test_durs))} | {imbalance(test_durs):.2f} | "
                f"{fmt_seconds(setup_med)} |"
            )
        lines.append("")
    return "\n".join(lines)


# --- main ----------------------------------------------------------------------------

def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("run_ids", nargs="+", help="GHA workflow run IDs (one or more)")
    args = parser.parse_args(argv)

    runs = [collect_run(rid) for rid in args.run_ids]
    aggregated = aggregate_runs(runs)
    print(report(aggregated))

    tmp = Path(tempfile.mkdtemp(prefix="shard-poc-artifacts-"))
    for rid in args.run_ids:
        gh_download_artifacts(rid, tmp / rid)
    print(f"\n_Artifacts downloaded to {tmp}_", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
