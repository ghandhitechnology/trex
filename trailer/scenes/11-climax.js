// 11-climax: bars 29-32 (late-game chaos, riser into 60.0 s) and bars 33-34
// (breakdown: bones count up, locks flip, the Raptor unlocks). Uses the GPX kit
// from 10-gameplay.js. Every frame is a pure function of t.
(function () {
  const { W, H, P, BEAT } = PX;
  const X = window.GPX;
  const B = X.B, clamp = PX.clamp, hash = PX.hash;
  const shot = X.shot;
  const TK = (k, o = {}) => Object.assign({ k, track: true }, o);

  // ================================================================ bars 29-32
  // chaos: 14:12, ~9800 kills. The active items fire a huge nova on frame 17. From
  // frame 148 Rex takes a hit (red flash, then he blinks white), so the edit stays before it:
  // real time for bar 29, the numbers at 0.75x, slow motion under ENDLESS.
  const NOVA = 17 / 30, S30 = NOVA + PX.BAR, S31 = S30 + PX.BAR * 0.75;
  const CC = X.clock([
    [B(29), NOVA],
    [B(30), S30, 0.75],
    [B(31), S31, 0], // hit-stop under ENDLESS
    [B(31) + 0.2, S31, 0.62],
  ]);
  // Bar 32, the riser: the fight replays tight on Rex (the HUD stays out of frame),
  // always forward, ramping from 1x to 3x into the white.
  const ACC = 2 / PX.BAR;
  const RC = (t) => { const a = Math.max(0, t - B(32)); return 1.0 + a + 0.5 * ACC * a * a; };
  // HUD close-ups. The kill count sits over the arena wall, so frame it inside the footage, not the floor.
  const TIMER = { k: 4, cx: 160, cy: 20 }, KILLS = { k: 4, cx: 278, cy: 20, frame: true };
  const CH = 'chaos';
  const shots = [
    // Bar 29 (crash + impact): wide on the nova, then quarter-note punches.
    shot(B(29), CH, CC, {}, { type: 'flash' }),
    shot(B(29, 1), CH, CC, TK(2, { punch: true })),
    shot(B(29, 2), CH, CC, {}, { type: 'glitch' }),
    shot(B(29, 3), CH, CC, TK(3)),
    // Bar 30: the numbers. The clock, the kill count, back out.
    shot(B(30), CH, CC, TIMER, { type: 'whip', dir: 'd' }),
    shot(B(30, 1), CH, CC, KILLS, { type: 'cut', flash: true, color: P.gold }),
    shot(B(30, 2), CH, CC, {}, { type: 'blocks', cell: 16 }),
    shot(B(30, 3), CH, CC, TK(2)),
    // Bar 31 (crash): ENDLESS, then eighth-note cuts in and out.
    shot(B(31), CH, CC, {}, { type: 'flash', color: P.cream }),
    ...[TK(2), {}, TK(3), {}, TK(2, { tx: -30 }), TK(3), TK(2)].map((o, i) =>
      shot(B(31, 0.5 + i * 0.5), CH, CC, o, { type: i % 3 === 2 ? 'glitch' : 'cut' })),
    // Bar 32: the zoom climbs a step per beat, cuts go to 16ths on the last beat.
    shot(B(32), CH, RC, TK(3), { type: 'cut', flash: true }),
    shot(B(32, 1), CH, RC, TK(2, { tx: 24 }), { type: 'glitch' }),
    shot(B(32, 2), CH, RC, TK(3, { tx: -16 })),
    shot(B(32, 2.5), CH, RC, TK(2)),
    ...[TK(4), TK(3, { tx: -20 }), TK(4), TK(5)].map((o, i) => shot(B(32, 3 + i * 0.25), CH, RC, o, { type: i % 2 ? 'glitch' : 'cut' })),
  ];

  const HITS = [
    [B(29), 6, 0.45], [B(29, 2), 3], [B(30), 3], [B(30, 1), 3], [B(30, 2), 2],
    [B(31), 6, 0.45], [B(31, 1), 3], [B(31, 2), 4], [B(31, 3), 3],
    [B(32), 2], [B(32, 1), 3], [B(32, 2), 3], [B(32, 3), 4],
  ];
  const RISE = B(32, 3); // the white-out starts on the last beat

  function chaosDraw(t) {
    X.playReel(shots, t);
    const a29 = t - B(29), a31 = t - B(31);
    if (a29 < 2 / 30) X.grade(X.RAMP.sun, { gain: 1.8 });
    // ENDLESS: bone-graded hit frames, the word stamps on the crash low in frame and holds the bar.
    if (a31 >= 0 && a31 < 2 / 30) X.grade(X.RAMP.bone, { gain: 1.9 });
    if (t >= B(31) - 0.1 && t < B(32)) {
      X.callout('ENDLESS', t, B(31), { k: 4, y: 110, bands: 'white', style: 'stamp', out: B(32) - 0.1, exit: 'squash', sparkCols: [P.bone, P.gold, P.fog] });
    }
    // Shake: the hits, plus a tremor that grows through the riser.
    let [dx, dy] = X.shakeAt(t, HITS);
    if (t >= B(32)) { const amp = 1 + (t - B(32)) * 2.2, f = Math.floor(t * 30); dx += Math.round((hash(f, 5) - 0.5) * 2 * amp); dy += Math.round((hash(f, 6) - 0.5) * 2 * amp); }
    X.shakeFrame(dx, dy);
    // Riser: white dither over the last beat only (after the shake), full white on 60.0.
    const r = (t - RISE) / BEAT;
    if (r > 0) PX.dither(Math.floor(clamp(0.9 * r * r) * 16) / 16, P.bone);
    if (t > B(33) - 1 / 30) PX.clear(P.bone);
  }

  // ================================================================ bars 33-34
  // Hub stills, re-composed at 1x so overlays sit exactly on the game's UI.
  const still = (id) => PX.footageFrame(id, 0);
  // Draw a 1x composition (still + fn) scaled by k around (cx, cy), kept inside the frame.
  function stage(id, k, cx, cy, fn) {
    const L = X.grab('stage', () => {
      const im = still(id);
      if (im) PX.g.drawImage(im, 0, 0);
      if (fn) fn();
    });
    PX.clear(P.ink);
    const x = clamp(Math.round(W / 2 - cx * k), W - W * k, 0), y = clamp(Math.round(H / 2 - cy * k), H - H * k, 0);
    PX.g.drawImage(L, x, y, W * k, H * k);
    return (sx, sy) => [x + sx * k, y + sy * k];
  }
  // 4-point sparkle.
  function sparkle(x, y, r, c) {
    PX.rect(x - r, y, r * 2 + 1, 1, c); PX.rect(x, y - r, 1, r * 2 + 1, c);
    if (r > 2) PX.rect(x - 1, y - 1, 3, 3, c);
  }
  // Soft ink pool (ordered dither ellipse) that seats text over busy UI.
  function pool(cx, cy, rx, ry, a) {
    const x0 = Math.max(0, Math.floor(cx - rx)), x1 = Math.min(W, Math.ceil(cx + rx)), y0 = Math.max(0, Math.floor(cy - ry)), y1 = Math.min(H, Math.ceil(cy + ry));
    PX.g.fillStyle = P.ink;
    for (let y = y0; y < y1; y++) for (let x = x0; x < x1; x++) {
      const nx = (x - cx) / rx, ny = (y - cy) / ry, d = nx * nx + ny * ny;
      if (d < 1 && PX.bayer(x, y) < a * clamp((1 - d) * 3)) PX.g.fillRect(x, y, 1, 1);
    }
  }
  // Unlock flip of a locked 20x20 tile at 1x (top-left x, y), item sprite `name`, hit time h.
  // sel: the game's cursor sits on this tile (gold rim, one row taller).
  function tileFlip(t, x, y, name, h, sel) {
    const a = PX.step(t - h, 30);
    if (a < -0.12) return;
    if (a < 0) { // anticipation: the lock rattles
      const j = Math.floor(-a * 60) % 2 ? 1 : -1;
      PX.spr('lock', x + 13 + j, y + 11, 0);
      return;
    }
    // The tile, repainted open: night interior, slate (or cursor gold) rim, the item in full color.
    if (sel) PX.rect(x, y - 1, 20, 1, P.gold);
    PX.rect(x, y, 20, 20, sel ? P.gold : P.slate); PX.rect(x + 1, y + 1, 18, 18, P.night);
    PX.rect(x + 20, y + 11, 3, 11, P.ink); PX.rect(x + 13, y + 20, 10, 2, P.ink);
    if (a < 1 / 30) { PX.rect(x + 1, y + 1, 18, 18, P.bone); return; }
    const sq = a < 2 / 30 ? [1.35, 0.65] : a < 3 / 30 ? [0.85, 1.2] : a < 4 / 30 ? [1.06, 0.95] : [1, 1];
    X.item(name, x + 2, y + 2, 1, { sx: sq[0], sy: sq[1] });
    if (a < 0.25) { PX.rect(x, y, 20, 1, P.gold); PX.rect(x, y + 19, 20, 1, P.gold); PX.rect(x, y, 1, 20, P.gold); PX.rect(x + 19, y, 1, 20, P.gold); }
  }
  // A lock popping off (screen space): it flies up out of frame, tumbling.
  function lockFly(t, h, sx, sy, k, seed) {
    const a = t - h;
    if (a < 0 || a > 0.8) return;
    const vx = (hash(seed, 3) < 0.5 ? -1 : 1) * (50 + hash(seed, 4) * 40), vy = -420;
    const x = sx + vx * a, y = sy + vy * a + 300 * a * a;
    if (y < -20 || y > H + 20) return;
    PX.spr('lock', x, y, 0, { k, flip: Math.floor(a * 12) % 2 === 1 });
  }

  const SHOP = 'hub-shop', ITEMS = 'hub-items', LOCKED = 'hub-locked';
  // Item row 1 of the shop (hub-shop/hub-items): 20x20 tiles at y = 62, 23 px pitch.
  // Locked items in content order: railgun, laser eye, meteor call, quill burst...
  const TILE = (i) => [24 + 23 * i, 62];
  const FLIPS = [[1, 'laser-eye', B(34, 0.25)], [2, 'meteor-call', B(34, 0.75)], [3, 'quill-burst', B(34, 1.25)]];
  const BONES = 400; // the Raptor's unlock price (hub-locked: "NEED 400")
  const COUNT_AT = [0, 1, 2, 3, 4, 5, 6, 7].map((i) => B(33, i * 0.5));
  const RAPTOR_IN = B(34, 1.5), UNLOCK = B(34, 2), LAND = B(34, 3);
  const RAPTOR = { x: 46, y: 47 }; // the 32x32 hero portrait in hub-locked (footage px)
  const CTR = { x: W / 2, y: 76 }; // the bone counter, screen px

  function boneCounter(t) {
    const i = COUNT_AT.filter((h) => t >= h).length; // steps landed
    if (i === 0) return;
    const n = Math.round(BONES * i / COUNT_AT.length);
    const h = COUNT_AT[i - 1], a = t - h;
    const str = String(n);
    const k = 3;
    const tw = PX.titleWidth(str, k), iw = 9 * k + 8;
    const x0 = Math.round(CTR.x - (tw + iw) / 2), y = CTR.y;
    const pop = a < 1 / 30 ? -3 : a < 2 / 30 ? 1 : 0;
    const bone = PX.sprite('bone');
    if (bone) PX.g.drawImage(X.outlined(bone, k), x0 - 1, y + 6 + pop - 1);
    PX.title(str, x0 + iw, y + pop, { k, bands: i === COUNT_AT.length ? 'sun' : 'bone', align: 'left', depth: k + 1,
      fx: () => (a < 1 / 30 ? { flash: P.cream } : {}), shine: a < 0.3 ? Math.round(x0 + iw + y + a * 900 - 40) : null });
    PX.burst(a, x0 + iw + tw, y + 14, { n: 8, seed: 90 + i, speed: 60, life: 0.35, colors: [P.bone, P.gold, P.fog] });
  }
  // Bones arc in from the sides into the counter, one per 16th (stateless arcs).
  function boneStream(t) {
    const bone = PX.sprite('bone');
    if (!bone) return;
    const c = X.outlined(bone, 2);
    for (let i = 0; i < 16; i++) {
      const land = B(33, i * 0.25) + 0.04;
      const a = t - (land - 0.45);
      if (a < 0 || a > 0.45) continue;
      const u = a / 0.45;
      const sx = hash(i, 51) < 0.5 ? -10 : W + 10, sy = 120 + hash(i, 52) * 50;
      const ex = CTR.x - 30 + (hash(i, 53) - 0.5) * 16, ey = CTR.y + 14;
      const x = PX.lerp(sx, ex, PX.ease.inOut(u)), y = PX.lerp(sy, ey, u) - Math.sin(u * Math.PI) * 50;
      PX.g.drawImage(c, Math.round(x - c.width / 2), Math.round(y - c.height / 2));
    }
  }
  // Motes drifting up through the hub (calm motion under the breakdown).
  function motes(t) {
    for (let i = 0; i < 26; i++) {
      const sp = 6 + hash(i, 61) * 10;
      const y = ((hash(i, 62) * (H + 20) - t * sp) % (H + 20) + H + 20) % (H + 20) - 10;
      const x = hash(i, 63) * W + Math.sin(t * 1.3 + i) * 4;
      PX.px(x, y, Math.sin(t * 5 + i * 1.7) > 0.3 ? P.gold : P.haze);
    }
  }

  function shopShot(t) {
    // Bar 33: pan the shop's three tile rows while bones pour into the counter.
    const a = t - B(33);
    const u = PX.ease.inOut(clamp(a / PX.BAR));
    stage(SHOP, 2, Math.round(PX.lerp(104, 216, u)), 68);
    motes(t);
    pool(CTR.x, CTR.y + 15, 84, 26, 1.02);
    boneStream(t);
    boneCounter(t);
    if (a < 0.4) PX.dither(1 - a / 0.4, P.bone); // in from the riser's white
  }
  function itemsShot(t) {
    // Bar 34, beats 0-1: three items unlock, one per 8th.
    const k = 3, map = stage(ITEMS, k, 80, 88, () => { for (const [i, name, h] of FLIPS) tileFlip(t, TILE(i)[0], TILE(i)[1], name, h, i === 1); });
    FLIPS.forEach(([i, , h], j) => {
      const [tx, ty] = TILE(i);
      const [sx, sy] = map(tx + 10, ty + 10);
      const a = t - h;
      lockFly(t, h, sx + 9, sy + 6, 2, j);
      if (a >= 0 && a < 0.5) {
        PX.ring(a, sx, sy, { r0: 14, r1: 44, life: 0.3, color: P.cream, color2: P.gold });
        for (let s = 0; s < 4; s++) {
          const ang = s * Math.PI / 2 + 0.6 + j, d = 20 + a * 70;
          if (a < 0.35) sparkle(Math.round(sx + Math.cos(ang) * d), Math.round(sy + Math.sin(ang) * d), a < 0.12 ? 3 : 2, s % 2 ? P.gold : P.bone);
        }
      }
    });
    motes(t);
  }
  // The Raptor's portrait at 3x: the hero text and stats are painted over with floor.
  function raptorShot(t) {
    const k = 3, cx = 62, cy = 64 - Math.round((t - RAPTOR_IN) * 3);
    const a = t - UNLOCK;
    const fr = Math.floor(PX.step(t, 4) * 4) % 2;
    const map = stage(LOCKED, k, cx, cy, () => {
      const im = still(LOCKED);
      PX.g.drawImage(im, 84, 100, 40, 60, 84, 30, 40, 60); // floor over the name, blurb and stats
      PX.g.drawImage(im, 84, 100, 40, 12, 84, 88, 40, 12);
      if (a >= 0) PX.g.drawImage(im, 6, 45, 34, 32, 45, 45, 34, 32); // floor over the silhouette
    });
    const [bx, by] = map(RAPTOR.x + 16, RAPTOR.y + 32); // feet, screen px
    if (a < 0) {
      // Locked: a padlock on the silhouette rattles on the last 8th.
      const r = t - (UNLOCK - BEAT / 2);
      const j = r > 0 ? (Math.floor(r * 30) % 2 ? 2 : -2) : 0;
      PX.spr('lock', bx - 8 + j, by - 58, 0, { k: 2 });
    } else {
      // Hop: squash, jump, land exactly on beat 3.
      const h = t - (LAND - 0.32);
      const hop = h < 0 || h >= 0.4 ? { dy: 0, sx: 1, sy: 1 }
        : h < 0.06 ? { dy: 0, sx: 1.2, sy: 0.8 }
          : h < 0.32 ? { dy: -Math.round(Math.sin((h - 0.06) / 0.26 * Math.PI) * 10), sx: 0.88, sy: 1.15 }
            : { dy: 0, sx: 1.25, sy: 0.75 };
      const im = PX.sprite('raptor-idle', fr);
      if (im) {
        const kk = 2 * k;
        const w = Math.round(16 * kk * hop.sx), hh = Math.round(16 * kk * hop.sy);
        const x0 = bx - Math.round(w / 2), y0 = by - hh + hop.dy * k;
        if (a < 2 / 30) PX.circle(bx, by - 48, a < 1 / 30 ? 60 : 76, a < 1 / 30 ? P.bone : P.cream, a < 1 / 30);
        if (a < 1 / 30) PX.g.drawImage(X.flat(im, kk, P.bone), x0, y0, w, hh);
        else if (a < 2 / 30) PX.g.drawImage(X.flat(im, kk, P.cream), x0, y0, w, hh);
        else PX.g.drawImage(X.outlined(im, kk, P.ink), x0 - 1, y0 - 1, w + 2, hh + 2);
        // Blink on the half beat.
        const ex = x0 + Math.round(w * 0.72), ey = y0 + Math.round(hh * 0.28);
        if (a > BEAT * 0.5 && a < BEAT * 0.5 + 0.08) PX.rect(ex - 3, ey - 1, 9, 4, P.ink);
      }
      PX.ring(a, bx, by - 46, { r0: 16, r1: 120, life: 0.45, color: P.cream, color2: P.gold });
      PX.burst(a, bx, by - 46, { n: 28, seed: 131, speed: 150, life: 0.6, colors: [P.cream, P.gold, P.bone, P.lime] });
      for (let s = 0; s < 6; s++) {
        const ang = s * Math.PI / 3 + 0.3, d = 50 + Math.min(a, 0.5) * 60;
        if (a < 0.6 && Math.floor(t * 12 + s) % 3) sparkle(Math.round(bx + Math.cos(ang) * d), Math.round(by - 46 + Math.sin(ang) * d * 0.7), 3, s % 2 ? P.gold : P.bone);
      }
      lockFly(t, UNLOCK, bx - 8, by - 58, 2, 9);
      // Landing dust.
      const la = t - LAND;
      PX.burst(la, bx - 16, by - 2, { n: 7, seed: 141, speed: 50, life: 0.3, colors: [P.fog, P.haze], angle: Math.PI, spread: 0.8, up: 10 });
      PX.burst(la, bx + 16, by - 2, { n: 7, seed: 142, speed: 50, life: 0.3, colors: [P.fog, P.haze], angle: 0, spread: 0.8, up: 10 });
    }
    motes(t);
  }
  function metaDraw(t) {
    if (t < B(34)) shopShot(t);
    else if (t < RAPTOR_IN) itemsShot(t);
    else raptorShot(t);
    // Hard cuts, each with a flash frame.
    for (const c of [B(34), RAPTOR_IN]) X.flash(t - c, c === B(34) ? P.bone : P.cream);
    X.shakeFrame(...X.shakeAt(t, [[B(34), 2, 0.2], [UNLOCK, 3, 0.3], [LAND, 2, 0.15]]));
  }

  const cues = [
    [B(29), 'impact', 0.9], [B(29), 'boom', 0.6],
    [B(30) - 0.1, 'whoosh', 0.45], [B(30, 1), 'blip', 0.5],
    [B(31), 'slam', 0.85], [B(31), 'impact', 0.6],
    [B(31, 2), 'boom', 0.5],
    [B(32), 'riser', 0.7],
    [B(32, 3), 'glitch', 0.4],
  ];
  const metaCues = [
    [B(33), 'impact', 0.35], [B(33), 'shine', 0.5],
    ...COUNT_AT.map((h, i) => [h, 'bone', i % 2 ? 0.45 : 0.65]),
    ...FLIPS.map(([, , h]) => [h, 'pop', 0.6]),
    [RAPTOR_IN, 'blip', 0.4],
    [UNLOCK, 'shine', 0.7], [UNLOCK, 'pop', 0.6],
    [LAND, 'land', 0.45],
  ];

  PX.scene({ id: 'gameplay-climax', start: B(29), end: B(33), layer: 10, cues, draw: chaosDraw });
  PX.scene({ id: 'gameplay-meta', start: B(33), end: B(35), layer: 10, cues: metaCues, draw: metaDraw });
})();
