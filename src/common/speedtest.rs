//! Speed Test Utilities
//!
//! A comprehensive benchmarking and timing utility for unit tests.
//! AI agents should use these utilities when writing performance-sensitive tests.
//!
//! # Usage
//!
//! ```rust
//! use polymarket_websocket::common::speedtest::{SpeedTest, SpeedTestResult};
//!
//! // Simple timing
//! let result = SpeedTest::time("my_operation", || {
//!     // code to measure
//! });
//! println!("Took: {:?}", result.duration);
//!
//! // With assertions
//! SpeedTest::assert_faster_than_ms("fast_operation", 100, || {
//!     // should complete in under 100ms
//! });
//!
//! // Async timing
//! let result = SpeedTest::time_async("async_op", async {
//!     // async code
//! }).await;
//!
//! // Multiple iterations for averaging
//! let stats = SpeedTest::benchmark("operation", 100, || {
//!     // run 100 times
//! });
//! println!("Average: {:?}", stats.average);
//! ```

use std::fmt;
use std::future::Future;
use std::time::{Duration, Instant};

/// Result of a single speed test measurement
#[derive(Debug, Clone)]
pub struct SpeedTestResult<T> {
    /// The name/label of the test
    pub name: String,
    /// How long the operation took
    pub duration: Duration,
    /// The result of the operation
    pub result: T,
    /// Timestamp when the test started
    pub started_at: Instant,
}

impl<T> SpeedTestResult<T> {
    /// Get duration in milliseconds
    pub fn millis(&self) -> u128 {
        self.duration.as_millis()
    }

    /// Get duration in microseconds
    pub fn micros(&self) -> u128 {
        self.duration.as_micros()
    }

    /// Get duration in nanoseconds
    pub fn nanos(&self) -> u128 {
        self.duration.as_nanos()
    }

    /// Check if the operation completed within the given duration
    pub fn is_faster_than(&self, max_duration: Duration) -> bool {
        self.duration < max_duration
    }

    /// Check if the operation completed within the given milliseconds
    pub fn is_faster_than_ms(&self, max_ms: u64) -> bool {
        self.is_faster_than(Duration::from_millis(max_ms))
    }
}

impl<T: fmt::Debug> fmt::Display for SpeedTestResult<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[SpeedTest] {} completed in {:?} ({} ms)",
            self.name,
            self.duration,
            self.millis()
        )
    }
}

/// Statistics from running a benchmark multiple times
#[derive(Debug, Clone)]
pub struct BenchmarkStats {
    /// Name of the benchmark
    pub name: String,
    /// Number of iterations run
    pub iterations: usize,
    /// Total time for all iterations
    pub total: Duration,
    /// Average time per iteration
    pub average: Duration,
    /// Minimum time observed
    pub min: Duration,
    /// Maximum time observed
    pub max: Duration,
    /// Median time
    pub median: Duration,
    /// 95th percentile
    pub p95: Duration,
    /// 99th percentile
    pub p99: Duration,
    /// Standard deviation (in nanoseconds)
    pub std_dev_nanos: f64,
}

impl BenchmarkStats {
    /// Get operations per second based on average duration
    pub fn ops_per_second(&self) -> f64 {
        if self.average.as_nanos() == 0 {
            return f64::INFINITY;
        }
        1_000_000_000.0 / self.average.as_nanos() as f64
    }

    /// Check if the average time is within the threshold
    pub fn average_is_faster_than(&self, max_duration: Duration) -> bool {
        self.average < max_duration
    }

    /// Check if p95 is within the threshold
    pub fn p95_is_faster_than(&self, max_duration: Duration) -> bool {
        self.p95 < max_duration
    }
}

impl fmt::Display for BenchmarkStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "[Benchmark] {}", self.name)?;
        writeln!(f, "  Iterations: {}", self.iterations)?;
        writeln!(f, "  Total:      {:?}", self.total)?;
        writeln!(f, "  Average:    {:?}", self.average)?;
        writeln!(f, "  Min:        {:?}", self.min)?;
        writeln!(f, "  Max:        {:?}", self.max)?;
        writeln!(f, "  Median:     {:?}", self.median)?;
        writeln!(f, "  P95:        {:?}", self.p95)?;
        writeln!(f, "  P99:        {:?}", self.p99)?;
        writeln!(f, "  Std Dev:    {:.2} µs", self.std_dev_nanos / 1000.0)?;
        writeln!(f, "  Ops/sec:    {:.2}", self.ops_per_second())
    }
}

/// Main speed test utility class
///
/// Provides static methods for timing operations, running benchmarks,
/// and making performance assertions in tests.
pub struct SpeedTest;

impl SpeedTest {
    /// Time a synchronous operation
    ///
    /// # Example
    /// ```
    /// use polymarket_websocket::common::speedtest::SpeedTest;
    ///
    /// let result = SpeedTest::time("vector_allocation", || {
    ///     let v: Vec<i32> = (0..1000).collect();
    ///     v.len()
    /// });
    /// println!("{}", result);
    /// ```
    pub fn time<T, F>(name: &str, f: F) -> SpeedTestResult<T>
    where
        F: FnOnce() -> T,
    {
        let started_at = Instant::now();
        let result = f();
        let duration = started_at.elapsed();

        SpeedTestResult {
            name: name.to_string(),
            duration,
            result,
            started_at,
        }
    }

    /// Time an async operation
    ///
    /// # Example
    /// ```ignore
    /// let result = SpeedTest::time_async("http_request", async {
    ///     client.get("https://example.com").await
    /// }).await;
    /// ```
    pub async fn time_async<T, F, Fut>(name: &str, f: F) -> SpeedTestResult<T>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
    {
        let started_at = Instant::now();
        let result = f().await;
        let duration = started_at.elapsed();

        SpeedTestResult {
            name: name.to_string(),
            duration,
            result,
            started_at,
        }
    }

    /// Run a benchmark with multiple iterations
    ///
    /// # Example
    /// ```
    /// use polymarket_websocket::common::speedtest::SpeedTest;
    ///
    /// let stats = SpeedTest::benchmark("json_parse", 1000, || {
    ///     let _: serde_json::Value = serde_json::from_str("{}").unwrap();
    /// });
    /// println!("{}", stats);
    /// ```
    pub fn benchmark<F>(name: &str, iterations: usize, mut f: F) -> BenchmarkStats
    where
        F: FnMut(),
    {
        assert!(iterations > 0, "Iterations must be greater than 0");

        // Warmup run
        f();

        // Collect timings
        let mut durations: Vec<Duration> = Vec::with_capacity(iterations);
        let total_start = Instant::now();

        for _ in 0..iterations {
            let start = Instant::now();
            f();
            durations.push(start.elapsed());
        }

        let total = total_start.elapsed();

        Self::calculate_stats(name, durations, total)
    }

    /// Run an async benchmark with multiple iterations
    pub async fn benchmark_async<F, Fut>(name: &str, iterations: usize, mut f: F) -> BenchmarkStats
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = ()>,
    {
        assert!(iterations > 0, "Iterations must be greater than 0");

        // Warmup run
        f().await;

        // Collect timings
        let mut durations: Vec<Duration> = Vec::with_capacity(iterations);
        let total_start = Instant::now();

        for _ in 0..iterations {
            let start = Instant::now();
            f().await;
            durations.push(start.elapsed());
        }

        let total = total_start.elapsed();

        Self::calculate_stats(name, durations, total)
    }

    /// Calculate statistics from collected durations
    fn calculate_stats(name: &str, mut durations: Vec<Duration>, total: Duration) -> BenchmarkStats {
        let iterations = durations.len();

        // Sort for percentile calculations
        durations.sort();

        let min = durations[0];
        let max = durations[iterations - 1];
        let median = durations[iterations / 2];
        let p95 = durations[(iterations as f64 * 0.95) as usize];
        let p99 = durations[(iterations as f64 * 0.99).min(iterations as f64 - 1.0) as usize];

        let total_nanos: u128 = durations.iter().map(|d| d.as_nanos()).sum();
        let average = Duration::from_nanos((total_nanos / iterations as u128) as u64);

        // Calculate standard deviation
        let avg_nanos = average.as_nanos() as f64;
        let variance: f64 = durations
            .iter()
            .map(|d| {
                let diff = d.as_nanos() as f64 - avg_nanos;
                diff * diff
            })
            .sum::<f64>()
            / iterations as f64;
        let std_dev_nanos = variance.sqrt();

        BenchmarkStats {
            name: name.to_string(),
            iterations,
            total,
            average,
            min,
            max,
            median,
            p95,
            p99,
            std_dev_nanos,
        }
    }

    /// Assert that an operation completes within the given duration
    ///
    /// # Panics
    /// Panics if the operation takes longer than `max_duration`
    pub fn assert_faster_than<T, F>(name: &str, max_duration: Duration, f: F) -> SpeedTestResult<T>
    where
        F: FnOnce() -> T,
    {
        let result = Self::time(name, f);
        assert!(
            result.is_faster_than(max_duration),
            "[SpeedTest FAILED] {} took {:?}, expected less than {:?}",
            name,
            result.duration,
            max_duration
        );
        result
    }

    /// Assert that an operation completes within the given milliseconds
    ///
    /// # Panics
    /// Panics if the operation takes longer than `max_ms` milliseconds
    pub fn assert_faster_than_ms<T, F>(name: &str, max_ms: u64, f: F) -> SpeedTestResult<T>
    where
        F: FnOnce() -> T,
    {
        Self::assert_faster_than(name, Duration::from_millis(max_ms), f)
    }

    /// Assert that an async operation completes within the given duration
    pub async fn assert_faster_than_async<T, F, Fut>(
        name: &str,
        max_duration: Duration,
        f: F,
    ) -> SpeedTestResult<T>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
    {
        let result = Self::time_async(name, f).await;
        assert!(
            result.is_faster_than(max_duration),
            "[SpeedTest FAILED] {} took {:?}, expected less than {:?}",
            name,
            result.duration,
            max_duration
        );
        result
    }

    /// Assert that an async operation completes within the given milliseconds
    pub async fn assert_faster_than_ms_async<T, F, Fut>(
        name: &str,
        max_ms: u64,
        f: F,
    ) -> SpeedTestResult<T>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
    {
        Self::assert_faster_than_async(name, Duration::from_millis(max_ms), f).await
    }

    /// Run a benchmark and assert the average time is within threshold
    pub fn assert_benchmark_average<F>(
        name: &str,
        iterations: usize,
        max_average: Duration,
        f: F,
    ) -> BenchmarkStats
    where
        F: FnMut(),
    {
        let stats = Self::benchmark(name, iterations, f);
        assert!(
            stats.average_is_faster_than(max_average),
            "[SpeedTest FAILED] {} average {:?} exceeded threshold {:?}\n{}",
            name,
            stats.average,
            max_average,
            stats
        );
        stats
    }

    /// Run a benchmark and assert the p95 time is within threshold
    pub fn assert_benchmark_p95<F>(
        name: &str,
        iterations: usize,
        max_p95: Duration,
        f: F,
    ) -> BenchmarkStats
    where
        F: FnMut(),
    {
        let stats = Self::benchmark(name, iterations, f);
        assert!(
            stats.p95_is_faster_than(max_p95),
            "[SpeedTest FAILED] {} P95 {:?} exceeded threshold {:?}\n{}",
            name,
            stats.p95,
            max_p95,
            stats
        );
        stats
    }

    /// Compare two operations and assert one is faster
    ///
    /// Returns (slower_result, faster_result)
    pub fn assert_faster_than_baseline<T, U, F1, F2>(
        baseline_name: &str,
        baseline_fn: F1,
        optimized_name: &str,
        optimized_fn: F2,
    ) -> (SpeedTestResult<T>, SpeedTestResult<U>)
    where
        F1: FnOnce() -> T,
        F2: FnOnce() -> U,
    {
        let baseline = Self::time(baseline_name, baseline_fn);
        let optimized = Self::time(optimized_name, optimized_fn);

        assert!(
            optimized.duration < baseline.duration,
            "[SpeedTest FAILED] {} ({:?}) should be faster than {} ({:?})",
            optimized_name,
            optimized.duration,
            baseline_name,
            baseline.duration
        );

        (baseline, optimized)
    }

    /// Print a formatted speed test report
    pub fn print_report<T: fmt::Debug>(result: &SpeedTestResult<T>) {
        println!("{}", result);
    }

    /// Print a formatted benchmark report
    pub fn print_benchmark_report(stats: &BenchmarkStats) {
        println!("{}", stats);
    }
}

/// A guard that measures time from creation to drop
/// Useful for measuring scope duration
pub struct SpeedTestGuard {
    name: String,
    start: Instant,
    threshold: Option<Duration>,
}

impl SpeedTestGuard {
    /// Create a new guard that will print timing on drop
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            start: Instant::now(),
            threshold: None,
        }
    }

    /// Create a guard with a threshold that will warn if exceeded
    pub fn with_threshold(name: &str, max_duration: Duration) -> Self {
        Self {
            name: name.to_string(),
            start: Instant::now(),
            threshold: Some(max_duration),
        }
    }

    /// Get elapsed time so far (without dropping)
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

impl Drop for SpeedTestGuard {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        if let Some(threshold) = self.threshold {
            if duration > threshold {
                eprintln!(
                    "[SpeedTest WARNING] {} took {:?}, exceeded threshold {:?}",
                    self.name, duration, threshold
                );
            } else {
                println!(
                    "[SpeedTest] {} completed in {:?} (within {:?} threshold)",
                    self.name, duration, threshold
                );
            }
        } else {
            println!("[SpeedTest] {} completed in {:?}", self.name, duration);
        }
    }
}

/// Macro for quick inline timing
#[macro_export]
macro_rules! time_it {
    ($name:expr, $body:expr) => {{
        let __start = std::time::Instant::now();
        let __result = $body;
        let __duration = __start.elapsed();
        println!("[SpeedTest] {} took {:?}", $name, __duration);
        __result
    }};
}

/// Macro for asserting speed in tests
#[macro_export]
macro_rules! assert_fast {
    ($name:expr, $max_ms:expr, $body:expr) => {{
        let __start = std::time::Instant::now();
        let __result = $body;
        let __duration = __start.elapsed();
        assert!(
            __duration < std::time::Duration::from_millis($max_ms),
            "[SpeedTest FAILED] {} took {:?}, expected less than {}ms",
            $name,
            __duration,
            $max_ms
        );
        __result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_speed_test_time() {
        let result = SpeedTest::time("test_operation", || {
            thread::sleep(Duration::from_millis(10));
            42
        });

        assert_eq!(result.result, 42);
        assert!(result.duration >= Duration::from_millis(10));
        assert!(result.duration < Duration::from_millis(50)); // Allow some overhead
    }

    #[test]
    fn test_speed_test_result_methods() {
        let result = SpeedTest::time("test", || {
            thread::sleep(Duration::from_millis(5));
        });

        assert!(result.millis() >= 5);
        assert!(result.micros() >= 5000);
        assert!(result.is_faster_than(Duration::from_secs(1)));
        assert!(result.is_faster_than_ms(1000));
        assert!(!result.is_faster_than_ms(1));
    }

    #[test]
    fn test_benchmark_stats() {
        let stats = SpeedTest::benchmark("bench_test", 10, || {
            thread::sleep(Duration::from_micros(100));
        });

        assert_eq!(stats.iterations, 10);
        assert!(stats.min <= stats.average);
        assert!(stats.average <= stats.max);
        assert!(stats.median <= stats.max);
        assert!(stats.p95 <= stats.max);
        assert!(stats.ops_per_second() > 0.0);
    }

    #[test]
    fn test_assert_faster_than_ms_passes() {
        let result = SpeedTest::assert_faster_than_ms("fast_op", 1000, || {
            // Fast operation
            let sum: i32 = (0..100).sum();
            sum
        });

        assert_eq!(result.result, 4950);
    }

    #[test]
    #[should_panic(expected = "SpeedTest FAILED")]
    fn test_assert_faster_than_ms_fails() {
        SpeedTest::assert_faster_than_ms("slow_op", 1, || {
            thread::sleep(Duration::from_millis(10));
        });
    }

    #[test]
    fn test_speed_test_guard() {
        {
            let _guard = SpeedTestGuard::new("scoped_operation");
            thread::sleep(Duration::from_millis(5));
        }
        // Guard prints on drop
    }

    #[test]
    fn test_time_it_macro() {
        let result = time_it!("macro_test", {
            let v: Vec<i32> = (0..100).collect();
            v.len()
        });
        assert_eq!(result, 100);
    }

    #[test]
    fn test_assert_fast_macro() {
        let result = assert_fast!("fast_macro_test", 1000, {
            let sum: i32 = (0..100).sum();
            sum
        });
        assert_eq!(result, 4950);
    }

    #[tokio::test]
    async fn test_async_timing() {
        let result = SpeedTest::time_async("async_op", async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            "done"
        })
        .await;

        assert_eq!(result.result, "done");
        assert!(result.duration >= Duration::from_millis(10));
    }

    #[tokio::test]
    async fn test_async_benchmark() {
        let stats = SpeedTest::benchmark_async("async_bench", 5, || async {
            tokio::time::sleep(Duration::from_micros(100)).await;
        })
        .await;

        assert_eq!(stats.iterations, 5);
        println!("{}", stats);
    }

    #[test]
    fn test_display_formatting() {
        let result = SpeedTest::time("display_test", || 42);
        let display = format!("{}", result);
        assert!(display.contains("display_test"));
        assert!(display.contains("ms"));

        let stats = SpeedTest::benchmark("bench_display", 10, || {});
        let display = format!("{}", stats);
        assert!(display.contains("bench_display"));
        assert!(display.contains("Iterations: 10"));
    }
}
