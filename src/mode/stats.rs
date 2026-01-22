//! Single file stats analysis mode

use crate::analysis::{DYNAMICS_DISPLAY_THRESHOLD_PCT, get_bands};
use crate::chart;
use crate::output::{
    print_bands, print_diff_row, print_error, print_file_info, print_header, print_legend,
    print_row, print_row_masked, print_separator,
};

use super::{FileStats, analyze_file};

/// Run single file stats analysis
pub fn run_stats(filename: &str, use_k_weighting: bool, quiet: bool, image_path: Option<&str>) {
    let bands = get_bands();
    let stats: FileStats = analyze_file(filename, &bands, !quiet).unwrap_or_else(|e| {
        print_error(&e);
        std::process::exit(1);
    });

    if !quiet {
        println!();
        println!("Stats Analysis");
        print_file_info(
            &stats.name,
            stats.original_sample_rate,
            stats.channels,
            use_k_weighting,
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

    // Output chart if requested
    if let Some(path) = image_path {
        let pct_data = if use_k_weighting {
            &stats.k_pct
        } else {
            &stats.raw_pct
        };

        let chart_data = chart::TimelineChartData {
            filename: stats.name,
            time_labels: vec!["".to_string()], // Single bar, no label
            band_percentages: pct_data.iter().map(|&v| vec![v]).collect(),
        };

        let title = if use_k_weighting {
            "Band Distribution (K-weighted)"
        } else {
            "Band Distribution"
        };

        if let Err(e) = chart::render_stacked_chart(&chart_data, &bands, title, path) {
            print_error(&e);
        } else {
            eprintln!("Chart saved to: {}", path);
        }
    }
}
