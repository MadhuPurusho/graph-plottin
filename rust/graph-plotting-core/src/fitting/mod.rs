pub mod varpro;
pub mod stats;
pub mod models;
pub mod auto;

pub use models::{FitModelKind, ModelFitter, RawFitResult};
pub use stats::{compute_diagnostics, huber_weights, ModelDiagnostics};
pub use auto::AutoFitter;
