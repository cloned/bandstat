mod analysis;
mod audio;
mod chart;
mod mode;
mod output;

use clap::Parser;

use mode::{run_compare, run_stats, run_timeline};
use output::print_error;

#[derive(Parser)]
#[command(
    name = "bandstat",
    version,
    about = "Audio frequency band analyzer with K-weighting and dynamics analysis",
    after_help = "Examples:
  bandstat audio.wav                                   Single file analysis
  bandstat audio.wav --image chart.png                 Single file with chart output
  bandstat audio.wav --image chart.png -w              Chart with K-weighting
  bandstat my_mix.wav ref.wav                          Compare files (first is base)
  bandstat a.wav b.wav --image chart.png               Comparison chart output
  bandstat --time audio.wav                            Timeline analysis
  bandstat --time --image chart.png audio.wav          Timeline chart output
  bandstat --time -i 10 -w audio.wav                   10s intervals, K-weighted"
)]
struct Args {
    /// Audio files to analyze (WAV, AIFF, MP3, FLAC). Up to 10 files for comparison.
    #[arg(required = true)]
    files: Vec<String>,

    /// Timeline analysis mode (band distribution over time)
    #[arg(short, long)]
    time: bool,

    /// Timeline interval in seconds (minimum: 1)
    #[arg(short, long, default_value = "20", value_name = "SECONDS")]
    interval: u32,

    /// Use K-weighted values for analysis/chart output
    #[arg(short, long)]
    weighted: bool,

    /// Suppress explanations (show data only)
    #[arg(short, long)]
    quiet: bool,

    /// Disable colored output
    #[arg(long)]
    no_color: bool,

    /// Output chart as PNG image
    #[arg(long, value_name = "PATH")]
    image: Option<String>,
}

fn main() {
    let args = Args::parse();

    // Handle --no-color
    if args.no_color {
        colored::control::set_override(false);
    }

    // Validate file count
    if args.files.len() > 10 {
        print_error("Too many files specified (max 10)");
        std::process::exit(1);
    }

    // Validate interval
    if args.interval == 0 {
        print_error("Interval must be at least 1 second");
        std::process::exit(1);
    }

    // Validate option combinations
    if args.files.len() >= 2 && args.time {
        print_error("--time cannot be used with multiple files");
        std::process::exit(1);
    }

    if args.files.len() >= 2 && args.weighted {
        print_error("--weighted cannot be used with comparison mode");
        std::process::exit(1);
    }

    if args.weighted && args.image.is_none() && !args.time {
        use colored::*;
        eprintln!(
            "{} --weighted has no effect without --image in single-file mode",
            "Warning:".yellow()
        );
    }

    if !args.time && args.interval != 20 {
        print_error("--interval can only be used with --time");
        std::process::exit(1);
    }

    if args.image.is_some() && args.files.len() >= 2 && args.files.len() > chart::max_chart_files()
    {
        print_error(&format!(
            "--image supports up to {} files",
            chart::max_chart_files()
        ));
        std::process::exit(1);
    }

    // Validate image output path
    if let Some(ref path) = args.image {
        use std::path::Path;
        if let Some(parent) = Path::new(path).parent()
            && !parent.as_os_str().is_empty()
            && !parent.exists()
        {
            print_error(&format!("Directory does not exist: {}", parent.display()));
            std::process::exit(1);
        }
    }

    // Dispatch to appropriate mode
    if args.files.len() >= 2 {
        run_compare(&args.files, args.quiet, args.image.as_deref());
    } else if args.time {
        run_timeline(
            &args.files[0],
            args.weighted,
            args.interval,
            args.quiet,
            args.image.as_deref(),
        );
    } else {
        run_stats(
            &args.files[0],
            args.weighted,
            args.quiet,
            args.image.as_deref(),
        );
    }
}
