/* trex bitmap fonts. HUD is lifted from src/render/font.rs; DISPLAY is a 5x7
   cut for headings. Text renders as crisp inline SVG, one svg per glyph, so
   words wrap and headings can animate letter by letter. Size is the CSS var --u.
   data-px uses DISPLAY, data-px="hud" uses the 3x5 HUD font. */
(function () {
  // HUD: the game's own 3x5 font. Canvas text and tiny chips only.
  const HUD = {"A":".#.|#.#|###|#.#|#.#","B":"##.|#.#|##.|#.#|##.","C":".##|#..|#..|#..|.##","D":"##.|#.#|#.#|#.#|##.","E":"###|#..|##.|#..|###","F":"###|#..|##.|#..|#..","G":".##.|#...|#.##|#..#|.##.","H":"#.#|#.#|###|#.#|#.#","I":"###|.#.|.#.|.#.|###","J":"..#|..#|..#|#.#|.#.","K":"#..#|#.#.|##..|#.#.|#..#","L":"#..|#..|#..|#..|###","M":"#...#|##.##|#.#.#|#...#|#...#","N":"#..#|##.#|#.##|#..#|#..#","O":".##.|#..#|#..#|#..#|.##.","P":"##.|#.#|##.|#..|#..","Q":".##.|#..#|#..#|#.#.|.#.#","R":"##.|#.#|##.|#.#|#.#","S":".##|#..|.#.|..#|##.","T":"###|.#.|.#.|.#.|.#.","U":"#.#|#.#|#.#|#.#|###","V":"#.#|#.#|#.#|.#.|.#.","W":"#...#|#...#|#.#.#|##.##|#...#","X":"#.#|#.#|.#.|#.#|#.#","Y":"#.#|#.#|.#.|.#.|.#.","Z":"###|..#|.#.|#..|###","0":"###|#.#|#.#|#.#|###","1":".#.|##.|.#.|.#.|###","2":"##.|..#|.#.|#..|###","3":"##.|..#|.#.|..#|##.","4":"#.#|#.#|###|..#|..#","5":"###|#..|##.|..#|##.","6":".##|#..|###|#.#|###","7":"###|..#|.#.|.#.|.#.","8":"###|#.#|###|#.#|###","9":"###|#.#|###|..#|##.",".":".|.|.|.|#",",":"..|..|..|.#|#.",":":".|#|.|#|.",";":"..|.#|..|.#|#.","!":"#|#|#|.|#","?":"##.|..#|.#.|...|.#.","-":"...|...|###|...|...","+":"...|.#.|###|.#.|...","=":"...|###|...|###|...","*":"...|#.#|.#.|#.#|...","/":"..#|..#|.#.|#..|#..","%":"#.#|..#|.#.|#..|#.#","(":".#|#.|#.|#.|.#",")":"#.|.#|.#|.#|#.","[":"##|#.|#.|#.|##","]":"##|.#|.#|.#|##","<":"..#|.#.|#..|.#.|..#",">":"#..|.#.|..#|.#.|#..","'":"#|#|.|.|.","\"":"#.#|#.#|...|...|...","_":"...|...|...|...|###","#":".#.#.|#####|.#.#.|#####|.#.#."," ":"..|..|..|..|.."};
  // DISPLAY: a 5x7 cut for headings, so V, Y, N, M and W stay distinct at marquee size.
  const DISPLAY = {"A": ".###.|#...#|#...#|#####|#...#|#...#|#...#", "B": "####.|#...#|#...#|####.|#...#|#...#|####.", "C": ".###.|#...#|#....|#....|#....|#...#|.###.", "D": "####.|#...#|#...#|#...#|#...#|#...#|####.", "E": "#####|#....|#....|####.|#....|#....|#####", "F": "#####|#....|#....|####.|#....|#....|#....", "G": ".###.|#...#|#....|#.###|#...#|#...#|.####", "H": "#...#|#...#|#...#|#####|#...#|#...#|#...#", "I": "###|.#.|.#.|.#.|.#.|.#.|###", "J": "....#|....#|....#|....#|#...#|#...#|.###.", "K": "#...#|#..#.|#.#..|##...|#.#..|#..#.|#...#", "L": "#....|#....|#....|#....|#....|#....|#####", "M": "#...#|##.##|#.#.#|#.#.#|#...#|#...#|#...#", "N": "#...#|##..#|##..#|#.#.#|#..##|#..##|#...#", "O": ".###.|#...#|#...#|#...#|#...#|#...#|.###.", "P": "####.|#...#|#...#|####.|#....|#....|#....", "Q": ".###.|#...#|#...#|#...#|#.#.#|#..#.|.##.#", "R": "####.|#...#|#...#|####.|#.#..|#..#.|#...#", "S": ".###.|#...#|#....|.###.|....#|#...#|.###.", "T": "#####|..#..|..#..|..#..|..#..|..#..|..#..", "U": "#...#|#...#|#...#|#...#|#...#|#...#|.###.", "V": "#...#|#...#|#...#|#...#|#...#|.#.#.|..#..", "W": "#...#|#...#|#...#|#.#.#|#.#.#|##.##|#...#", "X": "#...#|#...#|.#.#.|..#..|.#.#.|#...#|#...#", "Y": "#...#|#...#|.#.#.|..#..|..#..|..#..|..#..", "Z": "#####|....#|...#.|..#..|.#...|#....|#####", "0": ".###.|#...#|#..##|#.#.#|##..#|#...#|.###.", "1": "..#..|.##..|..#..|..#..|..#..|..#..|.###.", "2": ".###.|#...#|....#|...#.|..#..|.#...|#####", "3": "####.|....#|....#|.###.|....#|....#|####.", "4": "...#.|..##.|.#.#.|#..#.|#####|...#.|...#.", "5": "#####|#....|####.|....#|....#|#...#|.###.", "6": ".###.|#....|#....|####.|#...#|#...#|.###.", "7": "#####|....#|...#.|..#..|..#..|..#..|..#..", "8": ".###.|#...#|#...#|.###.|#...#|#...#|.###.", "9": ".###.|#...#|#...#|.####|....#|....#|.###.", ".": ".|.|.|.|.|.|#", ",": "..|..|..|..|.#|.#|#.", "!": "#|#|#|#|#|.|#", "?": ".###.|#...#|....#|...#.|..#..|.....|..#..", "'": "#|#|.|.|.|.|.", ":": ".|.|#|.|.|#|.", "-": "....|....|....|####|....|....|....", "+": ".....|..#..|..#..|#####|..#..|..#..|.....", "/": "....#|...#.|...#.|..#..|.#...|.#...|#....", "·": ".|.|.|#|.|.|.", "×": ".....|.....|#...#|.#.#.|..#..|.#.#.|#...#", " ": "...|...|...|...|...|...|..."};
  const FONTS = { hud: HUD, display: DISPLAY };
  const NS = 'http://www.w3.org/2000/svg';
  const cache = new Map();

  function rows(ch, font) {
    const f = FONTS[font] || HUD;
    return (f[ch] || f['?']).split('|');
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

  // Band classes for gradient fills: top rows light, bottom rows hot.
  const BAND = { 5: [0, 1, 2, 3, 4], 7: [0, 1, 1, 2, 3, 3, 4] };

  // Build the svg markup for one glyph.
  // depth: rows of drop shadow under the shape, like the TREX logo extrude.
  function glyphSVG(ch, depth, bands, font) {
    const key = ch + depth + bands + font;
    if (cache.has(key)) return cache.get(key);
    const r = rows(ch, font);
    const H = r.length;
    const w = r[0].length;
    const on = (x, y) => y >= 0 && y < H && x >= 0 && x < w && r[y][x] === '#';
    let s = '';
    for (let k = depth; k >= 1; k--) s += `<path class="px-sh px-sh${k}" d="${runs(on, w, H, 0, k)}"/>`;
    if (bands) {
      const map = BAND[H] || BAND[5];
      for (let b = 0; b < 5; b++) {
        const d = runs((x, y) => map[y] === b && on(x, y), w, H, 0, 0);
        if (d) s += `<path class="px-b${b}" d="${d}"/>`;
      }
    } else {
      s += `<path class="px-fg" d="${runs(on, w, H, 0, 0)}"/>`;
    }
    const vbH = H + depth;
    const svg = `<svg class="px-ch" viewBox="0 0 ${w} ${vbH}" style="--gw:${w};--gh:${vbH};--i:@I@" aria-hidden="true" focusable="false" shape-rendering="crispEdges">${s}</svg>`;
    const res = { svg, w };
    cache.set(key, res);
    return res;
  }

  // Turn text into word spans of glyph svgs.
  function build(opts) {
    const depth = opts.depth ?? 0;
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
          const g = glyphSVG(ch, part.accent ? (opts.accentDepth ?? depth) : depth, part.accent || opts.bands, opts.font);
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
    const font = el.dataset.px === 'hud' ? 'hud' : 'display';
    const vis = build({
      parts,
      font,
      depth: el.dataset.pxDepth ? +el.dataset.pxDepth : 0,
      accentDepth: el.dataset.pxAccentDepth ? +el.dataset.pxAccentDepth : undefined,
      bands: 'pxBands' in el.dataset,
    });
    el.classList.add('px-' + font);
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

  // Canvas text in the HUD font (damage numbers, arena HUD).
  function drawText(ctx, text, x, y, size, color, outline) {
    text = String(text).toUpperCase();
    let cx = x;
    const pass = (c, ol) => {
      cx = x;
      for (let i = 0; i < text.length; i++) {
        const r = rows(text[i], 'hud');
        const w = r[0].length;
        ctx.fillStyle = c;
        for (let yy = 0; yy < 5; yy++) for (let xx = 0; xx < w; xx++) {
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
    for (const ch of String(text).toUpperCase()) w += rows(ch, 'hud')[0].length + 1;
    return Math.max(0, w - 1);
  }

  window.PX = { render, renderAll, drawText, measure };
})();
