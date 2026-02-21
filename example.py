#!/usr/bin/env python3
"""Example demonstrating speakhuman with Rust acceleration.

Run after installing:
    maturin develop
    python example.py
"""

from __future__ import annotations

import datetime as dt

import speakhuman

# Check if Rust extension is available
try:
    from speakhuman._speakhuman_rs import ordinal as _rs_ordinal

    print("Rust extension: LOADED\n")
except ImportError:
    print("Rust extension: NOT AVAILABLE (using pure Python fallback)\n")

# --- Numbers ---
print("=== Numbers ===")
print(f"  ordinal(1)        = {speakhuman.ordinal(1)!r}")
print(f"  ordinal(103)      = {speakhuman.ordinal(103)!r}")
print(f"  intcomma(1000000) = {speakhuman.intcomma(1_000_000)!r}")
print(f"  intword(1.2e9)    = {speakhuman.intword(1_200_000_000)!r}")
print(f"  apnumber(5)       = {speakhuman.apnumber(5)!r}")
print(f"  scientific(1000)  = {speakhuman.scientific(1000)!r}")
print(f"  fractional(0.3)   = {speakhuman.fractional(0.3)!r}")
print(f"  metric(1500, 'V') = {speakhuman.metric(1500, 'V')!r}")

# --- File sizes ---
print("\n=== File Sizes ===")
print(f"  naturalsize(3e6)             = {speakhuman.naturalsize(3_000_000)!r}")
print(f"  naturalsize(3000, gnu=True)  = {speakhuman.naturalsize(3000, gnu=True)!r}")
print(f"  naturalsize(3000, binary=True) = {speakhuman.naturalsize(3000, binary=True)!r}")

# --- Lists ---
print("\n=== Lists ===")
print(f"  natural_list(['a','b','c']) = {speakhuman.natural_list(['a', 'b', 'c'])!r}")
print(f"  natural_list(['a','b'])     = {speakhuman.natural_list(['a', 'b'])!r}")

# --- Time ---
print("\n=== Time ===")
print(f"  naturaldelta(timedelta(hours=3))    = {speakhuman.naturaldelta(dt.timedelta(hours=3))!r}")
print(f"  naturaldelta(timedelta(days=500))   = {speakhuman.naturaldelta(dt.timedelta(days=500))!r}")
print(f"  naturalday(date.today())            = {speakhuman.naturalday(dt.date.today())!r}")
print(f"  naturaltime(timedelta(seconds=45))  = {speakhuman.naturaltime(dt.timedelta(seconds=45))!r}")
delta = dt.timedelta(seconds=3633, days=2, microseconds=123000)
print(f"  precisedelta({delta!s}) = {speakhuman.precisedelta(delta)!r}")
