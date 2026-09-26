// PX: the trailer's pixel engine. Everything is drawn on one 320x180 buffer,
// then blown up 6x with no smoothing. Every frame is a pure function of the
// global time `t` (seconds), so HyperFrames can seek anywhere.
(function () {
  const W = 320, H = 180, SCALE = 6;
  const BPM = 128, BEAT = 60 / BPM, BAR = BEAT * 4;
  const DURATION = 75;

  // Tarpit 32, the game's one fixed palette (src/render/palette.rs).
  const P = {
    ink: '#0f0b18', night: '#1d1629', dusk: '#2d2340', slate: '#45395c', mauve: '#675a7c', haze: '#948aa6', fog: '#c9c2d4', bone: '#f5efe0',
    maroon: '#4a1530', blood: '#86203a', red: '#cc3a3f', ember: '#ef6b3a', amber: '#f7a041', gold: '#ffd25e', cream: '#fff4b0',
    deep: '#0f3134', moss: '#1b5a45', leaf: '#2e904f', lime: '#7cc84b', sprout: '#cdeb72',
    navy: '#1a2a5e', blue: '#2a5aa8', sky: '#3e9ce0', cyan: '#74dcee', ice: '#d2fbf6',
    plum: '#3e1b58', grape: '#7a36a0', pink: '#da4f9e', blush: '#ff99c4',
    umber: '#52301f', clay: '#955c38', sand: '#dca56f',
  };

  // Title color bands, top glyph row to bottom (9 rows), like the TREX logo.
  const BANDS = {
    sun: [P.cream, P.gold, P.gold, P.amber, P.amber, P.ember, P.ember, P.red, P.blood],
    dread: [P.blush, P.pink, P.pink, P.grape, P.grape, P.plum, P.plum, P.blood, P.maroon],
    ice: [P.ice, P.cyan, P.cyan, P.cyan, P.sky, P.sky, P.blue, P.blue, P.navy],
    venom: [P.sprout, P.lime, P.lime, P.leaf, P.leaf, P.moss, P.moss, P.deep, P.deep],
    bone: [P.bone, P.bone, P.fog, P.fog, P.haze, P.haze, P.mauve, P.slate, P.dusk],
  };
  const EDGE = { sun: P.maroon, dread: P.ink, ice: P.navy, venom: P.deep, bone: P.slate };

  // --- buffer ---------------------------------------------------------------
  const buf = document.createElement('canvas');
  buf.width = W; buf.height = H;
  const g = buf.getContext('2d', { willReadFrequently: false });
  g.imageSmoothingEnabled = false;

  function rect(x, y, w, h, c) { g.fillStyle = c; g.fillRect(Math.round(x), Math.round(y), Math.round(w), Math.round(h)); }
  function px(x, y, c) { g.fillStyle = c; g.fillRect(Math.round(x), Math.round(y), 1, 1); }
  function clear(c) { rect(0, 0, W, H, c || P.ink); }
  // Pixel line (Bresenham).
  function line(x0, y0, x1, y1, c) {
    x0 = Math.round(x0); y0 = Math.round(y0); x1 = Math.round(x1); y1 = Math.round(y1);
    const dx = Math.abs(x1 - x0), dy = -Math.abs(y1 - y0), sx = x0 < x1 ? 1 : -1, sy = y0 < y1 ? 1 : -1;
    let e = dx + dy;
    g.fillStyle = c;
    for (let i = 0; i < 2000; i++) {
      g.fillRect(x0, y0, 1, 1);
      if (x0 === x1 && y0 === y1) break;
      const e2 = 2 * e;
      if (e2 >= dy) { e += dy; x0 += sx; }
      if (e2 <= dx) { e += dx; y0 += sy; }
    }
  }
  // Pixel circle outline / disc.
  function circle(cx, cy, r, c, fill) {
    cx = Math.round(cx); cy = Math.round(cy); r = Math.round(r);
    g.fillStyle = c;
    for (let y = -r; y <= r; y++) {
      const w = Math.round(Math.sqrt(Math.max(0, r * r - y * y + r * 0.8)));
      if (fill) g.fillRect(cx - w, cy + y, w * 2 + 1, 1);
      else { g.fillRect(cx - w, cy + y, 1, 1); g.fillRect(cx + w, cy + y, 1, 1); }
    }
    if (!fill) for (let x = -r; x <= r; x++) {
      const h = Math.round(Math.sqrt(Math.max(0, r * r - x * x + r * 0.8)));
      g.fillRect(cx + x, cy - h, 1, 1); g.fillRect(cx + x, cy + h, 1, 1);
    }
  }

  // 4x4 Bayer threshold: dither(a) covers fraction a of the screen with color c.
  const BAYER = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5];
  function dither(a, c, x0 = 0, y0 = 0, w = W, h = H) {
    if (a <= 0) return;
    if (a >= 1) return rect(x0, y0, w, h, c);
    g.fillStyle = c;
    const th = a * 16;
    for (let y = y0; y < y0 + h; y++) for (let x = x0; x < x0 + w; x++) if (BAYER[(y & 3) * 4 + (x & 3)] < th) g.fillRect(x, y, 1, 1);
  }
  const bayer = (x, y) => BAYER[(y & 3) * 4 + (x & 3)] / 16;

  // --- math / time ------------------------------------------------------------
  const clamp = (v, a = 0, b = 1) => Math.max(a, Math.min(b, v));
  const lerp = (a, b, u) => a + (b - a) * u;
  const ease = {
    out: (u) => 1 - Math.pow(1 - u, 3),
    in: (u) => u * u * u,
    inOut: (u) => (u < 0.5 ? 4 * u * u * u : 1 - Math.pow(-2 * u + 2, 3) / 2),
    back: (u) => { const c = 1.9; return 1 + (c + 1) * Math.pow(u - 1, 3) + c * Math.pow(u - 1, 2); },
    elastic: (u) => (u <= 0 ? 0 : u >= 1 ? 1 : Math.pow(2, -10 * u) * Math.sin((u * 10 - 0.75) * (2 * Math.PI / 3)) + 1),
  };
  // Bar n (1-based) start time; beat b (0-based within the bar).
  const bar = (n, b = 0) => (n - 1) * BAR + b * BEAT;
  // Quantize time to a stop-motion frame rate ("on twos" at 12 fps by default).
  const step = (t, fps = 12) => Math.floor(t * fps + 1e-6) / fps;
  // Pulse that spikes on every beat and decays: 1 at the hit, ~0 by the next.
  const pulse = (t, every = BEAT, decay = 6) => { const u = ((t % every) + every) % every; return Math.exp(-u * decay); };

  // Seeded PRNG (mulberry32) and a stateless hash for per-index randomness.
  function rng(seed) {
    let a = seed >>> 0;
    return function () {
      a = (a + 0x6d2b79f5) >>> 0;
      let t = Math.imul(a ^ (a >>> 15), 1 | a);
      t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
      return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
    };
  }
  const hash = (i, s = 0) => { let h = Math.imul((i | 0) ^ 0x9e3779b9, 0x85ebca6b) ^ Math.imul(s | 0, 0xc2b2ae35); h ^= h >>> 15; h = Math.imul(h, 0x27d4eb2f); h ^= h >>> 13; return (h >>> 0) / 4294967296; };

  // --- assets ---------------------------------------------------------------
  const images = {};
  const pending = [];
  function load(key, src) {
    if (images[key]) return images[key];
    const im = new Image();
    images[key] = im;
    pending.push(new Promise((res) => { im.onload = res; im.onerror = () => { console.warn('PX: missing', src); res(); }; }));
    im.src = src;
    return im;
  }
  // Sprite frames by manifest name ("rex", "colossus", "logo"...), frame index wraps.
  let manifest = {};
  function sprite(name, frame = 0) {
    const m = manifest[name];
    if (!m) return null;
    return images['s:' + m.files[((frame % m.files.length) + m.files.length) % m.files.length]];
  }
  function spr(name, x, y, frame = 0, opts = {}) {
    const im = sprite(name, frame);
    if (!im || !im.width) return;
    const k = opts.k || 1;
    const w = im.width * k, h = im.height * k;
    g.save();
    if (opts.alpha != null) g.globalAlpha = opts.alpha;
    if (opts.flip) { g.translate(Math.round(x) + w, Math.round(y)); g.scale(-1, 1); g.drawImage(im, 0, 0, w, h); }
    else g.drawImage(im, Math.round(x), Math.round(y), w, h);
    g.restore();
    if (opts.flash) silhouette(im, x, y, k, opts.flash, opts.flip);
  }
  // Draw an image's opaque pixels as a flat color (hit flash, shadows, outlines).
  const tint = document.createElement('canvas'), tg = tint.getContext('2d');
  function silhouette(im, x, y, k, color, flip) {
    tint.width = im.width * k; tint.height = im.height * k;
    tg.imageSmoothingEnabled = false;
    tg.globalCompositeOperation = 'source-over';
    tg.clearRect(0, 0, tint.width, tint.height);
    if (flip) { tg.save(); tg.translate(tint.width, 0); tg.scale(-1, 1); }
    tg.drawImage(im, 0, 0, tint.width, tint.height);
    if (flip) tg.restore();
    tg.globalCompositeOperation = 'source-in';
    tg.fillStyle = color; tg.fillRect(0, 0, tint.width, tint.height);
    g.drawImage(tint, Math.round(x), Math.round(y));
  }
  const size = (name) => (manifest[name] ? manifest[name].size : [0, 0]);
  const frames = (name) => (manifest[name] ? manifest[name].files.length : 0);

  // Gameplay footage: assets/footage/<id>/NNNN.png at 30 fps, 320x180, from
  // the game's own frame dumper. `footage(id, localT)` draws the frame at localT.
  let footageIndex = {};
  function footage(id, localT, opts = {}) {
    const f = footageIndex[id];
    if (!f) { rect(0, 0, W, H, P.night); text(`MISSING ${id}`.toUpperCase(), 8, 8, P.red); return; }
    const i = clamp(Math.floor(localT * f.fps + 1e-6), 0, f.frames - 1);
    const im = images['f:' + id + ':' + i];
    if (!im || !im.width) return;
    const k = opts.k || 1;
    const cx = opts.cx != null ? opts.cx : f.w / 2, cy = opts.cy != null ? opts.cy : f.h / 2; // zoom focus, in footage px
    g.drawImage(im, Math.round(W / 2 - cx * k + (opts.dx || 0)), Math.round(H / 2 - cy * k + (opts.dy || 0)), f.w * k, f.h * k);
  }

  // The raw frame image of footage `id` at localT (for drawing into panes, masks, etc.).
  function footageFrame(id, localT) {
    const f = footageIndex[id];
    if (!f) return null;
    const i = clamp(Math.floor(localT * f.fps + 1e-6), 0, f.frames - 1);
    return images['f:' + id + ':' + i] || null;
  }

  // --- text -----------------------------------------------------------------
  // Small 3x5 game font (variable width).
  function glyphSmall(ch) { const gl = PX_FONT_SMALL.glyphs[ch] || PX_FONT_SMALL.glyphs[ch.toUpperCase()]; return gl || ['...', '...', '...', '...', '...']; }
  function textWidth(s, k = 1) { let w = 0; for (const ch of s) w += (ch === ' ' ? 3 : glyphSmall(ch)[0].length) + 1; return Math.max(0, w - 1) * k; }
  // Small text. opts: k (scale), outline color (default ink), shadow (bool), align ('left'|'center'|'right').
  function text(s, x, y, c, opts = {}) {
    const k = opts.k || 1;
    if (opts.align === 'center') x -= textWidth(s, k) / 2;
    else if (opts.align === 'right') x -= textWidth(s, k);
    x = Math.round(x); y = Math.round(y);
    const outline = opts.outline === undefined ? P.ink : opts.outline;
    const draw = (ox, oy, col) => {
      let cx = x;
      g.fillStyle = col;
      for (const ch of s) {
        if (ch === ' ') { cx += 4 * k; continue; }
        const gl = glyphSmall(ch);
        for (let j = 0; j < gl.length; j++) for (let i = 0; i < gl[j].length; i++) if (gl[j][i] === '#') g.fillRect(cx + i * k + ox, y + j * k + oy, k, k);
        cx += (gl[0].length + 1) * k;
      }
    };
    if (outline) {
      const offs = [[-1, -1], [0, -1], [1, -1], [-1, 0], [1, 0], [-1, 1], [0, 1], [1, 1]];
      if (opts.shadow !== false) offs.push([-1, 2], [0, 2], [1, 2]);
      for (const [a, b] of offs) draw(a, b, outline);
    }
    draw(0, 0, c);
  }

  // Display font: chunky 7x9 glyphs in the style of the TREX logo.
  const BIG = {
    A: ['.#####.', '#######', '##...##', '##...##', '#######', '#######', '##...##', '##...##', '##...##'],
    B: ['######.', '#######', '##...##', '######.', '######.', '##...##', '##...##', '#######', '######.'],
    C: ['.######', '#######', '##.....', '##.....', '##.....', '##.....', '##.....', '#######', '.######'],
    D: ['######.', '#######', '##...##', '##...##', '##...##', '##...##', '##...##', '#######', '######.'],
    E: ['#######', '#######', '##.....', '######.', '######.', '##.....', '##.....', '#######', '#######'],
    F: ['#######', '#######', '##.....', '######.', '######.', '##.....', '##.....', '##.....', '##.....'],
    G: ['.######', '#######', '##.....', '##.....', '##.####', '##.####', '##...##', '#######', '.#####.'],
    H: ['##...##', '##...##', '##...##', '#######', '#######', '##...##', '##...##', '##...##', '##...##'],
    I: ['######', '######', '..##..', '..##..', '..##..', '..##..', '..##..', '######', '######'],
    J: ['....###', '....###', '.....##', '.....##', '.....##', '##...##', '##...##', '#######', '.#####.'],
    K: ['##...##', '##..###', '##.###.', '#####..', '####...', '#####..', '##.###.', '##..###', '##...##'],
    L: ['##.....', '##.....', '##.....', '##.....', '##.....', '##.....', '##.....', '#######', '#######'],
    M: ['##...##', '###.###', '#######', '##.#.##', '##.#.##', '##...##', '##...##', '##...##', '##...##'],
    N: ['##...##', '###..##', '####.##', '#######', '##.####', '##..###', '##...##', '##...##', '##...##'],
    O: ['.#####.', '#######', '##...##', '##...##', '##...##', '##...##', '##...##', '#######', '.#####.'],
    P: ['######.', '#######', '##...##', '##...##', '#######', '######.', '##.....', '##.....', '##.....'],
    Q: ['.#####.', '#######', '##...##', '##...##', '##...##', '##.#.##', '##..###', '#######', '.######'],
    R: ['######.', '#######', '##...##', '##...##', '######.', '#####..', '##.###.', '##..###', '##...##'],
    S: ['.######', '#######', '##.....', '######.', '.######', '.....##', '.....##', '#######', '######.'],
    T: ['#######', '#######', '..###..', '..###..', '..###..', '..###..', '..###..', '..###..', '..###..'],
    U: ['##...##', '##...##', '##...##', '##...##', '##...##', '##...##', '##...##', '#######', '.#####.'],
    V: ['##...##', '##...##', '##...##', '##...##', '##...##', '###.###', '.#####.', '..###..', '...#...'],
    W: ['##...##', '##...##', '##...##', '##...##', '##.#.##', '##.#.##', '#######', '###.###', '##...##'],
    X: ['##...##', '##...##', '###.###', '.#####.', '..###..', '.#####.', '###.###', '##...##', '##...##'],
    Y: ['##...##', '##...##', '###.###', '.#####.', '..###..', '..###..', '..###..', '..###..', '..###..'],
    Z: ['#######', '#######', '....###', '...###.', '..###..', '.###...', '###....', '#######', '#######'],
    0: ['.#####.', '#######', '##...##', '##..###', '##.#.##', '###..##', '##...##', '#######', '.#####.'],
    1: ['..###', '.####', '#####', '..###', '..###', '..###', '..###', '#####', '#####'],
    2: ['.#####.', '#######', '##...##', '....###', '..####.', '.####..', '###....', '#######', '#######'],
    3: ['######.', '#######', '.....##', '..####.', '..#####', '.....##', '.....##', '#######', '######.'],
    4: ['##...##', '##...##', '##...##', '#######', '#######', '.....##', '.....##', '.....##', '.....##'],
    5: ['#######', '##.....', '######.', '#######', '.....##', '.....##', '##...##', '#######', '.#####.'],
    6: ['.#####.', '#######', '##.....', '######.', '#######', '##...##', '##...##', '#######', '.#####.'],
    7: ['#######', '#######', '....##.', '...##..', '...##..', '..##...', '..##...', '..##...', '..##...'],
    8: ['.#####.', '##...##', '##...##', '.#####.', '#######', '##...##', '##...##', '#######', '.#####.'],
    9: ['.#####.', '#######', '##...##', '##...##', '#######', '.######', '.....##', '#######', '.#####.'],
    '!': ['###', '###', '###', '###', '###', '###', '...', '###', '###'],
    '?': ['.#####.', '#######', '##...##', '....###', '..####.', '..###..', '.......', '..###..', '..###..'],
    ':': ['..', '##', '##', '..', '..', '##', '##', '..', '..'],
    '.': ['..', '..', '..', '..', '..', '..', '..', '##', '##'],
    ',': ['..', '..', '..', '..', '..', '..', '##', '##', '#.'],
    '-': ['.....', '.....', '.....', '#####', '#####', '.....', '.....', '.....', '.....'],
    "'": ['##', '##', '#.', '..', '..', '..', '..', '..', '..'],
    '+': ['.....', '.....', '..#..', '..#..', '#####', '..#..', '..#..', '.....', '.....'],
    ' ': ['...', '...', '...', '...', '...', '...', '...', '...', '...'],
  };
  const bigGlyph = (ch) => BIG[ch] || BIG[ch.toUpperCase()] || BIG[' '];
  function titleWidth(s, k = 2) { let w = 0; for (const ch of s) w += bigGlyph(ch)[0].length + 1; return Math.max(0, w - 1) * k; }

  // Big title text, TREX-logo style: row color bands, darker right/bottom
  // edge, ink outline, a 3px extrusion, and an optional diagonal shine.
  // opts: k (pixel scale, default 2), bands (name or array), fx(i) -> {dx, dy, sx, sy, vis, flash},
  //       shine (position along x+y in screen px, or null), align ('center' default | 'left'),
  //       outline (color), depth (extrusion px, default 3).
  function title(s, x, y, opts = {}) {
    const k = opts.k || 2;
    const bandName = typeof opts.bands === 'string' ? opts.bands : 'sun';
    const bands = Array.isArray(opts.bands) ? opts.bands : BANDS[bandName];
    const edgeC = opts.edge || EDGE[bandName] || P.maroon;
    const outline = opts.outline || P.ink;
    const depth = opts.depth == null ? 3 : opts.depth;
    const fx = opts.fx || (() => ({}));
    const tw = titleWidth(s, k);
    let cx = opts.align === 'left' ? x : x - tw / 2;
    const pix = [];
    [...s].forEach((ch, i) => {
      const m = bigGlyph(ch), gw = m[0].length, gh = m.length;
      const f = Object.assign({ dx: 0, dy: 0, sx: 1, sy: 1, vis: true }, fx(i) || {});
      if (f.vis !== false) {
        const ww = Math.max(1, Math.round(gw * k * f.sx)), hh = Math.max(1, Math.round(gh * k * f.sy));
        const ox = Math.round(cx + (gw * k) / 2 - ww / 2 + f.dx), oy = Math.round(y + gh * k - hh + f.dy);
        for (let j = 0; j < hh; j++) for (let ii = 0; ii < ww; ii++) {
          const gx = Math.min(gw - 1, Math.floor((ii / ww) * gw)), gy = Math.min(gh - 1, Math.floor((j / hh) * gh));
          if (m[gy][gx] !== '#') continue;
          pix.push([ox + ii, oy + j, gy, false, f.flash]);
        }
      }
      cx += (gw + 1) * k;
    });
    // Darker 1px edge on the right and bottom of every stroke, like the logo.
    const occ = new Set(pix.map(([a, b]) => a * 4096 + b));
    for (const q of pix) q[3] = !occ.has((q[0] + 1) * 4096 + q[1]) || !occ.has(q[0] * 4096 + q[1] + 1);
    for (const [a, b] of pix) for (let d = 1; d <= depth; d++) { g.fillStyle = d < depth ? edgeC : outline; g.fillRect(a, b + d + 1, 1, 1); }
    g.fillStyle = outline;
    for (const [a, b] of pix) g.fillRect(a - 1, b - 1, 3, 3);
    for (const [a, b, gy, edge, flash] of pix) {
      let c = edge ? edgeC : bands[Math.min(bands.length - 1, gy)];
      if (flash) c = flash;
      else if (opts.shine != null && !edge) { const d = a + b - opts.shine; if (d > -7 && d < 0) c = d > -3 ? '#ffffff' : P.cream; }
      g.fillStyle = c; g.fillRect(a, b, 1, 1);
    }
    return tw;
  }

  // Letter drop-in with squash and stretch. Returns an fx(i) for title().
  // `age` is seconds since the title started; quantize it with step() for stop motion.
  function dropIn(age, stagger = 0.06, height = 70) {
    return (i) => {
      const a = age - i * stagger;
      if (a < 0) return { vis: false };
      if (a < 0.16) { const u = a / 0.16; return { dy: -height * (1 - u * u), sx: 0.8, sy: 1.3 }; }
      if (a < 0.24) return { sx: 1.35, sy: 0.65, flash: a < 0.2 ? '#ffffff' : null };
      if (a < 0.32) return { dy: -3, sx: 0.9, sy: 1.12 };
      return {};
    };
  }
  // Slam: the whole word scales from `from` to 1 over `dur`, then squashes.
  function slamScale(age, dur = 0.14, from = 4) {
    if (age < 0) return null;
    if (age < dur) return lerp(from, 1, ease.in(age / dur));
    return 1;
  }

  // --- particles (stateless: position is a closed-form function of age) -------
  // burst(age, x, y, {n, seed, speed, gravity, life, colors, size, up})
  function burst(age, x, y, o = {}) {
    const n = o.n || 16, seed = o.seed || 1, speed = o.speed || 60, grav = o.gravity == null ? 160 : o.gravity;
    const life = o.life || 0.7, cols = o.colors || [P.bone, P.gold, P.amber];
    if (age < 0 || age > life * 1.6) return;
    for (let i = 0; i < n; i++) {
      const r1 = hash(i, seed), r2 = hash(i, seed + 7), r3 = hash(i, seed + 13);
      const l = life * (0.5 + r3 * 0.5);
      if (age > l) continue;
      const a = (o.angle != null ? o.angle + (r1 - 0.5) * (o.spread || 1) : r1 * Math.PI * 2);
      const s = speed * (0.35 + r2 * 0.65);
      const vx = Math.cos(a) * s, vy = Math.sin(a) * s - (o.up || 30);
      const px_ = x + vx * age, py_ = y + vy * age + 0.5 * grav * age * age;
      const sz = (o.size || 2) * (age < l * 0.5 ? 1 : 0.5);
      rect(px_, py_, Math.max(1, sz), Math.max(1, sz), cols[i % cols.length]);
    }
  }
  // Expanding pixel ring (impact / shockwave).
  function ring(age, x, y, o = {}) {
    const life = o.life || 0.4;
    if (age < 0 || age > life) return;
    const u = age / life;
    circle(x, y, (o.r0 || 4) + ease.out(u) * (o.r1 || 60), u < 0.5 ? (o.color || P.bone) : (o.color2 || o.color || P.mauve));
  }

  // --- camera shake -----------------------------------------------------------
  // Deterministic shake offset for `age` seconds after a hit.
  function shake(age, amp = 4, dur = 0.35, seed = 3) {
    if (age < 0 || age > dur) return [0, 0];
    const f = Math.floor(age * 30), d = 1 - age / dur;
    return [Math.round((hash(f, seed) - 0.5) * 2 * amp * d), Math.round((hash(f, seed + 1) - 0.5) * 2 * amp * d)];
  }

  // --- scenes -----------------------------------------------------------------
  // PX.scene({id, start, end, layer, draw(t, local, scene), cues:[[t, name, gain?]]})
  // Scenes whose [start, end) contains t draw in ascending `layer` order.
  const scenes = [];
  function scene(s) { s.layer = s.layer || 0; scenes.push(s); return s; }

  const out = { canvas: null, ctx: null };
  function render(t) {
    t = clamp(t, 0, DURATION - 1e-4);
    g.save();
    clear(P.ink);
    const live = scenes.filter((s) => t >= s.start && t < s.end).sort((a, b) => a.layer - b.layer);
    for (const s of live) { g.save(); s.draw(t, t - s.start, s); g.restore(); }
    g.restore();
    if (out.ctx) { out.ctx.imageSmoothingEnabled = false; out.ctx.drawImage(buf, 0, 0, W * SCALE, H * SCALE); }
  }

  async function boot(canvas, opts = {}) {
    out.canvas = canvas; out.ctx = canvas.getContext('2d');
    const base = opts.base || '';
    manifest = await (await fetch(base + 'assets/sprites/sprites.json')).json();
    for (const k in manifest) for (const f of manifest[k].files) load('s:' + f, base + 'assets/' + f);
    manifest.logo = manifest.logo || { size: [50, 15], files: ['sprites/ui/logo.png'] };
    load('s:sprites/ui/logo.png', base + 'assets/sprites/ui/logo.png');
    try { footageIndex = await (await fetch(base + 'assets/footage/index.json')).json(); } catch (e) { footageIndex = {}; }
    for (const id in footageIndex) {
      const f = footageIndex[id];
      for (let i = 0; i < f.frames; i++) load('f:' + id + ':' + i, `${base}assets/footage/${id}/${String(i).padStart(4, '0')}.png`);
    }
    for (const src of opts.extra || []) load(src, base + src);
    await Promise.all(pending);
  }

  window.PX = {
    W, H, SCALE, BPM, BEAT, BAR, DURATION, P, BANDS,
    g, rect, px, clear, line, circle, dither, bayer,
    clamp, lerp, ease, bar, step, pulse, rng, hash,
    images, load, sprite, spr, silhouette, size, frames, footage, footageFrame, get footageIndex() { return footageIndex; },
    text, textWidth, title, titleWidth, dropIn, slamScale,
    burst, ring, shake,
    scenes, scene, render, boot,
  };
})();
