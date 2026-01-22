//! Multiple file comparison mode

use colored::*;

use crate::analysis::{DYNAMICS_DISPLAY_THRESHOLD_PCT, get_bands};
use crate::chart;
use crate::output::{
    print_bands, print_diff_row_masked_styled, print_diff_row_styled, print_error, print_header,
    print_legend, print_row_masked_styled, print_row_styled, print_separator,
};

use super::analyze_file;

/// Run comparison analysis for multiple files
pub fn run_compare(filenames: &[String], quiet: bool, image_path: Option<&str>) {
    let bands = get_bands();
    let labels: Vec<char> = ('A'..='Z').collect();

    let stats: Vec<_> = filenames
        .iter()
        .map(|f| {
            analyze_file(f, &bands, !quiet).unwrap_or_else(|e| {
                print_error(&e);
                std::process::exit(1);
            })
        })
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
