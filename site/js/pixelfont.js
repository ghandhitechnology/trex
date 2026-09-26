/* trex bitmap font, lifted from src/render/font.rs via assets/font/font.json.
   Renders text as crisp inline SVG: one svg per glyph so words wrap and
   headings can animate letter by letter. Sizes come from the CSS var --u. */
(function () {
  const GLYPHS = {"A":".#.|#.#|###|#.#|#.#","B":"##.|#.#|##.|#.#|##.","C":".##|#..|#..|#..|.##","D":"##.|#.#|#.#|#.#|##.","E":"###|#..|##.|#..|###","F":"###|#..|##.|#..|#..","G":".##.|#...|#.##|#..#|.##.","H":"#.#|#.#|###|#.#|#.#","I":"###|.#.|.#.|.#.|###","J":"..#|..#|..#|#.#|.#.","K":"#..#|#.#.|##..|#.#.|#..#","L":"#..|#..|#..|#..|###","M":"#...#|##.##|#.#.#|#...#|#...#","N":"#..#|##.#|#.##|#..#|#..#","O":".##.|#..#|#..#|#..#|.##.","P":"##.|#.#|##.|#..|#..","Q":".##.|#..#|#..#|#.#.|.#.#","R":"##.|#.#|##.|#.#|#.#","S":".##|#..|.#.|..#|##.","T":"###|.#.|.#.|.#.|.#.","U":"#.#|#.#|#.#|#.#|###","V":"#.#|#.#|#.#|.#.|.#.","W":"#...#|#...#|#.#.#|##.##|#...#","X":"#.#|#.#|.#.|#.#|#.#","Y":"#.#|#.#|.#.|.#.|.#.","Z":"###|..#|.#.|#..|###","0":"###|#.#|#.#|#.#|###","1":".#.|##.|.#.|.#.|###","2":"##.|..#|.#.|#..|###","3":"##.|..#|.#.|..#|##.","4":"#.#|#.#|###|..#|..#","5":"###|#..|##.|..#|##.","6":".##|#..|###|#.#|###","7":"###|..#|.#.|.#.|.#.","8":"###|#.#|###|#.#|###","9":"###|#.#|###|..#|##.",".":".|.|.|.|#",",":"..|..|..|.#|#.",":":".|#|.|#|.",";":"..|.#|..|.#|#.","!":"#|#|#|.|#","?":"##.|..#|.#.|...|.#.","-":"...|...|###|...|...","+":"...|.#.|###|.#.|...","=":"...|###|...|###|...","*":"...|#.#|.#.|#.#|...","/":"..#|..#|.#.|#..|#..","%":"#.#|..#|.#.|#..|#.#","(":".#|#.|#.|#.|.#",")":"#.|.#|.#|.#|#.","[":"##|#.|#.|#.|##","]":"##|.#|.#|.#|##","<":"..#|.#.|#..|.#.|..#",">":"#..|.#.|..#|.#.|#..","'":"#|#|.|.|.","\"":"#.#|#.#|...|...|...","_":"...|...|...|...|###","#":".#.#.|#####|.#.#.|#####|.#.#."," ":"..|..|..|..|.."};
  const H = 5;
  const NS = 'http://www.w3.org/2000/svg';
  const cache = new Map();

  function rows(ch) {
    const g = GLYPHS[ch] || GLYPHS['?'];
    return g.split('|');
  }

  // Merge horizontal runs of set cells into one path.
  function runs(set, w, h, dx, dy) {
    let d = '';
    for (let y = 0; y < h; y++) {
      let x = 0;
      while (x < w) {
        if (!set(x, y)) { x++; continue; }
        let e = x;
        while (e < w && set(e, y)) e++;
        d += `M${x + dx} ${y + dy}h${e - x}v1h${x - e}z`;
        x = e;
      }
    }
    return d;
  }

  // Build the svg markup for one glyph.
  // ol: 'box' draws a 1px outline all round, 'none' draws no outline.
  // depth: rows of drop shadow under the shape, like the TREX logo extrude.
  function glyphSVG(ch, depth, bands, ol) {
    const key = ch + depth + bands + ol;
    if (cache.has(key)) return cache.get(key);
    const r = rows(ch);
    const w = r[0].length;
    const on = (x, y) => y >= 0 && y < H && x >= 0 && x < w && r[y][x] === '#';
    const pad = ol === 'box' ? 1 : 0;
    const ow = w + pad * 2, oh = H + pad * 2;
    const shape = pad
      ? (x, y) => { for (let j = -1; j <= 1; j++) for (let i = -1; i <= 1; i++) if (on(x - 1 + i, y - 1 + j)) return true; return false; }
      : (x, y) => on(x, y);
    let s = '';
    for (let k = depth; k >= 1; k--) s += `<path class="px-sh px-sh${k}" d="${runs(shape, ow, oh, -pad, -pad + k)}"/>`;
    if (pad) s += `<path class="px-ol" d="${runs(shape, ow, oh, -pad, -pad)}"/>`;
    if (bands) {
      for (let y = 0; y < H; y++) {
        const d = runs((x, yy) => yy === y && on(x, yy), w, H, 0, 0);
        if (d) s += `<path class="px-b${y}" d="${d}"/>`;
      }
    } else {
      s += `<path class="px-fg" d="${runs(on, w, H, 0, 0)}"/>`;
    }
    const vbH = H + pad * 2 + depth;
    const svg = `<svg class="px-ch" viewBox="${-pad} ${-pad} ${ow} ${vbH}" style="--gw:${ow};--gh:${vbH};--gp:${pad};--i:@I@" aria-hidden="true" focusable="false" shape-rendering="crispEdges">${s}</svg>`;
    const res = { svg, w };
    cache.set(key, res);
    return res;
  }

  // Turn text into word spans of glyph svgs.
  function build(text, opts) {
    const depth = opts.depth ?? 1;
    const frag = document.createElement('span');
    frag.className = 'px-vis';
    frag.setAttribute('aria-hidden', 'true');
    let n = 0;
    for (const part of opts.parts) {
      const words = part.text.toUpperCase().split(/\s+/).filter(Boolean);
      for (const word of words) {
        const ws = document.createElement('span');
        ws.className = 'px-word' + (part.accent ? ' px-accent' : '');
        let html = '';
        for (const ch of word) {
          const g = glyphSVG(ch, part.accent ? (opts.accentDepth ?? depth) : depth, part.accent || opts.bands, opts.ol);
          html += g.svg.replace('@I@', n++);
        }
        ws.innerHTML = html;
        frag.appendChild(ws);
      }
    }
    frag.style.setProperty('--chars', n);
    return frag;
  }

  function render(el) {
    if (el.dataset.pxDone) return;
    const parts = [];
    el.childNodes.forEach((node) => {
      if (node.nodeType === 3) parts.push({ text: node.textContent, accent: false });
      else if (node.nodeType === 1) parts.push({ text: node.textContent, accent: node.tagName === 'EM' || node.classList.contains('accent') });
    });
    const label = el.textContent.replace(/\s+/g, ' ').trim();
    const opts = {
      parts,
      depth: el.dataset.pxDepth ? +el.dataset.pxDepth : 1,
      accentDepth: el.dataset.pxAccentDepth ? +el.dataset.pxAccentDepth : undefined,
      bands: 'pxBands' in el.dataset,
      ol: el.dataset.pxOl || 'none',
    };
    const vis = build(label, opts);
    const sr = document.createElement('span');
    sr.className = 'sr-only';
    sr.textContent = label;
    el.textContent = '';
    el.append(sr, vis);
    el.dataset.pxDone = '1';
  }

  function renderAll(root = document) {
    root.querySelectorAll('[data-px]').forEach(render);
  }

  // Small helper for canvas drawing (damage numbers, arena HUD).
  function drawText(ctx, text, x, y, size, color, outline) {
    text = String(text).toUpperCase();
    let cx = x;
    const widths = [];
    for (const ch of text) widths.push(rows(ch)[0].length);
    const pass = (c, ol) => {
      cx = x;
      for (let i = 0; i < text.length; i++) {
        const r = rows(text[i]);
        const w = widths[i];
        ctx.fillStyle = c;
        for (let yy = 0; yy < H; yy++) for (let xx = 0; xx < w; xx++) {
          if (r[yy][xx] !== '#') continue;
          if (ol) ctx.fillRect(cx + (xx - 1) * size, y + (yy - 1) * size, size * 3, size * 3);
          else ctx.fillRect(cx + xx * size, y + yy * size, size, size);
        }
        cx += (w + 1) * size;
      }
    };
    if (outline) pass(outline, true);
    pass(color, false);
    return cx - x;
  }

  function measure(text) {
    let w = 0;
    for (const ch of String(text).toUpperCase()) w += rows(ch)[0].length + 1;
    return Math.max(0, w - 1);
  }

  window.PX = { render, renderAll, drawText, measure };
})();
