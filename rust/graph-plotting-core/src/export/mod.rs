use crate::io::Point2D;
use crate::interval::PolylineStrip;

/// Publication-quality SVG and LaTeX exporter.
pub struct PublicationExporter;

impl PublicationExporter {
    /// Export graph to publication-grade SVG string.
    pub fn export_svg(
        strips: &[PolylineStrip],
        points: &[Point2D],
        conf_upper: &[f32],
        conf_lower: &[f32],
        equation_unicode: &str,
        r_squared: Option<f64>,
        x_min: f64,
        x_max: f64,
        y_min: f64,
        y_max: f64,
        width: u32,
        height: u32,
    ) -> String {
        let w = width as f64;
        let h = height as f64;
        let margin_l = 60.0;
        let margin_r = 30.0;
        let margin_t = 30.0;
        let margin_b = 50.0;

        let plot_w = w - margin_l - margin_r;
        let plot_h = h - margin_t - margin_b;

        let x_span = (x_max - x_min).max(1e-6);
        let y_span = (y_max - y_min).max(1e-6);

        let to_sx = |x: f64| margin_l + (x - x_min) / x_span * plot_w;
        let to_sy = |y: f64| margin_t + (y_max - y) / y_span * plot_h;

        let mut svg = String::new();
        svg.push_str(&format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {} {}\" width=\"{}\" height=\"{}\">\n",
            width, height, width, height
        ));
        svg.push_str("  <defs>\n");
        svg.push_str(&format!(
            "    <clipPath id=\"plot-area\">\n      <rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" />\n    </clipPath>\n",
            margin_l, margin_t, plot_w, plot_h
        ));
        svg.push_str("  </defs>\n");
        svg.push_str("  <rect width=\"100%\" height=\"100%\" fill=\"#FFFFFF\" />\n");

        // Draw Grid
        svg.push_str("  <!-- Grid -->\n  <g stroke=\"#E5E5EA\" stroke-width=\"1\">\n");
        let n_ticks = 8;
        for i in 0..=n_ticks {
            let gx = x_min + (i as f64) * x_span / (n_ticks as f64);
            let sx = to_sx(gx);
            svg.push_str(&format!("    <line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" />\n", sx, margin_t, sx, margin_t + plot_h));

            let gy = y_min + (i as f64) * y_span / (n_ticks as f64);
            let sy = to_sy(gy);
            svg.push_str(&format!("    <line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" />\n", margin_l, sy, margin_l + plot_w, sy));
        }
        svg.push_str("  </g>\n");

        // Draw Axes (x=0 and y=0 lines if within bounds)
        svg.push_str("  <!-- Axes -->\n  <g stroke=\"#8E8E93\" stroke-width=\"1.5\">\n");
        if x_min <= 0.0 && x_max >= 0.0 {
            let sx0 = to_sx(0.0);
            svg.push_str(&format!("    <line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" />\n", sx0, margin_t, sx0, margin_t + plot_h));
        }
        if y_min <= 0.0 && y_max >= 0.0 {
            let sy0 = to_sy(0.0);
            svg.push_str(&format!("    <line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" />\n", margin_l, sy0, margin_l + plot_w, sy0));
        }
        svg.push_str("  </g>\n");

        // Axis Tick Labels
        svg.push_str("  <!-- Tick Labels -->\n  <g font-size=\"11\" fill=\"#636366\" text-anchor=\"middle\" font-family=\"sans-serif\">\n");
        for i in 0..=n_ticks {
            let gx = x_min + (i as f64) * x_span / (n_ticks as f64);
            let sx = to_sx(gx);
            svg.push_str(&format!("    <text x=\"{:.1}\" y=\"{:.1}\">{:.2}</text>\n", sx, margin_t + plot_h + 16.0, gx));
        }
        for i in 0..=n_ticks {
            let gy = y_min + (i as f64) * y_span / (n_ticks as f64);
            let sy = to_sy(gy);
            svg.push_str(&format!("    <text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\">{:.2}</text>\n", margin_l - 8.0, sy + 4.0, gy));
        }
        svg.push_str("  </g>\n");

        // Plot container with clipping
        svg.push_str("  <g clip-path=\"url(#plot-area)\">\n");

        // Draw 95% Confidence Band
        if !conf_upper.is_empty() && conf_upper.len() == conf_lower.len() && conf_upper.len() >= 4 {
            svg.push_str("    <!-- 95% Confidence Band -->\n    <polygon fill=\"rgba(0, 122, 255, 0.15)\" points=\"");
            let count = conf_upper.len() / 2;
            for i in 0..count {
                let x = conf_upper[i * 2] as f64;
                let y = conf_upper[i * 2 + 1] as f64;
                svg.push_str(&format!("{:.1},{:.1} ", to_sx(x), to_sy(y)));
            }
            for i in (0..count).rev() {
                let x = conf_lower[i * 2] as f64;
                let y = conf_lower[i * 2 + 1] as f64;
                svg.push_str(&format!("{:.1},{:.1} ", to_sx(x), to_sy(y)));
            }
            svg.push_str("\" />\n");
        }

        // Draw Curves
        for (idx, strip) in strips.iter().enumerate() {
            let n_pts = strip.point_count();
            if n_pts < 2 { continue; }
            let color = match idx % 4 {
                0 => "#007AFF", // Blue
                1 => "#FF9500", // Orange
                2 => "#34C759", // Green
                _ => "#AF52DE", // Purple
            };
            svg.push_str(&format!("    <path fill=\"none\" stroke=\"{}\" stroke-width=\"2.5\" stroke-linecap=\"round\" stroke-linejoin=\"round\" d=\"M ", color));
            for i in 0..n_pts {
                let px = strip.points[i * 2] as f64;
                let py = strip.points[i * 2 + 1] as f64;
                let sx = to_sx(px);
                let sy = to_sy(py);
                if i == 0 {
                    svg.push_str(&format!("{:.1},{:.1}", sx, sy));
                } else {
                    svg.push_str(&format!(" L {:.1},{:.1}", sx, sy));
                }
            }
            svg.push_str("\" />\n");
        }

        // Draw Data Points
        svg.push_str("    <!-- Data Points -->\n    <g fill=\"#007AFF\" stroke=\"#FFFFFF\" stroke-width=\"1.5\">\n");
        for p in points {
            let sx = to_sx(p.x);
            let sy = to_sy(p.y);
            svg.push_str(&format!("      <circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"4\" />\n", sx, sy));
        }
        svg.push_str("    </g>\n");
        svg.push_str("  </g>\n");

        // Border rectangle around plot
        svg.push_str(&format!(
            "  <rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"none\" stroke=\"#C7C7CC\" stroke-width=\"1\" />\n",
            margin_l, margin_t, plot_w, plot_h
        ));

        // Equation Card (if present)
        if !equation_unicode.is_empty() {
            let r2_str = if let Some(r2) = r_squared {
                format!("  |  R² = {:.4}", r2)
            } else {
                "".into()
            };
            svg.push_str(&format!(
                "  <!-- Equation Card -->\n  <g transform=\"translate({:.1}, {:.1})\">\n    <rect width=\"320\" height=\"34\" rx=\"8\" fill=\"rgba(255, 255, 255, 0.92)\" stroke=\"#D1D1D6\" stroke-width=\"1\" />\n    <text x=\"12\" y=\"22\" font-size=\"12\" font-family=\"monospace\" fill=\"#1C1C1E\">{}{}</text>\n  </g>\n",
                margin_l + 12.0, margin_t + 12.0, equation_unicode, r2_str
            ));
        }

        svg.push_str("</svg>\n");
        svg
    }

    /// Export graph to publication-ready LaTeX / TikZ code using PGFPlots.
    pub fn export_latex(
        equation_latex: &str,
        points: &[Point2D],
        r_squared: Option<f64>,
        x_min: f64,
        x_max: f64,
        y_min: f64,
        y_max: f64,
    ) -> String {
        let mut out = String::new();
        out.push_str("% Graph Plotting 2.0 Publication LaTeX / TikZ Export\n");
        out.push_str("\\begin{tikzpicture}\n");
        out.push_str("\\begin{axis}[\n");
        out.push_str("    width=10cm,\n");
        out.push_str("    height=7.5cm,\n");
        out.push_str(&format!("    xmin={:.2}, xmax={:.2},\n", x_min, x_max));
        out.push_str(&format!("    ymin={:.2}, ymax={:.2},\n", y_min, y_max));
        out.push_str("    axis lines=middle,\n");
        out.push_str("    grid=both,\n");
        out.push_str("    grid style={line width=.1pt, draw=gray!20},\n");
        out.push_str("    major grid style={line width=.2pt, draw=gray!40},\n");
        out.push_str("    xlabel={$x$},\n");
        out.push_str("    ylabel={$y$},\n");
        out.push_str("    legend pos=north west,\n");
        out.push_str("    legend style={font=\\footnotesize}\n");
        out.push_str("]\n\n");

        // Data points plot
        if !points.is_empty() {
            out.push_str("% Experimental Data Points\n");
            out.push_str("\\addplot[\n");
            out.push_str("    only marks,\n");
            out.push_str("    mark=*, mark size=2pt,\n");
            out.push_str("    color=blue!80!black\n");
            out.push_str("] coordinates {\n");
            for p in points {
                out.push_str(&format!("    ({:.4}, {:.4})\n", p.x, p.y));
            }
            out.push_str("};\n");
            out.push_str("\\addlegendentry{Data Points}\n\n");
        }

        // Fitted Model equation plot
        if !equation_latex.is_empty() {
            let r2_comment = if let Some(r2) = r_squared {
                format!(" % R^2 = {:.4}", r2)
            } else {
                "".into()
            };
            out.push_str(&format!("% Fitted Model: {}{}\n", equation_latex, r2_comment));
            out.push_str("% Tip: Insert expression into \\addplot[domain=...] {...};\n");
        }

        out.push_str("\\end{axis}\n");
        out.push_str("\\end{tikzpicture}\n");
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_svg_contains_key_elements() {
        let pts = vec![Point2D { x: 1.0, y: 2.0 }, Point2D { x: 2.0, y: 4.0 }];
        let svg = PublicationExporter::export_svg(
            &[],
            &pts,
            &[],
            &[],
            "y = 2x",
            Some(1.0),
            0.0,
            5.0,
            0.0,
            10.0,
            800,
            600,
        );
        assert!(svg.contains("<svg"));
        assert!(svg.contains("y = 2x"));
        assert!(svg.contains("R² = 1.0000"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_export_latex_contains_pgfplots() {
        let pts = vec![Point2D { x: 1.0, y: 2.0 }];
        let latex = PublicationExporter::export_latex("y = 2x", &pts, Some(1.0), 0.0, 5.0, 0.0, 10.0);
        assert!(latex.contains("\\begin{tikzpicture}"));
        assert!(latex.contains("\\begin{axis}"));
        assert!(latex.contains("(1.0000, 2.0000)"));
        assert!(latex.contains("\\end{tikzpicture}"));
    }
}
