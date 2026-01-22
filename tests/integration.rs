//! Integration tests for bandstat CLI

mod common;

use std::process::Command;
use tempfile::TempDir;

/// Get the path to the bandstat binary
fn bandstat_bin() -> std::path::PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    path.pop(); // Remove deps
    path.push("bandstat");
    path
}

/// Run bandstat with the given arguments
fn run_bandstat(args: &[&str]) -> std::process::Output {
    Command::new(bandstat_bin())
        .args(args)
        .output()
        .expect("failed to execute bandstat")
}

/// Create a test WAV file in the given directory
fn create_test_wav(dir: &TempDir, name: &str, freq: f32, duration: f32) -> std::path::PathBuf {
    let samples = common::generate_sine(freq, 48000, duration);
    let path = dir.path().join(format!("{}.wav", name));
    common::write_wav(&path, &samples, 48000).unwrap();
    path
}

/// Create a noise WAV file in the given directory
fn create_noise_wav(dir: &TempDir, name: &str, duration: f32) -> std::path::PathBuf {
    let samples = common::generate_noise(48000, duration, 12345);
    let path = dir.path().join(format!("{}.wav", name));
    common::write_wav(&path, &samples, 48000).unwrap();
    path
}

// =============================================================================
// Basic functionality tests
// =============================================================================

#[test]
fn test_help_flag() {
    let output = run_bandstat(&["--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Audio frequency band analyzer"));
    assert!(stdout.contains("--time"));
    assert!(stdout.contains("--interval"));
    assert!(stdout.contains("--weighted"));
    assert!(stdout.contains("--image"));
}

#[test]
fn test_version_flag() {
    let output = run_bandstat(&["--version"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("bandstat"));
}

// =============================================================================
// Single file analysis mode
// =============================================================================

#[test]
fn test_single_file_analysis() {
    let temp_dir = TempDir::new().unwrap();
    let wav_path = create_test_wav(&temp_dir, "test", 1000.0, 2.0);

    let output = run_bandstat(&["-q", wav_path.to_str().unwrap()]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[Band Power Distribution]"));
    assert!(stdout.contains("Raw(%)"));
    assert!(stdout.contains("K-wt(%)"));
    assert!(stdout.contains("[Dynamics]"));
}

#[test]
fn test_single_file_verbose() {
    let temp_dir = TempDir::new().unwrap();
    let wav_path = create_test_wav(&temp_dir, "test", 440.0, 1.0);

    let output = run_bandstat(&[wav_path.to_str().unwrap()]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Verbose mode includes file info and bands explanation
    assert!(stdout.contains("Stats Analysis"));
    assert!(stdout.contains("DC"));
    assert!(stdout.contains("AIR"));
}

#[test]
fn test_single_file_with_image() {
    let temp_dir = TempDir::new().unwrap();
    let wav_path = create_test_wav(&temp_dir, "test", 1000.0, 2.0);
    let image_path = temp_dir.path().join("output.png");

    let output = run_bandstat(&[
        "-q",
        wav_path.to_str().unwrap(),
        "--image",
        image_path.to_str().unwrap(),
    ]);
    assert!(output.status.success());

    // Check that the image file was created
    assert!(image_path.exists(), "Image file should be created");
    assert!(
        std::fs::metadata(&image_path).unwrap().len() > 0,
        "Image file should not be empty"
    );
}

#[test]
fn test_single_file_with_weighted_image() {
    let temp_dir = TempDir::new().unwrap();
    let wav_path = create_test_wav(&temp_dir, "test", 1000.0, 2.0);
    let image_path = temp_dir.path().join("output_k.png");

    let output = run_bandstat(&[
        "-q",
        "-w",
        wav_path.to_str().unwrap(),
        "--image",
        image_path.to_str().unwrap(),
    ]);
    assert!(output.status.success());
    assert!(image_path.exists());
}

#[test]
fn test_weighted_without_image_warning() {
    let temp_dir = TempDir::new().unwrap();
    let wav_path = create_test_wav(&temp_dir, "test", 1000.0, 1.0);

    let output = run_bandstat(&["-q", "-w", wav_path.to_str().unwrap()]);
    assert!(output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Warning:"),
        "Should warn about --weighted without --image"
    );
}

// =============================================================================
// Comparison mode (multiple files)
// =============================================================================

#[test]
fn test_compare_two_files() {
    let temp_dir = TempDir::new().unwrap();
    let wav1 = create_test_wav(&temp_dir, "mix", 440.0, 2.0);
    let wav2 = create_test_wav(&temp_dir, "ref", 880.0, 2.0);

    let output = run_bandstat(&["-q", wav1.to_str().unwrap(), wav2.to_str().unwrap()]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Comparison (base: [A])"));
    assert!(stdout.contains("[A]"));
    assert!(stdout.contains("[B]"));
    assert!(stdout.contains("B-A"));
}

#[test]
fn test_compare_three_files() {
    let temp_dir = TempDir::new().unwrap();
    let wav1 = create_test_wav(&temp_dir, "a", 440.0, 1.0);
    let wav2 = create_test_wav(&temp_dir, "b", 880.0, 1.0);
    let wav3 = create_test_wav(&temp_dir, "c", 1760.0, 1.0);

    let output = run_bandstat(&[
        "-q",
        wav1.to_str().unwrap(),
        wav2.to_str().unwrap(),
        wav3.to_str().unwrap(),
    ]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[A]"));
    assert!(stdout.contains("[B]"));
    assert!(stdout.contains("[C]"));
    assert!(stdout.contains("B-A"));
    assert!(stdout.contains("C-A"));
}

#[test]
fn test_compare_with_image() {
    let temp_dir = TempDir::new().unwrap();
    let wav1 = create_test_wav(&temp_dir, "a", 440.0, 2.0);
    let wav2 = create_test_wav(&temp_dir, "b", 880.0, 2.0);
    let image_path = temp_dir.path().join("comparison.png");

    let output = run_bandstat(&[
        "-q",
        wav1.to_str().unwrap(),
        wav2.to_str().unwrap(),
        "--image",
        image_path.to_str().unwrap(),
    ]);
    assert!(output.status.success());
    assert!(image_path.exists());
}

// =============================================================================
// Timeline mode
// =============================================================================

#[test]
fn test_timeline_mode() {
    let temp_dir = TempDir::new().unwrap();
    // Create a longer file for timeline analysis
    let wav_path = create_noise_wav(&temp_dir, "long", 45.0);

    let output = run_bandstat(&["-q", "-t", wav_path.to_str().unwrap()]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("TIME"));
    assert!(stdout.contains("00:00"));
    assert!(stdout.contains("00:20"));
    assert!(stdout.contains("AVG"));
    assert!(stdout.contains("Duration:"));
}

#[test]
fn test_timeline_with_custom_interval() {
    let temp_dir = TempDir::new().unwrap();
    let wav_path = create_noise_wav(&temp_dir, "long", 25.0);

    let output = run_bandstat(&["-q", "-t", "-i", "10", wav_path.to_str().unwrap()]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("00:00"));
    assert!(stdout.contains("00:10"));
    assert!(stdout.contains("00:20"));
}

#[test]
fn test_timeline_with_k_weighting() {
    let temp_dir = TempDir::new().unwrap();
    let wav_path = create_noise_wav(&temp_dir, "long", 25.0);

    let output = run_bandstat(&["-q", "-t", "-w", wav_path.to_str().unwrap()]);
    assert!(output.status.success());
}

#[test]
fn test_timeline_with_image() {
    let temp_dir = TempDir::new().unwrap();
    let wav_path = create_noise_wav(&temp_dir, "long", 45.0);
    let image_path = temp_dir.path().join("timeline.png");

    let output = run_bandstat(&[
        "-q",
        "-t",
        wav_path.to_str().unwrap(),
        "--image",
        image_path.to_str().unwrap(),
    ]);
    assert!(output.status.success());
    assert!(image_path.exists());
}

// =============================================================================
// Error cases
// =============================================================================

#[test]
fn test_no_files_error() {
    let output = run_bandstat(&[]);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("required"));
}

#[test]
fn test_nonexistent_file_error() {
    let output = run_bandstat(&["/nonexistent/path/audio.wav"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error:") || stderr.contains("error"));
}

#[test]
fn test_time_with_multiple_files_error() {
    let temp_dir = TempDir::new().unwrap();
    let wav1 = create_test_wav(&temp_dir, "a", 440.0, 1.0);
    let wav2 = create_test_wav(&temp_dir, "b", 880.0, 1.0);

    let output = run_bandstat(&["-t", wav1.to_str().unwrap(), wav2.to_str().unwrap()]);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--time cannot be used with multiple files"));
}

#[test]
fn test_weighted_with_comparison_error() {
    let temp_dir = TempDir::new().unwrap();
    let wav1 = create_test_wav(&temp_dir, "a", 440.0, 1.0);
    let wav2 = create_test_wav(&temp_dir, "b", 880.0, 1.0);

    let output = run_bandstat(&["-w", wav1.to_str().unwrap(), wav2.to_str().unwrap()]);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--weighted cannot be used with comparison mode"));
}

#[test]
fn test_interval_without_time_error() {
    let temp_dir = TempDir::new().unwrap();
    let wav_path = create_test_wav(&temp_dir, "test", 440.0, 1.0);

    let output = run_bandstat(&["-i", "10", wav_path.to_str().unwrap()]);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--interval can only be used with --time"));
}

#[test]
fn test_too_many_files_for_image_error() {
    let temp_dir = TempDir::new().unwrap();
    let wavs: Vec<_> = (0..5)
        .map(|i| create_test_wav(&temp_dir, &format!("f{}", i), 440.0 * (i + 1) as f32, 1.0))
        .collect();
    let image_path = temp_dir.path().join("chart.png");

    let args: Vec<&str> = std::iter::once("-q")
        .chain(wavs.iter().map(|p| p.to_str().unwrap()))
        .chain(["--image", image_path.to_str().unwrap()])
        .collect();

    let output = run_bandstat(&args);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--image supports up to"));
}

#[test]
fn test_image_invalid_directory_error() {
    let temp_dir = TempDir::new().unwrap();
    let wav_path = create_test_wav(&temp_dir, "test", 440.0, 1.0);

    let output = run_bandstat(&[
        "-q",
        wav_path.to_str().unwrap(),
        "--image",
        "/nonexistent/dir/chart.png",
    ]);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Directory does not exist"));
}

// =============================================================================
// Output format tests
// =============================================================================

#[test]
fn test_no_color_option() {
    let temp_dir = TempDir::new().unwrap();
    let wav_path = create_test_wav(&temp_dir, "test", 440.0, 1.0);

    let output = run_bandstat(&["--no-color", wav_path.to_str().unwrap()]);
    assert!(output.status.success());

    // Output should not contain ANSI escape codes
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("\x1b["),
        "Should not contain ANSI escape codes"
    );
}

#[test]
fn test_quiet_mode_reduces_output() {
    let temp_dir = TempDir::new().unwrap();
    let wav_path = create_test_wav(&temp_dir, "test", 440.0, 1.0);

    let verbose_output = run_bandstat(&[wav_path.to_str().unwrap()]);
    let quiet_output = run_bandstat(&["-q", wav_path.to_str().unwrap()]);

    let verbose_stdout = String::from_utf8_lossy(&verbose_output.stdout);
    let quiet_stdout = String::from_utf8_lossy(&quiet_output.stdout);

    // Quiet mode should have less output
    assert!(quiet_stdout.len() < verbose_stdout.len());

    // Quiet mode should not include legend
    assert!(!quiet_stdout.contains("Legend:") && !quiet_stdout.contains("Diff:"));
}

// =============================================================================
// Frequency accuracy tests
// =============================================================================

#[test]
fn test_750hz_sine_in_mid_band() {
    let temp_dir = TempDir::new().unwrap();
    // 750 Hz should be in the MID band (500-1000 Hz)
    let wav_path = create_test_wav(&temp_dir, "750hz", 750.0, 2.0);

    let output = run_bandstat(&["-q", wav_path.to_str().unwrap()]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse the Raw(%) line to verify MID band has significant power
    // The output format is: "Raw(%)   0.0   0.0   0.0   0.0  ..."
    // MID band (500-1000 Hz) should have the dominant power
    for line in stdout.lines() {
        if line.starts_with("Raw(%)") {
            let values: Vec<f64> = line
                .split_whitespace()
                .skip(1) // Skip "Raw(%)"
                .filter_map(|s| s.parse().ok())
                .collect();

            // MID band is index 6 (0-indexed: DC, SUB1, SUB2, BASS, UBAS, LMID, MID)
            if values.len() >= 7 {
                let mid_power = values[6];
                assert!(
                    mid_power > 50.0,
                    "750Hz sine should have >50% power in MID band, got {}%",
                    mid_power
                );
            }
            break;
        }
    }
}

#[test]
fn test_100hz_sine_in_bass_band() {
    let temp_dir = TempDir::new().unwrap();
    // 100 Hz should be in the BASS band (60-120 Hz)
    let wav_path = create_test_wav(&temp_dir, "100hz", 100.0, 2.0);

    let output = run_bandstat(&["-q", wav_path.to_str().unwrap()]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        if line.starts_with("Raw(%)") {
            let values: Vec<f64> = line
                .split_whitespace()
                .skip(1)
                .filter_map(|s| s.parse().ok())
                .collect();

            // BASS band is index 3 (DC, SUB1, SUB2, BASS)
            if values.len() >= 4 {
                let bass_power = values[3];
                assert!(
                    bass_power > 50.0,
                    "100Hz sine should have >50% power in BASS band, got {}%",
                    bass_power
                );
            }
            break;
        }
    }
}
