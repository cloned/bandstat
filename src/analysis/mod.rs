//! Audio frequency band analysis

mod bands;
mod fft;
mod kweight;

pub(crate) use bands::{Band, get_bands};
pub(crate) use fft::{
    DYNAMICS_DISPLAY_THRESHOLD_PCT, FFT_SIZE, analyze_interval, analyze_stats,
    create_hanning_window, powers_to_percentages,
};
pub(crate) use kweight::create_k_weight_table;

#[cfg(test)]
mod tests;
