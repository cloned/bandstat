mod analysis;
mod audio;
mod chart;
mod output;

use clap::Parser;
use rustfft::FftPlanner;

use analysis::{
    DYNAMICS_DISPLAY_THRESHOLD_PCT, FFT_SIZE, analyze_interval, analyze_stats,
    create_hanning_window, create_k_weight_table, get_bands, powers_to_percentages,
};
use audio::{TARGET_SAMPLE_RATE, load_audio};
use output::{
    format_time, get_display_name, print_bands, print_diff_row, print_diff_row_masked_styled,
    print_diff_row_styled, print_error, print_file_info, print_header, print_legend,
    print_percentages, print_row, print_row_masked, print_row_masked_styled, print_row_styled,
    print_separator,
};

#[derive(Parser)]
#[command(
    name = "bandstat",
    version,
    about = "Audio frequency band analyzer with K-weighting and dynamics analysis",
    after_help = "Examples:
  bandstat audio.wav                                   Single file analysis
  bandstat my_mix.wav ref.wav                          Compare files (first is base)
  bandstat a.wav b.wav --image chart.png               Output comparison chart
  bandstat --time audio.wav                            Timeline analysis
  bandstat --time --interval 10 --weighted audio.wav   10s intervals, K-weighted
  bandstat --no-color audio.wav                        Disable colored output"
)]
struct Args {
    /// Audio files to analyze (WAV, AIFF, MP3, FLAC). Up to 10 files for comparison.
    #[arg(required = true)]
    files: Vec<String>,

    /// Timeline analysis mode (band distribution over time)
    #[arg(short, long)]
    time: bool,

    /// Timeline interval in seconds
    #[arg(short, long, default_value = "20", value_name = "SECONDS")]
    interval: f32,

    /// Apply K-weighting to timeline analysis
    #[arg(short, long)]
    weighted: bool,

    /// Suppress explanations (show data only)
    #[arg(short, long)]
    quiet: bool,

    /// Disable colored output
    #[arg(long)]
    no_color: bool,

    /// Output comparison chart as PNG image (comparison mode only)
    #[arg(long, value_name = "PATH")]
    image: Option<String>,
}

// Stats analysis result for a single file
struct FileStats {
    name: String,
    original_sample_rate: u32,
    channels: u16,
    raw_pct: Vec<f64>,
    k_pct: Vec<f64>,
    dynamics: Vec<f64>,
}

fn analyze_file(filename: &str, bands: &[analysis::Band], show_progress: bool) -> FileStats {
    let display_name = get_display_name(filename).to_string();

    let audio = load_audio(filename).unwrap_or_else(|e| {
        print_error(&e.to_string());
        std::process::exit(1);
    });

    if show_progress {
        eprint!("Analyzing {}... 0%", display_name);
    }

    let k_weights = create_k_weight_table(FFT_SIZE, TARGET_SAMPLE_RATE);
    let result = analyze_stats(&audio, bands, &k_weights, |progress| {
        if show_progress {
            eprint!("\rAnalyzing {}... {}%", display_name, progress);
        }
    });

    if show_progress {
        eprintln!("\rAnalyzing {}... done", display_name);
    }

    FileStats {
        name: display_name,
        original_sample_rate: audio.original_sample_rate,
        channels: audio.channels,
        raw_pct: powers_to_percentages(&result.raw_powers),
        k_pct: powers_to_percentages(&result.k_powers),
        dynamics: result.dynamics,
    }
}

// Mode: Single file stats analysis
fn run_stats(filename: &str, quiet: bool) {
    let bands = get_bands();
    let stats = analyze_file(filename, &bands, !quiet);

    if !quiet {
        println!();
        println!("Stats Analysis");
        print_file_info(
            &stats.name,
            stats.original_sample_rate,
            stats.channels,
            false,
        );
        print_bands(&bands);
    }

    println!("[Band Power Distribution]");
    print_header(&bands, "        ");
    print_separator(&bands, 8);
    print_row("Raw(%)  ", &stats.raw_pct);
    print_row("K-wt(%) ", &stats.k_pct);
    print_separator(&bands, 8);
    print_diff_row("Diff    ", &stats.raw_pct, &stats.k_pct);

    println!();
    println!("[Dynamics]");
    print_header(&bands, "        ");
    print_separator(&bands, 8);
    print_row_masked(
        "Dyn(dB) ",
        &stats.dynamics,
        &stats.raw_pct,
        DYNAMICS_DISPLAY_THRESHOLD_PCT,
    );

    if !quiet {
        println!();
        print_legend();
    }
}

// Mode: Compare multiple files
fn run_compare(filenames: &[String], quiet: bool, image_path: Option<&str>) {
    use colored::*;

    let bands = get_bands();
    let labels: Vec<char> = ('A'..='Z').collect();

    let stats: Vec<FileStats> = filenames
        .iter()
        .map(|f| analyze_file(f, &bands, !quiet))
        .collect();

    println!("Comparison (base: [A]):");
    for (i, s) in stats.iter().enumerate() {
        let label = format!("[{}]", labels[i]);
        println!("  {} {}", label.bold(), s.name);
    }
    println!();

    if !quiet {
        print_bands(&bands);
    }

    println!("[Band Power Distribution]");
    print_header(&bands, "        ");
    print_separator(&bands, 8);

    let ref_label = format!("[{}]", labels[0]);
    print_row_styled(&ref_label, " Raw  ", &stats[0].raw_pct);
    print_row_styled(&ref_label, " K-wt ", &stats[0].k_pct);
    print_diff_row_styled(&ref_label, " Diff ", &stats[0].raw_pct, &stats[0].k_pct);

    for (i, s) in stats.iter().enumerate().skip(1) {
        print_separator(&bands, 8);
        let label = format!("[{}]", labels[i]);
        print_row_styled(&label, " Raw  ", &s.raw_pct);
        print_row_styled(&label, " K-wt ", &s.k_pct);
        print_diff_row_styled(&label, " Diff ", &s.raw_pct, &s.k_pct);
        print_separator(&bands, 8);
        let diff_label = format!("{}-A", labels[i]);
        print_diff_row_styled(&diff_label, " Raw  ", &stats[0].raw_pct, &s.raw_pct);
        print_diff_row_styled(&diff_label, " K-wt ", &stats[0].k_pct, &s.k_pct);
    }

    println!();
    println!("[Dynamics]");
    print_header(&bands, "        ");
    print_separator(&bands, 8);

    print_row_masked_styled(
        &format!("[{}]", labels[0]),
        " dB   ",
        &stats[0].dynamics,
        &stats[0].raw_pct,
        DYNAMICS_DISPLAY_THRESHOLD_PCT,
    );

    for (i, s) in stats.iter().enumerate().skip(1) {
        print_separator(&bands, 8);
        print_row_masked_styled(
            &format!("[{}]", labels[i]),
            " dB   ",
            &s.dynamics,
            &s.raw_pct,
            DYNAMICS_DISPLAY_THRESHOLD_PCT,
        );
        print_separator(&bands, 8);
        print_diff_row_masked_styled(
            &format!("{}-A", labels[i]),
            "      ",
            &stats[0].dynamics,
            &s.dynamics,
            &stats[0].raw_pct,
            &s.raw_pct,
            DYNAMICS_DISPLAY_THRESHOLD_PCT,
        );
    }

    if !quiet {
        println!();
        print_legend();
    }

    // Output chart image if requested
    if let Some(path) = image_path {
        let file_data: Vec<chart::FileChartData> = stats
            .iter()
            .enumerate()
            .map(|(i, s)| chart::FileChartData {
                label: labels[i],
                name: s.name.clone(),
                raw_pct: s.raw_pct.clone(),
                k_pct: s.k_pct.clone(),
            })
            .collect();

        if let Err(e) = chart::render_comparison_chart(&file_data, &bands, path) {
            print_error(&e);
        } else {
            eprintln!("Chart saved to: {}", path);
        }
    }
}

// Mode: Timeline analysis (band distribution over time)
fn run_timeline(filename: &str, use_k_weighting: bool, interval_secs: f32, quiet: bool) {
    let bands = get_bands();

    let audio = load_audio(filename).unwrap_or_else(|e| {
        print_error(&e.to_string());
        std::process::exit(1);
    });

    if !quiet {
        let display_name = get_display_name(filename);
        print_file_info(
            display_name,
            audio.original_sample_rate,
            audio.channels,
            use_k_weighting,
        );
        print_bands(&bands);
    }

    if audio.samples.is_empty() {
        print_error("No samples found in file");
        std::process::exit(1);
    }

    let freq_per_bin = TARGET_SAMPLE_RATE as f32 / FFT_SIZE as f32;
    let window = create_hanning_window(FFT_SIZE);
    let k_weights = if use_k_weighting {
        Some(create_k_weight_table(FFT_SIZE, TARGET_SAMPLE_RATE))
    } else {
        None
    };

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(FFT_SIZE);

    let samples_per_interval = (interval_secs * TARGET_SAMPLE_RATE as f32) as usize;
    let total_duration = audio.samples.len() as f32 / TARGET_SAMPLE_RATE as f32;
    let num_intervals = audio.samples.len().div_ceil(samples_per_interval);

    if num_intervals == 0 {
        print_error("File too short for analysis");
        std::process::exit(1);
    }

    print_header(&bands, "TIME  ");
    print_separator(&bands, 6);

    let mut total_band_powers = vec![0.0f64; bands.len()];

    for interval_idx in 0..num_intervals {
        let interval_start = interval_idx * samples_per_interval;
        let interval_end = ((interval_idx + 1) * samples_per_interval).min(audio.samples.len());

        if interval_end <= interval_start {
            break;
        }

        let interval_samples = &audio.samples[interval_start..interval_end];
        let band_powers = analyze_interval(
            interval_samples,
            &fft,
            &window,
            &bands,
            freq_per_bin,
            k_weights.as_deref(),
        );

        if band_powers.iter().all(|&p| p == 0.0) {
            continue;
        }

        for (total, power) in total_band_powers.iter_mut().zip(&band_powers) {
            *total += power;
        }

        let time_secs = interval_start as f32 / TARGET_SAMPLE_RATE as f32;
        print!("{}", format_time(time_secs));
        print_percentages(&band_powers, &bands);
        println!();
    }

    print_separator(&bands, 6);

    print!("AVG   ");
    print_percentages(&total_band_powers, &bands);
    println!();

    println!();
    println!("Duration: {}", format_time(total_duration));
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
    if args.interval <= 0.0 {
        print_error("Interval must be positive");
        std::process::exit(1);
    }

    // Validate option combinations
    if args.files.len() >= 2 && args.time {
        print_error("--time cannot be used with multiple files");
        std::process::exit(1);
    }

    if !args.time && args.weighted {
        print_error("--weighted can only be used with --time");
        std::process::exit(1);
    }

    if !args.time && args.interval != 20.0 {
        print_error("--interval can only be used with --time");
        std::process::exit(1);
    }

    if args.image.is_some() && args.files.len() < 2 {
        print_error("--image can only be used with comparison mode (2+ files)");
        std::process::exit(1);
    }

    if args.image.is_some() && args.files.len() > chart::max_chart_files() {
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
        run_timeline(&args.files[0], args.weighted, args.interval, args.quiet);
    } else {
        run_stats(&args.files[0], args.quiet);
    }
}
