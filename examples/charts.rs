//! Draws the model comparison charts of docs/BENCHMARKS.md from the measured
//! numbers below: `cargo run --release --example charts` rewrites
//! docs/models-quality.svg and docs/models-efficiency.svg. The numbers are typed
//! in by hand after each benchmark run, so the chart and the table cannot drift.

struct Model {
    alias: &'static str,
    family: &'static str,
    params_m: u32,
    /// Expected note among the five returned, text only, per family:
    /// topic of a note (24), buried detail (24), named identifier (12), first benchmark (36).
    top5: [u8; 4],
    /// Peak resident memory of an isolated call, MB, and its wall time in seconds.
    rss_mb: u32,
    latency_s: f32,
}

const CASES: [u32; 4] = [24, 24, 12, 36];
const FAMILIES: [&str; 4] = ["topic of a note (24)", "buried detail (24)", "named identifier (12)", "first benchmark (36)"];

const MODELS: &[Model] = &[
    Model { alias: "granite-multilingual", family: "XLM-RoBERTa", params_m: 278, top5: [83, 58, 75, 81], rss_mb: 315, latency_s: 0.18 },
    Model { alias: "e5-small", family: "BERT", params_m: 118, top5: [62, 58, 100, 64], rss_mb: 222, latency_s: 0.20 },
    Model { alias: "e5-base", family: "XLM-RoBERTa", params_m: 278, top5: [71, 58, 100, 64], rss_mb: 478, latency_s: 0.20 },
    Model { alias: "e5-large", family: "XLM-RoBERTa", params_m: 560, top5: [79, 71, 100, 78], rss_mb: 1527, latency_s: 0.55 },
    Model { alias: "granite-multilingual-r2", family: "ModernBERT", params_m: 97, top5: [83, 54, 100, 72], rss_mb: 290, latency_s: 0.43 },
    Model { alias: "granite-en", family: "ModernBERT", params_m: 149, top5: [67, 71, 100, 50], rss_mb: 370, latency_s: 0.24 },
    Model { alias: "gte-modernbert", family: "ModernBERT", params_m: 149, top5: [50, 50, 92, 39], rss_mb: 370, latency_s: 0.27 },
];

const INK: &str = "#10202b";
const MUTED: &str = "#7c8a95";
const GRID: &str = "#e3e7eb";

/// One colour per encoder family, so the legend says what the shape of the model is.
fn colour(family: &str) -> &'static str {
    match family {
        "XLM-RoBERTa" => "#2d6a9f",
        "BERT" => "#3d7d5a",
        _ => "#b7410e",
    }
}

fn measured(m: &Model) -> bool {
    m.top5.iter().any(|&v| v > 0)
}

/// Hit rate over the 96 queries, weighted by the number of cases of each family.
fn overall(m: &Model) -> f32 {
    let hits: f32 = m.top5.iter().zip(CASES).map(|(&p, n)| p as f32 * n as f32 / 100.0).sum();
    100.0 * hits / CASES.iter().sum::<u32>() as f32
}

fn head(w: u32, h: u32, title: &str, subtitle: &str) -> String {
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\" font-family=\"Helvetica, Arial, sans-serif\" font-size=\"12\">\n  <title>{title}</title>\n  <rect width=\"{w}\" height=\"{h}\" fill=\"#ffffff\"/>\n  <text x=\"20\" y=\"26\" font-size=\"15\" font-weight=\"700\" fill=\"{INK}\">{title}</text>\n  <text x=\"20\" y=\"44\" font-size=\"11\" fill=\"{MUTED}\">{subtitle}</text>\n"
    )
}

fn legend(y: u32) -> String {
    let mut s = String::new();
    let mut x = 20;
    for (family, label) in [("XLM-RoBERTa", "XLM-RoBERTa"), ("BERT", "BERT (MiniLM)"), ("ModernBERT", "ModernBERT")] {
        s += &format!(
            "  <rect x=\"{x}\" y=\"{}\" width=\"12\" height=\"12\" fill=\"{}\"/><text x=\"{}\" y=\"{}\" font-size=\"11\" fill=\"#485864\">{label}</text>\n",
            y - 10,
            colour(family),
            x + 18,
            y
        );
        x += 40 + label.len() as u32 * 7;
    }
    s
}

/// Four panels, one per query family, each model a horizontal bar sorted by score.
fn quality_svg() -> String {
    let models: Vec<&Model> = MODELS.iter().filter(|m| measured(m)).collect();
    let (w, panel_w, bar_h, gap) = (820u32, 360u32, 14u32, 6u32);
    let panel_h = 40 + models.len() as u32 * (bar_h + gap) + 20;
    let h = 70 + 2 * panel_h + 30;
    let mut s = head(
        w,
        h,
        "Expected note among the five returned, by model",
        "Text only, same 96 blind queries and the same notes for every model, Q8 linear layers. Standard error: 10 points at 24 cases, 14 at 12, 8 at 36.",
    );
    s += &legend(62);
    for (i, family) in FAMILIES.iter().enumerate() {
        let (px, py) = (20 + (i as u32 % 2) * (panel_w + 40), 80 + (i as u32 / 2) * panel_h);
        s += &format!("  <text x=\"{px}\" y=\"{}\" font-weight=\"700\" fill=\"{INK}\">{family}</text>\n", py + 12);
        let (x0, scale) = (px + 130, 1.8f32);
        for tick in [0, 50, 100] {
            let x = x0 as f32 + tick as f32 * scale;
            s += &format!("  <line x1=\"{x}\" y1=\"{}\" x2=\"{x}\" y2=\"{}\" stroke=\"{GRID}\"/>\n", py + 20, py + panel_h - 22);
            s += &format!("  <text x=\"{x}\" y=\"{}\" text-anchor=\"middle\" fill=\"{MUTED}\" font-size=\"10\">{tick}</text>\n", py + panel_h - 10);
        }
        let mut sorted = models.clone();
        sorted.sort_by(|a, b| b.top5[i].cmp(&a.top5[i]).then(a.alias.cmp(b.alias)));
        for (j, m) in sorted.iter().enumerate() {
            let y = py + 26 + j as u32 * (bar_h + gap);
            let width = m.top5[i] as f32 * scale;
            s += &format!("  <text x=\"{}\" y=\"{}\" text-anchor=\"end\" fill=\"#485864\" font-size=\"11\">{}</text>\n", x0 - 8, y + 11, m.alias);
            s += &format!("  <rect x=\"{x0}\" y=\"{y}\" width=\"{width}\" height=\"{bar_h}\" rx=\"2\" fill=\"{}\"/>\n", colour(m.family));
            s += &format!("  <text x=\"{}\" y=\"{}\" fill=\"{INK}\" font-size=\"11\">{}</text>\n", x0 as f32 + width + 5.0, y + 11, m.top5[i]);
        }
    }
    s += "</svg>\n";
    s
}

/// Result against cost: overall hit rate over the 96 queries against the peak
/// resident memory of an isolated call. Dot area follows the parameter count.
fn efficiency_svg() -> String {
    let models: Vec<&Model> = MODELS.iter().filter(|m| measured(m) && m.rss_mb > 0).collect();
    let (w, h) = (820u32, 420u32);
    // The plot stops at x1; the labels stack in a column to its right, one line
    // each, joined to their dot by a leader, so they can never overlap.
    let (x0, x1, y0, y1) = (70.0f32, 520.0f32, 360.0f32, 80.0f32);
    let max_mb = models.iter().map(|m| m.rss_mb).max().unwrap_or(1000).next_multiple_of(500) as f32;
    let mut s = head(
        w,
        h,
        "Result against memory, by model",
        "Hit rate over the 96 blind queries (text only) against the peak resident memory of one isolated search. Dot area follows the parameter count.",
    );
    s += &legend(62);
    let step = if max_mb <= 1000.0 { 250 } else { 500 };
    for tick in (0..=max_mb as u32).step_by(step) {
        let x = x0 + tick as f32 / max_mb * (x1 - x0);
        s += &format!("  <line x1=\"{x}\" y1=\"{y1}\" x2=\"{x}\" y2=\"{y0}\" stroke=\"{GRID}\"/>\n  <text x=\"{x}\" y=\"{}\" text-anchor=\"middle\" fill=\"{MUTED}\" font-size=\"10\">{tick}</text>\n", y0 + 16.0);
    }
    for tick in [40, 50, 60, 70, 80, 90] {
        let y = y0 - (tick as f32 - 40.0) / 50.0 * (y0 - y1);
        s += &format!("  <line x1=\"{x0}\" y1=\"{y}\" x2=\"{x1}\" y2=\"{y}\" stroke=\"{GRID}\"/>\n  <text x=\"{}\" y=\"{}\" text-anchor=\"end\" fill=\"{MUTED}\" font-size=\"10\">{tick}</text>\n", x0 - 8.0, y + 4.0);
    }
    s += &format!(
        "  <text x=\"{}\" y=\"{}\" text-anchor=\"middle\" fill=\"#485864\" font-size=\"11\">peak resident memory of an isolated search (MB)</text>\n",
        (x0 + x1) / 2.0,
        y0 + 34.0
    );
    s += &format!("  <text x=\"18\" y=\"{}\" transform=\"rotate(-90 18 {})\" text-anchor=\"middle\" fill=\"#485864\" font-size=\"11\">expected note in the top five (%)</text>\n", (y0 + y1) / 2.0, (y0 + y1) / 2.0);
    let mut placed: Vec<(f32, f32, f32, &Model)> = models
        .iter()
        .map(|m| (x0 + m.rss_mb as f32 / max_mb * (x1 - x0), y0 - (overall(m) - 40.0) / 50.0 * (y0 - y1), 4.0 + (m.params_m as f32).sqrt() * 0.45, *m))
        .collect();
    placed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    // Label rows: in dot order from the top, at least 16 px apart, then pushed back
    // up as a block if the last one would fall under the plot.
    let mut rows: Vec<f32> = Vec::new();
    for (_, y, _, _) in &placed {
        let low = rows.last().map_or(y1, |p| p + 16.0);
        rows.push(y.max(low));
    }
    let overflow = rows.last().map_or(0.0, |last| (last - y0).max(0.0));
    let rows: Vec<f32> = rows.iter().map(|r| r - overflow).collect();
    let column = x1 + 40.0;
    for ((x, y, r, m), ly) in placed.iter().zip(rows) {
        let (x, y, r) = (*x, *y, *r);
        s += &format!(
            "  <circle cx=\"{x}\" cy=\"{y}\" r=\"{r:.1}\" fill=\"{}\" fill-opacity=\"0.85\" stroke=\"#ffffff\" stroke-width=\"2\"/>\n",
            colour(m.family)
        );
        s +=
            &format!("  <polyline points=\"{},{y} {},{ly} {},{ly}\" fill=\"none\" stroke=\"{MUTED}\" stroke-width=\"0.8\"/>\n", x + r, x1 + 20.0, column - 6.0);
        s += &format!(
            "  <text x=\"{column}\" y=\"{}\" fill=\"{INK}\" font-size=\"11\">{} <tspan fill=\"{MUTED}\">{:.0} %, {} MB, {:.2} s</tspan></text>\n",
            ly + 4.0,
            m.alias,
            overall(m),
            m.rss_mb,
            m.latency_s
        );
    }
    s += "</svg>\n";
    s
}

fn main() {
    std::fs::write("docs/models-quality.svg", quality_svg()).expect("docs/models-quality.svg");
    std::fs::write("docs/models-efficiency.svg", efficiency_svg()).expect("docs/models-efficiency.svg");
    for m in MODELS.iter().filter(|m| measured(m)) {
        println!(
            "{:<22} {:>3} {:>3} {:>3} {:>3}  overall {:.0} %  {} MB  {:.2} s",
            m.alias,
            m.top5[0],
            m.top5[1],
            m.top5[2],
            m.top5[3],
            overall(m),
            m.rss_mb,
            m.latency_s
        );
    }
}
