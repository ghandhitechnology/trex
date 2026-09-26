// tmux.js: the kit behind the cold open (00-coldopen.js) and the outro (30-outro.js).
// A pixel tmux screen (Ghostty window, Claude Code pane, btop strip, trex pane,
// status bar), a 3x5 monospace terminal font with lowercase, string-art sprites
// written in the game's palette letters, Clawd, Rex's acting poses, the TREX logo
// with per-letter motion, solid backdrops and an integer-zoom camera. Every draw is
// a pure function of its arguments; nothing is kept between frames except memoized parses.
(function () {
  const { W, H, P } = PX;
  const g = PX.g;

  // ------------------------------------------------------------ palette letters
  // Same letters as src/render/palette.rs CHARS, so art can be pasted from the game.
  const CH = 'kndsvepwmcroayYtfglLNbBiIuPhHzZx';
  const PAL = [P.ink, P.night, P.dusk, P.slate, P.mauve, P.haze, P.fog, P.bone, P.maroon, P.blood, P.red, P.ember, P.amber, P.gold, P.cream,
    P.deep, P.moss, P.leaf, P.lime, P.sprout, P.navy, P.blue, P.sky, P.cyan, P.ice, P.plum, P.grape, P.pink, P.blush, P.umber, P.clay, P.sand];
  const pcol = (c) => PAL[CH.indexOf(c)];

  // ------------------------------------------------------------ string art
  // parse(rows) -> {w, h, px:[[x, y, color]]}, with the game's 1px ink ring (4-neighbour).
  const artCache = new Map();
  function parse(rows, outline = true) {
    const key = (outline ? '1' : '0') + rows.join('|');
    let d = artCache.get(key);
    if (d) return d;
    const o = outline ? 1 : 0;
    const w0 = Math.max(...rows.map((r) => r.length)), h0 = rows.length;
    const w = w0 + 2 * o, h = h0 + 2 * o;
    const grid = new Array(w * h).fill(null);
    rows.forEach((r, j) => { for (let i = 0; i < r.length; i++) { const c = r[i]; if (c !== '.' && c !== ' ') grid[(j + o) * w + i + o] = pcol(c) || P.bone; } });
    if (outline) {
      const src = grid.slice();
      const solid = (x, y) => x >= 0 && y >= 0 && x < w && y < h && src[y * w + x];
      for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
        if (!src[y * w + x] && (solid(x - 1, y) || solid(x + 1, y) || solid(x, y - 1) || solid(x, y + 1))) grid[y * w + x] = P.ink;
      }
    }
    const px = [];
    for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) if (grid[y * w + x]) px.push([x, y, grid[y * w + x]]);
    d = { w, h, px, w0 };
    artCache.set(key, d);
    return d;
  }
  // edit(rows, [[i, j, ch]...]) -> rows with single letters replaced (memoized).
  const editCache = new Map();
  function edit(rows, edits) {
    if (!edits || !edits.length) return rows;
    const key = rows.join('|') + '#' + edits.map((e) => e.join(',')).join(';');
    let r = editCache.get(key);
    if (r) return r;
    const a = rows.map((s) => s.split(''));
    for (const [i, j, c] of edits) if (a[j] && i >= 0 && i < a[j].length) a[j][i] = c;
    r = a.map((s) => s.join(''));
    editCache.set(key, r);
    return r;
  }
  // overlay(base, top, dx, dy): paste `top` art over `base` (non-dot letters win), growing as needed.
  const ovCache = new Map();
  function overlay(base, top, dx = 0, dy = 0) {
    const key = base.join('|') + '#' + top.join('|') + '#' + dx + ',' + dy;
    let r = ovCache.get(key);
    if (r) return r;
    const y0 = Math.min(0, dy), x0 = Math.min(0, dx);
    const h = Math.max(base.length, top.length + dy) - y0;
    const w = Math.max(...base.map((s) => s.length), ...top.map((s) => s.length + dx)) - x0;
    const a = Array.from({ length: h }, () => new Array(w).fill('.'));
    const paste = (rows, ox, oy) => rows.forEach((s, j) => { for (let i = 0; i < s.length; i++) if (s[i] !== '.' && s[i] !== ' ') a[j + oy - y0][i + ox - x0] = s[i]; });
    paste(base, 0, 0); paste(top, dx, dy);
    r = a.map((s) => s.join(''));
    ovCache.set(key, r);
    return r;
  }
  // Draw art with its bottom at `by`. The anchor column o.ax (art px, default the middle)
  // sits on bx, mirrored with o.flip, so poses of different widths stand on the same feet.
  // o: k (px per art px), kx/ky (squash), flip, flash (solid color, ink kept),
  // color(c, i, j) remap (return null to skip), outline (default true).
  function art(rows, bx, by, o = {}) {
    const d = parse(rows, o.outline !== false);
    const ol = o.outline !== false ? 1 : 0;
    const kx = o.kx || o.k || 1, ky = o.ky || o.k || 1;
    let axp = d.w / 2;
    if (o.ax != null) axp = o.flip ? d.w - (o.ax + ol) : o.ax + ol;
    const x0 = Math.round(bx - axp * kx), y0 = Math.round(by - d.h * ky);
    for (const [i, j, c0] of d.px) {
      const ii = o.flip ? d.w - 1 - i : i;
      let c = c0;
      if (o.flash && c0 !== P.ink) c = o.flash;
      if (o.color) { c = o.color(c, i, j); if (!c) continue; }
      g.fillStyle = c;
      g.fillRect(x0 + ii * kx, y0 + j * ky, kx, ky);
    }
    return { x: x0, y: y0, w: d.w * kx, h: d.h * ky };
  }
  // Same, from a {x,y: color} pixel map built in code (Clawd).
  function mapArt(map, ox, oy, k, outline = true) {
    const pts = [...map.values()];
    const has = new Set(pts.map(([x, y]) => x * 1000 + y));
    if (outline) {
      g.fillStyle = P.ink;
      const ring = new Set();
      for (const [x, y] of pts) for (const [a, b] of [[1, 0], [-1, 0], [0, 1], [0, -1]]) {
        const q = (x + a) * 1000 + (y + b);
        if (!has.has(q) && !ring.has(q)) { ring.add(q); g.fillRect(ox + (x + a) * k, oy + (y + b) * k, k, k); }
      }
    }
    for (const [x, y, c] of pts) { g.fillStyle = c; g.fillRect(ox + x * k, oy + y * k, k, k); }
  }

  // ------------------------------------------------------------ terminal font
  // 3x5 caps and digits, lowercase with a 4-row x-height and a descender row,
  // on a 4px cell and 7px line, so text sits on a terminal grid.
  const M = {
    A: ['.#.', '#.#', '###', '#.#', '#.#'], B: ['##.', '#.#', '##.', '#.#', '##.'], C: ['.##', '#..', '#..', '#..', '.##'],
    D: ['##.', '#.#', '#.#', '#.#', '##.'], E: ['###', '#..', '##.', '#..', '###'], F: ['###', '#..', '##.', '#..', '#..'],
    G: ['.##', '#..', '#.#', '#.#', '.##'], H: ['#.#', '#.#', '###', '#.#', '#.#'], I: ['###', '.#.', '.#.', '.#.', '###'],
    J: ['..#', '..#', '..#', '#.#', '.#.'], K: ['#.#', '#.#', '##.', '#.#', '#.#'], L: ['#..', '#..', '#..', '#..', '###'],
    M: ['#.#', '###', '###', '#.#', '#.#'], N: ['##.', '#.#', '#.#', '#.#', '#.#'], O: ['.#.', '#.#', '#.#', '#.#', '.#.'],
    P: ['##.', '#.#', '##.', '#..', '#..'], Q: ['.#.', '#.#', '#.#', '##.', '.##'], R: ['##.', '#.#', '##.', '#.#', '#.#'],
    S: ['.##', '#..', '.#.', '..#', '##.'], T: ['###', '.#.', '.#.', '.#.', '.#.'], U: ['#.#', '#.#', '#.#', '#.#', '###'],
    V: ['#.#', '#.#', '#.#', '.#.', '.#.'], W: ['#.#', '#.#', '###', '###', '#.#'], X: ['#.#', '#.#', '.#.', '#.#', '#.#'],
    Y: ['#.#', '#.#', '.#.', '.#.', '.#.'], Z: ['###', '..#', '.#.', '#..', '###'],
    a: ['...', '.##', '#.#', '#.#', '.##'], b: ['#..', '##.', '#.#', '#.#', '##.'], c: ['...', '.##', '#..', '#..', '.##'],
    d: ['..#', '.##', '#.#', '#.#', '.##'], e: ['...', '.#.', '###', '#..', '.##'], f: ['.##', '.#.', '###', '.#.', '.#.'],
    g: ['...', '.##', '#.#', '.##', '..#', '##.'], h: ['#..', '##.', '#.#', '#.#', '#.#'], i: ['.#.', '...', '##.', '.#.', '###'],
    j: ['..#', '...', '..#', '..#', '#.#', '.#.'], k: ['#..', '#.#', '##.', '##.', '#.#'], l: ['##.', '.#.', '.#.', '.#.', '###'],
    m: ['...', '##.', '###', '#.#', '#.#'], n: ['...', '##.', '#.#', '#.#', '#.#'], o: ['...', '.#.', '#.#', '#.#', '.#.'],
    p: ['...', '##.', '#.#', '#.#', '##.', '#..'], q: ['...', '.##', '#.#', '#.#', '.##', '..#'], r: ['...', '#.#', '##.', '#..', '#..'],
    s: ['...', '.##', '#..', '..#', '##.'], t: ['.#.', '###', '.#.', '.#.', '.##'], u: ['...', '#.#', '#.#', '#.#', '.##'],
    v: ['...', '#.#', '#.#', '#.#', '.#.'], w: ['...', '#.#', '#.#', '###', '#.#'], x: ['...', '#.#', '.#.', '.#.', '#.#'],
    y: ['...', '#.#', '#.#', '.##', '..#', '##.'], z: ['...', '###', '..#', '.#.', '###'],
    0: ['###', '#.#', '#.#', '#.#', '###'], 1: ['.#.', '##.', '.#.', '.#.', '###'], 2: ['##.', '..#', '.#.', '#..', '###'],
    3: ['##.', '..#', '.#.', '..#', '##.'], 4: ['#.#', '#.#', '###', '..#', '..#'], 5: ['###', '#..', '##.', '..#', '##.'],
    6: ['.##', '#..', '###', '#.#', '###'], 7: ['###', '..#', '.#.', '.#.', '.#.'], 8: ['###', '#.#', '###', '#.#', '###'],
    9: ['###', '#.#', '###', '..#', '##.'],
    '.': ['...', '...', '...', '...', '.#.'], ',': ['...', '...', '...', '...', '.#.', '#..'], ':': ['...', '.#.', '...', '.#.', '...'],
    ';': ['...', '.#.', '...', '.#.', '#..'], '!': ['.#.', '.#.', '.#.', '...', '.#.'], '?': ['##.', '..#', '.#.', '...', '.#.'],
    '-': ['...', '...', '###', '...', '...'], '+': ['...', '.#.', '###', '.#.', '...'], '=': ['...', '###', '...', '###', '...'],
    '*': ['...', '#.#', '.#.', '#.#', '...'], '/': ['..#', '..#', '.#.', '#..', '#..'], '%': ['#.#', '..#', '.#.', '#..', '#.#'],
    '(': ['.#.', '#..', '#..', '#..', '.#.'], ')': ['.#.', '..#', '..#', '..#', '.#.'], '[': ['##.', '#..', '#..', '#..', '##.'],
    ']': ['.##', '..#', '..#', '..#', '.##'], '<': ['..#', '.#.', '#..', '.#.', '..#'], '>': ['#..', '.#.', '..#', '.#.', '#..'],
    "'": ['.#.', '.#.', '...', '...', '...'], '"': ['#.#', '#.#', '...', '...', '...'], '_': ['...', '...', '...', '...', '###'],
    '#': ['.#.#', '####', '.#.#', '####', '.#.#'], '&': ['.#.', '#.#', '.#.', '#.#', '.##'], '|': ['.#.', '.#.', '.#.', '.#.', '.#.'],
    '~': ['...', '...', '.#.', '#.#', '...'], '@': ['.#.', '#.#', '###', '#..', '.##'], '^': ['.#.', '#.#', '...', '...', '...'],
    '❯': ['#..', '.#.', '..#', '.#.', '#..'], '⏺': ['...', '###', '###', '###', '...'], '⎿': ['#..', '#..', '###', '...', '...'],
    '✓': ['...', '..#', '..#', '#.#', '.#.'], '·': ['...', '...', '.#.', '...', '...'], '…': ['...', '...', '...', '...', '#.#'],
    '↓': ['.#.', '.#.', '.#.', '###', '.#.'], '↑': ['.#.', '###', '.#.', '.#.', '.#.'], '█': ['###', '###', '###', '###', '###', '###'],
  };
  const CELL = 4, LINE = 7;
  function mono(s, x, y, c) {
    x = Math.round(x); y = Math.round(y);
    g.fillStyle = c;
    let cx = x;
    for (const ch of s) {
      const gl = M[ch];
      if (gl) { for (let j = 0; j < gl.length; j++) { const r = gl[j]; for (let i = 0; i < r.length; i++) if (r[i] === '#') g.fillRect(cx + i, y + j, 1, 1); } }
      else if (ch === '─') g.fillRect(cx, y + 2, CELL, 1);
      cx += gl && gl[0].length > 3 ? CELL + 1 : CELL;
    }
    return cx;
  }
  const monoW = (s) => [...s].reduce((w, ch) => w + (M[ch] && M[ch][0].length > 3 ? CELL + 1 : CELL), 0);
  // Colored runs on one line: [[text, color], ...]. Returns the x after the last run.
  function runs(x, y, list) { for (const [s, c] of list) x = mono(s, x, y, c); return x; }

  // Claude Code's spinner glyphs (· ✢ ✳ ✶ ✻ ✽ and back), 5x5.
  const SPIN = [
    ['.....', '.....', '..#..', '.....', '.....'],
    ['.....', '..#..', '.###.', '..#..', '.....'],
    ['..#..', '.#.#.', '#.#.#', '.#.#.', '..#..'],
    ['..#..', '#.#.#', '.###.', '#.#.#', '..#..'],
    ['#.#.#', '.###.', '##.##', '.###.', '#.#.#'],
    ['.#.#.', '#####', '.#.#.', '#####', '.#.#.'],
  ];
  const SPIN_SEQ = [0, 1, 2, 3, 4, 5, 5, 4, 3, 2, 1, 0];
  function spinner(i, x, y, c) {
    const f = SPIN[SPIN_SEQ[((i % SPIN_SEQ.length) + SPIN_SEQ.length) % SPIN_SEQ.length]];
    g.fillStyle = c;
    for (let j = 0; j < 5; j++) for (let k = 0; k < 5; k++) if (f[j][k] === '#') g.fillRect(x + k, y + j, 1, 1);
  }

  // ------------------------------------------------------------ layers and camera
  const pool = {};
  function layer(name) {
    if (!pool[name]) {
      const c = document.createElement('canvas');
      c.width = W; c.height = H;
      const x = c.getContext('2d');
      x.imageSmoothingEnabled = false;
      pool[name] = { c, x };
    }
    return pool[name];
  }
  // Draw fn() into the main buffer at 1x, keep a copy on a named layer, return it.
  function grab(name, fn) {
    g.save(); PX.clear(P.ink); fn(); g.restore();
    const L = layer(name);
    L.x.clearRect(0, 0, W, H);
    L.x.drawImage(g.canvas, 0, 0);
    return L.c;
  }
  // Show a world layer at integer zoom k with world point (fx, fy) at the screen
  // center (clamped to the world), plus a screen-px offset (shake, drift).
  function view(c, k, fx, fy, ox = 0, oy = 0) {
    k = Math.max(1, Math.round(k));
    const hw = W / (2 * k), hh = H / (2 * k);
    fx = PX.clamp(fx, hw, W - hw); fy = PX.clamp(fy, hh, H - hh);
    PX.clear(P.ink);
    g.drawImage(c, Math.round(W / 2 - fx * k + ox), Math.round(H / 2 - fy * k + oy), W * k, H * k);
  }
  // Screen <- world mapping for the same camera (for overlays drawn at screen scale).
  function toScreen(k, fx, fy, x, y, ox = 0, oy = 0) {
    const hw = W / (2 * k), hh = H / (2 * k);
    fx = PX.clamp(fx, hw, W - hw); fy = PX.clamp(fy, hh, H - hh);
    return [Math.round(W / 2 - fx * k + ox) + x * k, Math.round(H / 2 - fy * k + oy) + y * k];
  }
  // Shift the finished buffer (whole-frame shake); ink shows at the edges.
  function shakeFrame(dx, dy) {
    if (!dx && !dy) return;
    const L = layer('tmx-shk');
    L.x.clearRect(0, 0, W, H);
    L.x.drawImage(g.canvas, 0, 0);
    PX.clear(P.ink);
    g.drawImage(L.c, Math.round(dx), Math.round(dy));
  }
  // Blow the finished buffer up k times around screen point (cx, cy) (a punch-in on a drawn frame).
  function punch(k, cx, cy) {
    if (k <= 1) return;
    const L = layer('tmx-pun');
    L.x.clearRect(0, 0, W, H);
    L.x.drawImage(g.canvas, 0, 0);
    PX.clear(P.ink);
    g.drawImage(L.c, Math.round(W / 2 - cx * k), Math.round(H / 2 - cy * k), W * k, H * k);
  }
  // Radial speed lines around (cx, cy): dir 1 streams outward (push in), -1 inward (pull back).
  // a in 0..1 is the density; lines thicken to 2px toward the frame edge.
  function speedLines(t, cx, cy, a, seed = 5, dir = 1, cols) {
    if (a <= 0) return;
    const n = Math.round(160 * a), R = 250;
    const C = cols || [P.bone, P.fog, P.haze];
    for (let i = 0; i < n; i++) {
      const ang = PX.hash(i, seed) * Math.PI * 2, ca = Math.cos(ang), sa = Math.sin(ang);
      const sp = 560 + PX.hash(i, seed + 1) * 560;
      const r0 = 34 + ((((PX.hash(i, seed + 2) * R + dir * t * sp) % R) + R) % R);
      const len = (12 + PX.hash(i, seed + 3) * 40) * (0.5 + a * 0.8) * (0.6 + r0 / 160);
      const h = PX.hash(i, seed + 4);
      const c = h < 0.4 ? C[0] : h < 0.75 ? C[1] : C[2];
      const x0 = cx + ca * r0, y0 = cy + sa * r0, x1 = cx + ca * (r0 + len), y1 = cy + sa * (r0 + len);
      PX.line(x0, y0, x1, y1, c);
      if (r0 > 70) PX.line(x0 - sa, y0 + ca, x1 - sa, y1 + ca, c);
    }
  }
  // Clip drawing to a rect while fn runs.
  function clip(x, y, w, h, fn) {
    g.save(); g.beginPath(); g.rect(x, y, w, h); g.clip(); fn(); g.restore();
  }

  // ------------------------------------------------------------ inactive-pane dim
  // One palette step darker for palette colors (tmux's inactive-pane style), 0.6x for
  // the game's own in-between shades. Works on the finished pixels of a rect.
  const DIM = {
    ink: 'ink', night: 'ink', dusk: 'night', slate: 'dusk', mauve: 'slate', haze: 'mauve', fog: 'haze', bone: 'fog',
    maroon: 'night', blood: 'maroon', red: 'blood', ember: 'red', amber: 'clay', gold: 'amber', cream: 'sand',
    deep: 'ink', moss: 'deep', leaf: 'moss', lime: 'leaf', sprout: 'lime',
    navy: 'night', blue: 'navy', sky: 'blue', cyan: 'sky', ice: 'cyan',
    plum: 'night', grape: 'plum', pink: 'grape', blush: 'pink', umber: 'maroon', clay: 'umber', sand: 'clay',
  };
  const rgbOf = (hx) => [parseInt(hx.slice(1, 3), 16), parseInt(hx.slice(3, 5), 16), parseInt(hx.slice(5, 7), 16)];
  const DIMMAP = new Map();
  for (const k in DIM) { const [r, gg, b] = rgbOf(P[k]); DIMMAP.set((r << 16) | (gg << 8) | b, rgbOf(P[DIM[k]])); }
  function dimRect(x, y, w, h, steps = 1) {
    x = Math.max(0, Math.round(x)); y = Math.max(0, Math.round(y));
    w = Math.min(W - x, Math.round(w)); h = Math.min(H - y, Math.round(h));
    if (w <= 0 || h <= 0) return;
    const im = g.getImageData(x, y, w, h), d = im.data;
    for (let i = 0; i < d.length; i += 4) {
      for (let s = 0; s < steps; s++) {
        const key = (d[i] << 16) | (d[i + 1] << 8) | d[i + 2];
        const m = DIMMAP.get(key);
        if (m) { d[i] = m[0]; d[i + 1] = m[1]; d[i + 2] = m[2]; }
        else { d[i] = Math.round(d[i] * 0.6); d[i + 1] = Math.round(d[i + 1] * 0.6); d[i + 2] = Math.round(d[i + 2] * 0.62); }
      }
    }
    g.putImageData(im, x, y);
  }

  // ------------------------------------------------------------ tmux screen
  // Left: Claude Code. Right: a btop strip on top of the trex pane, which is
  // exactly the pane footage size so the game draws at 1x.
  function layout() {
    const f = PX.footageIndex['pane-title'] || { w: 224, h: 126 };
    const top = 9, bot = 172;
    const gx = W - f.w, gy = bot - f.h;
    return { top, bot, gx, gy, gw: f.w, gh: f.h, bx: gx - 2, hy: gy - 2, aw: gx - 3 };
  }
  function titleBar(title) {
    PX.rect(0, 0, W, 9, P.night);
    PX.rect(0, 8, W, 1, P.ink);
    [P.red, P.amber, P.leaf].forEach((c, i) => PX.rect(5 + i * 6, 3, 3, 3, c));
    mono(title, Math.round(W / 2 - monoW(title) / 2), 2, P.haze);
    for (const [x, y] of [[0, 0], [1, 0], [0, 1], [W - 1, 0], [W - 2, 0], [W - 1, 1]]) PX.px(x, y, P.ink);
  }
  // tmux colors the border segments that touch the active pane. o.flash: bone instead
  // of lime; o.wide: 2px (the moment focus flips).
  function borders(Lo, focus, o = {}) {
    const on = o.flash ? P.bone : P.lime, off = P.slate;
    const agentOn = focus === 'agent', gameOn = focus === 'trex';
    const w = o.wide ? 2 : 1, ox = o.wide ? (agentOn ? -1 : 0) : 0, oy = o.wide ? -1 : 0;
    PX.rect(Lo.bx, Lo.top, 1, Lo.hy - Lo.top, off);
    PX.rect(Lo.bx, Lo.hy, 1, Lo.bot - Lo.hy, off);
    PX.rect(Lo.bx, Lo.hy, W - Lo.bx, 1, off);
    if (agentOn) PX.rect(Lo.bx + ox, Lo.top, w, Lo.bot - Lo.top, on);
    if (gameOn) { PX.rect(Lo.bx, Lo.hy, w, Lo.bot - Lo.hy, on); PX.rect(Lo.bx, Lo.hy + oy, W - Lo.bx, w, on); }
  }
  // Sparks racing along the newly active border, age seconds after the flip.
  function borderSparks(Lo, focus, age) {
    if (age < 0 || age > 0.4) return;
    const pts = [];
    if (focus === 'agent') { pts.push([Lo.bx, Lo.hy, 0, -1, Lo.hy - Lo.top], [Lo.bx, Lo.hy, 0, 1, Lo.bot - Lo.hy]); }
    else { pts.push([Lo.bx, Lo.hy, 1, 0, W - Lo.bx], [Lo.bx, Lo.hy, 0, 1, Lo.bot - Lo.hy]); }
    const u = PX.ease.out(Math.min(1, age / 0.3));
    for (const [x0, y0, dx, dy, len] of pts) {
      const d = Math.round(u * len);
      const hx = x0 + dx * d, hy = y0 + dy * d;
      // Bright head, a short hot trail, and sparks thrown off it.
      for (let s = 0; s < 10; s++) { const q = d - s; if (q < 0) break; PX.rect(x0 + dx * q - 1, y0 + dy * q - 1, 3, 3, s < 3 ? P.bone : s < 6 ? P.cream : P.gold); }
      if (age < 0.32) {
        PX.rect(hx - 2, hy - 2, 5, 5, P.bone);
        PX.burst(age, hx, hy, { n: 5, seed: 300 + Math.round(dx * 3 + dy * 7), speed: 50, gravity: 120, life: 0.18, colors: [P.cream, P.gold, P.bone], size: 2, up: 10 });
      }
    }
    PX.burst(age, Lo.bx, Lo.hy, { n: 14, seed: 311, speed: 90, gravity: 200, life: 0.3, colors: [P.bone, P.cream, P.gold], size: 2, up: 30 });
  }
  function statusBar(Lo, focus, flash) {
    const y = Lo.bot;
    PX.rect(0, y, W, H - y, P.moss);
    mono('[api]', 2, y + 1, P.ink);
    mono('0:zsh', 26, y + 1, P.deep);
    mono('1:dev', 50, y + 1, P.ink);
    const clock = '22:47';
    const cx = W - 2 - monoW(clock);
    mono(clock, cx, y + 1, P.deep);
    const tag = focus === 'agent' ? 'claude' : 'trex';
    const tw = monoW(tag) + 3, tx = cx - 5 - tw;
    PX.rect(tx, y + 1, tw, 6, flash ? P.bone : P.lime);
    mono(tag, tx + 2, y + 1, P.ink);
  }
  // btop's cpu box: a braille graph scrolling at its own update rate, and a process list.
  function btop(Lo, t, busy) {
    const x0 = Lo.gx + 1, y0 = Lo.top + 2, w = Lo.gw - 3, h = Lo.hy - Lo.top - 4;
    const bc = P.dusk;
    PX.rect(x0 + 2, y0, w - 4, 1, bc); PX.rect(x0 + 2, y0 + h - 1, w - 4, 1, bc);
    PX.rect(x0, y0 + 2, 1, h - 4, bc); PX.rect(x0 + w - 1, y0 + 2, 1, h - 4, bc);
    for (const [a, b] of [[1, 1], [w - 2, 1], [1, h - 2], [w - 2, h - 2]]) PX.px(x0 + a, y0 + b, bc);
    const rate = 5, s0 = Math.floor(t * rate + 1e-6);
    const cpu = (s) => {
      const n = (PX.hash(s, 11) + PX.hash(s - 1, 11) + PX.hash(s + 1, 11)) / 3;
      return PX.clamp(0.12 + n * 0.5 + (busy ? busy(s / rate) : 0) * 0.45, 0, 1);
    };
    const now = cpu(s0);
    PX.rect(x0 + 5, y0 - 2, monoW('cpu') + 2, 5, P.ink);
    mono('cpu', x0 + 6, y0 - 2, P.pink);
    const pct = Math.round(now * 100) + '%';
    PX.rect(x0 + 21, y0 - 2, monoW(pct) + 2, 5, P.ink);
    mono(pct, x0 + 22, y0 - 2, P.haze);
    const gx0 = x0 + 4, gy1 = y0 + h - 4, gh = h - 7, n = 74;
    for (let c = 0; c < n; c++) {
      const v = cpu(s0 - (n - 1 - c));
      const dots = Math.round(v * (gh / 2));
      for (let d = 0; d < dots; d++) {
        const u = d / (gh / 2);
        PX.px(gx0 + c * 2, gy1 - d * 2, u > 0.72 ? P.ember : u > 0.45 ? P.gold : P.lime);
      }
    }
    const lx = x0 + 158;
    const procs = [['claude', 8 + 30 * cpu(s0 - 2)], ['trex', 2 + 2 * PX.hash(s0, 3)], ['zsh', 0]];
    procs.forEach(([name, v], i) => {
      const y = y0 + 4 + i * 8;
      mono(name, lx, y, i === 0 ? P.fog : P.haze);
      const num = v.toFixed(1);
      mono(num, x0 + w - 5 - monoW(num), y, P.mauve);
      PX.rect(lx, y + 6, Math.round(v * 1.3) + 1, 1, i === 0 ? P.ember : P.leaf);
    });
  }
  // Claude Code in the agent pane. st: {lines, rows, input, cursor, placeholder,
  // spin: {i, verb, sub} | null, hint}. Lines are {runs:[[s, c]...], bg}.
  function claude(Lo, st) {
    const w = Lo.aw, x0 = 2;
    const rows = st.rows || 15, y0 = Lo.top + 2;
    const shown = st.lines.slice(Math.max(0, st.lines.length - rows));
    shown.forEach((ln, i) => {
      const y = y0 + i * LINE;
      if (ln.bg) PX.rect(0, y - 1, w, LINE, ln.bg);
      if (ln.runs) runs(x0 + (ln.indent || 0), y, ln.runs);
    });
    const ry = Lo.bot - 21;
    PX.rect(0, ry, w, 1, P.slate);
    PX.rect(0, ry + 11, w, 1, P.slate);
    mono('>', x0, ry + 3, P.bone);
    if (st.input) {
      const ex = mono(st.input, x0 + 8, ry + 3, P.bone);
      if (st.cursor) PX.rect(ex, ry + 2, 3, 6, P.bone);
    } else {
      if (st.cursor) PX.rect(x0 + 8, ry + 2, 3, 6, P.bone);
      if (st.placeholder) mono(st.placeholder, x0 + 12, ry + 3, P.mauve);
    }
    mono(st.hint || '? for shortcuts', x0, ry + 14, P.mauve);
    if (st.spin) {
      const sx = 2, sy = ry - 16;
      spinner(st.spin.i, sx, sy, P.ember);
      const v = st.spin.verb + '…';
      const at = (st.spin.i % (v.length + 6)) - 3;
      [...v].forEach((ch, i) => mono(ch, sx + 7 + i * CELL, sy, i >= at && i < at + 3 ? P.amber : P.ember));
      if (st.spin.sub) mono(st.spin.sub, sx + 7, sy + LINE, P.mauve);
    }
  }
  // The whole screen. o: {title, focus, flash, wide, agent: st, t, busy, game(Lo),
  // dimAgent, dimGame (tmux inactive-pane dim), agentBg(Lo) (under the agent text),
  // gameTop(Lo) (over the game after it dims)}.
  function screen(o) {
    const Lo = layout();
    PX.clear(P.ink);
    titleBar(o.title || '~/api · tmux');
    clip(0, Lo.top, Lo.aw, Lo.bot - Lo.top, () => { if (o.agentBg) o.agentBg(Lo); claude(Lo, o.agent); });
    if (o.dimAgent) dimRect(0, Lo.top, Lo.aw, Lo.bot - Lo.top);
    clip(Lo.gx, Lo.top, Lo.gw, Lo.hy - Lo.top, () => btop(Lo, o.t, o.busy));
    dimRect(Lo.gx, Lo.top, Lo.gw, Lo.hy - Lo.top); // btop never has focus
    clip(Lo.gx, Lo.gy, Lo.gw, Lo.gh, () => { PX.rect(Lo.gx, Lo.gy, Lo.gw, Lo.gh, P.ink); if (o.game) o.game(Lo); });
    if (o.dimGame) dimRect(Lo.gx, Lo.gy, Lo.gw, Lo.gh, o.dimGame === true ? 1 : o.dimGame);
    if (o.gameTop) clip(Lo.gx, Lo.gy, Lo.gw, Lo.gh, () => o.gameTop(Lo));
    borders(Lo, o.focus, { flash: o.flash, wide: o.wide });
    statusBar(Lo, o.focus, o.flash);
    return Lo;
  }

  // ------------------------------------------------------------ Clawd
  // Clawd, Claude Code's mascot, as the site draws it (site/js/main.js): a 12x12
  // orange block with tall slit eyes and stubby arms, headphones with a light that
  // pulses on the beat, sitting at a keyboard. Art box 22x19, drawn at k px per art
  // px with its keyboard's bottom-center at (bx, by).
  // st: arms [l, r] of 'rest' | 'hover' | 'key' | 'up' | 'wave0' | 'wave1' | 'wave2' | 'slam';
  // eyes 'open' | 'blink' | 'half' | 'happy' | 'wide' | 'shut'; look [dx, dy]; lean -1..1
  // (-1 sits back); nod 0..1; squash -1..1; hop (art px); phones (default on); beat
  // (headphone light); sweat; keyboard (default on).
  function clawd(bx, by, st = {}, k = 2) {
    const map = new Map();
    const put = (x, y, c) => map.set(x + ',' + y, [x, y, c]);
    const rect = (x, y, w, h, c) => { for (let j = 0; j < h; j++) for (let i = 0; i < w; i++) put(x + i, y + j, c); };
    const kb = st.keyboard !== false;
    const lean = st.lean || 0, nod = st.nod || 0, hop = st.hop || 0, sq = st.squash || 0;
    const bodyBot = 14 + (lean < 0 ? 0 : 1) - hop;
    const headTop = 4 + lean + nod - hop + (sq > 0 ? 1 : sq < 0 ? -1 : 0);
    const bw = sq > 0 ? 14 : 12, L = 11 - bw / 2, R = L + bw - 1;
    const O = P.ember;
    // Legs show when there is no keyboard or Clawd hops off it.
    if (!kb || hop > 0) for (const lx of [L + 1, L + 3, R - 3, R - 1]) rect(lx, bodyBot + 1, 1, kb ? Math.min(3, hop + 1) : 3, O);
    rect(L, headTop, bw, bodyBot - headTop + 1, O);
    rect(L, headTop, bw - 1, 1, P.amber);
    rect(L + 1, bodyBot, bw - 1, 1, P.red);
    rect(R, headTop + 1, 1, bodyBot - headTop, P.red);
    // Eyes: tall slits, 2 in from each side.
    const [lx, ly] = st.look || [0, 0];
    const eyes = st.eyes || 'open';
    const eb = headTop + 4 + ly; // bottom row of the eyes
    for (const ex0 of [L + 2, R - 2]) {
      const ex = ex0 + lx;
      if (eyes === 'open') rect(ex, eb - 2, 1, 3, P.ink);
      else if (eyes === 'half') rect(ex, eb - 1, 1, 2, P.ink);
      else if (eyes === 'blink' || eyes === 'shut') rect(ex - (ex0 === L + 2 ? 0 : 0), eb, 1, 1, P.ink);
      else if (eyes === 'wide') rect(ex0 === L + 2 ? ex : ex - 1, eb - 2, 2, 3, P.ink);
      else if (eyes === 'happy') { put(ex - 1, eb, P.ink); put(ex, eb - 1, P.ink); put(ex + 1, eb, P.ink); }
    }
    // Headphones: a band arcing over the head, cups on the sides, a light on the beat.
    if (st.phones !== false) {
      const hb = headTop - 3;
      rect(L + 2, hb, bw - 4, 1, P.slate); rect(L, hb + 1, 2, 1, P.slate); rect(R - 1, hb + 1, 2, 1, P.slate);
      put(L - 1, hb + 2, P.slate); put(R + 1, hb + 2, P.slate);
      rect(L - 2, headTop, 2, 4, P.night); rect(R + 1, headTop, 2, 4, P.night);
      rect(L - 2, headTop, 1, 4, P.slate); rect(R + 2, headTop, 1, 4, P.slate);
      if (st.beat) { rect(L - 1, headTop + 1, 1, 2, st.beat > 1 ? P.blush : P.pink); rect(R + 1, headTop + 1, 1, 2, st.beat > 1 ? P.blush : P.pink); }
    }
    if (kb) {
      rect(2, 16, 18, 2, P.dusk);
      for (let x = 2; x < 20; x += 2) { put(x + 1, 16, P.mauve); put(x, 17, P.mauve); }
      rect(1, 18, 20, 1, P.slate);
    }
    // Arms.
    const arms = st.arms || ['rest', 'rest'];
    arms.forEach((a, side) => {
      const s = side ? 1 : -1;
      const ax = side ? R + 1 : L - 2; // the 2-wide column hugging the body
      const sh = bodyBot - 4; // shoulder row
      if (a === 'rest') rect(ax, sh, 2, 3, O);
      else if (a === 'hover') rect(ax, 11, 2, 3, O);
      else if (a === 'key' || a === 'slam') {
        rect(ax, a === 'slam' ? 12 : 13, 2, a === 'slam' ? 4 : 3, O);
        if (kb) {
          const kx = side ? 17 : 3;
          put(kx, 16, P.cream); put(kx + 1, 16, P.cream); put(kx + (side ? 1 : 0), 17, P.gold);
          if (a === 'slam') { put(kx - s, 16, P.gold); put(kx + 1 + s, 16, P.gold); }
        }
      } else if (a === 'up' || a === 'wave0' || a === 'wave1' || a === 'wave2') {
        // Shoulder stub, then the arm raised clear of the headphone cup, hand above the head.
        const ox = side ? ax + 2 : ax - 2; // column outside the cup
        rect(ax, sh - 1, 2, 2, O);
        // wave0 swings out, wave2 swings in over the cup; the forearm bends at the elbow.
        const tip = a === 'wave0' ? 2 * s : a === 'wave2' ? -2 * s : 0;
        rect(ox, headTop + 2, 2, sh - headTop - 1, O);
        rect(ox + Math.round(tip / 2), headTop, 2, 2, O);
        rect(ox + tip, headTop - 3, 2, 3, O);
        put(ox + tip + (side ? 1 : 0), headTop - 3, P.amber);
      }
    });
    if (st.sweat) { put(R + 3, headTop - 1, P.cyan); put(R + 3, headTop, P.ice); put(R + 4, headTop, P.cyan); put(R + 3, headTop + 1, P.cyan); }
    const ox = Math.round(bx - 11 * k), oy = Math.round(by - 19 * k);
    mapArt(map, ox, oy, k);
    return { x: ox, y: oy, head: oy + headTop * k, left: ox + L * k, right: ox + (R + 1) * k, cups: [ox + (L - 2) * k, ox + (R + 2) * k], cupY: oy + (headTop + 1) * k };
  }

  // ------------------------------------------------------------ Rex
  // Poses in the game's letters (g leaf, l lime, w bone, k ink, f moss, x sand, h pink).
  // idle0 is the game's own frame (src/meta/sprites.rs); the rest are drawn for acting.
  const REX = {
    idle0: ['.......ggggg..', '......glllllg.', '......gllwklgg', '......glllgggg', '......gggggggg', '......gkwkwkk.', '...g..gffffff.',
      '..gg.gggxxg...', '.ggggggggxxgg.', 'gg..gfgggxx...', '....ffgggg....', '....ff..gg....', '...fff..ggg...'],
    idle1: ['.......ggggg..', '......glllllg.', '......glllllgg', '......glllgggg', '......gggggggg', '......gkwkwkk.', '...g..gffffff.',
      '..gg.gggxxg...', '.ggggggggxxgg.', 'gg..gfgggxx...', '....ff..gg....', '...fff..ggg...'],
    crouch: ['.......ggggg..', '......glllllg.', '......gllwklgg', '......glllgggg', '......gggggggg', '......gkwkwkk.', '...g..gffffff.',
      '..gg.gggxxg...', '.ggggggggxxgg.', 'gg.fgfgggxxgg.', '..fff...ggg...'],
    air: ['.......ggggg..', '......glllllg.', '......gllwklgg', '......glllgggg', '......gggggggg', '......gkwkwkk.', '...g..gffffff.',
      '..gg.gggxxg...', '.ggggggggxxgg.', 'gg..gfgggxx...', '....ffggggg...', '...ff....gg...', '..ff......gg..'],
    // Tail flick: the tail tip snaps up (a reaction to a loud key).
    flick: ['.......ggggg..', '......glllllg.', '......gllwklgg', '......glllgggg', '......gggggggg', '..g...gkwkwkk.', '..g...gffffff.',
      '..g..gggxxg...', '.ggggggggxxgg.', '.g..gfgggxx...', '....ffgggg....', '....ff..gg....', '...fff..ggg...'],
    yawn0: ['........ggggg.', '.......glllllg', '.......gllkklg', '.......gllllgg', '......ggggggg.', '......gkkkkkk.', '...g..gkhhhk..',
      '..gg.gggggg...', '.ggggggggxxgg.', 'gg..gfgggxx...', '....ffgggg....', '....ff..gg....', '...fff..ggg...'],
    yawn: ['........ggggg...', '.......glllllg..', '.......gllkklgg.', '.......glllgggg.', '......ggggggggg.', '......gkkkkkkkk.', '......gkhhhhhk..',
      '...g..gkkkkkkk..', '..gg.gwgggggwg..', '.ggggggggxxg.g..', 'gg..gfgggxx.g...', '....ffgggg......', '....ff..gg......', '...fff..ggg.....'],
    stretch: ['......ggggg..', '.....glllllg.', '.....gllkklgg', '.....glllgggg', '.....gggggggg', '.....gkkkkkg.', '..g..gffffg.g',
      '..g.gggxxgg.g', '.gggggggxxg..', 'gg..gfgggxx..', '....ffgggg...', '....ff..gg...', '....f....g...', '...ff....gg..'],
    roar: ['.......ggggg....', '......glllllg...', '......glkkllgg..', '......glllggggg.', '......gwgwgwgwg.', '......gkkkkkkk..', '......gkhhhhk...',
      '...g..gkkkkkkk..', '..gg.ggwgwgwgwg.', '.gggggggggggg...', 'gg..gfgggxxgg...', '....ffgggg......', '....ff..gg......', '...fff..ggg.....'],
    // Head thrown back, jaw shut, chest puffed: the inhale before the roar.
    roarBack: ['...........gg...', '.........gglgg..', '........gllllg..', '.......glllllgg.', '.......glkkllgg.', '......gglllgggg.', '......ggggggkgg.',
      '...g..gggfffgg..', '..gg.gggxxxg....', '.ggggggggxxxg...', 'gg..gfgggxxg....', '....ffgggg......', '....ff..gg......', '...fff..ggg.....'],
    // Leaning into it, jaw wide open (the breath frame).
    roarWide: ['........ggggg...', '.......glllllg..', '.......gllkklgg.', '.......glllggggg', '......ggwgwgwgwg', '......gkkkkkkkk.', '......gkhhhhhk..',
      '...g..gkhhhhhk..', '..gg.ggkkkkkkkk.', '.ggggggwgwgwgwg.', 'gg..gfgggxxgg...', '....ffgggg......', '....ff..gg......', '...fff..ggg.....'],
    // Mid-turn toward the camera: the eye widens, the far brow shows.
    turn: ['.......ggggg...', '......gllllllg.', '......gllwwllgg', '......gllwkllggg', '......glllllgggg', '......ggggggggg', '......gkwkwkwkk',
      '...g..gffffff..', '..gg.gggxxg....', '.ggggggggxxgg..', 'gg..gfgggxx....', '....ffgggg.....', '....ff..gg.....', '...fff..ggg....'],
  };
  // 3/4 view, head turned to the camera: the snout still points right, the near eye
  // looks straight out, the far eye peeks over the bridge, tan belly, tiny arms.
  const Q_HEAD = [
    '......ggggg......',
    '.....gllllllg....',
    '....glllllllgg...',
    '....gwwwllllgwg..',
    '....gwkwlllllkggg',
    '....glllllgggggg.',
    '....ggggggggggggg',
    '....gkwkwkwkwkwk.',
    '.....gffffffffff.',
  ];
  const Q_BODY = [
    '......gxxxxgg....',
    '....lggxxxxgg.g..',
    '.....gfxxxxgggg..',
    '.gggffggxxgg.....',
    'gg..ff....gg.....',
    '...fff....ggg....',
  ];
  const Q_EYES = {
    open: [],
    shut: [[5, 3, 'l'], [6, 3, 'l'], [7, 3, 'l'], [5, 4, 'k'], [6, 4, 'k'], [7, 4, 'k'], [13, 3, 'g'], [13, 4, 'k']],
    half: [[5, 3, 'k'], [6, 3, 'k'], [7, 3, 'k'], [13, 3, 'k']],
    wink: [[5, 3, 'l'], [6, 3, 'l'], [7, 3, 'l'], [5, 4, 'k'], [6, 4, 'k'], [7, 4, 'k']],
    side: [[6, 4, 'w'], [7, 4, 'k']],
  };
  const GRIN = [[5, 7, 'w'], [7, 7, 'w'], [9, 7, 'w'], [11, 7, 'w'], [13, 7, 'w'], [15, 7, 'k'], [6, 8, 'k'], [7, 8, 'h'], [8, 8, 'h'], [9, 8, 'h'], [10, 8, 'h'], [11, 8, 'h'], [12, 8, 'h'], [13, 8, 'h'], [14, 8, 'k']];
  // Near arm raised for the wave: 3 positions and a cocked anticipation. Rows 0-2 sit
  // above the head, row 12 joins the chest; the head leans 2 columns away from it.
  const Q_ARM = {
    dip: ['.......', '.......', '.......', '.......', '.......', '.......', '.......', '.......', '.......', '.......', '.......', '...wgg.', '....ggg'],
    A: ['w.w....', 'g.g....', 'ggg....', 'ggg....', '.gg....', '.ggg...', '..gg...', '..gg...', '..gg...', '..ggf..', '...ggf.', '....gg.', '.....gg'],
    B: ['.w.w...', '.g.g...', '.ggg...', '.ggg...', '..gg...', '..gg...', '..gg...', '..gg...', '..gg...', '..ggf..', '...ggf.', '....gg.', '.....gg'],
    C: ['...w.w.', '...g.g.', '...ggg.', '...ggg.', '...gg..', '..ggg..', '..gg...', '..gg...', '..gg...', '..ggf..', '...ggf.', '....gg.', '.....gg'],
  };
  function qPose(o = {}) {
    let head = edit(Q_HEAD, Q_EYES[o.eyes || 'open'] || []);
    if (o.grin) head = edit(head, GRIN);
    if (!o.arm) return overlay(['..................'], [...head, ...Q_BODY], 0, 0);
    const lean = o.arm === 'dip' ? 1 : 2;
    const body = o.arm === 'dip' ? Q_BODY.filter((_, j) => j !== 4) : Q_BODY;
    const base = overlay(['...................', '', ''], [...head.map((r) => '.'.repeat(lean) + r), ...body], 0, 3);
    return overlay(base, Q_ARM[o.arm], 0, 0);
  }
  // Peeking around the pane border: the body stays home, the neck stretches across and
  // the head hangs over the agent pane. Faces right here and is drawn flipped; the
  // claws on the border line are drawn separately, in front of it.
  const PEEK_BODY = ['...g..ggg.', '..gg.gggg.', '.gggggggxg', 'gg..gfggxx', '....ffgggg', '....ff..gg', '...fff..ggg'];
  const PEEK_BODY_UP = ['.g....ggg.', '.g...gggg.', '.gg.gggxxg', '..gggfggxx', '....fggggg', '....f...gg', '....f....g', '...ff....gg'];
  const PEEK_HEAD = {
    fwd: ['...ggggg...', '..glllllg..', '..gllwklgg.', '..glllgggg.', '.gggggggggg', 'ggkwkwkwkk.', '..ffffff...'],
    down: ['...ggggg...', '..glllllg..', '..gllwwlgg.', '..gllwkggg.', '.gggggggggg', 'ggkwkwkwkk.', '..ffffff...'],
    shut: ['...ggggg...', '..glllllg..', '..gllkklgg.', '..glllgggg.', '.gggggggggg', 'ggkwkwkwkk.', '..ffffff...'],
    yawn: ['..ggggg...', '.glllllg..', '.gllkklgg.', '.glllgggg.', 'ggggggggg.', 'gkkkkkkkk.', 'gkhhhhhk..', 'gkkkkkkk..'],
  };
  function peekPose(head = 'fwd', bob = 0, up = false) {
    const body = up ? PEEK_BODY_UP : PEEK_BODY;
    const base = overlay(['..................'], body, 0, 6);
    return overlay(base, PEEK_HEAD[head], 7, (head === 'yawn' ? -1 : 0) + bob);
  }
  // Eye edits on the side-view poses (art coords before the outline).
  const EYES = {
    fwd: [], back: [[9, 2, 'k'], [10, 2, 'w']],
    down: [[9, 1, 'w'], [10, 1, 'w'], [9, 2, 'w'], [10, 2, 'k']],
    up: [[9, 1, 'w'], [10, 1, 'k'], [9, 2, 'w'], [10, 2, 'w']],
    wide: [[8, 1, 'w'], [9, 1, 'w'], [10, 1, 'w'], [8, 2, 'w'], [9, 2, 'k'], [10, 2, 'w']],
    shut: [[9, 2, 'k'], [10, 2, 'k']],
    half: [[9, 1, 'l'], [9, 2, 'k'], [10, 2, 'k']],
    happy: [[9, 2, 'k'], [10, 1, 'k'], [11, 2, 'k']],
  };
  const SIDE = new Set(['idle0', 'crouch', 'air', 'flick']);
  // Where each pose stands: the art column between its feet.
  const AX = { q: 8, peek: 7 };
  // rex(pose, bx, by, o): pose is a REX name, 'q' (3/4; o.eyes, o.grin, o.arm) or
  // 'peek' (o.head, o.bob, o.up). o.eyes (side poses), o.k (default 2), o.flip, o.flash.
  function rex(pose, bx, by, o = {}) {
    let rows;
    if (pose === 'q') rows = qPose(o);
    else if (pose === 'peek') rows = peekPose(o.head, o.bob || 0, o.up);
    else {
      rows = REX[pose] || REX.idle0;
      if (o.eyes && EYES[o.eyes] && SIDE.has(pose)) rows = edit(rows, EYES[o.eyes]);
    }
    if (o.edits) rows = edit(rows, o.edits);
    return art(rows, bx, by, Object.assign({ k: 2, ax: AX[pose] != null ? AX[pose] : 7 }, o));
  }
  const BANG = ['yy', 'yy', 'yy', 'yy', '..', 'yy'];

  // ------------------------------------------------------------ backdrops
  // Solid wedges turning around (cx, cy); nothing inside radius r0. Rasterized per
  // scanline so every edge stays a palette pixel.
  function sunburst(cx, cy, a0, n, c, r0 = 0, y0 = 0, y1 = H) {
    g.fillStyle = c;
    const seg = (Math.PI * 2) / n, r2 = r0 * r0;
    for (let y = y0; y < y1; y++) {
      let run = -1;
      for (let x = 0; x <= W; x++) {
        let on = false;
        if (x < W) {
          const dx = x - cx, dy = (y - cy) * 1.3;
          if (dx * dx + dy * dy >= r2) {
            const ang = Math.atan2(dy, dx) - a0;
            on = Math.floor((((ang % (2 * Math.PI)) + 2 * Math.PI) % (2 * Math.PI)) / seg) % 2 === 0;
          }
        }
        if (on && run < 0) run = x;
        else if (!on && run >= 0) { g.fillRect(run, y, x - run, 1); run = -1; }
      }
    }
  }
  // Solid ink outside an ellipse, with one 50% dithered band at its rim (vignette / iris).
  function vignette(r0, band = 0.08, cx = W / 2, cy = H / 2, rx = W / 2, ry = H / 2) {
    g.fillStyle = P.ink;
    for (let y = 0; y < H; y++) {
      let run = -1;
      for (let x = 0; x <= W; x++) {
        let on = false;
        if (x < W) {
          const dx = (x - cx) / rx, dy = (y - cy) / ry, d = Math.sqrt(dx * dx + dy * dy);
          on = d >= r0 + band || (d >= r0 && ((x + y) & 1) === 0);
        }
        if (on && run < 0) run = x;
        else if (!on && run >= 0) { g.fillRect(run, y, x - run, 1); run = -1; }
      }
    }
  }
  // Solid disc (flat glow) with a 1px ring of `rim` around it.
  function disc(cx, cy, r, c, rim) {
    if (rim) PX.circle(cx, cy, r + 1, rim, true);
    PX.circle(cx, cy, r, c, true);
  }
  // A 4-point impact star with a white core (hit frames).
  function star(cx, cy, r, c, core) {
    cx = Math.round(cx); cy = Math.round(cy);
    for (let i = -r; i <= r; i++) {
      const w = Math.max(0, Math.round((r - Math.abs(i)) / 3));
      PX.rect(cx - w, cy + i, w * 2 + 1, 1, c);
      PX.rect(cx + i, cy - w, 1, w * 2 + 1, c);
    }
    if (core) { const q = Math.max(1, Math.round(r / 4)); PX.rect(cx - q, cy - q, q * 2 + 1, q * 2 + 1, core); }
  }

  // ------------------------------------------------------------ TREX logo
  // The game's logo art (src/ui/sprites.rs), drawn per pixel so letters can move.
  const LOGO = [
    'YYYYYYYYYY..YYYYYYYYY...YYYYYYYYY..YYYY....YYYY.',
    'yyyyyyyyyym.yyyyyyyyyy..yyyyyyyyym.yyyym...yyyym',
    'yyyyyyyyyym.yyyymmyyyym.yyyymmmmmm..yyyy..yyyymm',
    '.mmaaaammmm.aaaam..aaam.aaaam........aaaaaaaamm.',
    '...aaaam....aaaam.aaaam.aaaam.........aaaaaamm..',
    '...aaaam....aaaaaaaaamm.aaaaaaaa.......aaaamm...',
    '...oooom....oooooooomm..oooooooom.....oooooo....',
    '...oooom....oooomoooo...oooommmmm....oooooooo...',
    '...oooom....oooom.oooo..oooom.......oooommoooo..',
    '...rrrrm....rrrrm.rrrrm.rrrrm......rrrrmm..rrrr.',
    '...rrrrm....rrrrm..rrrm.rrrrrrrrr..rrrrm...rrrrm',
    '...ccccm....ccccm..cccm.cccccccccm.ccccm...ccccm',
    '....mmmm.....mmmm...mmm..mmmmmmmmm..mmmm....mmmm',
  ];
  const SPANS = [[0, 11], [12, 23], [24, 34], [35, 48]]; // outlined columns per letter T R E X
  const letterOf = (i) => { for (let n = 0; n < 4; n++) if (i >= SPANS[n][0] && i <= SPANS[n][1]) return n; return 3; };
  // logo(cx, y, o): top-left of the unmoved logo is (cx - w/2, y). o.k (4), o.letter(n) ->
  // {dx, dy, kx, ky, vis, flash}, o.wave(i) -> dy px per art column, o.shine (art x+y),
  // o.depth (extrusion px), o.flash (all letters), o.shadow (extrusion color).
  function logo(cx, y, o = {}) {
    const d = parse(LOGO);
    const k = o.k || 4, depth = o.depth == null ? Math.round(k * 1.5) : o.depth;
    const x0 = Math.round(cx - (d.w * k) / 2), base = y + d.h * k;
    const L = [0, 1, 2, 3].map((n) => Object.assign({ dx: 0, dy: 0, kx: k, ky: k, vis: true }, o.letter ? o.letter(n) || {} : {}));
    const place = (i, j) => {
      const n = letterOf(i), f = L[n];
      if (!f.vis) return null;
      const mid = (SPANS[n][0] + SPANS[n][1] + 1) / 2;
      const lx = x0 + mid * k + f.dx, wv = o.wave ? o.wave(i) : 0;
      return [Math.round(lx + (i - mid) * f.kx), Math.round(base + f.dy + wv - (d.h - j) * f.ky), f.kx, f.ky, f];
    };
    for (const [i, j] of d.px) {
      const q = place(i, j);
      if (!q) continue;
      g.fillStyle = o.shadow || P.maroon; g.fillRect(q[0], q[1] + depth, q[2], q[3]);
      g.fillStyle = P.ink; g.fillRect(q[0], q[1] + depth + q[3], q[2], Math.max(1, Math.round(k / 2)));
    }
    for (const [i, j, c0] of d.px) {
      const q = place(i, j);
      if (!q) continue;
      let c = c0;
      const fl = q[4].flash || o.flash;
      if (fl && c0 !== P.ink) c = fl;
      else if (o.shine != null && c0 !== P.ink && c0 !== P.maroon) { const s = i + j - o.shine; if (s > -4 && s < 0) c = s > -2 ? P.bone : P.cream; }
      g.fillStyle = c; g.fillRect(q[0], q[1], q[2], q[3]);
    }
    return { x: x0, y, w: d.w * k, h: d.h * k, cols: d.w };
  }

  window.TMX = {
    pcol, parse, edit, overlay, art, mapArt, mono, monoW, runs, spinner, CELL, LINE,
    layer, grab, view, toScreen, shakeFrame, punch, speedLines, clip, dimRect,
    layout, titleBar, borders, borderSparks, statusBar, btop, claude, screen,
    clawd, REX, rex, BANG, LOGO, logo, sunburst, vignette, disc, star,
  };
})();
