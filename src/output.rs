use crate::analysis::Band;

pub fn print_percentages(powers: &[f64], bands: &[Band]) {
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

pub fn print_separator(bands: &[Band], prefix_width: usize) {
    print!("{}", "-".repeat(prefix_width));
    for _ in bands {
        print!("------");
    }
    println!();
}

pub fn format_time(seconds: f32) -> String {
    let mins = (seconds / 60.0) as u32;
    let secs = (seconds % 60.0) as u32;
    format!("{:02}:{:02} ", mins, secs)
}

pub fn get_display_name(filename: &str) -> &str {
    std::path::Path::new(filename)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(filename)
}

pub fn print_bands(bands: &[Band]) {
    println!("Bands:");
    for band in bands {
        println!(
            "  {:>4}: {:5.0}-{:5.0} Hz",
            band.label, band.low_hz, band.high_hz
        );
    }
    println!();
}

pub fn print_header(bands: &[Band], prefix: &str) {
    print!("{}", prefix);
    for band in bands {
        print!(" {:>5}", band.label);
    }
    println!();
}

pub fn print_row(label: &str, values: &[f64]) {
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

pub fn print_diff_row(label: &str, a: &[f64], b: &[f64]) {
    print!("{}", label);
    for (va, vb) in a.iter().zip(b) {
        let diff = vb - va;
        if diff.is_finite() {
            print!(" {:>+5.1}", diff);
        } else {
            print!("     -");
        }
    }
    println!();
}

pub fn print_file_info(display_name: &str, sample_rate: u32, channels: u16, k_weighted: bool) {
    println!("File: {}", display_name);
    println!("Sample rate: {} Hz, Channels: {}", sample_rate, channels);
    if k_weighted {
        println!("Weighting: K-weighted (ITU-R BS.1770)");
    }
    println!();
}

pub fn print_legend() {
    println!("Raw: Percentage of total power in each band");
    println!("K-wt: Same as Raw, but with K-weighting applied");
    println!("Diff: Difference between K-wt and Raw");
    println!("Std: Standard deviation of band power over time (dB)");
}
