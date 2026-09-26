// 20-roster: bars 21-24 (37.5-45 s), "7 HEROES".
// A stop-motion roster on a sunset ridge under a smoking volcano. The "7" slams on the
// bar 21 downbeat and HEROES rolls in on 32nds into Rex's pop. Each hero pops onto the
// ridge on its beat and lands straight into a compressed version of its real kit, then
// a 1-beat gameplay flash of that hero. Older heroes only idle. The title hops off after
// bar 21 and slams back for the finale: all 7 land in unison and fire one converging volley.
// Shared pixel helpers live on window.RX (also used by 21-bosses.js).
(function () {
  const { P, BEAT, W, H } = PX;
  const g = PX.g;

  // --- RX: per-pixel sprite drawing (squash/stretch, silhouettes, dithered reveals) ---
  const pixCache = new Map();
  const scratch = document.createElement('canvas');
  const sg = scratch.getContext('2d', { willReadFrequently: true });
  // Source pixels of a sprite frame as [{i, j, c}] (memoized per image; pure).
  function pixels(name, frame) {
    const im = PX.sprite(name, frame);
    if (!im || !im.width) return null;
    let d = pixCache.get(im);
    if (d) return d;
    scratch.width = im.width; scratch.height = im.height;
    sg.clearRect(0, 0, im.width, im.height);
    sg.drawImage(im, 0, 0);
    const raw = sg.getImageData(0, 0, im.width, im.height).data;
    const list = [];
    const hex = (v) => v.toString(16).padStart(2, '0');
    for (let j = 0; j < im.height; j++) for (let i = 0; i < im.width; i++) {
      const o = (j * im.width + i) * 4;
      if (raw[o + 3] < 128) continue;
      list.push({ i, j, c: '#' + hex(raw[o]) + hex(raw[o + 1]) + hex(raw[o + 2]) });
    }
    d = { w: im.width, h: im.height, list };
    pixCache.set(im, d);
    return d;
  }
  // Draw a sprite anchored at its bottom-center (bx, by) with pixel scale k and
  // squash (sx, sy). Each source pixel maps to an integer dest rect, so pixels stay crisp.
  // o.color(c, i, j) -> color|null lets callers flash, silhouette or dissolve per pixel.
  function drawSpr(name, frame, bx, by, o = {}) {
    const d = pixels(name, frame);
    if (!d) return;
    const k = o.k || 2, kx = k * (o.sx || 1), ky = k * (o.sy || 1);
    const w = d.w * kx, h = d.h * ky;
    const x0 = Math.round(bx - w / 2), y0 = Math.round(by - h);
    for (const p of d.list) {
      const i = o.flip ? d.w - 1 - p.i : p.i;
      let c = o.color ? o.color(p.c, p.i, p.j) : p.c;
      if (!c) continue;
      const ax = x0 + Math.round(i * kx), bx2 = x0 + Math.round((i + 1) * kx);
      const ay = y0 + Math.round(p.j * ky), by2 = y0 + Math.round((p.j + 1) * ky);
      g.fillStyle = c;
      g.fillRect(ax, ay, Math.max(1, bx2 - ax), Math.max(1, by2 - ay));
    }
  }
  // Solid 1px outline of a sprite (drawn before the sprite) for rim light / readability.
  function outlineSpr(name, frame, bx, by, color, o = {}) {
    for (const [dx, dy] of [[-1, 0], [1, 0], [0, -1], [0, 1]]) drawSpr(name, frame, bx + dx, by + dy, Object.assign({}, o, { color: () => color }));
  }
  // Hit flash in palette: every color steps two shades up its ramp, ink lines stay, so the
  // sprite keeps its features while it lights up.
  const RAMPS = [
    ['night', 'dusk', 'slate', 'mauve', 'haze', 'fog', 'bone'],
    ['maroon', 'blood', 'red', 'ember', 'amber', 'gold', 'cream', 'bone'],
    ['deep', 'moss', 'leaf', 'lime', 'sprout', 'cream', 'bone'],
    ['navy', 'blue', 'sky', 'cyan', 'ice', 'bone'],
    ['plum', 'grape', 'pink', 'blush', 'bone'],
    ['umber', 'clay', 'sand', 'cream', 'bone'],
  ];
  const LIGHT = {};
  for (const r of RAMPS) r.forEach((n, i) => { if (!LIGHT[P[n]]) LIGHT[P[n]] = P[r[Math.min(r.length - 1, i + 2)]]; });
  const hitFlash = (c) => LIGHT[c] || c;
  // Vertical sky gradient made of dithered bands: stops [[y, color], ...].
  function sky(stops) {
    for (let s = 0; s < stops.length - 1; s++) {
      const [y0, c0] = stops[s], [y1, c1] = stops[s + 1];
      PX.rect(0, y0, PX.W, y1 - y0, c0);
      const n = y1 - y0;
      for (let y = 0; y < n; y++) {
        const a = y / n;
        if (a > 0.5) PX.dither((a - 0.5) * 2, c1, 0, y0 + y, PX.W, 1);
      }
    }
  }
  // Ridge silhouette: height function h(x) -> top y; fills to the bottom.
  function ridge(fn, color, top) {
    g.fillStyle = color;
    for (let x = 0; x < PX.W; x++) { const y = Math.round(fn(x)); g.fillRect(x, y, 1, PX.H - y); if (top) { g.fillStyle = top; g.fillRect(x, y, 1, 1); g.fillStyle = color; } }
  }
  // Short directional streak (a projectile trail).
  function streak(x, y, vx, vy, len, c) {
    const m = Math.hypot(vx, vy) || 1;
    PX.line(x, y, x - (vx / m) * len, y - (vy / m) * len, c);
  }
  // Posterized glow: filled discs from the outside in, each edge softened by a Bayer band.
  function glow(cx, cy, rings) {
    for (const [r, c] of rings) {
      const R = r + 3;
      g.fillStyle = c;
      for (let y = Math.max(0, Math.floor(cy - R)); y <= Math.min(PX.H - 1, cy + R); y++) {
        for (let x = Math.max(0, Math.floor(cx - R)); x <= Math.min(PX.W - 1, cx + R); x++) {
          const d = Math.hypot(x - cx, y - cy);
          if (d > r && d <= R && PX.bayer(x, y) < 0.5 * (1 - (d - r) / 3.5)) g.fillRect(x, y, 1, 1);
        }
      }
      PX.circle(cx, cy, r, c, true);
    }
  }
  // Filled / outlined pixel ellipse. half: 'back' (upper arc) | 'front' (lower arc).
  function ellipse(cx, cy, rx, ry, c, fill, half) {
    cx = Math.round(cx); cy = Math.round(cy); rx = Math.max(1, Math.round(rx)); ry = Math.max(1, Math.round(ry));
    g.fillStyle = c;
    const ok = (y) => !half || (half === 'back' ? y <= 0 : y > 0);
    for (let y = -ry; y <= ry; y++) {
      if (!ok(y)) continue;
      const w = Math.round(rx * Math.sqrt(Math.max(0, 1 - (y * y) / (ry * ry))));
      if (fill) g.fillRect(cx - w, cy + y, w * 2 + 1, 1);
      else { g.fillRect(cx - w, cy + y, 1, 1); g.fillRect(cx + w, cy + y, 1, 1); }
    }
    if (!fill) for (let x = -rx; x <= rx; x++) {
      const h = Math.round(ry * Math.sqrt(Math.max(0, 1 - (x * x) / (rx * rx))));
      if (ok(-h)) g.fillRect(cx + x, cy - h, 1, 1);
      if (ok(h)) g.fillRect(cx + x, cy + h, 1, 1);
    }
  }
  // Flat shockwave ring on the ground. Draw the 'back' half before a sprite, 'front' after.
  function groundRing(age, cx, cy, o = {}, half) {
    const life = o.life || 0.4;
    if (age < 0 || age > life) return;
    const u = age / life, r = (o.r0 || 8) + PX.ease.out(u) * (o.r1 || 90);
    const c = u < 0.5 ? o.color || P.bone : o.color2 || P.mauve;
    ellipse(cx, cy, r, r * (o.flat || 0.22), c, false, half);
    if (u < 0.35) ellipse(cx, cy, r - 1, (r - 1) * (o.flat || 0.22), c, false, half);
  }
  // Four-point star glint.
  function glint(x, y, s, c, core = P.bone) {
    x = Math.round(x); y = Math.round(y);
    PX.rect(x - s, y, s * 2 + 1, 1, c);
    PX.rect(x, y - s, 1, s * 2 + 1, c);
    const d = Math.max(1, Math.floor(s / 3));
    PX.rect(x - d, y - d, d * 2 + 1, d * 2 + 1, c);
    PX.px(x, y, core);
  }
  // Shake the finished buffer by (dx, dy), clamping the edges instead of showing ink seams.
  const sk = document.createElement('canvas'); sk.width = PX.W; sk.height = PX.H;
  const skg = sk.getContext('2d');
  function shakeBuf(dx, dy) {
    if (!dx && !dy) return;
    const W = PX.W, H = PX.H;
    const grab = () => { skg.clearRect(0, 0, W, H); skg.drawImage(g.canvas, 0, 0); };
    grab();
    g.drawImage(sk, dx, dy);
    if (dy > 0) g.drawImage(sk, 0, 0, W, 1, dx, 0, W, dy);
    if (dy < 0) g.drawImage(sk, 0, H - 1, W, 1, dx, H + dy, W, -dy);
    if (dx) {
      grab();
      if (dx > 0) g.drawImage(sk, dx, 0, 1, H, 0, 0, dx, H);
      else g.drawImage(sk, W + dx - 1, 0, 1, H, W + dx, 0, -dx, H);
    }
  }

  // --- RX titles: contact on the hit frame, palette-only flashes and shine ---------
  // PX.title plus a diagonal shine drawn in palette colors (the engine's shine and
  // dropIn flash write #ffffff). The band is clipped with whole-pixel rects, so no AA.
  function title(str, cx, y, o = {}) {
    const base = Object.assign({}, o, { shine: null });
    PX.title(str, cx, y, base);
    if (o.shine == null) return;
    const fx = o.fx || (() => ({}));
    const band = (lo, hi, col) => {
      g.save();
      g.beginPath();
      for (let yy = 0; yy < H; yy++) {
        const x0 = Math.max(0, Math.round(o.shine + lo - yy)), x1 = Math.min(W, Math.round(o.shine + hi - yy));
        if (x1 > x0) g.rect(x0, yy, x1 - x0, 1);
      }
      g.clip();
      PX.title(str, cx, y, Object.assign({}, base, { fx: (i) => Object.assign({}, fx(i) || {}, { flash: col }) }));
      g.restore();
    };
    band(-7, -3, P.cream);
    band(-3, 0, P.bone);
  }
  // Letters that fall and make contact at their own times: land(i) = contact age of letter i.
  // Contact frames (age >= land) squash and flash bone, then overshoot and settle.
  function dropFx(age, land, o = {}) {
    const fall = o.fall || 0.12, height = o.height || 60;
    return (i) => {
      const a = age - land(i);
      if (a < -fall) return { vis: false };
      if (a < 0) { const u = 1 + a / fall; return { dy: -Math.round(height * (1 - u * u)), sx: 0.8, sy: 1.3 }; }
      if (a < 2 / 30) return { sx: 1.35, sy: 0.65, flash: P.bone };
      if (a < 4 / 30) return { dy: -3, sx: 0.9, sy: 1.12 };
      if (a < 6 / 30) return { sx: 1.05, sy: 0.95 };
      return {};
    };
  }
  // Whole-word slam that is already in contact on the hit frame (age 0): wide squash and
  // flash on frames 0-1, overshoot on 2-3, settle. Squash spreads letters from the word center.
  function slamFx(str, k, age, o = {}) {
    const f = Math.floor(age * 30 + 1e-6);
    let sx = 1, sy = 1, flash = null;
    if (f <= 0) { sx = 1.3; sy = 0.7; flash = o.noFlash ? null : P.bone; }
    else if (f === 1) { sx = 1.14; sy = 0.84; flash = o.noFlash ? null : P.bone; }
    else if (f <= 3) { sx = 0.95; sy = 1.1; }
    else if (f === 4) { sx = 1.03; sy = 0.97; }
    const tw = PX.titleWidth(str, k);
    return (i) => {
      const left = i === 0 ? 0 : PX.titleWidth(str.slice(0, i), k) + k;
      const c = left + PX.titleWidth(str[i], k) / 2 - tw / 2;
      return { dx: Math.round(c * (sx - 1)), sx, sy, flash };
    };
  }
  window.RX = { pixels, drawSpr, outlineSpr, hitFlash, sky, ridge, streak, glow, ellipse, groundRing, glint, shakeBuf, title, dropFx, slamFx };

  // --- timing ------------------------------------------------------------------
  const T0 = PX.bar(21);
  const T1 = PX.bar(25);
  const beatT = (b) => T0 + b * BEAT;
  const S32 = BEAT / 8;
  const popBeat = (i) => 1 + i * 2;   // heroes pop on beats 1,3,...,13
  const flashBeat = (i) => 2 + i * 2; // their gameplay flash on beats 2,4,...,14
  const ANTIC_B = 14.75;             // the lineup is back a 16th early, crouching
  const FINALE_B = 15;               // unison landing, title slams back, converging volley
  const EXIT_B = 3.5;                // title letters hop off
  // Title letter contacts (s after the downbeat): the "7" on the downbeat, HEROES on
  // 32nds 3..8, so the roll resolves on beat 1 with Rex's pop.
  const TITLE = '7 HEROES';
  const landAt = (i) => (i === 0 ? 0 : (i + 1) * S32);
  const TK = 3, TY = 10; // title pixel scale and top

  const HEROES = [
    { id: 'rex', name: 'REX', shot: 'bolt', col: P.gold },
    { id: 'ptera', name: 'PTERA', fly: true, shot: 'feather-shot', col: P.cyan },
    { id: 'trike', name: 'TRIKE', shot: 'horn', col: P.amber },
    { id: 'raptor', name: 'RAPTOR', shot: 'claw', col: P.red },
    { id: 'spino', name: 'SPINO', shot: 'bubble', col: P.ice },
    { id: 'stego', name: 'STEGO', shot: 'spike', col: P.blush },
    { id: 'pachy', name: 'PACHY', shot: 'pebble', col: P.sand },
  ];
  const K = 2;
  // World x of each hero's spot. The camera trucks right, so screen x = world x + camX.
  const slotX = (i) => 160 + (i - 3) * 40;
  // The camera trucks right with the roll call so the newest hero has room for its move,
  // then the cut back for the finale reframes on the centered lineup and holds.
  const camAt = (local) => (local >= ANTIC_B * BEAT ? 0 : Math.round(-local * 4));
  // The ridge the heroes stand on (world x): highest in the middle.
  const groundY = (x) => 150 - 10 * Math.cos(((x - 172) / 180) * (Math.PI / 2)) + 2 * Math.sin(x * 0.09);
  const feetY = (i) => Math.round(groundY(slotX(i)));

  // --- world -------------------------------------------------------------------
  // Layers scroll with the camera by their depth p: screen x = world x + camX * p.
  const VOLC = 58; // far-range volcano, world x at p = 0.2
  function world(t, local, camX, camY) {
    // The opening tilt: each layer drops by its depth, the sky least.
    const lay = (p, fn) => { g.save(); g.translate(0, Math.round(camY * p)); fn(); g.restore(); };
    const vx = VOLC + camX * 0.2;
    PX.clear(P.plum);
    lay(0.3, () => sky([[0, P.plum], [22, P.grape], [44, P.pink], [66, P.ember], [86, P.amber], [104, P.gold], [118, P.cream]]));
    lay(0.45, () => {
      // Low sun, sinking a hair over the section.
      const sunX = 214 + Math.round(camX * 0.05), sunY = 86 + Math.round(local * 0.8);
      glow(sunX, sunY, [[38, P.amber], [30, P.gold]]);
      PX.circle(sunX, sunY, 22, P.gold, true);
      PX.circle(sunX, sunY, 17, P.cream, true);
      for (let y = sunY - 18; y < sunY + 22; y += 5) PX.rect(sunX - 24, y, 48, 1, P.amber);
      // Clouds drifting left.
      for (let c = 0; c < 5; c++) {
        const w = 26 + Math.floor(PX.hash(c, 5) * 30);
        const x = Math.round(((PX.hash(c, 6) * 420 - local * (4 + c * 1.5) + camX * 0.1) % 420 + 420) % 420) - 60;
        const y = 26 + Math.floor(PX.hash(c, 7) * 40);
        PX.rect(x, y, w, 2, P.blush);
        PX.rect(x + 6, y - 2, w - 14, 2, P.blush);
        PX.rect(x + 3, y + 2, w - 4, 1, P.pink);
      }
      // A flock of pteros crossing the sky, far away.
      for (let n = 0; n < 5; n++) {
        const x = Math.round(330 - local * 26 - n * 13 - (n % 2) * 6);
        const y = Math.round(40 + n * 4 + (n % 2) * 3 + Math.sin(local * 3 + n) * 2);
        if (x > -20 && x < W) PX.spr('ptero', x, y, Math.floor(PX.step(t, 8) * 8) + n, { flash: P.plum });
      }
      // Volcano smoke, rising and drifting with the wind.
      const puffs = [];
      for (let k = 0; k < 16; k++) {
        const u = ((local * 0.3 + k / 16) % 1 + 1) % 1;
        puffs.push([u, vx + u * u * 46 + Math.sin(u * 6 + k * 1.7) * 3, 71 - u * 62, 2 + u * 11]);
      }
      puffs.sort((p1, p2) => p2[0] - p1[0]);
      // Rim light on the whole plume's sunward edge, then the body on top.
      for (const [u, px, py, r] of puffs) if (u <= 0.85) PX.circle(px + 1, py - 1, r, P.grape, true);
      for (const [u, px, py, r] of puffs) {
        if (u > 0.85) { PX.dither(0.5, P.plum, Math.round(px - r), Math.round(py - r), Math.round(r * 2), Math.round(r * 2)); continue; }
        PX.circle(px, py, r, P.plum, true);
      }
    });
    // Far range (with the volcano), mid hills, the near ridge.
    lay(0.65, () => {
      ridge((x) => {
        const wx = x - camX * 0.2;
        let y = 116 - 22 * Math.abs(Math.sin(wx * 0.021)) - 8 * Math.sin(wx * 0.07);
        const d = Math.abs(wx - VOLC);
        if (d < 44) y = Math.min(y, d < 3 ? 75 : 72 + Math.max(0, d - 5) * 1.12);
        return y;
      }, P.grape, P.pink);
      // Crater glow.
      const glowOn = Math.floor(PX.step(t, 6) * 6) % 3;
      PX.rect(vx - 4, 72, 9, 1, glowOn ? P.ember : P.amber);
      PX.rect(vx - 2, 73, 5, 1, P.gold);
    });
    lay(0.85, () => ridge((x) => { const wx = x - camX * 0.5; return 130 - 10 * Math.sin(wx * 0.035) - 4 * Math.sin(wx * 0.11); }, P.plum, P.grape));
    lay(1, () => {
      ridge((x) => groundY(x - camX), P.umber, P.clay);
      // Soil texture and grass on the ridge top (anchored to the world).
      PX.dither(0.25, P.maroon, 0, 154, W, 26);
      const sway = PX.step(t, 12);
      for (let wx = Math.floor(-camX / 3) * 3 - 3; wx < W - camX + 3; wx += 3) {
        const x = wx + camX, y = Math.round(groundY(wx));
        const hgt = 2 + Math.floor(PX.hash(wx, 11) * 3);
        const lean = Math.round(Math.sin(sway * 5 + wx * 0.12) * 1.2);
        PX.line(x, y, x + lean, y - hgt, PX.hash(wx, 12) < 0.5 ? P.leaf : P.lime);
      }
    });
    // Foreground bank, sliding faster than the ridge.
    lay(1.3, () => {
      ridge((x) => { const wx = x - camX * 1.6; return 172 - 3 * Math.sin(wx * 0.05) - 2 * Math.sin(wx * 0.13); }, P.maroon, P.umber);
      for (let n = 0; n < 9; n++) {
        const x = Math.round(((n * 47 + camX * 1.6) % 360 + 360) % 360) - 20;
        PX.spr(['p-fern', 'p-tuft', 'p-rock'][n % 3], x, 162 + (n % 3) * 2, 0, { k: 2 });
      }
    });
    // Dust motes floating up through the light.
    for (let n = 0; n < 24; n++) {
      const sp = 6 + PX.hash(n, 21) * 10;
      const x = Math.round((PX.hash(n, 22) * 320 + Math.sin(t * 1.3 + n) * 4 + camX * 0.8 + 640) % 320);
      const y = Math.round(((PX.hash(n, 23) * 160 - t * sp) % 160 + 160) % 160);
      PX.px(x, y, n % 3 ? P.cream : P.gold);
    }
  }

  // --- heroes ------------------------------------------------------------------
  // The signature move, compressed so the payoff lands 2/12 s after the landing and stays
  // on screen ~0.2 s before the gameplay flash. st: stop-motion age since the pop (12 fps),
  // a: smooth age (projectiles, particles). Landing is at 1/12, fire at 2/12, payoff at 3/12.
  const FIRE = 2 / 12, PAY = 3 / 12;
  function act(h, i, st, a, x, fy) {
    const pose = { dx: 0, dy: 0, sx: 1, sy: 1, color: null };
    const f = Math.floor(st * 12 + 1e-6);
    const mouthX = x + 10, mouthY = fy - 20 + (h.fly ? -8 : 0);
    const shot = (name, t0, ang, speed, sx, sy, n = 0, life = 0.45) => {
      const u = a - t0;
      if (u < 0 || u > life) return;
      const px = sx + Math.cos(ang) * speed * u, py = sy + Math.sin(ang) * speed * u;
      const [w, hh] = PX.size(name);
      streak(px + w, py + hh, Math.cos(ang), Math.sin(ang), Math.min(10, speed * u), P.cream);
      PX.spr(name, Math.round(px), Math.round(py), Math.floor(u * 12) + n, { k: 2 });
    };
    const recoil = () => { if (f === 2) { pose.dx = -1; pose.sx = 0.9; pose.sy = 1.1; } };
    const hit = () => { if (f === 3) { pose.dx = -2; pose.color = hitFlash; } else if (f === 4) pose.dx = -1; };
    // An enemy spit arcs in from the right and lands the hit at PAY.
    const spitIn = (tx, ty) => {
      const u = (a - 1 / 12) / (PAY - 1 / 12);
      if (u < 0 || u >= 1) return;
      PX.spr('spit', Math.round(tx + 64 * (1 - u)), Math.round(ty - 10 - Math.sin(u * Math.PI) * 16), 0, { k: 2 });
    };
    const impact = (ix, iy) => {
      const u = a - PAY;
      if (u >= 0 && u < 0.12) glint(ix, iy, u < 0.04 ? 7 : 4, u < 0.06 ? P.bone : P.gold);
    };
    switch (h.id) {
      case 'rex': {
        // Spits two embers up and away, then the knockback dash: lunge + shockwave.
        recoil();
        shot('bolt', FIRE, -0.5, 240, mouthX, mouthY - 2);
        shot('bolt', FIRE + 1 / 24, -0.32, 240, mouthX, mouthY);
        PX.burst(a - FIRE, mouthX + 4, mouthY + 2, { n: 6, seed: 3, speed: 40, life: 0.2, colors: [P.gold, P.ember], up: 10 });
        if (f === 3) { pose.dx = 7; pose.sx = 1.22; pose.sy = 0.86; }
        else if (f === 4) { pose.dx = 10; pose.sx = 1.06; pose.sy = 0.95; }
        else if (f >= 5) pose.dx = 9;
        if (a >= PAY) {
          for (let s = 0; s < 3; s++) PX.rect(x - 14 - s * 5, fy - 22 + s * 7, 8 - s, 1, P.cream);
          PX.burst(a - PAY, x - 2, fy - 2, { n: 8, seed: 9, speed: 30, life: 0.3, colors: [P.sand, P.clay], up: 20, gravity: 60 });
          PX.ring(a - PAY, x + 22, fy - 12, { r0: 6, r1: 30, life: 0.3, color: P.cream, color2: P.amber });
        }
        break;
      }
      case 'ptera': {
        // Twin feathers, then a dash that fires a 4-feather volley up and ahead.
        recoil();
        for (const s of [-1, 1]) shot('feather-shot', FIRE, -0.45 + s * 0.12, 240, mouthX, mouthY + 2);
        if (f === 3) { pose.dx = 8; pose.sx = 1.25; pose.sy = 0.85; }
        else if (f === 4) { pose.dx = 11; pose.sx = 1.08; }
        else if (f >= 5) pose.dx = 10;
        if (a >= PAY) {
          for (let s = 0; s < 3; s++) PX.rect(x - 16 - s * 4, fy - 30 + s * 6, 8 - s, 1, P.ice);
          for (let v = 0; v < 4; v++) shot('feather-shot', PAY, -1.15 + v * 0.3, 230, x + 14, fy - 24, v);
        }
        break;
      }
      case 'trike': {
        // Piercing horns, then an enemy spit hits it and the hit makes a blast.
        recoil();
        shot('horn', FIRE, -0.22, 210, mouthX, mouthY + 6);
        shot('horn', FIRE + 1 / 24, -0.22, 210, mouthX - 2, mouthY + 6, 1);
        spitIn(x + 8, fy - 12);
        hit();
        impact(x + 10, fy - 18);
        if (a >= PAY) {
          PX.ring(a - PAY, x, fy - 12, { r0: 6, r1: 34, life: 0.3, color: P.gold, color2: P.ember });
          PX.burst(a - PAY, x, fy - 12, { n: 18, seed: 31, speed: 90, life: 0.35, colors: [P.cream, P.gold, P.ember, P.red], gravity: 120 });
        }
        break;
      }
      case 'raptor': {
        // Three fast claw swipes; the third crits and bleeds.
        for (let s = 0; s < 3; s++) {
          const u = a - (FIRE + s / 24);
          if (u >= 0 && u < 0.14) PX.spr('claw', Math.round(x + 12 + s * 2 + u * 70), fy - 24 + (s % 2) * 6, s, { k: 2 });
        }
        if (f === 2) pose.dx = 3;
        else if (f === 3) { pose.dx = 6; pose.sx = 1.15; pose.sy = 0.9; }
        else if (f >= 4) pose.dx = 4;
        if (a >= PAY) {
          const u = a - PAY;
          PX.line(x + 16, fy - 30, x + 34, fy - 8, u < 0.05 ? P.bone : P.red);
          if (u < 0.35) PX.text('CRIT', x + 20, fy - 40 - Math.round(u * 24), Math.floor(u * 12) % 2 ? P.gold : P.cream);
          PX.burst(u, x + 28, fy - 18, { n: 12, seed: 44, speed: 50, life: 0.4, colors: [P.red, P.blood, P.red], up: 30, gravity: 260 });
        }
        break;
      }
      case 'spino': {
        // Three homing bubbles curve up and pop into a slow.
        recoil();
        for (let b = 0; b < 3; b++) {
          const u = a - (FIRE + b / 24);
          if (u < 0) continue;
          const ex = mouthX + 30 + b * 8, ey = mouthY - 26 + b * 6;
          if (u < SPINO_FLY) {
            const v = u / SPINO_FLY;
            const bx = mouthX + (ex - mouthX) * v, by = mouthY + (ey - mouthY) * v * v + Math.sin(v * Math.PI) * 8 * (b % 2 ? 1 : -1);
            PX.spr('bubble', Math.round(bx), Math.round(by), Math.floor(u * 12), { k: 2 });
          } else {
            PX.burst(u - SPINO_FLY, ex + 6, ey + 6, { n: 8, seed: 50 + b, speed: 40, life: 0.25, colors: [P.ice, P.cyan, P.sky], up: 0, gravity: 0 });
            if (u - SPINO_FLY < 0.1) glint(ex + 6, ey + 6, 3, P.cyan, P.ice);
          }
        }
        break;
      }
      case 'stego': {
        // Five spikes fan out, then a hit fires the ring of twelve.
        const fire = (t0, n, sp, a0, spread) => {
          const u = a - t0;
          if (u < 0 || u > 0.24) return;
          for (let s = 0; s < n; s++) {
            const ang = a0 + (s / n) * spread;
            const r = 8 + sp * u;
            const cx = x + Math.cos(ang) * r, cy = fy - 14 + Math.sin(ang) * r;
            PX.line(cx, cy, cx - Math.cos(ang) * 4, cy - Math.sin(ang) * 4, P.bone);
            PX.px(cx, cy, P.cream);
          }
        };
        recoil();
        fire(FIRE, 5, 130, -Math.PI + 0.35, Math.PI * 1.1);
        spitIn(x + 8, fy - 12);
        hit();
        impact(x + 10, fy - 18);
        if (a >= PAY) { fire(PAY, 12, 120, -Math.PI / 2, Math.PI * 2); PX.ring(a - PAY, x, fy - 14, { r0: 6, r1: 22, life: 0.2, color: P.bone, color2: P.blush }); }
        break;
      }
      case 'pachy': {
        // Pebble shotgun, then a headbutt dash that explodes.
        recoil();
        for (let s = -1; s <= 1; s++) shot('pebble', FIRE, -0.3 + s * 0.16, 230, mouthX, mouthY + 4, s + 1);
        if (f === 3) { pose.dx = 8; pose.sx = 1.22; pose.sy = 0.85; }
        else if (f === 4) { pose.dx = 11; pose.sx = 1.06; pose.sy = 0.95; }
        else if (f >= 5) pose.dx = 10;
        if (a >= PAY) {
          PX.ring(a - PAY, x + 26, fy - 10, { r0: 4, r1: 26, life: 0.3, color: P.cream, color2: P.ember });
          PX.burst(a - PAY, x + 26, fy - 10, { n: 16, seed: 70, speed: 80, life: 0.35, colors: [P.cream, P.gold, P.ember] });
          if (a - PAY < 0.1) glint(x + 26, fy - 10, 6, P.gold);
        }
        break;
      }
    }
    return pose;
  }
  const SPINO_FLY = 0.14; // bubble flight time; first pop at FIRE + SPINO_FLY

  function idleFrame(i, t) {
    // Breathing: the idle frame 1 sits a pixel lower; a short beat of it every ~second.
    const per = 0.9 + PX.hash(i, 77) * 0.4, ph = (PX.step(t, 12) + PX.hash(i, 78) * per) % per;
    return ph < 3 / 12 ? 1 : 0;
  }

  function drawHero(h, i, t, local, camX) {
    const age = t - beatT(popBeat(i));
    if (age < 0) return;
    const st = PX.step(age, 12);
    const x = slotX(i) + camX, fy = feetY(i);
    const f = Math.floor(st * 12 + 1e-6);
    let pose = { dx: 0, dy: 0, sx: 1, sy: 1, color: null, rim: null };
    let acting = false;
    const antic = local - ANTIC_B * BEAT, land = local - FINALE_B * BEAT;
    if (land >= 0) {
      // Finale: unison landing, recoil as they fire, settle.
      const lf = Math.floor(PX.step(land, 12) * 12 + 1e-6);
      if (lf === 0) Object.assign(pose, { sx: 1.32, sy: 0.7, rim: P.bone });
      else if (lf === 1) Object.assign(pose, { dx: -2, sx: 0.88, sy: 1.14 });
      else if (lf === 2) Object.assign(pose, { sx: 1.06, sy: 0.95 });
      acting = true;
    } else if (antic >= 0) {
      // Anticipation on the 16th before the beat: crouch, then up in the air.
      if (antic < 1 / 12) Object.assign(pose, { sx: 1.22, sy: 0.78 });
      else Object.assign(pose, { dy: -7, sx: 0.84, sy: 1.2 });
      acting = true;
    } else if (f === 0) {
      // Pop: in the air stretched.
      Object.assign(pose, { dy: -18, sx: 0.75, sy: 1.3 });
      acting = true;
    } else if (f === 1) {
      Object.assign(pose, { sx: 1.35, sy: 0.7, rim: P.bone });
      acting = true;
    } else if (f <= 6) {
      pose = Object.assign(pose, act(h, i, st, age, x, fy));
      acting = true;
    } else {
      // Idle. Secondary motion: a little hop when a newer hero slams down nearby.
      for (let j = i + 1; j < HEROES.length; j++) {
        const la = PX.step(t - beatT(popBeat(j)) - 1 / 12, 12);
        if (la < 0 || la >= 3 / 12) continue;
        const lf = Math.floor(la * 12 + 1e-6), near = Math.abs(j - i) <= 2 ? 1 : 0.5;
        if (lf === 0) { pose.dy -= Math.round(4 * near); pose.sy *= 1.08; pose.sx *= 0.94; }
        else if (lf === 1) pose.dy -= Math.round(1 * near);
        else { pose.sy *= 0.92; pose.sx *= 1.06; }
      }
    }
    let name = h.id + '-idle', frame = acting ? 0 : idleFrame(i, t);
    if (h.fly) { name = h.id; frame = Math.floor(PX.step(t, 12) * 8) % 2; }
    const hover = h.fly ? -8 + Math.round(Math.sin(PX.step(t, 12) * 7) * 2) : 0;
    // Shadow, landing dust, the sprite.
    PX.rect(x - 10, fy - 1, 20, 2, P.maroon);
    if (f >= 1) PX.burst(age - 1 / 12, x, fy - 1, { n: 10, seed: 100 + i, speed: 45, life: 0.3, colors: [P.sand, P.clay, P.cream], up: 25, gravity: 200, size: 2 });
    if (land >= 0) PX.burst(land, x, fy - 1, { n: 10, seed: 140 + i, speed: 55, life: 0.35, colors: [P.sand, P.clay, P.cream], up: 30, gravity: 200, size: 2 });
    const bx = x + pose.dx, by = fy + pose.dy + hover;
    outlineSpr(name, frame, bx, by, pose.rim || P.ink, { k: K, sx: pose.sx, sy: pose.sy });
    drawSpr(name, frame, bx, by, { k: K, sx: pose.sx, sy: pose.sy, color: pose.color });
    // Landing glint.
    if (land < 0 && (f === 1 || f === 2)) {
      const s2 = f === 1 ? 9 : 5, gx = x + 13, gy = fy - 30 + hover;
      PX.rect(gx - s2, gy, s2 * 2 + 1, 1, P.bone); PX.rect(gx, gy - s2, 1, s2 * 2 + 1, P.bone);
      PX.rect(gx - 1, gy - 1, 3, 3, P.cream);
    }
    // Name under the feet, landing a frame after the hero.
    if (f >= 1) {
      const ny = fy + 5 + (f === 1 ? -2 : 0);
      PX.text(h.name, x, ny, f <= 2 ? P.bone : P.cream, { align: 'center' });
    }
  }

  // Finale volley: every hero fires its own shot on the recoil frame and they converge
  // over the ridge on the 8th, bursting under the title.
  const CONV = [160, 62], V0 = 1 / 12, V1 = BEAT / 2;
  function volley(local, camX) {
    const land = local - FINALE_B * BEAT;
    if (land < V0) return;
    const u = (land - V0) / (V1 - V0);
    if (u < 1) {
      HEROES.forEach((h, i) => {
        const x0 = slotX(i) + camX + 10, y0 = feetY(i) - 20 + (h.fly ? -10 : 0);
        const p = (v) => [x0 + (CONV[0] - x0) * v, y0 + (CONV[1] - y0) * v - Math.sin(v * Math.PI) * 22];
        const [px, py] = p(u), [qx, qy] = p(Math.max(0, u - 0.16)), [rx, ry] = p(Math.max(0, u - 0.08));
        PX.line(qx, qy, px, py, h.col);
        PX.line(rx, ry + 1, px, py + 1, P.cream);
        const [w, hh] = PX.size(h.shot);
        PX.spr(h.shot, Math.round(px - w), Math.round(py - hh), i, { k: 2 });
      });
      return;
    }
    const a = land - V1;
    glow(CONV[0], CONV[1], a < 0.08 ? [[16, P.amber], [11, P.gold], [6, P.cream]] : []);
    if (a < 0.16) for (let r = 0; r < 8; r++) {
      const ang = r * (Math.PI / 4) + 0.2, r0 = 8 + a * 260, r1 = r0 + 14 + (r % 2) * 10;
      PX.line(CONV[0] + Math.cos(ang) * r0, CONV[1] + Math.sin(ang) * r0, CONV[0] + Math.cos(ang) * r1, CONV[1] + Math.sin(ang) * r1, r % 2 ? P.gold : P.cream);
    }
    PX.ring(a, CONV[0], CONV[1], { r0: 8, r1: 80, life: 0.3, color: P.cream, color2: P.gold });
    PX.ring(a - 0.05, CONV[0], CONV[1], { r0: 4, r1: 46, life: 0.25, color: P.gold, color2: P.ember });
    PX.burst(a, CONV[0], CONV[1], { n: 36, seed: 160, speed: 170, life: 0.4, colors: [P.cream, P.gold, P.cyan, P.blush, P.lime], gravity: 90, up: 10 });
    if (a < 0.12) glint(CONV[0], CONV[1], a < 0.04 ? 14 : 8, a < 0.06 ? P.bone : P.cream);
  }

  // Beat-long gameplay flash: integer zoom around the hero in its own clip, punching in
  // one step for the first frame. in: source in-point (s); cx, cy: framing (footage px).
  const CLIP = {
    rex: { in: 0.13, k: 2, cx: 190, cy: 80 },
    ptera: { in: 0.27, k: 2, cx: 170, cy: 98 },
    trike: { in: 0.27, k: 2, cx: 220, cy: 72 },
    raptor: { in: 0.07, k: 2, cx: 160, cy: 89 },
    spino: { in: 0.27, k: 2, cx: 204, cy: 88 },
    stego: { in: 0.53, k: 3, cx: 146, cy: 86 },
    pachy: { in: 0.2, k: 2, cx: 162, cy: 98 },
  };
  function flash(h, t) {
    const age = t - beatT(flashBeat(HEROES.indexOf(h)));
    const c = CLIP[h.id];
    const k = age < 1 / 30 ? c.k + 1 : c.k;
    const cx = PX.clamp(c.cx, W / (2 * k), 320 - W / (2 * k)), cy = PX.clamp(c.cy, H / (2 * k), 180 - H / (2 * k));
    PX.clear(P.ink);
    PX.footage('hero-' + h.id, c.in + age, { k, cx, cy });
    if (age < 1 / 30) PX.dither(0.5, P.bone);
  }

  function drawTitle(local) {
    // Bar 21: the "7" slams on the downbeat, HEROES rolls in, then the letters hop off.
    if (local < 4 * BEAT) {
      const exit = (i) => EXIT_B * BEAT + i * (BEAT / 32);
      const drop = dropFx(local, landAt, { fall: 0.1, height: 56 });
      title(TITLE, 160, TY, {
        k: TK, bands: 'sun', fx: (i) => {
          const a = local - exit(i);
          // The "7" lands on the flash frame: keep its colors so it reads against the flash.
          if (a < 0) return i === 0 && local < 2 / 30 ? Object.assign(drop(i), { flash: null }) : drop(i);
          if (a < 1 / 30) return { sx: 1.2, sy: 0.8 };
          const u = a - 1 / 30, dy = -Math.round(420 * u + 2400 * u * u);
          return dy < -80 ? { vis: false } : { dy, sx: 0.82, sy: 1.25 };
        },
      });
      // The "7" hits the downbeat with a shock ring behind it.
      const sevenX = 160 - PX.titleWidth(TITLE, TK) / 2 + 10;
      PX.ring(local, sevenX, TY + 13, { r0: 6, r1: 46, life: 0.3, color: P.cream, color2: P.gold });
      PX.ring(local - 0.05, sevenX, TY + 13, { r0: 4, r1: 28, life: 0.25, color: P.gold, color2: P.ember });
      for (let i = 0; i < TITLE.length; i++) {
        if (TITLE[i] === ' ') continue;
        const lx = 160 - PX.titleWidth(TITLE, TK) / 2 + (i === 0 ? 0 : PX.titleWidth(TITLE.slice(0, i), TK) + TK) + 10;
        PX.burst(local - landAt(i), lx, TY + 27, { n: i === 0 ? 10 : 4, seed: 200 + i, speed: i === 0 ? 60 : 30, life: 0.3, colors: [P.cream, P.gold], up: 10 });
      }
      return;
    }
    // Finale: the word falls during the anticipation and slams on beat 15, then shines.
    const land = local - FINALE_B * BEAT;
    if (land < -0.1) return;
    const tw = PX.titleWidth(TITLE, TK);
    const sh = land - 2 / 30;
    const shine = sh >= 0 && sh < 0.3 ? 160 - tw / 2 + 10 + sh * 620 : null;
    title(TITLE, 160, TY, { k: TK, bands: 'sun', fx: dropFx(land, () => 0, { fall: 0.1, height: 50 }), shine });
    PX.burst(land, 160 - tw / 2 - 2, TY + 27, { n: 10, seed: 211, speed: 70, life: 0.35, colors: [P.cream, P.gold, P.amber], angle: Math.PI * 1.1, spread: 1.3, up: 20, gravity: 200 });
    PX.burst(land, 160 + tw / 2 + 2, TY + 27, { n: 10, seed: 213, speed: 70, life: 0.35, colors: [P.cream, P.gold, P.amber], angle: -Math.PI * 0.1, spread: 1.3, up: 20, gravity: 200 });
  }

  function drawRoster(t, local) {
    const b = Math.floor(local / BEAT + 1e-6);
    if (local < ANTIC_B * BEAT) {
      const hi = HEROES.findIndex((_, i) => flashBeat(i) === b);
      if (hi >= 0) return flash(HEROES[hi], t);
    }
    const camX = camAt(local);
    const camY = Math.round(38 * (1 - PX.ease.out(PX.clamp(local / (BEAT * 0.95)))));
    // Kicks: the "7" slam, each hero's landing, the finale landing.
    const beatAge = local - b * BEAT;
    let sh = [0, 0];
    if (b === 0) sh = PX.shake(beatAge, 3, 0.3, 40);
    else if (b >= FINALE_B) sh = beatAge < V1 ? PX.shake(beatAge, 4, 0.3, 55) : PX.shake(beatAge - V1, 2.5, 0.2, 57);
    else if (b % 2 === 1) sh = PX.shake(beatAge - 1 / 12, 1.5, 0.15, 40 + b);
    g.save();
    world(t, local, camX, camY);
    g.save(); g.translate(0, camY);
    HEROES.forEach((h, i) => drawHero(h, i, t, local, camX));
    g.restore();
    const land = local - FINALE_B * BEAT;
    if (land >= 0) {
      // Combined shockwave across the ridge from the lineup's landing.
      groundRing(land, 160, 146, { r0: 20, r1: 200, life: 0.35, flat: 0.1, color: P.cream, color2: P.gold });
      volley(local, camX);
    }
    // Hit flashes: the downbeat, each cut back from gameplay, the finale landing.
    if (b === 0 && beatAge < 1 / 30) PX.dither(0.6, P.bone);
    else if (b === 0 && beatAge < 2 / 30) PX.dither(0.3, P.bone);
    else if (b > 1 && b < FINALE_B && b % 2 === 1 && beatAge < 1 / 30) PX.dither(0.5, P.bone);
    else if (land >= 0 && land < 1 / 30) PX.dither(0.5, P.bone);
    else if (land >= 0 && land < 2 / 30) PX.dither(0.2, P.bone);
    drawTitle(local);
    g.restore();
    shakeBuf(sh[0], sh[1]);
  }

  const cues = [];
  cues.push([beatT(0), 'slam', 0.8]);
  for (let i = 2; i < TITLE.length; i++) cues.push([beatT(0) + landAt(i), 'land', i === TITLE.length - 1 ? 0.3 : 0.4]);
  const actSfx = { rex: 'dash', ptera: 'whoosh', trike: 'boom', raptor: 'dash', spino: 'blip', stego: 'boom', pachy: 'boom' };
  HEROES.forEach((h, i) => {
    cues.push([beatT(popBeat(i)), 'pop', 0.9]);
    cues.push([beatT(popBeat(i)) + 1 / 12, 'land', 0.5]);
    const at = h.id === 'spino' ? FIRE + SPINO_FLY : PAY;
    cues.push([beatT(popBeat(i)) + at, actSfx[h.id], 0.45]);
    cues.push([beatT(flashBeat(i)), 'whoosh', 0.35]);
  });
  cues.push([beatT(ANTIC_B), 'whoosh', 0.4]);
  cues.push([beatT(FINALE_B), 'slam', 0.9]);
  cues.push([beatT(FINALE_B) + 2 / 30, 'shine', 0.7]);
  cues.push([beatT(FINALE_B) + V1, 'boom', 0.6]);

  PX.scene({ id: 'roster-heroes', start: T0, end: T1, layer: 0, draw: drawRoster, cues });
})();
