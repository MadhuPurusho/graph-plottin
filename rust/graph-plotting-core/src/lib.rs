use std::sync::Arc;

pub mod parser;
pub mod interval;
pub mod calculus;
pub mod fitting;
pub mod io;
pub mod export;

pub use parser::vm::CompiledExpression;
pub use interval::PolylineStrip;

uniffi::setup_scaffolding!("graph_plotting_core");

// ─────────────────────────────────────────────────────────────────────────────
// UniFFI Exported Data Records & Enums
// ─────────────────────────────────────────────────────────────────────────────

#[derive(uniffi::Record, Clone, Debug, PartialEq)]
pub struct CoreDataPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(uniffi::Enum, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FitType {
    Auto,
    Linear,
    Quadratic,
    Cubic,
    Degree4,
    Degree5,
    Exponential,
    Logarithmic,
    Power,
    Sinusoidal,
    Cosinusoidal,
    Tangential,
    Hyperbolic,
    SquareRoot,
    AbsoluteValue,
    Gaussian,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct CoreFitResult {
    pub fit_type: FitType,
    pub parameters: Vec<f64>,
    pub parameter_names: Vec<String>,
    pub standard_errors: Vec<f64>,
    pub equation: String,
    pub latex_equation: String,
    pub r_squared: f64,
    pub adjusted_r_squared: f64,
    pub rmse: f64,
    pub mae: f64,
    pub aicc: f64,
    pub bic: f64,
    pub loocv_rmse: f64,
    pub residuals: Vec<f64>,
    pub cooks_distances: Vec<f64>,
    pub confidence_band_upper: Vec<CoreDataPoint>,
    pub confidence_band_lower: Vec<CoreDataPoint>,
    pub prediction_band_upper: Vec<CoreDataPoint>,
    pub prediction_band_lower: Vec<CoreDataPoint>,
    pub domain_note: Option<String>,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct FitBundle {
    pub winner: CoreFitResult,
    pub candidates: Vec<CoreFitResult>,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct CoreRootResult {
    pub x: f64,
    pub converged: bool,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct CoreExtremumResult {
    pub x: f64,
    pub y: f64,
    pub is_minimum: bool,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct CoreIntegralResult {
    pub value: f64,
    pub error_estimate: f64,
}

#[derive(uniffi::Error, thiserror::Error, Debug)]
pub enum CoreError {
    #[error("Parse error: {message}")]
    Parse { message: String },
    #[error("Fit error: {message}")]
    Fit { message: String },
    #[error("Calculus error: {message}")]
    Calculus { message: String },
    #[error("IO error: {message}")]
    Io { message: String },
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

impl From<FitType> for fitting::FitModelKind {
    fn from(ft: FitType) -> Self {
        match ft {
            FitType::Linear => fitting::FitModelKind::Linear,
            FitType::Quadratic => fitting::FitModelKind::Quadratic,
            FitType::Cubic => fitting::FitModelKind::Cubic,
            FitType::Degree4 => fitting::FitModelKind::Degree4,
            FitType::Degree5 => fitting::FitModelKind::Degree5,
            FitType::Exponential => fitting::FitModelKind::Exponential,
            FitType::Logarithmic => fitting::FitModelKind::Logarithmic,
            FitType::Power => fitting::FitModelKind::Power,
            FitType::Sinusoidal => fitting::FitModelKind::Sinusoidal,
            FitType::Cosinusoidal => fitting::FitModelKind::Cosinusoidal,
            FitType::Tangential => fitting::FitModelKind::Tangential,
            FitType::Hyperbolic => fitting::FitModelKind::Hyperbolic,
            FitType::SquareRoot => fitting::FitModelKind::SquareRoot,
            FitType::AbsoluteValue => fitting::FitModelKind::AbsoluteValue,
            FitType::Gaussian => fitting::FitModelKind::Gaussian,
            FitType::Auto => fitting::FitModelKind::Linear,
        }
    }
}

impl From<fitting::FitModelKind> for FitType {
    fn from(k: fitting::FitModelKind) -> Self {
        match k {
            fitting::FitModelKind::Linear => FitType::Linear,
            fitting::FitModelKind::Quadratic => FitType::Quadratic,
            fitting::FitModelKind::Cubic => FitType::Cubic,
            fitting::FitModelKind::Degree4 => FitType::Degree4,
            fitting::FitModelKind::Degree5 => FitType::Degree5,
            fitting::FitModelKind::Exponential => FitType::Exponential,
            fitting::FitModelKind::Logarithmic => FitType::Logarithmic,
            fitting::FitModelKind::Power => FitType::Power,
            fitting::FitModelKind::Sinusoidal => FitType::Sinusoidal,
            fitting::FitModelKind::Cosinusoidal => FitType::Cosinusoidal,
            fitting::FitModelKind::Tangential => FitType::Tangential,
            fitting::FitModelKind::Hyperbolic => FitType::Hyperbolic,
            fitting::FitModelKind::SquareRoot => FitType::SquareRoot,
            fitting::FitModelKind::AbsoluteValue => FitType::AbsoluteValue,
            fitting::FitModelKind::Gaussian => FitType::Gaussian,
        }
    }
}

fn to_core_points(packed: &[f32]) -> Vec<CoreDataPoint> {
    let count = packed.len() / 2;
    let mut pts = Vec::with_capacity(count);
    for i in 0..count {
        pts.push(CoreDataPoint {
            x: packed[i * 2] as f64,
            y: packed[i * 2 + 1] as f64,
        });
    }
    pts
}

fn raw_to_core_result(raw: fitting::RawFitResult) -> CoreFitResult {
    let d = raw.diagnostics;
    CoreFitResult {
        fit_type: raw.kind.into(),
        parameters: raw.parameters,
        parameter_names: raw.parameter_names,
        standard_errors: d.standard_errors,
        equation: raw.equation_unicode,
        latex_equation: raw.equation_latex,
        r_squared: d.r_squared,
        adjusted_r_squared: d.adjusted_r_squared,
        rmse: d.rmse,
        mae: d.mae,
        aicc: d.aicc,
        bic: d.bic,
        loocv_rmse: d.loocv_rmse,
        residuals: d.residuals,
        cooks_distances: d.cooks_distances,
        confidence_band_upper: to_core_points(&d.conf_band_upper),
        confidence_band_lower: to_core_points(&d.conf_band_lower),
        prediction_band_upper: to_core_points(&d.pred_band_upper),
        prediction_band_lower: to_core_points(&d.pred_band_lower),
        domain_note: raw.domain_note,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// UniFFI Exported Functions
// ─────────────────────────────────────────────────────────────────────────────

#[uniffi::export]
pub fn parse_expression(input: String) -> Result<Arc<CompiledExpression>, CoreError> {
    let expr = parser::parse_expression(&input).map_err(|e| CoreError::Parse {
        message: e.to_string(),
    })?;
    let compiled = CompiledExpression::compile(&expr, &input);
    Ok(Arc::new(compiled))
}

#[uniffi::export]
pub fn format_expression(input: String) -> Result<String, CoreError> {
    let expr = parser::parse_expression(&input).map_err(|e| CoreError::Parse {
        message: e.to_string(),
    })?;
    Ok(expr.to_string())
}

#[uniffi::export]
pub fn derivative(input: String) -> Result<String, CoreError> {
    let expr = parser::parse_expression(&input).map_err(|e| CoreError::Parse {
        message: e.to_string(),
    })?;
    let diff = parser::differentiate(&expr);
    Ok(diff.to_string())
}

#[uniffi::export]
pub fn second_derivative(input: String) -> Result<String, CoreError> {
    let expr = parser::parse_expression(&input).map_err(|e| CoreError::Parse {
        message: e.to_string(),
    })?;
    let diff2 = parser::differentiate2(&expr);
    Ok(diff2.to_string())
}

#[uniffi::export]
pub fn tessellate(
    expr: Arc<CompiledExpression>,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    viewport_width: u32,
    params: Vec<f64>,
) -> Vec<PolylineStrip> {
    interval::Tessellator::tessellate(
        &expr,
        x_min,
        x_max,
        y_min,
        y_max,
        viewport_width,
        &params,
    )
}

#[uniffi::export]
pub fn fit_curve(
    points: Vec<CoreDataPoint>,
    fit_type: FitType,
    use_robust: bool,
) -> Result<CoreFitResult, CoreError> {
    if points.is_empty() {
        return Err(CoreError::Fit { message: "No data points provided".into() });
    }
    let x: Vec<f64> = points.iter().map(|p| p.x).collect();
    let y: Vec<f64> = points.iter().map(|p| p.y).collect();

    if fit_type == FitType::Auto {
        let (winner, _) = fitting::AutoFitter::auto_fit(&x, &y, use_robust).map_err(|e| CoreError::Fit { message: e })?;
        return Ok(raw_to_core_result(winner));
    }

    let kind: fitting::FitModelKind = fit_type.into();
    let raw = fitting::ModelFitter::fit(kind, &x, &y, use_robust).map_err(|e| CoreError::Fit { message: e })?;
    Ok(raw_to_core_result(raw))
}

#[uniffi::export]
pub fn auto_fit(points: Vec<CoreDataPoint>, use_robust: bool) -> Result<FitBundle, CoreError> {
    if points.is_empty() {
        return Err(CoreError::Fit { message: "No data points provided".into() });
    }
    let x: Vec<f64> = points.iter().map(|p| p.x).collect();
    let y: Vec<f64> = points.iter().map(|p| p.y).collect();

    let (winner, candidates) = fitting::AutoFitter::auto_fit(&x, &y, use_robust).map_err(|e| CoreError::Fit { message: e })?;
    Ok(FitBundle {
        winner: raw_to_core_result(winner),
        candidates: candidates.into_iter().map(raw_to_core_result).collect(),
    })
}

#[uniffi::export]
pub fn find_root(
    expr: Arc<CompiledExpression>,
    guess: f64,
    x_min: f64,
    x_max: f64,
    params: Vec<f64>,
) -> Result<CoreRootResult, CoreError> {
    match calculus::Calculus::find_root_near(&expr, guess, x_min, x_max, &params, 1e-10) {
        Some(res) => Ok(CoreRootResult {
            x: res.x,
            converged: res.converged,
        }),
        None => Err(CoreError::Calculus {
            message: "No root found near guess".into(),
        }),
    }
}

#[uniffi::export]
pub fn find_extrema(
    expr: Arc<CompiledExpression>,
    x_min: f64,
    x_max: f64,
    params: Vec<f64>,
) -> Vec<CoreExtremumResult> {
    let extrema = calculus::Calculus::find_extrema(&expr, x_min, x_max, &params);
    extrema
        .into_iter()
        .map(|e| CoreExtremumResult {
            x: e.x,
            y: e.y,
            is_minimum: e.kind == calculus::ExtremumType::LocalMinimum,
        })
        .collect()
}

#[uniffi::export]
pub fn find_intersections(
    expr1: Arc<CompiledExpression>,
    expr2: Arc<CompiledExpression>,
    x_min: f64,
    x_max: f64,
    params: Vec<f64>,
) -> Vec<f64> {
    calculus::Calculus::find_intersections(&expr1, &expr2, x_min, x_max, &params)
}

#[uniffi::export]
pub fn integrate(
    expr: Arc<CompiledExpression>,
    a: f64,
    b: f64,
    params: Vec<f64>,
) -> Result<CoreIntegralResult, CoreError> {
    let f = |x: f64| expr.eval_internal(x, &params);
    let res = calculus::Calculus::integrate(f, a, b, 1e-9);
    Ok(CoreIntegralResult {
        value: res.value,
        error_estimate: res.error_estimate,
    })
}

#[uniffi::export]
pub fn parse_csv(content: String) -> Result<Vec<CoreDataPoint>, CoreError> {
    let pts = io::DataIO::parse_csv(&content).map_err(|e| CoreError::Io { message: e })?;
    Ok(pts.into_iter().map(|p| CoreDataPoint { x: p.x, y: p.y }).collect())
}

#[uniffi::export]
pub fn export_csv(points: Vec<CoreDataPoint>) -> String {
    let pts: Vec<io::Point2D> = points.into_iter().map(|p| io::Point2D { x: p.x, y: p.y }).collect();
    io::DataIO::export_csv(&pts)
}

#[uniffi::export]
pub fn export_svg(
    strips: Vec<PolylineStrip>,
    points: Vec<CoreDataPoint>,
    conf_upper: Vec<CoreDataPoint>,
    conf_lower: Vec<CoreDataPoint>,
    equation_unicode: String,
    r_squared: Option<f64>,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    width: u32,
    height: u32,
) -> String {
    let pts: Vec<io::Point2D> = points.into_iter().map(|p| io::Point2D { x: p.x, y: p.y }).collect();
    let mut cu = Vec::with_capacity(conf_upper.len() * 2);
    for p in conf_upper {
        cu.push(p.x as f32);
        cu.push(p.y as f32);
    }
    let mut cl = Vec::with_capacity(conf_lower.len() * 2);
    for p in conf_lower {
        cl.push(p.x as f32);
        cl.push(p.y as f32);
    }

    export::PublicationExporter::export_svg(
        &strips,
        &pts,
        &cu,
        &cl,
        &equation_unicode,
        r_squared,
        x_min,
        x_max,
        y_min,
        y_max,
        width,
        height,
    )
}

#[uniffi::export]
pub fn export_latex(
    equation_latex: String,
    points: Vec<CoreDataPoint>,
    r_squared: Option<f64>,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
) -> String {
    let pts: Vec<io::Point2D> = points.into_iter().map(|p| io::Point2D { x: p.x, y: p.y }).collect();
    export::PublicationExporter::export_latex(
        &equation_latex,
        &pts,
        r_squared,
        x_min,
        x_max,
        y_min,
        y_max,
    )
}
