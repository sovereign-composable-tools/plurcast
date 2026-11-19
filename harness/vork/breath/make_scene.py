"""A breathing pacer: in for 4, hold for 4, out for 6. A 14-second loop. 2D canvas.

Usage: python make_scene.py > scene.json   (then vork render scene.json --out breath.mp4)
"""
import json, math, sys

IN, HOLD, OUT = 4, 4, 6
TOTAL, FPS = IN + HOLD + OUT, 30
W = 900
C = W / 2
FONT = "C:/Windows/Fonts/georgiai.ttf"
RMAX, RMIN = 300, 110


def circle(r):
    return f"M{-r} 0a{r} {r} 0 1 0 {2*r} 0a{r} {r} 0 1 0 {-2*r} 0Z"


def ease(u):
    return 0.5 - 0.5 * math.cos(math.pi * max(0.0, min(1.0, u)))


nodes = [
    {"type": "path", "name": "bg", "d": f"M0 0H{W}V{W}H0Z", "fill": "#10151c"},
    {"type": "path", "name": "guide_out", "translation": [C, C - 30, 0], "d": circle(RMAX), "fill": "none",
     "stroke": "#2c3a4a", "stroke-width": 2},
    {"type": "path", "name": "guide_in", "translation": [C, C - 30, 0], "d": circle(RMIN), "fill": "none",
     "stroke": "#2c3a4a", "stroke-width": 2},
    {"type": "path", "name": "halo", "translation": [C, C - 30, 0], "d": circle(RMAX), "fill": "#3f6f86", "opacity": 0.35},
    {"type": "path", "name": "lung", "translation": [C, C - 30, 0], "d": circle(RMAX), "fill": "#8fc3c7"},
    {"type": "text", "name": "word", "translation": [C - 60, W - 110, 0], "content": "in", "font": FONT,
     "font-size": 54, "fill": "#c9d6d8"},
]
scale, halo = [], []
for f in range(TOTAL * FPS + 1):
    t = f / FPS
    if t < IN:
        s = RMIN + (RMAX - RMIN) * ease(t / IN)
    elif t < IN + HOLD:
        s = RMAX
    else:
        s = RMAX - (RMAX - RMIN) * ease((t - IN - HOLD) / OUT)
    k = round(s / RMAX, 5)
    scale.append([f"{f}/{FPS}", [k, k, 1]])
    h = round(min(1.0, k * 1.06 + 0.02), 5)
    halo.append([f"{f}/{FPS}", [h, h, 1]])
tracks = [
    {"path": ["nodes", "lung", "scale"], "keys": scale},
    {"path": ["nodes", "halo", "scale"], "keys": halo},
    {"path": ["nodes", "word", "content"], "interpolation": "STEP",
     "keys": [[0, "in"], [IN, "hold"], [IN + HOLD, "out"]]},
    {"path": ["nodes", "word", "translation"], "interpolation": "STEP",
     "keys": [[0, [C - 22, W - 110, 0]], [IN, [C - 52, W - 110, 0]], [IN + HOLD, [C - 38, W - 110, 0]]]},
]
json.dump({
    "$schema": "https://raw.githubusercontent.com/voyager-so/vork/v0.6.1/packages/ts/scene.schema.json",
    "version": 2, "composition": {"space": "2d"},
    "animation": [{"name": "breath", "tracks": tracks}],
    "nodes": nodes,
    "render": {"resolution": [W, W], "duration": TOTAL, "fps": FPS},
}, sys.stdout, separators=(",", ":"))
