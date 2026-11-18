"""A pendulum wave. Fifteen pendulums; in CYCLE seconds the k-th completes 16 + k swings,
so they drift out of step and all come back together exactly at the end.
Lengths are the real ones for those periods: L = g (T / 2 pi)^2.

Usage: python make_scene.py > scene.json   (then vork render scene.json --out wave.mp4)
"""
import json, math, sys

N, CYCLE, FPS, AMP, G = 15, 12, 30, 0.42, 9.81
SPACING, BOB = 0.022, 0.0085
frames = CYCLE * FPS

nodes, tracks = [], []
x0 = -SPACING * (N - 1) / 2
TOP = 0.20
for k in range(N):
    swings = 16 + k
    period = CYCLE / swings
    L = G * (period / (2 * math.pi)) ** 2
    name = f"pivot{k}"
    hue = k / (N - 1)
    nodes.append({
        "type": "node", "name": name, "translation": [round(x0 + k * SPACING, 5), TOP, 0],
        "children": [
            {"type": "mesh", "name": f"string{k}", "translation": [0, round(-L / 2, 5), 0],
             "geometry": {"std": "cylinder", "radius": 0.0006, "height": round(L, 5), "segments": 6},
             "materials": ["string"]},
            {"type": "mesh", "name": f"bob{k}", "translation": [0, round(-L, 5), 0],
             "geometry": {"std": "sphere", "radius": BOB, "segments": 32, "rings": 16},
             "materials": [{"inputs": {"base_color": [round(0.95 - 0.75 * hue, 3), round(0.45 + 0.1 * hue, 3), round(0.2 + 0.7 * hue, 3)],
                                       "specular_roughness": 0.25}}]},
        ]})
    keys = []
    for f in range(frames + 1):
        th = AMP * math.cos(2 * math.pi * swings * f / frames)
        keys.append([f"{f}/{FPS}", [round(math.sin(th / 2), 6), 0, 0, round(math.cos(th / 2), 6)]])
    tracks.append({"path": ["nodes", name, "rotation"], "keys": keys})

nodes += [
    {"type": "mesh", "name": "beam", "translation": [0, TOP + 0.004, 0],
     "geometry": {"std": "box", "size": [SPACING * N + 0.03, 0.008, 0.012]}, "materials": ["wood"]},
    {"type": "mesh", "name": "floor", "geometry": {"std": "grid", "size": [10, 10]}, "materials": ["floor"]},
    {"type": "light", "shape": "rectangle", "translation": [-0.25, 0.75, 0.35], "target": [0, 0.08, 0],
     "energy": 14, "size": 0.25, "color": [1.0, 0.9, 0.8]},
    {"type": "light", "shape": "rectangle", "translation": [0.5, 0.4, 0.5], "target": [0, 0.1, 0],
     "energy": 3, "size": 0.6, "color": [0.6, 0.72, 1.0]},
    {"type": "camera", "translation": [0.0, 0.41, 0.46], "target": [0, 0.095, 0.0]},
]
for post_x in (-SPACING * N / 2 - 0.01, SPACING * N / 2 + 0.01):
    nodes.append({"type": "mesh", "name": f"post{post_x:.3f}", "translation": [round(post_x, 4), TOP / 2, 0],
                  "geometry": {"std": "box", "size": [0.008, TOP, 0.012]}, "materials": ["wood"]})

json.dump({
    "$schema": "https://raw.githubusercontent.com/voyager-so/vork/v0.6.1/packages/ts/scene.schema.json",
    "version": 2,
    "animation": [{"name": "swing", "tracks": tracks}],
    "materials": {
        "string": {"inputs": {"base_color": [0.6, 0.6, 0.6], "specular_roughness": 0.6}},
        "wood": {"inputs": {"base_color": [0.12, 0.075, 0.045], "specular_roughness": 0.6}},
        "floor": {"inputs": {"base_color": [0.05, 0.055, 0.065], "specular_roughness": 0.5}},
    },
    "nodes": nodes,
    "render": {"resolution": [1080, 1080], "duration": CYCLE, "fps": FPS},
}, sys.stdout, separators=(",", ":"))
