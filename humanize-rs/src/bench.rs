//! Benchmark binary for humanize-rs.
//!
//! Runs the same workloads as the Python benchmark and outputs JSON with timings.

use std::time::Instant;

use humanize::filesize::naturalsize;
use humanize::lists::natural_list;
use humanize::number::{apnumber, fractional, intcomma, intword, metric, ordinal, scientific};
use humanize::time::{naturaldelta_td, precisedelta_td, TimeDelta};

const ITERATIONS: u64 = 100_000;

fn bench<F: Fn()>(f: F) -> f64 {
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        f();
    }
    start.elapsed().as_secs_f64()
}

fn main() {
    // Warm up
    let _ = naturalsize(3_000_000.0, false, false, "%.1f");

    let mut results = Vec::new();

    // --- naturalsize ---
    let t = bench(|| {
        let _ = naturalsize(3_000_000.0, false, false, "%.1f");
        let _ = naturalsize(1024.0 * 31.0, true, false, "%.1f");
        let _ = naturalsize(3000.0, false, true, "%.1f");
    });
    results.push(("naturalsize", t));

    // --- intcomma ---
    let t = bench(|| {
        let _ = intcomma("1000000", None);
        let _ = intcomma("1234567.25", None);
        let _ = intcomma("10311", None);
    });
    results.push(("intcomma", t));

    // --- intword ---
    let t = bench(|| {
        let _ = intword("1000000", "%.1f");
        let _ = intword("1200000000", "%.1f");
        let _ = intword("8100000000000000000000000000000000", "%.1f");
    });
    results.push(("intword", t));

    // --- ordinal ---
    let t = bench(|| {
        let _ = ordinal("1");
        let _ = ordinal("103");
        let _ = ordinal("111");
    });
    results.push(("ordinal", t));

    // --- scientific ---
    let t = bench(|| {
        let _ = scientific("1000", 2);
        let _ = scientific("0.3", 2);
        let _ = scientific("5781651000", 2);
    });
    results.push(("scientific", t));

    // --- fractional ---
    let t = bench(|| {
        let _ = fractional("0.3");
        let _ = fractional("1.3");
        let _ = fractional("0.3333333333333333");
    });
    results.push(("fractional", t));

    // --- metric ---
    let t = bench(|| {
        let _ = metric(1500.0, "V", 3);
        let _ = metric(2e8, "W", 3);
        let _ = metric(220e-6, "F", 3);
    });
    results.push(("metric", t));

    // --- apnumber ---
    let t = bench(|| {
        let _ = apnumber("0");
        let _ = apnumber("5");
        let _ = apnumber("10");
    });
    results.push(("apnumber", t));

    // --- naturaldelta ---
    let t = bench(|| {
        let d1 = TimeDelta::from_days_seconds_micros(7, 0, 0);
        let d2 = TimeDelta::from_seconds(30.0);
        let d3 = TimeDelta::from_days_seconds_micros(500, 0, 0);
        let _ = naturaldelta_td(d1, true, "seconds");
        let _ = naturaldelta_td(d2, true, "seconds");
        let _ = naturaldelta_td(d3, true, "seconds");
    });
    results.push(("naturaldelta", t));

    // --- natural_list ---
    let t = bench(|| {
        let _ = natural_list(&["one", "two", "three"]);
        let _ = natural_list(&["one", "two"]);
        let _ = natural_list(&["one"]);
    });
    results.push(("natural_list", t));

    // --- precisedelta ---
    let t = bench(|| {
        let d1 = TimeDelta::from_days_seconds_micros(2, 3633, 123000);
        let d2 = TimeDelta::from_seconds(1.0);
        let d3 = TimeDelta::from_days_seconds_micros(370, 4 * 3600 + 3, 0);
        let _ = precisedelta_td(d1, "seconds", &[], "%0.2f");
        let _ = precisedelta_td(d2, "seconds", &[], "%0.2f");
        let _ = precisedelta_td(d3, "seconds", &[], "%0.2f");
    });
    results.push(("precisedelta", t));

    // Output JSON
    print!("{{");
    for (i, (name, time)) in results.iter().enumerate() {
        if i > 0 {
            print!(",");
        }
        print!(" \"{}\": {:.6}", name, time);
    }
    println!(" }}");
}
