(function () {
  'use strict';
  const T = window.TREX;
  const $ = (s, r = document) => r.querySelector(s);
  const $$ = (s, r = document) => Array.from(r.querySelectorAll(s));
  const RM = matchMedia('(prefers-reduced-motion: reduce)').matches;
  const FINE = matchMedia('(hover: hover) and (pointer: fine)').matches;
  const clamp = (v, a, b) => Math.max(a, Math.min(b, v));
  const rand = (a, b) => a + Math.random() * (b - a);
  const pick = (arr) => arr[(Math.random() * arr.length) | 0];
  const C = {
    ink: '#0f0b18', night: '#1d1629', dusk: '#2d2340', bone: '#f5efe0', fog: '#c9c2d4',
    red: '#cc3a3f', ember: '#ef6b3a', amber: '#f7a041', gold: '#ffd25e', cream: '#fff4b0',
    lime: '#7cc84b', sprout: '#cdeb72', leaf: '#2e904f', sky: '#3e9ce0', cyan: '#74dcee', ice: '#d2fbf6',
    grape: '#7a36a0', pink: '#da4f9e', blush: '#ff99c4', clay: '#955c38', sand: '#dca56f',
  };
  const store = {
    get(k) { try { return localStorage.getItem(k); } catch (e) { return null; } },
    set(k, v) { try { localStorage.setItem(k, v); } catch (e) { /* private mode */ } },
  };

  // ---------- Sprites ----------
  function spec(slug) {
    const s = T.spr[slug];
    if (!s) return null;
    return { src: 'assets/' + s[0], w: s[1], h: s[2], n: s[3] };
  }
  function sprite(sp, scale, opt = {}) {
    if (typeof sp === 'string') sp = spec(sp);
    const el = document.createElement('span');
    el.className = 'spr' + (sp.n > 1 && opt.anim !== false ? ' anim' : '') + (opt.flip ? ' flip' : '');
    el.style.cssText = `--w:${sp.w};--h:${sp.h};--n:${sp.n};--s:${scale}` + (opt.fd ? `;--fd:${opt.fd}ms` : '');
    const img = new Image();
    img.src = sp.src; img.alt = ''; img.decoding = 'async'; img.draggable = false;
    if (opt.lazy) img.loading = 'lazy';
    el.appendChild(img);
    return el;
  }
  const fit = (sp, box) => clamp(Math.floor(box / Math.max(sp.w, sp.h)), 1, 8);

  PX.renderAll();

  // ---------- FX canvas: pixel particles and floating text ----------
  const fx = (() => {
    const cv = $('.fx');
    const ctx = cv.getContext('2d');
    let parts = [], raf = 0, last = 0, dpr = 1;
    function size() {
      dpr = Math.min(2, window.devicePixelRatio || 1);
      cv.width = innerWidth * dpr; cv.height = innerHeight * dpr;
    }
    size();
    addEventListener('resize', size);
    function tick(t) {
      const dt = Math.min(0.05, (t - (last || t)) / 1000); last = t;
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      ctx.clearRect(0, 0, innerWidth, innerHeight);
      ctx.imageSmoothingEnabled = false;
      parts = parts.filter((p) => (p.life -= dt) > 0);
      for (const p of parts) {
        if (p.target) {
          const tr = p.target();
          const k = 1 - p.life / p.max;
          const ease = k * k * (3 - 2 * k);
          p.x = p.sx + (tr.x - p.sx) * ease + Math.sin(k * Math.PI) * p.arc;
          p.y = p.sy + (tr.y - p.sy) * ease - Math.sin(k * Math.PI) * 60;
          if (p.life - dt <= 0 && p.done) p.done();
        } else {
          p.vy += (p.g || 0) * dt;
          p.x += p.vx * dt; p.y += p.vy * dt;
        }
        const a = p.fade === false ? 1 : Math.min(1, p.life / (p.max * 0.4));
        ctx.globalAlpha = Math.round(a * 4) / 4;
        if (p.text) {
          const w = PX.measure(p.text) * p.size;
          PX.drawText(ctx, p.text, Math.round(p.x - w / 2), Math.round(p.y), p.size, p.color, C.ink);
        } else {
          ctx.fillStyle = p.color;
          ctx.fillRect(Math.round(p.x), Math.round(p.y), p.size, p.size);
        }
      }
      ctx.globalAlpha = 1;
      if (parts.length) raf = requestAnimationFrame(tick);
      else { raf = 0; last = 0; ctx.clearRect(0, 0, innerWidth, innerHeight); }
    }
    function add(p) {
      if (RM) return;
      p.max = p.life;
      parts.push(p);
      if (!raf) raf = requestAnimationFrame(tick);
    }
    return {
      burst(x, y, n, colors, o = {}) {
        for (let i = 0; i < n; i++) {
          const ang = o.up ? rand(-Math.PI * 0.85, -Math.PI * 0.15) : rand(0, Math.PI * 2);
          const sp = rand(o.min || 60, o.max || 180);
          add({ x: x + rand(-(o.spread || 0), o.spread || 0), y, vx: Math.cos(ang) * sp, vy: Math.sin(ang) * sp, g: o.g ?? 360, life: rand(0.45, 0.9), color: pick(colors), size: pick(o.sizes || [4, 4, 8]) });
        }
      },
      text(x, y, text, color, size = 3) {
        add({ x, y, vx: 0, vy: -70, g: 90, life: 0.9, text, color, size });
      },
      fly(x, y, target, color, done) {
        add({ sx: x, sy: y, x, y, target, life: 0.7, color, size: 8, arc: rand(-40, 40), done, fade: false });
      },
    };
  })();

  // ---------- Buttons: bursts and copy ----------
  $$('[data-burst]').forEach((b) => {
    let t = 0;
    b.addEventListener('pointerenter', () => {
      const now = performance.now();
      if (now - t < 700) return; t = now;
      const r = b.getBoundingClientRect();
      fx.burst(r.left + r.width / 2, r.top, 14, [C.gold, C.cream, C.amber, C.ember], { up: true, spread: r.width / 2, min: 80, max: 200, g: 420 });
    });
  });

  async function copyText(txt) {
    try { await navigator.clipboard.writeText(txt); return true; } catch (e) {
      const ta = document.createElement('textarea');
      ta.value = txt; ta.setAttribute('readonly', ''); ta.style.cssText = 'position:fixed;opacity:0';
      document.body.appendChild(ta); ta.select();
      let ok = false; try { ok = document.execCommand('copy'); } catch (e2) { ok = false; }
      ta.remove(); return ok;
    }
  }
  $$('[data-copy]').forEach((b) => {
    const label = b.textContent;
    b.addEventListener('click', async () => {
      const ok = await copyText(b.dataset.copy);
      if (!ok) return;
      b.textContent = 'Copied'; b.classList.add('done');
      const r = b.getBoundingClientRect();
      fx.burst(r.left + r.width / 2, r.top + r.height / 2, 16, [C.cyan, C.ice, C.sky], { min: 80, max: 220 });
      fx.text(r.left + r.width / 2, r.top - 18, 'COPIED!', C.cyan);
      clearTimeout(b._t);
      b._t = setTimeout(() => { b.textContent = label; b.classList.remove('done'); }, 1600);
    });
  });

  // ---------- Hero: props, light, terminal ----------
  const hero = $('.hero');
  (function props() {
    const box = $('.props');
    const kinds = ['p-tuft', 'p-tuft', 'p-rock', 'p-bone', 'p-fern', 'p-bloom', 'p-mossrock', 'p-skull', 'p-tuft-dim', 'p-ribs', 'gem'];
    let seed = 7;
    const rnd = () => ((seed = (seed * 16807) % 2147483647) / 2147483647);
    let placed = 0;
    while (placed < 22) {
      const x = rnd() * 96 + 1, y = rnd() * 92 + 4;
      const k = kinds[(rnd() * kinds.length) | 0];
      if (x < 72 && y < 62) continue; // keep the copy clear
      const el = sprite(k, 4, { anim: false });
      el.style.left = x.toFixed(1) + '%';
      el.style.top = y.toFixed(1) + '%';
      box.appendChild(el);
      placed++;
    }
  })();

  const lit = $('.floor-lit');
  if (FINE && !RM) {
    let mx = 0, my = 0, pend = false;
    hero.addEventListener('pointermove', (e) => {
      const r = hero.getBoundingClientRect();
      mx = e.clientX - r.left; my = e.clientY - r.top;
      if (!pend) { pend = true; requestAnimationFrame(() => { pend = false; lit.style.setProperty('--mx', (mx | 0) + 'px'); lit.style.setProperty('--my', (my | 0) + 'px'); }); }
    });
  }

  // tmux status clock
  const clock = $('.tmux-status .clock');
  const tickClock = () => { const d = new Date(); clock.textContent = String(d.getHours()).padStart(2, '0') + ':' + String(d.getMinutes()).padStart(2, '0'); };
  tickClock(); setInterval(tickClock, 15000);

  // Agent pane: a generic coding agent grinding through a task.
  const log = $('.agent-log');
  const TASKS = [
    { ask: 'fix the flaky ledger tests', read: ['tests/ledger_test.rs', 'src/ledger/mod.rs', 'src/ledger/refund.rs'], found: 'refund() races the balance write.', edit: ['src/ledger/refund.rs', 18, 6], test: 'cargo test ledger', total: 412 },
    { ask: 'move sessions to the new store', read: ['src/auth/session.rs', 'src/store/mod.rs', 'migrations/0042_sessions.sql'], found: 'three call sites still hit the old cache.', edit: ['src/auth/session.rs', 64, 41], test: 'cargo test auth', total: 188 },
    { ask: 'bump deps and fix what breaks', read: ['Cargo.toml', 'src/http/client.rs', 'src/http/retry.rs'], found: 'reqwest 0.13 renamed the timeout builder.', edit: ['src/http/client.rs', 9, 9], test: 'cargo test', total: 1204 },
  ];
  const SPIN = ['▖', '▘', '▝', '▗'];
  const MAXL = 40;
  function line(html) {
    const s = document.createElement('span');
    s.className = 'l'; s.innerHTML = html || ' ';
    log.appendChild(s);
    while (log.children.length > MAXL) log.firstChild.remove();
    return s;
  }
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
  async function agentLoop() {
    let n = 0;
    for (;;) {
      const t = TASKS[n++ % TASKS.length];
      if (n > 1) { line(''); line(''); }
      line(`<span class="dim">~/api</span> <span class="ok">❯</span> <span class="you">agent "${t.ask}"</span>`);
      line('');
      await sleep(900);
      for (const f of t.read) { line(`<span class="tool">●</span> Read <span class="dim">${f}</span>`); await sleep(rand(500, 900)); }
      line(`<span class="warn">●</span> Found it: ${t.found}`); await sleep(1300);
      line(`<span class="tool">●</span> Edit <span class="dim">${t.edit[0]}</span> <span class="add">+${t.edit[1]}</span> <span class="del">-${t.edit[2]}</span>`); await sleep(1100);
      line(`<span class="tool">●</span> Bash <span class="dim">${t.test}</span>`);
      const bar = line('');
      const W = 16;
      let done = 0, k = 0;
      const start = performance.now();
      while (done < t.total) {
        done = Math.min(t.total, done + Math.ceil(rand(0.004, 0.02) * t.total));
        const f = Math.round((done / t.total) * W);
        const secs = ((performance.now() - start) / 1000).toFixed(0);
        bar.innerHTML = `  <span class="bar">${'█'.repeat(f)}</span><span class="dim">${'░'.repeat(W - f)}</span> ${done}/${t.total} <span class="dim">${SPIN[k++ % 4]} ${secs}s</span>`;
        await sleep(rand(260, 520));
      }
      bar.innerHTML = `  <span class="ok">✓ ${t.total} passed</span> <span class="dim">in ${((performance.now() - start) / 1000).toFixed(1)}s</span>`;
      await sleep(700);
      line(`<span class="ok">●</span> Done. Waiting on your review.`);
      await sleep(7000);
    }
  }

  const pane = $('.pane-trex');
  const typed = $('.typed');
  const reel = $('.reel');
  function bootReel() {
    pane.classList.add('booted');
    if (RM) return;
    const p = reel.play();
    if (p && p.catch) p.catch(() => {});
  }
  async function terminal() {
    if (RM) {
      typed.textContent = 'trex';
      line(`<span class="dim">~/api</span> <span class="ok">❯</span> <span class="you">agent "${TASKS[0].ask}"</span>`);
      line(`<span class="tool">●</span> Bash <span class="dim">${TASKS[0].test}</span>`);
      bootReel();
      return;
    }
    agentLoop();
    await sleep(1150);
    for (const ch of 'trex') { typed.textContent += ch; await sleep(rand(90, 150)); }
    await sleep(320);
    bootReel();
  }

  // Tilt the terminal flat as the hero scrolls away.
  const term = $('.term');
  if (!RM) {
    let pend = false;
    const onScroll = () => {
      if (pend) return; pend = true;
      requestAnimationFrame(() => {
        pend = false;
        const p = clamp(scrollY / (innerHeight * 0.55), 0, 1);
        term.style.setProperty('--tilt', (9 * (1 - p)).toFixed(2) + 'deg');
        term.style.setProperty('--tscale', (0.965 + 0.035 * p).toFixed(4));
      });
    };
    addEventListener('scroll', onScroll, { passive: true });
    onScroll();
    // Pause the reel when it scrolls out of view.
    new IntersectionObserver(([e]) => {
      if (!pane.classList.contains('booted')) return;
      if (e.isIntersecting) { const p = reel.play(); if (p && p.catch) p.catch(() => {}); } else reel.pause();
    }).observe(pane);
  }

  // ---------- Entry ----------
  requestAnimationFrame(() => requestAnimationFrame(() => {
    document.body.classList.add('ready');
    terminal();
  }));

  // ---------- Nav: XP bar, level, active link ----------
  const secs = $$('main > section');
  const xpFill = $('.xp-fill');
  const lvEl = $('.xp-lv');
  let lv = 0;
  const navLinks = $$('.nav-links a');
  function onScrollNav() {
    const max = document.documentElement.scrollHeight - innerHeight;
    const p = max > 0 ? clamp(scrollY / max, 0, 1) : 0;
    xpFill.style.transform = `scaleX(${p.toFixed(4)})`;
    const line = scrollY + innerHeight * 0.5;
    let n = 1;
    for (const s of secs) if (s.offsetTop < line) n = secs.indexOf(s) + 1;
    if (lv && n > lv && !RM) {
      lvEl.classList.remove('up'); void lvEl.offsetWidth; lvEl.classList.add('up');
      const r = lvEl.getBoundingClientRect();
      fx.burst(r.left + r.width / 2, r.bottom, 10, [C.cyan, C.sky, C.gold], { min: 40, max: 140 });
    }
    lv = n;
    lvEl.textContent = 'LV ' + lv;
    let cur = null;
    for (const s of secs) if (s.id && s.offsetTop - 120 < scrollY) cur = s.id;
    navLinks.forEach((a) => a.classList.toggle('on', a.getAttribute('href') === '#' + cur));
  }
  let navPend = false;
  addEventListener('scroll', () => { if (!navPend) { navPend = true; requestAnimationFrame(() => { navPend = false; onScrollNav(); }); } }, { passive: true });
  addEventListener('resize', onScrollNav);
  onScrollNav();

  // ---------- Reveal ----------
  const rv = new IntersectionObserver((es) => {
    for (const e of es) if (e.isIntersecting) { e.target.classList.add('in'); rv.unobserve(e.target); }
  }, { threshold: 0.12, rootMargin: '0px 0px -6% 0px' });
  $$('[data-reveal]').forEach((el) => rv.observe(el));

  // ---------- Lazy videos ----------
  const canWebm = document.createElement('video').canPlayType('video/webm; codecs="vp9"') !== '';
  const lazyV = new IntersectionObserver((es) => {
    for (const e of es) {
      const v = e.target;
      if (e.isIntersecting) {
        if (!v.dataset.loaded) {
          v.querySelectorAll('source[data-src]').forEach((s) => { s.src = s.dataset.src; });
          v.dataset.loaded = '1';
          v.load();
        }
        if (!RM) { const p = v.play(); if (p && p.catch) p.catch(() => {}); }
      } else if (v.dataset.loaded) v.pause();
    }
  }, { rootMargin: '200px 0px' });
  $$('video[data-lazy]').forEach((v) => lazyV.observe(v));

  // ---------- Marquee ----------
  (function marquee() {
    const track = $('.marquee-track');
    const phrases = ['Runs in Ghostty', 'Kitty', 'tmux', '60 fps in a pane', '8 fps when you look away', 'One Rust binary', 'Every pixel drawn in code', 'No sound'];
    const walkers = ['rex', 'grub', 'slime', 'beetle', 'wisp', 'frog', 'mite', 'ptero'];
    const build = () => {
      const f = document.createDocumentFragment();
      phrases.forEach((p, i) => {
        const h = document.createElement('span');
        h.setAttribute('data-px', ''); h.textContent = p;
        f.appendChild(h);
        f.appendChild(sprite(walkers[i % walkers.length], 3));
      });
      return f;
    };
    track.appendChild(build());
    track.appendChild(build());
    PX.renderAll(track);
  })();

  // ---------- Run player ----------
  (function run() {
    const CH = [
      { id: 'early', t: 23, name: 'First picks', hero: 'rex', d: 'Empty arena, first level-up. Bone Storm, Cold Snap or Iron Jaw?' },
      { id: 'boss-mire-queen', t: 185, name: 'Mire Queen', hero: 'rex', boss: 1, d: 'First boss. A crowned slime that rings the arena in pink globs.' },
      { id: 'boss-colossus', t: 368, name: 'Bone Colossus', hero: 'rex', boss: 1, d: 'A triceratops skeleton. Explosion rings, cyan spray, no room to breathe.' },
      { id: 'lightning-build', t: 611, name: 'Lightning build', hero: 'ptera', d: 'Bone Dunes on a two-heart bird. Meteors overhead, chain lightning below.' },
      { id: 'boss-sandmaw', t: 745, name: 'Sandmaw', hero: 'spino', boss: 1, d: 'A level-up mid-fight, then the floor rolls over into Spore Marsh.' },
      { id: 'late-chaos', t: 870, name: 'Level 41', hero: 'rex', d: 'Hundreds on screen, damage numbers everywhere, 7,600 kills and counting.' },
    ];
    const list = $('.chapters');
    const marks = $('.track-marks');
    const video = $('.run-video');
    const poster = $('.run-poster');
    const vid = $('.run .vid');
    const label = $('.run-label');
    const count = $('.run-count');
    const fill = $('.track-fill');
    const fmt = (s) => `${Math.floor(s / 60)}:${String(Math.floor(s % 60)).padStart(2, '0')}`;
    let cur = 0, visible = false, raf = 0;
    CH.forEach((c, i) => {
      const li = document.createElement('li');
      const b = document.createElement('button');
      b.type = 'button'; b.className = 'chapter';
      b.innerHTML = `<span class="t">${fmt(c.t)}</span><span class="n">${c.name}</span><span class="d"><span>${c.d}</span></span>`;
      b.querySelector('.n').appendChild(Object.assign(sprite(c.hero, 2), { className: 'spr anim who' }));
      b.addEventListener('click', () => go(i, true));
      li.appendChild(b); list.appendChild(li);
      const m = document.createElement('button');
      m.type = 'button'; m.className = 'track-mark' + (c.boss ? ' is-boss' : '');
      m.style.left = (c.t / 900 * 100) + '%';
      m.tabIndex = -1;
      m.setAttribute('aria-label', c.name);
      m.addEventListener('click', () => go(i, true));
      marks.appendChild(m);
    });
    const btns = $$('.chapter', list);
    const mks = $$('.track-mark', marks);
    function go(i, user) {
      cur = (i + CH.length) % CH.length;
      const c = CH[cur];
      btns.forEach((b, k) => { b.classList.toggle('on', k === cur); b.setAttribute('aria-current', k === cur ? 'true' : 'false'); });
      mks.forEach((m, k) => m.classList.toggle('on', k === cur));
      label.textContent = `${fmt(c.t)} · ${c.hero} · ${c.name}`;
      count.textContent = `${cur + 1} / ${CH.length}`;
      if (!RM) { vid.classList.remove('swapping'); void vid.offsetWidth; vid.classList.add('swapping'); }
      poster.src = `assets/video/${c.id}.png`;
      video.poster = `assets/video/${c.id}.png`;
      fill.style.transform = `scaleX(${c.t / 900})`;
      video.src = `assets/video/${c.id}.${canWebm ? 'webm' : 'mp4'}`;
      if (visible && (!RM || user)) { const p = video.play(); if (p && p.catch) p.catch(() => {}); }
      if (user && btns[cur] && innerWidth <= 1100) btns[cur].scrollIntoView({ block: 'nearest', inline: 'nearest', behavior: RM ? 'auto' : 'smooth' });
    }
    video.addEventListener('ended', () => go(cur + 1));
    function loop() {
      const c = CH[cur];
      if (!video.paused) fill.style.transform = `scaleX(${clamp((c.t + video.currentTime) / 900, 0, 1)})`;
      raf = visible ? requestAnimationFrame(loop) : 0;
    }
    let started = false;
    new IntersectionObserver(([e]) => {
      visible = e.isIntersecting;
      if (visible) {
        if (!started) { started = true; go(0); }
        else if (!RM) { const p = video.play(); if (p && p.catch) p.catch(() => {}); }
        if (!raf) raf = requestAnimationFrame(loop);
      } else video.pause();
    }, { rootMargin: '100px 0px' }).observe(vid);
    if (RM) vid.addEventListener('click', () => { if (video.paused) video.play().catch(() => {}); else video.pause(); });
  })();

  // ---------- Pipeline steps ----------
  (function pipe() {
    const pvs = $$('#pipe-screen .pv');
    const steps = $$('.step');
    steps.forEach((s, i) => {
      const box = $('.inline-vis', s);
      const wrap = document.createElement('div');
      wrap.className = 'pipe-screen px-frame';
      const clone = pvs[i].cloneNode(true);
      clone.classList.add('on');
      wrap.appendChild(clone);
      box.appendChild(wrap);
    });
    // Type the escape sequences out when their panel shows.
    function typeOut(root) {
      if (RM) return;
      const nodes = [];
      const walk = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
      while (walk.nextNode()) nodes.push([walk.currentNode, walk.currentNode.textContent]);
      nodes.forEach(([n]) => { n.textContent = ''; });
      let i = 0, j = 0;
      clearInterval(root._t);
      root._t = setInterval(() => {
        for (let k = 0; k < 4 && i < nodes.length; k++) {
          const [n, full] = nodes[i];
          n.textContent = full.slice(0, ++j);
          if (j >= full.length) { i++; j = 0; }
        }
        if (i >= nodes.length) clearInterval(root._t);
      }, 16);
    }
    // Packet speed follows the width of each connector.
    const sizeFlows = () => $$('.flow i').forEach((i) => i.style.setProperty('--fw', i.offsetWidth + 'px'));
    addEventListener('resize', sizeFlows);
    let active = -1;
    const io = new IntersectionObserver((es) => {
      for (const e of es) {
        if (!e.isIntersecting) continue;
        const i = +e.target.dataset.step;
        e.target.classList.add('seen');
        steps.forEach((s, k) => s.classList.toggle('on', k === i));
        pvs.forEach((p, k) => p.classList.toggle('on', k === i));
        if (i !== active) { active = i; const esc = $('.esc', pvs[i]); if (esc) typeOut(esc); sizeFlows(); }
      }
    }, { rootMargin: '-45% 0px -45% 0px' });
    steps.forEach((s) => io.observe(s));
    // headings on mobile reveal when seen at all
    const io2 = new IntersectionObserver((es) => { for (const e of es) if (e.isIntersecting) e.target.classList.add('seen'); }, { threshold: 0.3 });
    steps.forEach((s) => io2.observe(s));
  })();

  // ---------- Hub: pick a dino ----------
  (function hub() {
    const tiles = $('.tiles');
    const dino = $('.hub-dino');
    const title = $('.hub-title');
    const unlock = $('.hub-unlock');
    const desc = $('.hub-desc');
    const hearts = $('.hearts');
    const weapon = $('.hub-weapon');
    const bars = $$('.bar i');
    const TRICK = { rex: 'Dash: shockwave', ptera: 'Dash: feather volley', trike: 'Hurt: explode', raptor: 'Crit: set them on fire', spino: 'Hit: slow', stego: 'Hurt: ring of 12 spikes', pachy: 'Dash: headbutt blast' };
    const SHOT = { bolt: 'ember bolt', 'feather-shot': 'twin feathers', horn: 'piercing horn', claw: 'short claw', bubble: 'homing bubble', spike: 'spike ring', pebble: 'pebble spread' };
    const GOAL = (g) => {
      const m = g.match(/(\w+)\(([\d.]+)\)/); if (!m) return g;
      const v = +m[2];
      const t = (s) => `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`;
      return ({ Survive: `survive ${t(v)}`, Level: `reach LV ${v}`, Kills: `${v} kills in a run`, TotalKills: `${v.toLocaleString('en-US')} kills total`, Runs: `play ${v} runs` })[m[1]] || g;
    };
    let cur = 0;
    const tabs = T.heroes.map((h, i) => {
      const li = document.createElement('li');
      const b = document.createElement('button');
      b.type = 'button'; b.className = 'tile'; b.setAttribute('role', 'tab');
      b.setAttribute('aria-label', h.name);
      b.appendChild(sprite(h.idle, 4, { fd: 260 }));
      if (h.unlock) {
        const l = sprite('lock', 3); l.classList.add('lock'); b.appendChild(l);
        const p = document.createElement('span'); p.className = 'price'; p.textContent = h.unlock + ' BONES'; b.appendChild(p);
      }
      b.addEventListener('click', () => select(i));
      li.appendChild(b); tiles.appendChild(li);
      return b;
    });
    function select(i, focus) {
      cur = (i + T.heroes.length) % T.heroes.length;
      const h = T.heroes[cur];
      tabs.forEach((b, k) => { b.setAttribute('aria-selected', k === cur ? 'true' : 'false'); b.tabIndex = k === cur ? 0 : -1; });
      if (focus) tabs[cur].focus();
      dino.classList.add('swap');
      setTimeout(() => {
        const old = $('.spr', dino); if (old) old.remove();
        dino.appendChild(sprite(h.idle, innerWidth < 720 ? 10 : 12, { fd: 300 }));
        requestAnimationFrame(() => dino.classList.remove('swap'));
      }, RM ? 0 : 120);
      title.textContent = h.name; delete title.dataset.pxDone; PX.render(title);
      unlock.innerHTML = h.unlock ? `<b>${h.unlock}</b> bones${h.feat ? ` · or ${h.feat.name}: ${GOAL(h.feat.goal)}` : ''}` : '<b>Starter</b>';
      desc.textContent = h.desc;
      hearts.innerHTML = '';
      for (let k = 0; k < Math.ceil(h.hp / 2); k++) hearts.appendChild(sprite(h.hp - k * 2 >= 2 ? 'heart-full' : 'heart-half', 3));
      bars.forEach((b) => b.style.setProperty('--v', h[b.dataset.k + 'Bar']));
      weapon.innerHTML = '';
      weapon.appendChild(sprite(h.shot, 4));
      const t = document.createElement('span');
      t.innerHTML = `Shoots <b>${SHOT[h.shot] || h.shot}</b> · ${TRICK[h.id] || ''}`;
      weapon.appendChild(t);
    }
    select(0);
    tiles.addEventListener('keydown', (e) => {
      if (e.key === 'ArrowRight') { select(cur + 1, true); e.preventDefault(); }
      if (e.key === 'ArrowLeft') { select(cur - 1, true); e.preventDefault(); }
    });
    let inView = false;
    new IntersectionObserver(([e]) => { inView = e.intersectionRatio > 0.45; }, { threshold: [0, 0.45, 0.6] }).observe($('.hub'));
    addEventListener('keydown', (e) => {
      if (!inView || e.metaKey || e.ctrlKey || e.altKey || document.activeElement?.closest('.arena, input, textarea')) return;
      if (e.key === 'a' || e.key === 'A') select(cur - 1);
      if (e.key === 'd' || e.key === 'D') select(cur + 1);
    });
  })();

  // ---------- Builds: dealer, synergies, armory ----------
  (function builds() {
    const items = T.items;
    const byId = Object.fromEntries(items.map((it) => [it.id, it]));
    const SYN_FALLBACK = { napalm: 'bolt', thunder_dash: 'spark' };
    const cardsEl = $('.cards');
    const tray = $('.tray');
    const synsEl = $('.syns');
    const tip = $('.tip');
    const held = [];
    const lit = new Set();
    let hand = [];

    // Synergy list
    $('.syn-total').textContent = T.synergies.length;
    const synRows = T.synergies.map((s) => {
      const li = document.createElement('li');
      li.className = 'syn'; li.dataset.id = s.id;
      const eq = document.createElement('span'); eq.className = 'eq';
      s.needs.forEach((n, k) => {
        if (k) eq.append('+');
        eq.appendChild(sprite(byId[n].sprite, 2));
      });
      eq.append('=');
      const res = document.createElement('span'); res.className = 'res';
      const sp = s.sprite || spec(SYN_FALLBACK[s.id] || 'spark');
      res.appendChild(sprite(sp, fit(sp, 30)));
      eq.appendChild(res);
      li.appendChild(eq);
      li.insertAdjacentHTML('beforeend', `<span class="nm">${s.name}</span><span class="ds">${s.desc}</span>`);
      li.addEventListener('pointerenter', () => hot(s.needs));
      li.addEventListener('pointerleave', () => hot([]));
      synsEl.appendChild(li);
      return li;
    });

    // Armory
    const groups = [['weapon', 'Weapons', 'fire on their own'], ['active', 'Actives', 'fire on Space'], ['passive', 'Passives', 'always on']];
    const armory = $('.armory-groups');
    const iconEls = {};
    groups.forEach(([k, name, sub]) => {
      const g = document.createElement('div');
      g.className = 'agroup ' + k;
      g.innerHTML = `<h4>${name}<span>${sub}</span></h4>`;
      const wrap = document.createElement('div'); wrap.className = 'icons';
      items.filter((it) => it.kind === k).forEach((it) => {
        const b = document.createElement('button');
        b.type = 'button';
        b.className = 'icon' + (it.rarity !== 'Common' ? ' ' + it.rarity.toLowerCase() : '');
        b.setAttribute('aria-label', `${it.name}: ${it.desc}`);
        b.appendChild(sprite(it.sprite, 2));
        const show = () => {
          tip.innerHTML = `<b>${it.name}</b>${it.desc}<span class="r ${it.rarity.toLowerCase()}">${it.rarity}${it.unlock ? ` · unlock ${it.unlock} bones` : ''}</span>`;
          const r = b.getBoundingClientRect();
          tip.classList.add('on');
          const tr = tip.getBoundingClientRect();
          tip.style.left = clamp(r.left + r.width / 2 - tr.width / 2, 8, innerWidth - tr.width - 8) + 'px';
          tip.style.top = (r.top - tr.height - 14 < 8 ? r.bottom + 14 : r.top - tr.height - 14) + 'px';
          synRows.forEach((row, i) => row.classList.toggle('peek', T.synergies[i].needs.includes(it.id)));
        };
        const hide = () => tip.classList.remove('on');
        b.addEventListener('pointerenter', show); b.addEventListener('focus', show);
        b.addEventListener('pointerleave', hide); b.addEventListener('blur', hide);
        b.addEventListener('click', () => { if (!held.includes(it.id)) take(it.id, b); });
        iconEls[it.id] = b;
        wrap.appendChild(b);
      });
      g.appendChild(wrap);
      armory.appendChild(g);
    });
    addEventListener('scroll', () => tip.classList.remove('on'), { passive: true });
    function hot(ids) { Object.entries(iconEls).forEach(([id, el]) => el.classList.toggle('hot', ids.includes(id))); }

    // Dealer
    function partners() {
      const out = [];
      for (const s of T.synergies) {
        if (lit.has(s.id)) continue;
        const have = s.needs.filter((n) => held.includes(n));
        if (have.length) s.needs.forEach((n) => { if (!held.includes(n)) out.push(n); });
      }
      return out;
    }
    function deal() {
      const pool = items.filter((it) => !held.includes(it.id)).map((it) => it.id);
      const p = partners().filter((id) => pool.includes(id));
      hand = [];
      if (p.length && Math.random() < 0.6) hand.push(pick(p));
      while (hand.length < 3 && hand.length < pool.length) {
        const id = pick(pool);
        if (!hand.includes(id)) hand.push(id);
      }
      hand.sort(() => Math.random() - 0.5);
      cardsEl.innerHTML = '';
      hand.forEach((id, k) => {
        const it = byId[id];
        const b = document.createElement('button');
        b.type = 'button'; b.className = 'card deal-in'; b.style.setProperty('--k', k);
        const tag = it.rarity === 'Common' ? '<span class="tag">NEW</span>' : `<span class="tag ${it.rarity.toLowerCase()}">${it.rarity.toUpperCase()}</span>`;
        b.innerHTML = `${tag}<span class="glow"></span><span class="nm">${it.name}</span><span class="ds">${it.desc}</span><span class="kn">${k + 1}</span>`;
        $('.glow', b).appendChild(sprite(it.sprite, 4));
        b.setAttribute('aria-label', `Take ${it.name}: ${it.desc}`);
        b.addEventListener('click', () => take(id, b));
        cardsEl.appendChild(b);
      });
      if (!hand.length) cardsEl.innerHTML = '<p class="ds" style="grid-column:1/-1;margin:0;color:var(--muted)">Nothing left to offer. Reset the run.</p>';
    }
    function take(id, from) {
      if (held.length >= 14) return;
      held.push(id);
      const it = byId[id];
      const r = from.getBoundingClientRect();
      if (from.classList.contains('card')) from.classList.add('taken');
      if (tray.querySelector('.empty')) tray.innerHTML = '';
      const slot = document.createElement('span');
      slot.className = 'slot'; slot.title = it.name;
      slot.appendChild(sprite(it.sprite, 2));
      tray.appendChild(slot);
      fx.burst(r.left + r.width / 2, r.top + r.height / 3, 10, [C.cream, C.gold, C.bone], { min: 60, max: 180 });
      iconEls[id].style.opacity = '.35';
      // synergies
      for (const s of T.synergies) {
        if (lit.has(s.id) || !s.needs.every((n) => held.includes(n))) continue;
        lit.add(s.id);
        const row = synRows[T.synergies.indexOf(s)];
        row.classList.add('lit');
        const rr = row.getBoundingClientRect();
        fx.burst(rr.left + 30, rr.top + rr.height / 2, 22, [C.gold, C.cream, C.amber, C.ember], { min: 80, max: 260 });
        fx.text(r.left + r.width / 2, r.top - 10, s.name + '!', C.gold, 4);
      }
      $('.syn-count').textContent = lit.size;
      setTimeout(deal, RM ? 0 : 240);
    }
    function reset() {
      held.length = 0; lit.clear();
      tray.innerHTML = '<span class="empty">Your build is empty. Take a card.</span>';
      synRows.forEach((r) => r.classList.remove('lit'));
      Object.values(iconEls).forEach((el) => { el.style.opacity = ''; });
      $('.syn-count').textContent = 0;
      deal();
    }
    $('.deal-reset').addEventListener('click', reset);
    $('.deal-reroll').addEventListener('click', deal);
    deal();
    let inView = false;
    new IntersectionObserver(([e]) => { inView = e.isIntersecting; }, { threshold: 0.5 }).observe($('.deal'));
    addEventListener('keydown', (e) => {
      if (!inView || e.metaKey || e.ctrlKey || e.altKey || document.activeElement?.closest('.arena, input, textarea')) return;
      const n = +e.key;
      if (n >= 1 && n <= 3 && hand[n - 1]) { const b = cardsEl.children[n - 1]; if (b) take(hand[n - 1], b); }
      if (e.key === 'r' || e.key === 'R') deal();
    });
  })();

  // ---------- Bestiary ----------
  (function bestiary() {
    const DESC = {
      grub: 'Walks at you. There are always more.', wisp: 'Weaves in fast and fades on death.', mite: 'Tiny, quick, arrives in clouds.',
      slime: 'Splits into two slimelets when popped.', slimelet: 'What is left of a slime. Still angry.', shellback: 'A 30 point shell soaks hits before the HP.',
      beetle: 'Winds up, then charges in a straight line.', brute: 'Charges. Hits for a full heart.', ptero: 'Circles you, then dives.',
      boomer: 'Runs in, lights the fuse, explodes.', golem: 'Slow. Slams the ground in a wide ring.', frog: 'Spits from range.',
      dilo: 'Three-way spit. Keep moving.', puffer: 'Puffs a ring of eight spores.', eyestalk: 'Snipes needles in bursts of three.',
      mole: 'Burrows, surfaces, sprays grit.', ghost: 'Blinks next to you and fires orbs.', broodmother: 'Births mites while you watch.',
    };
    const bosses = $('.bosses');
    const fmt = (s) => `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`;
    T.bosses.forEach((b, k) => {
      const el = document.createElement('div');
      el.className = 'boss'; el.style.setProperty('--k', k);
      el.innerHTML = `<span class="at">${fmt((k + 1) * 180)}</span><div class="pad"></div><span class="bn">${b.name}</span><span class="hp"><i style="--v:${(b.hp / 600).toFixed(3)}"></i></span><span class="hpn">${b.hp} HP</span>`;
      $('.pad', el).appendChild(sprite(b.sprite, innerWidth < 720 ? 4 : 5, { fd: 200 }));
      bosses.appendChild(el);
    });
    const beasts = $('.beasts');
    T.enemies.forEach((e) => {
      const li = document.createElement('li');
      li.className = 'beast';
      li.innerHTML = `<div class="fig"></div><span class="nm">${e.id}</span><span class="ds">${DESC[e.id] || ''}</span><span class="hpn">${e.hp} HP · SPD ${e.speed}</span>`;
      $('.fig', li).appendChild(sprite(e.sprite, 4, { lazy: true }));
      beasts.appendChild(li);
    });
    const STAGES = ['Tar Pits', 'Fern Hollow', 'Ember Flats', 'Frost Caves', 'Bone Dunes', 'Spore Marsh', 'Storm Peak', 'The Rift'];
    const ol = $('.stages');
    STAGES.forEach((s, i) => { ol.insertAdjacentHTML('beforeend', `<li><span>${fmt(i * 150)}</span>${s.toUpperCase()}</li>`); });
    ol.insertAdjacentHTML('beforeend', `<li class="loop"><span>20:00</span>BACK TO FERN HOLLOW, HARDER</li>`);
  })();

  // ---------- Keycaps light up with real keys ----------
  (function keys() {
    const map = { w: 'w', arrowup: 'w', a: 'a', arrowleft: 'a', s: 's', arrowdown: 's', d: 'd', arrowright: 'd', ' ': ' ', p: 'p', escape: 'p', q: 'q' };
    const el = {};
    $$('.key[data-key]').forEach((k) => { el[k.dataset.key] = k; });
    const set = (e, on) => { const k = map[e.key.toLowerCase()]; if (k && el[k]) el[k].classList.toggle('down', on); };
    addEventListener('keydown', (e) => set(e, true));
    addEventListener('keyup', (e) => set(e, false));
    addEventListener('blur', () => Object.values(el).forEach((k) => k.classList.remove('down')));
    window.TREX_KEYS = el;
  })();

  // ---------- Final horde ----------
  (function horde() {
    const track = $('.horde-track');
    const kinds = ['grub', 'slime', 'beetle', 'frog', 'mite', 'wisp', 'brute', 'boomer', 'slimelet', 'dilo', 'ghost', 'ptero', 'puffer', 'grub', 'mite', 'slime'];
    for (let r = 0; r < 2; r++) {
      track.appendChild(sprite('rex', 3));
      const gap = document.createElement('span'); gap.style.width = '80px'; track.appendChild(gap);
      kinds.forEach((k) => track.appendChild(sprite(k, 3)));
      const gap2 = document.createElement('span'); gap2.style.width = '240px'; track.appendChild(gap2);
    }
  })();

  // ---------- Kills (margin critters) and footer HUD ----------
  let kills = 0;
  const killsEl = $('.kills');
  const killsN = $('.kills-n');
  const killsFoot = $('.kills-foot');
  function addKill() {
    kills++;
    killsN.textContent = kills;
    killsFoot.textContent = kills;
    killsEl.classList.add('on');
    killsEl.classList.remove('pop'); void killsEl.offsetWidth; killsEl.classList.add('pop');
  }

  (function critters() {
    if (RM || !FINE) return;
    const layer = $('.critters');
    const KINDS = [
      ['grub', [C.sand, C.clay, C.bone]], ['slime', [C.grape, C.pink, C.blush]], ['mite', [C.red, C.ember]],
      ['wisp', [C.cyan, C.ice, C.sky]], ['beetle', [C.red, C.blush]], ['frog', [C.lime, C.sprout, C.leaf]],
    ];
    const list = [];
    let W = innerWidth, H = innerHeight, zones = [];
    function layout() {
      W = innerWidth; H = innerHeight;
      const content = Math.min(1240, W - 80);
      const side = (W - content) / 2;
      zones = side >= 80 ? [[10, side - 50], [W - side + 10, W - 50]] : [];
      list.forEach((c) => { c.el.style.display = zones.length ? '' : 'none'; });
    }
    layout();
    addEventListener('resize', layout);
    let mx = -999, my = -999;
    addEventListener('pointermove', (e) => { mx = e.clientX; my = e.clientY; }, { passive: true });
    function spawn(c) {
      const z = zones[(Math.random() * zones.length) | 0] || [10, 60];
      c.z = z;
      c.x = rand(z[0], z[1]);
      c.y = Math.random() < 0.5 ? -40 : H + 10;
      c.tx = rand(z[0], z[1]); c.ty = rand(120, H - 80);
      c.speed = rand(22, 40);
      c.dead = false; c.el.classList.remove('hit'); c.el.style.opacity = '1';
    }
    function make(i) {
      const [kind, colors] = KINDS[i % KINDS.length];
      const sp = spec(kind);
      const el = document.createElement('span');
      el.className = 'critter';
      const s = sprite(sp, 3, { fd: 200 });
      el.appendChild(s);
      layer.appendChild(el);
      const c = { el, s, sp, colors, hp: 2, dead: false, face: 1 };
      spawn(c);
      c.y = rand(120, H - 80);
      el.addEventListener('pointerdown', (e) => {
        if (c.dead) return;
        e.preventDefault();
        const cx = c.x + sp.w * 1.5, cy = c.y + sp.h * 1.5;
        c.hp--;
        fx.text(cx + rand(-6, 6), cy - 24, String((Math.random() * 9 + 3) | 0), C.bone);
        el.classList.add('hit');
        setTimeout(() => el.classList.remove('hit'), 90);
        if (c.hp > 0) { c.x += (c.x < W / 2 ? -1 : 1) * 6; return; }
        c.dead = true;
        fx.burst(cx, cy, 18, colors, { min: 60, max: 200 });
        el.style.opacity = '0';
        const target = () => { const r = $('.kills .spr').getBoundingClientRect(); return { x: r.left + r.width / 2, y: r.top + r.height / 2 }; };
        killsEl.classList.add('on');
        fx.fly(cx, cy, target, C.cyan, addKill);
        setTimeout(() => { c.hp = 2; spawn(c); }, rand(2500, 5000));
      });
      return c;
    }
    for (let i = 0; i < 5; i++) list.push(make(i));
    layout();
    let last = 0;
    function tick(t) {
      const dt = Math.min(0.05, (t - (last || t)) / 1000); last = t;
      if (zones.length) {
        for (const c of list) {
          if (c.dead) continue;
          const cx = c.x + c.sp.w * 1.5, cy = c.y + c.sp.h * 1.5;
          const near = Math.hypot(mx - cx, my - cy) < 180 && mx > c.z[0] - 60 && mx < c.z[1] + 90;
          let tx = c.tx, ty = c.ty, sp = c.speed;
          if (near) { tx = clamp(mx - c.sp.w * 1.5, c.z[0], c.z[1]); ty = my - c.sp.h * 1.5; sp *= 1.7; }
          const dx = tx - c.x, dy = ty - c.y, d = Math.hypot(dx, dy);
          if (d < 4 && !near) { c.tx = rand(c.z[0], c.z[1]); c.ty = rand(120, H - 80); }
          else if (d > 1) { c.x += dx / d * sp * dt; c.y += dy / d * sp * dt; if (Math.abs(dx) > 2) c.face = dx < 0 ? -1 : 1; }
          c.el.style.transform = `translate3d(${c.x.toFixed(1)}px, ${c.y.toFixed(1)}px, 0)`;
          c.s.style.transform = c.face < 0 ? 'scaleX(-1)' : '';
        }
      }
      requestAnimationFrame(tick);
    }
    requestAnimationFrame(tick);
  })();

  (function survived() {
    const s = $('.survived'), b = $('.best');
    const fmt = (x) => `${Math.floor(x / 60)}:${String(x % 60).padStart(2, '0')}`;
    let best = +(store.get('trex-site-best') || 0);
    let t = 0;
    b.textContent = fmt(best);
    setInterval(() => {
      if (document.hidden) return; // pauses like the game does on focus-out
      t++;
      s.textContent = fmt(t);
      if (t > best) { best = t; b.textContent = fmt(best); if (t % 5 === 0) store.set('trex-site-best', best); }
    }, 1000);
  })();
})();
