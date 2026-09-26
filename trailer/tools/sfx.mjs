#!/usr/bin/env node
// Synthesized one-shot SFX for the trailer cues (see PLAN.md "Cues").
// renderSfx(name, index) -> [L, R] Float32Arrays at 48 kHz, peak ~0.8.
// `index` varies repeated hits deterministically (keyboard clicks etc.).
// CLI: node tools/sfx.mjs <outDir>   writes every cue to <outDir>/<name>.wav
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { SR, TAU, Osc, SVF, rng, hash, mtof, clamp, smooth, panGains, reverb, writeWav } from './lib/dsp.mjs';

const noise = (r) => r() * 2 - 1;
const exp = Math.exp;

// Build a stereo buffer from fn(t, i) -> number | [l, r].
function make(dur, fn) {
  const n = Math.round(dur * SR), L = new Float32Array(n), R = new Float32Array(n);
  for (let i = 0; i < n; i++) {
    const v = fn(i / SR, i);
    if (Array.isArray(v)) { L[i] = v[0]; R[i] = v[1]; } else { L[i] = v; R[i] = v; }
  }
  return [L, R];
}
function addInto(dst, src, at = 0, g = 1, pan = 0) {
  const [gl, gr] = panGains(pan), s = Math.round(at * SR);
  for (let i = 0; i < src[0].length && s + i < dst[0].length; i++) { dst[0][s + i] += src[0][i] * g * gl; dst[1][s + i] += src[1][i] * g * gr; }
  return dst;
}
function silence(dur) { const n = Math.round(dur * SR); return [new Float32Array(n), new Float32Array(n)]; }
function normalize(buf, peak = 0.8) {
  let p = 1e-9; for (const ch of buf) for (const v of ch) p = Math.max(p, Math.abs(v));
  for (const ch of buf) for (let i = 0; i < ch.length; i++) ch[i] *= peak / p;
  // 3 ms fade-out guard
  const n = buf[0].length, f = Math.min(n, 144);
  for (let i = 0; i < f; i++) { const g = i / f; buf[0][n - 1 - i] *= g; buf[1][n - 1 - i] *= g; }
  return buf;
}
function withVerb(buf, amt = 0.3, room = 0.8, tail = 0.8) {
  const n = buf[0].length + Math.round(tail * SR);
  const L = new Float32Array(n), R = new Float32Array(n); L.set(buf[0]); R.set(buf[1]);
  const [wL, wR] = reverb(L, R, { room, damp: 0.35, predelay: 0.012 });
  for (let i = 0; i < n; i++) { L[i] += wL[i] * amt; R[i] += wR[i] * amt; }
  return [L, R];
}
// Square/pulse note with a simple envelope; freq may be a function of t.
function tone(dur, freq, { duty = 0.5, att = 0.002, dec = 0.1, wave = 'pulse', vib = 0, cut = 8000 } = {}) {
  const o = new Osc(0), lp = new SVF(cut, 0.7);
  return make(dur, (t) => {
    const f = (typeof freq === 'function' ? freq(t) : freq) * (1 + vib * Math.sin(TAU * 6 * t));
    const x = wave === 'tri' ? o.nesTri(f) : wave === 'sine' ? o.sine(f) : wave === 'saw' ? o.saw(f) : o.pulse(f, duty);
    return lp.process(x) * Math.min(1, t / att) * exp(-t / dec) * (1 - smooth((t - dur + 0.01) / 0.01));
  });
}

const SFX = {
  // Mechanical keyboard: sharp top click, a woody thock, a small bottom-out tick.
  key(idx) {
    const h = (k) => hash(idx, k), r = rng(idx * 7919 + 1);
    const pitch = 0.9 + 0.2 * h(1), bottom = 0.011 + 0.008 * h(2), bright = 0.8 + 0.4 * h(3);
    const clickBp = new SVF(3800 * bright, 2.2), thockLp = new SVF(1600 * pitch, 0.9), tickBp = new SVF(5200 * bright, 3);
    let p1 = h(4), p2 = h(5);
    const f1 = 310 * pitch, f2 = 720 * pitch;
    const buf = make(0.09, (t) => {
      const n = noise(r);
      const click = clickBp.process(n) * exp(-t / 0.0022) * 2.0;
      p1 += f1 / SR; p2 += f2 / SR;
      const body = (Math.sin(TAU * p1) * 0.8 + Math.sin(TAU * p2) * 0.35) * exp(-t / 0.018) * Math.min(1, t / 0.0008);
      const thock = thockLp.process(n) * exp(-t / 0.012) * 0.7;
      const tb = t - bottom;
      const tick = tb > 0 ? tickBp.process(n) * exp(-tb / 0.0035) * 0.55 : (tickBp.process(0), 0);
      return click + body * 0.4 + thock + tick;
    });
    return normalize(buf, 0.75 + 0.15 * h(6));
  },
  // Return key: bigger, lower, with a stabilizer rattle.
  enter(idx) {
    const r = rng(4242 + idx);
    const bp = new SVF(2600, 2), lp = new SVF(1100, 0.9), rt = new SVF(4200, 4);
    let p = 0;
    const buf = make(0.16, (t) => {
      const n = noise(r);
      p += (190 + 40 * exp(-t / 0.01)) / SR;
      const body = Math.sin(TAU * p) * exp(-t / 0.035);
      const click = bp.process(n) * exp(-t / 0.003) * 1.2;
      const thock = lp.process(n) * exp(-t / 0.02);
      const t2 = t - 0.022, rattle = t2 > 0 ? rt.process(n) * exp(-t2 / 0.012) * (0.6 + 0.4 * Math.sin(TAU * 90 * t2)) : (rt.process(0), 0);
      return click + body * 0.8 + thock * 0.7 + rattle * 0.6;
    });
    return normalize(buf, 0.85);
  },
  // Text tick: tiny chip blip, slightly varied.
  type(idx) {
    const f = [1568, 1760, 1976, 1661][Math.floor(hash(idx, 11) * 4)];
    return normalize(tone(0.035, f, { duty: 0.25, dec: 0.012, cut: 6000 }), 0.55);
  },
  whoosh(idx) {
    const r = rng(77 + idx), bl = new SVF(400, 1.4), br = new SVF(420, 1.4);
    const dur = 0.5;
    const buf = make(dur, (t, i) => {
      const u = t / dur, fc = 500 + 3200 * Math.sin(Math.PI * Math.pow(u, 0.8));
      if (i % 16 === 0) { bl.set(fc, 1.4); br.set(fc * 1.1, 1.4); }
      const env = Math.pow(Math.sin(Math.PI * Math.pow(u, 0.7)), 2);
      const pan = -0.8 + 1.6 * u, [gl, gr] = panGains(pan);
      return [bl.process(noise(r)) * env * gl, br.process(noise(r)) * env * gr];
    });
    return normalize(buf, 0.7);
  },
  // One-bar riser that peaks at its end (cue marks the start).
  riser(idx) {
    const r = rng(99 + idx), dur = 1.875, bl = new SVF(300, 2), br = new SVF(300, 2), lp = new SVF(500, 1);
    const o1 = new Osc(0), o2 = new Osc(0.4);
    const buf = make(dur, (t, i) => {
      const u = t / dur;
      if (i % 16 === 0) { const fc = 300 * Math.pow(30, u); bl.set(fc, 2.4); br.set(fc * 1.07, 2.4); lp.set(600 + 5000 * u * u, 1); }
      const f = mtof(52 + 30 * u * u);
      const s = lp.process(o1.pulse(f, 0.25 + 0.2 * u) + o2.pulse(f * 1.01, 0.5)) * 0.3;
      const flutter = 1 + 0.3 * Math.sin(TAU * (4 + 24 * u * u) * t);
      const env = u * u * flutter * (1 - smooth((u - 0.97) / 0.03));
      return [(bl.process(noise(r)) * 1.3 + s) * env, (br.process(noise(r)) * 1.3 + s) * env];
    });
    return normalize(buf, 0.7);
  },
  // Big title/logo hit: sub thump + crack + metallic clang + room.
  slam(idx) {
    const r = rng(501 + idx), hp = new SVF(1800, 0.8), lp = new SVF(300, 0.8);
    const metal = [310, 457, 653, 881, 1197].map((f, k) => ({ o: new Osc(hash(k, 3)), f }));
    let p = 0;
    const dry = make(0.9, (t) => {
      p += (42 + 120 * exp(-t / 0.04)) / SR;
      const sub = Math.sin(TAU * p) * exp(-t / 0.3) * 1.2;
      const n = noise(r);
      const crack = hp.process(n) * exp(-t / 0.03) * 1.1;
      const body = lp.process(n) * exp(-t / 0.12) * 1.5;
      let m = 0; for (const x of metal) m += x.o.pulse(x.f, 0.5);
      const clang = m * 0.12 * exp(-t / 0.18);
      return Math.tanh(1.6 * (sub + crack + body + clang));
    });
    return normalize(withVerb(dry, 0.45, 0.82, 0.7), 0.9);
  },
  // Heavy boom: long sub drop + rumble.
  impact(idx) {
    const r = rng(611 + idx), lp = new SVF(400, 0.8), hp = new SVF(2500, 0.7);
    let p = 0;
    const dry = make(1.8, (t, i) => {
      p += (28 + 70 * exp(-t / 0.2)) / SR;
      if (i % 16 === 0) lp.set(90 + 900 * exp(-t / 0.1), 0.8);
      const n = noise(r);
      const sub = Math.sin(TAU * p) * exp(-t / 0.6);
      const rum = lp.process(n) * exp(-t / 0.5) * 1.4;
      const hit = hp.process(n) * exp(-t / 0.015) * 0.6;
      return Math.tanh(1.4 * (sub + rum + hit)) * (1 - smooth((t - 1.6) / 0.2));
    });
    return normalize(withVerb(dry, 0.3, 0.85, 0.6), 0.9);
  },
  // Small letter landing: soft thud + tick.
  land(idx) {
    const r = rng(33 + idx), bp = new SVF(3000, 2);
    const pf = 1 + 0.12 * (hash(idx, 2) - 0.5);
    let p = 0;
    return normalize(make(0.13, (t) => {
      p += (70 + 110 * exp(-t / 0.02)) * pf / SR;
      return Math.tanh(1.3 * (Math.sin(TAU * p) * exp(-t / 0.05) + bp.process(noise(r)) * exp(-t / 0.006) * 0.8));
    }), 0.7);
  },
  // Character appears: bubbly upward pop.
  pop(idx) {
    const k = 1 + 0.15 * (hash(idx, 4) - 0.5);
    const a = tone(0.09, (t) => (380 + 1600 * (1 - exp(-t / 0.025))) * k, { duty: 0.5, dec: 0.035, cut: 5000 });
    const b = tone(0.09, (t) => (760 + 3200 * (1 - exp(-t / 0.025))) * k, { wave: 'sine', dec: 0.03 });
    return normalize(addInto(a, b, 0, 0.4), 0.7);
  },
  // Sparkle: quick rising bell arpeggio with shimmer.
  shine(idx) {
    const out = silence(0.8);
    const notes = [88, 95, 100, 104, 107];
    notes.forEach((m, k) => {
      const f = mtof(m);
      const bell = make(0.45, (t) => (Math.sin(TAU * f * t) + 0.35 * Math.sin(TAU * f * 2.76 * t) * exp(-t / 0.05)) * exp(-t / 0.14) * Math.min(1, t / 0.001));
      addInto(out, bell, k * 0.045, 0.6, (k % 2 ? 0.5 : -0.5));
    });
    return normalize(withVerb(out, 0.35, 0.8, 0.4), 0.6);
  },
  // UI move: short NES blip.
  blip(idx) {
    const f = hash(idx, 8) < 0.5 ? 988 : 1047;
    return normalize(tone(0.06, f, { duty: 0.25, dec: 0.03, cut: 7000 }), 0.6);
  },
  // Card pick: two-note up-chime.
  select(idx) {
    const out = silence(0.35);
    addInto(out, tone(0.1, mtof(84), { duty: 0.5, dec: 0.06, cut: 7000 }), 0);
    addInto(out, tone(0.25, mtof(91), { duty: 0.25, dec: 0.1, cut: 7000, vib: 0.004 }), 0.07);
    addInto(out, tone(0.25, mtof(103), { wave: 'sine', dec: 0.08 }), 0.07, 0.3);
    return normalize(withVerb(out, 0.2, 0.7, 0.3), 0.7);
  },
  // Currency: coin-like, hollow and bony (triangle + knock).
  bone(idx) {
    const up = hash(idx, 5) < 0.5 ? 0 : 2;
    const out = silence(0.3);
    const r = rng(9 + idx), bp = new SVF(1800, 3);
    addInto(out, make(0.02, (t) => bp.process(noise(r)) * exp(-t / 0.004)), 0, 0.8);
    addInto(out, tone(0.06, mtof(83 + up), { wave: 'tri', dec: 0.05 }), 0);
    addInto(out, tone(0.22, mtof(88 + up), { wave: 'tri', dec: 0.09 }), 0.055);
    addInto(out, tone(0.22, mtof(88 + up), { duty: 0.125, dec: 0.05, cut: 5000 }), 0.055, 0.25);
    return normalize(out, 0.65);
  },
  // Lightning: descending buzz with crackle.
  zap(idx) {
    const r = rng(1234 + idx), hp = new SVF(1500, 0.8);
    const o = new Osc(0);
    let gate = 1;
    const buf = make(0.4, (t, i) => {
      if (i % 240 === 0) gate = r() < 0.7 ? 1 : 0.2;
      const f = 2800 * exp(-t / 0.08) + 120;
      const saw = o.saw(f) * 0.6;
      const cr = hp.process(noise(r)) * gate * exp(-t / 0.12);
      const env = Math.min(1, t / 0.001) * exp(-t / 0.15);
      return Math.tanh(1.5 * (saw * env + cr * 0.9));
    });
    return normalize(buf, 0.7);
  },
  // Explosion: crunchy 8-bit noise burst with sub.
  boom(idx) {
    const r = rng(808 + idx), lp = new SVF(2000, 0.7);
    let held = 0, p = 0;
    const dry = make(0.9, (t, i) => {
      const hold = Math.round(2 + 20 * t);
      if (i % hold === 0) held = noise(r);
      if (i % 16 === 0) lp.set(300 + 3500 * exp(-t / 0.08), 0.7);
      p += (45 + 80 * exp(-t / 0.05)) / SR;
      const crunch = Math.round(lp.process(held) * 8) / 8;
      return Math.tanh(1.4 * (crunch * exp(-t / 0.25) + Math.sin(TAU * p) * exp(-t / 0.2)));
    });
    return normalize(withVerb(dry, 0.2, 0.7, 0.4), 0.8);
  },
  // Dash: quick upward swoosh + pitch sweep.
  dash(idx) {
    const r = rng(55 + idx), bp = new SVF(1000, 1.5);
    const o = new Osc(0);
    const buf = make(0.28, (t, i) => {
      const u = t / 0.28;
      if (i % 16 === 0) bp.set(800 + 6000 * u, 1.5);
      const sq = o.pulse(300 + 1400 * Math.sqrt(u), 0.25) * exp(-t / 0.06) * 0.35;
      const air = bp.process(noise(r)) * Math.sin(Math.PI * u);
      const [gl, gr] = panGains(-0.4 + 0.8 * u);
      return [(sq + air) * gl, (sq + air) * gr];
    });
    return normalize(buf, 0.65);
  },
  // Digital stutter.
  glitch(idx) {
    const r = rng(666 + idx);
    const o = new Osc(0);
    let f = 400, mode = 0, held = 0;
    const buf = make(0.36, (t, i) => {
      if (i % 1100 === 0) { f = 200 + r() * 2400; mode = Math.floor(r() * 3); }
      if (i % 12 === 0) held = noise(r);
      const x = mode === 0 ? o.pulse(f, 0.125) : mode === 1 ? held * 0.8 : o.pulse(f * 0.5, 0.5) * (Math.floor(t * 60) % 2);
      const env = t < 0.33 ? 1 : 1 - (t - 0.33) / 0.03;
      const pan = mode === 1 ? -0.5 : 0.5, [gl, gr] = panGains(pan);
      return [x * env * gl * 0.7, x * env * gr * 0.7];
    });
    return normalize(buf, 0.6);
  },
  // Pause: classic two-note chime, then a tape-stop whirr down.
  pause(idx) {
    const out = silence(1.0);
    addInto(out, tone(0.12, mtof(88), { duty: 0.5, dec: 0.08, cut: 6000 }), 0);
    addInto(out, tone(0.12, mtof(83), { duty: 0.5, dec: 0.08, cut: 6000 }), 0.09);
    addInto(out, tone(0.3, mtof(88), { duty: 0.25, dec: 0.12, cut: 6000 }), 0.18);
    const o = new Osc(0), lp = new SVF(2000, 0.7);
    const whirr = make(0.5, (t) => {
      const u = t / 0.5, f = 440 * Math.pow(1 - u, 1.6) + 30;
      if (Math.round(t * SR) % 16 === 0) lp.set(200 + 3000 * (1 - u), 0.7);
      return lp.process(o.saw(f)) * (1 - smooth((u - 0.6) / 0.4)) * 0.5;
    });
    addInto(out, whirr, 0.02, 0.5);
    return normalize(withVerb(out, 0.3, 0.8, 0.5), 0.65);
  },
  // Agent done: warm ascending major triad on triangle + sine.
  done(idx) {
    const out = silence(0.9);
    [84, 88, 91, 96].forEach((m, k) => {
      addInto(out, tone(0.5, mtof(m), { wave: 'tri', dec: 0.22 }), k * 0.07, 0.7, (k - 1.5) * 0.25);
      addInto(out, tone(0.5, mtof(m + 12), { wave: 'sine', dec: 0.12 }), k * 0.07, 0.25, (k - 1.5) * 0.25);
    });
    return normalize(withVerb(out, 0.3, 0.8, 0.5), 0.6);
  },
};

export const NAMES = Object.keys(SFX);

// Default mix level per cue (linear, before the cue's own gain).
export const LEVEL = {
  key: 0.32, enter: 0.45, type: 0.22, whoosh: 0.4, riser: 0.35, slam: 1.0, impact: 1.0, land: 0.45,
  pop: 0.4, shine: 0.35, blip: 0.3, select: 0.45, bone: 0.35, zap: 0.5, boom: 0.6, dash: 0.4,
  glitch: 0.4, pause: 0.6, done: 0.55,
};

const cache = new Map();
export function renderSfx(name, idx = 0) {
  const fn = SFX[name];
  if (!fn) return null;
  const key = name + ':' + idx;
  if (!cache.has(key)) cache.set(key, fn(idx));
  return cache.get(key);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const dir = process.argv[2];
  if (!dir) { console.log('usage: node tools/sfx.mjs <outDir>'); process.exit(0); }
  fs.mkdirSync(dir, { recursive: true });
  for (const n of NAMES) { const [L, R] = renderSfx(n, 0); writeWav(path.join(dir, n + '.wav'), L, R); }
  for (let i = 1; i < 4; i++) { const [L, R] = renderSfx('key', i); writeWav(path.join(dir, `key-${i}.wav`), L, R); }
  console.log('sfx: wrote', NAMES.length, 'cues to', dir);
}
