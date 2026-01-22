//! Comparison chart rendering (bar chart with K-weighted overlay lines)

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

use super::colors::{COLOR_BACKGROUND, COLOR_GRID, COLOR_SETS, COLOR_TEXT};
use super::{CHART_HEIGHT, CHART_WIDTH, FileChartData, build_band_label};
use crate::analysis::Band;

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
