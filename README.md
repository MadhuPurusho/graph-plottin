# Graph Plotting 2.0

A native macOS app for interactive graph plotting and curve fitting, built with
**Rust** (core engine) + **SwiftUI** + **Metal**.

> Version 2.0 is a complete rewrite of the original Electron prototype as a
> first-class Apple-silicon application with GPU-accelerated rendering.

---

## Architecture

```
┌───────────────────────────────────────────────────┐
│                   SwiftUI Shell                   │
│  NavigationSplitView · Inspector · Liquid Glass   │
├───────────────────────────────────────────────────┤
│              Metal Rendering Layer                │
│  MSL shaders · grid · curves · points · bands     │
├───────────────────────────────────────────────────┤
│            UniFFI Swift Bindings                  │
│  Generated .swift + C header + XCFramework        │
├───────────────────────────────────────────────────┤
│            graph-plotting-core (Rust)             │
│  Expression parser · Interval tessellator         │
│  15-type curve fitter · Calculus engine           │
│  CSV/TSV & .gplot I/O · SVG/LaTeX export          │
└───────────────────────────────────────────────────┘
```

### Directory Layout

```
Graph Plottin 2/
├── rust/graph-plotting-core/   # Rust library crate
│   └── src/
│       ├── parser/             # Lexer, Pratt parser, AST, symbolic diff, bytecode VM
│       ├── fitting/            # 15 fit types + Auto, VarPro, diagnostics
│       ├── interval/           # Interval arithmetic & adaptive tessellation
│       ├── calculus/           # Brent root-finding, adaptive G7K15 integration
│       ├── io/                 # CSV/TSV sniffing, .gplot JSON schema
│       ├── export/             # SVG plotter, LaTeX/TikZ export
│       └── lib.rs              # UniFFI proc-macro bridge
├── macos/GraphPlottingApp/     # SwiftUI macOS app (Swift 6.1)
│   └── Sources/
│       ├── App/                # @main, DocumentGroup, AppIntents
│       ├── Models/             # @Observable state, data types
│       ├── Views/              # ContentView, Metal graph, overlays, inspectors
│       ├── Metal/              # Compute coordinator, pipeline setup
│       └── Bridge/             # Mock → Generated UniFFI bindings
├── GraphPlottingCoreFFI/       # Local Swift package wrapping the XCFramework
├── scripts/
│   └── build_rust.sh           # One-command build & binding generation
├── .gitignore
└── README.md
```

---

## Features

### Expression Plotting
- Full math expression parser with implicit multiplication (`2x`, `sin(x)cos(x)`)
- 40+ built-in functions: trig, hyperbolic, logarithmic, rounding, gamma, …
- Constants: `π`, `τ`, `e`
- Symbolic differentiation (f′, f″) with algebraic simplification
- Bytecode VM for fast per-pixel evaluation
- Interval-arithmetic adaptive tessellation (auto discontinuity detection)

### Curve Fitting (15 Types + Auto)

| #  | Fit Type       | Description                        |
|----|----------------|------------------------------------|
| 1  | Linear         | y = ax + b                         |
| 2  | Quadratic      | y = ax² + bx + c                   |
| 3  | Cubic          | y = ax³ + bx² + cx + d             |
| 4  | Degree 4       | Quartic polynomial                 |
| 5  | Degree 5       | Quintic polynomial                 |
| 6  | Exponential    | y = a·eᵇˣ + c                     |
| 7  | Logarithmic    | y = a·ln(x) + b                   |
| 8  | Power          | y = a·xᵇ                          |
| 9  | Sinusoidal     | y = a·sin(bx + c) + d             |
| 10 | Cosinusoidal   | y = a·cos(bx + c) + d             |
| 11 | Tangential     | y = a·tan(bx + c) + d             |
| 12 | Hyperbolic     | y = a/(x − h) + k                 |
| 13 | Square Root    | y = a·√(x − h) + k                |
| 14 | Absolute Value | y = a·|x − h| + k                 |
| 15 | Gaussian       | y = a·exp(−(x − μ)²/(2σ²)) + d    |

**Auto Fit** races all 15 models in parallel with Rayon and ranks by AICc
(tie-break: LOOCV RMSE).

Each fit reports: parameters ± SE, R², Adjusted R², RMSE, MAE, AICc, BIC,
LOOCV RMSE, residuals, Cook's distance, and 95% confidence/prediction bands.
Robust Huber IRLS is available for outlier-resistant fitting.

### Calculus
- **Root finding** — Brent's method
- **Numerical integration** — Adaptive Gauss 7-point / Kronrod 15-point

### Data I/O
- CSV / TSV import with automatic delimiter sniffing
- Native `.gplot` JSON document format (FileDocument)

### Export
- SVG with styled axes, curves, and data points
- LaTeX / TikZ for publication-ready figures

### Rendering
- Metal shaders: grid, curve polylines, scatter points, confidence bands
- 120 fps capable (`enableSetNeedsDisplay` for power efficiency)
- Liquid Glass overlays for equation display, viewport pill, and probe HUD

---

## Prerequisites

| Tool   | Minimum Version | Notes                              |
|--------|-----------------|------------------------------------|
| macOS  | 26+             | Required for Liquid Glass APIs     |
| Xcode  | 27+             | Swift 6.1, Metal                   |
| Rust   | 1.75+           | Install via [rustup.rs](https://rustup.rs) |

---

## Building

### 1. Build the Rust core and generate bindings

```bash
./scripts/build_rust.sh
```

This will:
1. Compile the Rust static library for `aarch64-apple-darwin` (release)
2. Generate UniFFI Swift bindings into `macos/GraphPlottingApp/Sources/Bridge/Generated/`
3. Create an XCFramework at `GraphPlottingCoreFFI/GraphPlottingCoreFFI.xcframework`

### 2. Build and run the macOS app

```bash
cd macos/GraphPlottingApp
swift build
swift run GraphPlottingApp    # or open in Xcode
```

### Run tests

```bash
source "$HOME/.cargo/env"
cd rust/graph-plotting-core
cargo test
```

### Clean build

```bash
./scripts/build_rust.sh --clean
```

---

## V1 (Legacy)

The original Electron prototype lives alongside in the repo root
(`index.html`, `main.js`, `renderer.js`). To run it:

```bash
npm install
npm start
```

---

## License

Private — all rights reserved.
