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
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
  const mmss = (s) => `${Math.floor(s / 60)}:${String(Math.floor(s % 60)).padStart(2, '0')}`;
  const play = (v) => { const p = v.play(); if (p && p.catch) p.catch(() => {}); };
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

  // ---------- Sprites: every one comes out of a single atlas ----------
  const AT = T.atlas;
  document.documentElement.style.setProperty('--aw', AT.w);
  document.documentElement.style.setProperty('--ah', AT.h);
  function spec(slug) {
    const s = T.spr[slug];
    return s ? { src: 'assets/' + s[0], w: s[1], h: s[2], n: s[3] } : null;
  }
  function sprite(sp, scale, opt = {}) {
    if (typeof sp === 'string') sp = spec(sp);
    const at = AT.at[sp.src.replace(/^assets\//, '')] || [0, 0];
    const el = document.createElement('span');
    el.className = 'spr' + (sp.n > 1 && opt.anim !== false ? ' anim' : '') + (opt.flip ? ' flip' : '');
    el.style.cssText = `--w:${sp.w};--h:${sp.h};--n:${sp.n};--s:${scale};--x:${at[0]};--y:${at[1]}` + (opt.fd ? `;--fd:${opt.fd}ms` : '');
    return el;
  }
  const fitScale = (sp, box) => clamp(Math.floor(box / Math.max(sp.w, sp.h)), 1, 8);

  PX.renderAll();

  // ---------- Whole-pixel sizing for game footage ----------
  // Every clip is 256x144 game pixels. Size each one to a whole multiple
  // of that in device pixels so no game pixel is ever resampled.
  function fit(w, h, nw = 256, nh = 144) {
    const d = window.devicePixelRatio || 1;
    const k = Math.max(1, Math.floor(Math.min(w * d / nw, h * d / nh) + 1e-6));
    return { w: k * nw / d, h: k * nh / d, k };
  }
  function cover(w, h, nw = 256, nh = 144) {
    const d = window.devicePixelRatio || 1;
    const k = Math.ceil(Math.max(w * d / nw, h * d / nh));
    return { w: k * nw / d, h: k * nh / d, k };
  }
  const setSize = (el, f) => { el.style.width = f.w + 'px'; el.style.height = f.h + 'px'; };
  const layouts = [];
  function relayout() { layouts.forEach((f) => f()); }

  // ---------- FX canvas: pixel particles and floating text ----------
  const fx = (() => {
    const cv = $('.fx');
    const ctx = cv.getContext('2d');
    let parts = [], raf = 0, last = 0, dpr = 1;
    function size() { dpr = Math.min(2, window.devicePixelRatio || 1); cv.width = innerWidth * dpr; cv.height = innerHeight * dpr; }
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
    function add(p) { if (RM) return; p.max = p.life; parts.push(p); if (!raf) raf = requestAnimationFrame(tick); }
    return {
      burst(x, y, n, colors, o = {}) {
        for (let i = 0; i < n; i++) {
          const ang = o.up ? rand(-Math.PI * 0.85, -Math.PI * 0.15) : rand(0, Math.PI * 2);
          const sp = rand(o.min || 60, o.max || 180);
          add({ x: x + rand(-(o.spread || 0), o.spread || 0), y, vx: Math.cos(ang) * sp, vy: Math.sin(ang) * sp, g: o.g ?? 360, life: rand(0.45, 0.9), color: pick(colors), size: pick(o.sizes || [4, 4, 8]) });
        }
      },
      text(x, y, text, color, size = 3) { add({ x, y, vx: 0, vy: -70, g: 90, life: 0.9, text, color, size }); },
      fly(x, y, target, color, done) { add({ sx: x, sy: y, x, y, target, life: 0.7, color, size: 8, arc: rand(-40, 40), done, fade: false }); },
    };
  })();

  // ---------- Buttons: bursts, copy, star count ----------
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
      if (!(await copyText(b.dataset.copy))) return;
      b.textContent = 'Copied'; b.classList.add('done');
      const r = b.getBoundingClientRect();
      fx.burst(r.left + r.width / 2, r.top + r.height / 2, 16, [C.cyan, C.ice, C.sky], { min: 80, max: 220 });
      fx.text(r.left + r.width / 2, r.top - 18, 'COPIED!', C.cyan);
      clearTimeout(b._t);
      b._t = setTimeout(() => { b.textContent = label; b.classList.remove('done'); }, 1600);
    });
  });

  // Long commands scroll sideways under a fade instead of wrapping.
  const scrollers = $$('.install code, .code pre');
  function edge(el) {
    const over = el.scrollWidth > el.clientWidth + 1;
    const end = el.scrollLeft + el.clientWidth >= el.scrollWidth - 2;
    el.classList.toggle('ovf', over);
    el.classList.toggle('ovf-end', over && end);
    el.classList.toggle('ovf-mid', over && !end && el.scrollLeft > 2);
  }
  scrollers.forEach((el) => el.addEventListener('scroll', () => edge(el), { passive: true }));
  layouts.push(() => scrollers.forEach(edge));

  (function stars() {
    const box = $('.hero .stars');
    const show = (n) => { if (n > 0) { $('.stars-n', box).textContent = n.toLocaleString('en-US'); box.hidden = false; } };
    let cached = null;
    try { cached = JSON.parse(store.get('trex-stars') || 'null'); } catch (e) { cached = null; }
    if (cached) show(cached.n);
    if (cached && Date.now() - cached.t < 3600e3) return;
    fetch('https://api.github.com/repos/ghandhitechnology/trex', { headers: { Accept: 'application/vnd.github+json' } })
      .then((r) => (r.ok ? r.json() : null))
      .then((j) => { if (!j || typeof j.stargazers_count !== 'number') return; store.set('trex-stars', JSON.stringify({ n: j.stargazers_count, t: Date.now() })); show(j.stargazers_count); })
      .catch(() => {});
  })();

  // ---------- Small sprite fills ----------
  $('.kills-ico').appendChild(sprite('skull', 3));
  $('.hb-ico').appendChild(sprite('bone', 2));
  $$('.lvl-gem').forEach((g) => g.appendChild(sprite('gem-big', 4)));

  // ---------- Hero: props, light, terminal ----------
  const hero = $('.hero');
  (function props() {
    const box = $('.props');
    const kinds = ['p-tuft', 'p-tuft', 'p-rock', 'p-bone', 'p-fern', 'p-bloom', 'p-mossrock', 'p-skull', 'p-tuft-dim', 'p-ribs', 'gem'];
    let seed = 7;
    const rnd = () => ((seed = (seed * 16807) % 2147483647) / 2147483647);
    let placed = 0;
    while (placed < 16) {
      const x = rnd() * 96 + 1, y = rnd() * 30 + 4;
      if (x < 70) continue; // keep the headline clear
      const el = sprite(kinds[(rnd() * kinds.length) | 0], 4, { anim: false });
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

  const clock = $('.tmux-status .clock');
  const tickClock = () => { const d = new Date(); clock.textContent = String(d.getHours()).padStart(2, '0') + ':' + String(d.getMinutes()).padStart(2, '0'); };
  tickClock(); setInterval(tickClock, 15000);

  const term = $('.term');
  const termBody = $('.term-body');
  const pane = $('.pane-trex');
  const screen = $('.screen', pane);
  const reel = $('.reel');
  const typed = $('.typed');
  const border = $('.tborder');
  const focusTag = $('.focus-tag');
  const stampS = $('.stamp-s');
  const termTitle = $('.term-title');

  // tmux draws the split as a column of │; fill it to the pane height.
  border.innerHTML = '<div class="half top"></div><div class="half bot"></div>';
  const fillBorder = () => { const n = Math.ceil(pane.offsetHeight / 19) + 1; $$('.half', border).forEach((h) => { h.textContent = '│\n'.repeat(n); }); };

  const narrow = matchMedia('(max-width: 900px)');
  layouts.push(function heroLayout() {
    if (window.trexHeroFit) window.trexHeroFit();
    if (!narrow.matches) fillBorder();
    scrollLog();
  });

  const HERO = window.TREX_HERO = {
    pane, reel, user: false,
    focus(which) {
      const was = term.dataset.focus;
      term.dataset.focus = which;
      focusTag.textContent = which === 'agent' ? 'agent' : 'trex';
      if (was && was !== which && !RM) { focusTag.classList.remove('flash'); void focusTag.offsetWidth; focusTag.classList.add('flash'); }
    },
    title(text) { termTitle.textContent = text || 'tmux · ~/api'; termTitle.classList.toggle('wait', !!text); },
    stamp(text) { stampS.textContent = text; },
  };
  HERO.focus('trex');
  $('.pane-agent').addEventListener('click', () => HERO.focus('agent'));

  // Agent pane: Claude Code working through tasks. When it finishes,
  // focus moves over, the game pauses, you review, focus comes back.
  const log = $('.agent-log');
  const HISTORY = [
    '<span class="dim">~/api</span> <span class="ok">❯</span> <span class="you">git log --oneline -3</span>',
    '<span class="warn">a41c9e2</span> Retry refunds on a stale balance read.',
    '<span class="warn">7d02b18</span> Split ledger writes into their own module.',
    '<span class="warn">c93e0f4</span> Add the refund_twice regression test.',
    '<span class="dim">~/api</span> <span class="ok">❯</span> <span class="you">git switch -c fix/ledger-race</span>',
    '<span class="dim">Switched to a new branch \'fix/ledger-race\'</span>',
    '<span class="dim">~/api</span> <span class="ok">❯</span> <span class="you">cargo test ledger 2>&1 | tail -2</span>',
    '<span class="del">test ledger::refund_twice ... FAILED</span>',
    'test result: <span class="del">FAILED</span>. 411 passed; 1 failed',
    '<span class="dim">~/api</span> <span class="ok">❯</span> <span class="you">tmux split-window -h</span>',
    '',
  ];
  const TASKS = [
    {
      ask: 'fix the flaky ledger tests',
      reads: [['tests/ledger_test.rs', 212], ['src/ledger/refund.rs', 96]],
      found: '`refund()` reads the balance before the write lands. Locking the row first closes the race.',
      file: 'src/ledger/refund.rs',
      diff: [[41, ' ', '    let tx = db.begin().await?;'], [42, '-', '    let bal = balance(&tx, id).await?;'], [42, '+', '    let row = lock_row(&tx, id).await?;'], [43, '+', '    let bal = row.balance;'], [44, ' ', '    refund_into(&tx, bal, amount).await?;']],
      test: 'cargo test ledger', total: 412, secs: '2.91',
      done: 'Fixed. `refund()` now locks the row before reading the balance, and all 412 ledger tests pass.',
      reply: 'ship it', ship: ['git push -u origin fix/ledger-race', "branch 'fix/ledger-race' set up to track 'origin/fix/ledger-race'."], after: 'Pushed. PR #214 is up.',
    },
    {
      ask: 'move sessions to the new store',
      reads: [['src/auth/session.rs', 148], ['src/store/mod.rs', 203], ['migrations/0042_sessions.sql', 31]],
      found: 'Three call sites in `session.rs` still go through the old cache.',
      file: 'src/auth/session.rs',
      diff: [[57, ' ', 'pub async fn load(id: SessionId) -> Result<Session> {'], [58, '-', '    let c = cache::global();'], [59, '-', '    c.get(id).await'], [58, '+', '    let s = Store::sessions();'], [59, '+', '    s.get(id).await?.ok_or(Error::Expired)'], [60, ' ', '}']],
      test: 'cargo test auth', total: 188, secs: '1.74',
      done: 'Sessions now read and write through `Store`. All 188 auth tests pass.',
      reply: 'lgtm, merge', ship: ['gh pr merge --squash', '✓ Squashed and merged pull request #215'], after: 'Merged into main.',
    },
    {
      ask: 'bump deps and fix what breaks',
      reads: [['Cargo.toml', 64], ['src/http/client.rs', 121]],
      found: 'reqwest 0.13 renamed the timeout builder. Only `client.rs` uses it.',
      file: 'src/http/client.rs',
      diff: [[22, ' ', '    let http = reqwest::Client::builder()'], [23, '-', '        .timeout(Duration::from_secs(30))'], [23, '+', '        .read_timeout(Duration::from_secs(30))'], [24, ' ', '        .build()?;']],
      test: 'cargo test', total: 1204, secs: '8.02',
      done: 'Bumped 14 crates. One rename in `client.rs`, and all 1204 tests pass.',
      reply: 'nice. push it', ship: ['git push', '   4f1c2e9..b83d0a7  main -> main'], after: 'Pushed. CI is green.',
    },
  ];
  // Spinner frames, verbs and colours match Claude Code's dark theme.
  const FR = ['·', '✢', '✳', '✶', '✻', '✽'];
  const FRAMES = FR.concat(FR.slice().reverse());
  const VERBS = ['Brewing', 'Cogitating', 'Percolating', 'Noodling', 'Pondering', 'Simmering', 'Tinkering', 'Wrangling', 'Clauding', 'Marinating', 'Reticulating', 'Spelunking'];
  const DONE = ['Baked', 'Brewed', 'Churned', 'Cogitated', 'Cooked', 'Crunched', 'Worked'];
  const esc = (s) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  const md = (s) => esc(s).replace(/`([^`]+)`/g, '<span class="ic">$1</span>');
  const code = (s) => esc(s)
    .replace(/\b(pub|async|fn|let|await)\b/g, '<span class="kw">$1</span>')
    .replace(/\b([A-Z]\w*|[a-z_]\w*(?=\())/g, '<span class="fn">$1</span>');
  const plural = (n, w) => `<b>${n}</b> ${w}${n === 1 ? '' : 's'}`;

  const agentPane = $('.pane-agent');
  function scrollLog() {
    // Scroll by whole lines so the top row is never cut in half.
    const room = agentPane.clientHeight - 8;
    const h = log.getBoundingClientRect().height;
    const lh = parseFloat(getComputedStyle(log).lineHeight) || 19;
    const over = h - room;
    const y = over > -8 ? -Math.ceil(over / lh) * lh : 8;
    log.style.transform = `translateY(${y}px)`;
    // The welcome banner sticks to the top once the transcript scrolls past it.
    if (head.parentNode === log) { const top = head.offsetTop + y; head.style.transform = top < 0 ? `translateY(${-top}px)` : ''; }
  }

  // The bottom of Claude Code: spinner, prompt between two rules, footer hint.
  const live = document.createElement('span');
  live.className = 'cc-live';
  live.innerHTML =
    '<span class="cc-spin"><span class="l gap"> </span><span class="l spin"></span></span>' +
    '<span class="l gap"> </span><span class="l rule"></span>' +
    '<span class="l cc-in"><span class="pr">❯</span> <span class="txt"></span><span class="cursor"></span><span class="ph g"></span></span>' +
    '<span class="l rule"></span><span class="l g cc-foot"></span>';
  $$('.rule', live).forEach((r) => { r.textContent = '─'.repeat(240); });
  const spinEl = $('.spin', live), inTxt = $('.txt', live), inPh = $('.ph', live), foot = $('.cc-foot', live);

  // Welcome banner: Clawd beside the version, model and folder, like the CLI.
  // Clawd is drawn from the CLI's block-character art (one terminal quadrant per
  // pixel column), with headphones and a keyboard. It types along with whoever
  // is typing, faster while Claude works, and leans back nodding while it waits.
  const head = document.createElement('span');
  head.className = 'cc-head';
  head.innerHTML =
    '<svg class="clawd" viewBox="0 0 22 19" shape-rendering="crispEdges" aria-hidden="true"></svg>' +
    '<span class="l t"><b>Claude Code</b> <span class="g">v2.1.283</span></span>' +
    '<span class="l t g">Opus 5.5 · Claude Max</span>' +
    '<span class="l t g">~/api</span><span class="l gap"> </span>';
  const clawdEl = $('.clawd', head);
  const CLAWD = { o: '#d77757', e: '#0b0813', b: '#45395c', c: '#1d1629', p: '#da4f9e', k: '#2d2340', y: '#675a7c', Y: '#fff4b0', i: '#d2fbf6', t: '#74dcee', n: '#ff99c4' };
  const cz = { mode: '', t: 0, timer: 0, arms: [0, 0], last: 1, parts: [] };
  function clawdDraw(st) {
    const W = 22, H = 19, px = new Array(W * H).fill('');
    const put = (x, y, c) => { if (x >= 0 && x < W && y >= 0 && y < H) px[y * W + x] = c; };
    const rect = (x0, y0, w, h, c) => { for (let y = y0; y < y0 + h; y++) for (let x = x0; x < x0 + w; x++) put(x, y, c); };
    // One pixel per terminal quadrant across, three per quadrant down, so the
    // body keeps the CLI art's proportions and its tall slit eyes.
    const L = st.lean + 1, Hd = st.lean + st.nod + 1;
    [6, 8, 13, 15].forEach((x) => rect(x, 15 + L, 1, 3, 'o'));
    rect(5, 9 + L, 12, 6, 'o');
    rect(5, 3 + Hd, 12, 6, 'o');
    const eh = st.blink ? 1 : 3;
    rect(7, 9 - eh + Hd, 1, eh, 'e'); rect(14, 9 - eh + Hd, 1, eh, 'e');
    // headphones: band arcing over the top, cups on the sides, a light on the beat
    rect(7, Hd, 8, 1, 'b'); rect(5, 1 + Hd, 2, 1, 'b'); rect(15, 1 + Hd, 2, 1, 'b'); put(4, 2 + Hd, 'b'); put(17, 2 + Hd, 'b');
    rect(3, 3 + Hd, 2, 4, 'c'); rect(17, 3 + Hd, 2, 4, 'c');
    rect(3, 3 + Hd, 1, 4, 'b'); rect(18, 3 + Hd, 1, 4, 'b');
    if (st.beat) { rect(4, 4 + Hd, 1, 2, 'p'); rect(17, 4 + Hd, 1, 2, 'p'); }
    // arms: out when resting, down on the keys when typing
    if (st.type) { rect(3, 11 + 2 * st.arms[0], 2, 3, 'o'); rect(17, 11 + 2 * st.arms[1], 2, 3, 'o'); }
    else { rect(3, 9 + L, 2, 3, 'o'); rect(17, 9 + L, 2, 3, 'o'); }
    rect(2, 16, 18, 2, 'k');
    for (let x = 2; x < 20; x += 2) { put(x + 1, 16, 'y'); put(x, 17, 'y'); }
    rect(1, 18, 20, 1, 'b');
    if (st.type && st.arms[0]) { put(3, 16, 'Y'); put(4, 17, 'Y'); }
    if (st.type && st.arms[1]) { put(18, 16, 'Y'); put(17, 17, 'Y'); }
    for (const q of cz.parts) {
      const x = Math.round(q.x), y = Math.round(q.y), c = q.c[Math.min(q.age, q.c.length - 1)];
      put(x, y, c);
      if (q.note) { put(x + 1, y - 1, c); put(x + 1, y - 2, c); }
    }
    let out = '';
    for (let y = 0; y < H; y++) {
      for (let x = 0; x < W;) {
        const c = px[y * W + x];
        let n = 1;
        while (x + n < W && px[y * W + x + n] === c) n++;
        if (c) out += `<rect x="${x}" y="${y}" width="${n}" height="1" fill="${CLAWD[c]}"/>`;
        x += n;
      }
    }
    clawdEl.innerHTML = out;
  }
  function clawdTick() {
    if (!heroVisible || document.hidden) return;
    const t = ++cz.t;
    cz.parts = cz.parts.filter((q) => { q.age++; q.x += q.dx; q.y += q.dy; return q.age < q.c.length; });
    if (cz.mode === 'wait') {
      // Leaning back, nodding on every beat, a note off the cups now and then.
      const beat = t % 4 === 0;
      if (t % 8 === 0) { const r = (t / 8) % 2; cz.parts.push({ x: r ? 19 : 1, y: 4, dx: r ? 0.25 : -0.25, dy: -0.5, age: 0, note: true, c: ['n', 'n', 'n', 'p', 'p'] }); }
      clawdDraw({ lean: -1, nod: beat || t % 4 === 1 ? 1 : 0, beat: beat || t % 4 === 1, blink: t % 26 === 13 });
      return;
    }
    // Typing: a key goes down, comes up, the next one goes down.
    const down = cz.arms[0] || cz.arms[1];
    if (down && !(cz.mode === 'fast' && Math.random() < 0.2)) cz.arms = [0, 0];
    else {
      const side = Math.random() < 0.75 ? 1 - cz.last : cz.last;
      cz.last = side; cz.arms = side ? [0, 1] : [1, 0];
      if (Math.random() < 0.6) cz.parts.push({ x: side ? 19 : 2, y: 15, dx: side ? 1 : -1, dy: -1, age: 0, c: ['i', 't', 'y'] });
    }
    const press = cz.arms[0] || cz.arms[1];
    clawdDraw({ lean: 0, nod: press ? 1 : 0, type: true, arms: cz.arms, beat: press && cz.mode === 'fast', blink: false });
  }
  function clawdMode(m) {
    if (RM) { if (!cz.mode) { cz.mode = m; clawdDraw({ lean: 0, nod: 0, type: true, arms: [0, 0] }); } return; }
    if (cz.mode === m) return;
    cz.mode = m; cz.arms = [0, 0];
    clearInterval(cz.timer);
    cz.timer = setInterval(clawdTick, m === 'fast' ? 85 : m === 'slow' ? 150 : 125);
    clawdTick();
  }

  function line(html, cls) {
    const s = document.createElement('span');
    s.className = 'l' + (cls ? ' ' + cls : ''); s.innerHTML = html || ' ';
    log.insertBefore(s, live.parentNode === log ? live : null);
    if (log.children.length > 160 && !heroVisible) {
      while (log.children.length > 40) {
        const f = log.firstChild === head ? head.nextSibling : log.firstChild;
        if (!f || f === live) break;
        f.remove();
      }
    }
    scrollLog();
    return s;
  }
  const prompt = (cmd) => `<span class="dim">~/api</span> <span class="ok">❯</span> <span class="you">${cmd}</span>`;
  HISTORY.forEach((h) => line(h));

  let spinT = 0, spinK = 0, spin0 = 0, toks = 0, verb = '';
  function drawSpin() {
    const secs = Math.floor((performance.now() - spin0) / 1000);
    const at = (spinK % (verb.length + 8)) - 3;
    const v = Array.from(verb).map((c, i) => (i >= at && i < at + 3 ? `<span class="sh">${c}</span>` : c)).join('');
    const tk = toks < 1000 ? String(toks | 0) : (toks / 1000).toFixed(1) + 'k';
    spinEl.innerHTML = `<span class="cl">${FRAMES[spinK % FRAMES.length]} ${v}…</span> <span class="g">(${secs}s · ↓ ${tk} tokens)</span>`;
  }
  function spinStart() {
    verb = pick(VERBS); spin0 = performance.now(); spinK = 0; toks = rand(40, 120);
    live.classList.add('busy');
    clawdMode('fast');
    foot.textContent = '  esc to interrupt';
    drawSpin(); scrollLog();
    if (RM) return;
    clearInterval(spinT);
    spinT = setInterval(() => { spinK++; toks += rand(4, 28); drawSpin(); }, 120);
  }
  function spinStop() {
    clearInterval(spinT);
    live.classList.remove('busy');
    clawdMode('wait');
    foot.textContent = '  ? for shortcuts';
    scrollLog();
    return Math.max(1, Math.round((performance.now() - spin0) / 1000));
  }
  function setInput(text, placeholder) {
    inTxt.textContent = text;
    inPh.textContent = placeholder || '';
  }
  async function typeIn(text) {
    setInput('');
    clawdMode('slow');
    for (const ch of text) { inTxt.textContent += ch; await sleep(rand(45, 95)); }
    clawdMode('wait');
  }
  function submit() {
    const t = inTxt.textContent;
    setInput('');
    line('');
    line(`<span class="pr">❯</span> ${esc(t)}`, 'u w');
  }
  function tool(name, arg) {
    line('');
    return line(`<span class="dot wip">⏺</span> <b>${name}</b>(${esc(arg)})`, 'w');
  }
  const ok = (ln) => $('.dot', ln).classList.replace('wip', 'done');
  const res = (html) => line(`<span class="g">  ⎿  </span>${html}`);
  const say = (text) => { line(''); return line(`<span class="dot">⏺</span> ${md(text)}`, 'w'); };
  function diff(t) {
    const add = t.diff.filter((d) => d[1] === '+').length, del = t.diff.filter((d) => d[1] === '-').length;
    const sum = [add && 'Added ' + plural(add, 'line'), del && (add ? 'removed ' : 'Removed ') + plural(del, 'line')].filter(Boolean).join(', ');
    res(sum);
    for (const [n, m, src] of t.diff) {
      const k = m === '+' ? 'plus' : m === '-' ? 'minus' : 'ctx';
      line(`     <span class="dr ${k}"><span class="n">${String(n).padStart(3)} ${m}</span>${code(src)}</span>`);
    }
  }
  function header() {
    line('');
    log.insertBefore(head, null);
    log.appendChild(live);
    clawdMode('wait');
    foot.textContent = '  ? for shortcuts';
    setInput('', 'Try "fix lint errors"');
    scrollLog();
  }

  async function review(t) {
    if (HERO.user) return;
    HERO.title('waiting on you');
    await sleep(900);
    if (HERO.user) { HERO.title(); return; }
    HERO.focus('agent');
    HERO.stamp('focus-out · 8 fps');
    pane.classList.add('away');
    reel.pause();
    await sleep(2200);
    await typeIn(t.reply);
    HERO.title();
    if (HERO.user) { setInput(''); return; }
    await sleep(420);
    submit();
    spinStart();
    await sleep(900);
    const b = tool('Bash', t.ship[0]);
    const r = res('<span class="g">Running…</span>');
    await sleep(1300);
    ok(b);
    r.innerHTML = `<span class="g">  ⎿  </span>${esc(t.ship[1])}`;
    await sleep(500);
    say(t.after);
    const s = spinStop();
    line('');
    line(`<span class="g">✻ ${pick(DONE)} for ${s}s</span>`);
    await sleep(2400);
    if (HERO.user) return;
    HERO.focus('trex');
    pane.classList.remove('away');
    if (heroVisible) play(reel);
  }

  async function work(t) {
    submit();
    spinStart();
    await sleep(1400);
    for (const [f, k] of t.reads) {
      const r = tool('Read', f);
      await sleep(rand(380, 620));
      ok(r); res(`Read ${plural(k, 'line')}`);
      await sleep(rand(200, 400));
    }
    await sleep(900);
    say(t.found);
    await sleep(1300);
    const u = tool('Update', t.file);
    await sleep(900);
    ok(u); diff(t);
    await sleep(1200);
    const b = tool('Bash', t.test);
    const r = res('<span class="g">Running…</span>');
    await sleep(rand(1800, 2600));
    ok(b);
    r.innerHTML = `<span class="g">  ⎿  </span>running ${t.total} tests`;
    line(`     test result: <span class="pass">ok</span>. ${t.total} passed; 0 failed; finished in ${t.secs}s`);
    await sleep(800);
    say(t.done);
    const s = spinStop();
    line('');
    line(`<span class="g">✻ ${pick(DONE)} for ${s}s</span>`);
  }

  async function agentLoop() {
    const sh = line(prompt('') + '<span class="cursor"></span>');
    await sleep(500);
    for (const ch of 'claude') { $('.you', sh).textContent += ch; await sleep(rand(60, 110)); }
    await sleep(300);
    $('.cursor', sh).remove();
    header();
    let n = 0;
    for (;;) {
      const t = TASKS[n++ % TASKS.length];
      await sleep(n === 1 ? 900 : 300);
      await typeIn(t.ask);
      await sleep(350);
      await work(t);
      await review(t);
      await sleep(9000);
    }
  }

  let heroVisible = true;
  function bootReel() {
    pane.classList.add('booted');
    if (!RM && heroVisible && !HERO.user) play(reel);
  }
  HERO.boot = bootReel;
  async function terminal() {
    if (RM) {
      const t = TASKS[0];
      typed.textContent = 'trex';
      line(prompt('claude'));
      header();
      setInput(t.ask);
      submit();
      tool('Read', t.reads[1][0]).querySelector('.dot').classList.replace('wip', 'done');
      res(`Read ${plural(t.reads[1][1], 'line')}`);
      say(t.found);
      tool('Bash', t.test);
      res('<span class="g">Running…</span>');
      spinStart();
      bootReel();
      return;
    }
    agentLoop();
    await sleep(120);
    for (const ch of 'trex') { typed.textContent += ch; await sleep(rand(50, 80)); }
    await sleep(120);
    bootReel();
  }
  new IntersectionObserver(([e]) => {
    heroVisible = e.isIntersecting;
    if (!pane.classList.contains('booted') || RM) return;
    if (heroVisible && !pane.classList.contains('away') && !pane.classList.contains('playing')) play(reel);
    else reel.pause();
  }).observe(pane);

  // ---------- Entry ----------
  requestAnimationFrame(() => {
    relayout();
    requestAnimationFrame(() => { document.body.classList.add('ready'); terminal(); });
  });
  let rsz = 0;
  addEventListener('resize', () => { cancelAnimationFrame(rsz); rsz = requestAnimationFrame(relayout); });
  if (document.fonts && document.fonts.ready) document.fonts.ready.then(relayout);

  // ---------- Nav: XP bar and active link ----------
  const secs = $$('main > section');
  const xpFill = $('.xp-fill');
  const navLinks = $$('.nav-links a');
  function onScrollNav() {
    const max = document.documentElement.scrollHeight - innerHeight;
    const p = max > 0 ? clamp(scrollY / max, 0, 1) : 0;
    xpFill.style.transform = `scaleX(${p.toFixed(4)})`;
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
    for (const e of es) if (e.isIntersecting) { e.target.classList.add('in', 'seen'); rv.unobserve(e.target); }
  }, { threshold: 0, rootMargin: '0px 0px -10% 0px' });
  $$('[data-reveal]').forEach((el) => rv.observe(el));

  // ---------- Lazy videos ----------
  const lazyV = new IntersectionObserver((es) => {
    for (const e of es) {
      const v = e.target;
      if (e.isIntersecting) {
        if (!v.dataset.loaded) {
          v.querySelectorAll('source[data-src]').forEach((s) => { s.src = s.dataset.src; });
          v.dataset.loaded = '1';
          v.load();
        }
        if (!RM) play(v);
      } else if (v.dataset.loaded) v.pause();
    }
  }, { rootMargin: '200px 0px' });
  $$('video[data-lazy]').forEach((v) => { if (!(v.closest('.final-bg') && matchMedia('(max-width: 720px)').matches)) lazyV.observe(v); });

  const finalV = $('.final-bg video');
  const swarm = $('.final-swarm');
  const phone = matchMedia('(max-width: 720px)');
  layouts.push(() => {
    const r = $('.final').getBoundingClientRect();
    if (phone.matches) {
      const f = cover(r.width, r.height, 72, 120);
      swarm.style.backgroundSize = `${f.w}px ${f.h}px`;
      if (!finalV.paused) finalV.pause();
    } else setSize(finalV, cover(r.width, r.height));
  });

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

  // ---------- Run player: one run, told by floor and boss ----------
  (function run() {
    const FLOORS = [
      ['Tar Pits', 'tar'], ['Fern Hollow', 'fern'], ['Ember Flats', 'ember'],
      ['Frost Caves', 'frost'], ['Bone Dunes', 'dunes'], ['Spore Marsh', 'spore'],
    ];
    const BOSSES = [
      [180, 'mire-queen', 'Mire Queen'], [360, 'colossus', 'Bone Colossus'], [540, 'wraith', 'Storm Wraith'],
      [720, 'sandmaw', 'Sandmaw'], [900, 'hive-eye', 'Hive Eye'],
    ];
    const CH = [
      { id: 'early', t: 23, name: 'First picks' },
      { id: 'boss-mire-queen', t: 185, name: 'Mire Queen' },
      { id: 'boss-colossus', t: 368, name: 'Bone Colossus' },
      { id: 'lightning-build', t: 600, name: 'Lightning build' },
      { id: 'boss-sandmaw', t: 745, name: 'Sandmaw' },
      { id: 'late-chaos', t: 870, name: 'The swarm' },
    ];
    const floorAt = (t) => FLOORS[Math.min(FLOORS.length - 1, Math.floor(t / 150))];

    const marks = $('.track-marks');
    const video = $('.run-video');
    const poster = $('.run-poster');
    const vid = $('.run .vid');
    const label = $('.run-label');
    const where = $('.run-where');
    const fill = $('.track-fill');
    let cur = 0, visible = false, raf = 0;

    const floors = $('.route-floors');
    FLOORS.forEach(([n, k]) => {
      const f = document.createElement('span');
      f.className = 'seg fl-' + k; f.textContent = n;
      floors.appendChild(f);
    });
    const bossRow = $('.route-bosses');
    const pins = BOSSES.map(([t, id, n]) => {
      const p = document.createElement('span');
      p.className = 'pin'; p.style.left = (t / 900 * 100) + '%'; p.title = n;
      p.appendChild(sprite(id, 2, { fd: 260 }));
      bossRow.appendChild(p);
      return p;
    });

    CH.forEach((c, i) => {
      const m = document.createElement('button');
      m.type = 'button'; m.className = 'track-mark';
      m.style.left = (c.t / 900 * 100) + '%';
      m.setAttribute('aria-label', `${mmss(c.t)} ${c.name}`);
      m.addEventListener('click', () => go(i, true));
      marks.appendChild(m);
    });
    const mks = $$('.track-mark', marks);
    const fls = $$('.seg', floors);
    function mark(t) {
      const f = Math.min(FLOORS.length - 1, Math.floor(t / 150));
      fls.forEach((el, k) => { el.classList.toggle('past', k < f); el.classList.toggle('on', k === f); });
      pins.forEach((p, k) => p.classList.toggle('past', BOSSES[k][0] <= t + 5));
      fill.style.transform = `scaleX(${clamp(t / 900, 0, 1)})`;
    }
    function go(i, user) {
      cur = (i + CH.length) % CH.length;
      const c = CH[cur];
      mks.forEach((m, k) => { m.classList.toggle('on', k === cur); m.setAttribute('aria-current', k === cur ? 'true' : 'false'); });
      label.textContent = `${mmss(c.t)} · ${c.name}`;
      where.textContent = floorAt(c.t)[0];
      if (!RM) { vid.classList.remove('swapping'); void vid.offsetWidth; vid.classList.add('swapping'); }
      poster.src = `assets/video/${c.id}.png`;
      video.poster = `assets/video/${c.id}.png`;
      mark(c.t);
      video.src = `assets/video/${c.id}.mp4`;
      if (visible && (!RM || user)) play(video);
    }
    video.addEventListener('ended', () => go(cur + 1));
    function loop() {
      if (!video.paused) mark(CH[cur].t + video.currentTime);
      raf = visible ? requestAnimationFrame(loop) : 0;
    }
    let started = false;
    new IntersectionObserver(([e]) => {
      visible = e.isIntersecting;
      if (visible) {
        if (!started) { started = true; go(0); } else if (!RM) play(video);
        if (!raf) raf = requestAnimationFrame(loop);
      } else video.pause();
    }, { rootMargin: '100px 0px' }).observe(vid);
    if (RM) vid.addEventListener('click', () => { if (video.paused) play(video); else video.pause(); });

    // The player takes the full column at a whole-pixel scale; the rest of the section lines up to it.
    const sec = $('#run');
    const wrap = $('.wrap', sec);
    layouts.push(() => {
      const pad = parseFloat(getComputedStyle(vid.parentElement).paddingLeft) * 2;
      const s = fit(wrap.clientWidth - pad, Math.max(144, innerHeight - 190));
      setSize(vid, s);
      sec.style.setProperty('--run-w', (s.w + pad) + 'px');
    });
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
    const bars = $$('.stats .bar');
    bars.forEach((b) => { for (let k = 0; k < 10; k++) { const c = document.createElement('i'); c.style.setProperty('--c', k); b.appendChild(c); } });
    const TRICK = { rex: 'Dash: shockwave', ptera: 'Dash: feather volley', trike: 'Hurt: explode', raptor: 'Crit: set them on fire', spino: 'Hit: slow', stego: 'Hurt: ring of 12 spikes', pachy: 'Dash: headbutt blast' };
    const SHOT = { bolt: 'ember bolt', 'feather-shot': 'twin feathers', horn: 'piercing horn', claw: 'short claw', bubble: 'homing bubble', spike: 'spike ring', pebble: 'pebble spread' };
    const GOAL = (g) => {
      const m = g.match(/(\w+)\(([\d.]+)\)/); if (!m) return g;
      const v = +m[2];
      return ({ Survive: `survive ${mmss(v)}`, Level: `reach LV ${v}`, Kills: `${v} kills in a run`, TotalKills: `${v.toLocaleString('en-US')} kills total`, Runs: `play ${v} runs` })[m[1]] || g;
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
      bars.forEach((b) => {
        const n = clamp(Math.round(h[b.dataset.k + 'Bar'] * 10), 1, 10);
        $$('i', b).forEach((c, k) => c.classList.toggle('on', k < n));
      });
      weapon.innerHTML = `<span>Shoots <b>${SHOT[h.shot] || h.shot}</b></span><span>${TRICK[h.id] || ''}</span>`;
      range.set(h);
    }

    // A pocket firing range: the picked dino shoots its real shot at a grub.
    const range = (() => {
      const cv = $('.range');
      const box = cv.closest('.hub-range');
      const ctx = cv.getContext('2d');
      const RW = 128, RH = 72;
      const img = new Image();
      img.src = AT.src;
      const atl = (sp) => { if (typeof sp === 'string') sp = spec(sp); const at = AT.at[sp.src.replace(/^assets\//, '')] || [0, 0]; return { x: at[0], y: at[1], w: sp.w, h: sp.h, n: sp.n }; };
      const GRUB = atl('grub'), TUFT = atl('p-tuft'), ROCK = atl('p-rock');
      let hero = null, run = null, shot = null, pwr = 12;
      let shots = [], nums = [], bits = [], foe = null, fire = 0.3, n = 0, dash = 0, t = 0, last = 0, raf = 0, on = false;
      const floor = document.createElement('canvas');
      floor.width = RW; floor.height = RH;
      function paintFloor() {
        const f = floor.getContext('2d');
        for (let y = 0; y < RH; y += 8) for (let x = 0; x < RW; x += 8) {
          f.fillStyle = ((x + y) / 8) % 2 ? '#1b3a3a' : '#183434';
          f.fillRect(x, y, 8, 8);
          f.fillStyle = '#122a2a'; f.fillRect(x, y, 8, 1);
        }
        [[TUFT, 10, 10], [ROCK, 96, 8], [TUFT, 60, 60], [TUFT, 118, 54]].forEach(([s, x, y]) => f.drawImage(img, s.x, s.y, s.w, s.h, x, y, s.w, s.h));
        floor.done = true;
      }
      const put = (s, x, y, fr = 0, alpha = 1) => {
        ctx.globalAlpha = alpha;
        ctx.drawImage(img, s.x + (fr % s.n) * s.w, s.y, s.w, s.h, Math.round(x - s.w / 2), Math.round(y - s.h / 2), s.w, s.h);
        ctx.globalAlpha = 1;
      };
      const newFoe = () => ({ x: RW + 10, y: 40, hp: 5, flash: 0, kx: 0 });
      function step(dt) {
        t += dt;
        if (!foe) foe = newFoe();
        if (foe.x > 100) foe.x -= 26 * dt;
        foe.x += foe.kx * dt; foe.kx *= Math.pow(0.01, dt);
        foe.flash = Math.max(0, foe.flash - dt);
        dash = Math.max(0, dash - dt);
        fire -= dt;
        if (fire <= 0 && foe.x < RW - 6) {
          fire = 0.55;
          shots.push({ x: 32, y: 40, vx: 150 });
          if (++n % 7 === 0) dash = 0.5;
        }
        for (const b of shots) {
          b.x += b.vx * dt;
          if (foe && Math.abs(b.x - foe.x) < 5) {
            b.dead = true;
            const crit = Math.random() < 0.2;
            const d = Math.round(pwr * rand(0.85, 1.15) * (crit ? 2 : 1));
            nums.push({ x: foe.x + rand(-3, 3), y: foe.y - 12, n: d, crit, life: 0.7 });
            foe.flash = 0.08; foe.kx = 40; foe.hp--;
            if (foe.hp <= 0) {
              for (let k = 0; k < 14; k++) { const a = rand(0, Math.PI * 2), v = rand(20, 70); bits.push({ x: foe.x, y: foe.y, vx: Math.cos(a) * v, vy: Math.sin(a) * v, life: rand(0.3, 0.6), c: pick([C.sand, C.clay, C.bone]) }); }
              foe = null;
            }
          }
        }
        shots = shots.filter((b) => !b.dead && b.x < RW + 8);
        for (const q of nums) { q.y -= 16 * dt; q.life -= dt; }
        nums = nums.filter((q) => q.life > 0);
        for (const q of bits) { q.x += q.vx * dt; q.y += q.vy * dt; q.vx *= 0.9; q.vy *= 0.9; q.life -= dt; }
        bits = bits.filter((q) => q.life > 0);
      }
      function draw() {
        if (!img.complete || !hero) return;
        if (!floor.done) paintFloor();
        ctx.imageSmoothingEnabled = false;
        ctx.drawImage(floor, 0, 0);
        const fr = Math.floor(t * 1000 / 300);
        const hx = 22 + (dash > 0.25 ? (0.5 - dash) * 40 : dash * 40);
        ctx.fillStyle = 'rgba(15,11,24,.45)';
        ctx.fillRect(Math.round(hx - 6), 48, 12, 2);
        if (dash > 0) for (let k = 1; k <= 3; k++) put(run, hx - k * 5, 40, fr, 0.25);
        put(dash > 0 ? run : hero, hx, 40, dash > 0 ? Math.floor(t * 1000 / 120) : fr);
        if (foe) {
          ctx.fillStyle = 'rgba(15,11,24,.45)'; ctx.fillRect(Math.round(foe.x - 5), 46, 10, 2);
          put(GRUB, foe.x, foe.y, Math.floor(t * 1000 / 200));
          if (foe.flash > 0) { ctx.globalCompositeOperation = 'lighter'; put(GRUB, foe.x, foe.y, Math.floor(t * 1000 / 200)); put(GRUB, foe.x, foe.y, Math.floor(t * 1000 / 200)); ctx.globalCompositeOperation = 'source-over'; }
        }
        for (const b of shots) put(shot, b.x, b.y, Math.floor(t * 1000 / 100));
        for (const q of bits) { ctx.fillStyle = q.c; ctx.fillRect(Math.round(q.x), Math.round(q.y), 1, 1); }
        for (const q of nums) PX.drawText(ctx, q.n, Math.round(q.x - PX.measure(String(q.n)) / 2), Math.round(q.y), 1, q.crit ? C.gold : C.bone, C.ink);
      }
      function loop(now) {
        const dt = Math.min(0.05, (now - (last || now)) / 1000); last = now;
        step(dt); draw();
        raf = on ? requestAnimationFrame(loop) : 0;
      }
      img.onload = () => { if (!raf) { step(0.4); draw(); } };
      new IntersectionObserver(([e]) => {
        on = e.isIntersecting && !RM;
        if (on && !raf) { last = 0; raf = requestAnimationFrame(loop); }
      }).observe(cv);
      layouts.push(() => setSize(cv, fit(box.clientWidth - 16, 4096, RW, RH)));
      return {
        set(h) {
          hero = atl(h.idle); run = atl(h.sprite); shot = atl(h.shot); pwr = h.pwr;
          shots = []; nums = []; n = 0; dash = 0;
          if (RM || !raf) { for (let k = 0; k < 40; k++) step(0.03); draw(); }
        },
      };
    })();
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
    const cardsEl = $('.cards');
    const tray = $('.tray');
    const synsEl = $('.syns');
    const tip = $('.tip');
    const held = [];
    const lit = new Set();
    let hand = [];

    $('.syn-total').textContent = T.synergies.length;
    const synRows = T.synergies.map((s) => {
      const li = document.createElement('li');
      li.className = 'syn'; li.dataset.id = s.id;
      const eq = document.createElement('span'); eq.className = 'eq';
      s.needs.forEach((n, k) => { if (k) eq.append('+'); eq.appendChild(sprite(byId[n].sprite, 2)); });
      eq.append('=');
      const res = sprite(s.sprite, 2, { fd: 220 });
      res.classList.add('res');
      eq.appendChild(res);
      li.appendChild(eq);
      li.insertAdjacentHTML('beforeend', `<span class="nm">${s.name}</span><span class="ds">${s.desc}</span>`);
      li.addEventListener('pointerenter', () => hot(s.needs));
      li.addEventListener('pointerleave', () => hot([]));
      synsEl.appendChild(li);
      return li;
    });

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
          tip.innerHTML = `<b>${it.name}</b>${it.desc}<span class="r ${it.rarity.toLowerCase()}">${held.includes(it.id) ? 'In your build' : it.rarity + (it.unlock ? ` · unlock ${it.unlock} bones` : '')}</span>`;
          const r = b.getBoundingClientRect();
          tip.classList.add('on');
          const tr = tip.getBoundingClientRect();
          tip.style.left = clamp(r.left + r.width / 2 - tr.width / 2, 8, innerWidth - tr.width - 8) + 'px';
          tip.style.top = (r.top - tr.height - 14 < 8 ? r.bottom + 14 : r.top - tr.height - 14) + 'px';
          synRows.forEach((row, i) => row.classList.toggle('peek', T.synergies[i].needs.includes(it.id)));
        };
        const hide = () => { tip.classList.remove('on'); synRows.forEach((row) => row.classList.remove('peek')); };
        b.addEventListener('pointerenter', show); b.addEventListener('focus', show);
        b.addEventListener('pointerleave', hide); b.addEventListener('blur', hide);
        b.addEventListener('click', () => { if (!held.includes(it.id)) { take(it.id, b); show(); } });
        iconEls[it.id] = b;
        wrap.appendChild(b);
      });
      g.appendChild(wrap);
      armory.appendChild(g);
    });
    addEventListener('scroll', () => tip.classList.remove('on'), { passive: true });
    function hot(ids) { Object.entries(iconEls).forEach(([id, el]) => el.classList.toggle('hot', ids.includes(id))); }

    function partners() {
      const out = [];
      for (const s of T.synergies) {
        if (lit.has(s.id)) continue;
        if (s.needs.some((n) => held.includes(n))) s.needs.forEach((n) => { if (!held.includes(n)) out.push(n); });
      }
      return out;
    }
    function deal() {
      const pool = items.filter((it) => !held.includes(it.id)).map((it) => it.id);
      const p = partners().filter((id) => pool.includes(id));
      hand = [];
      if (p.length && Math.random() < 0.6) hand.push(pick(p));
      while (hand.length < 3 && hand.length < pool.length) { const id = pick(pool); if (!hand.includes(id)) hand.push(id); }
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
      if (held.length >= 14 || held.includes(id)) return;
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
      iconEls[id].classList.add('own');
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
      Object.values(iconEls).forEach((el) => el.classList.remove('own'));
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
    // First time each enemy shows up, from content/waves.ron (stage pools and events).
    const FIRST = { grub: 0, wisp: 0, mite: 100, frog: 150, beetle: 150, slime: 150, slimelet: 150, ptero: 150, brute: 200, boomer: 300, dilo: 300, mole: 300, shellback: 450, ghost: 450, puffer: 450, golem: 600, eyestalk: 600, broodmother: 600 };
    const STAGES = [['Tar Pits', '#2a2042'], ['Fern Hollow', '#1b3a32'], ['Ember Flats', '#43231f'], ['Frost Caves', '#1c2c4c'], ['Bone Dunes', '#3a3122'], ['Spore Marsh', '#2a3a1e']];

    const bosses = $('.bosses');
    T.bosses.forEach((b, k) => {
      const el = document.createElement('div');
      el.className = 'boss'; el.style.setProperty('--k', k);
      el.innerHTML = `<span class="at">${mmss((k + 1) * 180)}</span><div class="pad"></div><span class="bn">${b.name}</span><span class="hp"><i style="--v:${(b.hp / 600).toFixed(3)}"></i></span><span class="hpn">${b.hp} HP</span>`;
      $('.pad', el).appendChild(sprite(b.sprite, innerWidth < 720 ? 4 : 5, { fd: 200 }));
      bosses.appendChild(el);
    });

    // Hazard band
    const hz = $('.hazard-track');
    for (let i = 0; i < 16; i++) {
      const w = sprite('warn', 3, { fd: 240 });
      hz.appendChild(w);
      const b = document.createElement('b'); b.textContent = i % 2 ? 'BOSS INCOMING' : 'WARNING';
      hz.appendChild(b);
    }

    // Spawn table
    const stagesEl = $('.spawn-stages');
    STAGES.forEach(([name, col], i) => {
      const s = document.createElement('span');
      s.className = 'spawn-stage';
      s.style.cssText = `left:calc(var(--pad) + ${i / 6} * (100% - var(--pad) * 2));width:calc(${1 / 6} * (100% - var(--pad) * 2));background:${col}`;
      if (i === 0) { s.style.left = '4px'; s.style.width = `calc(var(--pad) - 4px + ${1 / 6} * (100% - var(--pad) * 2))`; }
      if (i === STAGES.length - 1) s.style.width = `calc(var(--pad) - 4px + ${1 / 6} * (100% - var(--pad) * 2))`;
      s.textContent = name;
      stagesEl.appendChild(s);
    });
    const lanes = $('.spawn-lanes');
    const byT = {};
    T.enemies.forEach((e) => { (byT[FIRST[e.id] ?? 0] ||= []).push(e); });
    const mobs = [];
    Object.keys(byT).map(Number).sort((a, b) => a - b).forEach((t) => {
      const col = document.createElement('div');
      col.className = 'spawn-col' + (t === 0 ? ' first' : '');
      col.style.left = (t / 900 * 100) + '%';
      byT[t].forEach((e) => {
        const b = document.createElement('button');
        b.type = 'button'; b.className = 'mob';
        b.setAttribute('aria-label', `${e.id}, first seen at ${mmss(t)}`);
        b.appendChild(sprite(e.sprite, 2, { fd: 200 }));
        b.addEventListener('pointerenter', () => show(e, t, false));
        b.addEventListener('focus', () => show(e, t, false));
        b.addEventListener('click', () => show(e, t, false));
        col.appendChild(b);
        mobs.push({ b, e, t });
      });
      lanes.appendChild(col);
    });
    const marks = T.bosses.map((b, k) => {
      const m = document.createElement('span');
      const t = (k + 1) * 180;
      m.className = 'spawn-bossmark' + (t >= 900 ? ' end' : '');
      m.style.left = (t / 900 * 100) + '%';
      m.appendChild(sprite(b.sprite, 2, { fd: 220 }));
      m.append(b.name);
      lanes.appendChild(m);
      return { m, t };
    });

    const ro = { fig: $('.ro-fig'), name: $('.ro-name'), desc: $('.ro-desc'), stats: $('.ro-stats'), tag: $('.ro-new') };
    let shown = null;
    function show(e, t, isNew) {
      mobs.forEach((m) => m.b.classList.toggle('sel', m.e === e));
      ro.tag.textContent = isNew ? 'New enemy' : `Seen from ${mmss(t)}`;
      ro.tag.classList.toggle('old', !isNew);
      if (shown === e) return;
      shown = e;
      ro.fig.innerHTML = '';
      ro.fig.appendChild(sprite(e.sprite, fitScale(e.sprite, innerWidth > 900 ? 64 : 84), { fd: 200 }));
      ro.name.textContent = e.id; delete ro.name.dataset.pxDone; PX.render(ro.name);
      ro.desc.textContent = DESC[e.id] || '';
      ro.stats.innerHTML = `HP <b>${e.hp}</b> · SPD <b>${e.speed}</b> · <b>${e.behavior}</b>`;
    }
    show(mobs[0].e, 0, true);

    const band = $('.spawn-band');
    const scroller = $('.spawn-scroll');
    const clockEl = $('.spawn-clock');
    const setT = (t) => { band.style.setProperty('--t', (t / 900).toFixed(4)); clockEl.textContent = mmss(t); };
    function light(t, pop) {
      let latest = null;
      for (const m of mobs) if (m.t <= t && !m.b.classList.contains('on')) {
        m.b.classList.add('on');
        if (pop) { m.b.classList.remove('pop'); void m.b.offsetWidth; m.b.classList.add('pop'); }
        latest = m;
      }
      if (latest) show(latest.e, latest.t, true);
      marks.forEach((b) => b.m.classList.toggle('on', b.t <= t));
    }
    if (RM) { setT(900); light(900, false); return; }
    setT(0); light(0, false);
    const io = new IntersectionObserver(([e]) => {
      if (!e.isIntersecting) return;
      io.disconnect();
      const t0 = performance.now();
      const dur = 7000;
      let touched = false;
      scroller.addEventListener('pointerdown', () => { touched = true; }, { once: true });
      const step = (now) => {
        const t = clamp((now - t0) / dur, 0, 1) * 900;
        setT(t); light(t, true);
        if (FINE && !touched && scroller.scrollWidth > scroller.clientWidth) scroller.scrollLeft = (t / 900) * band.offsetWidth - scroller.clientWidth * 0.4;
        if (t < 900) requestAnimationFrame(step);
      };
      requestAnimationFrame(step);
    }, { threshold: 0.5 });
    io.observe(scroller);

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
    $('.play-up').addEventListener('click', () => {
      pane.scrollIntoView({ block: 'center', behavior: RM ? 'auto' : 'smooth' });
      setTimeout(() => { if (window.TREX_ARENA) window.TREX_ARENA.takeover(); }, RM ? 0 : 500);
    });
  })();

  // ---------- Setup: step 3 result ----------
  (function mini() {
    const phone = matchMedia('(max-width: 720px)');
    const body = $('.mini-body');
    const shot = $('.mini-shot');
    layouts.push(() => setSize(shot, fit(body.clientWidth - (phone.matches ? 0 : Math.min(210, body.clientWidth * 0.42)), 4096)));
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
        killsEl.classList.add('on');
        const target = () => { const r = $('.kills .spr').getBoundingClientRect(); return { x: r.left + r.width / 2, y: r.top + r.height / 2 }; };
        fx.fly(cx, cy, target, C.cyan, addKill);
        setTimeout(() => { c.hp = 2; spawn(c); }, rand(2500, 5000));
      });
      return c;
    }
    [0, 1, 3].forEach((k) => list.push(make(k)));
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
          for (const o of list) {
            if (o === c || o.dead) continue;
            const sx = c.x - o.x, sy = c.y - o.y, sd = Math.hypot(sx, sy);
            if (sd < 96) { c.y += (sy >= 0 ? 1 : -1) * (96 - sd) * dt * 3; if (sd < 48) c.ty = clamp(c.ty + (sy >= 0 ? 60 : -60), 120, H - 80); }
          }
          c.el.style.transform = `translate3d(${Math.round(c.x)}px, ${Math.round(c.y)}px, 0)`;
          c.s.style.transform = c.face < 0 ? 'scaleX(-1)' : '';
        }
      }
      requestAnimationFrame(tick);
    }
    requestAnimationFrame(tick);
  })();

  (function survived() {
    const s = $('.survived'), b = $('.best');
    let best = +(store.get('trex-site-best') || 0);
    let t = 0;
    b.textContent = mmss(best);
    setInterval(() => {
      if (document.hidden) return; // pauses like the game does on focus-out
      t++;
      s.textContent = mmss(t);
      if (t > best) { best = t; b.textContent = mmss(best); if (t % 5 === 0) store.set('trex-site-best', best); }
    }, 1000);
  })();
})();
