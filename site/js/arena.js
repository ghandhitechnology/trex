/* A pocket version of trex: real sprites, WASD, Space to dash, auto-fire.
   It pauses when the canvas loses focus, the same way trex pauses on tmux focus-out. */
(function () {
  'use strict';
  const cv = document.querySelector('.arena');
  if (!cv) return;
  const T = window.TREX;
  const ctx = cv.getContext('2d');
  const over = document.querySelector('.arena-over');
  const overText = over.querySelector('[data-px]');
  const score = document.querySelector('.arena-score');
  const W = 256, H = 144;
  const RM = matchMedia('(prefers-reduced-motion: reduce)').matches;
  const P = { ink: '#0f0b18', night: '#1d1629', dusk: '#2d2340', slate: '#45395c', bone: '#f5efe0', fog: '#c9c2d4', haze: '#948aa6', red: '#cc3a3f', ember: '#ef6b3a', amber: '#f7a041', gold: '#ffd25e', cream: '#fff4b0', sky: '#3e9ce0', cyan: '#74dcee', lime: '#7cc84b', grape: '#7a36a0', pink: '#da4f9e', sand: '#dca56f', clay: '#955c38' };
  const rand = (a, b) => a + Math.random() * (b - a);

  const IMG = {};
  const need = { rex: 'rex', rexIdle: 'rex-idle', grub: 'grub', mite: 'mite', slime: 'slime', slimelet: 'slimelet', bolt: 'bolt', gem: 'gem', heart: 'heart-full', heartE: 'heart-empty', skull: 'skull', tuft: 'p-tuft', rock: 'p-rock', bone: 'p-bone' };
  let loaded = 0;
  const total = Object.keys(need).length;
  Object.entries(need).forEach(([k, slug]) => {
    const s = T.spr[slug];
    const img = new Image();
    img.onload = () => { if (++loaded === total) { draw(0); } };
    img.src = 'assets/' + s[0];
    IMG[k] = { img, w: s[1], h: s[2], n: s[3] };
  });

  function spr(k, x, y, frame = 0, flip = false) {
    const s = IMG[k];
    if (!s || !s.img.complete) return;
    const f = frame % s.n;
    x = Math.round(x - s.w / 2); y = Math.round(y - s.h / 2);
    if (flip) {
      ctx.save(); ctx.translate(x + s.w, y); ctx.scale(-1, 1);
      ctx.drawImage(s.img, f * s.w, 0, s.w, s.h, 0, 0, s.w, s.h);
      ctx.restore();
    } else ctx.drawImage(s.img, f * s.w, 0, s.w, s.h, x, y, s.w, s.h);
  }

  // Floor: checker tiles plus a few props, drawn once.
  const floor = document.createElement('canvas');
  floor.width = W; floor.height = H;
  function paintFloor() {
    const f = floor.getContext('2d');
    for (let y = 0; y < H; y += 16) for (let x = 0; x < W; x += 16) {
      f.fillStyle = ((x + y) / 16) % 2 ? '#1b1528' : '#1e172c';
      f.fillRect(x, y, 16, 16);
      f.fillStyle = '#16111f';
      f.fillRect(x, y, 16, 1); f.fillRect(x, y, 1, 16);
    }
    const props = [['tuft', 30, 24], ['rock', 210, 30], ['bone', 60, 120], ['tuft', 180, 110], ['tuft', 236, 80], ['rock', 20, 90], ['bone', 150, 40]];
    for (const [k, x, y] of props) { const s = IMG[k]; if (s.img.complete) f.drawImage(s.img, 0, 0, s.w, s.h, x, y, s.w, s.h); }
  }

  let state;
  function reset() {
    state = {
      t: 0, kills: 0, xp: 0, lv: 1, next: 5, hp: 4,
      p: { x: W / 2, y: H / 2 + 8, vx: 0, vy: 0, face: 1, dash: 0, dcd: 0, inv: 0, trail: [] },
      foes: [], shots: [], gems: [], parts: [], nums: [], spawn: 0.6, fire: 0, rate: 0.5, shake: 0, banner: 0, dead: 0,
    };
  }
  reset();

  const keys = new Set();
  let pointer = null;
  let playing = false, raf = 0, last = 0, frame = 0;

  const ARROWS = { arrowup: 'w', arrowleft: 'a', arrowdown: 's', arrowright: 'd' };
  cv.addEventListener('keydown', (e) => {
    const k = ARROWS[e.key.toLowerCase()] || e.key.toLowerCase();
    if (['w', 'a', 's', 'd', ' '].includes(k)) e.preventDefault();
    if (k === ' ' && !e.repeat) dash();
    if (k === 'p' || k === 'escape') { cv.blur(); return; }
    keys.add(k);
  });
  cv.addEventListener('keyup', (e) => { keys.delete(ARROWS[e.key.toLowerCase()] || e.key.toLowerCase()); });
  const toLocal = (e) => { const r = cv.getBoundingClientRect(); return { x: (e.clientX - r.left) / r.width * W, y: (e.clientY - r.top) / r.height * H }; };
  cv.addEventListener('pointerdown', (e) => { pointer = toLocal(e); cv.setPointerCapture(e.pointerId); if (e.pointerType !== 'mouse' && e.detail > 1) dash(); });
  cv.addEventListener('pointermove', (e) => { if (pointer) pointer = toLocal(e); });
  const up = () => { pointer = null; };
  cv.addEventListener('pointerup', up); cv.addEventListener('pointercancel', up);
  cv.addEventListener('dblclick', dash);

  function start() {
    over.classList.add('off');
    playing = true;
    cv.focus({ preventScroll: true });
    if (!raf) { last = 0; raf = requestAnimationFrame(loop); }
  }
  function pause(label) {
    playing = false;
    keys.clear(); pointer = null;
    overText.textContent = label; delete overText.dataset.pxDone; PX.render(overText);
    over.classList.remove('off');
  }
  over.addEventListener('click', start);
  cv.addEventListener('focus', () => { if (!playing) start(); });
  cv.addEventListener('blur', () => { if (playing) pause('Paused'); });
  new IntersectionObserver(([e]) => { if (!e.isIntersecting && playing) cv.blur(); }).observe(cv);

  function dash() {
    const p = state.p;
    if (p.dcd > 0 || !playing) return;
    p.dash = 0.16; p.dcd = 1.0;
    // Rex: dashes knock enemies back.
    for (const f of state.foes) {
      const dx = f.x - p.x, dy = f.y - p.y, d = Math.hypot(dx, dy) || 1;
      if (d < 30) { f.kx = dx / d * 170; f.ky = dy / d * 170; }
    }
    puff(p.x, p.y + 6, 8, [P.fog, P.haze]);
  }
  function puff(x, y, n, cols, sp = 60) {
    for (let i = 0; i < n; i++) { const a = rand(0, Math.PI * 2), v = rand(sp * 0.3, sp); state.parts.push({ x, y, vx: Math.cos(a) * v, vy: Math.sin(a) * v, life: rand(0.25, 0.55), c: cols[(Math.random() * cols.length) | 0] }); }
  }

  const TYPES = {
    grub: { hp: 12, spd: 18, r: 6, cols: [P.sand, P.clay] },
    mite: { hp: 4, spd: 34, r: 4, cols: [P.red, P.ember] },
    slime: { hp: 22, spd: 13, r: 6, cols: [P.grape, P.pink], split: 'slimelet' },
    slimelet: { hp: 6, spd: 26, r: 4, cols: [P.grape, P.pink] },
  };
  function spawnFoe(kind, x, y) {
    const t = TYPES[kind];
    if (x === undefined) {
      const side = (Math.random() * 4) | 0;
      x = side === 0 ? -8 : side === 1 ? W + 8 : rand(0, W);
      y = side === 2 ? -8 : side === 3 ? H + 8 : rand(0, H);
      if (side < 2) y = rand(10, H - 10);
    }
    state.foes.push({ kind, x, y, hp: t.hp, t, kx: 0, ky: 0, flash: 0, ph: rand(0, 6) });
  }

  function update(dt) {
    const s = state, p = s.p;
    s.t += dt;
    if (s.dead > 0) { s.dead -= dt; if (s.dead <= 0) reset(); return; }
    // input
    let ix = 0, iy = 0;
    if (keys.has('a')) ix--; if (keys.has('d')) ix++;
    if (keys.has('w')) iy--; if (keys.has('s')) iy++;
    if (pointer) { const dx = pointer.x - p.x, dy = pointer.y - p.y, d = Math.hypot(dx, dy); if (d > 4) { ix = dx / d; iy = dy / d; } }
    const il = Math.hypot(ix, iy) || 1;
    const spd = 68 * (p.dash > 0 ? 3.2 : 1);
    const k = 1 - Math.exp(-dt * 14);
    p.vx += (ix / il * spd - p.vx) * k; p.vy += (iy / il * spd - p.vy) * k;
    if (p.dash > 0) { p.dash -= dt; p.trail.push({ x: p.x, y: p.y, life: 0.2, face: p.face }); }
    p.dcd = Math.max(0, p.dcd - dt); p.inv = Math.max(0, p.inv - dt);
    p.x = Math.max(8, Math.min(W - 8, p.x + p.vx * dt));
    p.y = Math.max(10, Math.min(H - 16, p.y + p.vy * dt));
    if (Math.abs(p.vx) > 4) p.face = p.vx < 0 ? -1 : 1;
    p.trail = p.trail.filter((t) => (t.life -= dt) > 0);

    // spawns
    s.spawn -= dt;
    if (s.spawn <= 0 && s.foes.length < 14) {
      const r = Math.random();
      spawnFoe(r < 0.55 ? 'grub' : r < 0.85 ? 'mite' : 'slime');
      s.spawn = Math.max(0.35, 1.1 - s.t * 0.01);
    }
    // foes
    for (const f of s.foes) {
      const dx = p.x - f.x, dy = p.y - f.y, d = Math.hypot(dx, dy) || 1;
      let vx = dx / d * f.t.spd, vy = dy / d * f.t.spd;
      if (f.kind === 'mite') { f.ph += dt * 8; vx += -dy / d * Math.sin(f.ph) * 30; vy += dx / d * Math.sin(f.ph) * 30; }
      f.x += (vx + f.kx) * dt; f.y += (vy + f.ky) * dt;
      f.kx *= Math.pow(0.02, dt); f.ky *= Math.pow(0.02, dt);
      f.flash = Math.max(0, f.flash - dt);
      if (d < f.t.r + 5 && p.inv <= 0 && p.dash <= 0) {
        s.hp--; p.inv = 1; s.shake = 0.25;
        puff(p.x, p.y, 10, [P.red, P.ember]);
        f.kx = -dx / d * 120; f.ky = -dy / d * 120;
        if (s.hp <= 0) { s.dead = 1.6; puff(p.x, p.y, 30, [P.red, P.bone, P.lime], 90); }
      }
    }
    // separation
    for (let i = 0; i < s.foes.length; i++) for (let j = i + 1; j < s.foes.length; j++) {
      const a = s.foes[i], b = s.foes[j];
      const dx = b.x - a.x, dy = b.y - a.y, d = Math.hypot(dx, dy), m = a.t.r + b.t.r - 2;
      if (d > 0 && d < m) { const push = (m - d) / 2; a.x -= dx / d * push; a.y -= dy / d * push; b.x += dx / d * push; b.y += dy / d * push; }
    }
    // auto-fire at the nearest foe
    s.fire -= dt;
    if (s.fire <= 0) {
      let best = null, bd = 150;
      for (const f of s.foes) { const d = Math.hypot(f.x - p.x, f.y - p.y); if (d < bd && f.x > -4 && f.x < W + 4 && f.y > -4 && f.y < H + 4) { bd = d; best = f; } }
      if (best) {
        const dx = best.x - p.x, dy = best.y - p.y, d = Math.hypot(dx, dy) || 1;
        s.shots.push({ x: p.x + p.face * 5, y: p.y - 2, vx: dx / d * 170, vy: dy / d * 170, life: 1 });
        s.fire = s.rate;
      } else s.fire = 0.1;
    }
    for (const b of s.shots) {
      b.x += b.vx * dt; b.y += b.vy * dt; b.life -= dt;
      for (const f of s.foes) {
        if (f.hp <= 0 || Math.hypot(f.x - b.x, f.y - b.y) > f.t.r + 2) continue;
        const dmg = 6 + ((Math.random() * 3) | 0);
        f.hp -= dmg; f.flash = 0.08; b.life = 0;
        f.kx += b.vx * 0.25; f.ky += b.vy * 0.25;
        s.nums.push({ x: f.x, y: f.y - 10, n: dmg, life: 0.6 });
        if (f.hp <= 0) {
          s.kills++;
          puff(f.x, f.y, 12, f.t.cols, 70);
          s.gems.push({ x: f.x, y: f.y, ph: 0 });
          if (f.t.split) { spawnFoe(f.t.split, f.x - 4, f.y); spawnFoe(f.t.split, f.x + 4, f.y); }
        } else puff(b.x, b.y, 3, [P.amber, P.gold], 40);
        break;
      }
    }
    s.shots = s.shots.filter((b) => b.life > 0 && b.x > -10 && b.x < W + 10 && b.y > -10 && b.y < H + 10);
    s.foes = s.foes.filter((f) => f.hp > 0);
    // gems
    for (const g of s.gems) {
      g.ph += dt;
      const dx = p.x - g.x, dy = p.y - g.y, d = Math.hypot(dx, dy);
      if (d < 30) { g.x += dx / d * 150 * dt; g.y += dy / d * 150 * dt; }
      if (d < 6) {
        g.got = true; s.xp++;
        if (s.xp >= s.next) { s.lv++; s.xp = 0; s.next = Math.round(s.next * 1.4); s.rate = Math.max(0.16, s.rate * 0.86); s.banner = 1.4; puff(p.x, p.y, 18, [P.cyan, P.gold, P.cream], 90); }
      }
    }
    s.gems = s.gems.filter((g) => !g.got);
    for (const q of s.parts) { q.x += q.vx * dt; q.y += q.vy * dt; q.vx *= 0.9; q.vy *= 0.9; q.life -= dt; }
    s.parts = s.parts.filter((q) => q.life > 0);
    for (const n of s.nums) { n.y -= 22 * dt; n.life -= dt; }
    s.nums = s.nums.filter((n) => n.life > 0);
    s.shake = Math.max(0, s.shake - dt);
    s.banner = Math.max(0, s.banner - dt);
  }

  function draw(t) {
    if (loaded < total) return;
    if (!floor._done) { paintFloor(); floor._done = true; }
    const s = state, p = s.p;
    ctx.imageSmoothingEnabled = false;
    ctx.save();
    if (s.shake > 0 && !RM) ctx.translate(Math.round(rand(-2, 2)), Math.round(rand(-2, 2)));
    ctx.drawImage(floor, 0, 0);
    // light pool around the player
    const g = ctx.createRadialGradient(p.x, p.y, 10, p.x, p.y, 70);
    g.addColorStop(0, 'rgba(116,220,238,0.10)'); g.addColorStop(1, 'rgba(116,220,238,0)');
    ctx.fillStyle = g; ctx.fillRect(0, 0, W, H);
    const fr = Math.floor(t / 180);
    for (const gm of s.gems) spr('gem', gm.x, gm.y + Math.round(Math.sin(gm.ph * 6)), Math.floor(gm.ph * 5));
    for (const f of s.foes) {
      ctx.fillStyle = 'rgba(15,11,24,.45)'; ctx.fillRect(Math.round(f.x - 5), Math.round(f.y + f.t.r - 1), 10, 2);
      spr(f.kind, f.x, f.y, fr + (f.ph * 3 | 0), f.x > p.x);
      if (f.flash > 0) { ctx.fillStyle = 'rgba(245,239,224,.8)'; ctx.fillRect(Math.round(f.x - f.t.r), Math.round(f.y - f.t.r), f.t.r * 2, f.t.r * 2 - 2); }
    }
    for (const tr of p.trail) { ctx.globalAlpha = 0.35; spr('rex', tr.x, tr.y, 0, tr.face < 0); }
    ctx.globalAlpha = 1;
    if (s.dead <= 0 && !(p.inv > 0 && Math.floor(t / 70) % 2)) {
      ctx.fillStyle = 'rgba(15,11,24,.5)'; ctx.fillRect(Math.round(p.x - 6), Math.round(p.y + 7), 12, 2);
      const moving = Math.hypot(p.vx, p.vy) > 10;
      spr(moving ? 'rex' : 'rexIdle', p.x, p.y, moving ? Math.floor(t / 120) : Math.floor(t / 300), p.face < 0);
    }
    for (const b of s.shots) spr('bolt', b.x, b.y, fr);
    for (const q of s.parts) { ctx.fillStyle = q.c; ctx.fillRect(Math.round(q.x), Math.round(q.y), 1, 1); }
    for (const n of s.nums) PX.drawText(ctx, n.n, Math.round(n.x - PX.measure(String(n.n)) / 2), Math.round(n.y), 1, P.bone, P.ink);
    ctx.restore();
    // HUD
    for (let i = 0; i < 4; i++) spr(i < s.hp ? 'heart' : 'heartE', 9 + i * 10, 8);
    const secs = Math.floor(s.t), clock = `${Math.floor(secs / 60)}:${String(secs % 60).padStart(2, '0')}`;
    PX.drawText(ctx, clock, Math.round(W / 2 - PX.measure(clock)), 4, 2, P.bone, P.ink);
    const kt = String(s.kills);
    spr('skull', W - 12 - PX.measure(kt) - 8, 8);
    PX.drawText(ctx, kt, W - 6 - PX.measure(kt), 6, 1, P.bone, P.ink);
    // xp bar
    ctx.fillStyle = P.ink; ctx.fillRect(4, H - 7, W - 8, 4);
    ctx.fillStyle = P.dusk; ctx.fillRect(5, H - 6, W - 10, 2);
    ctx.fillStyle = P.sky; ctx.fillRect(5, H - 6, Math.round((W - 10) * s.xp / s.next), 2);
    PX.drawText(ctx, 'LV' + s.lv, 5, H - 15, 1, P.gold, P.ink);
    if (s.banner > 0) { const txt = 'LEVEL UP'; PX.drawText(ctx, txt, Math.round(W / 2 - PX.measure(txt)), 40, 2, P.gold, P.ink); }
    if (s.dead > 0) { const txt = 'YOU DIED'; PX.drawText(ctx, txt, Math.round(W / 2 - PX.measure(txt) * 1.5), 60, 3, P.red, P.ink); }
    score.textContent = `${s.kills} kills · LV ${s.lv}`;
  }

  function loop(t) {
    const dt = Math.min(0.05, (t - (last || t)) / 1000); last = t;
    if (playing) update(dt);
    draw(t);
    raf = playing ? requestAnimationFrame(loop) : 0;
  }
})();
