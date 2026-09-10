# graph-plottin

a graphing tool built with electron. plot points, fit curves, or just type in a function and see it drawn out.

---

## what it does

**plot points mode**

add x and y coordinates and the app will automatically fit a curve to them. you can also pick the fit type yourself if you know what shape your data should be.

supported fit types:
- linear, quadratic, cubic, degree 4, degree 5
- exponential, logarithmic, power
- sinusoidal, cosinusoidal, tangential
- hyperbolic, square root, absolute value, gaussian

after fitting, the equation and r² value are shown in the sidebar and overlaid on the graph.

**function mode**

type any expression like `sin(x) * x` or `x^2 - 4` and it gets plotted directly. set your own x range too.

---

## the math keyboard

clicking the keyboard icon next to any input opens a popup keyboard with:
- constants: π, φ (golden ratio), e, ∞
- operations: √, ∛, |x|, x², x³, xⁿ
- trig: sin, cos, tan, and their inverses
- hyperbolic: sinh, cosh, tanh
- misc: ln, log, exp, floor, ceil, round, sign, max, min

you can type freely or tap the keyboard buttons, they insert at the cursor position.

---

## navigating the graph

- click and drag to pan
- scroll to zoom in/out
- zoom buttons and a reset button are in the sidebar
- current cursor coordinates are shown in the bottom right corner

---

## audio

the fitted curve or function can be sonified. it sweeps across x and maps the y value to a frequency, so you can hear the shape of the graph. speed and volume are adjustable.

---

## running it

```
npm install
npm start
```

requires node and electron.
