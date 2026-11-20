# Developer and Contributor Guide: Graph Plotting 2.0

## 1. Prerequisites and Build Environment

The system relies on a unified Apple Silicon developer environment.

| Component | Minimum Version | Verification Command | Purpose |
|---|---|---|---|
| **macOS** | 26.0 (Tahoe) | `sw_vers` | Required for Liquid Glass SwiftUI APIs |
| **Xcode Command Line Tools** | 27.0+ (Swift 6.2) | `swiftc --version` | Swift 6 strict concurrency compiler |
| **Rust Toolchain** | 1.84.0+ (Stable) | `rustc --version` | Numerical core and UniFFI codegen |
| **Rust Target** | `aarch64-apple-darwin` | `rustup target list \| grep installed` | Target architecture for Apple Silicon |

### 1.1 Environment Setup
Ensure Rust and the Apple Silicon target are active:
```bash
# Install Rust via rustup if not already present
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Add the native Apple Silicon compilation target
rustup target add aarch64-apple-darwin
```

---

## 2. Compilation and Build Pipeline

### 2.1 The Master Build Script (`scripts/build_rust.sh`)
The repository includes an automated pipeline script that synchronizes the Rust static library, UniFFI Swift bindings, and XCFramework packaging in a single step:

```bash
./scripts/build_rust.sh
```

**Under the Hood**:
1. Compiles `rust/graph-plotting-core` in release mode (`--release --target aarch64-apple-darwin`), producing `libgraph_plotting_core.a`.
2. Invokes the crate's `uniffi-bindgen` binary to generate the Swift bindings (`graph_plotting_core.swift`), C header (`graph_plotting_coreFFI.h`), and modulemap into `macos/GraphPlottingApp/Sources/Bridge/Generated/`.
3. Injects the module import directive into the generated Swift binding so it binds with `GraphPlottingCoreFFI`.
4. Creates a static XCFramework (`GraphPlottingCoreFFI.xcframework`) in the `GraphPlottingCoreFFI` package.

To perform a clean rebuild:
```bash
./scripts/build_rust.sh --clean
```

---

### 2.2 Building and Running the macOS App
Once `build_rust.sh` has executed, the macOS SwiftUI app can be built and run either via the terminal or inside Xcode:

```bash
cd macos/GraphPlottingApp

# Compile the app
swift build

# Run the test suite
swift test

# Launch the app executable
swift run GraphPlottingApp
```

---

## 3. Extending the Numerical Core

### 3.1 Adding a New Empirical Fit Model
To introduce a new model (e.g., Logistic / Sigmoidal: $y = \frac{L}{1 + e^{-k(x - x_0)}} + c$):

1. **Update Enum Definitions (`rust/graph-plotting-core/src/fitting/models.rs`)**:
   - Add the variant to `FitModelKind`:
     ```rust
     pub enum FitModelKind {
         // ... existing models
         Logistic,
     }
     ```
   - Specify metadata:
     - `name(&self) -> &'static str`: Display name.
     - `min_points(&self) -> usize`: Minimum observations required (e.g., 4).
     - Add to `all_models()`.

2. **Implement Evaluation and Jacobian (`models.rs`)**:
   - Add to `eval_fn(kind)`:
     ```rust
     FitModelKind::Logistic => |x, p| p[0] / (1.0 + (-p[1] * (x - p[2])).exp()) + p[3],
     ```
   - Add to `jac_fn(kind)`:
     Provide exact analytical partial derivatives $[\frac{\partial f}{\partial p_0}, \dots, \frac{\partial f}{\partial p_k}]$.

3. **Implement Parameter Initialization (`models.rs`)**:
   - In `fit_raw(kind, x, y)`:
     Implement robust initial heuristic estimates for parameters to seed the Levenberg-Marquardt optimizer.

4. **Register in UniFFI and Auto-Fit**:
   - Add variant to `FitType` in `src/lib.rs`.
   - Update mapping in `From<FitType> for FitModelKind` and `From<FitModelKind> for FitType`.
   - Re-run `./scripts/build_rust.sh`.

---

### 3.2 Extending the Pratt Parser and AST
To add a new mathematical function (e.g., Error Function $\text{erf}(x)$):

1. **Add Token Variant (`src/parser/lexer.rs`)**:
   - Recognize `"erf"` in the lexer keyword matcher, mapping to `Token::Ident("erf")` or dedicated `Token::Erf`.
2. **Update AST and VM (`src/parser/ast.rs`, `src/parser/vm.rs`)**:
   - Add `OpCode::Erf` to the bytecode instruction set.
   - Implement evaluation: calling `libm::erf` or a polynomial Chebyshev approximation.
3. **Symbolic Differentiation (`src/parser/diff.rs`)**:
   - Implement the derivative rule:
     $$\frac{d}{dx}\text{erf}(u) = \frac{2}{\sqrt{\pi}} e^{-u^2} \frac{du}{dx}$$
4. **Interval Arithmetic (`src/interval/mod.rs`)**:
   - Because $\text{erf}(x)$ is strictly monotonic increasing over $\mathbb{R}$:
     $$\text{erf}([a, b]) = [\text{erf}(a), \text{erf}(b)]$$
   - Add the interval extension to prevent false singularity triggers.

---

## 4. Verification and Testing Protocols

### 4.1 Rust Engine Tests
The Rust test suite enforces regression correctness, SVD stability, symbolic differentiation accuracy, and singularity handling:

```bash
cd rust/graph-plotting-core
cargo test
```

### 4.2 Swift Testing Suite
The Swift test suite validates the FFI bridge, document serialization, and UI-layer calculations:

```bash
cd macos/GraphPlottingApp
swift test
```

All contributions must maintain zero test failures and zero compiler warnings under Swift 6 strict concurrency before landing.
