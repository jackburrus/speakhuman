#!/usr/bin/env python3
"""Benchmark comparison: Pure-Python vs Rust-accelerated speakhuman.

Runs identical workloads through both the pure-Python implementation and
the Rust-accelerated version (via PyO3), displaying a side-by-side
performance comparison with ASCII bar charts.

Usage:
    maturin develop --release
    python benchmarks/bench_compare.py

Prerequisites:
    - speakhuman must be installed with Rust extension (maturin develop --release)
"""

from __future__ import annotations

import datetime as dt
import sys
import time
from pathlib import Path

ITERATIONS = 100_000
REPO_ROOT = Path(__file__).resolve().parent.parent

sys.path.insert(0, str(REPO_ROOT / "src"))


def _check_rust_available() -> bool:
    """Check if Rust extension is available."""
    try:
        import speakhuman._speakhuman_rs  # noqa: F401

        return True
    except ImportError:
        return False


# ---------------------------------------------------------------------------
# Pure-Python benchmark (bypass Rust, call _py_* functions directly)
# ---------------------------------------------------------------------------


def bench_pure_python() -> dict[str, float]:
    """Run all benchmarks through the pure-Python implementation."""
    from speakhuman.filesize import _py_naturalsize
    from speakhuman.lists import _py_natural_list
    from speakhuman.number import (
        _py_apnumber,
        _py_fractional,
        _py_intcomma,
        _py_intword,
        _py_metric,
        _py_ordinal,
        _py_scientific,
    )
    from speakhuman.time import (
        _py_naturaldelta,
        _py_precisedelta,
    )

    results = {}

    # --- naturalsize ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        _py_naturalsize(3_000_000)
        _py_naturalsize(1024 * 31, True)
        _py_naturalsize(3000, False, True)
    results["naturalsize"] = time.perf_counter() - start

    # --- intcomma ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        _py_intcomma(1_000_000)
        _py_intcomma(1_234_567.25)
        _py_intcomma("10311")
    results["intcomma"] = time.perf_counter() - start

    # --- intword ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        _py_intword("1000000")
        _py_intword("1200000000")
        _py_intword("8100000000000000000000000000000000")
    results["intword"] = time.perf_counter() - start

    # --- ordinal ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        _py_ordinal(1)
        _py_ordinal(103)
        _py_ordinal(111)
    results["ordinal"] = time.perf_counter() - start

    # --- scientific ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        _py_scientific(1000)
        _py_scientific(0.3)
        _py_scientific(5781651000)
    results["scientific"] = time.perf_counter() - start

    # --- fractional ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        _py_fractional(0.3)
        _py_fractional(1.3)
        _py_fractional(1 / 3)
    results["fractional"] = time.perf_counter() - start

    # --- metric ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        _py_metric(1500, "V")
        _py_metric(2e8, "W")
        _py_metric(220e-6, "F")
    results["metric"] = time.perf_counter() - start

    # --- apnumber ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        _py_apnumber(0)
        _py_apnumber(5)
        _py_apnumber(10)
    results["apnumber"] = time.perf_counter() - start

    # --- naturaldelta ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        _py_naturaldelta(dt.timedelta(days=7))
        _py_naturaldelta(dt.timedelta(seconds=30))
        _py_naturaldelta(dt.timedelta(days=500))
    results["naturaldelta"] = time.perf_counter() - start

    # --- natural_list ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        _py_natural_list(["one", "two", "three"])
        _py_natural_list(["one", "two"])
        _py_natural_list(["one"])
    results["natural_list"] = time.perf_counter() - start

    # --- precisedelta ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        _py_precisedelta(dt.timedelta(seconds=3633, days=2, microseconds=123000))
        _py_precisedelta(dt.timedelta(seconds=1))
        _py_precisedelta(dt.timedelta(days=370, hours=4, seconds=3))
    results["precisedelta"] = time.perf_counter() - start

    return results


# ---------------------------------------------------------------------------
# Rust-accelerated benchmark (use the normal public API, which dispatches to Rust)
# ---------------------------------------------------------------------------


def bench_rust_accelerated() -> dict[str, float]:
    """Run all benchmarks through the Rust-accelerated public API."""
    import speakhuman

    results = {}

    # --- naturalsize ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        speakhuman.naturalsize(3_000_000)
        speakhuman.naturalsize(1024 * 31, True)
        speakhuman.naturalsize(3000, False, True)
    results["naturalsize"] = time.perf_counter() - start

    # --- intcomma ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        speakhuman.intcomma(1_000_000)
        speakhuman.intcomma(1_234_567.25)
        speakhuman.intcomma("10311")
    results["intcomma"] = time.perf_counter() - start

    # --- intword ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        speakhuman.intword("1000000")
        speakhuman.intword("1200000000")
        speakhuman.intword("8100000000000000000000000000000000")
    results["intword"] = time.perf_counter() - start

    # --- ordinal ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        speakhuman.ordinal(1)
        speakhuman.ordinal(103)
        speakhuman.ordinal(111)
    results["ordinal"] = time.perf_counter() - start

    # --- scientific ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        speakhuman.scientific(1000)
        speakhuman.scientific(0.3)
        speakhuman.scientific(5781651000)
    results["scientific"] = time.perf_counter() - start

    # --- fractional ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        speakhuman.fractional(0.3)
        speakhuman.fractional(1.3)
        speakhuman.fractional(1 / 3)
    results["fractional"] = time.perf_counter() - start

    # --- metric ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        speakhuman.metric(1500, "V")
        speakhuman.metric(2e8, "W")
        speakhuman.metric(220e-6, "F")
    results["metric"] = time.perf_counter() - start

    # --- apnumber ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        speakhuman.apnumber(0)
        speakhuman.apnumber(5)
        speakhuman.apnumber(10)
    results["apnumber"] = time.perf_counter() - start

    # --- naturaldelta ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        speakhuman.naturaldelta(dt.timedelta(days=7))
        speakhuman.naturaldelta(dt.timedelta(seconds=30))
        speakhuman.naturaldelta(dt.timedelta(days=500))
    results["naturaldelta"] = time.perf_counter() - start

    # --- natural_list ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        speakhuman.natural_list(["one", "two", "three"])
        speakhuman.natural_list(["one", "two"])
        speakhuman.natural_list(["one"])
    results["natural_list"] = time.perf_counter() - start

    # --- precisedelta ---
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        speakhuman.precisedelta(dt.timedelta(seconds=3633, days=2, microseconds=123000))
        speakhuman.precisedelta(dt.timedelta(seconds=1))
        speakhuman.precisedelta(dt.timedelta(days=370, hours=4, seconds=3))
    results["precisedelta"] = time.perf_counter() - start

    return results


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
    print(f"{c['bold']}  SPEAKHUMAN BENCHMARK: Pure Python vs Rust-Accelerated  ({iters:,} iters × 3 calls){c['reset']}")
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
    print(f"  {c['python']}██ Pure Python{c['reset']}  {c['rust']}██ Rust-Accelerated{c['reset']}")
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
    has_rust = _check_rust_available()

    if not has_rust:
        print(
            f"\n{COLORS['red']}ERROR: Rust extension not available.{COLORS['reset']}\n"
            f"Build it first with: maturin develop --release\n"
        )
        sys.exit(1)

    print(f"\n{COLORS['bold']}Running pure-Python benchmarks...{COLORS['reset']}")
    py_results = bench_pure_python()

    print(f"{COLORS['bold']}Running Rust-accelerated benchmarks...{COLORS['reset']}")
    rs_results = bench_rust_accelerated()

    display_results(py_results, rs_results)


if __name__ == "__main__":
    main()
