"""A polished, etched slice of iron meteorite on a dark cloth. The pattern is drawn by etch.slang.

Usage: python make_scene.py > scene.json
"""
import json, math, sys

# An irregular slab outline, extruded: the slice. Built in XY and laid flat.
pts = []
for i in range(40):
    a = 2 * math.pi * i / 40
    r = 0.11 * (1 + 0.16 * math.sin(3 * a + 0.4) + 0.09 * math.sin(5 * a + 1.9) + 0.05 * math.sin(9 * a))
    pts.append([round(r * math.cos(a) * 1.25, 5), round(r * math.sin(a), 5)])
json.dump({
    "$schema": "https://raw.githubusercontent.com/voyager-so/vork/v0.6.1/packages/ts/scene.schema.json",
    "version": 2,
    "composition": {"space": "3d", "view": {"program": "std/agx", "inputs": {"exposure": -1.4}}},
    "programs": {"etch": {"file": "etch.slang"}},
    "materials": {
        "etched": {"program": "etch", "inputs": {"scale": 110.0}},
        "cloth": {"inputs": {"base_color": [0.02, 0.022, 0.03], "specular_roughness": 0.9}},
    },
    "nodes": [
        {"type": "mesh", "name": "cloth", "geometry": {"std": "grid", "size": [10, 10]}, "materials": ["cloth"]},
        {"type": "mesh", "name": "slice", "translation": [0, 0.004, 0], "rotation": {"euler": [-1.5708, 0, 0]},
         "geometry": {"std": "extrude", "outline": pts, "depth": 0.008, "bevel_depth": 0.0012}, "materials": ["etched"]},
        {"type": "light", "shape": "rectangle", "translation": [-0.45, 0.5, -0.5], "target": [0, 0, 0],
         "energy": 30, "size": 0.7, "color": [1.0, 0.95, 0.9]},
        {"type": "light", "shape": "rectangle", "translation": [0.6, 0.35, 0.4], "target": [0, 0, 0],
         "energy": 8, "size": 0.5, "color": [0.7, 0.8, 1.0]},
        {"type": "camera", "translation": [0.03, 0.42, 0.20], "target": [0.0, 0.0, 0.005]},
    ],
    "render": {"resolution": [1500, 1100], "duration": 1},
}, sys.stdout, separators=(",", ":"))
