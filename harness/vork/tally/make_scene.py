"""A page from Kef's notebook: lamps counted at each station (station world). 2D canvas.

Usage: python make_scene.py > scene.json
"""
import json, random, sys

W, H = 1100, 1400
HAND = "C:/Windows/Fonts/Inkfree.ttf"
INK, PENCIL, RED = "#262a3a", "#6d6f78", "#a3352a"
rng = random.Random(5)
nodes = []


def path(name, d, **kw):
    n = {"type": "path", "name": name, "d": d, "fill": "none"}
    n.update(kw)
    nodes.append(n)


def text(name, x, y, content, size=44, fill=INK, rot=0.0):
    n = {"type": "text", "name": name, "translation": [x, y, 0], "content": content,
         "font": HAND, "font-size": size, "fill": fill}
    if rot:
        import math
        n["rotation"] = [0, 0, round(math.sin(rot / 2), 5), round(math.cos(rot / 2), 5)]
    nodes.append(n)


def lamp(name, x, y, lit):
    r = 17 + rng.uniform(-1.5, 1.5)
    x += rng.uniform(-3, 3); y += rng.uniform(-3, 3)
    d = f"M{x-r:.1f} {y:.1f}a{r:.1f} {r*0.95:.1f} 0 1 0 {2*r:.1f} 1.5a{r:.1f} {r*0.95:.1f} 0 1 0 {-2*r:.1f} -1.5Z"
    path(name, d, stroke=INK, fill=(INK if lit else "none"), **{"stroke-width": 3})


path("table", f"M0 0H{W}V{H}H0Z", fill="#40372c")
path("page", f"M60 40H{W-50}V{H-40}H60Z", fill="#efe7d2", filter="drop-shadow(0px 8px 14px rgba(0,0,0,0.5))")
for i in range(17):
    y = 190 + i * 70
    path(f"line{i}", f"M60 {y}H{W-50}", stroke="#b9c4cf", **{"stroke-width": 1.3})
path("margin", f"M170 40V{H-40}", stroke="#d9a3a0", **{"stroke-width": 1.5})

text("title", 200, 110, "lamps", 62)
rows = [("st. 9", [0, 0, 0], "was 3"), ("st. 4", [1, 0, 0], "was 3"), ("st. 11", [1, 1, 1], ""), ("st. 6", [1, 1, 0], "one flickers")]
y = 262
for i, (name, lamps, note) in enumerate(rows):
    text(f"s{i}", 200, y - 52, name, 48)
    for j, lit in enumerate(lamps):
        lamp(f"l{i}_{j}", 430 + j * 62, y - 16, lit)
    if note:
        text(f"n{i}", 650, y - 46, note, 38, PENCIL)
    y += 140
text("st2", 200, y - 52, "st. 2", 48)
text("st2n", 430, y - 50, "didn't get that far", 40, PENCIL)
y += 150
text("k1", 200, y - 52, "keeper says count again", 44)
text("k2", 200, y + 18, "at the next dark.", 44)
y += 210
text("r1", 200, y - 52, "if 4 goes to none", 46, RED)
text("r2", 200, y + 18, "i walk it.", 46, RED)
path("under", f"M196 {y+78}q120 10 250 -4", stroke=RED, **{"stroke-width": 3})

json.dump({
    "$schema": "https://raw.githubusercontent.com/voyager-so/vork/v0.6.1/packages/ts/scene.schema.json",
    "version": 2, "composition": {"space": "2d"}, "nodes": nodes,
    "render": {"resolution": [W, H], "duration": 1},
}, sys.stdout, separators=(",", ":"))
