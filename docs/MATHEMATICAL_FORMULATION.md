# Mathematical and Algorithmic Formulations: Graph Plotting 2.0

## 1. Explicit Nonlinear Regression Models

Let $\mathcal{D} = \{(x_i, y_i)\}_{i=1}^n \subset \mathbb{R}^2$ denote a set of $n$ observed pairs. The regression problem seeks a parameter vector $\boldsymbol{\theta} \in \mathbb{R}^p$ that minimizes an objective loss function:

$$S(\boldsymbol{\theta}) = \sum_{i=1}^n \rho\left(y_i - f(x_i; \boldsymbol{\theta})\right)$$

where $\rho(r) = \frac{1}{2}r^2$ under ordinary least squares (OLS), and $f(x; \boldsymbol{\theta})$ represents one of the following 15 explicit model formulations.

---

### 1.1 Polynomial Family (Degree $d \in \{1, 2, 3, 4, 5\}$)

$$f_d(x; \mathbf{a}) = \sum_{j=0}^d a_j x^j$$

- **Linear ($d=1, p=2$)**: $f(x) = a_1 x + a_0$ (represented as $mx + b$)
- **Quadratic ($d=2, p=3$)**: $f(x) = a_2 x^2 + a_1 x + a_0$
- **Cubic ($d=3, p=4$)**: $f(x) = a_3 x^3 + a_2 x^2 + a_1 x + a_0$
- **Degree 4 (Quartic, $d=4, p=5$)**: $f(x) = \sum_{j=0}^4 a_j x^j$
- **Degree 5 (Quintic, $d=5, p=6$)**: $f(x) = \sum_{j=0}^5 a_j x^j$

#### Numerical Stabilization via Centered and Scaled Vandermonde SVD
Direct inversion of the unscaled Vandermonde system $V^T V \mathbf{a} = V^T \mathbf{y}$ suffers from severe condition number degradation: $\kappa(V^T V) = \mathcal{O}(x_{\max}^{2d})$. To prevent catastrophic floating-point cancellation, the core applies an affine coordinate transformation:

$$u_i = \frac{x_i - \mu_x}{\sigma_x}, \quad \mu_x = \frac{1}{n}\sum_{i=1}^n x_i, \quad \sigma_x = \sqrt{\frac{1}{n-1}\sum_{i=1}^n (x_i - \mu_x)^2}$$

The scaled Vandermonde matrix $\tilde{V} \in \mathbb{R}^{n \times (d+1)}$ with $\tilde{V}_{ij} = u_i^j$ is decomposed via Singular Value Decomposition:

$$\tilde{V} = U \Sigma V^T \implies \tilde{\mathbf{a}} = \sum_{k=0}^d \frac{\mathbf{u}_k^T \mathbf{y}}{\sigma_k} \mathbf{v}_k$$

Truncating singular values with $\sigma_k / \sigma_0 < 10^{-14}$. The normalized coefficients $\tilde{\mathbf{a}}$ are subsequently transformed back to the canonical monomial basis $\mathbf{a}$ in $\mathbb{R}[x]$ via binomial expansion:

$$a_k = \sum_{j=k}^d \tilde{a}_j \binom{j}{k} \frac{(-\mu_x)^{j-k}}{\sigma_x^j}$$

---

### 1.2 Growth, Saturation, and Power Models

- **Exponential ($p=2$)**:
  $$f(x; a, b) = a e^{bx}$$
  - *Domain*: $x \in \mathbb{R}$. If $y_i > 0 \, \forall i$, initialized via log-linearization $\ln y_i = \ln a + bx_i$.
  - *Jacobian*: $J = \left[ e^{bx}, \, a x e^{bx} \right]$.

- **Logarithmic ($p=2$)**:
  $$f(x; a, b) = a + b \ln(x)$$
  - *Domain Guard*: $\mathcal{X} = \{x_i \in \mathcal{D} \mid x_i > 0\}$. If $\exists x_i \le 0$, fit is rejected with a formal domain diagnostic.
  - *Jacobian*: $J = \left[ 1, \, \ln(x) \right]$. Solved analytically as a linear system in $u_i = \ln(x_i)$.

- **Power ($p=2$)**:
  $$f(x; a, b) = a x^b$$
  - *Domain Guard*: $x_i > 0, y_i > 0 \, \forall i$.
  - *Initialization*: Ordinary least squares on $\ln y_i = \ln a + b \ln x_i$.
  - *Jacobian*: $J = \left[ x^b, \, a x^b \ln(x) \right]$.

---

### 1.3 Periodic and Trigonometric Models

- **Sinusoidal ($p=4$)**:
  $$f(x; a, b, c, d) = a \sin(b x + c) + d$$
- **Cosinusoidal ($p=4$)**:
  $$f(x; a, b, c, d) = a \cos(b x + c) + d$$
- **Tangential ($p=4$)**:
  $$f(x; a, b, c, d) = a \tan(b x + c) + d$$
  - *Singularity Guard*: During evaluation, if $|\cos(b x_i + c)| < 10^{-3}$, the point is flagged and regularized to avoid numerical divergence.

#### Spectral Initialization via the Lomb-Scargle Periodogram
For unevenly distributed observation coordinates $x_i$, the classical Fast Fourier Transform (FFT) is mathematically inapplicable. The core evaluates the generalized Lomb-Scargle spectral power density $P_{LS}(\omega)$ across a logarithmically spaced frequency grid $\omega \in [\omega_{\min}, \omega_{\max}]$:

$$P_{LS}(\omega) = \frac{1}{2\sigma_y^2} \left[ \frac{\left(\sum_i (y_i - \bar{y}) \cos\omega(x_i - \tau)\right)^2}{\sum_i \cos^2\omega(x_i - \tau)} + \frac{\left(\sum_i (y_i - \bar{y}) \sin\omega(x_i - \tau)\right)^2}{\sum_i \sin^2\omega(x_i - \tau)} \right]$$

where the temporal offset $\tau$ satisfies:

$$\tan(2\omega\tau) = \frac{\sum_i \sin(2\omega x_i)}{\sum_i \cos(2\omega x_i)}$$

The initial angular frequency $b_0$ is assigned to the global spectral argmax:

$$b_0 = \arg\max_{\omega} P_{LS}(\omega)$$

Linear amplitude $a_0$, phase $c_0$, and baseline $d_0$ are subsequently initialized via linear projection of $\sin(b_0 x)$ and $\cos(b_0 x)$.

---

### 1.4 Specialized Non-Linear Formulations

- **Hyperbolic ($p=3$)**:
  $$f(x; a, h, k) = \frac{a}{x - h} + k$$
  - *Singularity Bound*: $|x_i - h| > 10^{-6}$.
  - *Search Space*: Asymptote parameter $h$ is explored through a multi-start grid over the sample quantiles of $x$, refined via VarPro.

- **Square Root ($p=3$)**:
  $$f(x; a, h, k) = a \sqrt{x - h} + k$$
  - *Reparameterization*: $h \le \min(x_i)$ is strictly enforced by parameterizing $h = \min(x_i) - \delta^2$.
  - *Jacobian*: $J = \left[ \sqrt{x - h}, \, \frac{-a}{2\sqrt{x - h}}, \, 1 \right]$.

- **Absolute Value ($p=3$)**:
  $$f(x; a, h, k) = a |x - h| + k$$
  - *Optimization*: Candidate corner locations $h \in \{x_1, \dots, x_n\}$ are evaluated. For each fixed $h$, $(a, k)$ are optimal solutions to a 2D linear regression over $|x_i - h|$. The minimum-loss candidate is chosen as the initialization for subgradient descent.

- **Gaussian ($p=4$)**:
  $$f(x; a, \mu, \sigma, c) = a \exp\left( -\frac{(x - \mu)^2}{2\sigma^2} \right) + c$$
  - *Initialization*:
    $$\mu_0 = \arg\max_{x_i} y_i, \quad a_0 = \max(y_i) - \min(y_i), \quad c_0 = \min(y_i)$$
    $$\sigma_0 = \frac{\text{FWHM}}{2\sqrt{2\ln 2}} \approx \frac{|x_{R} - x_{L}|}{2.355}$$
    where $x_L, x_R$ denote half-maximum intercept coordinates. $\sigma > 0$ is constrained via logarithmic mapping $\sigma = \exp(s)$.

---

## 2. The Golub-Pereyra Variable Projection (VarPro) Algorithm

Many models in Section 1 exhibit linear-nonlinear parameter separability:

$$f(x; \boldsymbol{\alpha}, \boldsymbol{\beta}) = \sum_{j=1}^m \alpha_j \phi_j(x; \boldsymbol{\beta})$$

where $\boldsymbol{\alpha} \in \mathbb{R}^m$ are linear coefficients and $\boldsymbol{\beta} \in \mathbb{R}^k$ are strictly nonlinear parameters.

### 2.1 The Reduced Variable Functional
Let $\Phi(\boldsymbol{\beta}) \in \mathbb{R}^{n \times m}$ denote the matrix with entries $\Phi_{ij}(\boldsymbol{\beta}) = \phi_j(x_i; \boldsymbol{\beta})$. For any fixed $\boldsymbol{\beta}$, the optimal linear parameter vector $\boldsymbol{\alpha}^*(\boldsymbol{\beta})$ is given analytically by the Moore-Penrose pseudoinverse $\Phi^\dagger(\boldsymbol{\beta})$:

$$\boldsymbol{\alpha}^*(\boldsymbol{\beta}) = \Phi^\dagger(\boldsymbol{\beta}) \mathbf{y} = \left(\Phi(\boldsymbol{\beta})^T \Phi(\boldsymbol{\beta})\right)^{-1} \Phi(\boldsymbol{\beta})^T \mathbf{y}$$

Substituting $\boldsymbol{\alpha}^*(\boldsymbol{\beta})$ into the residual objective yields the Variable Projection functional:

$$r_2(\boldsymbol{\beta}) = \mathbf{y} - \Phi(\boldsymbol{\beta})\Phi^\dagger(\boldsymbol{\beta}) \mathbf{y} = \left( I - P_{\Phi(\boldsymbol{\beta})} \right) \mathbf{y}$$

where $P_{\Phi(\boldsymbol{\beta})} = \Phi(\boldsymbol{\beta})\Phi^\dagger(\boldsymbol{\beta})$ is the orthogonal projector onto the range space of $\Phi(\boldsymbol{\beta})$.

### 2.2 Jacobian Computation via Kaufman's Simplification
The exact Fréchet derivative of $r_2(\boldsymbol{\beta})$ involves terms containing the derivative of the projector $P_{\Phi}^\perp$. The core evaluates the Kaufman approximation, which omits the computationally costly projection derivative term while preserving local superlinear convergence:

$$J_{\text{VarPro}}(\boldsymbol{\beta}) = - \left( I - \Phi \Phi^\dagger \right) \left[ \frac{\partial \Phi}{\partial \beta_1} \boldsymbol{\alpha}^*, \, \dots, \, \frac{\partial \Phi}{\partial \beta_k} \boldsymbol{\alpha}^* \right]$$

This reduces the parameter space dimension from $m + k$ down to $k$, eliminating convergence plateaus associated with collinearity between amplitude and curvature parameters.

---

## 3. Robust M-Estimation: Huber Loss via IRLS

To eliminate vulnerability to heavy-tailed noise and measurement outliers, the system provides an optional Huber robust estimator.

### 3.1 Objective Formulation
The Huber loss function $\rho_\delta(e)$ is defined as:

$$\rho_\delta(e) = \begin{cases} 
\frac{1}{2} e^2 & \text{if } |e| \le \delta \\ 
\delta |e| - \frac{1}{2} \delta^2 & \text{if } |e| > \delta 
\end{cases}$$

The scale parameter $\delta$ is estimated dynamically from the median absolute deviation (MAD) of the residuals:

$$\delta = 1.345 \cdot \hat{\sigma}_{\text{MAD}} = 1.345 \cdot \frac{\text{median}(|r_i - \text{median}(r)|)}{0.6745}$$

### 3.2 Iteratively Reweighted Least Squares (IRLS)
Differentiating $\rho_\delta(e)$ with respect to parameters yields equivalent weighted least squares equations with weights:

$$w_i(e_i) = \frac{\rho_\delta'(e_i)}{e_i} = \begin{cases} 
1 & \text{if } |e_i| \le \delta \\ 
\frac{\delta}{|e_i|} & \text{if } |e_i| > \delta 
\end{cases}$$

At iteration $(t+1)$, the parameter update solves:

$$\boldsymbol{\theta}^{(t+1)} = \arg\min_{\boldsymbol{\theta}} \sum_{i=1}^n w_i\left(y_i - f(x_i; \boldsymbol{\theta}^{(t)})\right) \left( y_i - f(x_i; \boldsymbol{\theta}) \right)^2$$

Iterations terminate when $\|\boldsymbol{\theta}^{(t+1)} - \boldsymbol{\theta}^{(t)}\|_\infty < 10^{-6}$ or at $t_{\max} = 6$.

---

## 4. Rigorous Statistical Diagnostics and Model Selection

### 4.1 Covariance Matrix and Parameter Standard Errors
At the converged parameter vector $\hat{\boldsymbol{\theta}}$, the Jacobian $J \in \mathbb{R}^{n \times p}$ is evaluated. The residual variance is estimated by:

$$s^2 = \frac{\sum_{i=1}^n r_i^2}{n - p}$$

The asymptotic parameter covariance matrix $\Sigma_{\hat{\boldsymbol{\theta}}}$ is computed via singular value decomposition of $J$:

$$\Sigma_{\hat{\boldsymbol{\theta}}} = s^2 \left( J^T J \right)^{-1}$$

The standard error $\text{SE}(\hat{\theta}_j)$ for each parameter is:

$$\text{SE}(\hat{\theta}_j) = \sqrt{\left[\Sigma_{\hat{\boldsymbol{\theta}}}\right]_{jj}}$$

---

### 4.2 Goodness-of-Fit Metrics

Let $SST = \sum_{i=1}^n (y_i - \bar{y})^2$ and $SSE = \sum_{i=1}^n r_i^2$.

- **Coefficient of Determination ($R^2$)**:
  $$R^2 = 1 - \frac{SSE}{SST}$$
- **Adjusted $R^2$**:
  $$R^2_{\text{adj}} = 1 - \left[ \frac{(1 - R^2)(n - 1)}{n - p} \right]$$
- **Root Mean Square Error (RMSE)**:
  $$\text{RMSE} = \sqrt{\frac{SSE}{n}}$$
- **Mean Absolute Error (MAE)**:
  $$\text{MAE} = \frac{1}{n} \sum_{i=1}^n |r_i|$$

---

### 4.3 Information Theoretic Criteria (Auto Fit Ranking)

To balance empirical fidelity against parametric complexity and eliminate overfitting:

- **Akaike Information Criterion with Small-Sample Correction ($\text{AIC}_c$)**:
  $$\text{AIC} = n \ln\left(\frac{SSE}{n}\right) + 2p$$
  $$\text{AIC}_c = \text{AIC} + \frac{2p(p + 1)}{n - p - 1}$$
  $\text{AIC}_c$ penalizes additional parameters aggressively when $n / p < 40$.
- **Bayesian Information Criterion ($\text{BIC}$)**:
  $$\text{BIC} = n \ln\left(\frac{SSE}{n}\right) + p \ln(n)$$

---

### 4.4 Analytical Leave-One-Out Cross-Validation (LOOCV)
Refitting $n$ distinct models to evaluate LOOCV is computationally prohibitive for interactive applications ($\mathcal{O}(n \cdot \text{Cost}_{\text{fit}})$). Graph Plotting 2.0 uses the exact algebraic projection theorem.

Let $H = J(J^T J)^{-1} J^T \in \mathbb{R}^{n \times n}$ denote the orthogonal projection (hat) matrix. The diagonal leverage values are:

$$h_{ii} = \mathbf{j}_i^T (J^T J)^{-1} \mathbf{j}_i$$

The exact leave-one-out cross-validation residual $r_{(-i)}$ without refitting is:

$$r_{(-i)} = \frac{r_i}{1 - h_{ii}}$$

The exact LOOCV RMSE is computed in $\mathcal{O}(n)$ time post-fit:

$$\text{RMSE}_{\text{LOOCV}} = \sqrt{\frac{1}{n} \sum_{i=1}^n \left( \frac{r_i}{1 - h_{ii}} \right)^2}$$

---

### 4.5 Cook's Distance Outlier Identification
Cook's distance $D_i$ measures the aggregate displacement of all fitted values when observation $i$ is omitted:

$$D_i = \frac{r_i^2}{p \, s^2} \left( \frac{h_{ii}}{(1 - h_{ii})^2} \right)$$

Observations satisfying $D_i > \frac{4}{n}$ are flagged as high-leverage outliers in the diagnostic inspector.

---

### 4.6 Confidence and Prediction Bands (Working-Hotelling / Scheffé)
For any coordinate $x_0 \in \mathbb{R}$, let $\mathbf{j}_0 = \nabla_{\boldsymbol{\theta}} f(x_0; \hat{\boldsymbol{\theta}})$.

- **95% Confidence Band (Uncertainty of the Mean Function)**:
  $$\hat{y}(x_0) \pm \sqrt{p \cdot F_{0.05, p, n-p}} \cdot s \sqrt{\mathbf{j}_0^T (J^T J)^{-1} \mathbf{j}_0}$$
- **95% Prediction Band (Uncertainty of a Future Observation)**:
  $$\hat{y}(x_0) \pm \sqrt{p \cdot F_{0.05, p, n-p}} \cdot s \sqrt{1 + \mathbf{j}_0^T (J^T J)^{-1} \mathbf{j}_0}$$

---

## 5. Topological Interval Arithmetic and Singularity Splitting

### 5.1 Moore Interval Arithmetic
Real intervals $X = [\underline{x}, \overline{x}]$ are evaluated with directed rounding:

$$[\underline{x}, \overline{x}] + [\underline{y}, \overline{y}] = [\underline{x} + \underline{y}, \, \overline{x} + \overline{y}]$$
$$[\underline{x}, \overline{x}] - [\underline{y}, \overline{y}] = [\underline{x} - \overline{y}, \, \overline{x} - \underline{y}]$$
$$[\underline{x}, \overline{x}] \cdot [\underline{y}, \overline{y}] = [\min(\underline{x}\underline{y}, \underline{x}\overline{y}, \overline{x}\underline{y}, \overline{x}\overline{y}), \, \max(\underline{x}\underline{y}, \underline{x}\overline{y}, \overline{x}\underline{y}, \overline{x}\overline{y})]$$
$$\frac{1}{[\underline{y}, \overline{y}]} = \begin{cases} 
[1/\overline{y}, \, 1/\underline{y}] & \text{if } 0 \notin [\underline{y}, \overline{y}] \\ 
[-\infty, \infty] & \text{if } 0 \in [\underline{y}, \overline{y}] \text{ (Singularity Triggered)} 
\end{cases}$$

### 5.2 Adaptive Subdivision and Pole Isolation
Given a viewport domain $[x_{\min}, x_{\max}]$:
1. The domain is partitioned into initial uniform subintervals $I_k = [x_k, x_{k+1}]$.
2. For each subinterval, the interval extension $F(I_k)$ is evaluated.
3. If $F(I_k) = [-\infty, \infty]$ or $\text{width}(F(I_k)) > 10 \cdot |y_{\max} - y_{\min}|$, the interval contains an asymptotic discontinuity or unbounded derivative.
4. The algorithm bisects $I_k$ recursively up to depth $D_{\max} = 8$ to isolate the pole location $x^* \pm \epsilon$.
5. The polyline strip is split immediately at the boundary of the pole, terminating the incoming curve and starting a new strip after the singularity.

---

## 6. Numerical Calculus Engines

### 6.1 Hybrid Brent-Dekker Root Finding
Finds solutions to $f(x) = 0$ on $[a, b]$ where $f(a)f(b) \le 0$. The algorithm combines:
1. **Bisection**: Guarantees linear convergence in pathological cases ($\text{step} = \frac{b - a}{2}$).
2. **Secant Method**: Accelerates convergence near linear regimes.
3. **Inverse Quadratic Interpolation (IQI)**: Uses three points $(a, f(a)), (b, f(b)), (c, f(c))$:
   $$x = \frac{f(b)f(c)a}{(f(a)-f(b))(f(a)-f(c))} + \frac{f(a)f(c)b}{(f(b)-f(a))(f(b)-f(c))} + \frac{f(a)f(b)c}{(f(c)-f(a))(f(c)-f(b))}$$
IQI is accepted only if the interpolated root lies strictly within the current bracketing bounds and the step length shrinks by at least a factor of two every two iterations, guaranteeing asymptotic order of convergence $\alpha \approx 1.618$ while retaining unconditional bisection stability.

---

### 6.2 Adaptive Gauss-Kronrod ($G_7/K_{15}$) Quadrature
Computes definite integrals $I = \int_a^b f(x) dx$.

On each subinterval $[\alpha, \beta]$, the algorithm evaluates both:
- A 7-point Gauss rule:
  $$G_7 = \sum_{j=1}^4 w_j^G \left[ f(x_j) + f(-x_j) \right]$$
- An embedded 15-point Kronrod extension:
  $$K_{15} = \sum_{j=1}^8 w_j^K \left[ f(x_j') + f(-x_j') \right]$$
All 7 nodes of $G_7$ are reused within $K_{15}$, minimizing function evaluations.

The truncation error estimate $E$ is:

$$E = |K_{15} - G_7|$$

If $E > \text{tol} = 10^{-10} \cdot |K_{15}| + 10^{-12}$, the subinterval $[\alpha, \beta]$ is split at its midpoint and evaluated recursively, guaranteeing 15-digit precision for arbitrary non-singular integrals.
