use std::time::Duration;

use anyhow::{Result, anyhow, bail};

pub fn parse_duration(input: &str) -> Result<Duration> {
    let input = input.trim();
    let maybe_duration = if input.ends_with("ns") {
        input
            .strip_suffix("ns")
            .and_then(|s| s.parse().ok())
            .map(Duration::from_nanos)
    } else if input.ends_with("us") {
        input
            .strip_suffix("us")
            .and_then(|s| s.parse().ok())
            .map(Duration::from_micros)
    } else if input.ends_with("mcs") {
        input
            .strip_suffix("mcs")
            .and_then(|s| s.parse().ok())
            .map(Duration::from_micros)
    } else if input.ends_with("ms") {
        input
            .strip_suffix("ms")
            .and_then(|s| s.parse().ok())
            .map(Duration::from_millis)
    } else if input.ends_with("s") {
        input
            .strip_suffix("s")
            .and_then(|s| s.parse().ok())
            .map(Duration::from_secs)
    } else if input.ends_with("min") {
        input
            .strip_suffix("min")
            .and_then(|s| s.parse().ok())
            .map(|mins: u64| Duration::from_secs(mins * 60))
    } else {
        bail!("unknown unit")
    };
    maybe_duration.ok_or_else(|| anyhow!("invalid duration scalar"))
}
