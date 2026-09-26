// Shared DSP for the trailer audio tools: oscillators, filters, effects,
// limiter, loudness (BS.1770), and 16-bit WAV IO. Deterministic, no deps.
import fs from 'node:fs';

export const SR = 48000;
export const TAU = Math.PI * 2;

export function rng(seed) {
  let a = seed >>> 0 || 1;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}
export function hash(i, seed = 0) {
  let h = Math.imul((i | 0) ^ 0x9e3779b9, 0x85ebca6b) ^ Math.imul((seed | 0) + 0x1234567, 0xc2b2ae35);
  h ^= h >>> 13; h = Math.imul(h, 0x27d4eb2f); h ^= h >>> 16;
  return (h >>> 0) / 4294967296;
}
export const mtof = (m) => 440 * Math.pow(2, (m - 69) / 12);
export const db = (d) => Math.pow(10, d / 20);
export const clamp = (x, a, b) => (x < a ? a : x > b ? b : x);
export const smooth = (x) => (x <= 0 ? 0 : x >= 1 ? 1 : x * x * (3 - 2 * x));

export class Stereo {
  constructor(n) { this.n = n; this.L = new Float32Array(n); this.R = new Float32Array(n); }
  // Equal-power pan, center = unity on both sides.
  add(i, v, pan = 0) {
    if (i < 0 || i >= this.n) return;
    const a = (clamp(pan, -1, 1) + 1) * Math.PI / 4;
    this.L[i] += v * Math.cos(a) * Math.SQRT2;
    this.R[i] += v * Math.sin(a) * Math.SQRT2;
  }
  addLR(i, l, r) { if (i < 0 || i >= this.n) return; this.L[i] += l; this.R[i] += r; }
}
export function panGains(pan) {
  const a = (clamp(pan, -1, 1) + 1) * Math.PI / 4;
  return [Math.cos(a) * Math.SQRT2, Math.sin(a) * Math.SQRT2];
}

// PolyBLEP residual for band-limited edges.
export function blep(t, dt) {
  if (t < dt) { t /= dt; return t + t - t * t - 1; }
  if (t > 1 - dt) { t = (t - 1) / dt; return t * t + t + t + 1; }
  return 0;
}

export class Osc {
  constructor(ph = 0) { this.ph = ph; }
  step(f) { this.ph += f / SR; if (this.ph >= 1) this.ph -= Math.floor(this.ph); }
  saw(f) { const dt = f / SR, p = this.ph; const v = 2 * p - 1 - blep(p, dt); this.step(f); return v; }
  pulse(f, duty) {
    const dt = f / SR, p = this.ph;
    let v = p < duty ? 1 : -1;
    v += blep(p, dt);
    let p2 = p - duty; if (p2 < 0) p2 += 1;
    v -= blep(p2, dt);
    this.step(f);
    return v;
  }
  sine(f) { const v = Math.sin(TAU * this.ph); this.step(f); return v; }
  tri(f) { const p = this.ph; const v = p < 0.5 ? 4 * p - 1 : 3 - 4 * p; this.step(f); return v; }
  // 4-bit stepped triangle, the NES flavor.
  nesTri(f) { const p = this.ph; const x = p < 0.5 ? p * 2 : 2 - p * 2; const v = Math.floor(x * 15.999) / 7.5 - 1; this.step(f); return v; }
}

// Topology-preserving state variable filter.
export class SVF {
  constructor(fc = 1000, q = 0.707) { this.ic1 = 0; this.ic2 = 0; this.set(fc, q); this.lp = this.bp = this.hp = 0; }
  set(fc, q = this.q) {
    this.q = q;
    const g = Math.tan(Math.PI * clamp(fc, 10, SR * 0.45) / SR);
    this.k = 1 / q;
    this.a1 = 1 / (1 + g * (g + this.k)); this.a2 = g * this.a1; this.a3 = g * this.a2;
  }
  process(v0) {
    const v3 = v0 - this.ic2;
    const v1 = this.a1 * this.ic1 + this.a2 * v3;
    const v2 = this.ic2 + this.a2 * this.ic1 + this.a3 * v3;
    this.ic1 = 2 * v1 - this.ic1; this.ic2 = 2 * v2 - this.ic2;
    this.lp = v2; this.bp = v1; this.hp = v0 - this.k * v1 - v2;
    return v2;
  }
}

export class Biquad {
  constructor(b, a) { this.b0 = b[0]; this.b1 = b[1]; this.b2 = b[2]; this.a1 = a[1]; this.a2 = a[2]; this.x1 = this.x2 = this.y1 = this.y2 = 0; }
  process(x) {
    const y = this.b0 * x + this.b1 * this.x1 + this.b2 * this.x2 - this.a1 * this.y1 - this.a2 * this.y2;
    this.x2 = this.x1; this.x1 = x; this.y2 = this.y1; this.y1 = y;
    return y;
  }
  static highpass(fc, q = 0.707) {
    const w = TAU * fc / SR, c = Math.cos(w), al = Math.sin(w) / (2 * q), a0 = 1 + al;
    return new Biquad([(1 + c) / 2 / a0, -(1 + c) / a0, (1 + c) / 2 / a0], [1, -2 * c / a0, (1 - al) / a0]);
  }
  static lowpass(fc, q = 0.707) {
    const w = TAU * fc / SR, c = Math.cos(w), al = Math.sin(w) / (2 * q), a0 = 1 + al;
    return new Biquad([(1 - c) / 2 / a0, (1 - c) / a0, (1 - c) / 2 / a0], [1, -2 * c / a0, (1 - al) / a0]);
  }
  static peak(fc, q, gainDb) {
    const A = Math.pow(10, gainDb / 40), w = TAU * fc / SR, c = Math.cos(w), al = Math.sin(w) / (2 * q), a0 = 1 + al / A;
    return new Biquad([(1 + al * A) / a0, -2 * c / a0, (1 - al * A) / a0], [1, -2 * c / a0, (1 - al / A) / a0]);
  }
}

export function filterInPlace(arr, mk) { const f = mk(); for (let i = 0; i < arr.length; i++) arr[i] = f.process(arr[i]); }

// Freeverb-style stereo reverb. Returns wet only.
export function reverb(inL, inR, { room = 0.84, damp = 0.25, predelay = 0.02, width = 1 } = {}) {
  const n = inL.length, s = SR / 44100;
  const combs = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617], aps = [556, 441, 341, 225];
  const out = [new Float32Array(n), new Float32Array(n)];
  const pd = Math.round(predelay * SR);
  for (let ch = 0; ch < 2; ch++) {
    const src = ch ? inR : inL, dst = out[ch], spread = ch ? 23 : 0;
    const cb = combs.map((l) => ({ buf: new Float32Array(Math.round((l + spread) * s)), i: 0, st: 0 }));
    const ab = aps.map((l) => ({ buf: new Float32Array(Math.round((l + spread) * s)), i: 0 }));
    for (let i = 0; i < n; i++) {
      const x = (i >= pd ? src[i - pd] : 0) * 0.015;
      let y = 0;
      for (const c of cb) {
        const o = c.buf[c.i];
        c.st = o * (1 - damp) + c.st * damp;
        c.buf[c.i] = x + c.st * room;
        if (++c.i >= c.buf.length) c.i = 0;
        y += o;
      }
      for (const a of ab) {
        const b = a.buf[a.i];
        a.buf[a.i] = y + b * 0.5;
        y = b - y;
        if (++a.i >= a.buf.length) a.i = 0;
      }
      dst[i] = y;
    }
  }
  if (width < 1) {
    for (let i = 0; i < n; i++) { const m = (out[0][i] + out[1][i]) / 2, d = (out[0][i] - out[1][i]) / 2 * width; out[0][i] = m + d; out[1][i] = m - d; }
  }
  return out;
}

// Ping-pong delay (mono-summed input bounces L -> R). Returns wet only.
export function pingpong(inL, inR, time, fb = 0.35, lp = 4500) {
  const n = inL.length, d = Math.round(time * SR);
  const outL = new Float32Array(n), outR = new Float32Array(n);
  const bl = new Float32Array(d), br = new Float32Array(d);
  const fl = new SVF(lp, 0.6), fr = new SVF(lp, 0.6);
  const hl = new SVF(250, 0.6), hr = new SVF(250, 0.6);
  let j = 0;
  for (let i = 0; i < n; i++) {
    const ol = bl[j], or = br[j];
    outL[i] = ol; outR[i] = or;
    const x = (inL[i] + inR[i]) * 0.5;
    fl.process(or * fb); hl.process(fl.lp);
    fr.process(ol * fb); hr.process(fr.lp);
    bl[j] = x + hl.hp;
    br[j] = hr.hp;
    if (++j >= d) j = 0;
  }
  return [outL, outR];
}

// Offline brickwall limiter: symmetric window-min + box smoothing guarantees
// the ceiling without delay; one-pole release on top.
export function limit(L, R, ceiling = db(-1), look = 0.004, release = 0.09) {
  const n = L.length, W = Math.max(1, Math.round(look * SR));
  const gr = new Float32Array(n);
  const pl = truePeakEnv(L), pr = truePeakEnv(R);
  for (let i = 0; i < n; i++) { const p = Math.max(pl[i], pr[i]); gr[i] = p > ceiling ? ceiling / p : 1; }
  const gm = slidingMin(gr, W);
  // box average over [i-W, i+W]
  const box = new Float32Array(n);
  let acc = 0; const span = 2 * W + 1;
  for (let i = -W; i <= W; i++) acc += gm[clamp(i, 0, n - 1)];
  for (let i = 0; i < n; i++) {
    box[i] = acc / span;
    acc += gm[Math.min(n - 1, i + W + 1)] - gm[Math.max(0, i - W)];
  }
  const rc = 1 - Math.exp(-1 / (release * SR));
  let g = 1;
  for (let i = 0; i < n; i++) {
    const t = box[i];
    g = t < g ? t : g + (t - g) * rc;
    L[i] *= g; R[i] *= g;
  }
  for (let i = 0; i < n; i++) { L[i] = clamp(L[i], -ceiling, ceiling); R[i] = clamp(R[i], -ceiling, ceiling); }
}
// Per-sample 4x-oversampled peak (windowed-sinc interpolation, 16 taps),
// so the limiter also catches inter-sample overs.
const TP_TAPS = (() => {
  const out = [];
  for (let ph = 1; ph < 4; ph++) {
    const f = ph / 4, taps = [];
    for (let k = -7; k <= 8; k++) {
      const x = k - f, w = 0.5 + 0.5 * Math.cos(Math.PI * x / 8.5);
      taps.push(x === 0 ? 1 : Math.sin(Math.PI * x) / (Math.PI * x) * w);
    }
    out.push(taps);
  }
  return out;
})();
export function truePeakEnv(x) {
  const n = x.length, out = new Float32Array(n);
  for (let i = 0; i < n; i++) {
    let p = Math.abs(x[i]);
    if (i >= 7 && i + 8 < n) {
      for (const taps of TP_TAPS) {
        let acc = 0;
        for (let k = 0; k < 16; k++) acc += x[i - 7 + k] * taps[k];
        const a = Math.abs(acc); if (a > p) p = a;
      }
    }
    out[i] = p;
  }
  return out;
}
export function truePeak(L, R) { let p = 0; for (const ch of [L, R]) for (const v of truePeakEnv(ch)) if (v > p) p = v; return p; }
function slidingMin(a, W) {
  const n = a.length, out = new Float32Array(n), dq = new Int32Array(n + 2 * W + 2);
  let h = 0, t = 0;
  const at = (i) => a[clamp(i, 0, n - 1)];
  for (let i = -W; i < n + W; i++) {
    const v = at(i);
    while (t > h && at(dq[t - 1] ) >= v) t--;
    dq[t++] = i;
    const c = i - W; // center
    if (c >= 0) {
      while (dq[h] < c - W) h++;
      out[c] = at(dq[h]);
    }
  }
  return out;
}

// Integrated loudness (ITU-R BS.1770-4) at 48 kHz.
export function lufs(L, R) {
  const k = () => [new Biquad([1.53512485958697, -2.69169618940638, 1.19839281085285], [1, -1.69065929318241, 0.73248077421585]),
    new Biquad([1, -2, 1], [1, -1.99004745483398, 0.99007225036621])];
  const [a1, a2] = k(), [b1, b2] = k();
  const n = L.length, sq = new Float64Array(n);
  for (let i = 0; i < n; i++) { const l = a2.process(a1.process(L[i])), r = b2.process(b1.process(R[i])); sq[i] = l * l + r * r; }
  const bl = Math.round(0.4 * SR), hop = Math.round(0.1 * SR);
  const pre = new Float64Array(n + 1);
  for (let i = 0; i < n; i++) pre[i + 1] = pre[i] + sq[i];
  const blocks = [];
  for (let s = 0; s + bl <= n; s += hop) blocks.push((pre[s + bl] - pre[s]) / bl);
  const ld = (z) => -0.691 + 10 * Math.log10(z);
  let g = blocks.filter((z) => ld(z) > -70);
  if (!g.length) return -Infinity;
  const rel = ld(g.reduce((a, b) => a + b, 0) / g.length) - 10;
  g = g.filter((z) => ld(z) > rel);
  return ld(g.reduce((a, b) => a + b, 0) / g.length);
}

// Normalize to a loudness target through the limiter.
export function master(L, R, target = -14, ceilingDb = -1) {
  let gain = 1;
  let oL, oR;
  for (let it = 0; it < 4; it++) {
    oL = Float32Array.from(L, (v) => v * gain); oR = Float32Array.from(R, (v) => v * gain);
    limit(oL, oR, db(ceilingDb));
    const l = lufs(oL, oR);
    if (Math.abs(l - target) < 0.05) break;
    gain *= db(target - l);
  }
  return [oL, oR];
}

export function writeWav(path, L, R) {
  const n = L.length, buf = Buffer.alloc(44 + n * 4);
  buf.write('RIFF', 0); buf.writeUInt32LE(36 + n * 4, 4); buf.write('WAVE', 8);
  buf.write('fmt ', 12); buf.writeUInt32LE(16, 16); buf.writeUInt16LE(1, 20); buf.writeUInt16LE(2, 22);
  buf.writeUInt32LE(SR, 24); buf.writeUInt32LE(SR * 4, 28); buf.writeUInt16LE(4, 32); buf.writeUInt16LE(16, 34);
  buf.write('data', 36); buf.writeUInt32LE(n * 4, 40);
  const r = rng(0xd17);
  for (let i = 0; i < n; i++) {
    for (let c = 0; c < 2; c++) {
      const d = (r() - r()) / 32768; // TPDF dither
      const v = clamp(Math.round(((c ? R : L)[i] + d) * 32767), -32768, 32767);
      buf.writeInt16LE(v, 44 + i * 4 + c * 2);
    }
  }
  fs.writeFileSync(path, buf);
}

export function readWav(path) {
  const b = fs.readFileSync(path);
  let p = 12, fmt = null;
  while (p < b.length) {
    const id = b.toString('ascii', p, p + 4), sz = b.readUInt32LE(p + 4);
    if (id === 'fmt ') fmt = { ch: b.readUInt16LE(p + 10), sr: b.readUInt32LE(p + 12), bits: b.readUInt16LE(p + 22) };
    if (id === 'data') {
      if (!fmt || fmt.bits !== 16) throw new Error(path + ': need 16-bit PCM');
      const n = Math.floor(sz / (2 * fmt.ch)), L = new Float32Array(n), R = new Float32Array(n);
      for (let i = 0; i < n; i++) {
        L[i] = b.readInt16LE(p + 8 + i * 2 * fmt.ch) / 32768;
        R[i] = fmt.ch > 1 ? b.readInt16LE(p + 8 + i * 2 * fmt.ch + 2) / 32768 : L[i];
      }
      return { L, R, sr: fmt.sr };
    }
    p += 8 + sz + (sz & 1);
  }
  throw new Error(path + ': no data chunk');
}
