//! CLI mode implementations

mod compare;
mod stats;
mod timeline;

pub use compare::run_compare;
pub use stats::run_stats;
pub use timeline::run_timeline;

use crate::analysis::{self, FFT_SIZE, powers_to_percentages};
use crate::audio::{TARGET_SAMPLE_RATE, load_audio};
use crate::output::get_display_name;

/// Stats analysis result for a single file
pub struct FileStats {
    pub name: String,
    pub original_sample_rate: u32,
    pub channels: u16,
    pub raw_pct: Vec<f64>,
    pub k_pct: Vec<f64>,
    pub dynamics: Vec<f64>,
}

/// Analyze a single audio file and return its statistics
pub fn analyze_file(
    filename: &str,
    bands: &[analysis::Band],
    show_progress: bool,
) -> Result<FileStats, String> {
    let display_name = get_display_name(filename).to_string();

    let audio = load_audio(filename)?;

    if show_progress {
        eprint!("Analyzing {}... 0%", display_name);
    }

    let k_weights = analysis::create_k_weight_table(FFT_SIZE, TARGET_SAMPLE_RATE);
    let result = analysis::analyze_stats(&audio, bands, &k_weights, |progress| {
        if show_progress {
            eprint!("\rAnalyzing {}... {}%", display_name, progress);
        }
    });

    if show_progress {
        eprintln!("\rAnalyzing {}... done", display_name);
    }

    Ok(FileStats {
        name: display_name,
        original_sample_rate: audio.original_sample_rate,
        channels: audio.channels,
        raw_pct: powers_to_percentages(&result.raw_powers),
        k_pct: powers_to_percentages(&result.k_powers),
        dynamics: result.dynamics,
    })
}
