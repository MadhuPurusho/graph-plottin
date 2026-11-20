use serde::{Deserialize, Serialize};

/// 2D coordinate point representation for I/O and FFI.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

/// Versioned .gplot project schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDoc {
    pub schema_version: u32,
    pub app_name: String,
    pub mode: String,
    pub expressions: Vec<SavedExpression>,
    pub points: Vec<SavedPoint>,
    pub fit_settings: SavedFitSettings,
    pub viewport: SavedViewport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedExpression {
    pub id: String,
    pub text: String,
    pub color_hex: String,
    pub is_visible: bool,
    pub show_derivative: bool,
    pub show_second_derivative: bool,
    pub domain_min: Option<f64>,
    pub domain_max: Option<f64>,
    pub integral_a: Option<f64>,
    pub integral_b: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedPoint {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub is_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedFitSettings {
    pub selected_fit_type: String,
    pub use_robust: bool,
    pub show_confidence_band: bool,
    pub show_prediction_band: bool,
    pub show_residuals: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedViewport {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

/// Delimiter-sniffing CSV/TSV parser and project I/O.
pub struct DataIO;

impl DataIO {
    /// Parse CSV or TSV content with automatic delimiter detection and header skipping.
    pub fn parse_csv(content: &str) -> Result<Vec<Point2D>, String> {
        let lines: Vec<&str> = content
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect();

        if lines.is_empty() {
            return Ok(Vec::new());
        }

        // Delimiter sniffing
        let delimiter = Self::sniff_delimiter(&lines);

        let mut points = Vec::new();
        let mut start_idx = 0;

        // Check if first line is a header
        if let Some(first_line) = lines.first() {
            let parts: Vec<&str> = first_line.split(delimiter).map(|s| s.trim()).collect();
            if parts.len() >= 2 {
                let p0_float = parts[0].parse::<f64>().is_ok();
                let p1_float = parts[1].parse::<f64>().is_ok();
                if !p0_float || !p1_float {
                    start_idx = 1; // Skip header
                }
            }
        }

        for (line_num, line) in lines.iter().enumerate().skip(start_idx) {
            let parts: Vec<&str> = line.split(delimiter).map(|s| s.trim()).collect();
            if parts.len() < 2 {
                continue;
            }
            let x = parts[0]
                .parse::<f64>()
                .map_err(|_| format!("Invalid float for x on line {}: '{}'", line_num + 1, parts[0]))?;
            let y = parts[1]
                .parse::<f64>()
                .map_err(|_| format!("Invalid float for y on line {}: '{}'", line_num + 1, parts[1]))?;

            if x.is_finite() && y.is_finite() {
                points.push(Point2D { x, y });
            }
        }

        Ok(points)
    }

    /// Export points to CSV format.
    pub fn export_csv(points: &[Point2D]) -> String {
        let mut out = String::from("x,y\n");
        for p in points {
            out.push_str(&format!("{:.8},{:.8}\n", p.x, p.y));
        }
        out
    }

    /// Serialize project document to formatted JSON.
    pub fn serialize_project(doc: &ProjectDoc) -> Result<String, String> {
        serde_json::to_string_pretty(doc).map_err(|e| format!("Serialization error: {}", e))
    }

    /// Deserialize project document from JSON.
    pub fn deserialize_project(json: &str) -> Result<ProjectDoc, String> {
        serde_json::from_str(json).map_err(|e| format!("Deserialization error: {}", e))
    }

    fn sniff_delimiter(lines: &[&str]) -> char {
        let sample = lines.iter().take(5);
        let mut comma_count = 0;
        let mut tab_count = 0;
        let mut semi_count = 0;

        for line in sample {
            comma_count += line.matches(',').count();
            tab_count += line.matches('\t').count();
            semi_count += line.matches(';').count();
        }

        if tab_count > comma_count && tab_count > semi_count {
            '\t'
        } else if semi_count > comma_count && semi_count > tab_count {
            ';'
        } else {
            ','
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_csv_simple() {
        let csv = "x,y\n1.0,2.0\n3.5,4.5\n";
        let pts = DataIO::parse_csv(csv).unwrap();
        assert_eq!(pts.len(), 2);
        assert_eq!(pts[0], Point2D { x: 1.0, y: 2.0 });
        assert_eq!(pts[1], Point2D { x: 3.5, y: 4.5 });
    }

    #[test]
    fn test_parse_tsv_no_header() {
        let tsv = "10.0\t20.0\n30.0\t40.0\n";
        let pts = DataIO::parse_csv(tsv).unwrap();
        assert_eq!(pts.len(), 2);
        assert_eq!(pts[0], Point2D { x: 10.0, y: 20.0 });
    }

    #[test]
    fn test_project_roundtrip() {
        let doc = ProjectDoc {
            schema_version: 2,
            app_name: "Graph Plotting 2.0".into(),
            mode: "PlotPoints".into(),
            expressions: vec![],
            points: vec![SavedPoint {
                id: "1".into(),
                x: 1.0,
                y: 2.0,
                is_enabled: true,
            }],
            fit_settings: SavedFitSettings {
                selected_fit_type: "Linear".into(),
                use_robust: false,
                show_confidence_band: true,
                show_prediction_band: false,
                show_residuals: false,
            },
            viewport: SavedViewport {
                x_min: -10.0,
                x_max: 10.0,
                y_min: -10.0,
                y_max: 10.0,
            },
        };

        let json = DataIO::serialize_project(&doc).unwrap();
        let loaded = DataIO::deserialize_project(&json).unwrap();
        assert_eq!(loaded.schema_version, 2);
        assert_eq!(loaded.points.len(), 1);
        assert_eq!(loaded.points[0].x, 1.0);
    }
}
