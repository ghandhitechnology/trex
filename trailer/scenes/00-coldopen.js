// 00-coldopen: bars 1-8 (0-15 s).
// A pixel tmux screen late at night. Clawd types a prompt into Claude Code while the
// trex title screen dozes in the unfocused pane next to it (real footage, focus-out).
// Enter hands focus to the game: the pane wakes, Rex notices Clawd, hops over,
// drapes himself over the pane border to watch, stretches, yawns, and turns to stare
// into the camera. Bar 6 dives into his pane; the bar-7 downbeat slams the TREX logo.
// Timing data and cues sit at the top as plain math (tools/cues.mjs reads them);
// drawing uses window.TMX from engine/tmux.js.
(function () {
  const BEAT = PX.BEAT, S = BEAT / 4;
  const B = (n, b = 0) => PX.bar(n, b);
  const at = (n) => n * S; // 16th step n -> seconds

  // ------------------------------------------------------------ timing
  const PROMPT = 'fix the flaky tests';
  const PROMPT_STEPS = [4, 5, 6, 8, 10, 11, 12, 14, 16, 17, 18, 19, 20, 22, 24, 25, 26, 27, 28];
  const KEYS = PROMPT_STEPS.map(at);
  const LOUD = KEYS.map((_, i) => PX.hash(i, 71) > 0.62 || i === 8 || i === 16);
  const T_WIND = at(30), T_ENTER = B(3); // both paws up, then Enter; focus moves to the game
  const T_NOTICE = B(3, 2), T_HOP1 = B(3, 3), T_LAND1 = B(3, 3.5), T_PEEK = B(4);
  const T_STRETCH = B(4, 2), T_CLOSE = B(4, 3), T_STAND = B(4, 3.5);
  const T_LETGO = T_CLOSE; // unhooks from the border and hops back to stand
  const T_STARE = B(5), T_LOOK = B(5, 1), T_BOTH = B(5, 2), T_BLINK = B(5, 3);
  const T_GRIN = B(6), T_HOP2 = B(6, 0.5), T_HOME = B(6, 1), T_DIVE = B(6, 2), T_EYE = B(6, 2.5), T_BREATH = B(6, 3);
  const T_DROP = B(7), T_END = B(9);
  const T_SLAM0 = T_DROP - 3 / 30; // the logo shrinks over 3 whole 30 fps frames and lands on the downbeat
  const BOUNCE = [0, 1, 2, 3].map((n) => B(7, 1 + n * 0.5));
  const T_INHALE = B(7, 3), T_ROAR = B(8), T_HIT2 = B(8, 1), T_HIT3 = B(8, 3);

  // Output lines appear on the grid: [time, kind, ...args].
  const OUT = [
    [T_ENTER, 'user', PROMPT],
    [B(3, 1), 'tool', 'Read', 'ledger.rs', B(3, 1.5)],
    [B(3, 1.5), 'res', 'Read 96 lines'],
    [B(3, 2.5), 'tool', 'Update', 'ledger.rs', B(3, 3.5)],
    [B(3, 3.5), 'res', '1 added, 1 removed'],
    [B(4), 'del', 42, 'b = bal(id);'],
    [B(4, 0.5), 'add', 42, 'b = lock(id)?;'],
    [B(4, 1.5), 'tool', 'Bash', 'cargo test', B(4, 3)],
    [B(4, 3), 'ok', '412 passed'],
    [B(6), 'say', 'Fixed. All green.'],
  ];
  // Test dots stream in on 16ths while cargo test runs (the percussion after the typing).
  const DOTS = [];
  for (let s = 0; s < 6; s++) DOTS.push(B(4, 1.75 + s * 0.25));

  // ------------------------------------------------------------ cues
  const cues = [];
  KEYS.forEach((k, i) => cues.push([k, 'key', LOUD[i] ? 0.85 : 0.5 + 0.2 * PX.hash(i, 72)]));
  cues.push([T_WIND, 'whoosh', 0.25]);
  cues.push([T_ENTER, 'enter', 1], [T_ENTER, 'blip', 0.6]);
  for (const o of OUT) if (o[1] !== 'user') cues.push([o[0], 'type', 0.35]);
  DOTS.forEach((d, i) => cues.push([d, 'type', 0.22 + 0.04 * i]));
  cues.push([T_NOTICE, 'pop', 0.8]);
  cues.push([T_HOP1, 'whoosh', 0.3], [T_LAND1, 'land', 0.5], [T_PEEK, 'land', 0.7]);
  cues.push([T_STRETCH, 'whoosh', 0.2]);
  cues.push([T_STARE, 'blip', 0.7], [T_LOOK, 'blip', 0.4], [T_BOTH, 'whoosh', 0.3], [T_BLINK, 'blip', 0.25]);
  cues.push([T_GRIN, 'riser', 0.9], [T_HOP2, 'whoosh', 0.6], [T_HOME, 'land', 0.5], [T_HOME, 'whoosh', 0.5], [T_DIVE, 'whoosh', 0.7], [T_EYE, 'whoosh', 0.8]);
  cues.push([T_DROP, 'slam', 1], [T_DROP, 'impact', 1]);
  BOUNCE.forEach((t) => cues.push([t, 'land', 0.7]));
  cues.push([T_INHALE, 'whoosh', 0.5]);
  cues.push([T_ROAR, 'shine', 0.9], [T_ROAR, 'boom', 0.8]);
  cues.push([T_HIT2, 'impact', 0.6], [T_HIT3, 'slam', 0.8], [T_HIT3, 'whoosh', 0.6]);

  // ------------------------------------------------------------ helpers
  const W = PX.W, H = PX.H;
  const clamp = PX.clamp;
  const CLAWD_X = 66;
  // Stepped age since t0 (stop motion at 12 fps).
  const q = (t, t0) => PX.step(t - t0);
  const lastKey = (t) => { let idx = -1; for (let i = 0; i < KEYS.length; i++) if (KEYS[i] <= t) idx = i; return idx; };
  const keysBefore = (t) => lastKey(t) + 1;
  // Beat hit: 1 for the first two 12 fps frames after each beat.
  const onBeat = (t, from = 0) => t >= from && PX.step((((t - from) % BEAT) + BEAT) % BEAT) < 2 / 12;
  // The trex title screen: frames 18-104 hold one full shine pass. Each shot plays a
  // forward stretch of it (seams sit on cuts); while the pane is unfocused the game
  // drops to 8 fps.
  const TITLE_A = 18 / 30, TITLE_SPAN = 86 / 30;
  function titleLocal(t) {
    let i = 0;
    for (let j = 0; j < SHOTS.length; j++) if (SHOTS[j][0] <= t) i = j;
    const t0 = SHOTS[i][0], t1 = i + 1 < SHOTS.length ? SHOTS[i + 1][0] : T_BREATH;
    const room = Math.max(0, TITLE_SPAN - (t1 - t0) - 1 / 30);
    const tt = t < T_ENTER ? PX.step(t - t0, 8) : t - t0;
    return TITLE_A + PX.hash(i, 5) * room + Math.min(tt, TITLE_SPAN - 1 / 30);
  }
  const REX_HOME = [208, 116], REX_MID = [160, 116];
  const BORDER_X = 94, REX_PEEK = [99, 116], REX_STAND = [131, 116];

  // ------------------------------------------------------------ Claude Code
  function logLines(t) {
    const P = PX.P;
    const L = [
      { runs: [['Claude Code', P.bone], [' v2.1', P.mauve]] },
      { runs: [['Opus 5.5 · Max', P.mauve]] },
      { runs: [['~/api', P.mauve]] },
      {},
      { runs: [['> ', P.haze], ['add retry backoff', P.fog]], bg: P.dusk },
      {},
      { runs: [['⏺ ', P.lime], ['Update', P.bone], ['(client.rs)', P.haze]] },
      { runs: [['  ⎿  ', P.mauve], ['Added 3 lines', P.haze]] },
      {},
      { runs: [['⏺ ', P.bone], ['Done. Retries back', P.fog]] },
      { runs: [['  off from 200ms.', P.fog]] },
      {},
      { runs: [['> ', P.haze], ['run the tests', P.fog]], bg: P.dusk },
      {},
      { runs: [['⏺ ', P.red], ['2 failed: ledger', P.fog]] },
    ];
    for (const o of OUT) {
      if (o[0] > t) break;
      const [, kind, a, b, done] = o;
      if (kind === 'user') L.push({}, { runs: [['> ', P.haze], [a, P.bone]], bg: P.dusk });
      else if (kind === 'tool') {
        const ok = t >= done, dot = ok ? P.lime : Math.floor((t - o[0]) * 4) % 2 ? P.mauve : P.haze;
        L.push({}, { runs: [['⏺ ', dot], [a, P.bone], ['(' + b + ')', P.haze]] });
        if (a === 'Bash' && !ok) {
          const n = DOTS.filter((d) => d <= t).length;
          if (n) L.push({ runs: [['  ⎿  ', P.mauve], ['.'.repeat(n * 2), P.lime]] });
        }
      } else if (kind === 'res') L.push({ runs: [['  ⎿  ', P.mauve], [a, P.haze]] });
      else if (kind === 'ok') L.push({ runs: [['  ⎿  ', P.mauve], [a + ' ', P.lime], ['✓', P.lime]] });
      else if (kind === 'del') L.push({ runs: [[' ' + a + ' -', P.blush], [' ' + b, P.bone]], bg: P.maroon });
      else if (kind === 'add') L.push({ runs: [[' ' + a + ' +', P.sprout], [' ' + b, P.bone]], bg: P.moss });
      else if (kind === 'say') L.push({}, { runs: [['⏺ ', P.bone], [a, P.fog]] });
    }
    return L;
  }
  function agentState(t) {
    const typed = PROMPT.slice(0, Math.min(PROMPT.length, keysBefore(t)));
    const working = t >= T_ENTER;
    return {
      lines: logLines(t),
      input: working ? '' : typed,
      cursor: working ? Math.floor(t / 0.5) % 2 === 0 : true,
      spin: working && t < B(6) ? { i: Math.floor((t - T_ENTER) / 0.12), verb: 'Clauding', sub: Math.floor(t - T_ENTER + 3) + 's' } : null,
      hint: working && t < B(6) ? 'esc to interrupt' : '? for shortcuts',
    };
  }
  const busy = (tt) => { let n = 0; for (const k of KEYS) if (k <= tt && k > tt - 0.6) n++; return Math.min(1, n / 5 + (tt > T_ENTER && tt < B(6) ? 0.5 : 0)); };

  // ------------------------------------------------------------ Clawd acting
  function clawdState(t) {
    const s = { arms: ['hover', 'hover'], eyes: 'open', look: [0, 1], lean: 0, nod: 0, beat: 0 };
    const beatLight = PX.pulse(t, BEAT, 7);
    s.beat = beatLight > 0.55 ? (beatLight > 0.85 ? 2 : 1) : 0;
    if (t < KEYS[0]) {
      // Headphones on, nodding along; a blink, then eyes drop to the keys.
      s.arms = ['rest', 'rest']; s.look = [0, 0];
      if (t >= 0.16 && t < 0.24) s.eyes = 'blink';
      if (t >= at(3)) { s.look = [0, 1]; s.nod = 1; s.arms = ['hover', 'hover']; }
      return s;
    }
    if (t < T_WIND) {
      const i = lastKey(t);
      const side = (i + (i % 5 === 3 ? 1 : 0)) % 2;
      if (i >= 0 && t - KEYS[i] < 2 / 12) { s.arms = side ? ['hover', 'key'] : ['key', 'hover']; s.nod = 1; }
      for (const b of [1.6, 2.95]) if (t >= b && t < b + 0.1) s.eyes = 'blink';
      return s;
    }
    if (t < T_ENTER) { s.arms = ['up', 'up']; s.squash = -1; s.eyes = 'happy'; s.look = [0, 0]; return s; }
    const e = q(t, T_ENTER);
    if (e < 2 / 12) { s.arms = ['slam', 'slam']; s.squash = 1; s.eyes = 'shut'; s.look = [0, 0]; return s; }
    // The agent works on its own: Clawd sits back and grooves, nodding on the beat.
    s.lean = -1; s.arms = ['rest', 'rest']; s.look = [0, 0]; s.eyes = 'open';
    s.nod = onBeat(t, T_ENTER) ? 1 : 0;
    if (e < 5 / 12) s.eyes = 'happy';
    for (const b of [4.55, 6.3]) if (t >= b && t < b + 0.1) s.eyes = 'blink';
    if (t >= T_PEEK + 0.2 && t < T_STARE) s.look = [1, 0]; // watching Rex watch
    if (t >= T_STARE) {
      s.nod = 0; s.beat = 0;
      if (t < T_LOOK) s.look = [1, 0];
      else if (t < T_LOOK + BEAT / 2) { s.look = [1, -1]; s.eyes = 'wide'; s.lean = 0; s.squash = q(t, T_LOOK) < 1 / 12 ? -1 : 0; }
      else if (t < T_GRIN) { s.look = [0, 0]; s.eyes = 'half'; s.sweat = t >= T_LOOK + BEAT; }
      if (t >= T_BLINK && t < T_BLINK + 2 / 12) s.eyes = 'shut';
      if (t >= T_GRIN) { s.look = [1, 0]; s.eyes = 'open'; s.lean = 0; }
    }
    return s;
  }
  // Key sparks off the typing paws, and music notes off the headphones while it grooves.
  function clawdFx(t, bx, by) {
    const P = PX.P;
    const i = lastKey(t);
    for (let j = Math.max(0, i - 3); j <= i; j++) {
      const age = t - KEYS[j];
      if (age < 0 || age > 0.3) continue;
      const side = (j + (j % 5 === 3 ? 1 : 0)) % 2;
      PX.burst(age, bx + (side ? 14 : -14), by - 6, { n: LOUD[j] ? 5 : 3, seed: 40 + j, speed: 40, gravity: 260, life: 0.26, colors: [P.ice, P.cyan, P.bone], size: 1, up: 44 });
    }
    PX.burst(t - T_ENTER, bx, by - 6, { n: 14, seed: 91, speed: 70, gravity: 240, life: 0.4, colors: [P.cream, P.gold, P.bone], size: 2, up: 50 });
    if (t >= T_ENTER + 0.3 && t < T_STARE) {
      for (let n = 0; n < 6; n++) {
        const t0 = T_ENTER + 0.3 + n * BEAT * 1.5, age = t - t0;
        if (age < 0 || age > 1.1) continue;
        const side = n % 2, a = PX.step(age);
        const x = Math.round(bx + (side ? 26 : -26) + (side ? 1 : -1) * a * 8 + Math.sin(a * 9) * 2), y = Math.round(by - 30 - a * 24);
        const c = a < 0.7 ? P.pink : P.grape;
        PX.rect(x, y, 2, 2, c); PX.rect(x + 1, y - 4, 1, 4, c); PX.rect(x + 2, y - 4, 2, 1, c);
      }
    }
  }

  // ------------------------------------------------------------ Rex acting
  // Returns what to draw for Rex, or null while the footage Rex is still the one on screen.
  function rexState(t) {
    if (t < B(2)) return null;
    const P = PX.P;
    if (t < T_ENTER) {
      // Dozing in the unfocused pane (8 fps, like the game), ticking at the loud keys
      // and turning his head toward the typing once.
      const lt = PX.step(t, 8);
      const f = Math.floor(lt * 4) % 2;
      const r = { pose: f ? 'idle1' : 'idle0', x: REX_HOME[0], y: REX_HOME[1], eyes: 'half' };
      const i = lastKey(t);
      if (i >= 0 && LOUD[i]) {
        const a = q(t, KEYS[i]);
        if (a < 1 / 12) { r.pose = 'idle0'; r.y -= 2; r.eyes = 'fwd'; }
        else if (a < 2 / 12) { r.pose = 'flick'; r.eyes = 'fwd'; }
      }
      const gl = t - B(2, 3);
      if (gl >= 0 && gl < 4 / 12) { r.pose = gl < 1 / 12 ? 'idle0' : 'idle0'; r.flip = true; r.x -= gl < 1 / 12 ? 1 : 3; r.eyes = 'fwd'; }
      return r;
    }
    const hop = (t0, t1, from, to, hgt) => {
      const u = clamp(q(t, t0) / (t1 - t0));
      return [Math.round(PX.lerp(from, to, u)), Math.round(REX_HOME[1] - Math.sin(u * Math.PI) * hgt)];
    };
    if (t < T_NOTICE) {
      // Focus lands on his pane: he jolts awake.
      const a = q(t, T_ENTER);
      if (a < 1 / 12) return { pose: 'crouch', x: REX_HOME[0], y: REX_HOME[1], eyes: 'wide' };
      if (a < 3 / 12) return { pose: 'air', x: REX_HOME[0], y: REX_HOME[1] - (a < 2 / 12 ? 6 : 3), eyes: 'wide' };
      return { pose: 'idle0', x: REX_HOME[0], y: REX_HOME[1], eyes: a < 5 / 12 ? 'wide' : 'fwd' };
    }
    if (t < T_HOP1 - 2 / 12) {
      const a = q(t, T_NOTICE);
      const lift = a < 1 / 12 ? 5 : a < 2 / 12 ? 2 : 0;
      return { pose: 'idle0', x: REX_HOME[0], y: REX_HOME[1] - lift, flip: true, eyes: a < 0.3 ? 'wide' : 'fwd', bang: T_NOTICE };
    }
    if (t < T_HOP1) return { pose: 'crouch', x: REX_HOME[0], y: REX_HOME[1], flip: true, bang: T_NOTICE };
    if (t < T_LAND1) { const [x, y] = hop(T_HOP1, T_LAND1, REX_HOME[0], REX_MID[0], 14); return { pose: 'air', x, y, flip: true }; }
    if (t < T_LAND1 + 1 / 12) return { pose: 'crouch', x: REX_MID[0], y: REX_HOME[1], flip: true };
    if (t < T_PEEK) { const [x, y] = hop(T_LAND1 + 1 / 12, T_PEEK, REX_MID[0], REX_PEEK[0] + 6, 10); return { pose: 'air', x, y, flip: true }; }
    if (t < T_LETGO) {
      // Draped over the border: claws hooked on it, head over Clawd's pane, bobbing
      // on the beat with Clawd; then a stretch and a big yawn, still hanging on.
      const r = { pose: 'peek', x: REX_PEEK[0], y: REX_HOME[1], flip: true, head: 'down', grip: true };
      const a = q(t, T_PEEK);
      if (a < 1 / 12) { r.head = 'fwd'; r.bob = 2; return r; }
      if (t < T_STRETCH) {
        const b = PX.step((((t - T_PEEK) % BEAT) + BEAT) % BEAT);
        r.bob = t >= T_PEEK + BEAT - 1e-6 ? (b < 1 / 12 ? 2 : b < 2 / 12 ? 1 : 0) : 0;
        if (t >= B(4, 1.5) && t < B(4, 1.5) + 2 / 12) r.head = 'shut';
        return r;
      }
      const s = q(t, T_STRETCH);
      if (s < 3 / 12) { r.up = true; r.head = 'shut'; return r; }
      if (s < 9 / 12) { r.head = 'yawn'; r.up = s < 6 / 12; return r; }
      if (s < 10 / 12) { r.head = 'shut'; return r; }
      r.head = 'fwd';
      return r;
    }
    if (t < T_STAND) {
      // Lets go of the border and hops back into his pane.
      const a = q(t, T_LETGO);
      if (a < 1 / 12) return { pose: 'peek', x: REX_PEEK[0], y: REX_HOME[1], flip: true, head: 'fwd', up: true };
      const [x, y] = hop(T_LETGO + 1 / 12, T_STAND, REX_PEEK[0] + 4, REX_STAND[0], 10);
      return { pose: 'air', x, y, flip: true };
    }
    if (t < T_STARE) {
      // Lands, and eyes the camera sideways.
      const a = q(t, T_STAND);
      return { pose: a < 1 / 12 ? 'crouch' : 'idle0', x: REX_STAND[0], y: REX_HOME[1], flip: true, eyes: a < 2 / 12 ? 'fwd' : 'back' };
    }
    if (t < T_GRIN) {
      // The stare: turn on the downbeat, then the deadpan hold.
      const a = q(t, T_STARE);
      if (a < 1 / 12) return { pose: 'turn', x: REX_STAND[0], y: REX_HOME[1], flip: true };
      const r = { pose: 'q', x: REX_STAND[0], y: REX_HOME[1], flip: true, eyes: a < 3 / 12 ? 'open' : 'half' };
      if (t >= T_BLINK && t < T_BLINK + 2 / 12) r.eyes = 'shut';
      return r;
    }
    if (t < T_HOP2) {
      const a = q(t, T_GRIN);
      return { pose: 'q', x: REX_STAND[0], y: REX_HOME[1], flip: true, grin: true, eyes: a < 2 / 12 ? 'open' : 'wink' };
    }
    if (t < T_HOP2 + 1 / 12) return { pose: 'crouch', x: REX_STAND[0], y: REX_HOME[1] };
    if (t < T_HOME) { const [x, y] = hop(T_HOP2 + 1 / 12, T_HOME, REX_STAND[0], REX_HOME[0], 22); return { pose: 'air', x, y }; }
    if (t < T_HOME + 1 / 12) return { pose: 'crouch', x: REX_HOME[0], y: REX_HOME[1] };
    if (t < T_DIVE) return { pose: 'idle0', x: REX_HOME[0], y: REX_HOME[1], eyes: 'fwd' };
    if (t < T_EYE) return { pose: q(t, T_DIVE) < 2 / 12 ? 'crouch' : 'idle0', x: REX_HOME[0], y: REX_HOME[1], eyes: 'back' };
    return { pose: 'idle0', x: REX_HOME[0], y: REX_HOME[1], eyes: 'wide' };
  }

  function drawRex(t, r) {
    const T = window.TMX, P = PX.P;
    // Contact shadow on the title-screen floor, like the game's.
    const air = Math.max(0, REX_HOME[1] - r.y);
    const rx = Math.max(4, 11 - Math.round(air / 3));
    const sx = r.x;
    if (sx > 100) {
      PX.rect(sx - rx + 2, REX_HOME[1] - 1, rx * 2 - 4, 3, P.ink);
      PX.rect(sx - rx, REX_HOME[1], rx * 2, 1, P.ink);
    }
    const o = { flip: r.flip, eyes: r.eyes, head: r.head, bob: r.bob, up: r.up, grin: r.grin };
    T.rex(r.pose, r.x, r.y, o);
    if (r.grip) {
      // The border runs in front of his chest; his claws hook over it.
      const Lo = T.layout();
      PX.rect(BORDER_X, r.y - 17, 1, Lo.bot - (r.y - 17), P.lime);
      for (const cy of [r.y - 16, r.y - 11]) {
        PX.rect(BORDER_X - 2, cy - 1, 5, 5, P.ink);
        PX.rect(BORDER_X - 1, cy, 3, 3, P.leaf);
        PX.rect(BORDER_X - 1, cy, 1, 3, P.bone);
        PX.px(BORDER_X + 1, cy, P.lime);
      }
    }
    if (r.bang != null) {
      const a = PX.step(t - r.bang);
      if (a >= 0 && a < 0.55) {
        const kx = a < 1 / 12 ? 5 : 3, ky = a < 1 / 12 ? 2 : a < 2 / 12 ? 4 : 3;
        const bx = r.x + (r.flip ? -30 : 30), by = r.y - 14 - (a < 2 / 12 ? 0 : 2);
        T.art(T.BANG, bx, by, { kx, ky });
      }
    }
  }

  // ------------------------------------------------------------ camera
  // [time, zoom, focus x, focus y, drift x, drift y (screen px/s)]; a shot holds until the next row.
  const SHOTS = [
    [0, 4, 62, 138, 0, 0],
    [KEYS[0], 3, 54, 131, 0, -3],
    [B(2), 2, 206, 86, -4, 0],
    [T_ENTER, 1, 160, 90, 0, 0],
    [T_PEEK, 2, 90, 112, 3, 0],
    [T_CLOSE, 3, 104, 108, -4, 0],
    [T_LOOK, 3, 50, 130, 4, 0],
    [T_BOTH, 2, 80, 120, 0, 0],
    [T_GRIN, 1, 160, 90, 0, 0],
    [T_HOME, 2, 208, 86, 0, -3],
    [T_DIVE, 3, 208, 110, 0, -4],
    [T_EYE, 4, 212, 98, 0, 0],
  ];
  function shotAt(t) { let s = SHOTS[0]; for (const r of SHOTS) if (r[0] <= t) s = r; return s; }

  // Once the drawn Rex takes over, hide the footage Rex under a same-frame copy of the
  // floor from the pane's left edge, with a dithered feather on the sides and bottom only
  // (the top edge sits in the clear band under the logo's shadow).
  const BAY = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5];
  function patchRex(img, gx, gy) {
    const g = PX.g, X0 = 95, Y0 = 39, X1 = 131, Y1 = 77, F = 5, OX = -94, OY = -1;
    g.drawImage(img, X0 + OX, Y0 + OY, X1 - X0, Y1 - Y0, gx + X0, gy + Y0, X1 - X0, Y1 - Y0);
    for (let y = Y0; y < Y1 + F; y++) for (let x = X0 - F; x < X1 + F; x++) {
      if (x >= X0 && x < X1 && y < Y1) continue;
      const a = 1 - Math.max(X0 - x, x - (X1 - 1), y - (Y1 - 1), 0) / F;
      if (a <= 0 || BAY[((gy + y) & 3) * 4 + ((gx + x) & 3)] / 16 >= a) continue;
      g.drawImage(img, x + OX, y + OY, 1, 1, gx + x, gy + y, 1, 1);
    }
  }

  // ------------------------------------------------------------ world
  function drawWorld(t) {
    const T = window.TMX, g = PX.g, P = PX.P;
    const r = rexState(t);
    const focus = t < T_ENTER ? 'agent' : 'trex';
    const fa = t - T_ENTER;
    const Lo = T.screen({
      t, focus, busy,
      flash: fa >= 0 && fa < 2 / 30,
      wide: fa >= 0 && fa < 0.2,
      agent: agentState(t),
      agentBg: t < B(2) ? glow : null,
      dimAgent: focus !== 'agent',
      game(Lo) {
        const img = PX.footageFrame('pane-title', titleLocal(t));
        if (!img) return;
        g.drawImage(img, Lo.gx, Lo.gy);
        if (r) patchRex(img, Lo.gx, Lo.gy);
        // The drawn Rex lives in the game while he is inside it, so he dims with it.
        if (r && r.pose !== 'peek' && t < T_ENTER) drawRex(t, r);
      },
      dimGame: focus !== 'trex',
    });
    // Clawd sits in front of his pane's text; the typing sparks stay inside it.
    T.clawd(CLAWD_X, Lo.bot - 21, clawdState(t), 2);
    T.clip(0, Lo.top, Lo.aw, Lo.bot - Lo.top, () => clawdFx(t, CLAWD_X, Lo.bot - 21));
    if (r && (t >= T_ENTER || r.pose === 'peek')) {
      if (r.pose === 'peek' || r.x < Lo.gx + 20) drawRex(t, r);
      else T.clip(Lo.gx, Lo.gy, Lo.gw, Lo.gh, () => drawRex(t, r));
    }
    if (fa >= 0) T.borderSparks(Lo, 'trex', fa);
    // Dust puffs where Rex lands, a spark where his claws hook the border.
    for (const [tl, x] of [[T_LAND1, REX_MID[0]], [T_HOME, REX_HOME[0]]]) {
      PX.burst(t - tl, x, REX_HOME[1] - 1, { n: 8, seed: Math.round(tl * 10), speed: 40, gravity: 60, life: 0.3, colors: [P.fog, P.haze, P.mauve], size: 1, up: 12, angle: -Math.PI / 2, spread: 3 });
    }
    PX.burst(t - T_PEEK, BORDER_X, REX_HOME[1] - 14, { n: 10, seed: 57, speed: 60, gravity: 180, life: 0.3, colors: [P.bone, P.lime, P.sprout], size: 1, up: 30 });
  }
  // Dust motes drifting through the light (screen space, over the opening close-ups).
  function motes(t, a) {
    const P = PX.P;
    if (a <= 0) return;
    for (let i = 0; i < 14; i++) {
      const sp = 3 + PX.hash(i, 81) * 5;
      const x = Math.round((PX.hash(i, 82) * W + t * sp * (PX.hash(i, 83) > 0.5 ? 1 : -1) + Math.sin(t * 0.7 + i) * 6 + W) % W);
      const y = Math.round((PX.hash(i, 84) * H - t * sp * 0.6 + H * 4) % H);
      const tw = Math.sin(t * 2.2 + i * 1.9);
      if (tw < -0.3 || PX.hash(i, 85) > a) continue;
      PX.rect(x, y, tw > 0.6 ? 2 : 1, tw > 0.6 ? 2 : 1, tw > 0.6 ? P.cream : P.sand);
    }
  }
  // Warm screen light pooling behind Clawd on the opening close-ups: two flat rings
  // of glow under the terminal text.
  function glow(Lo) {
    const P = PX.P, cx = CLAWD_X, cy = Lo.bot - 36;
    for (let y = cy - 44; y <= cy + 44; y++) for (let x = cx - 60; x <= cx + 60; x++) {
      const d = Math.sqrt(((x - cx) / 60) ** 2 + ((y - cy) / 40) ** 2);
      if (d > 1) continue;
      PX.px(x, y, d < 0.5 ? P.maroon : P.night);
    }
  }

  // Near-black breath before the drop: Rex's silhouette inhales, his eye catches the light.
  function drawBreath(t) {
    const T = window.TMX, P = PX.P;
    PX.clear(P.ink);
    const a = t - T_BREATH;
    const inhale = a > 0.12, glint = a > 0.26;
    const pose = inhale ? 'idle0' : 'crouch';
    const sil = (fill) => (c, i, j) => (c === P.ink ? null : glint && c === P.bone && j <= 4 ? P.bone : fill);
    T.rex(pose, 162, 176 - (inhale ? 6 : 0), { k: 6, eyes: 'wide', color: sil(P.dusk) });
    T.rex(pose, 160, 178 - (inhale ? 6 : 0), { k: 6, eyes: 'wide', color: sil(P.night) });
    T.vignette(0.62, 0.1);
  }

  PX.scene({
    id: 'coldopen', start: 0, end: T_SLAM0, layer: 0, cues,
    draw(t) {
      const T = window.TMX, P = PX.P;
      if (!T) return;
      if (t >= T_BREATH) return drawBreath(t);
      const [t0, k, fx, fy, vx, vy] = shotAt(t);
      const opening = t < B(2);
      const world = T.grab('co-world', () => drawWorld(t));
      const age = t - t0;
      T.view(world, k, fx, fy, Math.round(age * vx), Math.round(age * vy));
      if (opening) motes(t, 1);
      // Fade up from black over the first beat, one palette step per 12 fps frame.
      const fin = 4 - Math.floor(t * 12 + 1e-6);
      if (fin >= 4) PX.clear(P.ink);
      else if (fin > 0) T.dimRect(0, 0, W, H, fin);
      // Bar 6: speed lines streaming out of Rex and an iris closing on him.
      if (t >= T_HOME) {
        const u = clamp((t - T_HOME) / (T_BREATH - T_HOME));
        const [rx, ry] = T.toScreen(k, fx, fy, REX_HOME[0], REX_HOME[1] - 16, Math.round(age * vx), Math.round(age * vy));
        T.speedLines(t, rx, ry, 0.35 + u * 0.65, 9);
        if (u > 0.45) T.vignette(1.05 - (u - 0.45) * 1.1, 0.08, rx, ry);
      }
    },
  });

  // ------------------------------------------------------------ the drop
  // The dive lands in the game's world blown up: a night sky with maroon rays behind
  // the logo, a rocky ledge, Rex at 5x roaring, the logo slamming in at 4x.
  const LOGO_Y = 14, REX_X = 150, REX_Y = 174, LOGO_C = [160, 44];
  const PROPS = [['p-skull', 24, 0], ['p-tuft', 58, 1], ['p-bone', 238, 0], ['p-rock', 276, 1], ['p-tuft-dim', 206, 1], ['p-fern', 92, 0], ['p-ribs', 300, 0]];
  function ledge(t, a, lit) {
    const P = PX.P;
    // Solid ridge: a flat-topped rock shelf with a stepped silhouette.
    const top = (x) => 150 + Math.round(3 * Math.sin(x * 0.045 + 1.3) + 2 * Math.sin(x * 0.13));
    for (let x = 0; x < W; x++) {
      const y = top(x);
      PX.rect(x, y, 1, H - y, P.ink);
      PX.rect(x, y, 1, 1, lit ? P.amber : P.dusk);
      if (x % 7 < 4) PX.rect(x, y + 1, 1, 1, lit ? P.ember : P.night);
    }
    for (const [name, x, row] of PROPS) {
      const xx = Math.round(((x - a * 6) % (W + 40) + W + 40) % (W + 40) - 20);
      PX.spr(name, xx, top(xx + 8) - 14 + row * 2, 0, { k: 2 });
    }
  }
  function sky(t, a, P, pop) {
    const base = pop === 1 ? P.amber : pop === 2 ? P.maroon : P.night;
    const ray = pop === 1 ? P.gold : pop === 2 ? P.blood : amberBeat(t) ? P.blood : P.maroon;
    PX.clear(base);
    TMX.sunburst(LOGO_C[0], LOGO_C[1] + 8, a * 0.25 + 0.2, 16, ray, 0, 0, H);
    if (!pop) TMX.disc(LOGO_C[0], LOGO_C[1] + 8, 40, amberBeat(t) ? P.maroon : P.night, amberBeat(t) ? P.blood : P.maroon);
  }
  // On bar 8 every beat lights the scene amber for two frames.
  const amberBeat = (t) => t >= T_ROAR && onBeat(t, T_ROAR);
  function drawDrop(t) {
    const T = window.TMX, P = PX.P;
    const a = t - T_DROP;
    if (a < 0) {
      // Pre-roll over black: the logo shrinks from huge and lands on the downbeat.
      PX.clear(P.ink);
      const f = clamp(Math.floor((t - T_SLAM0) * 30 + 1e-6), 0, 2);
      const k = [14, 10, 7][f];
      T.logo(160, Math.round(LOGO_Y + 30 - (15 * k) / 2), { k, depth: k });
      return;
    }
    const pop = a < 1 / 30 ? 1 : a < 2 / 30 ? 2 : 0;
    sky(t, a, P, pop);
    embers(t, a);
    ledge(t, a, amberBeat(t) || (t >= T_ROAR && t < T_HIT3 + 0.3));
    // Rex: roars on the slam, settles, inhales on beat 3, roars fire through bar 8.
    const ra = q(t, T_DROP);
    let pose = 'roar', jx = 0, eyes;
    if (t < BOUNCE[1]) { pose = Math.floor(ra * 12) % 2 ? 'roarWide' : 'roar'; jx = Math.floor(ra * 12) % 2 ? 2 : -2; }
    else if (t < T_INHALE - 1 / 12) { pose = 'idle0'; eyes = 'fwd'; }
    else if (t < T_INHALE) pose = 'crouch';
    else if (t < T_ROAR) pose = 'roarBack';
    else { const rb = q(t, T_ROAR); pose = Math.floor(rb * 12) % 3 === 2 ? 'roar' : 'roarWide'; jx = Math.floor(rb * 12) % 2 ? 1 : -1; }
    const bob = pose === 'idle0' && onBeat(t, T_DROP) ? 3 : 0;
    const k = 5;
    T.rex(pose, REX_X + jx, REX_Y + bob, { k, eyes });
    const mouth = [REX_X + jx + (pose === 'roarWide' ? 42 : 36), REX_Y - (pose === 'roarWide' ? 50 : 46)];
    // Roar rings: forward half-arcs spreading from his mouth.
    if (t < BOUNCE[1]) for (let i = 0; i < 4; i++) {
      const rr = (((ra - i * 0.11) % 0.44) + 0.44) % 0.44;
      if (ra - i * 0.11 >= 0) arc(mouth[0] + 4, mouth[1], 8 + PX.ease.out(rr / 0.44) * 60, rr < 0.22 ? P.cream : P.amber);
    }
    if (t >= T_INHALE && t < T_ROAR) inhale(t, mouth[0], mouth[1]);
    if (t >= T_ROAR) breath(t, mouth[0], mouth[1]);
    // The logo lands, squashes, ripples letter by letter (each landing ON its beat), waves.
    const land = q(t, T_DROP);
    const hitSquash = (h) => { const s = t - h; return s >= 0 && s < 2 / 30; };
    const bounce = (n) => {
      const bt = BOUNCE[n], ba = t - bt;
      if (ba < -2 / 12 || ba > 3 / 12) return {};
      if (ba < -1 / 12) return { dy: -12, kx: 3, ky: 5 };
      if (ba < 0) return { dy: -5, kx: 4, ky: 4 };
      if (ba < 1 / 12) return { kx: 5, ky: 3, flash: P.bone };
      if (ba < 2 / 12) return { dy: -2, kx: 4, ky: 4 };
      return {};
    };
    const letter = (n) => {
      if (land < 1 / 12) return { kx: 5, ky: 3, flash: P.bone };
      if (land < 2 / 12) return { kx: 4, ky: 5, dy: -4 };
      if (hitSquash(T_HIT2) || hitSquash(T_HIT3)) return { kx: 5, ky: 3, dy: 2, flash: t - T_HIT3 < 1 / 30 && t >= T_HIT3 ? P.bone : null };
      if (hitSquash(T_HIT2 - 2 / 30) || hitSquash(T_HIT3 - 2 / 30)) return { kx: 4, ky: 5, dy: -3 };
      return bounce(n);
    };
    const tw = PX.step(t, 12);
    const wv = clamp((t - B(7, 3)) / 0.3);
    const wave = (i) => (wv > 0 ? Math.round(Math.sin(tw * 5.2 - i * 0.3) * 3 * wv) : 0);
    const sh = t >= T_ROAR ? -14 + (t - T_ROAR) * 150 : null;
    T.logo(160, LOGO_Y, { k: 4, letter, wave, shine: sh != null && sh < 80 ? sh : null, flash: t >= T_ROAR && t < T_ROAR + 1 / 30 ? P.bone : null });
    // Impacts: a white-core star, fat sparks and rings on the slam; smaller hits on bar 8.
    if (a < 3 / 30) T.star(LOGO_C[0], LOGO_C[1] + 6, a < 1 / 30 ? 70 : a < 2 / 30 ? 52 : 30, a < 2 / 30 ? P.cream : P.gold, P.bone);
    PX.ring(a, LOGO_C[0], LOGO_C[1] + 4, { r0: 16, r1: 240, life: 0.45, color: P.cream, color2: P.amber });
    PX.ring(a - 0.06, LOGO_C[0], LOGO_C[1] + 4, { r0: 10, r1: 170, life: 0.4, color: P.amber, color2: P.ember });
    PX.burst(a, LOGO_C[0], LOGO_C[1] + 30, { n: 40, seed: 7, speed: 230, gravity: 280, life: 0.9, colors: [P.cream, P.gold, P.amber, P.bone], size: 4, up: 80 });
    PX.burst(a, LOGO_C[0], LOGO_C[1] + 30, { n: 30, seed: 17, speed: 160, gravity: 240, life: 0.7, colors: [P.gold, P.ember], size: 3, up: 60 });
    for (const [h, seed] of [[T_HIT2, 21], [T_HIT3, 23]]) {
      const ha = t - h;
      PX.ring(ha, LOGO_C[0], LOGO_C[1] + 4, { r0: 18, r1: 200, life: 0.4, color: P.bone, color2: P.gold });
      PX.burst(ha, LOGO_C[0], LOGO_C[1] + 30, { n: 26, seed, speed: 190, gravity: 260, life: 0.7, colors: [P.bone, P.cream, P.gold], size: 3, up: 70 });
      if (ha >= 0 && ha < 2 / 30) T.star(LOGO_C[0], LOGO_C[1] + 6, ha < 1 / 30 ? 40 : 26, P.cream, P.bone);
    }
    const [sx, sy] = PX.shake(a, 7, 0.45, 11);
    const [sx2, sy2] = PX.shake(t - T_HIT2, 4, 0.3, 13);
    const [sx3, sy3] = PX.shake(t - T_HIT3, 5, 0.3, 15);
    const [sx4, sy4] = PX.shake(t - T_ROAR, 3, 0.25, 19);
    T.shakeFrame(sx + sx2 + sx3 + sx4, sy + sy2 + sy3 + sy4);
    // Last beat: punch in on Rex's roar and let 15.0 cut on the push.
    if (t >= T_HIT3) {
      const u = t - T_HIT3;
      T.punch(2, 178, 116 + Math.round(u * 12));
      T.speedLines(t, 160, 90, 0.8, 27, 1, [P.cream, P.gold, P.amber]);
    }
  }
  // Forward half-arc (right side only) around (cx, cy).
  function arc(cx, cy, r, c) {
    for (let d = -80; d <= 80; d += 4) {
      const an = (d * Math.PI) / 180;
      PX.rect(Math.round(cx + Math.cos(an) * r), Math.round(cy + Math.sin(an) * r * 0.9), 2, 2, c);
    }
  }
  // Embers sucked back into his mouth on the inhale.
  function inhale(t, x, y) {
    const P = PX.P, u = clamp((t - T_INHALE) / BEAT);
    for (let i = 0; i < 26; i++) {
      const ang = -1.2 + PX.hash(i, 61) * 2.2, d = (1 - ((u + PX.hash(i, 62)) % 1)) * 90;
      PX.rect(Math.round(x + Math.cos(ang) * d), Math.round(y + Math.sin(ang) * d * 0.8), 2, 2, i % 3 ? P.amber : P.gold);
    }
  }
  // The fire breath: a thick cone with a cream core, flickering at 12 fps, plus sparks.
  function breath(t, x, y) {
    const P = PX.P;
    const a = t - T_ROAR, fr = Math.floor(PX.step(a) * 12);
    const grow = clamp(a / 0.12);
    const len = 190 * grow, ang = -0.32, ca = Math.cos(ang), sa = Math.sin(ang);
    const bands = [[1, P.red], [0.8, P.ember], [0.6, P.amber], [0.4, P.gold], [0.2, P.cream]];
    for (let py = Math.max(0, Math.round(y - 110)); py < Math.min(H, Math.round(y + 60)); py++) {
      for (let px = Math.round(x); px < W; px++) {
        const dx = px - x, dy = py - y;
        const along = dx * ca + dy * sa, perp = -dx * sa + dy * ca;
        if (along < 0 || along > len) continue;
        const half = 5 + along * 0.26 + 3 * Math.sin(along * 0.09 - fr * 1.7);
        const r = Math.abs(perp) / half;
        if (r > 1) continue;
        let c = null;
        for (const [lim, col] of bands) if (r <= lim) c = col;
        const tip = along / len;
        if (tip > 0.85 && PX.bayer(px, py) < (tip - 0.85) / 0.15) continue;
        PX.g.fillStyle = c; PX.g.fillRect(px, py, 1, 1);
      }
    }
    PX.circle(x + 2, y, 6 + (fr % 2), P.cream, true);
    for (let i = 0; i < 18; i++) {
      const age = ((a * 2 + PX.hash(i, 33)) % 1) * 0.6;
      const sp = 180 + PX.hash(i, 34) * 120, an = ang + (PX.hash(i, 35) - 0.5) * 0.7;
      PX.rect(Math.round(x + Math.cos(an) * sp * age), Math.round(y + Math.sin(an) * sp * age - 20 * age), 3, 3, [P.cream, P.gold, P.amber][i % 3]);
    }
  }
  // Embers drifting up through the whole frame.
  function embers(t, a) {
    const P = PX.P;
    for (let i = 0; i < 40; i++) {
      const sp = 20 + PX.hash(i, 3) * 40;
      const y = H + 10 - ((a * sp + PX.hash(i, 4) * 220) % 220);
      const x = PX.hash(i, 5) * W + Math.sin(a * 2 + i) * 6;
      const c = [P.gold, P.amber, P.ember, P.cream][i % 4];
      const s = PX.hash(i, 6) < 0.3 ? 2 : 1;
      PX.rect(Math.round(x), Math.round(y), s, s, c);
    }
  }

  PX.scene({
    id: 'drop', start: T_SLAM0, end: T_END, layer: 0,
    draw(t) { if (window.TMX) drawDrop(t); },
  });
})();
