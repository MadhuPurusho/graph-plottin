# Scope, Boundaries, and Limitations: Graph Plotting 2.0

## 1. Scope of Applicability: What the Software Is and Is Not

Scientific integrity requires precise delimitation of an analytical tool's operational envelope. Graph Plotting 2.0 is engineered for specific mathematical domains and explicitly disclaims suitability for others.

### 1.1 What the Software Is
- **An Empirical 2D Regression Workbench**: A tool for fitting continuous parametric models $f(x; \boldsymbol{\theta})$ to planar observation pairs $(x_i, y_i) \in \mathbb{R}^2$.
- **A Deterministic Interactive Graphing Calculator**: A high-framerate (120 FPS), GPU-accelerated engine for evaluating, differentiating, and tessellating real univariate functions $f: \mathbb{R} \to \mathbb{R}$.
- **A Classical Statistical Diagnostic Suite**: An implementation of OLS and M-estimation inference, providing covariance matrices, information criteria ($\text{AIC}_c$, $\text{BIC}$), analytical cross-validation, and outlier detection for planar regression models.

### 1.2 What the Software Is Not
- **Not an Arbitrary-Precision Computer Algebra System (CAS)**: It is not a replacement for symbolic environments like Wolfram Mathematica, Maple, or SageMath. Symbolic differentiation is supported for elementary functional forms, but symbolic integration, non-elementary special functions, and arbitrary-precision integer/rational arithmetic are outside the system's design scope.
- **Not a Multivariate or High-Dimensional Regressor**: The engine does not process hypersurfaces $z = f(x_1, x_2, \dots, x_k)$ or multidimensional tensor fields. All operations are strictly defined over the real plane $\mathbb{R}^2$.
- **Not a Black-Box Machine Learning Framework**: It does not train deep neural networks, decision forests, or non-parametric kernel methods. Every supported model represents an explicit, interpretable closed-form equation with physical or geometric meaning.
- **Not an Automated Global Optimization Prover**: It cannot mathematically guarantee that a converged non-linear regression solution is the global minimum of the parameter space.

<p align="center">
  <img src="images/functions_mode.png" alt="Interactive Univariate Function Visualization" width="880">
</p>

---

## 2. Non-Convex Optimization and Local Minima Vulnerability

### 2.1 The Non-Convex Parameter Landscape
Except for the polynomial family ($d \in \{1, 2, 3, 4, 5\}$) and the logarithmic model (which admit strictly convex quadratic loss surfaces), non-linear least squares objectives:

$$S(\boldsymbol{\theta}) = \frac{1}{2} \sum_{i=1}^n \left( y_i - f(x_i; \boldsymbol{\theta}) \right)^2$$

are non-convex. The parameter loss surface contains multiple local minima, saddle points, and flat valleys.

```
Loss S(θ)
   ▲
   │        Local Minimum
   │          (Trap)                   Global Minimum
   │           ╭─╮                        (Target)
   │          ╭╯ ╰╮                         ╭─╮
   │         ╭╯   ╰╮                       ╭╯ ╰╮
   │    ────╯      ╰──────────────────────╯   ╰────
   └─────────────────────────────────────────────────► Parameter θ
```

### 2.2 Heuristic Initialization Boundaries
The engine uses robust analytical and spectral initializers (e.g., Lomb-Scargle periodograms for periodic models, FWHM estimators for Gaussians, and VarPro linear projection). However:
- **Phase Wrapping & Cycle Skipping**: In sinusoidal fitting ($y = a\sin(bx + c) + d$), if the observation sampling rate violates the Nyquist-Shannon sampling criterion ($\Delta x > \frac{\pi}{b}$), aliasing occurs. The periodogram will identify a spurious alias frequency, and Levenberg-Marquardt will converge to an incorrect harmonic.
- **Asymptote Location in Hyperbolic Models**: In $y = \frac{a}{x - h} + k$, if observations do not bracket the singularity $x \approx h$, the objective function becomes extremely flat with respect to $h$, resulting in an ill-conditioned Jacobian and potential failure to converge.
- **Multi-Modal Gaussian Distributions**: If data contains multiple overlapping Gaussian peaks, the single-peak Gaussian model ($p=4$) will fit to the dominant peak or find an unphysical intermediate dispersion parameter $\sigma$ covering multiple modes.

---

## 3. Ill-Conditioning and Numerical Stability Boundaries

### 3.1 The Vandermonde Barrier and Polynomial Overfitting
The system strictly bounds polynomial regression to degree $d \le 5$. This limitation is deliberate and mathematically necessary:
1. **Condition Number Degradation**: The condition number $\kappa(V)$ of an uncentered Vandermonde matrix on $[0, 1]$ grows as $\mathcal{O}\left((1 + \sqrt{2})^{2d}\right)$. Even with the affine centering and scaling transformation implemented in the core:
   $$u_i = \frac{x_i - \mu}{\sigma}$$
   polynomials of degree $d > 5$ become hyper-sensitive to numerical roundoff during canonical coefficient reconstruction.
2. **Runge's Phenomenon**: For high-degree polynomials fitted over equispaced data, polynomial interpolation oscillates wildly near the interval endpoints:
   $$\lim_{d \to \infty} \max_{x \in [-1, 1]} |f(x) - f_d(x)| = \infty \quad \text{for } f(x) = \frac{1}{1 + 25x^2}$$
   Degree 5 represents the highest safe degree for empirical polynomial modeling without regularization penalties.

### 3.2 Underdetermined Regimes ($n < p$)
Every model requires a strictly minimal sample size $n \ge p$:
- Linear: $n \ge 2$
- Quadratic, Hyperbolic, Square Root, Absolute Value: $n \ge 3$
- Cubic, Sinusoidal, Cosinusoidal, Tangential, Gaussian: $n \ge 4$
- Degree 4: $n \ge 5$
- Degree 5: $n \ge 6$

When $n < p$, the parameter system is underdetermined, $J^T J$ is singular, and regression is rejected. The engine does not inject arbitrary L2 (Ridge) priors unless explicitly requested, avoiding subjective bias in scientific results.

---

## 4. Floating-Point and Hardware Precision Limits

### 4.1 IEEE 754 Double-Precision Realities (`f64`)
The Rust core operates entirely on IEEE 754 binary64 floating-point values:
- Mantissa: 53 bits ($\approx 15.95$ decimal digits)
- Exponent range: $10^{-308}$ to $10^{+308}$
- Machine epsilon: $\epsilon_{\text{mach}} = 2^{-52} \approx 2.2204 \times 10^{-16}$

**Failure Modes**:
- **Catastrophic Cancellation**: Evaluating $f(x) = \sqrt{x+1} - \sqrt{x}$ for $x > 10^{15}$ produces zero due to mantissa exhaustion.
- **Extreme Range Ratios**: Datasets combining astronomical values ($x \approx 10^{20}$) with infinitesimal measurements ($y \approx 10^{-20}$) will suffer from precision loss during sums of squares accumulation ($\sum y_i^2$). Users are advised to normalize or nondimensionalize data before regression.

### 4.2 Metal GPU 32-Bit Coordinate Truncation
While the Rust core evaluates geometry in 64-bit precision (`f64`), Metal vertex buffers ingest 32-bit floats (`f32`) for hardware rasterization efficiency:
- Mantissa: 24 bits ($\approx 7.22$ decimal digits).
- **Subpixel Jitter Threshold**: When zooming into viewport spans smaller than $\Delta x < 10^{-6}$, 32-bit float resolution limits become visible as line staircasing or jitter on screen.

---

## 5. Quadrature and Singularity Failure Modes

### 5.1 Essential and Non-Integrable Singularities
The adaptive Gauss-Kronrod ($G_7/K_{15}$) integrator assumes the integrand is piecewise continuous and Riemann-integrable on $[a, b]$:
- **Non-Integrable Poles**: $\int_0^1 \frac{1}{x} dx$ diverges. The adaptive integrator will exhaust its maximum recursion depth ($D_{\max} = 50$) and return an explicit `CalculusError::ConvergenceFailure` diagnostic.
- **Rapidly Oscillating Integrands**: Functions such as $f(x) = \sin(10^7 x)$ or $f(x) = \sin(1/x)$ near $x=0$ have infinite variation. The quadrature nodes will miss oscillations, resulting in substantial integration error estimates.

### 5.2 Removable Discontinuities
Functions with isolated removable discontinuities, such as $f(x) = \frac{\sin x}{x}$ at $x = 0$, evaluate to `0.0 / 0.0 = NaN` in raw floating-point arithmetic. While interval arithmetic handles neighborhood bounds, users must supply continuous limits or reparameterizations for points of indeterminacy.

---

## 6. Dataset Scale and Memory Hierarchy

| Parameter | Recommended Range | Hard Operational Limit | Degradation Mode |
|---|---|---|---|
| Sample Size ($n$) | $2 \le n \le 10^5$ | $n = 5 \times 10^6$ | SVD memory allocation ($n \times p$ matrices); increased frame latency |
| Expression Complexity | AST depth $\le 20$ | AST depth $= 128$ | Stack overflow during recursive AST differentiation |
| Number of Free Parameters | $k \le 8$ | $k = 32$ | Exponential growth of parameter exploration space |
| Viewport Zoom Range | $10^{-5} \le \Delta x \le 10^8$ | $10^{-14} \le \Delta x \le 10^{14}$ | Floating-point underflow/overflow in dynamic 1-2-5 grid shader |
| Active Function Expressions | $1 \le m \le 16$ | $m = 64$ | GPU fill-rate saturation at 120 FPS |
