use crate::analysis::Band;
use colored::*;

fn style_label(label: &str) -> ColoredString {
    label.bold()
}

pub(crate) fn print_error(msg: &str) {
    eprintln!("{}: {}", "error".red().bold(), msg);
}

pub(crate) fn print_warning(msg: &str) {
    eprintln!("{}: {}", "warning".yellow().bold(), msg);
}

pub(crate) fn print_percentages(powers: &[f64], bands: &[Band]) {
    let total: f64 = powers.iter().sum();
    if total > 0.0 {
        for power in powers {
            print!(" {:>5.1}", (power / total) * 100.0);
        }
    } else {
        for _ in bands {
            print!("     -");
        }
    }
}

pub(crate) fn print_separator(bands: &[Band], prefix_width: usize) {
    print!("{}", "-".repeat(prefix_width));
    for _ in bands {
        print!("------");
    }
    println!();
}

pub(crate) fn format_time(seconds: f32) -> String {
    let mins = (seconds / 60.0) as u32;
    let secs = (seconds % 60.0) as u32;
    format!("{:02}:{:02} ", mins, secs)
}

pub(crate) fn get_display_name(filename: &str) -> &str {
    std::path::Path::new(filename)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(filename)
}

pub(crate) fn print_bands(bands: &[Band]) {
    println!("Bands:");
    for band in bands {
        if band.high_hz >= f32::MAX / 2.0 {
            println!("  {:>4}: {:5.0}+      Hz", band.label, band.low_hz);
        } else {
            println!(
                "  {:>4}: {:5.0}-{:5.0} Hz",
                band.label, band.low_hz, band.high_hz
            );
        }
    }
    println!();
}

pub(crate) fn print_header(bands: &[Band], prefix: &str) {
    print!("{}", prefix);
    for band in bands {
        print!(" {:>5}", band.label);
    }
    println!();
}

pub(crate) fn print_row(label: &str, values: &[f64]) {
    print!("{}", label);
    for v in values {
        if v.is_finite() {
            print!(" {:>5.1}", v);
        } else {
            print!("     -");
        }
    }
    println!();
}

pub(crate) fn print_row_styled(label_prefix: &str, label_suffix: &str, values: &[f64]) {
    print!("{}{}", style_label(label_prefix), label_suffix);
    for v in values {
        if v.is_finite() {
            print!(" {:>5.1}", v);
        } else {
            print!("     -");
        }
    }
    println!();
}

pub(crate) fn print_row_masked(label: &str, values: &[f64], mask: &[f64], threshold: f64) {
    print!("{}", label);
    for (v, m) in values.iter().zip(mask) {
        if *m < threshold || !v.is_finite() {
            print!("     -");
        } else {
            print!(" {:>5.1}", v);
        }
    }
    println!();
}

pub(crate) fn print_row_masked_styled(
    label_prefix: &str,
    label_suffix: &str,
    values: &[f64],
    mask: &[f64],
    threshold: f64,
) {
    print!("{}{}", style_label(label_prefix), label_suffix);
    for (v, m) in values.iter().zip(mask) {
        if *m < threshold || !v.is_finite() {
            print!("     -");
        } else {
            print!(" {:>5.1}", v);
        }
    }
    println!();
}

pub(crate) fn print_diff_row_styled(label_prefix: &str, label_suffix: &str, a: &[f64], b: &[f64]) {
    print!("{}{}", style_label(label_prefix), label_suffix);
    for (va, vb) in a.iter().zip(b) {
        let diff = vb - va;
        if diff.is_finite() {
            print_colored_diff(diff);
        } else {
            print!("     -");
        }
    }
    println!();
}

pub(crate) fn print_diff_row_masked_styled(
    label_prefix: &str,
    label_suffix: &str,
    a: &[f64],
    b: &[f64],
    mask_a: &[f64],
    mask_b: &[f64],
    threshold: f64,
) {
    print!("{}{}", style_label(label_prefix), label_suffix);
    for (((va, vb), ma), mb) in a.iter().zip(b).zip(mask_a).zip(mask_b) {
        if *ma < threshold || *mb < threshold {
            print!("     -");
        } else {
            let diff = vb - va;
            if diff.is_finite() {
                print_colored_diff(diff);
            } else {
                print!("     -");
            }
        }
    }
    println!();
}

fn print_colored_diff(diff: f64) {
    let rounded = (diff * 10.0).round() / 10.0;
    if rounded == 0.0 {
        print!("   0.0");
    } else {
        let formatted = format!("{:>+5.1}", diff);
        if rounded > 0.0 {
            print!(" {}", formatted.green());
        } else {
            print!(" {}", formatted.red());
        }
    }
}

pub(crate) fn print_diff_row(label: &str, a: &[f64], b: &[f64]) {
    print!("{}", label);
    for (va, vb) in a.iter().zip(b) {
        let diff = vb - va;
        if diff.is_finite() {
            print_colored_diff(diff);
        } else {
            print!("     -");
        }
    }
    println!();
}

pub(crate) fn print_file_info(
    display_name: &str,
    sample_rate: u32,
    channels: u16,
    k_weighted: bool,
) {
    println!("File: {}", display_name);
    println!("Sample rate: {} Hz, Channels: {}", sample_rate, channels);
    if k_weighted {
        println!("Weighting: K-weighted (ITU-R BS.1770)");
    }
    println!();
}

pub(crate) fn print_legend() {
    println!("Raw: Percentage of total power in each band");
    println!("K-wt: Same as Raw, but with K-weighting applied");
    println!("Diff: Difference between K-wt and Raw");
    println!(
        "Dyn: Per-band dynamics - standard deviation of power (dB). Lower values suggest compression."
    );
}
