#!/usr/bin/env python3
"""Benchmark comparison: Python speakhuman vs Rust speakhuman.

Runs identical workloads through both implementations and displays
a side-by-side performance comparison with ASCII bar charts.

Usage:
    python benchmarks/bench_compare.py

Prerequisites:
    - Python speakhuman must be installed (pip install -e .)
    - Rust speakhuman must be compiled (cd speakhuman-rs && cargo build --release)
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import time
from pathlib import Path

# ---------------------------------------------------------------------------
# Benchmark definitions
# Each benchmark is (name, callable_that_runs_N_iterations)
# ---------------------------------------------------------------------------

ITERATIONS = 100_000
REPO_ROOT = Path(__file__).resolve().parent.parent
RUST_BENCH_BIN = REPO_ROOT / "speakhuman-rs" / "target" / "release" / "speakhuman-bench"


def _import_humanize():
    sys.path.insert(0, str(REPO_ROOT / "src"))
    import speakhuman
    return speakhuman


def bench_python() -> dict[str, float]:
    """Run all benchmarks through the Python implementation."""
    h = _import_humanize()
    import datetime as dt

    results = {}

    # --- naturalsize ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        h.naturalsize(3_000_000)
        h.naturalsize(1024 * 31, True)
        h.naturalsize(3000, False, True)
    results["naturalsize"] = time.perf_counter() - start

    # --- intcomma ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        h.intcomma(1_000_000)
        h.intcomma(1_234_567.25)
        h.intcomma("10311")
    results["intcomma"] = time.perf_counter() - start

    # --- intword ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        h.intword("1000000")
        h.intword("1200000000")
        h.intword("8100000000000000000000000000000000")
    results["intword"] = time.perf_counter() - start

    # --- ordinal ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        h.ordinal(1)
        h.ordinal(103)
        h.ordinal(111)
    results["ordinal"] = time.perf_counter() - start

    # --- scientific ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        h.scientific(1000)
        h.scientific(0.3)
        h.scientific(5781651000)
    results["scientific"] = time.perf_counter() - start

    # --- fractional ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        h.fractional(0.3)
        h.fractional(1.3)
        h.fractional(1 / 3)
    results["fractional"] = time.perf_counter() - start

    # --- metric ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        h.metric(1500, "V")
        h.metric(2e8, "W")
        h.metric(220e-6, "F")
    results["metric"] = time.perf_counter() - start

    # --- apnumber ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        h.apnumber(0)
        h.apnumber(5)
        h.apnumber(10)
    results["apnumber"] = time.perf_counter() - start

    # --- naturaldelta ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        h.naturaldelta(dt.timedelta(days=7))
        h.naturaldelta(dt.timedelta(seconds=30))
        h.naturaldelta(dt.timedelta(days=500))
    results["naturaldelta"] = time.perf_counter() - start

    # --- natural_list ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        h.natural_list(["one", "two", "three"])
        h.natural_list(["one", "two"])
        h.natural_list(["one"])
    results["natural_list"] = time.perf_counter() - start

    # --- precisedelta ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        h.precisedelta(dt.timedelta(seconds=3633, days=2, microseconds=123000))
        h.precisedelta(dt.timedelta(seconds=1))
        h.precisedelta(dt.timedelta(days=370, hours=4, seconds=3))
    results["precisedelta"] = time.perf_counter() - start

    return results


def bench_rust() -> dict[str, float]:
    """Run all benchmarks through the Rust implementation."""
    # Build the bench binary if needed
    build_result = subprocess.run(
        ["cargo", "build", "--release", "--bin", "speakhuman-bench"],
        cwd=REPO_ROOT / "speakhuman-rs",
        capture_output=True,
        text=True,
    )
    if build_result.returncode != 0:
        print(f"Failed to build Rust benchmark binary:\n{build_result.stderr}")
        sys.exit(1)

    # Run it
    result = subprocess.run(
        [str(RUST_BENCH_BIN)],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print(f"Rust benchmark failed:\n{result.stderr}")
        sys.exit(1)

    return json.loads(result.stdout)


# ---------------------------------------------------------------------------
# Display
# ---------------------------------------------------------------------------

COLORS = {
    "python": "\033[93m",  # yellow
    "rust": "\033[96m",    # cyan
    "reset": "\033[0m",
    "bold": "\033[1m",
    "dim": "\033[2m",
    "green": "\033[92m",
    "red": "\033[91m",
    "bar_py": "\033[43m",   # yellow bg
    "bar_rs": "\033[46m",   # cyan bg
}


def bar(value: float, max_val: float, width: int = 30, color: str = "") -> str:
    """Draw a horizontal bar."""
    filled = int((value / max_val) * width) if max_val > 0 else 0
    filled = max(1, min(filled, width))
    reset = COLORS["reset"]
    return f"{color}{'█' * filled}{reset}{'░' * (width - filled)}"


def display_results(py_results: dict[str, float], rs_results: dict[str, float]) -> None:
    c = COLORS
    iters = ITERATIONS

    print()
    print(f"{c['bold']}{'═' * 80}{c['reset']}")
    print(f"{c['bold']}  SPEAKHUMAN BENCHMARK: Python vs Rust  ({iters:,} iterations × 3 calls each){c['reset']}")
    print(f"{c['bold']}{'═' * 80}{c['reset']}")
    print()

    # Header
    print(
        f"  {'Function':<16} {'Python':>10} {'Rust':>10} {'Speedup':>10}   "
        f"{'Comparison'}"
    )
    print(f"  {'─' * 16} {'─' * 10} {'─' * 10} {'─' * 10}   {'─' * 30}")

    total_py = 0.0
    total_rs = 0.0
    speedups = []

    for name in py_results:
        py_time = py_results[name]
        rs_time = rs_results.get(name, 0.0)
        total_py += py_time
        total_rs += rs_time

        speedup = py_time / rs_time if rs_time > 0 else float("inf")
        speedups.append(speedup)

        max_time = max(py_time, rs_time)

        py_bar = bar(py_time, max_time, 14, c["python"])
        rs_bar = bar(rs_time, max_time, 14, c["rust"])

        if speedup >= 2.0:
            speed_color = c["green"]
        elif speedup >= 1.0:
            speed_color = c["dim"]
        else:
            speed_color = c["red"]

        py_ms = py_time * 1000
        rs_ms = rs_time * 1000

        print(
            f"  {name:<16} {py_ms:>8.1f}ms {rs_ms:>8.1f}ms "
            f"{speed_color}{speedup:>8.1f}x{c['reset']}   "
            f"{c['python']}Py{c['reset']} {py_bar} {c['rust']}Rs{c['reset']} {rs_bar}"
        )

    # Totals
    print(f"  {'─' * 16} {'─' * 10} {'─' * 10} {'─' * 10}")
    total_speedup = total_py / total_rs if total_rs > 0 else float("inf")
    avg_speedup = sum(speedups) / len(speedups) if speedups else 0

    total_color = c["green"] if total_speedup >= 2.0 else c["dim"]
    print(
        f"  {c['bold']}{'TOTAL':<16}{c['reset']} "
        f"{total_py * 1000:>8.1f}ms {total_rs * 1000:>8.1f}ms "
        f"{total_color}{c['bold']}{total_speedup:>8.1f}x{c['reset']}"
    )

    print()
    print(f"{c['bold']}{'═' * 80}{c['reset']}")
    print(f"  {c['bold']}Average speedup: {avg_speedup:.1f}x faster{c['reset']}")
    print(f"  {c['python']}██ Python{c['reset']}  {c['rust']}██ Rust{c['reset']}")
    print(f"{c['bold']}{'═' * 80}{c['reset']}")
    print()

    # Ops/sec summary
    print(f"  {c['bold']}Throughput (operations/sec):{c['reset']}")
    print(f"  {'Function':<16} {'Python ops/s':>14} {'Rust ops/s':>14}")
    print(f"  {'─' * 16} {'─' * 14} {'─' * 14}")
    for name in py_results:
        py_time = py_results[name]
        rs_time = rs_results.get(name, 0.0)
        calls = iters * 3  # 3 calls per iteration
        py_ops = calls / py_time if py_time > 0 else 0
        rs_ops = calls / rs_time if rs_time > 0 else 0
        print(f"  {name:<16} {py_ops:>12,.0f}/s {rs_ops:>12,.0f}/s")
    print()


def main() -> None:
    print(f"\n{COLORS['bold']}Running Python benchmarks...{COLORS['reset']}")
    py_results = bench_python()

    print(f"{COLORS['bold']}Running Rust benchmarks...{COLORS['reset']}")
    rs_results = bench_rust()

    display_results(py_results, rs_results)


if __name__ == "__main__":
    main()
