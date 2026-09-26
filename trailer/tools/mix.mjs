#!/usr/bin/env node
// Final trailer mix: music.wav + synthesized SFX at every cue -> trailer.wav.
// Big hits (slam/impact/boom) duck the music briefly so they punch through.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { SR, readWav, writeWav, master, lufs, smooth } from './lib/dsp.mjs';
import { renderSfx, LEVEL } from './sfx.mjs';
import { collectCues } from './cues.mjs';

const AUDIO = path.join(path.dirname(fileURLToPath(import.meta.url)), '../assets/audio');
const DUR = 75, N = DUR * SR;
const SFX_BUS = 0.8;
const DUCK = { slam: 0.55, impact: 0.6, boom: 0.8, zap: 0.85 }; // music gain under the hit

const music = readWav(path.join(AUDIO, 'music.wav'));
if (music.sr !== SR) throw new Error('music.wav must be 48 kHz');
const cuesPath = path.join(AUDIO, 'cues.json');
const cues = fs.existsSync(cuesPath) ? JSON.parse(fs.readFileSync(cuesPath, 'utf8')) : collectCues().cues;

const sL = new Float32Array(N), sR = new Float32Array(N), duck = new Float32Array(N).fill(1);
const count = {}, skipped = new Set();
for (const [t, name, gain = 1] of cues) {
  const idx = count[name] = (count[name] ?? -1) + 1;
  const buf = renderSfx(name, idx);
  if (!buf) { skipped.add(name); continue; }
  const g = (LEVEL[name] ?? 0.5) * gain * SFX_BUS, s0 = Math.round(t * SR);
  for (let i = 0; i < buf[0].length && s0 + i < N; i++) { sL[s0 + i] += buf[0][i] * g; sR[s0 + i] += buf[1][i] * g; }
  if (DUCK[name]) {
    const depth = 1 - (1 - DUCK[name]) * Math.min(1, gain);
    for (let i = Math.max(0, s0 - 240); i < Math.min(N, s0 + SR * 0.45); i++) {
      const dt = (i - s0) / SR;
      const d = dt < 0 ? 1 - smooth((dt + 0.005) / 0.005) : smooth((dt - 0.08) / 0.35);
      duck[i] = Math.min(duck[i], depth + (1 - depth) * d);
    }
  }
}
if (skipped.size) console.warn('mix: unknown cue names skipped:', [...skipped].join(', '));

const L = new Float32Array(N), R = new Float32Array(N);
for (let i = 0; i < N; i++) {
  const m = i < music.L.length ? 1 : 0;
  L[i] = (m ? music.L[i] : 0) * duck[i] + sL[i];
  R[i] = (m ? music.R[i] : 0) * duck[i] + sR[i];
}
const [oL, oR] = master(L, R, -14, -1);
for (let i = N - 480; i < N; i++) { const g = (N - 1 - i) / 480; oL[i] *= g; oR[i] *= g; }
writeWav(path.join(AUDIO, 'trailer.wav'), oL, oR);
console.log(`mix: ${cues.length} cues, ${lufs(oL, oR).toFixed(1)} LUFS -> assets/audio/trailer.wav`);
