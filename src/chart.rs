//! Chart rendering for band balance comparison

use charming::{
    Chart, ImageRenderer,
    component::{Axis, Grid, Legend, Title},
    element::{
        AxisLabel, AxisType, Color, ColorStop, ItemStyle, Label, LabelPosition, LineStyle,
        SplitLine, Symbol, TextStyle,
    },
    renderer::ImageFormat,
    series::{Bar, Line},
};

use crate::analysis::Band;

/// Data for a single file in the chart
pub struct FileChartData {
    pub label: char,
    pub name: String,
    pub raw_pct: Vec<f64>,
    pub k_pct: Vec<f64>,
}

/// Chart dimensions (2x for Retina quality)
const CHART_WIDTH: u32 = 2800;
const CHART_HEIGHT: u32 = 1200;

/// Colors
const COLOR_BACKGROUND: &str = "#0A0A0C"; // Near black
const COLOR_TEXT: &str = "#FFFFFF"; // White
const COLOR_GRID: &str = "#505050"; // Grid lines

// [A] Blue family
const COLOR_A_TOP: &str = "#68B4FF"; // Blue
const COLOR_A_BOTTOM: &str = "#1888F8"; // Vivid blue
const COLOR_A_LINE: &str = "#88D4FF"; // Light blue for K-wt line

// [B] Pink/Magenta family
const COLOR_B_TOP: &str = "#FF68A8"; // Pink
const COLOR_B_BOTTOM: &str = "#F03888"; // Vivid magenta
const COLOR_B_LINE: &str = "#FF94C0"; // Light pink for K-wt line

// [C] Green family
const COLOR_C_TOP: &str = "#48F89C"; // Green
const COLOR_C_BOTTOM: &str = "#10D878"; // Vivid green
const COLOR_C_LINE: &str = "#78FFB4"; // Light green for K-wt line

// [D] Purple family
const COLOR_D_TOP: &str = "#A478FF"; // Purple
const COLOR_D_BOTTOM: &str = "#7840F8"; // Vivid purple
const COLOR_D_LINE: &str = "#C4A4FF"; // Light purple for K-wt line

/// Format frequency for display (e.g., 1000 -> "1k", 500 -> "500")
fn format_freq(hz: f32) -> String {
    if hz >= 1000.0 {
        let k = hz / 1000.0;
        if k == k.floor() {
            format!("{}k", k as u32)
        } else {
            format!("{:.1}k", k)
        }
    } else {
        format!("{}", hz as u32)
    }
}

/// Build band label with frequency range (2 lines)
fn build_band_label(band: &Band) -> String {
    let freq_range = if band.high_hz == f32::MAX {
        format!("{}+", format_freq(band.low_hz))
    } else {
        format!("{}-{}", format_freq(band.low_hz), format_freq(band.high_hz))
    };
    format!("{}\n{}", band.label, freq_range)
}

/// Color set for each file
struct ColorSet {
    top: &'static str,
    bottom: &'static str,
    line: &'static str,
}

const COLOR_SETS: [ColorSet; 4] = [
    ColorSet {
        top: COLOR_A_TOP,
        bottom: COLOR_A_BOTTOM,
        line: COLOR_A_LINE,
    },
    ColorSet {
        top: COLOR_B_TOP,
        bottom: COLOR_B_BOTTOM,
        line: COLOR_B_LINE,
    },
    ColorSet {
        top: COLOR_C_TOP,
        bottom: COLOR_C_BOTTOM,
        line: COLOR_C_LINE,
    },
    ColorSet {
        top: COLOR_D_TOP,
        bottom: COLOR_D_BOTTOM,
        line: COLOR_D_LINE,
    },
];

/// Maximum number of files supported for chart rendering
pub fn max_chart_files() -> usize {
    COLOR_SETS.len()
}

/// Render a comparison chart to a PNG file (supports 2-4 files)
pub fn render_comparison_chart(
    files: &[FileChartData],
    bands: &[Band],
    output_path: &str,
) -> Result<(), String> {
    if files.len() < 2 || files.len() > COLOR_SETS.len() {
        return Err(format!("Chart requires 2-{} files", COLOR_SETS.len()));
    }

    // Build band labels with frequency ranges (2 lines each)
    let band_labels: Vec<String> = bands.iter().map(build_band_label).collect();

    // Round values to 1 decimal place for display
    let round = |v: &f64| (v * 10.0).round() / 10.0;

    // Build subtitle showing all files
    let subtitle = files
        .iter()
        .map(|f| format!("[{}] {}", f.label, f.name))
        .collect::<Vec<_>>()
        .join("  vs  ");

    // Build legend data with rect icons
    let legend_data: Vec<(String, String)> = files
        .iter()
        .flat_map(|f| {
            vec![
                (format!("[{}] Raw", f.label), "rect".to_string()),
                (format!("[{}] K-wt", f.label), "rect".to_string()),
            ]
        })
        .collect();

    // Create base chart
    let mut chart = Chart::new()
        .background_color(Color::Value(COLOR_BACKGROUND.to_string()))
        .title(
            Title::new()
                .text("Band Energy Distribution")
                .subtext(subtitle)
                .left("center")
                .top("3%")
                .text_style(TextStyle::new().color(COLOR_TEXT).font_size(36))
                .subtext_style(TextStyle::new().color(COLOR_TEXT).font_size(24)),
        )
        .legend(
            Legend::new()
                .data(legend_data)
                .bottom("3%")
                .item_gap(40)
                .text_style(TextStyle::new().color(COLOR_TEXT).font_size(24)),
        )
        .grid(
            Grid::new()
                .left("3%")
                .right("3%")
                .bottom("7%")
                .top("15%")
                .contain_label(true),
        )
        .x_axis(
            Axis::new()
                .type_(AxisType::Category)
                .data(band_labels)
                .axis_label(AxisLabel::new().color(COLOR_TEXT).font_size(24)),
        )
        .y_axis(
            Axis::new()
                .type_(AxisType::Value)
                .name("%")
                .name_text_style(TextStyle::new().color(COLOR_TEXT).font_size(24))
                .axis_label(AxisLabel::new().color(COLOR_TEXT).font_size(24))
                .split_line(
                    SplitLine::new().line_style(LineStyle::new().width(0.5).color(COLOR_GRID)),
                ),
        );

    // Add line series first (background layer)
    for (i, file) in files.iter().enumerate() {
        let colors = &COLOR_SETS[i];
        let data_kwt: Vec<f64> = file.k_pct.iter().map(round).collect();

        chart = chart.series(
            Line::new()
                .name(format!("[{}] K-wt", file.label))
                .data(data_kwt)
                .symbol(Symbol::Circle)
                .symbol_size(10)
                .line_style(LineStyle::new().width(2))
                .item_style(ItemStyle::new().color(colors.line)),
        );
    }

    // Add bar series second (foreground layer)
    for (i, file) in files.iter().enumerate() {
        let colors = &COLOR_SETS[i];
        let data_raw: Vec<f64> = file.raw_pct.iter().map(round).collect();

        chart = chart.series(
            Bar::new()
                .name(format!("[{}] Raw", file.label))
                .data(data_raw)
                .item_style(
                    ItemStyle::new()
                        .color(Color::LinearGradient {
                            x: 0.0,
                            y: 0.0,
                            x2: 0.0,
                            y2: 1.0,
                            color_stops: vec![
                                ColorStop::new(0.0, colors.top),
                                ColorStop::new(1.0, colors.bottom),
                            ],
                        })
                        .opacity(0.9),
                )
                .label(
                    Label::new()
                        .show(true)
                        .position(LabelPosition::Top)
                        .color(COLOR_TEXT)
                        .font_size(20)
                        .formatter("{c}"),
                ),
        );
    }

    // Render to PNG
    let mut renderer = ImageRenderer::new(CHART_WIDTH, CHART_HEIGHT);
    renderer
        .save_format(ImageFormat::Png, &chart, output_path)
        .map_err(|e| format!("Failed to save chart: {}", e))?;

    Ok(())
}
