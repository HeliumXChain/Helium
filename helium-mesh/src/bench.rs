//! helium bench — sustained provider score, never the marketing peak.
//!
//! Measures what a job will actually get: multi-thread FP32 FMA throughput
//! sampled in windows (peak = best early window, sustained = mean of the
//! late windows), one-shot memory copy bandwidth, and available RAM.
//! Thermal throttling shows up as derate_pct — that is the point: a phone
//! advertising 80 TOPS but sustaining 40% of it scores 40%.
//!
//! Scores are RELATIVE: compare runs of the same binary on the same CPU
//! class. Always bench with a --release build (dev profile is ~10x slower
//! and meaningless). Cross-machine absolute calibration (reference table
//! per CPU class) is future work, not needed for match filtering.

use serde::Serialize;
use std::time::{Duration, Instant};

/// FP32 multiply-adds per loop iteration (2 FLOP each).
const FMAS_PER_ITER: u64 = 2;
/// Elements per worker buffer (fits L2, streams through it).
const BUF_LEN: usize = 4096;
/// Target wall time of one sampling window.
const WINDOW: Duration = Duration::from_millis(200);

#[derive(Debug, Clone, Serialize)]
pub struct BenchReport {
    /// Requested wall time (seconds).
    pub seconds: u64,
    /// Best early-window throughput (GFLOPS, all threads).
    pub gflops_peak: f64,
    /// Mean late-window throughput (GFLOPS, all threads).
    pub gflops_sustained: f64,
    /// 100 * (1 - sustained/peak), clamped >= 0. Throttling indicator.
    pub derate_pct: f64,
    /// Single memory copy bandwidth (GB/s).
    pub mem_bw_gbs: f64,
    pub ram_total_gb: f64,
    pub ram_avail_gb: f64,
    /// Provisional class from sustained numbers ("cpu" or "cpu-le").
    pub suggested_class: String,
}

/// Provisional floor for a full "cpu" class: 20 sustained GFLOPS + 8 GB free.
/// Below either -> "cpu-le" (phones, old laptops, throttled boxes).
pub fn suggest_class(gflops_sustained: f64, ram_avail_gb: f64) -> &'static str {
    if gflops_sustained >= 20.0 && ram_avail_gb >= 8.0 {
        "cpu"
    } else {
        "cpu-le"
    }
}

fn worker_fma(windows: &mut Vec<f64>, deadline: Instant) {
    let a: Vec<f32> = (0..BUF_LEN).map(|i| (i as f32) * 0.0001 + 1.0).collect();
    let b: Vec<f32> = (0..BUF_LEN).map(|i| (i as f32) * -0.0002 + 2.0).collect();
    // Eight independent accumulators: a single chain would serialize on FMA
    // latency (~5 cycles) and report one core at a fraction of its power.
    let mut acc = [0.0f32; 8];
    let mut window_ops: u64 = 0;
    let mut window_start = Instant::now();
    while Instant::now() < deadline {
        for i in (0..BUF_LEN).step_by(8) {
            acc[0] = a[i].mul_add(b[i], acc[0]);
            acc[1] = a[i + 1].mul_add(b[i + 1], acc[1]);
            acc[2] = a[i + 2].mul_add(b[i + 2], acc[2]);
            acc[3] = a[i + 3].mul_add(b[i + 3], acc[3]);
            acc[4] = a[i + 4].mul_add(b[i + 4], acc[4]);
            acc[5] = a[i + 5].mul_add(b[i + 5], acc[5]);
            acc[6] = a[i + 6].mul_add(b[i + 6], acc[6]);
            acc[7] = a[i + 7].mul_add(b[i + 7], acc[7]);
        }
        // Keep the accumulators live so the loop is not optimized away.
        if acc.iter().any(|x| x.is_infinite()) {
            acc = [0.0; 8];
        }
        window_ops += BUF_LEN as u64 * FMAS_PER_ITER;
        let elapsed = window_start.elapsed();
        if elapsed >= WINDOW {
            windows.push(window_ops as f64 / elapsed.as_secs_f64() / 1e9);
            window_ops = 0;
            window_start = Instant::now();
        }
    }
    if window_start.elapsed() > Duration::from_millis(10) && window_ops > 0 {
        windows.push(window_ops as f64 / window_start.elapsed().as_secs_f64() / 1e9);
    }
    std::hint::black_box(acc);
}

fn copy_bandwidth_gbs() -> f64 {
    const BYTES: usize = 256 * 1024 * 1024;
    let src = vec![0xA5u8; BYTES];
    let mut dst = vec![0u8; BYTES];
    let start = Instant::now();
    dst.copy_from_slice(&src);
    let secs = start.elapsed().as_secs_f64().max(1e-9);
    std::hint::black_box(dst);
    // Count read + write traffic.
    (2 * BYTES) as f64 / secs / 1e9
}

fn ram_gb() -> (f64, f64) {
    let mut sys = sysinfo::System::new();
    sys.refresh_memory();
    (
        sys.total_memory() as f64 / 1e9,
        sys.available_memory() as f64 / 1e9,
    )
}

/// Run the benchmark for (about) `seconds`. Short runs (< 5 s) still work
/// but only long runs (5 min recommended) expose thermal throttling.
pub fn run_bench(seconds: u64) -> BenchReport {
    let seconds = seconds.max(1);
    let deadline = Instant::now() + Duration::from_secs(seconds);
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(64);
    let mut handles = Vec::with_capacity(threads);
    for _ in 0..threads {
        handles.push(std::thread::spawn(move || {
            let mut windows = Vec::new();
            worker_fma(&mut windows, deadline);
            windows
        }));
    }
    let mut agg: Vec<f64> = Vec::new();
    for h in handles {
        let windows = h.join().unwrap_or_default();
        for (i, rate) in windows.into_iter().enumerate() {
            if agg.len() <= i {
                agg.push(0.0);
            }
            agg[i] += rate;
        }
    }
    // Peak = best of the first quarter of windows, sustained = mean of the
    // last quarter. Single-window runs report the same value twice.
    let q = (agg.len() / 4).max(1);
    let peak = agg.iter().take(q).cloned().fold(0.0f64, f64::max);
    let tail = &agg[agg.len().saturating_sub(q)..];
    let sustained = if tail.is_empty() {
        0.0
    } else {
        tail.iter().sum::<f64>() / tail.len() as f64
    };
    let derate_pct = if peak > 0.0 {
        (100.0 * (1.0 - sustained / peak)).max(0.0)
    } else {
        0.0
    };
    let (ram_total_gb, ram_avail_gb) = ram_gb();
    BenchReport {
        seconds,
        gflops_peak: peak,
        gflops_sustained: sustained,
        derate_pct,
        mem_bw_gbs: copy_bandwidth_gbs(),
        ram_total_gb,
        ram_avail_gb,
        suggested_class: suggest_class(sustained, ram_avail_gb).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_bench_reports_sane_numbers() {
        let r = run_bench(1);
        assert_eq!(r.seconds, 1);
        assert!(r.gflops_peak > 0.0, "peak must be positive");
        assert!(r.gflops_sustained > 0.0, "sustained must be positive");
        assert!(r.gflops_sustained <= r.gflops_peak * 1.5, "windows are noisy, not magic");
        assert!((0.0..=100.0).contains(&r.derate_pct));
        assert!(r.mem_bw_gbs > 0.0);
        assert!(r.ram_total_gb > 0.0);
        assert!(r.ram_avail_gb >= 0.0);
        assert!(r.suggested_class == "cpu" || r.suggested_class == "cpu-le");
    }

    #[test]
    fn class_thresholds() {
        assert_eq!(suggest_class(20.0, 8.0), "cpu");
        assert_eq!(suggest_class(100.0, 32.0), "cpu");
        assert_eq!(suggest_class(19.9, 64.0), "cpu-le");
        assert_eq!(suggest_class(200.0, 7.9), "cpu-le");
    }
}
