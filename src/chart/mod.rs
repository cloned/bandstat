//! Chart rendering for band balance visualization

mod colors;
mod comparison;
mod stacked;

pub use comparison::render_comparison_chart;
pub use stacked::render_stacked_chart;

use crate::analysis::Band;

/// Data for a single file in the comparison chart
pub struct FileChartData {
    pub label: char,
    pub name: String,
    pub raw_pct: Vec<f64>,
    pub k_pct: Vec<f64>,
}

/// Data for timeline/stacked chart
pub struct TimelineChartData {
    pub filename: String,
    pub time_labels: Vec<String>,
    /// Band percentages: band_percentages[band_idx][time_idx]
    pub band_percentages: Vec<Vec<f64>>,
}

/// Chart dimensions (2x for Retina quality)
pub(super) const CHART_WIDTH: u32 = 2800;
pub(super) const CHART_HEIGHT: u32 = 1200;

/// Maximum number of files supported for chart rendering
pub fn max_chart_files() -> usize {
    colors::COLOR_SETS.len()
}

/// Format frequency for display (e.g., 1000 -> "1k", 500 -> "500")
pub(super) fn format_freq(hz: f32) -> String {
    if hz >= 1000.0 {
        let k = hz / 1000.0;
        if k == k.floor() {
            format!("{}k", k as u32)
        } else {
            format!("{:.1}k", k)
        }
    } else {
        format!("{}", hz as u32)
    }
}

/// Build band label with frequency range (2 lines, for X-axis)
pub(super) fn build_band_label(band: &Band) -> String {
    let freq_range = if band.high_hz == f32::MAX {
        format!("{}+", format_freq(band.low_hz))
    } else {
        format!("{}-{}", format_freq(band.low_hz), format_freq(band.high_hz))
    };
    format!("{}\n{}", band.label, freq_range)
}

/// Build band label with frequency range (1 line, for legend)
pub(super) fn build_band_legend_label(band: &Band) -> String {
    let freq_range = if band.high_hz == f32::MAX {
        format!("{}+", format_freq(band.low_hz))
    } else {
        format!("{}-{}", format_freq(band.low_hz), format_freq(band.high_hz))
    };
    format!("{} ({})", band.label, freq_range)
}
