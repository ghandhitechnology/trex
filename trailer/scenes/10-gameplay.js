// 10-gameplay: bars 9-20 (core loop, level-up and build, synergies), plus the
// shared editing kit (window.GPX) that 11-climax.js also uses.
// Shots are cut on the beat grid; transitions are composited through pixel
// masks; text lands on a beat with a hit. Every frame is a pure function of t.
(function () {
  const { W, H, P, BEAT } = PX;
  const B = (n, b = 0) => PX.bar(n, b);
  const clamp = PX.clamp;
  const hash = PX.hash;

  // ================================================================ layers
  const pool = {};
  function layer(name) {
    if (!pool[name]) {
      const c = document.createElement('canvas');
      c.width = W; c.height = H;
      const x = c.getContext('2d', { willReadFrequently: true });
      x.imageSmoothingEnabled = false;
      pool[name] = { c, x };
    }
    return pool[name];
  }
  // Draw fn() into the main buffer, copy it to a named layer, return the canvas.
  function grab(name, fn) {
    const g = PX.g;
    g.save(); PX.clear(P.ink); fn(); g.restore();
    const L = layer(name);
    L.x.clearRect(0, 0, W, H);
    L.x.drawImage(g.canvas, 0, 0);
    return L.c;
  }
  // Copy of the current buffer.
  function snap(name) { const L = layer(name); L.x.clearRect(0, 0, W, H); L.x.drawImage(PX.g.canvas, 0, 0); return L.c; }
  const put = (c, x = 0, y = 0) => PX.g.drawImage(c, Math.round(x), Math.round(y));

  // Draw `src` over the main buffer wherever test(x, y) is true.
  let maskData = null;
  function masked(src, test, ox = 0, oy = 0) {
    const M = layer('mask');
    if (!maskData) maskData = M.x.createImageData(W, H);
    const d = maskData.data;
    for (let y = 0; y < H; y++) for (let x = 0; x < W; x++) d[(y * W + x) * 4 + 3] = test(x, y) ? 255 : 0;
    M.x.globalCompositeOperation = 'source-over';
    M.x.putImageData(maskData, 0, 0);
    M.x.globalCompositeOperation = 'source-in';
    M.x.drawImage(src, Math.round(ox), Math.round(oy));
    M.x.globalCompositeOperation = 'source-over';
    PX.g.drawImage(M.c, 0, 0);
  }

  // Whole-frame shake: shift the finished buffer and smear its edge rows and columns
  // into the exposed strips, so no ink sliver ever shows (or fill them with `fill`).
  function shakeFrame(dx, dy, fill) {
    dx = Math.round(dx); dy = Math.round(dy);
    if (!dx && !dy) return;
    const c = snap('shk'), g = PX.g;
    if (fill) PX.clear(fill);
    g.drawImage(c, dx, dy);
    if (fill) return;
    const ex = dx > 0 ? [0, 0, dx] : [W - 1, W + dx, -dx], ey = dy > 0 ? [0, 0, dy] : [H - 1, H + dy, -dy]; // src, dst, size
    if (dx) g.drawImage(c, ex[0], 0, 1, H, ex[1], dy, ex[2], H);
    if (dy) g.drawImage(c, 0, ey[0], W, 1, dx, ey[1], W, ey[2]);
    if (dx && dy) g.drawImage(c, ex[0], ey[0], 1, 1, ex[1], ey[1], ex[2], ey[2]);
  }
  // Horizontal slice glitch for a few frames after a hard cut.
  function slices(age, seed = 1, amp = 14, life = 0.1) {
    if (age < 0 || age > life) return;
    const c = snap('slc');
    const f = Math.floor(age * 30);
    let y = 0, i = 0;
    while (y < H) {
      const h = 3 + Math.floor(hash(i, seed * 31 + f) * 20);
      if (hash(i, seed * 17 + f) < 0.45) {
        const dx = Math.round((hash(i, seed * 7 + f) - 0.5) * 2 * amp);
        PX.g.drawImage(c, 0, y, W, h, dx, y, W, h);
        if (hash(i, seed * 5 + f) < 0.3) PX.rect(0, y, W, 1, hash(i, seed + f) < 0.5 ? P.cyan : P.pink);
      }
      y += h; i++;
    }
  }

  // ---- color: remap the buffer through a palette ramp by luminance ----------
  const rgbOf = {};
  for (const k in P) { const h = P[k]; rgbOf[h] = [parseInt(h.slice(1, 3), 16), parseInt(h.slice(3, 5), 16), parseInt(h.slice(5, 7), 16)]; }
  const hexRgb = (h) => rgbOf[h] || [parseInt(h.slice(1, 3), 16), parseInt(h.slice(3, 5), 16), parseInt(h.slice(5, 7), 16)];
  // ramp: palette colors dark -> light. o: gain, bias, x, y, w, h, dither (bool).
  function grade(ramp, o = {}) {
    const x0 = o.x || 0, y0 = o.y || 0, w = o.w || W, h = o.h || H;
    const img = PX.g.getImageData(x0, y0, w, h), d = img.data;
    const cols = ramp.map(hexRgb), n = cols.length, gain = o.gain || 1.6, bias = o.bias || 0;
    for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
      const i = (y * w + x) * 4;
      const l = (d[i] * 0.3 + d[i + 1] * 0.55 + d[i + 2] * 0.15) / 255;
      let v = clamp(l * gain + bias) * (n - 1);
      if (o.dither !== false) v += PX.bayer(x + x0, y + y0) - 0.5;
      const c = cols[clamp(Math.round(v), 0, n - 1)];
      d[i] = c[0]; d[i + 1] = c[1]; d[i + 2] = c[2];
    }
    PX.g.putImageData(img, x0, y0);
  }
  const RAMP = {
    ink: [P.ink, P.night, P.dusk, P.slate],
    sun: [P.maroon, P.blood, P.red, P.ember, P.amber, P.gold, P.cream],
    ice: [P.navy, P.blue, P.sky, P.cyan, P.ice],
    dread: [P.ink, P.plum, P.grape, P.pink, P.blush],
    bone: [P.ink, P.dusk, P.mauve, P.haze, P.fog, P.bone],
  };

  // ================================================================ masks
  // Each returns test(x, y) -> true where the incoming shot shows (u in 0..1).
  const bay = (x, y) => PX.bayer(x, y) + 1 / 32;
  const MASK = {
    dither: (u, o = {}) => { const c = o.cell || 2; return (x, y) => bay((x / c) | 0, (y / c) | 0) < u; },
    // Dither front sweeping across the frame. dir: 'r' | 'l' | 'd' | 'u'.
    sweep: (u, o = {}) => {
      const c = o.cell || 3, dir = o.dir || 'r';
      return (x, y) => {
        const s = dir === 'r' ? x / W : dir === 'l' ? 1 - x / W : dir === 'd' ? y / H : 1 - y / H;
        return s * 0.62 + bay((x / c) | 0, (y / c) | 0) * 0.38 < u;
      };
    },
    blocks: (u, o = {}) => { const c = o.cell || 20, sd = o.seed || 5; return (x, y) => hash(((x / c) | 0) + ((y / c) | 0) * 97, sd) < u; },
    iris: (u, o = {}) => {
      const cx = o.cx == null ? W / 2 : o.cx, cy = o.cy == null ? H / 2 : o.cy, R = 190 * PX.ease.in(u) * 1.02, c = o.cell || 1;
      return (x, y) => { const dx = ((x / c) | 0) * c - cx, dy = ((y / c) | 0) * c - cy; return dx * dx + dy * dy < R * R; };
    },
    diamonds: (u, o = {}) => {
      const c = o.cell || 24;
      return (x, y) => {
        const gx = (x / c) | 0, gy = (y / c) | 0, lx = x - gx * c - c / 2, ly = y - gy * c - c / 2;
        const lag = (gx / (W / c)) * 0.5; const v = clamp((u - lag) / 0.5);
        return Math.abs(lx) + Math.abs(ly) < v * c;
      };
    },
    // Diagonal slash: a hard diagonal edge crosses the frame.
    slash: (u, o = {}) => { const k = o.slope || 0.6; return (x, y) => x + y * k < u * (W + H * k); },
    bars: (u, o = {}) => { const n = o.n || 9, hz = o.dir !== 'v'; return (x, y) => { const p = hz ? y : x, len = hz ? H : W, s = len / n, i = (p / s) | 0, f = (p - i * s) / s; const v = clamp(u * 1.5 - (i % 2) * 0.5); return f < v; }; },
  };
  const TR_DEF = {
    cut: [0, 0], flash: [0, 0], glitch: [0, 0],
    dither: [0.08, 0.1], sweep: [0.1, 0.1], blocks: [0.06, 0.1], iris: [0, 0.26], diamonds: [0.1, 0.14], bars: [0.06, 0.1],
    slash: [0.06, 0.08], whip: [0.08, 0.08],
  };

  // ================================================================ footage
  const has = (id) => !!PX.footageIndex[id];
  const fi = (id) => PX.footageIndex[id];
  const dur = (id) => (has(id) ? fi(id).frames / fi(id).fps : 0);
  // Placeholder when a clip is missing: a dark arena with Rex in it.
  function placeholder(id, lt, o) {
    const k = o.k || 1;
    PX.clear(P.night);
    const off = Math.floor(lt * 30) % 16;
    for (let x = -16; x < W + 16; x += 16) PX.rect(x + off, 0, 1, H, P.dusk);
    for (let y = -16; y < H + 16; y += 16) PX.rect(0, y + off, W, 1, P.dusk);
    PX.spr('rex', W / 2 - 8 * k + (o.dx || 0), H / 2 - 8 * k + (o.dy || 0), Math.floor(lt * 8), { k });
    PX.text(`${id} ${lt.toFixed(2)}`.toUpperCase(), 6, H - 10, P.fog);
  }

  // Player track per clip (source px, per footage frame), from scenes/12-track.js.
  // Smoothed over a window so tracked punch-ins glide instead of jitter.
  function trackAt(id, lt, win = 6) {
    const T = (window.GTRACK || {})[id];
    const f = fi(id);
    if (!T || !f) return null;
    const i = clamp(Math.floor(lt * f.fps + 1e-6), 0, T.length - 1);
    let sx = 0, sy = 0, n = 0;
    for (let j = Math.max(0, i - win); j <= Math.min(T.length - 1, i + win); j++) if (T[j]) { sx += T[j][0]; sy += T[j][1]; n++; }
    return n ? [sx / n, sy / n] : null;
  }
  // The arena floor of each clip (footage px, [left, top, right, bottom), measured):
  // outside it is the dark arena wall (and the SPACE box), which must never fill the side of a punch-in.
  const PLAY = {
    early: [0, 0, 304, 180], 'syn-fire-wheel': [0, 0, 304, 180], chaos: [0, 0, 306, 180],
    build: [24, 0, 320, 180], 'syn-cryo-beam': [24, 0, 320, 180], 'syn-ball-lightning': [24, 0, 320, 180],
  };
  // Where clip `id` lands on screen at source time lt: {x, y, k}, source (0, 0) at (x, y).
  // o: k, cx, cy (zoom focus), track (+ tx, ty), dx, dy, frame. The view is clamped to the
  // playfield when zoomed (k > 1; `frame` clamps to the whole footage frame instead) and
  // to the footage frame at 1x, so no ink ever shows.
  function view(id, lt, o = {}) {
    const f = fi(id), k = o.k || 1;
    let cx = o.cx, cy = o.cy;
    if (o.track) { const p = trackAt(id, lt, o.win); if (p) { cx = p[0] + (o.tx || 0); cy = p[1] + (o.ty || 0); } }
    if (cx == null) cx = f.w / 2;
    if (cy == null) cy = f.h / 2;
    let x = Math.round(W / 2 - cx * k + (o.dx || 0)), y = Math.round(H / 2 - cy * k + (o.dy || 0));
    if (o.inside !== false) {
      const r = (k > 1 && !o.frame && PLAY[id]) || [0, 0, f.w, f.h];
      const fit = (lo, hi, span, full) => ((hi - lo) * k >= span ? [span - hi * k, -lo * k] : [span - full * k, 0]);
      const [x0, x1] = fit(r[0], r[2], W, f.w), [y0, y1] = fit(r[1], r[3], H, f.h);
      x = clamp(x, x0, x1); y = clamp(y, y0, y1);
    }
    return { x, y, k };
  }
  const toScreen = (v, sx, sy) => [Math.round(v.x + sx * v.k), Math.round(v.y + sy * v.k)];
  // Draw clip `id` at source time lt; returns its view.
  function foot(id, lt, o = {}) {
    if (!has(id)) { placeholder(id, lt, o); return { x: 0, y: 0, k: 1 }; }
    const f = fi(id), v = view(id, lt, o);
    const im = PX.footageFrame(id, Math.max(0, lt));
    if (im && im.width) PX.g.drawImage(im, v.x, v.y, f.w * v.k, f.h * v.k);
    return v;
  }

  // ================================================================ reels
  // shots: [{at, draw(local, t), tr: {type, pre, post, ...opts}}], ascending `at`.
  function drawShot(s, t) { PX.g.save(); s.draw(t - s.at, t); PX.g.restore(); }
  function trWin(s) {
    const tr = s.tr || { type: 'cut' };
    const d = TR_DEF[tr.type] || [0, 0];
    return [tr.pre != null ? tr.pre : d[0], tr.post != null ? tr.post : d[1]];
  }
  function playReel(shots, t) {
    let i = 0;
    for (let j = 0; j < shots.length; j++) if (shots[j].at <= t + 1e-9) i = j;
    const nx = shots[i + 1];
    if (nx) { const [pre] = trWin(nx); if (pre > 0 && t >= nx.at - pre) return transition(shots[i], nx, t); }
    const cur = shots[i];
    if (i > 0) { const [, post] = trWin(cur); if (post > 0 && t < cur.at + post) return transition(shots[i - 1], cur, t); }
    drawShot(cur, t);
    afterCut(cur, t);
  }
  // Hard-cut accents: flash frames and slice glitches right after `at`.
  function afterCut(s, t) {
    const tr = s.tr || {};
    const age = t - s.at;
    if (tr.type === 'flash' || tr.flash) flash(age, tr.color);
    if (tr.type === 'glitch') slices(age, Math.round(s.at * 10));
  }
  function flash(age, color) {
    if (age < 0) return;
    const c = color || P.bone;
    if (age < 1 / 30) PX.clear(c);
    else if (age < 2 / 30) PX.dither(0.5, c);
    else if (age < 3 / 30) PX.dither(0.2, c);
  }
  function transition(a, b, t) {
    const tr = b.tr, [pre, post] = trWin(b);
    const u = clamp((t - (b.at - pre)) / Math.max(1e-6, pre + post));
    if (tr.type === 'whip') {
      const e = PX.ease.inOut(u);
      const A = grab('A', () => drawShot(a, t));
      const Bc = grab('B', () => drawShot(b, t));
      const vert = tr.dir === 'u' || tr.dir === 'd';
      const sgn = tr.dir === 'd' || tr.dir === 'l' ? -1 : 1; // 'r': new shot enters from the right
      const span = vert ? H : W;
      const off = Math.round(e * span) * sgn;
      PX.clear(P.ink);
      if (vert) { put(A, 0, -off); put(Bc, 0, span * sgn - off); }
      else { put(A, -off, 0); put(Bc, span * sgn - off, 0); }
      // Speed streaks along the motion, strongest mid-whip.
      const m = Math.sin(u * Math.PI), f = Math.floor(t * 30);
      for (let i = 0; i < 14; i++) {
        if (PX.hash(i, f + 70) > m) continue;
        const len = Math.round((40 + PX.hash(i, f + 71) * 120) * m), p = Math.floor(PX.hash(i, f + 72) * (vert ? W : H));
        const q = Math.floor(PX.hash(i, f + 73) * ((vert ? H : W) + len)) - len, c = i % 3 ? P.fog : P.bone;
        if (vert) PX.rect(p, q, 1, len, c); else PX.rect(q, p, len, 1, c);
      }
      return;
    }
    const Bc = grab('B', () => drawShot(b, t));
    drawShot(a, t);
    const mk = MASK[tr.type] || MASK.dither;
    masked(Bc, mk(u, tr));
    // A bright edge on hard-edged wipes.
    if (tr.type === 'slash') {
      const k = tr.slope || 0.6, e = u * (W + H * k);
      for (let y = 0; y < H; y++) { const x = Math.round(e - y * k); PX.rect(x - 2, y, 3, 1, P.bone); PX.rect(x + 1, y, 1, 1, P.gold); }
    }
    if (tr.flash && t >= b.at) flash(t - b.at, tr.color);
  }

  // ================================================================ text
  const BANDS = Object.assign({}, PX.BANDS, {
    volt: [P.cream, P.gold, P.gold, P.cyan, P.cyan, P.sky, P.sky, P.blue, P.navy],
    tar: [P.cream, P.sand, P.sand, P.amber, P.clay, P.clay, P.umber, P.umber, P.maroon],
    pink: [P.blush, P.blush, P.pink, P.pink, P.grape, P.grape, P.plum, P.plum, P.maroon],
    blast: [P.cream, P.cream, P.gold, P.amber, P.ember, P.red, P.blood, P.maroon, P.maroon],
    white: [P.bone, P.bone, P.bone, P.fog, P.fog, P.haze, P.haze, P.mauve, P.slate],
  });
  const EDGE = { sun: P.maroon, dread: P.ink, ice: P.navy, venom: P.deep, bone: P.slate, volt: P.navy, tar: P.umber, pink: P.plum, blast: P.maroon, white: P.slate };
  // Callout that lands at `hit`.
  // o: k, x, y, style ('slam'|'drop'|'stamp'), bands, out (exit time), exit ('fall'|'squash'),
  //    from (slam scale), sparks, beat (bob on later beats).
  function callout(str, t, hit, o = {}) {
    const k = o.k || 3, style = o.style || 'slam';
    const x = o.x == null ? W / 2 : o.x;
    const y = o.y == null ? 24 : o.y;
    const bandsName = o.bands || 'sun';
    const bands = Array.isArray(bandsName) ? bandsName : BANDS[bandsName];
    const edge = o.edge || EDGE[bandsName] || P.maroon;
    const tw = PX.titleWidth(str, k);
    const lead = style === 'slam' ? 0.1 : style === 'drop' ? 0.16 : 0;
    // Quantize to 30 fps from the hit, so the landing frame is exactly `hit`.
    const age = Math.floor((t - hit) * 30 + 1e-6) / 30 + lead;
    if (age < 0) return;
    let fx;
    const glyphX = (i) => { const a = PX.titleWidth(str.slice(0, i), k), b = PX.titleWidth(str.slice(0, i + 1), k); return (i ? a + k : 0) + (b - (i ? a + k : 0)) / 2 - tw / 2; };
    if (o.out != null && t >= o.out) {
      const a = Math.floor((t - o.out) * 30 + 1e-6) / 30;
      if ((o.exit || 'fall') === 'squash') {
        if (a > 0.1) return;
        const sy = a < 1 / 30 ? 1.25 : a < 2 / 30 ? 0.35 : 0.12;
        fx = () => ({ sx: a < 1 / 30 ? 0.9 : 1.15, sy, dy: -Math.round(9 * k * (1 - sy) / 2), flash: a >= 1 / 30 ? P.bone : null });
      } else {
        // Letters hop and fall away, staggered from the left.
        fx = (i) => {
          const ai = a - i * 0.025;
          if (ai <= 0) return {};
          const dy = Math.round(-90 * ai + 1400 * ai * ai);
          if (dy > H) return { vis: false };
          return { dy, sx: 0.9, sy: 1.12 };
        };
      }
    } else if (style === 'drop') {
      fx = PX.dropIn(age, o.stagger || 0.03, o.height || 60);
    } else if (style === 'stamp') {
      fx = () => (age < 1 / 30 ? { sx: 1.3, sy: 1.3, flash: '#ffffff' } : age < 2 / 30 ? { sx: 0.9, sy: 1.12 } : age < 3 / 30 ? { sx: 1.04, sy: 0.97 } : {});
    } else {
      const s = PX.slamScale(age, lead, o.from || 3.2);
      fx = (i) => {
        if (age >= lead && age < lead + 1 / 30) return { sx: 1.22, sy: 0.74, flash: '#ffffff' };
        if (age >= lead + 1 / 30 && age < lead + 2 / 30) return { sx: 0.94, sy: 1.1 };
        if (age < lead) { const gx = glyphX(i); return { dx: Math.round(gx * (s - 1)), sx: s, sy: s, dy: Math.round((s - 1) * 4.5 * k) }; }
        return {};
      };
    }
    // Later beats: a small wave of hops across the letters (secondary motion).
    if (o.beat !== false && !(o.out != null && t >= o.out) && t - hit > 0.2) {
      const base = fx;
      const since = t - hit;
      const bi = Math.floor(since / BEAT + 1e-6), ba = since - bi * BEAT;
      fx = (i) => {
        const r = base(i) || {};
        if (bi >= 1) { const ai = ba - i * 0.022; if (ai >= 0 && ai < 0.07) return Object.assign({}, r, { dy: (r.dy || 0) - (ai < 0.035 ? 2 : 1) }); }
        return r;
      };
    }
    const sh = t - hit;
    const shine = sh > 0.05 && sh < 0.5 ? Math.round(PX.lerp(x - tw / 2 + y - 12, x + tw / 2 + y + 9 * k + 12, (sh - 0.05) / 0.45)) : null;
    PX.title(str, x, y, { k, bands, edge, fx, shine, depth: o.depth == null ? k + 1 : o.depth });
    // Impact sparks off the word's baseline.
    if (o.sparks !== false && (o.out == null || t < o.out)) {
      const ah = t - hit;
      if (ah >= 0 && ah < 0.8) {
        const cols = o.sparkCols || [P.bone, bands[1], bands[3]];
        PX.burst(ah, x - tw / 2 - 2, y + 9 * k, { n: 12, seed: 3 + (hit * 10 | 0), speed: 90, life: 0.45, colors: cols, angle: Math.PI, spread: 1.4 });
        PX.burst(ah, x + tw / 2 + 2, y + 9 * k, { n: 12, seed: 9 + (hit * 10 | 0), speed: 90, life: 0.45, colors: cols, angle: 0, spread: 1.4 });
      }
    }
  }

  // Sum of whole-frame shakes: hits = [[time, amp, dur]].
  function shakeAt(t, hits) {
    let dx = 0, dy = 0;
    hits.forEach(([h, a, d], i) => { const [x, y] = PX.shake(t - h, a, d || 0.3, 11 + i); dx += x; dy += y; });
    return [dx, dy];
  }

  // ================================================================ sprites
  // Outlined sprite canvases (ink rim), cached per image, color and scale.
  const olCache = new Map();
  function outlined(im, k = 1, rim = P.ink, sx = 0, sy = 0, sw = 0, sh = 0) {
    if (!im || !im.width) return null;
    sw = sw || im.width; sh = sh || im.height;
    const key = im.src + '|' + k + '|' + rim + '|' + sx + ',' + sy;
    let c = olCache.get(key);
    if (c) return c;
    const w = sw * k, h = sh * k;
    const base = document.createElement('canvas'); base.width = w; base.height = h;
    const bg = base.getContext('2d'); bg.imageSmoothingEnabled = false;
    bg.drawImage(im, sx, sy, sw, sh, 0, 0, w, h);
    const sil = document.createElement('canvas'); sil.width = w; sil.height = h;
    const sg = sil.getContext('2d'); sg.imageSmoothingEnabled = false;
    sg.drawImage(base, 0, 0); sg.globalCompositeOperation = 'source-in'; sg.fillStyle = rim; sg.fillRect(0, 0, w, h);
    c = document.createElement('canvas'); c.width = w + 2; c.height = h + 2;
    const cg = c.getContext('2d'); cg.imageSmoothingEnabled = false;
    for (const [a, b] of [[0, 1], [2, 1], [1, 0], [1, 2]]) cg.drawImage(sil, a, b);
    cg.drawImage(base, 1, 1);
    olCache.set(key, c);
    return c;
  }
  // Sticker: the sprite with a bone rim and an ink rim outside it (cached). Reads over busy footage.
  const stCache = new Map();
  function sticker(im, k = 2, rim = P.bone, sx = 0, sy = 0, sw = 0, sh = 0) {
    if (!im || !im.width) return null;
    const key = im.src + '|' + k + '|' + rim + '|' + sx + ',' + sy;
    let c = stCache.get(key);
    if (c) return c;
    const inner = outlined(im, k, rim, sx, sy, sw, sh);
    const w = inner.width, h = inner.height;
    const sil = document.createElement('canvas'); sil.width = w; sil.height = h;
    const sg = sil.getContext('2d'); sg.drawImage(inner, 0, 0); sg.globalCompositeOperation = 'source-in'; sg.fillStyle = P.ink; sg.fillRect(0, 0, w, h);
    c = document.createElement('canvas'); c.width = w + 2; c.height = h + 3;
    const cg = c.getContext('2d'); cg.imageSmoothingEnabled = false;
    for (const [a, b] of [[0, 1], [2, 1], [1, 0], [1, 2], [0, 2], [2, 2], [1, 3]]) cg.drawImage(sil, a, b);
    cg.drawImage(inner, 1, 1);
    stCache.set(key, c);
    return c;
  }
  // Flat-colored silhouette canvas of a sprite (cached).
  const silCache = new Map();
  function flat(im, k, color) {
    const key = im.src + '|' + k + '|' + color;
    let c = silCache.get(key);
    if (c) return c;
    c = document.createElement('canvas'); c.width = im.width * k; c.height = im.height * k;
    const g = c.getContext('2d'); g.imageSmoothingEnabled = false;
    g.drawImage(im, 0, 0, c.width, c.height); g.globalCompositeOperation = 'source-in'; g.fillStyle = color; g.fillRect(0, 0, c.width, c.height);
    silCache.set(key, c);
    return c;
  }
  // Item sprite with a 1px ink rim at scale k, top-left at (x, y). o: flash (color), sx/sy squash about bottom-center.
  function item(name, x, y, k = 2, o = {}) {
    const im = PX.sprite(name, o.frame || 0);
    if (!im || !im.width) return;
    const c = o.sticker ? sticker(im, k) : outlined(im, k);
    const pad = o.sticker ? 2 : 1;
    const sx = o.sx || 1, sy = o.sy || 1;
    const w = Math.max(1, Math.round(c.width * sx)), h = Math.max(1, Math.round(c.height * sy));
    const X = Math.round(x - pad + (c.width - w) / 2), Y = Math.round(y - pad + c.height - h);
    PX.g.drawImage(c, X, Y, w, h);
    if (o.flash) PX.g.drawImage(flat(im, k, o.flash), X + pad, Y + pad, w - pad * 2, h - pad * 2 - (o.sticker ? 1 : 0));
  }
  // Synergy icon strips (16x16 frames) are not in the sprite manifest.
  const SYN_IDS = ['ball-lightning', 'cryo-beam', 'extinction', 'fire-wheel'];
  SYN_IDS.forEach((s) => PX.load('syn:' + s, `assets/sprites/syn/${s}-strip.png`));
  function synIcon(name, x, y, t, k = 2, o = {}) {
    const im = PX.images['syn:' + name];
    if (!im || !im.width) return;
    const n = Math.max(1, Math.round(im.width / 16));
    const f = Math.floor(PX.step(t, 6) * 6) % n;
    const c = o.sticker ? sticker(im, k, P.bone, f * 16, 0, 16, 16) : outlined(im, k, o.rim || P.ink, f * 16, 0, 16, 16);
    const sx = o.sx || 1, sy = o.sy || 1;
    const w = Math.max(1, Math.round(c.width * sx)), h = Math.max(1, Math.round(c.height * sy));
    PX.g.drawImage(c, Math.round(x - 1 + (c.width - w) / 2), Math.round(y - 1 + (c.height - h) / 2), w, h);
  }
  // Drop a thing in so it lands at `hit`: returns [dy, sx, sy], or null before it appears.
  function dropY(t, hit, h = 50) {
    const a = PX.step(t - hit + 0.12, 30);
    if (a < 0) return null;
    if (a < 0.12) { const u = a / 0.12; return [-h * (1 - u * u), 0.8, 1.25]; }
    if (a < 0.16) return [0, 1.3, 0.7];
    if (a < 0.22) return [-2, 0.9, 1.12];
    return [0, 1, 1];
  }
  // Item stickers blown out of (x, y) in every upward-ish direction, falling out of frame.
  // Stateless: each path is closed-form in age. o: seed, speed, gravity, up, stagger, k.
  function erupt(age, x, y, names, o = {}) {
    if (age < 0) return;
    const seed = o.seed || 7, grav = o.gravity || 700, k = o.k || 2, up = o.up == null ? 120 : o.up;
    names.forEach((nm, i) => {
      const a = age - hash(i, seed + 3) * (o.stagger || 0.05);
      if (a < 0) return;
      const ang = -Math.PI / 2 + (hash(i, seed) - 0.5) * (o.spread || 5);
      const sp = (o.speed || 340) * (0.35 + 0.65 * hash(i, seed + 1));
      const im = PX.sprite(nm, 0);
      if (!im || !im.width) return;
      const c = sticker(im, k);
      const px_ = x + Math.cos(ang) * sp * a, py_ = y + (Math.sin(ang) * sp - up) * a + 0.5 * grav * a * a;
      if (px_ < -c.width || px_ > W + c.width || py_ > H + c.height) return;
      // Stop-motion tumble: some flip on alternate 12 fps frames.
      const flip = hash(i, seed + 9) < 0.5 && Math.floor(a * 12) % 2 === 1;
      const X0 = Math.round(px_ - c.width / 2), Y0 = Math.round(py_ - c.height / 2);
      if (flip) { PX.g.save(); PX.g.translate(X0 + c.width, Y0); PX.g.scale(-1, 1); PX.g.drawImage(c, 0, 0); PX.g.restore(); }
      else PX.g.drawImage(c, X0, Y0);
    });
  }

  // ================================================================ fx
  // Anime speed lines converging on (cx, cy), leaving a clear ellipse (rx, ry). Flicker at 20 fps.
  function speedLines(t, cx, cy, o = {}) {
    const f = Math.floor(t * 20), n = o.n || 46, rx = o.rx || 70, ry = o.ry || 44;
    const cols = o.colors || [P.bone, P.fog];
    for (let i = 0; i < n; i++) {
      const a = (i + hash(i, f * 3 + 1) * 0.8) / n * Math.PI * 2;
      const r0 = 1 + hash(i, f * 7 + 2) * 0.55;
      const ca = Math.cos(a), sa = Math.sin(a);
      const x0 = cx + ca * rx * r0, y0 = cy + sa * ry * r0;
      const x1 = cx + ca * 400, y1 = cy + sa * 400;
      const c = cols[i % cols.length];
      PX.line(x0, y0, x1, y1, c);
      if (hash(i, f * 5 + 3) < 0.35) PX.line(x0 + (sa > 0 ? 1 : -1), y0, x1 + (sa > 0 ? 1 : -1), y1, c);
    }
  }
  // A foil glint: a diagonal white band sweeping across a screen rect.
  function glint(u, x0, y0, x1, y1) {
    if (u <= 0 || u >= 1) return;
    const s = PX.lerp(x0 + y0 * 0.5 - 16, x1 + y1 * 0.5 + 16, u);
    for (let y = Math.max(0, y0); y < Math.min(H, y1); y++) {
      const a = Math.round(s - y * 0.5);
      const seg = (l, r, c) => { l = Math.max(x0, l); r = Math.min(x1, r); if (r > l) PX.rect(l, y, r - l, 1, c); };
      seg(a - 12, a - 9, P.cream); seg(a - 9, a - 3, '#ffffff'); seg(a - 3, a, P.cream);
    }
  }

  window.GPX = {
    B, layer, grab, snap, put, masked, shakeFrame, slices, grade, RAMP, MASK, foot, view, toScreen, has, dur, fi, trackAt, placeholder,
    playReel, drawShot, flash, callout, shakeAt, BANDS, outlined, sticker, flat, item, synIcon, dropY, erupt, speedLines, glint,
  };
})();

// ---------------------------------------------------------------------------
// Synergy weather: themed accents over footage while a synergy is named. They
// flare on the downbeat and sink within two beats, so the footage carries the
// effect; only light particles stay. fx(t, a, live): a = seconds since the
// downbeat, live = 1 while the bar plays (0 in its last frames).
(function () {
  const { W, H, P, BEAT } = PX;
  const X = window.GPX;
  const hash = PX.hash, clamp = PX.clamp;
  const g = PX.g;
  // Edge strength: on in 0.08 s, holds a beat, gone by beat 2.
  const edgeEnv = (a) => (a < 0 ? 0 : a < 0.08 ? a / 0.08 : a < BEAT ? 1 : Math.max(0, 1 - (a - BEAT) / BEAT));

  // Per-pixel pass over the buffer with ImageData: fn(x, y) -> [r, g, b] or null.
  const rgb = (h) => [parseInt(h.slice(1, 3), 16), parseInt(h.slice(3, 5), 16), parseInt(h.slice(5, 7), 16)];
  function pass(x0, y0, w, h, fn) {
    x0 = Math.max(0, x0 | 0); y0 = Math.max(0, y0 | 0); w = Math.min(W - x0, w | 0); h = Math.min(H - y0, h | 0);
    if (w <= 0 || h <= 0) return;
    const img = g.getImageData(x0, y0, w, h), d = img.data;
    for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
      const c = fn(x + x0, y + y0);
      if (!c) continue;
      const i = (y * w + x) * 4;
      d[i] = c[0]; d[i + 1] = c[1]; d[i + 2] = c[2];
    }
    g.putImageData(img, x0, y0);
  }

  // Fire: a stop-motion flame bank flares up the bottom edge on the hit, embers rise all bar.
  const FIRE = [P.red, P.ember, P.amber, P.gold, P.cream].map(rgb);
  function fire(t, a, live) {
    const e = edgeEnv(a) * live;
    const ts = PX.step(t, 12);
    if (e > 0.02) pass(0, H - 30, W, 30, (x, y) => {
      const xx = x >> 1;
      const hgt = e * (8 + 5 * Math.sin(xx * 0.31 + ts * 11) + 4 * Math.sin(xx * 0.13 - ts * 7) + hash(xx, Math.floor(ts * 12)) * 5);
      const d = (H - y) - hgt * (0.4 + 0.6 * (0.5 + 0.5 * Math.sin(x * 0.02 + 1.3)));
      if (d > 0) return null;
      const v = clamp(-d / (hgt + 1)) * 4 + (PX.bayer(x, y) - 0.5) * 0.9;
      return FIRE[clamp(Math.floor(4 - v), 0, 4)];
    });
    for (let i = 0; i < 30; i++) {
      if (hash(i, 8) > live) continue;
      const sp = 40 + hash(i, 5) * 70;
      const y = H + 6 - ((a * sp + hash(i, 6) * 260) % 230);
      const x = hash(i, 7) * W + Math.sin(a * 3 + i) * 7;
      if (y < -4) continue;
      const f = Math.floor(t * 12 + i) % 3;
      const s = hash(i, 9) < 0.3 ? 2 : 1;
      PX.rect(x, y, s, s, [P.gold, P.amber, P.cream][f]);
    }
  }

  // Frost: ice bites in from the frame edges on the hit and melts back; snow drifts all bar.
  const FROST = [P.ice, P.ice, P.cyan, P.sky, P.blue].map(rgb);
  function frost(t, a, live) {
    const grow = edgeEnv(a) * live;
    if (grow > 0.01) {
      const band = 22;
      const fnc = (x, y) => {
        const d = Math.min(x, W - 1 - x, y * 1.4, (H - 1 - y) * 1.4);
        if (d > band) return null;
        const n = hash((x >> 2) + (y >> 2) * 131, 17) * 0.6 + hash((x >> 1) + (y >> 1) * 257, 19) * 0.4;
        const lim = grow * (5 + n * 17);
        if (d >= lim) return null;
        const v = d / Math.max(1, lim) * 4 + (PX.bayer(x, y) - 0.5) * 1.2;
        return FROST[clamp(Math.floor(v), 0, 4)];
      };
      pass(0, 0, W, 16, fnc); pass(0, H - 16, W, 16, fnc);
      pass(0, 16, band, H - 32, fnc); pass(W - band, 16, band, H - 32, fnc);
      // Crystal spikes from the edges.
      for (let i = 0; i < 16; i++) {
        const side = i % 4, p = hash(i, 23);
        const len = grow * (12 + hash(i, 29) * 22);
        if (len < 2) continue;
        let x0, y0, dx, dy;
        if (side === 0) { x0 = p * W; y0 = 0; dx = (hash(i, 31) - 0.5) * 0.8; dy = 1; }
        else if (side === 1) { x0 = p * W; y0 = H - 1; dx = (hash(i, 31) - 0.5) * 0.8; dy = -1; }
        else if (side === 2) { x0 = 0; y0 = p * H; dx = 1; dy = (hash(i, 31) - 0.5) * 0.8; }
        else { x0 = W - 1; y0 = p * H; dx = -1; dy = (hash(i, 31) - 0.5) * 0.8; }
        const x1 = x0 + dx * len, y1 = y0 + dy * len;
        PX.line(x0, y0, x1, y1, P.ice);
        PX.line(x0 + (dy ? 1 : 0), y0 + (dx ? 1 : 0), x1, y1, P.cyan);
        const bx = x0 + dx * len * 0.55, by = y0 + dy * len * 0.55;
        PX.line(bx, by, bx + (dx + dy) * len * 0.25, by + (dy - dx) * len * 0.25, P.ice);
        PX.line(bx, by, bx + (dx - dy) * len * 0.25, by + (dy + dx) * len * 0.25, P.ice);
      }
    }
    for (let i = 0; i < 36; i++) {
      if (hash(i, 41) > live) continue;
      const x = ((hash(i, 43) * W + a * (14 + hash(i, 44) * 20)) % W + W) % W;
      const y = ((hash(i, 45) * H + a * (20 + hash(i, 46) * 26)) % H + H) % H;
      PX.px(x, y, hash(i, 47) < 0.5 ? P.ice : P.bone);
    }
  }

  // Lightning: two bolts crack down on the hit (three frames), sparks crawl all bar.
  function bolt(seed, x0, y0, x1, y1, life, fork) {
    const n = 9;
    const pts = [[x0, y0]];
    for (let j = 1; j <= n; j++) {
      const u = j / n;
      const jx = j === n ? 0 : (hash(j, seed) - 0.5) * 26;
      pts.push([x0 + (x1 - x0) * u + jx, y0 + (y1 - y0) * u + (hash(j, seed + 1) - 0.5) * 6]);
    }
    const glow = life === 0 ? P.cyan : P.sky;
    for (let j = 0; j < n; j++) {
      const [a1, b1] = pts[j], [a2, b2] = pts[j + 1];
      if (life < 2) for (const d of [-2, 2]) PX.line(a1 + d, b1, a2 + d, b2, life === 0 ? P.sky : P.blue);
      if (life < 2) for (const d of [-1, 1]) PX.line(a1 + d, b1, a2 + d, b2, glow);
      PX.line(a1, b1, a2, b2, life === 0 ? P.bone : life === 1 ? P.ice : P.cyan);
    }
    if (fork && life < 2) {
      const [fx, fy] = pts[4];
      const ex = fx + (hash(3, seed + 5) - 0.5) * 90, ey = fy + 40;
      PX.line(fx, fy, (fx + ex) / 2 + 6, (fy + ey) / 2, P.ice);
      PX.line((fx + ex) / 2 + 6, (fy + ey) / 2, ex, ey, P.cyan);
    }
  }
  function volt(t, a, live) {
    const life = Math.floor(a * 30);
    if (a >= 0 && life <= 2 && live > 0) {
      bolt(7, 26, -4, 60, H + 4, life, true);
      bolt(19, W - 30, -4, W - 70, H + 4, life, true);
    }
    const f = Math.floor(t * 20);
    for (let i = 0; i < 18; i++) {
      if (hash(i, f) > 0.45 * live) continue;
      const x = hash(i, f + 3) * W, y = hash(i, f + 4) * H;
      PX.rect(x, y, 1, 3, P.cyan); PX.rect(x - 1, y + 1, 3, 1, P.ice);
    }
  }

  // Impact: the real meteors land on the 8ths (the shake carries the hits); ash drifts all bar.
  function impact(t, a, live) {
    for (let i = 0; i < 30; i++) {
      if (hash(i, 58) > live) continue;
      const sp = 30 + hash(i, 55) * 50;
      const y = -6 + ((a * sp + hash(i, 56) * 200) % 196);
      const x = hash(i, 57) * W + Math.sin(a * 2 + i) * 6;
      PX.px(x, y, i % 3 ? P.haze : P.amber);
    }
  }

  X.WEATHER = { fire, frost, volt, impact };
})();

// ---------------------------------------------------------------------------
// Source clocks and footage shots. A clock maps global time to a clip's source
// time, piecewise linear, so speed ramps, hit-stops and stutters stay
// continuous across cuts: segs = [[t0, src0, rate = 1], ...] ascending t0.
(function () {
  const X = window.GPX;
  function clock(segs) {
    return (t) => {
      let s = segs[0];
      for (const q of segs) if (q[0] <= t + 1e-9) s = q;
      return s[1] + (t - s[0]) * (s[2] == null ? 1 : s[2]);
    };
  }
  // Stutter segments: n 16th-note slices starting at t0, repeating each source slice `rep` times.
  function stutter(t0, src0, n = 4, rep = 2) {
    const q = PX.BEAT / 4, out = [];
    for (let i = 0; i < n; i++) out.push([t0 + i * q, src0 + Math.floor(i / rep) * q, 1]);
    return out;
  }
  // A shot of clip `id` driven by clock `clk`. o: k, track, tx, ty, cx, cy, drift [vx, vy], punch, post(local, t).
  function shot(at, id, clk, o = {}, tr) {
    const opts = (local) => {
      const dr = o.drift || [0, 0];
      let k = o.k || 1;
      if (o.punch && local < 1 / 30) k += 1;
      return Object.assign({}, o, { k, dx: Math.round((o.dx || 0) + dr[0] * local), dy: Math.round((o.dy || 0) + dr[1] * local) });
    };
    return {
      at, tr: tr || { type: 'cut' }, id, o, clk,
      draw(local, t) {
        X.foot(id, clk(t), opts(local));
        if (o.post) o.post(local, t);
      },
      view(t) { return X.view(id, clk(t), opts(t - at)); },
    };
  }
  // Where the player is on screen during shot `s` at time t (for overlays).
  function rexOnScreen(s, t) {
    const p = X.trackAt(s.id, s.clk(t), 0);
    if (!p) return [PX.W / 2, PX.H / 2];
    return X.toScreen(s.view(t), p[0], p[1]);
  }
  const current = (shots, t) => { let s = shots[0]; for (const q of shots) if (q.at <= t + 1e-9) s = q; return s; };
  Object.assign(X, { clock, stutter, shot, rexOnScreen, current });
})();

// ---------------------------------------------------------------------------
// Bars 9-20: the core loop, the level-up pick and the build, the synergies.
(function () {
  const { W, H, P, BEAT } = PX;
  const X = window.GPX;
  const B = X.B, clamp = PX.clamp;
  const Q = BEAT / 4;
  const shot = X.shot;
  const TK = (k, o = {}) => Object.assign({ k, track: true }, o); // punch-in that follows the player

  // Every item in content/items.ron (54).
  const ITEMS = ['scattergun', 'railgun', 'saw-ring', 'bone-rang', 'tesla-rod', 'seeker-pod', 'laser-eye', 'mine-layer', 'magma-core',
    'frost-halo', 'meteor-call', 'quill-burst', 'starfall', 'cold-snap', 'turtle-shell', 'adrenaline', 'big-roar', 'bone-storm',
    'med-kit', 'gravity-egg', 'mine-burst', 'war-drum', 'hot-lead', 'twin-barrel', 'rubber-ball', 'drill-bit', 'powder-keg',
    'spark-plug', 'black-coffee', 'magnet', 'heart-jar', 'static-coil', 'iron-jaw', 'sharp-tooth', 'raptor-legs', 'big-bones',
    'amber-resin', 'feather', 'moss-pouch', 'tail-club', 'frost-fang', 'blood-drop', 'target-lock', 'long-neck', 'boulder-shot',
    'lucky-claw', 'thorn-hide', 'shrapnel', 'spore-pod', 'glass-cannon', 'storm-cell', 'fossil', 'golden-egg', 'hot-foot'];

  // ------------------------------------------------------------ bars 9-12
  // early (0:33): Rex auto-fires by the arena's right wall. Frame 113: the dash pops
  // (white disc); 114-116: he slides out of it with his cyan afterimage; 117-118: he
  // blinks (i-frames); 120: he lands in his own burst; 130: the LV4 nova.
  const DASH = B(11);
  const EC = X.clock([
    [B(9), 113 / 30 - (DASH - B(9))], // real time into the dash
    [DASH, 113 / 30, 0.5], // the pop and the slide at half speed...
    [DASH + 0.2, 116 / 30 + 0.002, 0], // ...held on Rex and his full trail for the beat
    [B(11, 1), 120 / 30, 0.71], // lands in his burst; the nova hits beat 2
  ]);
  // Middle of Rex plus his afterimage in the dash frames (footage px, measured).
  const DASH_AT = { 113: [241, 85], 114: [244, 91], 115: [245, 93], 116: [247, 92] };
  const DASH_V = { k: 2, cx: 224, cy: 84, drift: [0, -8] };

  // levelup (0:50): the card screen flashes in on frame 32, holds 38-73, the pick bursts on 74.
  const PICK = B(13);
  const LC = X.clock([
    [B(12), 32 / 30], // the cards flash in on the downbeat
    [B(12) + 0.2, 38 / 30, (73 - 38) / 30 / (PICK - B(12) - 0.2 - 0.01)], // the static hold, sped up
    [PICK, 74 / 30, 0.35], // the pick burst in slow motion
  ]);
  const CARD = [43, 43, 117, 127], COMBO_TAG = [102, 49]; // card 1 (Amber Resin, COMBO) in footage px
  // Centre of the game's pick burst per frame (footage px, measured); it drifts as the camera follows Rex.
  const BURST_AT = [[74, 164, 95], [75, 162, 92], [76, 160, 90], [77, 159, 89], [78, 157, 87], [79, 156, 86], [80, 155, 85], [84, 152, 82]];
  const burstAt = (src) => {
    const f = src * 30;
    let i = 0;
    while (i < BURST_AT.length - 2 && BURST_AT[i + 1][0] <= f) i++;
    const [fa, xa, ya] = BURST_AT[i], [fb, xb, yb] = BURST_AT[i + 1], u = clamp((f - fa) / (fb - fa));
    return [PX.lerp(xa, xb, u), PX.lerp(ya, yb, u)];
  };

  const loop = [
    shot(B(9), 'early', EC, TK(2, { ty: -4 }), { type: 'flash' }),
    shot(B(9, 2), 'early', EC, {}),
    shot(B(10), 'early', EC, TK(2, { ty: -10 }), { type: 'flash', color: P.gold }),
    shot(B(10, 2), 'early', EC, { punch: true }), // hard punch-out to wide
    // Bar 10's last beat is a snare fill: a 16th-note punch stutter.
    shot(B(10, 3), 'early', EC, TK(2)),
    shot(B(10, 3.25), 'early', EC, {}),
    shot(B(10, 3.5), 'early', EC, TK(2)),
    shot(B(10, 3.75), 'early', EC, TK(3)),
    // The bar 11 crash: the dash, held on Rex and his trail.
    shot(DASH, 'early', EC, DASH_V, { type: 'flash', color: P.ice }),
    shot(B(11, 1), 'early', EC, TK(3)),
    shot(B(11, 2), 'early', EC, {}, { type: 'slash' }),
    shot(B(11, 3), 'early', EC, TK(2)),
    // Bar 12: level up. Step in on the COMBO card, one zoom per beat.
    shot(B(12), 'levelup', LC, {}, { type: 'flash', color: P.cream }),
    shot(B(12, 1), 'levelup', LC, { k: 2, cx: 80, cy: 86 }),
    shot(B(12, 2), 'levelup', LC, { k: 3, cx: 82, cy: 72 }),
    shot(B(12, 3), 'levelup', LC, { k: 4, cx: 88, cy: 60 }),
  ];

  // ------------------------------------------------------------ bars 13-16
  // The pick completes Tar Pit; then the stacked build (4:40), stretched to fit:
  // stutters on the fills, a hit-stop and 0.42x under 54 ITEMS.
  const S14 = B(14, 3) - B(13, 2);
  const S15 = S14 + 2 * Q;
  const S16 = S15 + (PX.BAR - 0.1) * 0.42;
  const BC = X.clock([
    [B(13, 2), 0],
    ...X.stutter(B(14, 3), S14),
    [B(15), S15, 0], // hit-stop on the crash
    [B(15) + 0.1, S15, 0.42],
    [B(16), S16],
    ...X.stutter(B(16, 3), S16 + BEAT * 3),
  ]);
  const build = [
    shot(PICK, 'levelup', LC, { k: 2, cx: 160, cy: 100 }), // the sun-graded hit frames are the flash
    shot(B(13, 1), 'levelup', LC, { k: 3, cx: 160, cy: 92 }),
    shot(B(13, 2), 'build', BC, {}, { type: 'whip', dir: 'u' }),
    shot(B(14), 'build', BC, TK(2, { punch: true }), { type: 'flash', color: P.gold }),
    shot(B(14, 2), 'build', BC, {}),
    shot(B(14, 3), 'build', BC, TK(3)),
    shot(B(14, 3.25), 'build', BC, TK(2)),
    shot(B(14, 3.5), 'build', BC, TK(3)),
    shot(B(14, 3.75), 'build', BC, {}),
    shot(B(15), 'build', BC, {}),
    shot(B(16), 'build', BC, TK(2), { type: 'sweep', dir: 'l' }),
    shot(B(16, 2), 'build', BC, {}),
    shot(B(16, 3), 'build', BC, TK(3)),
    shot(B(16, 3.25), 'build', BC, TK(2)),
    shot(B(16, 3.5), 'build', BC, TK(3)),
    shot(B(16, 3.75), 'build', BC, TK(2)),
  ];

  // ------------------------------------------------------------ bars 17-20
  // Each bar opens on a 2x shot where the effect is on screen, the name slams, and
  // on beat 2 it goes wide while the recipe (content/synergies.ron) drops and fuses on beat 3.
  const at = (b, beat, src, rate = 1) => [B(b, beat), src / 30, rate];
  const e8 = (b, beats, src) => beats.map((bt) => at(b, bt, src));
  const SYN = [
    { name: 'FIRE WHEEL', id: 'syn-fire-wheel', a: 'saw-ring', b: 'hot-lead', icon: 'fire-wheel', bands: 'sun', fx: 'fire', style: 'drop',
      // The saws burn around Rex; the nova (frame 46) lands on the fuse.
      clk: [at(17, 0, 0, 1.09)],
      shots: [[0, TK(2), { type: 'iris', pre: Q, post: 0 }], [1, TK(3)], [2, {}]] },
    { name: 'CRYO BEAM', id: 'syn-cryo-beam', a: 'laser-eye', b: 'frost-halo', icon: 'cryo-beam', bands: 'ice', fx: 'frost', style: 'slam',
      // Frame 42: a freeze nova; 44-48: the ice beams fan out. Slow on the 2x, real time on the wide.
      clk: [at(18, 0, 42, 0.25), at(18, 2, 30)],
      shots: [[0, { k: 2, cx: 112, cy: 58 }, { type: 'whip', dir: 'l' }], [1, { k: 3, cx: 104, cy: 52 }], [2, {}]] },
    { name: 'BALL LIGHTNING', id: 'syn-ball-lightning', a: 'bone-rang', b: 'static-coil', icon: 'ball-lightning', bands: 'volt', k: 2, style: 'stamp', fx: 'volt',
      // Frames 14-17: the rangs arc lightning across the crowd. The crackle replays on every 8th.
      clk: [...[0, 0.5, 1, 1.5].map((bt) => at(19, bt, 14, 0.5)), at(19, 2, 27)],
      shots: [[0, { k: 2, cx: 222, cy: 116 }, { type: 'flash', color: P.ice }], [1, { k: 3, cx: 216, cy: 118 }], [2, {}]] },
    { name: 'EXTINCTION', id: 'syn-extinction', a: 'meteor-call', b: 'big-bones', icon: 'extinction', bands: 'blast', fx: 'impact', style: 'drop', height: 110,
      // Frame 8: the first meteor lands, 12: the second. Replayed on every 8th, then 16ths in the fill.
      clk: [at(20, 0, 7.5, 0.5), ...e8(20, [1, 1.5, 2], 7), ...[2.5, 2.75, 3, 3.25, 3.5, 3.75].map((bt, j) => at(20, bt, j % 2 ? 11 : 7.5))],
      shots: [[0, { k: 2, cx: 110, cy: 80 }, { type: 'slash' }], [1, { k: 3, cx: 72, cy: 62 }], [1.5, { k: 2, cx: 118, cy: 96 }], [2, {}],
        [2.5, { k: 2, cx: 110, cy: 80 }], [3, TK(2)], [3.25, TK(3)], [3.5, TK(2)], [3.75, TK(3)]] },
  ];
  const syn = [];
  SYN.forEach((s, i) => {
    const b = 17 + i;
    s.clock = X.clock(s.clk);
    s.shots.forEach(([bt, o, tr]) => syn.push(shot(B(b, bt), s.id, s.clock, o, tr)));
  });

  const REEL = [...loop, ...build, ...syn];

  // ------------------------------------------------------------ overlays
  // Recipe on the wide: A lands on beat 2, + pops, B lands on 2.5, they fuse into the icon on 3.
  function recipe(t, s, t0, i) {
    const cx = W / 2, y = 136;
    const fuse = t0 + BEAT * 3, go = fuse - 0.1;
    const band = X.BANDS[s.bands];
    if (t < fuse) {
      const u = t >= go ? PX.ease.in(clamp((t - go) / 0.1)) : 0;
      const off = Math.round(38 * (1 - u));
      const da = X.dropY(t, t0 + BEAT * 2, 40), db = X.dropY(t, t0 + BEAT * 2.5, 40);
      const fl = (h) => (t >= h && t - h < 0.05 ? P.bone : null);
      if (da) X.item(s.a, cx - off - 16, y + da[0], 2, { sx: da[1], sy: da[2], flash: fl(t0 + BEAT * 2), sticker: true });
      if (db) X.item(s.b, cx + off - 16, y + db[0], 2, { sx: db[1], sy: db[2], flash: fl(t0 + BEAT * 2.5), sticker: true });
      if (t >= t0 + BEAT * 2.25 && u === 0) {
        const pa = t - (t0 + BEAT * 2.25);
        PX.title('+', cx, y + 9 - (pa < 0.05 ? 2 : 0), { k: 2, bands: 'bone', depth: 2 });
      }
      return;
    }
    const a = t - fuse;
    const pop = a < 1 / 30 ? 1.5 : a < 2 / 30 ? 0.85 : a < 3 / 30 ? 1.08 : 1;
    if (a < 0.07) PX.circle(cx, y + 16, Math.max(2, 24 - a * 200), P.bone, true);
    PX.ring(a, cx, y + 16, { r0: 8, r1: 50, life: 0.35, color: band[0], color2: band[3] });
    X.synIcon(s.icon, cx - 24, y - 8, t, 3, { sx: pop, sy: pop, sticker: true });
    PX.burst(a, cx, y + 16, { n: 22, seed: 70 + i, speed: 120, life: 0.5, colors: [band[0], band[2], band[4], P.bone] });
  }

  function overlays(t, cur) {
    // AUTO-FIRE rides the 2x shot and squashes out as the punch-out lands.
    if (t >= B(10) - 0.2 && t < B(10, 2)) X.callout('AUTO-FIRE', t, B(10), { k: 3, y: 16, out: B(10, 2) - 0.1, exit: 'squash' });
    // DASH: two ice-graded frames, speed lines converging on Rex and his afterimage.
    if (t >= DASH && t < B(11, 1)) {
      const a = t - DASH;
      if (a < 2 / 30) X.grade(X.RAMP.ice, { gain: 1.8 });
      const f = Math.floor(EC(t) * 30 + 1e-6), p = DASH_AT[f] || DASH_AT[116];
      const [rx, ry] = X.toScreen(cur.view(t), p[0], p[1]);
      X.speedLines(t, rx, ry, { rx: 40 + a * 30, ry: 44 + a * 20, n: 44, colors: [P.ice, P.bone, P.cyan] });
    }
    if (t >= DASH - 0.2 && t < B(11, 2)) X.callout('DASH', t, DASH, { k: 5, x: 82, y: 14, bands: 'ice', out: B(11, 2) - 0.1, exit: 'squash' });
    // Cards: a glint sweeps the COMBO card, then a ring pings its tag, as the camera steps in.
    if (t >= B(12, 2) && t < PICK) {
      const a = t - B(12, 2), v = cur.view(t);
      const [x0, y0] = X.toScreen(v, CARD[0] + 1, CARD[1] + 1), [x1, y1] = X.toScreen(v, CARD[2] - 1, CARD[3] - 1);
      X.glint((a - 0.04) / 0.3, x0, y0, x1, y1);
      const [tx, ty] = X.toScreen(v, COMBO_TAG[0], COMBO_TAG[1]);
      PX.ring(a - BEAT, tx, ty, { r0: 20, r1: 80, life: 0.3, color: P.blush, color2: P.pink });
    }
    // The pick: sun-graded hit frames; a ring and sparks from the game's own burst; TAR PIT.
    if (t >= PICK && t < PICK + 0.7) {
      const a = t - PICK;
      if (a < 2 / 30) X.grade(X.RAMP.sun, { gain: 1.5, bias: 0.32 });
      const p = burstAt(LC(t)), [bx, by] = X.toScreen(cur.view(t), p[0], p[1]);
      PX.ring(a, bx, by, { r0: 10, r1: 120, life: 0.45, color: P.cream, color2: P.gold });
      PX.burst(a, bx, by, { n: 30, seed: 41, speed: 160, life: 0.6, colors: [P.cream, P.gold, P.amber, P.bone] });
    }
    if (t >= PICK - 0.2 && t < B(13, 2)) X.callout('TAR PIT', t, PICK, { k: 3, y: 16, bands: 'tar', out: B(13, 2) - 0.1, exit: 'squash' });
    // 54 ITEMS: the world drops to ink for the whole bar while every item erupts from the word.
    if (t >= B(15) - 0.2 && t < B(16) + 0.4) {
      const a = t - B(15);
      if (a >= 0 && t < B(16)) {
        // The color seeps back through a dither over the last beat, after the items have fallen out.
        const back = clamp((a - BEAT * 3) / BEAT), full = back > 0 ? X.snap('full') : null;
        if (a < 2 / 30) X.grade(X.RAMP.sun, { gain: 1.5, bias: 0.32 });
        else X.grade(X.RAMP.ink, { gain: 1.3 });
        if (full) X.masked(full, X.MASK.dither(PX.ease.in(back), { cell: 1 }));
        X.erupt(a, W / 2, 76, ITEMS, { k: 2, speed: 340, gravity: 520, up: 100, stagger: 0.05, seed: 11 });
      }
      X.callout('54 ITEMS', t, B(15), { k: 4, y: 58, out: B(16), exit: 'fall' });
    }
    // Synergies: the accent flares, the name rides the 2x shot, the recipe plays on the wide.
    SYN.forEach((s, i) => {
      const t0 = B(17 + i), t1 = B(18 + i);
      if (t < t0 - 0.2 || t >= t1) return;
      if (t >= t0) X.WEATHER[s.fx](t, t - t0, t > t1 - 0.1 ? 0 : 1);
      if (t < B(17 + i, 2)) X.callout(s.name, t, t0, { k: s.k || 3, y: 16, bands: s.bands, out: B(17 + i, 2) - 0.1, exit: 'squash', style: s.style, height: s.height || 60, stagger: 0.025 });
      if (t < t1 - 0.05) recipe(t, s, t0, i);
    });
  }

  // Whole-frame shakes on the hits.
  const HITS = [
    [B(9), 3], [B(10), 3], [B(10, 2), 3, 0.2], [DASH, 5, 0.4], [B(11, 2), 3], [B(12), 2], [PICK, 4], [B(14), 2], [B(15), 5, 0.45],
    ...SYN.map((s, i) => [B(17 + i), 4]),
    ...SYN.map((s, i) => [B(17 + i, 3), 2]),
    ...[1, 1.5, 2].map((bt) => [B(20, bt), 2, 0.2]),
    [B(20, 3), 2], [B(20, 3.5), 3],
  ];

  const cues = [
    [B(9), 'impact', 0.6],
    [B(10) - 0.1, 'whoosh', 0.35], [B(10), 'slam', 0.7],
    [B(10, 2), 'impact', 0.3],
    [DASH, 'dash', 0.9], [DASH, 'slam', 0.6],
    [B(11, 2) - 0.06, 'whoosh', 0.4], [B(11, 2), 'boom', 0.45],
    [B(12) - 0.1, 'whoosh', 0.4], [B(12), 'pop', 0.5], [B(12, 1), 'blip', 0.4], [B(12, 2), 'blip', 0.45], [B(12, 2) + 0.04, 'shine', 0.35], [B(12, 3), 'blip', 0.5],
    [PICK, 'select', 0.9], [PICK, 'slam', 0.5], [PICK, 'shine', 0.4],
    [B(13, 2) - 0.1, 'whoosh', 0.5],
    [B(15), 'slam', 0.8], [B(15), 'impact', 0.5],
    [B(15) + 0.03, 'pop', 0.3], [B(15) + 0.11, 'pop', 0.3], [B(15) + 0.2, 'pop', 0.26],
    [B(16) - 0.1, 'whoosh', 0.4],
    [B(17) - Q, 'whoosh', 0.35],
    ...SYN.map((s, i) => [B(17 + i), 'slam', 0.65]),
    ...SYN.flatMap((s, i) => [[B(17 + i, 2), 'land', 0.4], [B(17 + i, 2.5), 'land', 0.45], [B(17 + i, 3), 'shine', 0.55]]),
    [B(18) - 0.1, 'whoosh', 0.45],
    [B(19), 'zap', 0.7],
    [B(20), 'boom', 0.6], [B(20, 1), 'boom', 0.35], [B(20, 1.5), 'boom', 0.3], [B(20, 2), 'boom', 0.35],
  ];

  PX.scene({
    id: 'gameplay-a', start: B(9), end: B(21), layer: 10, cues,
    draw(t) {
      X.playReel(REEL, t);
      overlays(t, X.current(REEL, t));
      const [dx, dy] = X.shakeAt(t, HITS);
      X.shakeFrame(dx, dy);
    },
  });
})();
