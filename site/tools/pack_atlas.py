"""Pack every sprite the site uses into one atlas and compose the synergy badges.

Run from anywhere with Pillow installed:  python site/tools/pack_atlas.py
Reads js/data.js, writes assets/sprites/atlas.png, assets/sprites/syn/*.png
and rewrites js/data.js with an `atlas` map of path -> [x, y].
"""
import json
import math
import os

from PIL import Image

SITE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SPR = os.path.join(SITE, 'assets', 'sprites')
DATA = os.path.join(SITE, 'js', 'data.js')

INK = (15, 11, 24, 255)
DUSK = (45, 35, 64, 255)
SLATE = (69, 57, 92, 255)
EMBER = (239, 107, 58, 255)
AMBER = (247, 160, 65, 255)
GOLD = (255, 210, 94, 255)
CREAM = (255, 244, 176, 255)
CYAN = (116, 220, 238, 255)
ICE = (210, 251, 246, 255)
SKY = (62, 156, 224, 255)
BONE = (245, 239, 224, 255)
GRAPE = (122, 54, 160, 255)
PINK = (218, 79, 158, 255)


def load(path):
    return Image.open(os.path.join(SPR, path)).convert('RGBA')


def frames(path, w, h, n):
    im = load(path)
    return [im.crop((i * w, 0, i * w + w, h)) for i in range(n)]


def badge():
    """16x16 socket every synergy result sits in."""
    im = Image.new('RGBA', (16, 16), (0, 0, 0, 0))
    px = im.load()
    for y in range(16):
        for x in range(16):
            d = math.hypot(x - 7.5, y - 7.5)
            if d <= 7.6:
                px[x, y] = SLATE if d > 6.6 else DUSK
    return im


def put(base, spr, cx, cy):
    base.alpha_composite(spr, (int(round(cx - spr.width / 2)), int(round(cy - spr.height / 2))))


def dots(im, pts, col):
    px = im.load()
    for x, y in pts:
        if 0 <= x < 16 and 0 <= y < 16:
            px[x, y] = col


def compose(sid, spr):
    """Return a list of 16x16 frames for synergy `sid`."""
    src = spr and frames(spr['src'].replace('assets/sprites/', ''), spr['w'], spr['h'], spr['n'])
    out = []
    n = len(src) if src else 2
    for f in range(n):
        im = badge()
        s = src[f % len(src)] if src else None
        if sid == 'fire_wheel':
            put(im, s, 8, 8)
            ring = [(8, 1), (14, 8), (8, 14), (1, 8)] if f % 2 == 0 else [(3, 3), (12, 3), (12, 12), (3, 12)]
            dots(im, ring, AMBER)
        elif sid == 'ball_lightning':
            put(im, s, 8, 8)
            arc = [(2, 4), (3, 5), (2, 6), (13, 10), (12, 11), (13, 12)] if f % 2 == 0 else [(4, 2), (5, 3), (6, 2), (10, 13), (11, 12), (12, 13)]
            dots(im, arc, CYAN)
        elif sid == 'cluster_mines':
            put(im, s, 8, 9)
            dots(im, [(3, 3), (12, 2), (13, 13), (2, 12)] if f % 2 == 0 else [(2, 5), (13, 4), (11, 14), (4, 14)], GOLD)
        elif sid == 'driller':
            put(im, s, 9, 7)
            dots(im, [(3, 11), (4, 10), (2, 13), (5, 12)] if f % 2 == 0 else [(3, 12), (5, 10), (2, 11)], SLATE)
        elif sid == 'cryo_beam':
            for x in range(1, 15):
                dots(im, [(x, 7), (x, 8)], CYAN)
                if (x + f) % 3 == 0:
                    dots(im, [(x, 6), (x, 9)], SKY)
            for x in range(2, 14, 2):
                dots(im, [(x + f % 2, 7)], ICE)
            dots(im, [(4, 3), (11, 4), (6, 12), (12, 12)] if f % 2 == 0 else [(3, 4), (10, 3), (7, 13), (13, 11)], ICE)
        elif sid == 'swarm':
            for i, (x, y) in enumerate([(5, 5), (11, 6), (7, 11)]):
                put(im, s, x + (f if i == 1 else 0), y)
            dots(im, [(1, 8), (2, 3), (3, 13)], SLATE)
        elif sid == 'extinction':
            put(im, s, 8, 9)
            dots(im, [(13, 1), (14, 2), (12, 2), (14, 0), (11, 1)] if f % 2 == 0 else [(13, 2), (14, 1), (12, 1), (15, 0)], AMBER)
        elif sid == 'tar_pit':
            c = s.getpixel((1, 1))
            hi = s.getpixel((0, 0)) if s.getpixel((0, 0))[3] else c
            pool = [(x, y) for y in range(6, 13) for x in range(2, 14) if ((x - 7.5) / 6) ** 2 + ((y - 9) / 3.4) ** 2 <= 1]
            dots(im, pool, c)
            dots(im, [(5, 8), (6, 8), (9, 10)], hi)
            dots(im, [(10, 5), (10, 4)] if f % 2 == 0 else [(6, 4), (6, 3)], hi)
        elif sid == 'porcupine':
            for x, y in [(4, 4), (12, 4), (4, 12), (12, 12)]:
                put(im, s, x, y)
            dots(im, [(8, 8), (7, 8), (8, 7), (7, 7)], BONE)
        elif sid == 'pinball':
            put(im, s, 5 if f % 2 == 0 else 6, 10)
            put(im, s, 11, 5 if f % 2 == 0 else 6)
            dots(im, [(8, 8), (9, 8)], PINK)
        elif sid == 'napalm':
            put(im, s, 8, 8)
            dots(im, [(4, 12), (6, 13), (10, 13), (12, 12), (8, 14)] if f % 2 == 0 else [(5, 13), (7, 12), (9, 14), (11, 13)], EMBER)
            dots(im, [(5, 4), (11, 4)] if f % 2 == 0 else [(4, 5), (12, 5)], GOLD)
        elif sid == 'thunder_dash':
            for i, x in enumerate([4, 8, 12]):
                put(im, s, x, 8 + (1 if (i + f) % 2 else -1))
            dots(im, [(1, 8), (2, 8)], SKY)
        else:
            put(im, s, 8, 8)
        out.append(im)
    return out


def main():
    raw = open(DATA).read()
    head = raw[:raw.index('{')]
    d = json.loads(raw[raw.index('{'):raw.rindex('}') + 1])

    fallback = {'napalm': 'bolt', 'thunder_dash': 'spark'}
    os.makedirs(os.path.join(SPR, 'syn'), exist_ok=True)
    for s in d['synergies']:
        sp = s.get('sprite')
        if not sp:
            f = d['spr'][fallback[s['id']]]
            sp = {'src': 'assets/' + f[0], 'w': f[1], 'h': f[2], 'n': f[3]}
        fr = compose(s['id'], sp)
        strip = Image.new('RGBA', (16 * len(fr), 16))
        for i, im in enumerate(fr):
            strip.alpha_composite(im, (i * 16, 0))
        rel = 'syn/%s-strip.png' % s['id'].replace('_', '-')
        strip.save(os.path.join(SPR, rel))
        s['sprite'] = {'src': 'assets/sprites/' + rel, 'w': 16, 'h': 16, 'n': len(fr)}

    # Every path the page can draw.
    paths = set(v[0] for v in d['spr'].values())

    def walk(o):
        if isinstance(o, dict):
            if 'src' in o and isinstance(o['src'], str) and o['src'].startswith('assets/sprites/'):
                paths.add(o['src'][len('assets/'):])
            for v in o.values():
                walk(v)
        elif isinstance(o, list):
            for v in o:
                walk(v)
    walk({k: v for k, v in d.items() if k != 'spr'})
    paths.discard('sprites/atlas.png')

    ims = {p: Image.open(os.path.join(SITE, 'assets', p)).convert('RGBA') for p in sorted(paths)}
    order = sorted(ims, key=lambda p: (-ims[p].height, -ims[p].width, p))
    W = 256
    x = y = row = 0
    pos = {}
    for p in order:
        w, h = ims[p].size
        if x + w > W:
            x, y, row = 0, y + row + 1, 0
        pos[p] = [x, y]
        x += w + 1
        row = max(row, h)
    H = y + row
    atlas = Image.new('RGBA', (W, H))
    for p, (ax, ay) in pos.items():
        atlas.alpha_composite(ims[p], (ax, ay))
    atlas.save(os.path.join(SPR, 'atlas.png'), optimize=True)
    d['atlas'] = {'src': 'assets/sprites/atlas.png', 'w': W, 'h': H, 'at': pos}
    open(DATA, 'w').write(head + json.dumps(d, separators=(',', ':')) + ';\n')
    print('atlas', W, H, len(pos), 'sprites')


if __name__ == '__main__':
    main()
