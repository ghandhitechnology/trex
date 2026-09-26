#!/usr/bin/env node
// Collect SFX cues from every scene listed in index.html into
// assets/audio/cues.json as sorted [[t, name, gain], ...].
// Scenes run in a Node vm with a stub PX; missing or broken scenes are skipped.
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import { fileURLToPath } from 'node:url';

const ROOT = path.join(path.dirname(fileURLToPath(import.meta.url)), '..');
const OUT = process.argv[2] || path.join(ROOT, 'assets/audio/cues.json');
const BEAT = 60 / 128, BAR = BEAT * 4;

// A value that absorbs any use: callable, indexable, numeric 0, iterable (empty).
function inert() {
  const f = function () {};
  return new Proxy(f, {
    get(_, k) {
      if (k === Symbol.toPrimitive) return () => 0;
      if (k === Symbol.iterator) return function* () {};
      if (k === 'then') return undefined;
      if (k === 'length') return 0;
      return inert();
    },
    set() { return true; },
    apply() { return inert(); },
    construct() { return inert(); },
    has() { return true; },
  });
}

// Pure helpers copied from engine/px.js so load-time math matches the browser.
const hash = (i, s = 0) => { let h = Math.imul((i | 0) ^ 0x9e3779b9, 0x85ebca6b) ^ Math.imul(s | 0, 0xc2b2ae35); h ^= h >>> 15; h = Math.imul(h, 0x27d4eb2f); h ^= h >>> 13; return (h >>> 0) / 4294967296; };
function rng(seed) {
  let a = seed >>> 0;
  return function () {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}
const ease = {
  out: (u) => 1 - Math.pow(1 - u, 3),
  in: (u) => u * u * u,
  inOut: (u) => (u < 0.5 ? 4 * u * u * u : 1 - Math.pow(-2 * u + 2, 3) / 2),
  back: (u) => { const c = 1.9; return 1 + (c + 1) * Math.pow(u - 1, 3) + c * Math.pow(u - 1, 2); },
  elastic: (u) => (u <= 0 ? 0 : u >= 1 ? 1 : Math.pow(2, -10 * u) * Math.sin((u * 10 - 0.75) * (2 * Math.PI / 3)) + 1),
};

function makePX(scenes) {
  const P = new Proxy({}, { get: (_, k) => (typeof k === 'string' ? '#000000' : undefined) });
  const base = {
    W: 320, H: 180, SCALE: 6, BPM: 128, BEAT, BAR, DURATION: 75, P, BANDS: new Proxy({}, { get: () => [] }),
    scenes,
    scene(s) { scenes.push(s); return s; },
    bar: (n, b = 0) => (n - 1) * BAR + b * BEAT,
    load() {},
    clamp: (v, a = 0, b = 1) => Math.max(a, Math.min(b, v)),
    lerp: (a, b, u) => a + (b - a) * u,
    ease, hash, rng,
    step: (t, fps = 12) => Math.floor(t * fps + 1e-6) / fps,
    pulse: (t, every = BEAT, decay = 6) => { const u = ((t % every) + every) % every; return Math.exp(-u * decay); },
  };
  return new Proxy(base, { get: (o, k) => (k in o ? o[k] : inert()), set: (o, k, v) => { o[k] = v; return true; } });
}

function sceneFiles() {
  const html = fs.readFileSync(path.join(ROOT, 'index.html'), 'utf8');
  return [...html.matchAll(/<script[^>]+src="(scenes\/[^"]+\.js)"/g)].map((m) => m[1]);
}

export function collectCues() {
  const scenes = [], PX = makePX(scenes);
  const doc = inert();
  const sandbox = { PX, console, Math, JSON, document: doc, Image: function () { return inert(); }, fetch: () => Promise.resolve(inert()) };
  sandbox.window = sandbox; sandbox.globalThis = sandbox; sandbox.self = sandbox;
  const ctx = vm.createContext(sandbox);
  for (const rel of sceneFiles()) {
    const file = path.join(ROOT, rel);
    if (!fs.existsSync(file)) { console.warn('cues: missing', rel); continue; }
    const before = scenes.length;
    try { vm.runInContext(fs.readFileSync(file, 'utf8'), ctx, { filename: rel, timeout: 5000 }); }
    catch (e) { console.warn(`cues: ${rel} failed at load (${e.message}); keeping ${scenes.length - before} scene(s) it registered`); }
  }
  const cues = [];
  for (const s of scenes) {
    let list;
    try { list = typeof s.cues === 'function' ? s.cues() : s.cues; } catch { list = null; }
    if (!Array.isArray(list)) continue;
    for (const c of list) {
      if (!Array.isArray(c)) continue;
      const t = Number(c[0]), name = String(c[1]), g = c[2] == null ? 1 : Number(c[2]);
      if (!Number.isFinite(t) || t < 0 || t >= 75 || !Number.isFinite(g)) continue;
      cues.push([Math.round(t * 1e5) / 1e5, name, g]);
    }
  }
  cues.sort((a, b) => a[0] - b[0] || (a[1] < b[1] ? -1 : 1));
  return { cues, scenes: scenes.length };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const { cues, scenes } = collectCues();
  fs.mkdirSync(path.dirname(OUT), { recursive: true });
  fs.writeFileSync(OUT, '[\n' + cues.map((c) => '  ' + JSON.stringify(c)).join(',\n') + (cues.length ? '\n' : '') + ']\n');
  const counts = {}; for (const [, n] of cues) counts[n] = (counts[n] || 0) + 1;
  console.log(`cues: ${cues.length} from ${scenes} scene(s) ->`, path.relative(process.cwd(), OUT), JSON.stringify(counts));
}
