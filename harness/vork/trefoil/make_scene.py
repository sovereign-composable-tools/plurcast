"""A trefoil knot turning on a table. The knot has three-fold symmetry about its
axis, so a third of a turn brings it back onto itself and the clip loops.

Usage: python make_scene.py > scene.json   (then vork render scene.json --out knot.mp4)
"""
import json, math, sys

SECONDS, FPS, SEG = 6, 30, 360
path = []
for i in range(SEG):
    t = 2 * math.pi * i / SEG
    r = 2 + math.cos(3 * t)
    path.append([round(0.05 * r * math.cos(2 * t), 5), round(0.05 * math.sin(3 * t), 5), round(0.05 * r * math.sin(2 * t), 5)])
path.append(path[0])
profile = [[round(0.017 * math.cos(2 * math.pi * k / 24), 5), round(0.017 * math.sin(2 * math.pi * k / 24), 5)] for k in range(24)]
profile.append(profile[0])

frames = SECONDS * FPS
keys = []
for f in range(frames + 1):
    a = (2 * math.pi / 3) * f / frames / 2
    keys.append([f"{f}/{FPS}", [0, round(math.sin(a), 6), 0, round(math.cos(a), 6)]])

json.dump({
    "$schema": "https://raw.githubusercontent.com/voyager-so/vork/v0.6.1/packages/ts/scene.schema.json",
    "version": 2,
    "animation": [{"name": "turn", "tracks": [{"path": ["nodes", "knot", "rotation"], "keys": keys}]}],
    "materials": {
        "copper": {"inputs": {"base_color": [0.93, 0.52, 0.36], "base_metalness": 1.0, "specular_roughness": 0.22}},
        "table": {"inputs": {"base_color": [0.035, 0.04, 0.05], "specular_roughness": 0.35}},
    },
    "nodes": [
        {"type": "mesh", "name": "table", "geometry": {"std": "grid", "size": [10, 10]}, "materials": ["table"]},
        {"type": "mesh", "name": "knot", "translation": [0, 0.085, 0],
         "geometry": {"std": "sweep", "profile": profile, "path": path}, "materials": ["copper"]},
        {"type": "light", "shape": "rectangle", "translation": [-0.35, 0.5, 0.25], "target": [0, 0.08, 0],
         "energy": 12, "size": 0.35, "color": [1.0, 0.9, 0.8]},
        {"type": "light", "shape": "rectangle", "translation": [0.45, 0.25, -0.3], "target": [0, 0.08, 0],
         "energy": 6, "size": 0.5, "color": [0.55, 0.7, 1.0]},
        {"type": "camera", "translation": [0.0, 0.74, 0.24], "target": [0, 0.07, 0.0]},
    ],
    "render": {"resolution": [900, 900], "duration": SECONDS, "fps": FPS},
}, sys.stdout, separators=(",", ":"))
