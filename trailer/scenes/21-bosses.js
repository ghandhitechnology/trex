// 21-bosses: bars 25-28 (45-52.5 s), "5 BOSSES".
// Half-time and heavy. Bar 25 opens on the five silhouettes under a blood moon, their
// eyes igniting one per 16th while the camera pushes in. Then each boss gets a slam: its
// name in contact on the hit frame, a shake, an impact, the sprite at 4x with a
// silhouette-to-color reveal, then its fight framed on the boss. Sandmaw holds drawn (its
// capture never surfaces). The Hive Eye opens last and the camera falls into its pupil,
// with its fight inside, for the hard cut into the climax.
// Uses the RX pixel helpers from 20-roster.js.
(function () {
  const { P, BEAT, W, H } = PX;
  const g = PX.g;
  const T0 = PX.bar(25);
  const T1 = PX.bar(29);
  const bt = (b) => T0 + b * BEAT;
  const clamp = PX.clamp;
  const ST = (a, fps = 12) => PX.step(a, fps);
  const { ellipse, groundRing, glint, glow, ridge, shakeBuf } = RX;

  const BANDS = {
    blood: [P.bone, P.blush, P.red, P.red, P.red, P.blood, P.blood, P.maroon, P.maroon],
    sand: [P.cream, P.cream, P.sand, P.sand, P.amber, P.clay, P.clay, P.umber, P.umber],
    // Venom and ice stay bright to the bottom row so the words hold against dark skies.
    venom: [P.sprout, P.sprout, P.lime, P.lime, P.lime, P.leaf, P.leaf, P.leaf, P.moss],
    ice: [P.ice, P.ice, P.cyan, P.cyan, P.cyan, P.sky, P.sky, P.sky, P.blue],
  };

  // --- shot list (beats from bar 25) ------------------------------------------
  // at: slam beat, clip: first gameplay beat, end: next shot. cuts: one footage framing per
  // clip beat: source in-point (s) at the clip beat's start, integer zoom k around (cx, cy).
  const BOSSES = [
    { id: 'mire-queen', name: 'MIRE QUEEN', at: 2, clip: 3, end: 4, title: { k: 2, y: 12, bands: BANDS.venom, edge: P.deep, sparks: [P.sprout, P.lime, P.bone] },
      cuts: [{ in: 1.45, k: 2, cx: 213, cy: 90 }] },
    { id: 'colossus', name: 'BONE COLOSSUS', at: 4, clip: 6, end: 8, title: { k: 2, y: 12, bands: 'bone', sparks: [P.bone, P.fog, P.bone] },
      cuts: [{ in: 0.0, k: 3, cx: 252, cy: 116 }, { in: 1.6, k: 3, cx: 252, cy: 128 }] },
    { id: 'wraith', name: 'STORM WRAITH', at: 8, clip: 10, end: 12, title: { k: 2, y: 12, bands: BANDS.ice, edge: P.navy, sparks: [P.ice, P.cyan, P.bone] },
      cuts: [{ in: 0.9, k: 2, cx: 86, cy: 80 }, { in: 0.9 + BEAT, k: 3, cx: 75, cy: 76 }] },
    { id: 'sandmaw', name: 'SANDMAW', at: 12, clip: 14, end: 14, title: { k: 3, y: 10, bands: BANDS.sand, edge: P.umber, sparks: [P.cream, P.sand, P.bone] },
      cuts: [] },
    { id: 'hive-eye', name: 'HIVE EYE', at: 14, clip: 15, end: 16, title: { k: 3, y: 10, bands: 'dread', sparks: [P.blush, P.pink, P.bone] },
      cuts: [{ k: 2, cx: 117, cy: 116 }] },
  ];
  const PACK_TITLE = { k: 3, y: 8, bands: BANDS.blood, edge: P.maroon, sparks: [P.cream, P.red, P.blush] };

  // --- drawing helpers ---------------------------------------------------------
  // Name slam: in contact on the hit frame (age 0), then a palette shine a beat later.
  function slamTitle(str, age, o) {
    if (age < 0) return;
    const k = o.k || 2, y = o.y;
    const sh = age - 0.3;
    const shine = sh >= 0 && sh < 0.6 ? -60 + sh * 900 : null;
    RX.title(str, 160, y, { k, bands: o.bands || 'dread', edge: o.edge, fx: RX.slamFx(str, k, age, { noFlash: o.noFlash }), shine });
    // Sparks off both ends of the word on impact.
    const tw = PX.titleWidth(str, k) * 1.3;
    const cols = o.sparks || [P.cream, P.gold, P.bone];
    const by = y + 9 * k - 2;
    PX.burst(age, 160 - tw / 2 - 2, by, { n: 10, seed: 11 + str.length, speed: 90, life: 0.35, colors: cols, angle: Math.PI * 1.1, spread: 1.3, up: 20, gravity: 200 });
    PX.burst(age, 160 + tw / 2 + 2, by, { n: 10, seed: 17 + str.length, speed: 90, life: 0.35, colors: cols, angle: -Math.PI * 0.1, spread: 1.3, up: 20, gravity: 200 });
  }

  // Boss sprite anchored bottom-center with a silhouette-to-color reveal.
  // o: k, sx, sy, frame, reveal (0 silhouette .. 1 color), dir ('up'|'down'),
  //    lit(c, i, j) -> true for pixels that stay lit in silhouette, sil (silhouette color),
  //    rim (1px outer rim color), flash (flat color), alpha (Bayer dissolve 0..1).
  function boss(name, bx, by, o = {}) {
    const d = RX.pixels(name, o.frame || 0);
    if (!d) return;
    const u = o.reveal == null ? 1 : o.reveal;
    const lit = o.lit || (() => false);
    const hole = o.holes || (() => false); // see-through pixels while in silhouette
    const sil = o.sil || P.ink;
    const dis = o.alpha == null ? 1 : o.alpha;
    const gone = (i, j) => dis < 1 && PX.bayer(i, j) >= dis;
    const col = (c, i, j) => {
      if (gone(i, j)) return null;
      if (o.flash) return o.flash;
      if (u >= 1) return c;
      const s = o.dir === 'down' ? j / d.h : (d.h - 1 - j) / d.h;
      if (s * 0.62 + PX.bayer(i, j) * 0.38 < u * 1.02) return c;
      if (hole(c)) return null;
      return lit(c, i, j) ? (o.litColor ? o.litColor(c) : c) : sil;
    };
    const opt = { k: o.k || 4, sx: o.sx || 1, sy: o.sy || 1, flip: o.flip };
    if (o.rim) {
      const rimCol = (c, i, j) => (gone(i, j) || (u < 1 && hole(c)) ? null : o.rim);
      for (const [dx, dy] of [[-1, 0], [1, 0], [0, -1], [0, 1]]) RX.drawSpr(name, o.frame || 0, bx + dx, by + dy, Object.assign({}, opt, { color: rimCol }));
    }
    RX.drawSpr(name, o.frame || 0, bx, by, Object.assign({}, opt, { color: col }));
  }

  // Dithered filled ellipse covering fraction a (dust, mist).
  function ditherEllipse(cx, cy, rx, ry, c, a) {
    if (a <= 0) return;
    g.fillStyle = c;
    for (let y = Math.max(0, Math.round(cy - ry)); y <= Math.min(H - 1, cy + ry); y++) {
      const w = rx * Math.sqrt(Math.max(0, 1 - ((y - cy) * (y - cy)) / (ry * ry)));
      for (let x = Math.max(0, Math.round(cx - w)); x <= Math.min(W - 1, cx + w); x++) if (PX.bayer(x, y) < a) g.fillRect(x, y, 1, 1);
    }
  }
  // Jagged lightning bolt from (x0, y0) down to y1, with a glow and a fork.
  function bolt(x0, y0, y1, seed, core = P.bone, glowC = P.cyan) {
    const pts = [[x0, y0]];
    let x = x0, y = y0, i = 0;
    while (y < y1 && i < 40) {
      y = Math.min(y1, y + 7 + PX.hash(i, seed) * 11);
      x += (PX.hash(i, seed + 1) - 0.5) * 24;
      pts.push([x, y]); i++;
    }
    const seg = (a, b, c, ox = 0) => PX.line(a[0] + ox, a[1], b[0] + ox, b[1], c);
    for (let n = 1; n < pts.length; n++) { seg(pts[n - 1], pts[n], glowC, -1); seg(pts[n - 1], pts[n], glowC, 1); }
    const m = Math.floor(pts.length / 2);
    if (pts[m]) {
      let fx = pts[m][0], fy = pts[m][1];
      for (let n = 0; n < 4; n++) { const nx = fx + 6 + PX.hash(n, seed + 5) * 8, ny = fy + 6 + PX.hash(n, seed + 6) * 8; PX.line(fx, fy, nx, ny, glowC); fx = nx; fy = ny; }
    }
    for (let n = 1; n < pts.length; n++) seg(pts[n - 1], pts[n], core);
    return pts[pts.length - 1];
  }
  // Screen-space impact flash in 30 fps frames: a half dither, then a light one.
  function flashIn(age, c = P.bone) {
    if (age < 0) return;
    const f = Math.floor(age * 30 + 1e-6);
    if (f === 0) PX.dither(0.5, c);
    else if (f === 1) PX.dither(0.2, c);
  }
  // Post pass on the finished buffer: slice glitch.
  const sl = document.createElement('canvas'); sl.width = W; sl.height = H;
  const slg = sl.getContext('2d');
  slg.imageSmoothingEnabled = false;
  function slices(age, seed, amp = 12) {
    if (age < 0 || age > 0.1) return;
    slg.clearRect(0, 0, W, H); slg.drawImage(g.canvas, 0, 0);
    const f = Math.floor(age * 30);
    let y = 0, i = 0;
    while (y < H) {
      const h = 3 + Math.floor(PX.hash(i, seed * 31 + f) * 18);
      if (PX.hash(i, seed * 17 + f) < 0.45) g.drawImage(sl, 0, y, W, h, Math.round((PX.hash(i, seed * 7 + f) - 0.5) * 2 * amp), y, W, h);
      y += h; i++;
    }
  }
  // Floating motes / embers / ash drifting with the wind.
  function motes(t, n, cols, seed, o = {}) {
    for (let i = 0; i < n; i++) {
      const sp = (o.speed || 14) * (0.5 + PX.hash(i, seed) * 0.8);
      const x = Math.round(((PX.hash(i, seed + 1) * 340 + t * (o.wind || -10) + Math.sin(t * 2 + i) * 3) % 340 + 340) % 340) - 10;
      const y = Math.round(((PX.hash(i, seed + 2) * 190 - t * sp) % 190 + 190) % 190) - 5;
      PX.px(x, y, cols[i % cols.length]);
      if (i % 5 === 0) PX.px(x, y + 1, cols[(i + 1) % cols.length]);
    }
  }
  // A boss sprite pixel (sprite coords i, j) in screen space for a sprite drawn at (bx, by, k).
  const at = (name, bx, by, k, i, j) => { const [w, h] = PX.size(name); return [Math.round(bx - (w * k) / 2 + i * k + k / 2), Math.round(by - h * k + j * k + k / 2)]; };

  // Pixels that stay lit on each boss while it is a silhouette (its eyes and glowing bits).
  const inRect = (i, j, x0, y0, x1, y1) => i >= x0 && i <= x1 && j >= y0 && j <= y1;
  const LIT = {
    'mire-queen': (c, i, j) => inRect(i, j, 5, 8, 18, 11) && (c === P.bone || c === P.blush),
    colossus: (c) => c === P.ember || c === P.amber,
    wraith: (c, i, j) => inRect(i, j, 9, 5, 14, 8) && (c === P.ice || c === P.cyan),
    sandmaw: (c, i, j) => inRect(i, j, 7, 6, 14, 10) && (c === P.red || c === P.blood),
    'hive-eye': (c) => c === P.red || c === P.blood,
  };

  // --- bar 25, beats 0-1: "5 BOSSES", the pack under a blood moon ------------------
  // Spaced and staggered so every outline reads: the two flyers high on the flanks, the
  // Colossus in profile against the moon, Mire Queen and Sandmaw low in front.
  // eyes: sprite px of each eye (glow + glint); depth drives the slow push-in.
  const PACK = [
    { id: 'wraith', x: 58, y: 104, k: 3, rim: P.sky, glowC: P.cyan, eyes: [[10, 6], [13, 6]], float: true, order: 2, depth: 0.5 },
    { id: 'hive-eye', x: 262, y: 102, k: 3, rim: P.pink, glowC: P.red, eyes: [[11, 9]], float: true, order: 4, depth: 0.5 },
    { id: 'colossus', x: 160, y: 158, k: 4, rim: P.haze, glowC: P.ember, eyes: [[18, 7]], order: 1, depth: 0.7, holes: (c) => c === P.night },
    { id: 'mire-queen', x: 52, y: 182, k: 3, rim: P.leaf, glowC: P.leaf, eyes: [[7, 9], [16, 9]], order: 0, depth: 1, litColor: (c) => (c === P.bone ? P.sprout : P.lime) },
    { id: 'sandmaw', x: 270, y: 182, k: 3, rim: P.clay, glowC: P.red, eyes: [[10, 8]], order: 3, depth: 1 },
  ];
  const IGNITE = (i) => (i + 2) * (BEAT / 4); // eyes light up on sixteenths
  const packPos = (b, age, t) => {
    // Slow integer push-in: nearer layers spread from the center and sink more.
    const u = age / (2 * BEAT);
    const bob = b.float ? Math.round(Math.sin(ST(t, 12) * 4 + b.order * 2) * 2) : 0;
    return [Math.round(b.x + Math.sign(b.x - 160) * u * 10 * b.depth), Math.round(b.y + u * 8 * b.depth) + bob];
  };
  function shotPack(t, age) {
    // Lightning flicker in the sky whenever an eye ignites.
    const flick = PACK.some((b) => { const a = age - IGNITE(b.order); return a >= 0 && a < 1 / 30; });
    RX.sky(flick
      ? [[0, P.night], [36, P.dusk], [76, P.slate], [112, P.blood], [140, P.blood]]
      : [[0, P.ink], [36, P.night], [76, P.dusk], [112, P.maroon], [140, P.maroon]]);
    const mx = 160, my = 86 - Math.round(age * 6);
    glow(mx, my, [[62, P.maroon], [55, P.blood]]);
    PX.circle(mx, my, 50, P.blood, true);
    PX.circle(mx + 3, my - 3, 45, P.red, true);
    PX.dither(0.4, P.blood, mx - 50, my - 50, 26, 100);
    for (const [cx, cy, r] of [[-18, -22, 7], [14, 10, 10], [24, -24, 5], [-26, 16, 6], [2, -8, 3]]) PX.circle(mx + cx, my + cy, r, P.blood, true);
    // Moonlit cloud banks sliding behind the flyers.
    for (let c = 0; c < 10; c++) {
      const w = 40 + Math.floor(PX.hash(c, 41) * 70);
      const x = Math.round(((PX.hash(c, 42) * 460 + age * (14 + c * 6)) % 460 + 460) % 460) - 90;
      const y = 46 + Math.floor(PX.hash(c, 43) * 60);
      const col = flick ? P.slate : c % 2 ? P.dusk : P.maroon;
      PX.rect(x, y, w, 3, col);
      PX.rect(x + 10, y - 2, w - 24, 2, col);
      PX.rect(x + 4, y + 3, w - 8, 1, c % 2 ? P.night : P.blood);
    }
    // Distant bolt on each ignition.
    PACK.forEach((b) => {
      const a = age - IGNITE(b.order);
      if (a >= 0 && a < 0.07) bolt(30 + PX.hash(b.order, 61) * 260, 0, 60 + PX.hash(b.order, 62) * 40, 70 + b.order, P.fog, P.mauve);
    });
    ridge((x) => 150 - 8 * Math.abs(Math.sin(x * 0.05)) - 5 * Math.abs(Math.sin(x * 0.13 + 1)), P.night, P.maroon);
    const idle = Math.floor(ST(t, 4)) % 2;
    PACK.forEach((b, i) => {
      const ea = age - IGNITE(b.order);
      const on = ea >= 0;
      const [x, y] = packPos(b, age, t);
      const frame = b.id === 'hive-eye' ? (on ? 0 : 5) : b.id === 'wraith' ? Math.floor(ST(t, 8)) % 3 : idle;
      boss(b.id, x, y, { k: b.k, frame, reveal: 0, lit: () => false, sil: P.ink, rim: b.rim, holes: b.holes });
      if (!on) return;
      // Eyes ignite: a glow under the lit pixels, then a glint that settles to a gleam.
      for (const [i2, j2] of b.eyes) {
        const [ex, ey] = at(b.id, x, y, b.k, i2, j2);
        glow(ex, ey, [[ea < 0.1 ? 4 : 3, b.glowC]]);
      }
      RX.drawSpr(b.id, frame, x, y, { k: b.k, color: (c, i2, j2) => (LIT[b.id](c, i2, j2) ? (b.litColor ? b.litColor(c) : c) : null) });
      for (const [i2, j2] of b.eyes) {
        const [ex, ey] = at(b.id, x, y, b.k, i2, j2);
        if (ea < 0.16) glint(ex, ey, ea < 0.05 ? 9 : ea < 0.1 ? 6 : 4, ea < 0.08 ? P.bone : P.cream);
        else if (Math.floor(ST(t, 12) * 12 + i) % 6 < 3) glint(ex, ey, 2, b.id === 'colossus' ? P.gold : P.cream);
      }
    });
    ridge((x) => 170 - 3 * Math.sin(x * 0.04) - 2 * Math.sin(x * 0.17), P.ink, P.night);
    motes(t, 44, [P.ember, P.red, P.amber], 5, { speed: 18, wind: 8 });
    // The downbeat is a lightning strike: everything but the pack goes white.
    const f = Math.floor(age * 30 + 1e-6);
    if (f === 0) {
      PX.clear(P.bone);
      PACK.forEach((b) => { const [x, y] = packPos(b, age, t); boss(b.id, x, y, { k: b.k, frame: b.id === 'hive-eye' ? 5 : 0, reveal: 0, sil: P.ink, holes: b.holes }); });
      ridge((x) => 170 - 3 * Math.sin(x * 0.04) - 2 * Math.sin(x * 0.17), P.ink);
      bolt(112, 0, 150, 7, P.ink, P.fog);
    } else if (f === 1) PX.dither(0.4, P.fog);
    else if (f === 2) PX.dither(0.15, P.fog);
    if (f >= 1 && age < 0.2 && f % 3 !== 2) bolt(112, 0, 150, 7, P.bone, P.blush);
  }

  // --- MIRE QUEEN: rises out of the bog -------------------------------------------
  function shotMire(t, age) {
    const f = Math.floor(ST(age) * 12 + 1e-6);
    RX.sky([[0, P.ink], [34, P.deep], [96, P.moss], [128, P.moss]]);
    const WY = 136; // bog surface
    g.save(); g.beginPath(); g.rect(0, 0, W, WY); g.clip();
    glow(160, WY, [[86, P.moss], [66, P.leaf], [44, P.lime]]);
    g.restore();
    // Dead mangroves.
    const tree = (x, h, seed, c) => {
      PX.rect(x - 2, WY - h, 5, h, c);
      for (let b = 0; b < 4; b++) {
        const y = WY - h + 6 + b * (h / 5), dir = b % 2 ? 1 : -1, len = 10 + PX.hash(b, seed) * 16;
        PX.line(x, y, x + dir * len, y - 6 - PX.hash(b, seed + 1) * 10, c);
        PX.line(x, y + 1, x + dir * len, y - 5 - PX.hash(b, seed + 1) * 10, c);
      }
      for (let r = -2; r <= 2; r++) PX.line(x, WY - 6, x + r * 5, WY + 2, c);
    };
    tree(24, 104, 3, P.deep); tree(298, 116, 4, P.deep); tree(64, 78, 5, P.ink); tree(258, 86, 6, P.ink);
    PX.rect(0, WY, W, H - WY, P.deep);
    for (let r = 0; r < 14; r++) {
      const y = WY + 3 + r * 3 + (r > 6 ? r - 6 : 0);
      const x = Math.round(((PX.hash(r, 90) * 360 + age * (r % 2 ? 14 : -10)) % 360 + 360) % 360) - 30;
      PX.rect(x, y, 16 + (r % 4) * 8, 1, r % 3 ? P.moss : P.leaf);
    }
    // Reflection of the glow on the bog.
    for (let r = 0; r < 6; r++) PX.rect(160 - 40 + r * 4 + (r % 2) * 3, WY + 2 + r * 2, 80 - r * 8, 1, r < 2 ? P.lime : P.leaf);
    // The queen, rising (clipped at the waterline).
    const bx = 160;
    let by = WY + 18, sx = 1, sy = 1;
    if (f === 0) { by = WY + 56; sx = 0.86; sy = 1.2; }
    else if (f === 1) { by = WY + 8; sx = 0.84; sy = 1.24; }
    else if (f === 2) { by = WY + 14; sx = 1.14; sy = 0.86; }
    else if (f === 3) { by = WY + 16; sx = 0.97; sy = 1.04; }
    else by = WY + 18 + (Math.floor(ST(t, 4)) % 2);
    const rev = clamp((age - 0.1) / 0.22);
    groundRing(age, bx, WY + 1, { r0: 50, r1: 110, life: 0.45, flat: 0.12, color: P.sprout, color2: P.leaf }, 'back');
    g.save(); g.beginPath(); g.rect(0, 0, W, WY + 1); g.clip();
    boss('mire-queen', bx, by, { k: 4, sx, sy, frame: Math.floor(ST(t, 6)) % 2, reveal: rev, lit: LIT['mire-queen'], sil: P.ink, rim: rev < 1 ? P.sprout : null });
    g.restore();
    groundRing(age, bx, WY + 1, { r0: 50, r1: 110, life: 0.45, flat: 0.12, color: P.sprout, color2: P.leaf }, 'front');
    groundRing(age - 0.12, bx, WY + 1, { r0: 50, r1: 80, life: 0.4, flat: 0.12, color: P.lime, color2: P.moss });
    PX.rect(bx - 54, WY, 108, 1, P.lime);
    // Splash: bog water and royal slime.
    PX.burst(age, bx, WY - 4, { n: 34, seed: 71, speed: 150, life: 0.6, colors: [P.lime, P.leaf, P.grape, P.blush, P.sprout], angle: -Math.PI / 2, spread: 2.2, up: 40, gravity: 320, size: 3 });
    // Slimelets hop out of the splash.
    for (let n = 0; n < 4; n++) {
      const a = age - 0.1 - n * 0.05;
      if (a < 0) continue;
      const dir = n % 2 ? 1 : -1, dist = 70 + n * 16;
      const u = Math.min(1, a / 0.34);
      const x = bx + dir * (30 + dist * u), y = WY - 2 - Math.sin(u * Math.PI) * (36 + n * 8);
      PX.spr('slimelet', Math.round(x - 10), Math.round(y - 16), Math.floor(ST(t, 8)) + n, { k: 2, flip: dir < 0 });
    }
    // Bubbles popping on the surface.
    for (let n = 0; n < 8; n++) {
      const per = 0.5 + PX.hash(n, 80) * 0.4, ph = ((t + PX.hash(n, 81) * per) % per) / per;
      const x = 20 + Math.floor(PX.hash(n, 82) * 280), y = WY + 6 + Math.floor(PX.hash(n, 83) * 30);
      if (ph < 0.8) PX.circle(x, y, 1 + Math.floor(ph * 3), P.leaf, false);
      else PX.burst((ph - 0.8) * per, x, y, { n: 4, seed: 84 + n, speed: 20, life: 0.2, colors: [P.sprout], up: 10, gravity: 60, size: 1 });
    }
    motes(t, 26, [P.sprout, P.lime], 9, { speed: 6, wind: 4 });
  }

  // --- BONE COLOSSUS: lands in the bone field, then stomps --------------------------
  function shotColossus(t, age) {
    const f = Math.floor(ST(age) * 12 + 1e-6);
    RX.sky([[0, P.ink], [30, P.night], [84, P.dusk], [120, P.slate], [130, P.slate]]);
    const MX = 270, MY = 70;
    glow(MX, MY, [[30, P.slate], [24, P.mauve]]);
    PX.circle(MX, MY, 19, P.fog, true);
    PX.circle(MX - 3, MY - 3, 15, P.bone, true);
    for (const [cx, cy, r] of [[7, 6, 3], [-6, 8, 2], [4, -8, 2]]) PX.circle(MX + cx, MY + cy, r, P.fog, true);
    // Titan ribcage on the horizon.
    ridge((x) => 124 - 8 * Math.abs(Math.sin(x * 0.03 + 0.5)), P.dusk, P.mauve);
    for (let r = 0; r < 7; r++) {
      const x0 = 18 + r * 13, h = 30 + Math.sin(r * 0.6) * 14;
      for (let y = 0; y < h; y++) {
        const u = y / h, x = x0 + Math.round(Math.sin(u * 2.2) * 10);
        PX.rect(x, 124 - y, 3, 1, P.mauve);
        PX.px(x, 124 - y, P.haze);
      }
    }
    PX.rect(10, 92, 96, 3, P.mauve);
    PX.rect(10, 92, 96, 1, P.haze);
    for (let l = 0; l < 3; l++) PX.dither(0.18 + l * 0.08, P.haze, 0, 110 + l * 6 - Math.round(Math.sin(t + l) * 2), W, 8);
    const GY = 152;
    PX.rect(0, 128, W, H - 128, P.dusk);
    PX.dither(0.3, P.slate, 0, 128, W, 22);
    for (let n = 0; n < 9; n++) {
      const x = Math.round(PX.hash(n, 30) * 300), y = 134 + Math.floor(PX.hash(n, 31) * 38);
      if (Math.abs(x - 150) < 60 && y < 156) continue;
      PX.spr(['p-ribs', 'p-skull', 'p-bone'][n % 3], x, y, 0, { k: 2 });
    }
    // Cut straight onto the landing.
    const bx = 160;
    let sx = 1, sy = 1, dy = 0;
    if (f === 0) { sx = 1.24; sy = 0.74; }
    else if (f === 1) { sx = 0.9; sy = 1.14; dy = -6; }
    else if (f === 2) { sx = 1.05; sy = 0.96; }
    // Second beat: rears up and stomps.
    const r2 = age - BEAT;
    const rf = Math.floor(ST(Math.max(0, r2)) * 12 + 1e-6);
    if (r2 >= 0) {
      if (rf === 0) { sx = 0.9; sy = 1.16; dy = -8; }
      else if (rf === 1) { sx = 1.18; sy = 0.82; }
      else if (rf === 2) { sx = 0.98; sy = 1.03; }
    }
    const rev = clamp((age - 0.08) / 0.26);
    const R1 = { r0: 40, r1: 140, life: 0.5, flat: 0.2, color: P.bone, color2: P.haze };
    const R2 = { r0: 40, r1: 100, life: 0.35, flat: 0.2, color: P.fog, color2: P.mauve };
    groundRing(age, bx, GY, R1, 'back'); groundRing(r2, bx, GY, R2, 'back');
    ellipse(bx, GY, 50, 4, P.night, true);
    boss('colossus', bx, GY + dy, { k: 4, sx, sy, frame: r2 >= 0 && rf < 3 ? 1 : 0, reveal: rev, dir: 'down', lit: LIT.colossus, sil: P.ink, rim: rev < 1 ? P.bone : null });
    groundRing(age, bx, GY, R1, 'front'); groundRing(r2, bx, GY, R2, 'front');
    // Eye flare on the stomp.
    if (r2 >= 0 && r2 < 0.3) {
      const [ex, ey] = at('colossus', bx, GY + dy, 4, 17, 7);
      glint(ex, ey, r2 < 0.08 ? 10 : r2 < 0.18 ? 7 : 4, r2 < 0.1 ? P.gold : P.ember, P.bone);
    }
    // Dust, bone shards.
    for (const [a0, sd] of [[age, 91], [r2, 93]]) {
      PX.burst(a0, bx - 44, GY - 2, { n: 16, seed: sd, speed: 90, life: 0.7, colors: [P.fog, P.haze, P.mauve], angle: Math.PI * 1.08, spread: 0.7, up: 30, gravity: 60, size: 4 });
      PX.burst(a0, bx + 44, GY - 2, { n: 16, seed: sd + 1, speed: 90, life: 0.7, colors: [P.fog, P.haze, P.mauve], angle: -Math.PI * 0.08, spread: 0.7, up: 30, gravity: 60, size: 4 });
    }
    for (let n = 0; n < 7; n++) {
      const a = age - n * 0.02;
      if (a < 0 || a > 0.7) continue;
      const vx = (PX.hash(n, 95) - 0.5) * 280, vy = -130 - PX.hash(n, 96) * 120;
      PX.spr('shard', Math.round(bx + vx * a), Math.round(GY - 20 + vy * a + 320 * a * a), Math.floor(a * 12), { k: 2 });
    }
    motes(t, 30, [P.fog, P.haze], 13, { speed: 5, wind: -6 });
  }

  // Orbs spiralling out around the wraith (back half behind it, front half in front).
  function orbs(t, age, bx, by, back) {
    const r2 = Math.max(0, age - BEAT); // on the second strike they spiral out at the camera
    for (let n = 0; n < 6; n++) {
      const a = ST(t, 24) * 2.4 + (n / 6) * Math.PI * 2 + r2 * 3;
      if ((Math.sin(a) < 0) !== back) continue;
      const r = 66 * Math.min(1, age / 0.3) + r2 * r2 * 700;
      const x = bx + Math.cos(a) * r, y = by - 44 + Math.sin(a) * r * 0.42;
      PX.spr('orb', Math.round(x - 6), Math.round(y - 6), Math.floor(ST(t, 8)) + n, { k: 2 });
    }
  }

  // --- STORM WRAITH: a strike, it appears in the flash, then phases in --------------
  function shotWraith(t, age) {
    const r2 = age - BEAT;
    const strike = (a) => a >= 0 && a < 0.1 && Math.floor(a * 60) % 4 !== 2;
    const lit = strike(age) || strike(r2);
    // The top strip stays dark so the name holds; the storm hangs below it.
    RX.sky(lit ? [[0, P.night], [36, P.navy], [80, P.blue], [130, P.sky]] : [[0, P.ink], [50, P.night], [104, P.navy], [130, P.navy]]);
    // Heavy clouds.
    const cloud = (c, bright) => {
      const w = 50 + Math.floor(PX.hash(c, 51) * 60);
      const x = Math.round(((PX.hash(c, 52) * 440 - age * (10 + c * 2)) % 440 + 440) % 440) - 80;
      const y = 48 + Math.floor(PX.hash(c, 53) * 46);
      ellipse(x, y, w / 2, 7 + (c % 3) * 2, bright ? P.blue : P.night, true);
      ellipse(x + 6, y - 3, w / 3, 5, bright ? P.sky : P.navy, true);
    };
    for (let c = 0; c < 12; c++) cloud(c, lit);
    // Sheet lightning flickering inside the clouds.
    const fl = Math.floor(age * 15);
    if (!lit && PX.hash(fl, 131) < 0.3) cloud(Math.floor(PX.hash(fl, 132) * 12), true);
    ridge((x) => 134 - 12 * Math.abs(Math.sin(x * 0.02 + 1)) - 5 * Math.sin(x * 0.09), lit ? P.blue : P.night, lit ? P.sky : P.navy);
    const GY = 152;
    PX.rect(0, 142, W, H - 142, lit ? P.navy : P.ink);
    for (let r = 0; r < 9; r++) PX.rect(Math.round(PX.hash(r, 57) * 280), 146 + r * 4, 20 + r * 3, 1, lit ? P.blue : P.navy);
    // Bolts: beat 1 on the left, beat 2 on the right; each leaves a splash of light on the ground.
    if (strike(age)) { bolt(96, 36, GY - 2, 21); ellipse(96, GY - 2, 24, 3, P.cyan, true); }
    if (strike(r2)) { bolt(228, 36, GY - 2, 23); ellipse(228, GY - 2, 24, 3, P.cyan, true); }
    const bx = 160, by = GY - 14 + Math.round(Math.sin(ST(t, 12) * 3.2) * 3);
    const ph = clamp((age - 0.1) / 0.3);
    const fr = Math.floor(ST(t, 8)) % 3;
    // Reflection under it; orbs behind it.
    ellipse(bx, GY + 4, 34, 3, P.navy, true);
    orbs(t, age, bx, by, true);
    if (age < 0.1) boss('wraith', bx, by, { k: 4, frame: fr, reveal: 0, lit: LIT.wraith, sil: P.ink });
    else {
      if (ph < 1) boss('wraith', bx, by, { k: 4, frame: fr, reveal: 0, lit: LIT.wraith, sil: P.ink, rim: P.cyan, alpha: 1 - ph * 0.9 });
      boss('wraith', bx, by, { k: 4, frame: fr, alpha: 0.1 + ph * 0.9 });
    }
    // Eyes flare on the second strike.
    if (r2 >= 0 && r2 < 0.28) {
      for (const i of [10, 13]) {
        const [ex, ey] = at('wraith', bx, by, 4, i, 6);
        glint(ex, ey, r2 < 0.08 ? 8 : r2 < 0.18 ? 5 : 3, r2 < 0.12 ? P.ice : P.cyan);
      }
    }
    orbs(t, age, bx, by, false);
    // Rain.
    for (let n = 0; n < 120; n++) {
      const sp = 280 + PX.hash(n, 61) * 140;
      const y = ((PX.hash(n, 62) * 220 + t * sp) % 220) - 20;
      const x = ((PX.hash(n, 63) * 380 - (y + 20) * 0.35) % 380 + 380) % 380 - 30;
      PX.line(x, y, x + 2, y - 6, lit ? P.ice : n % 3 ? P.blue : P.sky);
    }
  }

  // --- SANDMAW: erupts out of a dune, then roars and spits grit ----------------------
  function shotSandmaw(t, age) {
    const f = Math.floor(ST(age) * 12 + 1e-6);
    const r2 = age - BEAT; // second beat: the roar
    const rf = Math.floor(ST(Math.max(0, r2)) * 12 + 1e-6);
    RX.sky([[0, P.maroon], [36, P.blood], [80, P.red], [112, P.ember], [128, P.amber]]);
    glow(92, 114, [[40, P.ember], [32, P.amber]]);
    PX.circle(92, 114, 26, P.gold, true);
    PX.circle(92, 114, 20, P.cream, true);
    for (let y = 100; y < 142; y += 5) PX.rect(62, y, 60, 1, P.amber);
    ridge((x) => 124 - 10 * Math.sin(x * 0.018 + 1) - 4 * Math.sin(x * 0.05), P.clay, P.sand);
    ridge((x) => 140 - 7 * Math.sin(x * 0.025 + 3) - 3 * Math.sin(x * 0.06), P.umber, P.clay);
    const GY = 156;
    const bx = 196;
    let by = GY + 6, sx = 1, sy = 1, frame = Math.floor(ST(t, 8)) % 2;
    if (f === 0) { by = GY + 44; sx = 0.84; sy = 1.2; }
    else if (f === 1) { by = GY - 4; sx = 0.82; sy = 1.26; }
    else if (f === 2) { by = GY + 8; sx = 1.14; sy = 0.86; }
    else if (f === 3) { by = GY + 5; sx = 0.97; sy = 1.04; }
    if (r2 >= 0) {
      // Rears back on the beat, lunges into the roar, holds it shaking.
      frame = 1;
      if (rf === 0) { by = GY + 10; sx = 1.12; sy = 0.88; }
      else if (rf === 1) { by = GY - 6; sx = 0.88; sy = 1.2; }
      else if (rf === 2) { by = GY - 4; sx = 1.06; sy = 1.08; }
      else by = GY - 4 + (rf % 2);
    }
    const rev = clamp((age - 0.08) / 0.22);
    groundRing(age, bx, GY, { r0: 30, r1: 120, life: 0.45, flat: 0.18, color: P.cream, color2: P.sand }, 'back');
    groundRing(r2 - 1 / 12, bx, GY, { r0: 30, r1: 150, life: 0.4, flat: 0.16, color: P.gold, color2: P.amber }, 'back');
    // Roar: the maw glows, shock rings roll off it.
    if (r2 >= 1 / 12) {
      const [mx, my] = at('sandmaw', bx, by, 4, 10.5, 8);
      glow(mx, my, [[30 + (rf % 2) * 3, P.ember], [20, P.amber]]);
      for (let n = 0; n < 3; n++) PX.ring(r2 - 1 / 12 - n * 0.09, mx, my, { r0: 30, r1: 90, life: 0.3, color: P.cream, color2: P.gold });
    }
    g.save(); g.beginPath(); g.rect(0, 0, W, GY + 1); g.clip();
    boss('sandmaw', bx, by, { k: 4, sx, sy, frame, reveal: rev, lit: LIT.sandmaw, sil: P.ink, rim: rev < 1 ? P.gold : null });
    g.restore();
    // Dune in front, heaped up where it burst out.
    ridge((x) => GY - 2 * Math.sin(x * 0.03) + (Math.abs(x - bx) < 54 ? -Math.round(7 * Math.cos(((x - bx) / 54) * Math.PI / 2)) : 0), P.sand, P.cream);
    PX.dither(0.3, P.clay, 0, GY + 8, W, H - GY - 8);
    groundRing(age, bx, GY, { r0: 30, r1: 120, life: 0.45, flat: 0.18, color: P.cream, color2: P.sand }, 'front');
    groundRing(r2 - 1 / 12, bx, GY, { r0: 30, r1: 150, life: 0.4, flat: 0.16, color: P.gold, color2: P.amber }, 'front');
    // Dust cloud rolling out from the base, sand fountain, grit.
    if (age < 0.6) {
      const u = age / 0.6;
      ditherEllipse(bx, GY - 4, 50 + u * 90, 10 + u * 14, P.sand, 0.7 * (1 - u));
      ditherEllipse(bx, GY - 6, 40 + u * 60, 8 + u * 10, P.cream, 0.45 * (1 - u));
    }
    PX.burst(age, bx, GY - 10, { n: 60, seed: 111, speed: 230, life: 0.8, colors: [P.sand, P.cream, P.clay, P.umber], angle: -Math.PI / 2, spread: 1.6, up: 90, gravity: 260, size: 3 });
    PX.burst(age, bx, GY - 4, { n: 24, seed: 113, speed: 120, life: 0.5, colors: [P.cream, P.sand], angle: -Math.PI / 2, spread: 3.0, up: 20, gravity: 200, size: 2 });
    for (let n = 0; n < 8; n++) {
      const a = age - 0.1 - n * 0.03;
      if (a < 0 || a > 0.6) continue;
      const ang = -Math.PI / 2 + (n - 3.5) * 0.36;
      PX.spr('grit', Math.round(bx + Math.cos(ang) * (30 + 220 * a) - 5), Math.round(GY - 50 + Math.sin(ang) * (30 + 220 * a) - 5), Math.floor(a * 12), { k: 2 });
    }
    // The roar spits a fan of grit straight at the camera; it grows as it comes.
    for (let n = 0; n < 10; n++) {
      const a = r2 - 1 / 12 - (n % 3) * 0.03;
      if (a < 0 || a > 0.4) continue;
      const ang = (n / 10) * Math.PI * 2 + 0.3;
      const [mx, my] = at('sandmaw', bx, by, 4, 10.5, 8);
      const d = 20 + 520 * a * a + 120 * a;
      const kk = a < 0.12 ? 2 : a < 0.25 ? 3 : 4;
      PX.spr('grit', Math.round(mx + Math.cos(ang) * d - 2.5 * kk), Math.round(my + Math.sin(ang) * d * 0.8 - 2.5 * kk), Math.floor(a * 12) + n, { k: kk });
    }
    PX.burst(r2 - 1 / 12, bx, GY - 8, { n: 40, seed: 117, speed: 200, life: 0.5, colors: [P.sand, P.cream, P.clay], angle: -Math.PI / 2, spread: 2.4, up: 60, gravity: 240, size: 3 });
    // Sandstorm streaks.
    for (let n = 0; n < 60; n++) {
      const sp = 300 + PX.hash(n, 121) * 200;
      const x = ((PX.hash(n, 122) * 400 - t * sp) % 400 + 400) % 400 - 40;
      const y = Math.floor(PX.hash(n, 123) * 180);
      PX.rect(x, y, 6 + Math.floor(PX.hash(n, 124) * 10), 1, n % 3 ? P.sand : P.cream);
    }
  }

  // --- HIVE EYE: the eye opens -------------------------------------------------------
  function hive(t, scroll) {
    PX.clear(P.maroon);
    const s = 10, hx = s * 1.5, hy = s * 1.732;
    for (let cy = -1; cy < 13; cy++) for (let cx = -2; cx < 24; cx++) {
      const x = cx * hx + (scroll % (hx * 2)), y = cy * hy + (cx & 1 ? hy / 2 : 0);
      const lit = PX.hash(cx * 31 + cy * 7, 140) < 0.14;
      if (lit) {
        const p = 0.5 + 0.5 * Math.sin(t * 5 + cx * 1.3 + cy);
        const c = p > 0.66 ? P.pink : p > 0.33 ? P.grape : P.plum;
        for (let yy = -8; yy <= 8; yy++) {
          const w = Math.round(8.66 - Math.abs(yy) * 0.5);
          g.fillStyle = c; g.fillRect(Math.round(x - w), Math.round(y + yy), w * 2 + 1, 1);
        }
      }
      for (let e = 0; e < 6; e++) {
        const a0 = (e / 6) * Math.PI * 2, a1 = ((e + 1) / 6) * Math.PI * 2;
        PX.line(x + Math.cos(a0) * s, y + Math.sin(a0) * s, x + Math.cos(a1) * s, y + Math.sin(a1) * s, P.plum);
      }
    }
    PX.dither(0.5, P.ink, 0, 0, W, 22);
    PX.dither(0.25, P.ink, 0, 22, W, 14);
    PX.dither(0.25, P.ink, 0, 150, W, 10);
    PX.dither(0.5, P.ink, 0, 160, W, 20);
  }
  const OPEN = 3 / 12;
  function shotHive(t, age) {
    const bx = 160, by = 150 + Math.round(Math.sin(ST(t, 12) * 3) * 2);
    hive(t, t * 8);
    const open = age >= OPEN;
    const rev = clamp((age - 0.06) / 0.18);
    glow(bx, by - 42, open ? [[66, P.plum], [56, P.pink]] : [[60, P.plum]]);
    // Mites swarming.
    for (let n = 0; n < 12; n++) {
      const a = ST(t, 24) * (1.6 + PX.hash(n, 150)) + PX.hash(n, 151) * 6.28;
      const r = 76 + PX.hash(n, 152) * 70;
      const x = bx + Math.cos(a) * r, y = by - 44 + Math.sin(a * 1.3) * r * 0.45;
      PX.spr('mite', Math.round(x - 8), Math.round(y - 7), Math.floor(ST(t, 12)) + n, { k: 2, flip: Math.sin(a) > 0 });
    }
    let sx = 1, sy = 1;
    const f = Math.floor(ST(age) * 12 + 1e-6);
    if (f === 0) { sx = 1.14; sy = 0.88; }
    else if (f === 1) { sx = 0.94; sy = 1.07; }
    const of = Math.floor(ST(age - OPEN) * 12 + 1e-6);
    if (open && of === 0) { sx = 0.9; sy = 1.14; }
    else if (open && of === 1) { sx = 1.08; sy = 0.94; }
    const frame = open ? [0, 0, 1, 2, 0, 3, 4][Math.min(6, Math.floor(ST(age - OPEN, 8) * 8))] : 5;
    boss('hive-eye', bx, by, { k: 4, sx, sy, frame, reveal: rev, lit: LIT['hive-eye'], sil: P.ink, rim: rev < 1 ? P.blush : null, flash: open && of === 0 ? P.bone : null });
    // Needles fan out when it opens.
    const na = age - OPEN;
    if (na >= 0) {
      for (let n = 0; n < 12; n++) {
        const ang = (n / 12) * Math.PI * 2 + 0.26;
        const r = 48 + na * 280;
        PX.spr('needle', Math.round(bx + Math.cos(ang) * r - 5), Math.round(by - 42 + Math.sin(ang) * r - 5), n, { k: 2 });
      }
      PX.ring(na, bx, by - 42, { r0: 34, r1: 140, life: 0.3, color: P.blush, color2: P.pink });
    }
    if (open && na < 1 / 30) PX.dither(0.4, P.bone);
  }
  // Last beat: close on the eye; the pupil dilates with its fight inside, until the
  // fight fills the frame by 52.44 s.
  const PUPIL_FRAMES = [10, 10, 11, 11, 12, 12, 13, 13, 14, 14, 18, 19, 20, 21];
  const fb = document.createElement('canvas'); fb.width = W; fb.height = H;
  const fbg = fb.getContext('2d'); fbg.imageSmoothingEnabled = false;
  function shotPupil(t, age, b) {
    const u = clamp(age / BEAT);
    hive(t, t * 8);
    const k = 8, pcx = 11, pcy = 9.5;
    const [sx, sy] = PX.shake(age, 1 + u * 4, BEAT + 0.1, 170);
    const ox = Math.round(W / 2 - pcx * k) + sx, oy = Math.round(H / 2 - pcy * k) + sy;
    const d = RX.pixels('hive-eye', 0);
    glow(W / 2 + sx, H / 2 + sy, [[96, P.plum], [88, P.pink]]);
    RX.drawSpr('hive-eye', 0, ox + (d.w * k) / 2, oy + d.h * k, { k });
    // The fight, framed on the eye, drawn inside the pupil disc.
    // Footage frames picked to keep the eye open (its hit-flash frames 15-17 are skipped),
    // eased in slow at first, landing on the open red iris at the hand-off.
    const fr = b.cuts[0];
    const seq = PUPIL_FRAMES[Math.min(PUPIL_FRAMES.length - 1, Math.floor(age * 30 + 1e-6))];
    const im = PX.footageFrame('boss-' + b.id, (seq + 0.5) / 30);
    fbg.fillStyle = P.ink; fbg.fillRect(0, 0, W, H);
    if (im) fbg.drawImage(im, Math.round(W / 2 - fr.cx * fr.k), Math.round(H / 2 - fr.cy * fr.k), im.width * fr.k, im.height * fr.k);
    const R = Math.round(11 + u * u * 232);
    const cx = W / 2 + sx, cy = H / 2 + sy;
    for (let y = Math.max(0, cy - R); y < Math.min(H, cy + R + 1); y++) {
      const w = Math.round(Math.sqrt(Math.max(0, R * R - (y - cy) * (y - cy))));
      const x0 = Math.max(0, cx - w), x1 = Math.min(W, cx + w + 1);
      if (x1 > x0) g.drawImage(fb, x0, y, x1 - x0, 1, x0, y, x1 - x0, 1);
    }
    PX.circle(cx, cy, R + 1, P.blood, false);
    PX.circle(cx, cy, R + 2, P.red, false);
    PX.circle(cx, cy, R + 3, P.blood, false);
  }

  // --- gameplay clip ---------------------------------------------------------------
  function shotClip(t, b, beatInClip) {
    const fr = b.cuts[Math.min(beatInClip, b.cuts.length - 1)];
    const src = fr.in + (t - bt(b.clip + beatInClip));
    const k = fr.k, cx = clamp(fr.cx, W / (2 * k), 320 - W / (2 * k)), cy = clamp(fr.cy, H / (2 * k), 180 - H / (2 * k));
    PX.clear(P.ink);
    PX.footage('boss-' + b.id, src, { k, cx, cy });
  }

  // --- dispatcher ------------------------------------------------------------------
  const SLAM = { 'mire-queen': shotMire, colossus: shotColossus, wraith: shotWraith, sandmaw: shotSandmaw, 'hive-eye': shotHive };
  function draw(t, local) {
    const b = Math.floor(local / BEAT + 1e-6);
    const beatAge = local - b * BEAT;
    let shakeAmp = 0, shakeAge = 0, shakeDur = 0.4, glitchAge = -1;
    g.save();
    if (b < 2) {
      shotPack(t, local);
      slamTitle('5 BOSSES', local, Object.assign({ noFlash: local < 1 / 30 }, PACK_TITLE));
      shakeAmp = 6; shakeAge = local; shakeDur = 0.5;
    } else {
      const bo = BOSSES.find((x) => b >= x.at && b < x.end);
      const age = t - bt(bo.at);
      if (b < bo.clip) {
        SLAM[bo.id](t, age);
        if (bo.id !== 'wraith') flashIn(age); // the wraith's flash is its lightning
        slamTitle(bo.name, age, bo.title);
        shakeAmp = bo.id === 'hive-eye' ? 4 : 6; shakeAge = age;
        if (age >= BEAT && (bo.id === 'colossus' || bo.id === 'sandmaw')) { shakeAmp = 4; shakeAge = age - BEAT; }
        if (bo.id === 'wraith' && age >= BEAT) { shakeAmp = 2; shakeAge = age - BEAT; }
        if (bo.id === 'hive-eye' && age >= OPEN) { shakeAmp = 3; shakeAge = age - OPEN; }
      } else if (bo.id === 'hive-eye') {
        shotPupil(t, t - bt(bo.clip), bo);
      } else {
        shotClip(t, bo, b - bo.clip);
        glitchAge = beatAge;
        shakeAmp = 2; shakeAge = beatAge; shakeDur = 0.15;
      }
    }
    g.restore();
    const [dx, dy] = PX.shake(shakeAge, shakeAmp, shakeDur, 7 + b);
    shakeBuf(dx, dy);
    if (glitchAge >= 0) slices(glitchAge, b + 3);
  }

  const S32 = BEAT / 8;
  const cues = [];
  // Bar 25 downbeat sits on the music's biggest hit: keep the stack light, zap a 32nd late.
  cues.push([bt(0), 'impact', 0.85], [bt(0), 'slam', 0.6], [bt(0) + S32, 'zap', 0.35]);
  for (let i = 0; i < 5; i++) cues.push([bt(0) + IGNITE(i), 'blip', 0.3]);
  cues.push([bt(2), 'slam', 0.9], [bt(2) + 1 / 12, 'boom', 0.5], [bt(3), 'whoosh', 0.35]);
  cues.push([bt(4), 'impact', 0.9], [bt(4), 'slam', 0.7], [bt(5), 'boom', 0.6], [bt(6), 'whoosh', 0.35], [bt(7), 'whoosh', 0.3]);
  cues.push([bt(8), 'impact', 0.85], [bt(8), 'slam', 0.7], [bt(8) + S32, 'zap', 0.6], [bt(9), 'zap', 0.6], [bt(10), 'whoosh', 0.35], [bt(11), 'whoosh', 0.3]);
  cues.push([bt(12), 'impact', 0.9], [bt(12), 'slam', 0.75], [bt(12) + 1 / 12, 'boom', 0.55], [bt(13) + 1 / 12, 'boom', 0.6]);
  cues.push([bt(14), 'slam', 0.9], [bt(14) + OPEN, 'impact', 0.5], [bt(15), 'whoosh', 0.6]);

  PX.scene({ id: 'roster-bosses', start: T0, end: T1, layer: 0, draw, cues });
})();
