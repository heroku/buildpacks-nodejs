"""Tests for analyze_shard_timings. Run with: python3 -m pytest scripts/test_analyze_shard_timings.py"""

from analyze_shard_timings import (
    parse_junit_xml,
    step_durations,
    imbalance,
    aggregate_runs,
)


def test_parse_junit_xml_extracts_name_and_time():
    xml = """<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="nextest-run">
  <testsuite name="npm_integration_test">
    <testcase name="test_npm_install_with_lockfile" time="42.5" />
    <testcase name="test_npm_build_scripts" time="17.0" />
  </testsuite>
</testsuites>"""
    result = parse_junit_xml(xml)
    assert result == [
        ("npm_integration_test::test_npm_install_with_lockfile", 42.5),
        ("npm_integration_test::test_npm_build_scripts", 17.0),
    ]


def test_parse_junit_xml_handles_empty():
    xml = """<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="nextest-run"></testsuites>"""
    assert parse_junit_xml(xml) == []


def test_step_durations_returns_dict_keyed_by_step_name():
    job = {
        "name": "Integration PoC (builder:24, arm64, 1/2)",
        "steps": [
            {
                "name": "Checkout",
                "startedAt": "2026-05-26T10:00:00Z",
                "completedAt": "2026-05-26T10:00:30Z",
            },
            {
                "name": "Run integration tests (sharded)",
                "startedAt": "2026-05-26T10:01:00Z",
                "completedAt": "2026-05-26T10:09:00Z",
            },
        ],
    }
    durations = step_durations(job)
    assert durations["Checkout"] == 30.0
    assert durations["Run integration tests (sharded)"] == 480.0


def test_step_durations_ignores_steps_without_timestamps():
    job = {
        "steps": [
            {"name": "Skipped step", "startedAt": None, "completedAt": None},
            {
                "name": "Real step",
                "startedAt": "2026-05-26T10:00:00Z",
                "completedAt": "2026-05-26T10:00:10Z",
            },
        ]
    }
    durations = step_durations(job)
    assert "Skipped step" not in durations
    assert durations["Real step"] == 10.0


def test_imbalance_perfectly_balanced_returns_one():
    assert imbalance([10.0, 10.0, 10.0, 10.0]) == 1.0


def test_imbalance_one_slow_shard_inflates_ratio():
    # mean = 12.5, max = 20 -> imbalance = 1.6
    assert imbalance([10.0, 10.0, 10.0, 20.0]) == 1.6


def test_imbalance_single_shard_returns_one():
    assert imbalance([42.0]) == 1.0


def test_imbalance_empty_list_returns_zero():
    assert imbalance([]) == 0.0


def test_aggregate_runs_takes_median_per_cell():
    # Three runs, same cell, with three different test_duration values
    runs = [
        {("builder:24", "arm64", 2, 1): {"test_duration": 600.0, "setup_duration": 100.0}},
        {("builder:24", "arm64", 2, 1): {"test_duration": 660.0, "setup_duration": 110.0}},
        {("builder:24", "arm64", 2, 1): {"test_duration": 720.0, "setup_duration": 105.0}},
    ]
    result = aggregate_runs(runs)
    assert result[("builder:24", "arm64", 2, 1)]["test_duration"] == 660.0
    assert result[("builder:24", "arm64", 2, 1)]["setup_duration"] == 105.0


def test_aggregate_runs_handles_missing_cells():
    # Cell present in only 2 of 3 runs -> still aggregated from those 2
    runs = [
        {("builder:24", "arm64", 2, 1): {"test_duration": 600.0, "setup_duration": 100.0}},
        {},
        {("builder:24", "arm64", 2, 1): {"test_duration": 700.0, "setup_duration": 110.0}},
    ]
    result = aggregate_runs(runs)
    assert result[("builder:24", "arm64", 2, 1)]["test_duration"] == 650.0
