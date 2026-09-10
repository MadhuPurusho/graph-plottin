let points = []
let currentFit = null
let activeFitType = 'auto'
let audioCtx = null
let audioStopFlag = false
let isPlaying = false
let playheadX = null
let appMode = 'points'
let fnCurve = null

const canvas = document.getElementById('graph')
const ctx = canvas.getContext('2d')

let viewX = 0
let viewY = 0
let scale = 60

const inputX = document.getElementById('inputX')
const inputY = document.getElementById('inputY')
const btnAdd = document.getElementById('btnAdd')
const btnClear = document.getElementById('btnClear')
const btnFit = document.getElementById('btnFit')
const pointList = document.getElementById('pointList')
const pointCount = document.getElementById('pointCount')
const fitType = document.getElementById('fitType')
const resultSection = document.getElementById('resultSection')
const resultType = document.getElementById('resultType')
const resultEq = document.getElementById('resultEq')
const resultR2 = document.getElementById('resultR2')
const btnPlay = document.getElementById('btnPlay')
const btnStop = document.getElementById('btnStop')
const audioSpeed = document.getElementById('audioSpeed')
const audioVolume = document.getElementById('audioVolume')
const audioStatus = document.getElementById('audioStatus')
const btnZoomIn = document.getElementById('btnZoomIn')
const btnZoomOut = document.getElementById('btnZoomOut')
const btnReset = document.getElementById('btnReset')
const coordsLabel = document.getElementById('coordsLabel')
const mathKeyboard = document.getElementById('mathKeyboard')
const inputFn = document.getElementById('inputFn')
const fnXMin = document.getElementById('fnXMin')
const fnXMax = document.getElementById('fnXMax')
const btnPlotFn = document.getElementById('btnPlotFn')
const fnError = document.getElementById('fnError')
const modePoints = document.getElementById('modePoints')
const modeFunction = document.getElementById('modeFunction')
const pointsPanel = document.getElementById('pointsPanel')
const functionPanel = document.getElementById('functionPanel')
const pointsListPanel = document.getElementById('pointsListPanel')

const PHI = (1 + Math.sqrt(5)) / 2

function fmt(n) {
  if (n === null || n === undefined || !isFinite(n)) return '?'
  if (Math.abs(n) < 1e-10) return '0'
  if (Math.abs(n) >= 1e6 || (Math.abs(n) < 0.001 && n !== 0)) return n.toExponential(3)
  return parseFloat(n.toPrecision(6)).toString()
}

function sign(n) { return n >= 0 ? ' + ' : ' - ' }

function rSquared(xs, ys, predict) {
  const mean = ys.reduce((a, b) => a + b, 0) / ys.length
  const ssTot = ys.reduce((s, y) => s + (y - mean) ** 2, 0)
  const ssRes = xs.reduce((s, x, i) => s + (ys[i] - predict(x)) ** 2, 0)
  if (ssTot === 0) return 1
  return Math.max(0, 1 - ssRes / ssTot)
}

function gaussianElimination(A, b) {
  const n = b.length
  const M = A.map((row, i) => [...row, b[i]])
  for (let col = 0; col < n; col++) {
    let maxRow = col
    for (let row = col + 1; row < n; row++)
      if (Math.abs(M[row][col]) > Math.abs(M[maxRow][col])) maxRow = row
    ;[M[col], M[maxRow]] = [M[maxRow], M[col]]
    if (Math.abs(M[col][col]) < 1e-12) return null
    for (let row = col + 1; row < n; row++) {
      const f = M[row][col] / M[col][col]
      for (let k = col; k <= n; k++) M[row][k] -= f * M[col][k]
    }
  }
  const x = new Array(n).fill(0)
  for (let i = n - 1; i >= 0; i--) {
    x[i] = M[i][n]
    for (let j = i + 1; j < n; j++) x[i] -= M[i][j] * x[j]
    x[i] /= M[i][i]
  }
  return x
}

function polyFit(xs, ys, degree) {
  if (xs.length < degree + 1) return null
  const size = degree + 1
  const A = [], b = []
  for (let i = 0; i < size; i++) {
    A.push([])
    b.push(0)
    for (let j = 0; j < size; j++)
      A[i].push(xs.reduce((s, x) => s + Math.pow(x, i + j), 0))
    b[i] = xs.reduce((s, x, idx) => s + Math.pow(x, i) * ys[idx], 0)
  }
  const coeffs = gaussianElimination(A, b)
  if (!coeffs) return null
  const predict = x => coeffs.reduce((s, c, i) => s + c * Math.pow(x, i), 0)
  return { coeffs, predict, r2: rSquared(xs, ys, predict) }
}

function expFit(xs, ys) {
  if (ys.some(y => y <= 0)) return null
  const res = polyFit(xs, ys.map(y => Math.log(y)), 1)
  if (!res) return null
  const a = Math.exp(res.coeffs[0]), b = res.coeffs[1]
  const predict = x => a * Math.exp(b * x)
  return { a, b, predict, r2: rSquared(xs, ys, predict) }
}

function logFit(xs, ys) {
  if (xs.some(x => x <= 0)) return null
  const res = polyFit(xs.map(x => Math.log(x)), ys, 1)
  if (!res) return null
  const b = res.coeffs[0], a = res.coeffs[1]
  const predict = x => a * Math.log(x) + b
  return { a, b, predict, r2: rSquared(xs, ys, predict) }
}

function powerFit(xs, ys) {
  if (xs.some(x => x <= 0) || ys.some(y => y <= 0)) return null
  const res = polyFit(xs.map(x => Math.log(x)), ys.map(y => Math.log(y)), 1)
  if (!res) return null
  const a = Math.exp(res.coeffs[0]), b = res.coeffs[1]
  const predict = x => a * Math.pow(x, b)
  return { a, b, predict, r2: rSquared(xs, ys, predict) }
}

function sinFit(xs, ys) {
  if (xs.length < 4) return null
  const yMin = Math.min(...ys), yMax = Math.max(...ys)
  const a = (yMax - yMin) / 2, d = (yMax + yMin) / 2
  const xRange = Math.max(...xs) - Math.min(...xs)
  if (xRange === 0) return null
  const b = (2 * Math.PI) / xRange
  let best = null, bestR2 = -Infinity
  for (let s = 0; s < 40; s++) {
    const c = (s / 40) * 2 * Math.PI
    const predict = x => a * Math.sin(b * x + c) + d
    const r2 = rSquared(xs, ys, predict)
    if (r2 > bestR2) { bestR2 = r2; best = { a, b, c, d, predict, r2 } }
  }
  return best
}

function cosFit(xs, ys) {
  if (xs.length < 4) return null
  const yMin = Math.min(...ys), yMax = Math.max(...ys)
  const a = (yMax - yMin) / 2, d = (yMax + yMin) / 2
  const xRange = Math.max(...xs) - Math.min(...xs)
  if (xRange === 0) return null
  const b = (2 * Math.PI) / xRange
  let best = null, bestR2 = -Infinity
  for (let s = 0; s < 40; s++) {
    const c = (s / 40) * 2 * Math.PI
    const predict = x => a * Math.cos(b * x + c) + d
    const r2 = rSquared(xs, ys, predict)
    if (r2 > bestR2) { bestR2 = r2; best = { a, b, c, d, predict, r2 } }
  }
  return best
}

function tanFit(xs, ys) {
  if (xs.length < 4) return null
  const xRange = Math.max(...xs) - Math.min(...xs)
  if (xRange === 0) return null
  const b = Math.PI / xRange
  const yMid = (Math.max(...ys) + Math.min(...ys)) / 2
  const a = (Math.max(...ys) - Math.min(...ys)) / 4 || 1
  let best = null, bestR2 = -Infinity
  for (let s = 0; s < 20; s++) {
    const c = (s / 20) * Math.PI
    const predict = x => {
      const v = a * Math.tan(b * x + c) + yMid
      return Math.abs(v) > 1e6 ? NaN : v
    }
    const validPairs = xs.map((x, i) => [x, ys[i]]).filter(([x]) => isFinite(predict(x)))
    if (validPairs.length < 2) continue
    const vxs = validPairs.map(p => p[0]), vys = validPairs.map(p => p[1])
    const r2 = rSquared(vxs, vys, predict)
    if (r2 > bestR2) { bestR2 = r2; best = { a, b, c, d: yMid, predict, r2 } }
  }
  return best
}

function hyperbolicFit(xs, ys) {
  if (xs.some(x => Math.abs(x) < 1e-10)) return null
  const res = polyFit(xs.map(x => 1 / x), ys, 1)
  if (!res) return null
  const a = res.coeffs[1], b = res.coeffs[0]
  const predict = x => a / x + b
  return { a, b, predict, r2: rSquared(xs, ys, predict) }
}

function sqrtFit(xs, ys) {
  if (xs.some(x => x < 0)) return null
  const res = polyFit(xs.map(x => Math.sqrt(x)), ys, 1)
  if (!res) return null
  const a = res.coeffs[1], b = res.coeffs[0]
  const predict = x => x < 0 ? NaN : a * Math.sqrt(x) + b
  return { a, b, predict, r2: rSquared(xs, ys, predict) }
}

function absoluteFit(xs, ys) {
  const xMean = xs.reduce((s, v) => s + v, 0) / xs.length
  const yMean = ys.reduce((s, v) => s + v, 0) / ys.length
  let best = null, bestR2 = -Infinity
  for (let bi = -2; bi <= 2; bi += 0.5) {
    const bVal = xMean + bi
    const absXs = xs.map(x => Math.abs(x - bVal))
    const res = polyFit(absXs, ys, 1)
    if (!res) continue
    const a = res.coeffs[1], c = res.coeffs[0]
    const predict = x => a * Math.abs(x - bVal) + c
    const r2 = rSquared(xs, ys, predict)
    if (r2 > bestR2) { bestR2 = r2; best = { a, b: bVal, c, predict, r2 } }
  }
  return best
}

function gaussianFit(xs, ys) {
  if (ys.some(y => y <= 0)) return null
  const maxY = Math.max(...ys)
  const maxIdx = ys.indexOf(maxY)
  const b = xs[maxIdx]
  const logYs = ys.map(y => Math.log(y))
  const shiftXs = xs.map(x => (x - b) ** 2)
  const res = polyFit(shiftXs, logYs, 1)
  if (!res) return null
  const a = Math.exp(res.coeffs[0])
  const c2 = -1 / res.coeffs[1]
  if (c2 <= 0) return null
  const c = Math.sqrt(c2)
  const predict = x => a * Math.exp(-(((x - b) / c) ** 2))
  return { a, b, c, predict, r2: rSquared(xs, ys, predict) }
}

function buildFits(xs, ys) {
  const fits = {}
  for (let deg = 1; deg <= 5; deg++) {
    const key = deg === 1 ? 'linear' : `poly${deg}`
    const res = polyFit(xs, ys, deg)
    if (res) fits[key] = res
  }
  const e = expFit(xs, ys); if (e) fits.exponential = e
  const l = logFit(xs, ys); if (l) fits.logarithmic = l
  const p = powerFit(xs, ys); if (p) fits.power = p
  const s = sinFit(xs, ys); if (s) fits.sinusoidal = s
  const co = cosFit(xs, ys); if (co) fits.cosinusoidal = co
  const t = tanFit(xs, ys); if (t) fits.tangential = t
  const h = hyperbolicFit(xs, ys); if (h) fits.hyperbolic = h
  const sq = sqrtFit(xs, ys); if (sq) fits.sqrt = sq
  const ab = absoluteFit(xs, ys); if (ab) fits.absolute = ab
  const g = gaussianFit(xs, ys); if (g) fits.gaussian = g
  return fits
}

function bestFit(fits) {
  let best = null, bestScore = -Infinity
  const penalty = { linear: 0, poly2: 0.02, poly3: 0.04, poly4: 0.06, poly5: 0.08 }
  for (const [type, fit] of Object.entries(fits)) {
    const pen = penalty[type] ?? 0.01
    const score = fit.r2 - pen
    if (score > bestScore) { bestScore = score; best = type }
  }
  return best
}

function equationLabel(type, fit) {
  const c = fit.coeffs
  switch (type) {
    case 'linear': return `y = ${fmt(c[1])}x${sign(c[0])}${fmt(Math.abs(c[0]))}`
    case 'poly2': return `y = ${fmt(c[2])}x²${sign(c[1])}${fmt(Math.abs(c[1]))}x${sign(c[0])}${fmt(Math.abs(c[0]))}`
    case 'poly3': return `y = ${fmt(c[3])}x³${sign(c[2])}${fmt(Math.abs(c[2]))}x²${sign(c[1])}${fmt(Math.abs(c[1]))}x${sign(c[0])}${fmt(Math.abs(c[0]))}`
    case 'poly4': return `y = ${fmt(c[4])}x⁴${sign(c[3])}${fmt(Math.abs(c[3]))}x³${sign(c[2])}${fmt(Math.abs(c[2]))}x²${sign(c[1])}${fmt(Math.abs(c[1]))}x${sign(c[0])}${fmt(Math.abs(c[0]))}`
    case 'poly5': return `y = ${fmt(c[5])}x⁵${sign(c[4])}${fmt(Math.abs(c[4]))}x⁴${sign(c[3])}${fmt(Math.abs(c[3]))}x³${sign(c[2])}${fmt(Math.abs(c[2]))}x²${sign(c[1])}${fmt(Math.abs(c[1]))}x${sign(c[0])}${fmt(Math.abs(c[0]))}`
    case 'exponential': return `y = ${fmt(fit.a)}·e^(${fmt(fit.b)}x)`
    case 'logarithmic': return `y = ${fmt(fit.a)}·ln(x)${sign(fit.b)}${fmt(Math.abs(fit.b))}`
    case 'power': return `y = ${fmt(fit.a)}·x^${fmt(fit.b)}`
    case 'sinusoidal': return `y = ${fmt(fit.a)}·sin(${fmt(fit.b)}x${sign(fit.c)}${fmt(Math.abs(fit.c))})${sign(fit.d)}${fmt(Math.abs(fit.d))}`
    case 'cosinusoidal': return `y = ${fmt(fit.a)}·cos(${fmt(fit.b)}x${sign(fit.c)}${fmt(Math.abs(fit.c))})${sign(fit.d)}${fmt(Math.abs(fit.d))}`
    case 'tangential': return `y = ${fmt(fit.a)}·tan(${fmt(fit.b)}x${sign(fit.c)}${fmt(Math.abs(fit.c))})${sign(fit.d)}${fmt(Math.abs(fit.d))}`
    case 'hyperbolic': return `y = ${fmt(fit.a)}/x${sign(fit.b)}${fmt(Math.abs(fit.b))}`
    case 'sqrt': return `y = ${fmt(fit.a)}·√x${sign(fit.b)}${fmt(Math.abs(fit.b))}`
    case 'absolute': return `y = ${fmt(fit.a)}·|x${sign(-fit.b)}${fmt(Math.abs(fit.b))}|${sign(fit.c)}${fmt(Math.abs(fit.c))}`
    case 'gaussian': return `y = ${fmt(fit.a)}·e^(-((x${sign(-fit.b)}${fmt(Math.abs(fit.b))})/${fmt(fit.c)})²)`
    default: return ''
  }
}

function typeLabel(type) {
  return {
    linear: 'Linear', poly2: 'Quadratic', poly3: 'Cubic', poly4: 'Degree 4', poly5: 'Degree 5',
    exponential: 'Exponential', logarithmic: 'Logarithmic', power: 'Power',
    sinusoidal: 'Sinusoidal', cosinusoidal: 'Cosinusoidal', tangential: 'Tangential',
    hyperbolic: 'Hyperbolic', sqrt: 'Square Root', absolute: 'Absolute Value', gaussian: 'Gaussian'
  }[type] || type
}

function evalExpr(expr, xVal) {
  const safeExpr = expr
    .replace(/π/g, '(' + Math.PI + ')')
    .replace(/φ/g, '(' + PHI + ')')
    .replace(/∞/g, 'Infinity')
    .replace(/\bpi\b/gi, '(' + Math.PI + ')')
    .replace(/\bphi\b/gi, '(' + PHI + ')')
    .replace(/\be\b/g, '(' + Math.E + ')')
    .replace(/\bsqrt\b/g, 'Math.sqrt')
    .replace(/\bcbrt\b/g, 'Math.cbrt')
    .replace(/\babs\b/g, 'Math.abs')
    .replace(/\bsin\b/g, 'Math.sin')
    .replace(/\bcos\b/g, 'Math.cos')
    .replace(/\btan\b/g, 'Math.tan')
    .replace(/\basin\b/g, 'Math.asin')
    .replace(/\bacos\b/g, 'Math.acos')
    .replace(/\batan\b/g, 'Math.atan')
    .replace(/\bsinh\b/g, 'Math.sinh')
    .replace(/\bcosh\b/g, 'Math.cosh')
    .replace(/\btanh\b/g, 'Math.tanh')
    .replace(/\bln\b/g, 'Math.log')
    .replace(/\blog\b/g, 'Math.log10')
    .replace(/\bexp\b/g, 'Math.exp')
    .replace(/\bfloor\b/g, 'Math.floor')
    .replace(/\bceil\b/g, 'Math.ceil')
    .replace(/\bround\b/g, 'Math.round')
    .replace(/\bsign\b/g, 'Math.sign')
    .replace(/\bmax\b/g, 'Math.max')
    .replace(/\bmin\b/g, 'Math.min')
    .replace(/\^/g, '**')
    .replace(/\bx\b/g, '(' + xVal + ')')
  return Function('"use strict"; return (' + safeExpr + ')')()
}

function parseInputValue(str) {
  if (!str || !str.trim()) return NaN
  try { return evalExpr(str.trim(), 0) } catch { return NaN }
}

function toScreen(wx, wy) {
  return [
    canvas.width / 2 + (wx - viewX) * scale,
    canvas.height / 2 - (wy - viewY) * scale
  ]
}

function toWorld(sx, sy) {
  return [
    viewX + (sx - canvas.width / 2) / scale,
    viewY - (sy - canvas.height / 2) / scale
  ]
}

function niceStep(rawStep) {
  const mag = Math.pow(10, Math.floor(Math.log10(rawStep)))
  const norm = rawStep / mag
  if (norm < 1.5) return mag
  if (norm < 3.5) return 2 * mag
  if (norm < 7.5) return 5 * mag
  return 10 * mag
}

function drawEquationOnCanvas(label) {
  if (!label) return
  const W = canvas.width
  ctx.save()
  ctx.font = 'bold 13px monospace'
  const padding = 10
  const textW = ctx.measureText(label).width
  const boxW = textW + padding * 2
  const boxH = 28
  const bx = W - boxW - 14
  const by = 14
  ctx.fillStyle = 'rgba(15,17,23,0.82)'
  ctx.strokeStyle = '#2e3450'
  ctx.lineWidth = 1
  ctx.beginPath()
  ctx.roundRect(bx, by, boxW, boxH, 6)
  ctx.fill()
  ctx.stroke()
  ctx.fillStyle = '#7c9ef8'
  ctx.textBaseline = 'middle'
  ctx.fillText(label, bx + padding, by + boxH / 2)
  ctx.restore()
}

function draw() {
  const W = canvas.width, H = canvas.height
  ctx.clearRect(0, 0, W, H)

  ctx.fillStyle = '#0f1117'
  ctx.fillRect(0, 0, W, H)

  const targetGridPx = 80
  const rawStep = targetGridPx / scale
  const step = niceStep(rawStep)

  const [wxLeft, wyTop] = toWorld(0, 0)
  const [wxRight, wyBottom] = toWorld(W, H)

  const xStart = Math.floor(wxLeft / step) * step
  const xEnd = Math.ceil(wxRight / step) * step
  const yStart = Math.floor(wyBottom / step) * step
  const yEnd = Math.ceil(wyTop / step) * step

  ctx.strokeStyle = '#1a1d2a'
  ctx.lineWidth = 1
  for (let x = xStart; x <= xEnd + step * 0.01; x += step) {
    const [sx] = toScreen(x, 0)
    ctx.beginPath(); ctx.moveTo(sx, 0); ctx.lineTo(sx, H); ctx.stroke()
  }
  for (let y = yStart; y <= yEnd + step * 0.01; y += step) {
    const [, sy] = toScreen(0, y)
    ctx.beginPath(); ctx.moveTo(0, sy); ctx.lineTo(W, sy); ctx.stroke()
  }

  const [ox] = toScreen(0, 0)
  const [, oy] = toScreen(0, 0)
  ctx.strokeStyle = '#2e3450'
  ctx.lineWidth = 1.5
  if (ox >= 0 && ox <= W) { ctx.beginPath(); ctx.moveTo(ox, 0); ctx.lineTo(ox, H); ctx.stroke() }
  if (oy >= 0 && oy <= H) { ctx.beginPath(); ctx.moveTo(0, oy); ctx.lineTo(W, oy); ctx.stroke() }

  ctx.fillStyle = '#3a3f5c'
  ctx.font = '11px monospace'
  ctx.textAlign = 'center'
  for (let x = xStart; x <= xEnd + step * 0.01; x += step) {
    if (Math.abs(x) < step * 0.01) continue
    const [sx] = toScreen(x, 0)
    const labelY = Math.min(Math.max(oy + 14, 14), H - 4)
    ctx.fillText(fmt(x), sx, labelY)
  }
  ctx.textAlign = 'right'
  for (let y = yStart; y <= yEnd + step * 0.01; y += step) {
    if (Math.abs(y) < step * 0.01) continue
    const [, sy] = toScreen(0, y)
    const labelX = Math.min(Math.max(ox - 6, 4), W - 4)
    ctx.fillText(fmt(y), labelX, sy + 4)
  }

  if (appMode === 'function' && fnCurve) {
    ctx.strokeStyle = '#f8c77c'
    ctx.lineWidth = 2.5
    ctx.lineJoin = 'round'
    ctx.beginPath()
    let started = false
    const steps = Math.max(W * 2, 800)
    for (let i = 0; i <= steps; i++) {
      const wx = wxLeft + (i / steps) * (wxRight - wxLeft)
      let wy
      try { wy = fnCurve(wx) } catch { started = false; continue }
      if (!isFinite(wy) || Math.abs(wy) > 1e10) { started = false; continue }
      const [sx, sy] = toScreen(wx, wy)
      if (!started) { ctx.moveTo(sx, sy); started = true }
      else ctx.lineTo(sx, sy)
    }
    ctx.stroke()
    if (fnCurve._label) drawEquationOnCanvas('y = ' + fnCurve._label)
  }

  if (appMode === 'points' && currentFit) {
    ctx.strokeStyle = '#7c9ef8'
    ctx.lineWidth = 2.5
    ctx.lineJoin = 'round'
    ctx.beginPath()
    let started = false
    const steps = Math.max(W * 2, 800)
    for (let i = 0; i <= steps; i++) {
      const wx = wxLeft + (i / steps) * (wxRight - wxLeft)
      const wy = currentFit.predict(wx)
      if (!isFinite(wy) || Math.abs(wy) > 1e10) { started = false; continue }
      const [sx, sy] = toScreen(wx, wy)
      if (!started) { ctx.moveTo(sx, sy); started = true }
      else ctx.lineTo(sx, sy)
    }
    ctx.stroke()
    if (currentFit._label) drawEquationOnCanvas(currentFit._label)
  }

  points.forEach(p => {
    const [sx, sy] = toScreen(p.x, p.y)
    ctx.beginPath()
    ctx.arc(sx, sy, 6, 0, Math.PI * 2)
    ctx.fillStyle = '#f87c7c'
    ctx.fill()
    ctx.strokeStyle = '#ff9a9a'
    ctx.lineWidth = 1.5
    ctx.stroke()
  })

  if (isPlaying && playheadX !== null) {
    const [sx] = toScreen(playheadX, 0)
    ctx.strokeStyle = 'rgba(124,198,248,0.7)'
    ctx.lineWidth = 1.5
    ctx.setLineDash([4, 4])
    ctx.beginPath(); ctx.moveTo(sx, 0); ctx.lineTo(sx, H); ctx.stroke()
    ctx.setLineDash([])
  }
}

function resizeCanvas() {
  const rect = canvas.parentElement.getBoundingClientRect()
  canvas.width = rect.width
  canvas.height = rect.height
  draw()
}

window.addEventListener('resize', resizeCanvas)
resizeCanvas()

let isPanning = false
let panStart = { x: 0, y: 0, vx: 0, vy: 0 }

canvas.addEventListener('mousedown', e => {
  isPanning = true
  panStart = { x: e.clientX, y: e.clientY, vx: viewX, vy: viewY }
  canvas.style.cursor = 'grabbing'
})

window.addEventListener('mousemove', e => {
  if (isPanning) {
    const dx = (e.clientX - panStart.x) / scale
    const dy = (e.clientY - panStart.y) / scale
    viewX = panStart.vx - dx
    viewY = panStart.vy + dy
    draw()
  }
  const rect = canvas.getBoundingClientRect()
  const [wx, wy] = toWorld(e.clientX - rect.left, e.clientY - rect.top)
  coordsLabel.textContent = `x = ${fmt(wx)},  y = ${fmt(wy)}`
})

window.addEventListener('mouseup', () => {
  isPanning = false
  canvas.style.cursor = 'grab'
})

canvas.addEventListener('wheel', e => {
  e.preventDefault()
  const zoomFactor = e.deltaY < 0 ? 1.12 : 1 / 1.12
  const rect = canvas.getBoundingClientRect()
  const mx = e.clientX - rect.left
  const my = e.clientY - rect.top
  const [wx, wy] = toWorld(mx, my)
  scale *= zoomFactor
  scale = Math.min(Math.max(scale, 2), 5000)
  viewX = wx - (mx - canvas.width / 2) / scale
  viewY = wy + (my - canvas.height / 2) / scale
  draw()
}, { passive: false })

btnZoomIn.addEventListener('click', () => { scale *= 1.4; draw() })
btnZoomOut.addEventListener('click', () => { scale /= 1.4; draw() })
btnReset.addEventListener('click', () => { scale = 60; viewX = 0; viewY = 0; draw() })

modePoints.addEventListener('click', () => {
  appMode = 'points'
  modePoints.classList.add('active')
  modeFunction.classList.remove('active')
  pointsPanel.style.display = ''
  functionPanel.style.display = 'none'
  pointsListPanel.style.display = ''
  document.querySelector('.section:has(#fitType)').style.display = ''
  fnCurve = null
  draw()
})

modeFunction.addEventListener('click', () => {
  appMode = 'function'
  modeFunction.classList.add('active')
  modePoints.classList.remove('active')
  pointsPanel.style.display = 'none'
  functionPanel.style.display = ''
  pointsListPanel.style.display = 'none'
  document.querySelector('.section:has(#fitType)').style.display = 'none'
  draw()
})

btnPlotFn.addEventListener('click', () => {
  const expr = inputFn.value.trim()
  if (!expr) return
  fnError.textContent = ''
  try {
    evalExpr(expr, 1)
    fnCurve = x => evalExpr(expr, x)
    fnCurve._label = expr
    resultSection.style.display = 'flex'
    resultType.textContent = 'Function'
    resultEq.textContent = 'y = ' + expr
    resultR2.innerHTML = ''
    btnPlay.disabled = false
    draw()
  } catch (err) {
    fnError.textContent = 'Error: ' + err.message
  }
})

inputFn.addEventListener('keydown', e => { if (e.key === 'Enter') btnPlotFn.click() })

function fitCurve(selectedType) {
  if (points.length < 2) return
  const xs = points.map(p => p.x)
  const ys = points.map(p => p.y)
  const fits = buildFits(xs, ys)
  let type = selectedType === 'auto' ? bestFit(fits) : selectedType
  if (!fits[type]) type = bestFit(fits)
  if (!type) {
    playErrorSound()
    audioStatus.textContent = 'No valid fit found for these points.'
    return
  }
  currentFit = fits[type]
  currentFit._label = equationLabel(type, currentFit)
  activeFitType = type

  resultSection.style.display = 'flex'
  resultType.textContent = typeLabel(type)
  resultEq.textContent = equationLabel(type, currentFit)
  resultR2.innerHTML = `R² = <span>${fmt(currentFit.r2)}</span>`

  btnPlay.disabled = false
  draw()
}

function refreshList() {
  pointList.innerHTML = ''
  pointCount.textContent = `(${points.length})`
  points.forEach((p, i) => {
    const li = document.createElement('li')
    li.innerHTML = `<span>(${fmt(p.x)}, ${fmt(p.y)})</span><button data-i="${i}" title="Remove">×</button>`
    pointList.appendChild(li)
  })
  pointList.querySelectorAll('button').forEach(btn => {
    btn.addEventListener('click', () => {
      points.splice(parseInt(btn.dataset.i), 1)
      refreshList()
      if (points.length >= 2) fitCurve(fitType.value)
      else { currentFit = null; btnPlay.disabled = true; resultSection.style.display = 'none'; draw() }
    })
  })
}

btnAdd.addEventListener('click', () => {
  const x = parseInputValue(inputX.value)
  const y = parseInputValue(inputY.value)
  if (isNaN(x) || isNaN(y)) { playErrorSound(); return }
  points.push({ x, y })
  inputX.value = ''; inputY.value = ''; inputX.focus()
  refreshList()
  fitCurve(fitType.value)
})

;[inputX, inputY].forEach(el => el.addEventListener('keydown', e => { if (e.key === 'Enter') btnAdd.click() }))

btnClear.addEventListener('click', () => {
  points = []
  currentFit = null
  refreshList()
  btnPlay.disabled = true
  resultSection.style.display = 'none'
  stopAudio()
  draw()
})

btnFit.addEventListener('click', () => fitCurve(fitType.value))
fitType.addEventListener('change', () => fitCurve(fitType.value))

let activeKbdTarget = null

document.querySelectorAll('.kbd-toggle').forEach(btn => {
  btn.addEventListener('click', e => {
    e.stopPropagation()
    const targetId = btn.dataset.target
    const targetInput = document.getElementById(targetId)
    if (activeKbdTarget === targetInput && mathKeyboard.style.display !== 'none') {
      mathKeyboard.style.display = 'none'
      activeKbdTarget = null
      return
    }
    activeKbdTarget = targetInput
    const section = btn.closest('.section') || btn.closest('.sidebar')
    const sidebar = document.querySelector('.sidebar')
    const btnRect = btn.getBoundingClientRect()
    const sidebarRect = sidebar.getBoundingClientRect()
    sidebar.insertBefore(mathKeyboard, section ? section.nextSibling : null)
    mathKeyboard.style.display = 'block'
    targetInput.focus()
  })
})

document.querySelectorAll('.kbd-btn[data-insert]').forEach(btn => {
  btn.addEventListener('mousedown', e => {
    e.preventDefault()
    if (!activeKbdTarget) return
    const ins = btn.dataset.insert
    const el = activeKbdTarget
    const start = el.selectionStart ?? el.value.length
    const end = el.selectionEnd ?? el.value.length
    el.value = el.value.slice(0, start) + ins + el.value.slice(end)
    const pos = start + ins.length
    el.setSelectionRange(pos, pos)
    el.focus()
  })
})

document.getElementById('kbdBackspace').addEventListener('mousedown', e => {
  e.preventDefault()
  if (!activeKbdTarget) return
  const el = activeKbdTarget
  const start = el.selectionStart
  const end = el.selectionEnd
  if (start === end && start > 0) {
    el.value = el.value.slice(0, start - 1) + el.value.slice(end)
    el.setSelectionRange(start - 1, start - 1)
  } else if (start !== end) {
    el.value = el.value.slice(0, start) + el.value.slice(end)
    el.setSelectionRange(start, start)
  }
  el.focus()
})

document.getElementById('kbdDone').addEventListener('click', () => {
  mathKeyboard.style.display = 'none'
  activeKbdTarget = null
})

document.addEventListener('click', e => {
  if (!mathKeyboard.contains(e.target) && !e.target.classList.contains('kbd-toggle')) {
    mathKeyboard.style.display = 'none'
    activeKbdTarget = null
  }
})

function getAudioCtx() {
  if (!audioCtx) audioCtx = new (window.AudioContext || window.webkitAudioContext)()
  return audioCtx
}

function playErrorSound() {
  const ac = getAudioCtx()
  const osc = ac.createOscillator()
  const gain = ac.createGain()
  osc.connect(gain); gain.connect(ac.destination)
  osc.type = 'sawtooth'
  osc.frequency.setValueAtTime(120, ac.currentTime)
  osc.frequency.exponentialRampToValueAtTime(60, ac.currentTime + 0.25)
  gain.gain.setValueAtTime(0.3 * parseFloat(audioVolume.value), ac.currentTime)
  gain.gain.exponentialRampToValueAtTime(0.001, ac.currentTime + 0.3)
  osc.start(ac.currentTime)
  osc.stop(ac.currentTime + 0.3)
}

function stopAudio() {
  audioStopFlag = true
  isPlaying = false
  playheadX = null
  const hasFit = appMode === 'function' ? !!fnCurve : (points.length >= 2 && !!currentFit)
  btnPlay.disabled = !hasFit
  btnStop.disabled = true
  audioStatus.textContent = ''
  draw()
}

btnStop.addEventListener('click', stopAudio)

btnPlay.addEventListener('click', async () => {
  const activeFit = appMode === 'function' ? fnCurve : currentFit
  if (!activeFit) return
  if (isPlaying) stopAudio()

  const ac = getAudioCtx()
  if (ac.state === 'suspended') await ac.resume()

  audioStopFlag = false
  isPlaying = true
  btnPlay.disabled = true
  btnStop.disabled = false

  let fromX, toX
  if (appMode === 'function') {
    fromX = parseInputValue(fnXMin.value) || -10
    toX = parseInputValue(fnXMax.value) || 10
  } else {
    const xs = points.map(p => p.x)
    const xMin = Math.min(...xs), xMax = Math.max(...xs)
    const xPad = (xMax - xMin) * 0.1 || 1
    fromX = xMin - xPad
    toX = xMax + xPad
  }

  const speed = parseFloat(audioSpeed.value)
  const totalDuration = 4 / speed
  const steps = 300
  const stepDuration = totalDuration / steps

  const ys = []
  for (let i = 0; i <= steps; i++) {
    const x = fromX + (i / steps) * (toX - fromX)
    try {
      const y = activeFit(x)
      ys.push(isFinite(y) ? y : null)
    } catch { ys.push(null) }
  }

  const validYs = ys.filter(y => y !== null)
  if (validYs.length === 0) { playErrorSound(); stopAudio(); return }

  const yMin = Math.min(...validYs)
  const yMax = Math.max(...validYs)
  const yRange = yMax - yMin || 1
  const freqMin = 120, freqMax = 1400
  const vol = parseFloat(audioVolume.value)

  audioStatus.textContent = 'Playing...'

  let t = ac.currentTime + 0.05

  for (let i = 0; i <= steps; i++) {
    if (audioStopFlag) break
    const x = fromX + (i / steps) * (toX - fromX)
    const y = ys[i]

    if (y === null) {
      await new Promise(r => setTimeout(r, stepDuration * 1000))
      continue
    }

    const norm = (y - yMin) / yRange
    const freq = freqMin * Math.pow(freqMax / freqMin, norm)

    const osc = ac.createOscillator()
    const gain = ac.createGain()
    osc.connect(gain); gain.connect(ac.destination)
    osc.type = 'sine'
    osc.frequency.setValueAtTime(freq, t)
    gain.gain.setValueAtTime(0, t)
    gain.gain.linearRampToValueAtTime(vol * 0.25, t + stepDuration * 0.1)
    gain.gain.setValueAtTime(vol * 0.25, t + stepDuration * 0.85)
    gain.gain.linearRampToValueAtTime(0, t + stepDuration)
    osc.start(t)
    osc.stop(t + stepDuration)

    playheadX = x
    draw()

    await new Promise(r => setTimeout(r, stepDuration * 1000))
    t = ac.currentTime
  }

  if (!audioStopFlag) {
    isPlaying = false
    playheadX = null
    btnPlay.disabled = false
    btnStop.disabled = true
    audioStatus.textContent = ''
    draw()
  }
})

draw()
