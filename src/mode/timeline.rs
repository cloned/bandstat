//! Timeline analysis mode (band distribution over time)

use rustfft::FftPlanner;

use crate::analysis::{
    FFT_SIZE, analyze_interval, create_hanning_window, create_k_weight_table, get_bands,
    powers_to_percentages,
};
use crate::audio::{TARGET_SAMPLE_RATE, load_audio};
use crate::chart;
use crate::output::{
    format_time, get_display_name, print_bands, print_error, print_file_info, print_header,
    print_percentages, print_separator,
};

/// Run timeline analysis showing band distribution over time
pub fn run_timeline(
    filename: &str,
    use_k_weighting: bool,
    interval_secs: u32,
    quiet: bool,
    image_path: Option<&str>,
) {
    let bands = get_bands();
    let display_name = get_display_name(filename).to_string();

    let audio = load_audio(filename).unwrap_or_else(|e| {
        print_error(&e.to_string());
        std::process::exit(1);
    });

    if !quiet {
        print_file_info(
            &display_name,
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

    let samples_per_interval = (interval_secs as usize) * (TARGET_SAMPLE_RATE as usize);
    let total_duration = audio.samples.len() as f32 / TARGET_SAMPLE_RATE as f32;
    let num_intervals = audio.samples.len().div_ceil(samples_per_interval);

    if num_intervals == 0 {
        print_error("File too short for analysis");
        std::process::exit(1);
    }

    print_header(&bands, "TIME  ");
    print_separator(&bands, 6);

    let mut total_band_powers = vec![0.0f64; bands.len()];

    // For chart: collect percentages per band per interval
    let mut chart_time_labels: Vec<String> = Vec::new();
    let mut chart_band_pcts: Vec<Vec<f64>> = vec![Vec::new(); bands.len()];

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

        // Convert to percentages for display and chart
        let percentages = powers_to_percentages(&band_powers);
        for (pct, band) in percentages.iter().zip(&bands) {
            let formatted = if *pct < 0.05 {
                "   0.0".to_string()
            } else {
                format!("{:>6.1}", pct)
            };
            let width = band.label.len().max(4);
            print!("{:>width$}", formatted, width = width + 2);
        }
        println!();

        // Store for chart
        if image_path.is_some() {
            chart_time_labels.push(format_time(time_secs).trim().to_string());
            for (band_idx, pct) in percentages.iter().enumerate() {
                chart_band_pcts[band_idx].push(*pct);
            }
        }
    }

    print_separator(&bands, 6);

    print!("AVG   ");
    print_percentages(&total_band_powers);
    println!();

    println!();
    println!("Duration: {}", format_time(total_duration));

    // Output chart if requested
    if let Some(path) = image_path {
        let chart_data = chart::TimelineChartData {
            filename: display_name,
            time_labels: chart_time_labels,
            band_percentages: chart_band_pcts,
        };

        let title = if use_k_weighting {
            "Band Distribution Over Time (K-weighted)"
        } else {
            "Band Distribution Over Time"
        };

        if let Err(e) = chart::render_stacked_chart(&chart_data, &bands, title, path) {
            print_error(&e);
        } else {
            eprintln!("Chart saved to: {}", path);
        }
    }
}
