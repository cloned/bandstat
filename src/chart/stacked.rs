//! Stacked bar chart rendering (for timeline and single-file modes)

use charming::{
    Chart, ImageRenderer,
    component::{Axis, Grid, Legend, Title},
    element::{
        AxisLabel, AxisType, Color, ItemStyle, Label, LabelPosition, LineStyle, SplitLine,
        TextStyle,
    },
    renderer::ImageFormat,
    series::Bar,
};

use super::colors::{COLOR_BACKGROUND, COLOR_GRID, COLOR_TEXT, TIMELINE_BAND_COLORS};
use super::{CHART_WIDTH, TimelineChartData, build_band_legend_label};
use crate::analysis::Band;

/// Chart height for stacked charts
const STACKED_CHART_HEIGHT: u32 = 1200;

/// Render a stacked bar chart for band distribution
/// Used for both timeline mode (multiple time points) and single-file stats mode (single bar)
pub fn render_stacked_chart(
    data: &TimelineChartData,
    bands: &[Band],
    title: &str,
    output_path: &str,
) -> Result<(), String> {
    if data.time_labels.is_empty() {
        return Err("No data to render".to_string());
    }

    // Build legend data with frequency ranges (1-line format for legend)
    let legend_data: Vec<String> = bands.iter().map(build_band_legend_label).collect();

    // For single-bar mode, hide x-axis labels
    let is_single_bar = data.time_labels.len() == 1;

    let mut chart = Chart::new()
        .background_color(Color::Value(COLOR_BACKGROUND.to_string()))
        .title(
            Title::new()
                .text(title)
                .subtext(&data.filename)
                .left("center")
                .top("3%")
                .text_style(TextStyle::new().color(COLOR_TEXT).font_size(36))
                .subtext_style(TextStyle::new().color(COLOR_TEXT).font_size(24)),
        )
        .legend(
            Legend::new()
                .data(legend_data)
                .bottom("3%")
                .item_gap(16)
                .text_style(TextStyle::new().color(COLOR_TEXT).font_size(16)),
        )
        .grid(
            Grid::new()
                .left("5%")
                .right("3%")
                .bottom("10%")
                .top("15%")
                .contain_label(true),
        )
        .y_axis(
            Axis::new()
                .type_(AxisType::Value)
                .name("%")
                .max(100)
                .name_text_style(TextStyle::new().color(COLOR_TEXT).font_size(24))
                .axis_label(AxisLabel::new().color(COLOR_TEXT).font_size(20))
                .split_line(
                    SplitLine::new().line_style(LineStyle::new().width(0.5).color(COLOR_GRID)),
                ),
        );

    // For single-bar mode, hide x-axis labels; otherwise show time labels
    let x_axis = Axis::new()
        .type_(AxisType::Category)
        .boundary_gap(true)
        .data(data.time_labels.clone());

    chart = chart.x_axis(if is_single_bar {
        x_axis.axis_label(AxisLabel::new().show(false))
    } else {
        x_axis.axis_label(AxisLabel::new().color(COLOR_TEXT).font_size(20))
    });

    // Calculate bar width based on grid and number of intervals
    // Grid width is ~92% of chart (5% left + 3% right margins)
    let grid_width = (CHART_WIDTH as f64) * 0.92;
    let num_intervals = data.time_labels.len().max(1) as f64;
    // For single bar, limit width to 1/3 of grid; otherwise fill grid
    let bar_width = if is_single_bar {
        grid_width / 3.0
    } else {
        grid_width / num_intervals
    };

    // Threshold for showing labels (percentage must be at least this value)
    const LABEL_THRESHOLD: f64 = 5.0;
    // Larger font for single bar mode
    let label_font_size = if is_single_bar { 18.0 } else { 14.0 };

    // Add stacked bar series for each band (low frequencies at bottom, high at top)
    for (band_idx, band) in bands.iter().enumerate() {
        let color = TIMELINE_BAND_COLORS
            .get(band_idx)
            .unwrap_or(&TIMELINE_BAND_COLORS[0]);

        let bar_data: Vec<f64> = data
            .band_percentages
            .get(band_idx)
            .map(|v| v.iter().map(|x| (x * 10.0).round() / 10.0).collect())
            .unwrap_or_default();

        // Check if any value in this band exceeds threshold (to decide if we show labels)
        let has_significant_values = bar_data.iter().any(|&v| v >= LABEL_THRESHOLD);

        // Use legend label (with frequency) as series name for legend matching
        let series_name = build_band_legend_label(band);

        let mut bar = Bar::new()
            .name(&series_name)
            .data(bar_data)
            .stack("total")
            .bar_width(bar_width)
            .item_style(ItemStyle::new().color(*color));

        // Only add labels for bands that have significant values
        if has_significant_values {
            bar = bar.label(
                Label::new()
                    .show(true)
                    .position(LabelPosition::Inside)
                    .color(COLOR_TEXT)
                    .font_size(label_font_size)
                    .font_weight("bold")
                    .formatter("{c}"),
            );
        }

        chart = chart.series(bar);
    }

    // Render to PNG
    let mut renderer = ImageRenderer::new(CHART_WIDTH, STACKED_CHART_HEIGHT);
    renderer
        .save_format(ImageFormat::Png, &chart, output_path)
        .map_err(|e| format!("Failed to save chart: {}", e))?;

    Ok(())
}
