#!/usr/bin/env node
// trex trailer score: modern chiptune, A minor, 128 BPM, 40 bars (75 s).
// Deterministic synth + sequencer -> assets/audio/music.wav (48 kHz stereo 16-bit).
// Usage: node tools/music.mjs [out.wav]
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  SR, TAU, Stereo, Osc, SVF, Biquad, rng, hash, mtof, db, clamp, smooth,
  reverb, pingpong, master, writeWav, panGains,
} from './lib/dsp.mjs';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const OUT = process.argv[2] || path.join(HERE, '../assets/audio/music.wav');

const DUR = 75, N = DUR * SR;
const BEAT = 60 / 128, BAR = BEAT * 4, STEP = BAR / 16;
const T = (bar, step = 0) => (bar - 1) * BAR + step * STEP;
const BREATH = [T(6, 12), T(7)];      // near-silent last beat of bar 6
const TAPE = T(37);                    // tape-stop (the PAUSE moment)
const FINAL = T(39);

// ---------------------------------------------------------------- harmony
// Pad voicings are voice-led around A3..F4; b = bass root (octave 1/2).
const CH = {
  Am: { b: 33, t: [57, 60, 64] }, F: { b: 29, t: [57, 60, 65] }, C: { b: 36, t: [55, 60, 64] },
  G: { b: 31, t: [55, 59, 62] }, E: { b: 28, t: [56, 59, 64] }, Bb: { b: 34, t: [58, 62, 65] },
  A: { b: 33, t: [57, 61, 64] },
};
const PROG = {
  1: 'Am', 2: 'F', 3: 'Am', 4: 'F', 5: 'C', 6: 'G',
  7: 'Am', 8: 'F', 9: 'C', 10: 'G', 11: 'Am', 12: 'F', 13: 'C', 14: ['G', 'E'],
  15: 'Am', 16: 'F', 17: 'C', 18: ['G', 'E'], 19: 'F', 20: 'G',
  21: 'C', 22: 'G', 23: 'Am', 24: 'F',
  25: 'Am', 26: 'Bb', 27: 'F', 28: 'E',
  29: 'Am', 30: 'F', 31: 'C', 32: ['G', 'E'],
  33: 'F', 34: 'G', 35: 'Am', 36: 'F', 37: 'C', 38: 'G', 39: ['F', 'G'], 40: 'A',
};
const chordAt = (bar, step = 0) => { const c = PROG[bar]; return CH[Array.isArray(c) ? c[step < 8 ? 0 : 1] : c]; };
const section = (bar) => bar <= 6 ? 'intro' : bar <= 20 ? 'drop' : bar <= 24 ? 'heroes' : bar <= 28 ? 'boss'
  : bar <= 32 ? 'climax' : bar <= 34 ? 'break' : bar <= 38 ? 'outro' : 'final';

// ---------------------------------------------------------------- melody
// [step, midi, lengthInSteps, slideSeconds?] on a 16th grid.
const HOOK = {
  // call: A C E-D E G E (Am)
  h1: [[0, 69, 2], [2, 72, 2], [4, 76, 3], [7, 74, 1], [8, 76, 2], [10, 79, 2], [12, 76, 4]],
  // answer, falling (F)
  h2: [[0, 77, 3], [3, 76, 1], [4, 72, 2], [6, 69, 2], [8, 72, 3], [11, 74, 1], [12, 72, 2], [14, 69, 2]],
  // call again, lifting to C6 (C)
  h3: [[0, 67, 2], [2, 72, 2], [4, 76, 3], [7, 74, 1], [8, 76, 2], [10, 79, 2], [12, 84, 4, 0.05]],
  // turnaround (G)
  h4: [[0, 83, 3, 0.03], [3, 81, 1], [4, 79, 2], [6, 74, 2], [8, 79, 4], [12, 74, 2], [14, 71, 2]],
  // turnaround into E, leading tone back to A (G|E)
  h4e: [[0, 83, 3, 0.03], [3, 81, 1], [4, 79, 2], [6, 74, 2], [8, 80, 3], [11, 76, 1], [12, 71, 2], [14, 68, 2]],
  // bars 19-20: lift over F, rising G7 run into the heroes section
  f19: [[0, 77, 4], [4, 76, 2], [6, 77, 2], [8, 81, 4, 0.04], [12, 79, 4]],
  f20: [[0, 74, 2], [2, 79, 2], [4, 83, 2], [6, 86, 2, 0.03], [8, 84, 1], [9, 83, 1], [10, 81, 1], [11, 79, 1], [12, 77, 2], [14, 74, 2]],
  // heroes: bouncy B melody, a hit on every quarter
  b1: [[0, 72, 2], [3, 76, 1], [4, 79, 2], [7, 76, 1], [8, 84, 3], [11, 83, 1], [12, 79, 2], [14, 76, 2]],
  b2: [[0, 74, 2], [3, 79, 1], [4, 83, 2], [7, 79, 1], [8, 86, 3], [11, 84, 1], [12, 83, 2], [14, 79, 2]],
  b3: [[0, 76, 2], [3, 81, 1], [4, 84, 2], [7, 83, 1], [8, 81, 3], [11, 79, 1], [12, 76, 2], [14, 72, 2]],
  b4: [[0, 77, 2], [3, 81, 1], [4, 84, 2], [7, 81, 1], [8, 79, 2], [10, 81, 2], [12, 79, 2], [14, 80, 2]],
  // bosses: slow, heavy half-time line
  k1: [[0, 76, 8, 0.06], [8, 74, 4], [12, 72, 4]],
  k2: [[0, 74, 8, 0.06], [8, 77, 8, 0.05]],
  k3: [[0, 72, 8, 0.06], [8, 69, 8]],
  k4: [[0, 71, 8, 0.06], [8, 68, 8, 0.04]],
  // final statement over F|G, landing on A
  fin: [[0, 69, 2], [2, 72, 2], [4, 76, 3], [7, 74, 1], [8, 74, 2], [10, 79, 2], [12, 83, 4, 0.03]],
  end: [[0, 81, 16, 0.04]],
};
const LEAD = {
  3: 'h1', 4: 'h2', 5: 'h3', 6: 'h4',
  7: 'h1', 8: 'h2', 9: 'h3', 10: 'h4', 11: 'h1', 12: 'h2', 13: 'h3', 14: 'h4e',
  15: 'h1', 16: 'h2', 17: 'h3', 18: 'h4e', 19: 'f19', 20: 'f20',
  21: 'b1', 22: 'b2', 23: 'b3', 24: 'b4', 25: 'k1', 26: 'k2', 27: 'k3', 28: 'k4',
  29: 'h1', 30: 'h2', 31: 'h3', 32: 'h4e', 35: 'h1', 36: 'h2', 37: 'h3', 38: 'h4', 39: 'fin', 40: 'end',
};
// Diatonic shift inside A minor (G# treated as G) for harmony lines.
const SCALE = [0, 2, 3, 5, 7, 8, 10]; // A B C D E F G relative to A
function diatonic(m, deg) {
  const rel = m - 57, oct = Math.floor(rel / 12); let pc = ((rel % 12) + 12) % 12;
  if (pc === 11) pc = 10;
  let idx = SCALE.indexOf(pc); if (idx < 0) idx = SCALE.findIndex((s) => s > pc) - 1;
  const j = idx + deg, o2 = oct + Math.floor(j / 7), k = ((j % 7) + 7) % 7;
  return 57 + o2 * 12 + SCALE[k];
}

// ---------------------------------------------------------------- buses
// sc: sidechain depth from the kick; rev/dly: effect sends.
const BUSES = {
  kick: { gain: 0.72, sc: 0, rev: 0.0, dly: 0 },
  snare: { gain: 1.1, sc: 0, rev: 0.22, dly: 0 },
  hats: { gain: 1.0, sc: 0.25, rev: 0.05, dly: 0 },
  cym: { gain: 0.55, sc: 0.3, rev: 0.25, dly: 0 },
  bass: { gain: 0.7, sc: 0.8, rev: 0, dly: 0 },
  pad: { gain: 0.45, hp: 180, sc: 0.65, rev: 0.35, dly: 0 },
  arp: { gain: 0.5, hp: 200, sc: 0.5, rev: 0.18, dly: 0.28 },
  stab: { gain: 0.45, hp: 160, sc: 0.35, rev: 0.2, dly: 0.12 },
  lead: { gain: 0.5, sc: 0.15, rev: 0.2, dly: 0.2 },
  harm: { gain: 0.3, hp: 150, sc: 0.2, rev: 0.25, dly: 0.15 },
  fx: { gain: 0.5, sc: 0, rev: 0.3, dly: 0 },
  boom: { gain: 0.65, sc: 0, rev: 0.25, dly: 0 },
};

class Ctx {
  constructor() {
    this.bus = {}; for (const k in BUSES) this.bus[k] = new Stereo(N);
    this.kicks = []; // [time, strength]
    this.r = rng(1337);
  }
}
const S = (t) => Math.round(t * SR);
const noise = (r) => r() * 2 - 1;

// ---------------------------------------------------------------- drums
function kick(c, t, v = 1, sc = 1) {
  const b = c.bus.kick, s0 = S(t), n = S(0.42); let ph = 0; const r = rng(S(t) + 7);
  const hp = new SVF(3000, 0.7);
  for (let i = 0; i < n; i++) {
    const tt = i / SR;
    const f = 50 + 150 * Math.exp(-tt / 0.03) + 60 * Math.exp(-tt / 0.006);
    ph += f / SR;
    const amp = Math.min(1, tt / 0.001) * (0.75 * Math.exp(-tt / 0.22) + 0.25 * Math.exp(-tt / 0.06));
    hp.process(noise(r));
    const click = hp.hp * Math.exp(-tt / 0.003) * 0.6;
    const x = Math.tanh(1.8 * (Math.sin(TAU * ph) * amp + click)) * (1 - smooth((tt - 0.36) / 0.06));
    b.add(s0 + i, x * v, 0);
  }
  if (sc > 0) c.kicks.push([t, sc]);
}
function snare(c, t, v = 1, o = {}) {
  const b = c.bus.snare, s0 = S(t), n = S(o.len || 0.32); const r = rng(S(t) + 11);
  const fl = new SVF(1700, 0.8), fr = new SVF(1800, 0.8), bp = new SVF(1200, 1.5);
  let ph = 0, ph2 = 0;
  const pan = o.pan || 0;
  for (let i = 0; i < n; i++) {
    const tt = i / SR;
    const f = 190 * (1 + 0.45 * Math.exp(-tt / 0.012));
    ph += f / SR; ph2 += f * 1.52 / SR;
    const tone = (Math.sin(TAU * ph) + 0.4 * Math.sin(TAU * ph2)) * Math.exp(-tt / 0.055) * 0.8;
    const nl = noise(r), nr = noise(r);
    fl.process(nl); fr.process(nr); bp.process((nl + nr) * 0.5);
    const decay = Math.exp(-tt / (o.decay || 0.13));
    const clap = o.clap ? bp.bp * (tt < 0.03 ? (Math.floor(tt / 0.009) % 2 === 0 ? 1.4 : 0.3) : Math.exp(-(tt - 0.03) / 0.08)) : 0;
    const l = Math.tanh(1.4 * (tone + (fl.hp * 0.8 + clap) * decay));
    const rr = Math.tanh(1.4 * (tone + (fr.hp * 0.8 + clap) * decay));
    const [gl, gr] = panGains(pan);
    b.addLR(s0 + i, l * v * gl, rr * v * gr);
  }
}
const METAL = [205.3, 304.4, 369.6, 522.7, 540, 800];
function hat(c, t, v = 1, open = false, pan = 0.15) {
  const b = c.bus.hats, s0 = S(t), n = S(open ? 0.4 : 0.07); const r = rng(S(t) + 3);
  const osc = METAL.map((_, k) => new Osc(hash(k, 5)));
  const bp = new SVF(9500, 1.2), hp = new SVF(7000, 0.7);
  const dec = open ? 0.16 : 0.028;
  for (let i = 0; i < n; i++) {
    const tt = i / SR;
    let m = 0; for (let k = 0; k < 6; k++) m += osc[k].pulse(METAL[k] * 1.7, 0.5);
    bp.process(m * 0.15 + noise(r) * 0.6);
    hp.process(bp.bp);
    const env = Math.min(1, tt / 0.0006) * Math.exp(-tt / dec);
    b.add(s0 + i, hp.hp * env * v * 1.6, pan);
  }
}
function crash(c, t, v = 1, reverse = false) {
  const b = c.bus.cym, len = 1.8, n = S(len); const r = rng(S(t) + 99);
  const L = new Float32Array(n), R = new Float32Array(n);
  const osc = METAL.map((_, k) => new Osc(hash(k, 9)));
  const hl = new SVF(3500, 0.7), hr = new SVF(3600, 0.7);
  for (let i = 0; i < n; i++) {
    const tt = i / SR;
    let m = 0; for (let k = 0; k < 6; k++) m += osc[k].pulse(METAL[k] * 2.3, 0.5);
    const env = Math.min(1, tt / 0.002) * (0.6 * Math.exp(-tt / 0.5) + 0.4 * Math.exp(-tt / 0.08)) * (1 - smooth((tt - len + 0.2) / 0.2));
    hl.process(noise(r) + m * 0.08); hr.process(noise(r) + m * 0.08);
    L[i] = hl.hp * env * 0.9; R[i] = hr.hp * env * 0.9;
  }
  if (!reverse) for (let i = 0; i < n; i++) b.addLR(S(t) + i, L[i] * v, R[i] * v);
  else { // swell that ends at t
    const s1 = S(t) - n;
    for (let i = 0; i < n; i++) { const k = n - 1 - i, w = smooth(i / (n * 0.3)); b.addLR(s1 + i, L[k] * v * w, R[k] * v * w); }
  }
}
function tom(c, t, v, f0, pan = 0) {
  const b = c.bus.snare, s0 = S(t), n = S(0.3); let ph = 0; const r = rng(S(t) + 21);
  for (let i = 0; i < n; i++) {
    const tt = i / SR;
    ph += f0 * (1 + 0.6 * Math.exp(-tt / 0.04)) / SR;
    const x = (Math.sin(TAU * ph) + noise(r) * 0.15 * Math.exp(-tt / 0.01)) * Math.exp(-tt / 0.16);
    b.add(s0 + i, Math.tanh(1.5 * x) * v, pan);
  }
}
function impact(c, t, v = 1) {
  const b = c.bus.boom, s0 = S(t), n = S(1.7); let ph = 0; const r = rng(S(t) + 5);
  const lp = new SVF(500, 0.8);
  for (let i = 0; i < n; i++) {
    const tt = i / SR;
    ph += (30 + 65 * Math.exp(-tt / 0.18)) / SR;
    const sub = Math.sin(TAU * ph) * Math.exp(-tt / 0.55);
    lp.set(120 + 1400 * Math.exp(-tt / 0.08));
    const rum = lp.process(noise(r)) * Math.exp(-tt / 0.35) * 1.3;
    const x = Math.tanh(1.3 * (sub + rum)) * (1 - smooth((tt - 1.5) / 0.2));
    b.add(s0 + i, x * v, 0);
  }
}
// Noise + saw riser from t0 to t1.
function riser(c, t0, t1, v = 1) {
  const b = c.bus.fx, s0 = S(t0), n = S(t1 - t0); const r = rng(S(t0) + 17);
  const bl = new SVF(300, 2), br = new SVF(300, 2), lp = new SVF(800, 0.9);
  const o1 = new Osc(0), o2 = new Osc(0.5);
  for (let i = 0; i < n; i++) {
    const u = i / n;
    const fc = 250 * Math.pow(40, u);
    if (i % 16 === 0) { bl.set(fc, 2.2); br.set(fc * 1.08, 2.2); lp.set(400 + 5000 * u * u, 0.9); }
    bl.process(noise(r)); br.process(noise(r));
    const f = mtof(45 + 36 * u * u);
    const saw = lp.process(o1.saw(f) + o2.saw(f * 1.012)) * 0.35;
    const env = u * u * (1 - smooth((u - 0.985) / 0.015));
    b.addLR(s0 + i, (bl.bp * 1.4 + saw) * env * v, (br.bp * 1.4 + saw) * env * v);
  }
}

// ---------------------------------------------------------------- tonal
function bassNote(c, t, dur, m, v = 1, o = {}) {
  const b = c.bus.bass, s0 = S(t), n = S(dur + 0.03);
  const o1 = new Osc(0), o2 = new Osc(0.33), sub = new Osc(0); const f = mtof(m);
  const lp = new SVF(600, 1.1);
  const base = o.cut ?? 380, env = o.env ?? 1400, drive = o.drive ?? 1.4;
  for (let i = 0; i < n; i++) {
    const tt = i / SR;
    if (i % 8 === 0) lp.set(base + env * Math.exp(-tt / (o.fdec || 0.07)), 1.1);
    const saw = o1.saw(f * 1.0052) + o2.saw(f * 0.9948);
    const y = lp.process(saw * 0.5);
    const amp = Math.min(1, tt / 0.003) * (tt < dur ? 1 : Math.max(0, 1 - (tt - dur) / 0.03));
    const x = Math.tanh(drive * (y * (o.sawMix ?? 1) + sub.sine(f) * (o.sub ?? 0.8))) * amp;
    b.add(s0 + i, x * v, 0);
  }
}
function padChord(c, t, dur, notes, v = 1, o = {}) {
  const b = c.bus[o.bus || 'pad'], s0 = S(t), rel = o.rel ?? 0.35, n = S(dur + rel);
  const att = o.att ?? 0.08;
  const voices = [];
  notes.forEach((m, k) => {
    for (const d of [-1, 1]) voices.push({ o: new Osc(hash(k * 2 + d, S(t))), f: mtof(m) * Math.pow(2, d * (o.detune ?? 9) / 1200), pan: d * 0.55 * (o.width ?? 1) });
  });
  const fl = new SVF(800, 0.8), fr = new SVF(800, 0.8);
  const c0 = o.cut0 ?? 1800, c1 = o.cut1 ?? c0;
  const g = v / Math.sqrt(voices.length);
  for (let i = 0; i < n; i++) {
    const tt = i / SR;
    if (i % 32 === 0) { const fc = c0 * Math.pow(c1 / c0, clamp(tt / dur, 0, 1)); fl.set(fc, 0.8); fr.set(fc * 1.05, 0.8); }
    let l = 0, r = 0;
    for (const vo of voices) {
      const x = o.pulse ? vo.o.pulse(vo.f, 0.3) : vo.o.saw(vo.f);
      const [gl, gr] = panGains(vo.pan); l += x * gl; r += x * gr;
    }
    const env = smooth(tt / att) * (tt < dur ? 1 : Math.exp(-(tt - dur) / (rel / 4)));
    b.addLR(s0 + i, fl.process(l) * env * g, fr.process(r) * env * g);
  }
}
function arpNote(c, t, dur, m, v, pan, o = {}) {
  const b = c.bus.arp, s0 = S(t), n = S(dur * 1.6 + 0.02); const os = new Osc(0); const f = mtof(m);
  const lp = new SVF(o.cut ?? 5000, 0.9);
  for (let i = 0; i < n; i++) {
    const tt = i / SR;
    const env = Math.min(1, tt / 0.002) * Math.exp(-tt / (o.dec ?? 0.09));
    const x = o.tri ? os.nesTri(f) : os.pulse(f, o.duty ?? 0.25);
    b.add(s0 + i, lp.process(x) * env * v, pan);
  }
}
function stab(c, t, notes, v, dur, o = {}) {
  const b = c.bus.stab, s0 = S(t), n = S(dur + 0.05);
  const vs = notes.map((m, k) => ({ o: new Osc(hash(k, 3)), f: mtof(m), pan: (k - (notes.length - 1) / 2) * 0.35 }));
  const lp = new SVF(4000, 1);
  for (let i = 0; i < n; i++) {
    const tt = i / SR;
    if (i % 16 === 0) lp.set((o.cut ?? 1200) + (o.env ?? 5000) * Math.exp(-tt / (o.fdec ?? 0.06)), 1);
    const env = Math.min(1, tt / 0.002) * Math.exp(-tt / (o.dec ?? 0.1)) * (tt < dur ? 1 : Math.max(0, 1 - (tt - dur) / 0.05));
    let l = 0, r = 0;
    for (const x of vs) { const s = o.saw ? x.o.saw(x.f) + 0.6 * x.o.saw(x.f * 1.007) : x.o.pulse(x.f, 0.4); const [gl, gr] = panGains(x.pan); l += s * gl; r += s * gr; }
    const g = env * v / Math.sqrt(vs.length);
    const y = lp.process((l + r) * 0.5) * g, side = (l - r) * 0.5 * g * 0.6;
    b.addLR(s0 + i, y + side, y - side);
  }
}

// Monophonic chip lead: continuous phase, legato slides, delayed vibrato,
// duty sweep (PWM), optional detuned and octave layers.
function mono(c, busName, notes, o = {}) {
  const b = c.bus[busName];
  const oa = new Osc(0), ob = new Osc(0.37), oc = new Osc(0.71);
  const lp = new SVF(o.cut ?? 6000, 0.7);
  let prev = null;
  notes.sort((x, y) => x.t - y.t);
  for (let k = 0; k < notes.length; k++) {
    const nt = notes[k], nx = notes[k + 1];
    const gate = nt.dur * (nt.gate ?? o.gate ?? 0.9);
    const rel = o.rel ?? 0.05;
    const s0 = S(nt.t);
    const avail = nx ? S(nx.t) - s0 : S(gate + rel);
    const n = Math.min(S(gate + rel), avail);
    const legato = prev && Math.abs(prev.t + prev.dur - nt.t) < 1e-3;
    const slide = legato ? (nt.sl ?? o.slide ?? 0.012) : 0;
    const m0 = legato ? prev.m : nt.m;
    const cut = nt.cut ?? o.cut ?? 6000; lp.set(cut, 0.7);
    for (let i = 0; i < n; i++) {
      const tt = i / SR;
      let m = slide > 0 && tt < slide ? m0 + (nt.m - m0) * smooth(tt / slide) : nt.m;
      if (o.scoop) m -= o.scoop * Math.exp(-tt / 0.025);
      const vd = o.vibDelay ?? 0.16;
      if (tt > vd) m += (o.vib ?? 0.22) * Math.min(1, (tt - vd) / 0.2) * Math.sin(TAU * 5.6 * (tt - vd));
      const f = mtof(m);
      const duty = o.duty ?? (0.12 + 0.3 * (1 - Math.exp(-tt / 0.18)) + 0.05 * Math.sin(TAU * 0.4 * (nt.t + tt)));
      let x = o.tri ? oa.nesTri(f) : oa.pulse(f, duty);
      if (o.det) x += o.det * ob.pulse(f * 1.0047, 0.5);
      if (o.oct) x += o.oct * oc.pulse(f * 2, 0.25);
      const a = o.att ?? 0.004;
      let env = Math.min(1, tt / a) * (0.78 + 0.22 * Math.exp(-tt / 0.12));
      if (tt > gate) env *= Math.max(0, 1 - (tt - gate) / rel);
      const tail = n - i; if (tail < 96) env *= tail / 96;
      b.add(s0 + i, lp.process(x) * env * nt.v * (o.gain ?? 1), o.pan ?? 0);
    }
    prev = nt;
  }
}

// ---------------------------------------------------------------- compose
function barNotes(bar, key, tr = 0, v = 1, extra = {}) {
  return HOOK[key].map(([st, m, len, sl]) => ({ t: T(bar, st), dur: len * STEP, m: m + tr, v, sl, ...extra }));
}

function compose(c, bars, lead, harm) {
  for (const bar of bars) {
    const sec = section(bar);
    const t0 = T(bar);
    const ch0 = chordAt(bar, 0);
    const grp = (bar - 7) % 4; // position in the 4-bar phrase for the drop

    // ---- lead
    const key = LEAD[bar];
    if (key) {
      let ns;
      if (sec === 'intro') ns = barNotes(bar, key, 0, 0.55 + 0.1 * (bar - 3) / 3);
      else if (sec === 'outro') ns = barNotes(bar, key, 0, 0.6);
      else if (sec === 'climax') ns = barNotes(bar, key, 12, 0.95);
      else if (sec === 'heroes') ns = barNotes(bar, key, 0, 1, { gate: 0.55 });
      else ns = barNotes(bar, key, 0, 1);
      if (bar === 6) ns = ns.filter((n) => n.t < BREATH[0] - 0.01).map((n) => ({ ...n, dur: Math.min(n.dur, BREATH[0] - n.t - 0.02) }));
      for (const n of ns) (sec === 'intro' || sec === 'outro' ? lead.soft : sec === 'boss' ? lead.boss : lead.main).push(n);
      // harmony: thirds below from bar 15, octave doubling in the climax
      if ((bar >= 15 && bar <= 18) || bar === 39) for (const n of ns) harm.push({ ...n, m: diatonic(n.m, -2), v: 0.8 });
      if (sec === 'climax') for (const n of ns) harm.push({ ...n, m: n.m - 12, v: 0.9 });
      if (sec === 'heroes') for (const n of ns) harm.push({ ...n, m: n.m - 12, v: 0.6, gate: 0.4 });
      if (bar === 40) for (const n of ns) harm.push({ ...n, m: 76, v: 0.8 }, );
    }

    // ---- pad
    const padSplit = Array.isArray(PROG[bar]) ? [[0, 8], [8, 8]] : [[0, 16]];
    for (const [st, len] of padSplit) {
      const ch = chordAt(bar, st), notes = [...ch.t, ch.t[0] + 12];
      const tt = T(bar, st), d = len * STEP;
      if (sec === 'intro') {
        const cut0 = 350 * Math.pow(9, (bar - 1) / 6), cut1 = 350 * Math.pow(9, bar / 6);
        const dd = bar === 6 ? Math.min(d, BREATH[0] - tt - 0.05) : d;
        padChord(c, tt, dd, notes, 0.55 + 0.08 * bar, { att: bar === 1 ? 1.2 : 0.2, cut0, cut1, rel: bar === 6 ? 0.08 : 0.4 });
      } else if (sec === 'drop') padChord(c, tt, d, notes, grp === 0 && bar === 7 ? 0.8 : 0.65, { att: 0.02, cut0: 2600, cut1: 1800 });
      else if (sec === 'boss') padChord(c, tt, d, ch.t.map((m) => m - 12), 0.8, { att: 0.05, cut0: 900, cut1: 500 });
      else if (sec === 'climax') padChord(c, tt, d, notes, 0.75, { att: 0.02, cut0: 3800, cut1: 2600 });
      else if (sec === 'break') padChord(c, tt, d, [...notes, ch.t[1] + 12], 0.85, { att: 0.3, cut0: 1200 * Math.pow(2, bar - 33), cut1: 2400 * Math.pow(2, bar - 33), width: 1.3 });
      else if (sec === 'outro') padChord(c, tt, d, notes, 0.6, { att: 0.3, cut0: 1100, cut1: 900 });
      else if (sec === 'final' && bar === 39) padChord(c, tt, d, notes, 0.7, { att: 0.02, cut0: 3000 });
      else if (bar === 40) padChord(c, tt, 1.0, [...notes, 73], 0.9, { att: 0.01, cut0: 4000, cut1: 1200, rel: 0.7 });
      // heroes: no pad, stabs instead
    }

    // ---- arp
    const arpOn = sec !== 'boss' || true;
    if (arpOn && bar < 40) {
      const eighth = sec === 'intro' || sec === 'outro';
      const stepN = eighth ? 2 : 1;
      for (let st = 0; st < 16; st += stepN) {
        if (bar === 6 && st >= 12) break;
        const ch = chordAt(bar, st);
        const tones = [...ch.t, ch.t[0] + 12];
        const wide = (bar >= 17 && bar <= 20) || sec === 'climax' || sec === 'break';
        const pool = wide ? [...tones.map((m) => m + 12), ...tones.slice(1).map((m) => m + 24)] : tones.map((m) => m + 12);
        const seq = [...pool, ...pool.slice(1, -1).reverse()];
        const idx = (st / stepN) % seq.length;
        let m = seq[idx];
        if (sec === 'boss') m -= 12;
        let v = { intro: 0.5 + 0.06 * bar, drop: 0.6, heroes: 0.45, boss: 0.45, climax: 0.95, break: 0.95, outro: 0.5, final: 0.7 }[sec];
        if (bar >= 17 && bar <= 20) v = 0.85;
        const acc = st % 4 === 0 ? 1 : 0.7;
        const pan = ((st / stepN) % 2 ? 0.45 : -0.45);
        const cut = sec === 'intro' ? 700 * Math.pow(5, (bar - 1) / 6) : sec === 'break' ? 1500 + 5000 * ((bar - 33) + st / 16) / 2 : sec === 'outro' ? 1800 : 6000;
        arpNote(c, T(bar, st), STEP * stepN, m, v * acc, pan, { duty: wide ? 0.125 : 0.25, cut, dec: eighth ? 0.16 : wide ? 0.08 : 0.07, tri: sec === 'intro' && bar <= 2 });
      }
    }

    // ---- bass
    const bassPat = (st) => {
      const ch = chordAt(bar, st), r = ch.b;
      return r + ([4, 10, 14].includes(st) ? 12 : 0);
    };
    if (sec === 'intro' && bar >= 3) {
      for (let st = 0; st < (bar === 6 ? 12 : 16); st += 2) bassNote(c, T(bar, st), STEP * 1.2, chordAt(bar, st).b + 12, 0.35 + 0.08 * (bar - 3), { cut: 150 + 60 * (bar - 3), env: 250 + 150 * (bar - 3), sub: 0.3 });
    } else if (sec === 'drop' || sec === 'final' && bar === 39) {
      for (let st = 0; st < 16; st += 2) bassNote(c, T(bar, st), STEP * 1.7, bassPat(st), 0.9, { cut: 330, env: 1700 });
      if (grp === 3 || bar === 20) bassNote(c, T(bar, 15), STEP * 0.8, chordAt(bar, 15).b + 7, 0.7);
    } else if (sec === 'heroes') {
      for (let st = 0; st < 16; st += 2) bassNote(c, T(bar, st), STEP * 1.0, chordAt(bar, st).b + (st % 4 ? 12 : 0), 0.95, { cut: 400, env: 2200, fdec: 0.05 });
    } else if (sec === 'boss') {
      bassNote(c, t0, STEP * 9.5, ch0.b, 0.62, { cut: 250, env: 900, fdec: 0.3, drive: 2.2, sub: 1.0 });
      bassNote(c, T(bar, 10), STEP * 5.5, ch0.b, 0.5, { cut: 250, env: 1200, fdec: 0.15, drive: 2.2, sub: 1.0 });
    } else if (sec === 'climax') {
      for (let st = 0; st < 16; st++) {
        const ch = chordAt(bar, st);
        bassNote(c, T(bar, st), STEP * 0.85, ch.b + (st % 4 === 2 ? 12 : st % 4 === 3 ? 7 : 0), st % 4 === 0 ? 1 : 0.8, { cut: 380, env: 2200, fdec: 0.04 });
      }
    } else if (sec === 'outro') {
      for (let st = 0; st < 16; st += 4) bassNote(c, T(bar, st), STEP * 3, ch0.b + 12, 0.5, { cut: 200, env: 400, sub: 0.6 });
    } else if (bar === 40) {
      bassNote(c, t0, 1.2, 33, 1.0, { cut: 300, env: 1500, fdec: 0.2, drive: 1.8, sub: 1.0 });
    }

    // ---- heroes stabs: accent on every quarter
    if (sec === 'heroes') {
      for (let q = 0; q < 4; q++) {
        const ch = chordAt(bar, q * 4);
        stab(c, T(bar, q * 4), [...ch.t.map((m) => m + 12), ch.t[0] + 24], q % 2 ? 0.85 : 1.0, 0.1, { dec: 0.09 });
      }
    }
    if (sec === 'boss') stab(c, t0, [ch0.b + 12, ch0.b + 24, ...ch0.t.map((m) => m - 12)], 1.2, 0.8, { saw: true, cut: 500, env: 3500, fdec: 0.15, dec: 0.5 });
    if (bar === 40) stab(c, t0, [57, 61, 64, 69, 73], 1.1, 0.7, { saw: true, cut: 900, env: 5000, fdec: 0.2, dec: 0.45 });

    drums(c, bar, sec, grp);
  }
}

function drums(c, bar, sec, grp) {
  const t0 = T(bar);
  const fillSmall = () => { for (let st = 12; st < 16; st++) snare(c, T(bar, st), 0.35 + 0.15 * (st - 12), { pan: (st - 13.5) * 0.15 }); };
  const fillBig = () => {
    tom(c, T(bar, 8), 0.8, 190, -0.3); tom(c, T(bar, 9), 0.75, 190, -0.3);
    tom(c, T(bar, 10), 0.8, 140, 0); tom(c, T(bar, 11), 0.75, 140, 0);
    for (let k = 0; k < 8; k++) snare(c, T(bar, 12 + k / 2), 0.3 + 0.08 * k, { len: 0.12 });
  };
  if (sec === 'intro') {
    if (bar >= 3) for (let st = 2; st < 16; st += 4) if (!(bar === 6 && st >= 12)) hat(c, T(bar, st), 0.25 + 0.06 * (bar - 3), false);
    if (bar >= 4) for (let st = 0; st < 16; st += 2) if (st % 4 && !(bar === 6 && st >= 12)) hat(c, T(bar, st + 1), 0.12, false, -0.2);
    if (bar === 5) for (let q = 0; q < 4; q++) kick(c, T(bar, q * 4), 0.45, 0.4);
    if (bar === 6) {
      for (let q = 0; q < 3; q++) kick(c, T(bar, q * 4), 0.55, 0.5);
      // snare roll: 8ths, 16ths, 32nds, then the breath
      const hits = [0, 2, 4, 5, 6, 7, 8, 8.5, 9, 9.5, 10, 10.5, 11, 11.5];
      hits.forEach((st, k) => snare(c, T(bar, st), 0.2 + 0.5 * (k / hits.length) ** 1.5, { len: 0.14 }));
      riser(c, T(5), BREATH[0], 0.9);
    }
    return;
  }
  if (sec === 'drop' || sec === 'climax' || (sec === 'final' && bar === 39)) {
    const climax = sec === 'climax';
    for (let q = 0; q < 4; q++) kick(c, T(bar, q * 4), 1, 1);
    snare(c, T(bar, 4), 0.95, { clap: true }); snare(c, T(bar, 12), 0.95, { clap: true });
    if (climax || grp >= 0 && bar >= 11) snare(c, T(bar, 14), 0.18, { len: 0.1 });
    for (let st = 0; st < 16; st++) {
      const off = st % 4 === 2;
      if (climax) hat(c, T(bar, st), off ? 0.9 : st % 2 ? 0.55 : 0.4, false, st % 2 ? 0.25 : -0.1);
      else if (off) hat(c, T(bar, st), 0.8, bar >= 11 && st === 14 && bar % 2 === 0);
      else if (st % 2) hat(c, T(bar, st), 0.3, false, -0.2);
    }
    if (climax) { hat(c, T(bar, 6), 0.5, true); hat(c, T(bar, 14), 0.55, true); }
    if (bar === 7) { crash(c, t0, 1.1); impact(c, t0, 1.0); }
    else if (sec === 'drop' && grp === 0) crash(c, t0, 0.85);
    if (bar === 29) { crash(c, t0, 1.1); impact(c, t0, 0.8); }
    if (bar === 31) crash(c, t0, 0.8);
    if (bar === 39) crash(c, t0, 0.8);
    if (bar === 32) {
      for (let k = 0; k < 16; k++) snare(c, T(bar, k), 0.25 + 0.5 * (k / 16) ** 1.3, { len: 0.1 });
      for (let k = 0; k < 8; k++) snare(c, T(bar, 12 + k / 2) + STEP / 4, 0.4 + 0.05 * k, { len: 0.08 });
      riser(c, T(31, 8), T(33) - 0.01, 1.0);
    } else if (bar === 20 || bar === 39) fillBig();
    else if (grp === 3) fillSmall();
    if (bar === 20) crash(c, T(21), 0.7, true);
    if (bar === 28) {} // handled in boss
    return;
  }
  if (sec === 'heroes') {
    for (let q = 0; q < 4; q++) kick(c, T(bar, q * 4), 1, 0.8);
    snare(c, T(bar, 4), 0.9, { clap: true }); snare(c, T(bar, 12), 0.9, { clap: true });
    for (let st = 0; st < 16; st++) if (st % 2) hat(c, T(bar, st), st % 4 === 3 ? 0.35 : 0.2, false, 0.2); else if (st % 4 === 2) hat(c, T(bar, st), 0.75, true);
    if (bar === 21) crash(c, t0, 0.9);
    if (bar === 24) { fillBig(); crash(c, T(25), 0.6, true); }
    return;
  }
  if (sec === 'boss') {
    kick(c, t0, 1.1, 1.2); kick(c, T(bar, 10), 0.9, 0.9); if (bar % 2 === 0) kick(c, T(bar, 14), 0.6, 0.5);
    impact(c, t0, 1.0); crash(c, t0, 0.9);
    snare(c, T(bar, 8), 1.1, { clap: true, decay: 0.2, len: 0.45 });
    for (let st = 2; st < 16; st += 4) hat(c, T(bar, st), 0.45, st === 6);
    if (bar === 28) { for (let k = 0; k < 4; k++) tom(c, T(bar, 12 + k), 0.9, 170 - k * 25, 0.3 - k * 0.2); crash(c, T(29), 0.6, true); }
    return;
  }
  if (sec === 'break') {
    if (bar === 34) crash(c, T(35), 0.35, true);
    return;
  }
  if (sec === 'outro') {
    for (let st = 2; st < 16; st += 4) hat(c, T(bar, st), 0.22, false);
    return;
  }
  if (bar === 40) {
    kick(c, t0, 1.2, 1); crash(c, t0, 1.2); impact(c, t0, 1.2);
    snare(c, t0, 0.8, { clap: true, decay: 0.25, len: 0.5 });
  }
}

function renderLeads(c, lead, harm) {
  mono(c, 'lead', lead.main, { det: 0.45, oct: 0.18, gain: 0.55, cut: 7000, slide: 0.014 });
  mono(c, 'lead', lead.soft, { duty: 0.5, det: 0.2, gain: 0.45, cut: 1500, att: 0.012, slide: 0.02, rel: 0.12 });
  mono(c, 'lead', lead.boss, { det: 0.8, gain: 0.55, cut: 2600, scoop: 1.5, vib: 0.3, slide: 0.05 });
  mono(c, 'harm', harm, { duty: 0.125, gain: 0.5, cut: 5000, pan: 0.2, vibDelay: 0.25 });
}

// Sum buses with sidechain + sends, run delay and reverb.
function mixdown(c) {
  const env = new Float32Array(N).fill(1);
  const ks = c.kicks.slice().sort((a, b) => a[0] - b[0]);
  for (let k = 0; k < ks.length; k++) {
    const [t, s] = ks[k], s0 = S(t), s1 = Math.min(N, k + 1 < ks.length ? S(ks[k + 1][0]) : s0 + S(0.3));
    for (let i = s0; i < s1; i++) {
      const dt = (i - s0) / SR;
      const d = dt < 0.004 ? dt / 0.004 : 1 - smooth((dt - 0.004) / 0.24);
      env[i] = Math.min(env[i], 1 - Math.min(1, s) * d);
    }
  }
  const L = new Float32Array(N), R = new Float32Array(N);
  const rvL = new Float32Array(N), rvR = new Float32Array(N), dlL = new Float32Array(N), dlR = new Float32Array(N);
  for (const k in BUSES) {
    const cfg = BUSES[k], b = c.bus[k];
    if (cfg.hp) {
      const fl = Biquad.highpass(cfg.hp), fr = Biquad.highpass(cfg.hp);
      for (let i = 0; i < N; i++) { b.L[i] = fl.process(b.L[i]); b.R[i] = fr.process(b.R[i]); }
    }
    for (let i = 0; i < N; i++) {
      const g = cfg.gain * (1 - cfg.sc * (1 - env[i]));
      const l = b.L[i] * g, r = b.R[i] * g;
      L[i] += l; R[i] += r;
      if (cfg.rev) { rvL[i] += l * cfg.rev; rvR[i] += r * cfg.rev; }
      if (cfg.dly) { dlL[i] += l * cfg.dly; dlR[i] += r * cfg.dly; }
    }
  }
  if (process.env.MUSIC_DEBUG) {
    const rows = [];
    for (const k in BUSES) {
      const cfg = BUSES[k], b = c.bus[k];
      const r = [[7, 11], [21, 25], [25, 29], [29, 33], [33, 35]].map(([x, y]) => {
        let e = 0; for (let i = S(T(x)); i < S(T(y)); i++) { const g = cfg.gain * (1 - cfg.sc * (1 - env[i])); e += (b.L[i] * g) ** 2 + (b.R[i] * g) ** 2; }
        return (10 * Math.log10(e / (2 * (S(T(y)) - S(T(x)))) + 1e-12)).toFixed(1).padStart(6);
      });
      rows.push(k.padEnd(6) + r.join(''));
    }
    console.log('bus    drop  heroes boss  climax break\n' + rows.join('\n'));
  }
  const [dL, dR] = pingpong(dlL, dlR, BEAT * 0.75, 0.38, 4000);
  for (let i = 0; i < N; i++) { L[i] += dL[i] * 0.8; R[i] += dR[i] * 0.8; rvL[i] += dL[i] * 0.2; rvR[i] += dR[i] * 0.2; }
  const hl = Biquad.highpass(220), hr = Biquad.highpass(220);
  for (let i = 0; i < N; i++) { rvL[i] = hl.process(rvL[i]); rvR[i] = hr.process(rvR[i]); }
  const [wL, wR] = reverb(rvL, rvR, { room: 0.86, damp: 0.3, predelay: 0.025 });
  for (let i = 0; i < N; i++) { L[i] += wL[i] * 0.9; R[i] += wR[i] * 0.9; }
  return [L, R];
}

function renderSegment(bars) {
  const c = new Ctx();
  const lead = { main: [], soft: [], boss: [] }, harm = [];
  compose(c, bars, lead, harm);
  renderLeads(c, lead, harm);
  return mixdown(c);
}

// Tape-stop: speed ramps 1 -> 0 over `len`, reading from the source at t0.
function tapeStop(L, R, t0, len = 0.42) {
  const s0 = S(t0), n = S(len), srcL = L.slice(s0, s0 + n), srcR = R.slice(s0, s0 + n);
  const lpL = new SVF(12000, 0.7), lpR = new SVF(12000, 0.7);
  for (let i = 0; i < n; i++) {
    const tau = i / SR, u = tau / len;
    const pos = (tau - tau * tau / (2 * len)) * SR * 1.0; // integral of the rate
    const j = Math.floor(pos), fr = pos - j;
    const a = srcL[j] ?? 0, b = srcL[j + 1] ?? 0, a2 = srcR[j] ?? 0, b2 = srcR[j + 1] ?? 0;
    if (i % 16 === 0) { const fc = 14000 * Math.pow(1 - u, 2.2) + 120; lpL.set(fc, 0.7); lpR.set(fc, 0.7); }
    const amp = 1 - smooth((u - 0.55) / 0.45);
    L[s0 + i] = lpL.process(a + (b - a) * fr) * amp;
    R[s0 + i] = lpR.process(a2 + (b2 - a2) * fr) * amp;
  }
  for (let i = s0 + n; i < N; i++) { L[i] = 0; R[i] = 0; }
}

function main() {
  const range = (a, b) => Array.from({ length: b - a + 1 }, (_, i) => a + i);
  console.log('music: rendering bars 1-38');
  const [aL, aR] = renderSegment(range(1, 38));
  // breath: last beat of bar 6 near-silent
  const [b0, b1] = BREATH;
  for (let i = S(b0 - 0.03); i < S(b1); i++) {
    const t = i / SR;
    const g = t < b0 ? 1 - smooth((t - (b0 - 0.03)) / 0.03) * 0.97 : t > b1 - 0.02 ? 0.03 + 0.97 * smooth((t - (b1 - 0.02)) / 0.02) : 0.03;
    aL[i] *= g; aR[i] *= g;
  }
  tapeStop(aL, aR, TAPE);

  console.log('music: rendering bars 39-40');
  const [fL, fR] = renderSegment([39, 40]);
  // swell into the final statement
  const cx = new Ctx(); crash(cx, FINAL, 0.8, true); riser(cx, FINAL - BAR * 0.5, FINAL - 0.01, 0.35);
  // soft pad under the pause
  padChord(cx, TAPE + 0.35, FINAL - TAPE - 0.35, [57, 60, 64, 71], 0.14, { att: 0.9, cut0: 700, cut1: 1100, rel: 0.25 });
  const [pL, pR] = mixdown(cx);

  const L = new Float32Array(N), R = new Float32Array(N);
  const hpL = Biquad.highpass(28), hpR = Biquad.highpass(28);
  for (let i = 0; i < N; i++) {
    const t = i / SR;
    const fade = t > DUR - 0.9 ? Math.pow(Math.max(0, (DUR - 0.02 - t) / 0.88), 2) : 1;
    L[i] = hpL.process(aL[i] + fL[i] + pL[i]) * fade;
    R[i] = hpR.process(aR[i] + fR[i] + pR[i]) * fade;
  }
  const [oL, oR] = master(L, R, -14.2, -1);
  writeWav(OUT, oL, oR);
  console.log('music: wrote', OUT);
}

main();
