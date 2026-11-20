# Graph Plotting 2.0: A Native High-Performance Scientific Workbench for macOS

[![Rust](https://img.shields.io/badge/Rust-1.84%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![Swift](https://img.shields.io/badge/Swift-6.2%20(macOS%2026%2B)-F05138?logo=swift)](https://developer.apple.com/swift/)
[![Metal](https://img.shields.io/badge/Metal-3.0%20(120%20FPS)-007AFF?logo=apple)](https://developer.apple.com/metal/)
[![Tests](https://img.shields.io/badge/Tests-78%20Passed%20(100%25)-brightgreen)](#test-suite-and-numerical-verification)
[![License](https://img.shields.io/badge/License-Proprietary-red)](#license)

Graph Plotting 2.0 is a native macOS scientific computing application designed for empirical non-linear regression, mathematical function exploration, and publication-grade figure generation. It combines a mathematically rigorous numerical engine implemented in pure Rust with a 120 FPS ProMotion Metal rendering pipeline and a native SwiftUI interface conforming to Apple's Human Interface Guidelines and Liquid Glass design system.

---

## Technical Documentation Index

For exhaustive theoretical derivations, empirical benchmarks, and architectural specifications, consult the dedicated technical monographs:

- **[System Architecture & Concurrency Contract (`docs/ARCHITECTURE.md`)](docs/ARCHITECTURE.md)**: Zero-IPC FFI bridge, memory safety guarantees, Swift 6 strict concurrency isolation, and Metal 3.0 graphics pipeline.
- **[Mathematical & Algorithmic Formulation (`docs/MATHEMATICAL_FORMULATION.md`)](docs/MATHEMATICAL_FORMULATION.md)**: Complete mathematical formulations of all 15 regression models, the Golub-Pereyra Variable Projection (VarPro) algorithm, Lomb-Scargle spectral initialization, Huber M-estimation, analytical LOOCV leverage theorems, and Moore interval arithmetic.
- **[Scope, Boundaries & Scientific Limitations (`docs/LIMITATIONS_AND_BOUNDS.md`)](docs/LIMITATIONS_AND_BOUNDS.md)**: An honest disclosure of operational boundaries: non-convexity, local minima traps, condition number degradation, floating-point limits, and what the software is and is not.
- **[Empirical Benchmarks & Validation (`docs/BENCHMARKS_AND_VALIDATION.md`)](docs/BENCHMARKS_AND_VALIDATION.md)**: Parameter recovery validation against synthetic ground-truth and SciPy MINPACK baselines, throughput profiles, and GPU render pass latency.
- **[Developer & Contributor Guide (`docs/DEVELOPER_GUIDE.md`)](docs/DEVELOPER_GUIDE.md)**: Toolchain requirements, building the static XCFramework, extending the AST/VM grammar, and contributing new fit models.

---

## Core Capabilities

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            Graph Plotting 2.0                               │
│                                                                             │
│  ┌───────────────────────┐ ┌─────────────────────────────────────────────┐  │
│  │   Navigation Sidebar  │ │             Metal Canvas Area               │  │
│  │ ───────────────────── │ │ ─────────────────────────────────────────── │  │
│  │ • 15 Regression Types │ │ • Dynamic 1-2-5 x 10^k Cartesian Grid (MSL) │  │
│  │ • Smart Auto-Fit Race │ │ • Anti-aliased Curve Polylines              │  │
│  │ • Point Data Table    │ │ • 95% Confidence & Prediction Bands         │  │
│  │ • Multi-Function AST  │ │ • Instanced Point Billboard Circles         │  │
│  │ • Free Param Sliders  │ │ • Liquid Glass Equation & R² Overlay Card   │  │
│  └───────────────────────┘ │ • Magnetic Hover Probe HUD (20px snap)      │  │
│  ┌───────────────────────┐ │ • Floating Viewport Navigation Pill         │  │
│  │  Inspector (Right)    │ └─────────────────────────────────────────────┘  │
│  │ ───────────────────── │                                                  │
│  │ • Exact R², Adj R²    │ • Publication Vector SVG Export                  │
│  │ • RMSE, MAE, AICc, BIC│ • Publication LaTeX / PGFPlots Export            │
│  │ • Parameter ± SE      │ • Fast Delimiter-Sniffing CSV / TSV I/O          │
│  │ • Cook's Distance     │ • Native .gplot JSON Document System             │
│  │ • Brent Root Finder   │                                                  │
│  │ • Extrema & G7K15 Int │                                                  │
│  └───────────────────────┘                                                  │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1. Empirical Curve Fitting (15 Explicit Models + Smart Auto Fit)
The system implements exact analytical and Levenberg-Marquardt optimization routines across 15 standard scientific models:
- **Polynomial**: Linear ($y=mx+b$), Quadratic ($y=ax^2+bx+c$), Cubic ($y=ax^3+bx^2+cx+d$), Quartic ($d=4$), and Quintic ($d=5$) solved via centered and scaled Vandermonde Singular Value Decomposition (SVD).
- **Growth & Power**: Exponential ($y=ae^{bx}$), Logarithmic ($y=a+b\ln x$, guarded for $x>0$), and Power ($y=ax^b$).
- **Periodic**: Sinusoidal ($y=a\sin(bx+c)+d$), Cosinusoidal ($y=a\cos(bx+c)+d$), and Tangential ($y=a\tan(bx+c)+d$), initialized via discrete Lomb-Scargle periodograms.
- **Specialized Nonlinear**: Hyperbolic ($y=\frac{a}{x-h}+k$), Square Root ($y=a\sqrt{x-h}+k$), Absolute Value ($y=a|x-h|+k$), and Gaussian ($y=a\exp(-\frac{(x-\mu)^2}{2\sigma^2})+c$).
- **Auto Fit (`FitType::Auto`)**: Races all 15 models simultaneously across CPU cores with Rayon, eliminates underdetermined candidates, and ranks models strictly by **Akaike Information Criterion with small-sample correction ($\text{AIC}_c$)**, broken by **LOOCV RMSE**.
- **Inference Suite**: Returns asymptotic parameter covariance matrices, parameter standard errors ($\pm\text{SE}$), Cook's distance leverage metrics ($D_i > \frac{4}{n}$), Working-Hotelling 95% confidence bands, 95% prediction bands, and optional Huber iteratively reweighted least squares (IRLS) robust regression.

### 2. High-Performance Function Engine & Calculus Suite
- **Pratt Parser**: Evaluates mathematical expressions with source-span error tracking, automatic implicit multiplication (`2x`, `(x+1)(x-2)`, `2sin(x)`), and arbitrary free parameters with real-time UI sliders.
- **Symbolic Differentiation**: Derives exact expressions for $f'(x)$ and $f''(x)$ across all elementary functions with algebraic simplification ($u \cdot 0 \to 0, u \cdot 1 \to u, \text{constant folding}$).
- **Bytecode Virtual Machine**: Evaluates expressions at $>1.8 \times 10^7$ evaluations/sec without AST tree-walking.
- **Topological Interval Tessellation**: Uses Moore interval arithmetic ($[a, b]$) to isolate poles ($\tan x, \frac{1}{x}$) and domain cuts ($\ln x, \sqrt{x}$), eliminating vertical jump artifacts across discontinuities.
- **Numerical Calculus**: Brent-Dekker hybrid root finding ($f(x)=0$), critical point local extrema classification ($f'(x)=0$ with $f''(x)$ sign test), and adaptive 15-point Gauss-Kronrod quadrature ($G_7/K_{15}$) for definite integration $\int_a^b f(x)dx$.

### 3. ProMotion Metal Graphics Architecture
- **120 FPS Event-Driven Canvas**: Operates with `MTKView.isPaused = true` and `enableSetNeedsDisplay = true`, consuming zero GPU energy when idle.
- **Metal Shading Language Pipelines**: Fullscreen signed distance field (SDF) grid ($1\text{–}2\text{–}5 \times 10^k$ decade scaling), anti-aliased polyline strips, confidence band triangle strips, and instanced circle billboard markers.
- **Natural Trackpad Interaction**: Seamless inertial 2-finger panning, cursor-centered pinch-to-zoom, and magnetic probe snapping to curve vertices within 20 screen pixels.

---

## System Requirements

| Environment | Requirement |
|---|---|
| **Operating System** | macOS 26.0 (Tahoe) or newer |
| **Architecture** | Apple Silicon (`arm64` / `aarch64-apple-darwin`) |
| **Compiler Toolchain** | Xcode 27+ (Swift 6.2 strict concurrency), Rust 1.84+ |
| **Graphics API** | Metal 3.0 |

---

## Quick Start & Verification

### 1. Build the Rust Static Core and UniFFI Bridge
```bash
./scripts/build_rust.sh
```
This compiles the Rust core in release mode, generates the UniFFI Swift bindings, and creates `GraphPlottingCoreFFI.xcframework`.

### 2. Run the Verification Test Suite
```bash
# 1. Execute the pure Rust unit test suite (67 tests)
cd rust/graph-plotting-core
cargo test

# 2. Execute the Swift integration and FFI test suite (11 tests)
cd ../../macos/GraphPlottingApp
swift test
```

### 3. Launch the Application
```bash
cd macos/GraphPlottingApp
swift run GraphPlottingApp
```
Or open `macos/GraphPlottingApp/Package.swift` directly in Xcode.

---

## Test Suite and Numerical Verification

The repository enforces complete regression coverage across both layers:

```text
Suite                             Tests Run   Passed   Failed   Duration
────────────────────────────────────────────────────────────────────────
Rust Core Numerical Engine           67         67       0       0.02s
Swift Integration & FFI Suite        11         11       0       0.01s
────────────────────────────────────────────────────────────────────────
Total Verification Suite             78         78       0       0.03s
```

All 78 tests pass deterministically with zero warnings or errors.

---

## Project Structure

```text
.
├── docs/                                  # Research-grade technical documentation
│   ├── ARCHITECTURE.md                    # Core architecture & concurrency model
│   ├── MATHEMATICAL_FORMULATION.md        # Explicit mathematical derivations
│   ├── LIMITATIONS_AND_BOUNDS.md          # Scientific boundaries & failure modes
│   ├── BENCHMARKS_AND_VALIDATION.md       # Empirical benchmarks vs. SciPy/R
│   └── DEVELOPER_GUIDE.md                 # Contributor and developer guide
├── GraphPlottingCoreFFI/                  # Local Swift Package wrapping XCFramework
│   ├── GraphPlottingCoreFFI.xcframework   # Binary static framework (aarch64)
│   └── Package.swift                      # SPM binary target specification
├── macos/GraphPlottingApp/                # Native SwiftUI application
│   ├── Package.swift                      # Swift 6.2 package specification
│   ├── Sources/
│   │   ├── App/                           # Entry point, DocumentGroup, AppIntents
│   │   ├── Bridge/                        # UniFFI Swift glue & namespace bridge
│   │   ├── Metal/                         # Runtime-compiled Metal Shading Language
│   │   ├── Models/                        # @Observable AppState, ViewportState
│   │   └── Views/                         # MetalGraphView, Overlays, Inspectors
│   └── Tests/GraphPlottingAppTests/       # Swift Testing integration suite
├── rust/graph-plotting-core/              # Pure Rust numerical engine
│   ├── Cargo.toml                         # Crate dependencies (nalgebra, LM, uniffi)
│   └── src/
│       ├── calculus/                      # Brent roots, Gauss-Kronrod G7K15
│       ├── export/                        # Publication vector SVG & LaTeX/TikZ
│       ├── fitting/                       # 15 regression models, VarPro, Rayon auto-fit
│       ├── interval/                      # Moore interval arithmetic & tessellation
│       ├── io/                            # CSV/TSV sniffing, .gplot schema
│       ├── parser/                        # Pratt parser, AST, symbolic diff, VM
│       └── lib.rs                         # UniFFI proc-macro scaffolding
├── scripts/
│   └── build_rust.sh                      # Reproducible master build pipeline
├── .gitignore                             # Git ignore rules
└── README.md                              # Main project documentation
```

---

## License

Proprietary — All rights reserved.
