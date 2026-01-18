mod analysis;
mod audio;
mod output;

use rustfft::FftPlanner;
use std::env;

use analysis::{
    FFT_SIZE, analyze_interval, analyze_stats, create_hanning_window, create_k_weight_table,
    get_bands, powers_to_percentages,
};
use audio::load_audio;
use output::{
    format_time, get_display_name, print_bands, print_diff_row, print_file_info, print_header,
    print_legend, print_percentages, print_row, print_separator,
};

// Stats analysis result for a single file
struct FileStats {
    name: String,
    sample_rate: u32,
    channels: u16,
    raw_pct: Vec<f64>,
    k_pct: Vec<f64>,
    dynamics: Vec<f64>,
}

fn analyze_file(filename: &str, bands: &[analysis::Band], show_progress: bool) -> FileStats {
    let display_name = get_display_name(filename).to_string();
    if show_progress {
        eprint!("Analyzing {}... 0%", display_name);
    }

    let audio = load_audio(filename).unwrap_or_else(|e| {
        if show_progress {
            eprintln!(" error");
        }
        eprintln!("{}", e);
        std::process::exit(1);
    });

    let k_weights = create_k_weight_table(FFT_SIZE, audio.sample_rate);
    let display_name_clone = display_name.clone();
    let result = analyze_stats(&audio, bands, &k_weights, |progress| {
        if show_progress {
            eprint!("\rAnalyzing {}... {}%", display_name_clone, progress);
        }
    });

    if show_progress {
        eprintln!("\rAnalyzing {}... done", display_name);
    }

    FileStats {
        name: display_name,
        sample_rate: audio.sample_rate,
        channels: audio.channels,
        raw_pct: powers_to_percentages(&result.raw_powers),
        k_pct: powers_to_percentages(&result.k_powers),
        dynamics: result.dynamics,
    }
}

// Mode: Single file stats analysis
fn run_stats(filename: &str) {
    let bands = get_bands();
    let stats = analyze_file(filename, &bands, true);

    println!();
    println!("Stats Analysis");
    print_file_info(&stats.name, stats.sample_rate, stats.channels, false);
    print_bands(&bands);

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
    print_row("Std(dB) ", &stats.dynamics);

    println!();
    print_legend();
}

// Mode: Compare multiple files
fn run_compare(filenames: &[String]) {
    let bands = get_bands();
    let labels: Vec<char> = ('A'..='Z').collect();

    let stats: Vec<FileStats> = filenames
        .iter()
        .map(|f| analyze_file(f, &bands, true))
        .collect();

    println!("Comparison (reference: [A]):");
    for (i, s) in stats.iter().enumerate() {
        println!("  [{}] {}", labels[i], s.name);
    }
    println!();

    print_bands(&bands);

    println!("[Band Power Distribution]");
    print_header(&bands, "        ");
    print_separator(&bands, 8);

    let ref_label = format!("[{}]", labels[0]);
    print_row(&format!("{}Raw  ", ref_label), &stats[0].raw_pct);
    print_row(&format!("{}K-wt ", ref_label), &stats[0].k_pct);
    print_diff_row(
        &format!("{}Diff ", ref_label),
        &stats[0].raw_pct,
        &stats[0].k_pct,
    );

    for (i, s) in stats.iter().enumerate().skip(1) {
        print_separator(&bands, 8);
        let label = format!("[{}]", labels[i]);
        print_row(&format!("{}Raw  ", label), &s.raw_pct);
        print_row(&format!("{}K-wt ", label), &s.k_pct);
        print_diff_row(&format!("{}Diff ", label), &s.raw_pct, &s.k_pct);
        print_separator(&bands, 8);
        let diff_label = format!("{}-A", labels[i]);
        print_diff_row(
            &format!("{} Raw ", diff_label),
            &stats[0].raw_pct,
            &s.raw_pct,
        );
        print_diff_row(&format!("{} K-wt", diff_label), &stats[0].k_pct, &s.k_pct);
    }

    println!();
    println!("[Dynamics]");
    print_header(&bands, "        ");
    print_separator(&bands, 8);

    print_row(&format!("[{}]dB   ", labels[0]), &stats[0].dynamics);

    for (i, s) in stats.iter().enumerate().skip(1) {
        print_separator(&bands, 8);
        print_row(&format!("[{}]dB   ", labels[i]), &s.dynamics);
        print_diff_row(
            &format!("{}-A     ", labels[i]),
            &stats[0].dynamics,
            &s.dynamics,
        );
    }

    println!();
    print_legend();
}

// Mode: Timeline analysis (band distribution over time)
fn run_timeline(filename: &str, use_k_weighting: bool, interval_secs: f32) {
    let bands = get_bands();

    let audio = load_audio(filename).unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });

    let display_name = get_display_name(filename);
    print_file_info(
        display_name,
        audio.sample_rate,
        audio.channels,
        use_k_weighting,
    );
    print_bands(&bands);

    if audio.samples.is_empty() {
        eprintln!("No samples found in file");
        std::process::exit(1);
    }

    let freq_per_bin = audio.sample_rate as f32 / FFT_SIZE as f32;
    let window = create_hanning_window(FFT_SIZE);
    let k_weights = if use_k_weighting {
        Some(create_k_weight_table(FFT_SIZE, audio.sample_rate))
    } else {
        None
    };

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(FFT_SIZE);

    let samples_per_interval = (interval_secs * audio.sample_rate as f32) as usize;
    let total_duration = audio.samples.len() as f32 / audio.sample_rate as f32;
    let num_intervals = (audio.samples.len() + samples_per_interval - 1) / samples_per_interval;

    if num_intervals == 0 {
        eprintln!("File too short for analysis");
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

        let time_secs = interval_start as f32 / audio.sample_rate as f32;
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
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <file> [file2] [file3] ...", args[0]);
        eprintln!();
        eprintln!("Supported formats: WAV, AIFF");
        eprintln!();
        eprintln!("Modes:");
        eprintln!("  <file>              Stats analysis (K-weighting contribution + dynamics)");
        eprintln!("  <file1> <file2> ... Compare files (first file is reference)");
        eprintln!();
        eprintln!("Options:");
        eprintln!("  --time                Timeline analysis (band distribution over time)");
        eprintln!("  --interval <seconds>  Timeline interval (default: 20)");
        eprintln!("  --weighted, -w        Apply K-weighting to timeline");
        std::process::exit(1);
    }

    let mut use_k_weighting = false;
    let mut time_analysis = false;
    let mut interval_secs = 20.0f32;
    let mut filenames: Vec<String> = Vec::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--weighted" | "-w" => {
                use_k_weighting = true;
            }
            "--time" => {
                time_analysis = true;
            }
            "--interval" => {
                if i + 1 >= args.len() {
                    eprintln!("--interval requires a value");
                    std::process::exit(1);
                }
                i += 1;
                interval_secs = args[i].parse().unwrap_or_else(|_| {
                    eprintln!("Invalid interval value: {}", args[i]);
                    std::process::exit(1);
                });
                if interval_secs <= 0.0 {
                    eprintln!("Interval must be positive");
                    std::process::exit(1);
                }
            }
            arg if !arg.starts_with('-') => {
                filenames.push(arg.to_string());
            }
            arg => {
                eprintln!("Unknown option: {}", arg);
                std::process::exit(1);
            }
        }
        i += 1;
    }

    if filenames.is_empty() {
        eprintln!("No input file specified");
        std::process::exit(1);
    }

    if filenames.len() > 5 {
        eprintln!("Too many files specified (max 5)");
        std::process::exit(1);
    }

    // Validate option combinations
    if filenames.len() >= 2 && time_analysis {
        eprintln!("--time cannot be used with multiple files");
        std::process::exit(1);
    }

    if !time_analysis && use_k_weighting {
        eprintln!("--weighted can only be used with --time");
        std::process::exit(1);
    }

    if !time_analysis && interval_secs != 20.0 {
        eprintln!("--interval can only be used with --time");
        std::process::exit(1);
    }

    // Dispatch to appropriate mode
    if filenames.len() >= 2 {
        run_compare(&filenames);
    } else if time_analysis {
        run_timeline(&filenames[0], use_k_weighting, interval_secs);
    } else {
        run_stats(&filenames[0]);
    }
}
