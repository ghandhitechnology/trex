// 30-outro: bars 35-40 (63.75-75 s).
// The camera pulls back out of the hub screen the meta beat ended on: it was the trex
// pane all along, in the same tmux screen. A run starts; Clawd's agent finishes, Clawd
// winds up and slams the tmux key, focus jumps to its pane and the game pauses on the
// bar-37 downbeat. Rex climbs out from behind the PAUSED panel, turns to the camera and
// waves; Clawd waves back. Bars 39-40: the end card, a living title with both of them.
// Timing data and cues sit at the top as plain math (tools/cues.mjs reads them).
(function () {
  const BEAT = PX.BEAT, S = BEAT / 4;
  const B = (n, b = 0) => PX.bar(n, b);

  // ------------------------------------------------------------ timing
  const T0 = B(35), T_OUT1 = B(35, 0.5), T_OUT2 = B(35, 1), T_RUN = B(35, 2);
  const T_DONE = B(36), T_LOOKOVER = B(36, 2), T_POV = B(36, 2.5), T_RAISE = B(36, 3), T_PAUSE = B(37);
  const T_PANE = B(37, 1), T_UP = B(37, 2), T_LAND = B(37, 2.5), T_TURN = B(37, 3);
  const T_WAVE = B(38), T_BACKWAVE = B(38, 2), T_PUSH = B(38, 3), T_PUSH2 = B(38, 3.5);
  const T_CARD = B(39), T_HIT = B(40), T_FADE = B(40, 3), T_END = 75;
  const RUN_END = 179 / 30; // pane-run's last frame (0:39) is the one that pauses
  const LETTERS = [0, 1, 2, 3].map((n) => B(39, n * 0.5));
  const T_REX_ON = B(39, 2), T_CLAWD_ON = B(39, 2.5);
  const REPO = 'github.com/ghandhitechnology/trex';
  const CHUNKS = [6, 11, 18, 28, 33]; // github | .com/ | ghandhi | technology | /trex
  const TYPE_KEYS = CHUNKS.map((_, i) => T_CLAWD_ON + (i + 1) * S);

  // ------------------------------------------------------------ cues
  const cues = [];
  cues.push([T0, 'whoosh', 0.6], [T_OUT1, 'whoosh', 0.5], [T_OUT2, 'whoosh', 0.7]);
  cues.push([T_RUN, 'select', 0.6], [B(35, 3), 'type', 0.3]);
  cues.push([T_DONE, 'done', 1], [T_DONE, 'type', 0.3], [B(36, 0.5), 'type', 0.3], [B(36, 1), 'type', 0.25]);
  cues.push([T_LOOKOVER, 'blip', 0.45], [T_RAISE, 'whoosh', 0.45]);
  cues.push([T_PAUSE, 'key', 1], [T_PAUSE, 'pause', 1]);
  cues.push([T_PANE, 'pop', 0.6], [T_UP, 'whoosh', 0.35], [T_LAND, 'land', 0.6], [T_TURN, 'blip', 0.5]);
  cues.push([T_WAVE, 'pop', 0.7], [T_BACKWAVE, 'pop', 0.6], [T_PUSH, 'whoosh', 0.6], [T_PUSH2, 'whoosh', 0.8]);
  LETTERS.forEach((t) => cues.push([t, 'land', 0.8]));
  cues.push([T_REX_ON, 'pop', 0.7], [T_CLAWD_ON, 'pop', 0.7]);
  TYPE_KEYS.forEach((t, i) => cues.push([t, 'key', 0.45 + i * 0.07]));
  cues.push([T_HIT, 'slam', 1], [T_HIT, 'impact', 1], [T_HIT, 'shine', 0.8]);

  // ------------------------------------------------------------ helpers
  const W = PX.W, H = PX.H, clamp = PX.clamp;
  const CLAWD_X = 70;
  const q = (t, t0) => PX.step(t - t0); // stepped age since t0 (12 fps)
  const onBeat = (t, from) => t >= from && PX.step((((t - from) % BEAT) + BEAT) % BEAT) < 2 / 12;

  // ------------------------------------------------------------ Claude Code
  function logLines(t) {
    const P = PX.P;
    const L = [
      { runs: [['⏺ ', P.lime], ['Update', P.bone], ['(ledger.rs)', P.haze]] },
      { runs: [['  ⎿  ', P.mauve], ['1 added, 1 removed', P.haze]] },
      {},
      { runs: [['⏺ ', P.lime], ['Bash', P.bone], ['(cargo test)', P.haze]] },
      { runs: [['  ⎿  ', P.mauve], ['412 passed ', P.lime], ['✓', P.lime]] },
      {},
      { runs: [['⏺ ', P.bone], ['Fixed. All green.', P.fog]] },
      {},
      { runs: [['> ', P.haze], ['ship it', P.bone]], bg: P.dusk },
      {},
    ];
    const done = t >= T_DONE;
    const dot = done ? P.lime : Math.floor(t * 4) % 2 ? P.mauve : P.haze;
    L.push({ runs: [['⏺ ', dot], ['Bash', P.bone], ['(git push)', P.haze]] });
    if (t >= B(35, 3) && !done) L.push({ runs: [['  ⎿  ', P.mauve], ['Writing objects', P.haze]] });
    if (done) L.push({ runs: [['  ⎿  ', P.mauve], ['main -> main', P.haze]] });
    if (t >= B(36, 0.5)) L.push({}, { runs: [['⏺ ', P.bone], ['Pushed. PR #214', P.fog]] });
    if (t >= B(36, 1)) L.push({ runs: [['  is up.', P.fog]] }, {}, { runs: [['✻ Baked for 72s', P.mauve]] });
    return L;
  }
  function agentState(t) {
    const done = t >= T_DONE;
    return {
      lines: logLines(t),
      input: '',
      cursor: done ? Math.floor((t - T_DONE) / 0.5) % 2 === 0 : true,
      spin: done ? null : { i: Math.floor(t / 0.12), verb: 'Pushing', sub: '71s' },
      hint: done ? '? for shortcuts' : 'esc to interrupt',
    };
  }
  const busy = (tt) => (tt < T_DONE ? 0.7 : 0.05);

  // ------------------------------------------------------------ Clawd
  function clawdState(t) {
    const s = { arms: ['rest', 'rest'], eyes: 'open', look: [0, 0], lean: -1, nod: 0 };
    const bl = PX.pulse(t, BEAT, 7);
    s.beat = bl > 0.55 ? (bl > 0.85 ? 2 : 1) : 0;
    if (t < T_DONE) {
      // The agent pushes on its own: Clawd sits back and nods to the beat.
      s.nod = onBeat(t, T0) ? 1 : 0;
      s.eyes = 'half';
      if (t >= 64.9 && t < 65.0) s.eyes = 'blink';
      return s;
    }
    const a = q(t, T_DONE);
    s.lean = 0;
    if (a < 4 / 12) { s.arms = ['up', 'up']; s.squash = -1; s.hop = a < 3 / 12 ? 2 : 1; s.eyes = 'happy'; s.beat = 2; return s; }
    if (t < T_LOOKOVER) { s.eyes = 'happy'; s.nod = onBeat(t, T_DONE) ? 1 : 0; return s; }
    if (t < T_RAISE) {
      // Notices the game still running next door.
      const f = q(t, T_LOOKOVER);
      s.look = [1, 0]; s.eyes = f < 2 / 12 ? 'open' : 'half';
      return s;
    }
    if (t < T_PAUSE) {
      // Winds up: paw high, body stretched.
      const f = q(t, T_RAISE);
      s.look = [1, 0]; s.eyes = 'half'; s.arms = ['rest', 'up']; s.squash = f < 2 / 12 ? 0 : -1;
      return s;
    }
    const p = q(t, T_PAUSE);
    if (p < 2 / 12) { s.arms = ['rest', 'slam']; s.squash = 1; s.eyes = 'shut'; return s; }
    s.look = [1, 0];
    s.eyes = p < 5 / 12 ? 'happy' : 'open';
    if (t >= T_BACKWAVE) {
      const f = Math.floor(PX.step(t - T_BACKWAVE, 10) * 10) % 4;
      s.arms = ['rest', ['wave0', 'wave1', 'wave2', 'wave1'][f]];
      s.eyes = 'happy'; s.look = [1, -1];
    }
    for (const b of [68.25, 69.95]) if (t >= b && t < b + 0.1) s.eyes = 'blink';
    return s;
  }

  // ------------------------------------------------------------ Rex
  // He froze behind the PAUSED panel at pane x 147; he climbs out onto its top edge.
  const EDGE_Y = 78, REX_X = 243;
  function rexState(t) {
    if (t < T_PANE) return null;
    const a = q(t, T_PANE);
    if (t < T_UP) {
      const rise = a < 1 / 12 ? 8 : 15;
      const look = a < 3 / 12 ? { flip: false, eyes: 'wide' } : a < 5 / 12 ? { flip: true, eyes: 'fwd' } : { flip: false, eyes: 'fwd' };
      return Object.assign({ pose: 'idle0', x: REX_X, y: EDGE_Y + 30 - rise, clip: true }, look);
    }
    if (t < T_LAND) {
      const u = clamp(q(t, T_UP) / (T_LAND - T_UP));
      if (u < 0.2) return { pose: 'crouch', x: REX_X, y: EDGE_Y + 18, clip: true };
      return { pose: 'air', x: REX_X, y: Math.round(EDGE_Y - Math.sin(u * Math.PI) * 18 + (1 - u) * 16), clip: u < 0.6 };
    }
    if (t < T_LAND + 1 / 12) return { pose: 'crouch', x: REX_X, y: EDGE_Y };
    if (t < T_TURN) return { pose: 'idle0', x: REX_X, y: EDGE_Y, eyes: 'fwd' };
    if (t < T_WAVE) {
      // The turn, on camera: side, mid-turn, 3/4.
      const f = q(t, T_TURN);
      if (f < 1 / 12) return { pose: 'idle0', x: REX_X, y: EDGE_Y, eyes: 'back' };
      if (f < 2 / 12) return { pose: 'turn', x: REX_X, y: EDGE_Y };
      return { pose: 'q', x: REX_X, y: EDGE_Y, eyes: 'open' };
    }
    if (t < T_PUSH) {
      // Anticipation dip, then the wave swings out, up, in, up.
      const f = q(t, T_WAVE);
      if (f < 2 / 12) return { pose: 'q', arm: 'dip', grin: true, x: REX_X, y: EDGE_Y };
      const w = Math.floor(PX.step(t - T_WAVE - 2 / 12, 10) * 10) % 4;
      return { pose: 'q', arm: ['A', 'B', 'C', 'B'][w], grin: true, x: REX_X, y: EDGE_Y };
    }
    const f = q(t, T_PUSH2);
    return { pose: 'q', grin: true, eyes: t >= T_PUSH2 && f >= 1 / 12 && f < 5 / 12 ? 'wink' : 'open', x: REX_X, y: EDGE_Y };
  }
  function drawRex(r) {
    const T = window.TMX, Lo = T.layout();
    const draw = () => T.rex(r.pose, r.x, r.y, { flip: r.flip, eyes: r.eyes, grin: r.grin, arm: r.arm });
    if (r.clip) T.clip(Lo.gx, Lo.gy, Lo.gw, EDGE_Y - Lo.gy, draw);
    else draw();
  }

  // ------------------------------------------------------------ the game pane
  // The hub as the meta beat left it: the Raptor just unlocked (its silhouette and lock
  // painted over with the real sprite).
  function drawHub(t, Lo) {
    const g = PX.g, P = PX.P;
    const im = PX.images['outro:hub'];
    if (!im || !im.width) return;
    const x0 = Lo.gx, y0 = Lo.gy;
    g.drawImage(im, x0, y0);
    // The big preview: the floor from below over the silhouette, then the idling sprite.
    g.drawImage(im, 10, 79, 36, 31, x0 + 10, y0 + 45, 36, 31);
    const fr = Math.floor(PX.step(t, 4) * 4) % 2;
    PX.spr('raptor-idle', x0 + 12, y0 + 47, fr, { k: 2 });
    // Its card: clear the lock, redraw the gold frame, the sprite at 1x.
    PX.rect(x0 + 103, y0 + 18, 18, 17, P.night);
    PX.rect(x0 + 102, y0 + 35, 20, 1, P.gold);
    PX.rect(x0 + 121, y0 + 16, 1, 20, P.gold);
    PX.spr('raptor-idle', x0 + 104, y0 + 18, fr, { k: 1 });
  }
  const PANEL = [52, 32, 121, 63]; // the PAUSED box in pane-paused (x, y, w, h)
  function drawGame(t, Lo) {
    const g = PX.g, P = PX.P;
    if (t < T_RUN) return drawHub(t, Lo);
    if (t < T_PAUSE) {
      const img = PX.footageFrame('pane-run', RUN_END - (T_PAUSE - t));
      if (img) g.drawImage(img, Lo.gx, Lo.gy);
      if (t - T_RUN < 1 / 30) PX.rect(Lo.gx, Lo.gy, Lo.gw, Lo.gh, P.bone);
      return;
    }
    // Paused: the last frame sags as the tape stops, with a few torn scanlines. The
    // PAUSED box goes on top after the pane dims (drawPanel).
    const a = t - T_PAUSE;
    const img = PX.footageFrame('pane-run', RUN_END);
    if (!img) return;
    const sag = a < 0.3 ? Math.round(PX.ease.out(a / 0.3) * 3) : 3;
    g.drawImage(img, Lo.gx, Lo.gy + sag);
    if (a < 0.12) {
      for (let i = 0; i < 6; i++) {
        const f = Math.floor(a * 30);
        const y = Lo.gy + Math.floor(PX.hash(i, 40 + f) * Lo.gh), h = 3 + Math.floor(PX.hash(i, 5) * 8);
        const dx = Math.round((PX.hash(i, 9 + f) - 0.5) * 18);
        g.drawImage(img, 0, y - Lo.gy, Lo.gw, h, Lo.gx + dx, y + sag, Lo.gw, h);
      }
    }
  }
  // The PAUSED box lands as a hit over the dimmed pane: a bone flash a size up, a squash,
  // a stretch, then it sits.
  function drawPanel(t, Lo) {
    const g = PX.g, P = PX.P;
    const a = t - T_PAUSE;
    if (a < 0) return;
    const im = PX.footageFrame('pane-paused', 0);
    if (!im) return;
    const [px, py, pw, ph] = PANEL;
    const sag = a < 0.3 ? Math.round(PX.ease.out(a / 0.3) * 3) : 3;
    const f = Math.floor(a * 30 + 1e-6);
    const sc = f < 2 ? [1.14, 1.14] : f < 4 ? [1.1, 0.82] : f < 6 ? [0.96, 1.08] : [1, 1];
    const w = Math.round(pw * sc[0]), h = Math.round(ph * sc[1]);
    const cx = Lo.gx + px + pw / 2, by = Lo.gy + py + ph + sag;
    const x = Math.round(cx - w / 2), y = Math.round(by - h - (f < 2 ? 4 : 0));
    if (f < 2) { PX.rect(x - 1, y - 1, w + 2, h + 2, P.ink); PX.rect(x, y, w, h, P.bone); return; }
    g.drawImage(im, px, py, pw, ph, x, y, w, h);
  }

  // ------------------------------------------------------------ camera
  // [time, zoom, focus x, focus y]; a shot holds until the next row.
  const SHOTS = [
    [T0, 3, 124, 108],
    [T_OUT1, 2, 150, 100],
    [T_OUT2, 1, 160, 90],
    [T_DONE, 2, 80, 118],
    [T_POV, 2, 214, 104],
    [T_RAISE, 2, 84, 118],
    [T_PAUSE, 2, 170, 100],
    [T_PANE, 2, 216, 96],
    [T_TURN, 3, 236, 69],
    [T_BACKWAVE, 2, 80, 122],
    [T_PUSH, 3, 247, 58],
    [T_PUSH2, 4, 249, 56],
  ];
  function shotAt(t) { let s = SHOTS[0]; for (const r of SHOTS) if (r[0] <= t) s = r; return s; }

  function drawWorld(t) {
    const T = window.TMX, P = PX.P;
    const focus = t < T_PAUSE ? 'trex' : 'agent';
    const fa = t - T_PAUSE;
    const Lo = T.screen({
      t, focus, busy,
      flash: fa >= 0 && fa < 2 / 30,
      wide: fa >= 0 && fa < 0.25,
      agent: agentState(t),
      dimAgent: focus !== 'agent',
      game: (Lo) => drawGame(t, Lo),
      dimGame: focus !== 'trex',
      gameTop: (Lo) => drawPanel(t, Lo),
    });
    const st = clawdState(t);
    const c = T.clawd(CLAWD_X, Lo.bot - 21, st, 2);
    // The slam: sparks off the key, a bone ring.
    PX.burst(fa, CLAWD_X + 16, Lo.bot - 26, { n: 16, seed: 77, speed: 90, gravity: 260, life: 0.35, colors: [P.bone, P.cream, P.gold], size: 2, up: 60 });
    if (fa >= 0) T.borderSparks(Lo, 'agent', fa);
    PX.burst(t - T_DONE, CLAWD_X, c.head, { n: 12, seed: 71, speed: 60, gravity: 160, life: 0.5, colors: [P.lime, P.sprout, P.bone], size: 2, up: 50 });
    const r = rexState(t);
    if (r) drawRex(r);
    PX.burst(t - T_LAND, REX_X, EDGE_Y - 1, { n: 8, seed: 83, speed: 40, gravity: 60, life: 0.3, colors: [P.fog, P.haze], size: 1, up: 12, angle: -Math.PI / 2, spread: 3 });
  }

  PX.scene({
    id: 'outro', start: T0, end: T_CARD, layer: 0, cues,
    draw(t) {
      const T = window.TMX, P = PX.P;
      if (!T) return;
      const [t0, k, fx, fy] = shotAt(t);
      const world = T.grab('outro-world', () => drawWorld(t));
      const age = t - t0;
      const [sx, sy] = PX.shake(t - T_PAUSE, 3, 0.25, 29);
      T.view(world, k, fx, fy, sx, sy);
      // Pulling back out of the game: lines stream inward to the pane.
      if (t < T_OUT2 + 0.2) {
        const [cx, cy] = T.toScreen(k, fx, fy, 208, 109);
        T.speedLines(t, cx, cy, 0.8 * (1 - clamp((t - T0) / (T_OUT2 + 0.2 - T0))), 21, -1);
      }
      if (t >= T_PUSH) {
        const u = clamp((t - T_PUSH) / (T_CARD - T_PUSH));
        const [cx, cy] = T.toScreen(k, fx, fy, REX_X + 4, EDGE_Y - 22);
        T.speedLines(t, cx, cy, 0.35 + u * 0.65, 23);
      }
    },
  });

  // ------------------------------------------------------------ end card
  const LOGO_Y = 58, TAG_Y = 128, REPO_Y = 162;
  const SPANS_MID = [6, 17.5, 29, 41.5]; // art column centers of T R E X (outlined)
  const TAG_BANDS = () => { const P = PX.P; return [P.sprout, P.sprout, P.lime, P.lime, P.lime, P.lime, P.leaf, P.lime, P.sprout]; };
  function drawCard(t) {
    const T = window.TMX, P = PX.P;
    const a = t - T_CARD;
    const h = t - T_HIT;
    // A living backdrop: flat night, slow dusk rays, a warm disc behind the logo, embers.
    PX.clear(h >= 0 && h < 1 / 30 ? P.maroon : P.night);
    T.sunburst(160, 86, a * 0.12, 18, h >= 0 && h < 2 / 30 ? P.blood : P.dusk, 0);
    T.disc(160, 86, 70, P.night, P.dusk);
    for (let i = 0; i < 34; i++) {
      const sp = 10 + PX.hash(i, 13) * 22;
      const y = Math.round(H + 8 - ((a * sp + PX.hash(i, 14) * 200) % 200));
      const x = Math.round(PX.hash(i, 15) * W + Math.sin(a * 1.6 + i) * 5);
      PX.rect(x, y, 1, PX.hash(i, 16) < 0.3 ? 2 : 1, [P.gold, P.amber, P.ember, P.lime][i % 4]);
    }
    T.vignette(0.92, 0.08);
    // Logo letters drop in on eighths, then the logo waves.
    const letter = (n) => {
      const la = t - LETTERS[n];
      if (la < -0.14) return { vis: false };
      if (la < 0) { const u = 1 + la / 0.14; return { dy: -90 * (1 - u * u), kx: 3, ky: 5 }; }
      const f = PX.step(la);
      if (f < 1 / 12) return { kx: 5, ky: 3, flash: P.bone };
      if (f < 2 / 12) return { kx: 3, ky: 5, dy: -4 };
      if (h >= 0 && h < 2 / 30) return { kx: 5, ky: 3, flash: P.bone };
      if (h >= 2 / 30 && h < 4 / 30) return { kx: 4, ky: 5, dy: -3 };
      return {};
    };
    const tw = PX.step(t, 12);
    const wv = clamp((t - B(39, 2)) / 0.4);
    const wave = (i) => (wv > 0 ? Math.round(Math.sin(tw * 4.6 - i * 0.28) * 3 * wv) : 0);
    const sh = t >= T_HIT ? -14 + (t - T_HIT) * 120 : null;
    T.logo(160, LOGO_Y, { k: 4, letter, wave, shine: sh != null && sh < 80 ? sh : null });
    // Rex stands on the R, Clawd sits at his keyboard on the X.
    const x0 = 160 - 100;
    const top = (col) => LOGO_Y + wave(Math.round(col));
    const jump = h >= 0 && h < 0.35 ? Math.round(Math.sin((h / 0.35) * Math.PI) * 10) : 0;
    if (t >= T_REX_ON - 0.12) {
      const ra = t - T_REX_ON;
      const col = SPANS_MID[1];
      const drop = ra < 0 ? -Math.round(70 * Math.pow(-ra / 0.12, 2)) : 0;
      const f = PX.step(Math.max(0, ra));
      let pose = ra < 0 ? 'air' : f < 1 / 12 ? 'crouch' : 'q';
      const o = { k: 3, eyes: 'open' };
      if (t >= T_HIT) { pose = h < 0.6 ? 'roar' : 'q'; o.grin = true; }
      const bob = pose === 'q' && onBeat(t, T_CARD) ? 1 : 0;
      T.rex(pose, x0 + col * 4, top(col) + drop - jump + bob, o);
    }
    if (t >= T_CLAWD_ON - 0.12) {
      const ca = t - T_CLAWD_ON;
      const col = SPANS_MID[3];
      const drop = ca < 0 ? -Math.round(70 * Math.pow(-ca / 0.12, 2)) : 0;
      const f = PX.step(Math.max(0, ca));
      const st = { arms: ['hover', 'hover'], eyes: 'open', look: [0, 1], beat: onBeat(t, T_CARD) ? 1 : 0 };
      if (ca < 0) { st.squash = -1; st.arms = ['up', 'up']; }
      else if (f < 1 / 12) { st.squash = 1; st.eyes = 'shut'; }
      else if (t < T_HIT) {
        // Typing the repo line, one chunk per key.
        let i = -1; TYPE_KEYS.forEach((k, j) => { if (k <= t) i = j; });
        if (i >= 0 && t - TYPE_KEYS[i] < 2 / 12) { st.arms = i % 2 ? ['hover', 'key'] : ['key', 'hover']; st.nod = 1; }
      } else if (h < 0.6) { st.arms = ['up', 'up']; st.eyes = 'happy'; st.look = [0, 0]; st.squash = -1; st.beat = 2; }
      else { st.arms = ['rest', 'rest']; st.eyes = 'happy'; st.look = [0, 0]; st.nod = onBeat(t, T_CARD) ? 1 : 0; }
      T.clawd(x0 + col * 4, top(col) + 3 + drop - jump, st, 2);
    }
    // The repo line types out chunk by chunk; the final hit lands the tagline.
    let n = 0; TYPE_KEYS.forEach((k, j) => { if (k <= t) n = CHUNKS[j]; });
    if (n > 0) {
      const w = T.monoW(REPO);
      const rx = Math.round(160 - w / 2);
      const cx = rx + T.monoW(REPO.slice(0, n)) + 1;
      PX.rect(rx - 3, REPO_Y - 2, cx - rx + 7, 9, P.ink);
      T.mono(REPO.slice(0, n), rx, REPO_Y, n < REPO.length || t < T_HIT ? P.fog : P.haze);
      if (t < T_HIT || Math.floor((t - T_HIT) / 0.4) % 2 === 0) PX.rect(cx, REPO_Y - 1, 3, 6, P.bone);
    }
    const hf = Math.floor(h * 30 + 1e-6);
    if (hf >= -2) {
      const k = hf < 0 ? [4, 3][hf + 2] : 2;
      const tsh = t >= T_HIT ? -30 + (h - 0.25) * 260 : null;
      PX.title('IN YOUR TERMINAL', 160, TAG_Y - Math.round((9 * k - 18) / 2), { k, bands: TAG_BANDS(), edge: P.moss, shine: tsh, fx: () => (hf >= 0 && hf < 2 ? { flash: P.bone } : {}) });
    }
    PX.ring(h, 160, TAG_Y + 8, { r0: 16, r1: 240, life: 0.5, color: P.bone, color2: P.gold });
    PX.ring(h - 0.06, 160, TAG_Y + 8, { r0: 10, r1: 180, life: 0.45, color: P.gold, color2: P.ember });
    PX.burst(h, 160, TAG_Y - 2, { n: 50, seed: 41, speed: 220, gravity: 240, life: 1.0, colors: [P.bone, P.cream, P.gold, P.amber], size: 3, up: 90, angle: -Math.PI / 2, spread: 2.6 });
    for (let m = 0; m < 4; m++) PX.burst(t - LETTERS[m], x0 + SPANS_MID[m] * 4, LOGO_Y + 62, { n: 10, seed: 50 + m, speed: 70, gravity: 200, life: 0.5, colors: [P.fog, P.haze, P.gold], size: 2, up: 30 });
    const [sx, sy] = PX.shake(h, 6, 0.45, 17);
    const [lx, ly] = PX.shake(t - LETTERS[0], 2, 0.2, 19);
    T.shakeFrame(sx + lx, sy + ly);
    // Hold the finished card, then fade to black on the last beat, one palette step
    // darker every 12 fps frame.
    if (t >= T_FADE) {
      const steps = Math.floor((t - T_FADE) * 12 + 1e-6) + 1;
      if (steps >= 5) PX.clear(P.ink);
      else T.dimRect(0, 0, W, H, steps);
    }
    if (a < 1 / 30) PX.rect(0, 0, W, H, P.bone);
    else if (a < 2 / 30) PX.rect(0, 0, W, H, P.cream);
  }

  PX.scene({
    id: 'endcard', start: T_CARD, end: T_END, layer: 0,
    draw(t) { if (window.TMX) drawCard(t); },
  });
})();
