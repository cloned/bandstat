//! Color definitions for charts

/// Common colors
pub(super) const COLOR_BACKGROUND: &str = "#0A0A0C"; // Near black
pub(super) const COLOR_TEXT: &str = "#FFFFFF"; // White
pub(super) const COLOR_GRID: &str = "#505050"; // Grid lines

/// Color set for each file in comparison charts
pub(super) struct ColorSet {
    pub(super) top: &'static str,
    pub(super) bottom: &'static str,
    pub(super) line: &'static str,
}

/// Color sets for comparison chart files [A], [B], [C], [D]
pub(super) const COLOR_SETS: [ColorSet; 4] = [
    // [A] Blue family
    ColorSet {
        top: "#68B4FF",    // Blue
        bottom: "#1888F8", // Vivid blue
        line: "#88D4FF",   // Light blue for K-wt line
    },
    // [B] Pink/Magenta family
    ColorSet {
        top: "#FF68A8",    // Pink
        bottom: "#F03888", // Vivid magenta
        line: "#FF94C0",   // Light pink for K-wt line
    },
    // [C] Green family
    ColorSet {
        top: "#48F89C",    // Green
        bottom: "#10D878", // Vivid green
        line: "#78FFB4",   // Light green for K-wt line
    },
    // [D] Purple family
    ColorSet {
        top: "#A478FF",    // Purple
        bottom: "#7840F8", // Vivid purple
        line: "#C4A4FF",   // Light purple for K-wt line
    },
];

/// Timeline chart band colors (14 bands, grouped by frequency range)
/// Low bands (DC, SUB1, SUB2, BASS, UBAS): Blue gradient
/// Mid bands (LMID, MID, UMID, HMID): Green-Yellow gradient
/// High bands (PRES, BRIL, HIGH, UHIG, AIR): Orange-Red gradient
pub(super) const TIMELINE_BAND_COLORS: [&str; 14] = [
    "#1E3A5F", // DC - Dark blue
    "#2858A0", // SUB1 - Blue
    "#3878C0", // SUB2 - Medium blue
    "#4898E0", // BASS - Light blue
    "#58B8F0", // UBAS - Cyan-blue
    "#48C878", // LMID - Green
    "#78D848", // MID - Yellow-green
    "#B8E818", // UMID - Yellow
    "#E8D800", // HMID - Gold
    "#F8A800", // PRES - Orange
    "#F87800", // BRIL - Dark orange
    "#E84800", // HIGH - Red-orange
    "#C82828", // UHIG - Red
    "#982060", // AIR - Magenta
];
