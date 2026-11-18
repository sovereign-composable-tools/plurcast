"""Rule 30 on a ring, built as a tower: one course of bricks per generation.

W cells sit around a cylinder. Generation 0 (the bottom course) has a single
live cell. Each course above is the next generation of rule 30 with the ring's
ends joined: new = left XOR (centre OR right). Live cells are bricks that stand
proud of the wall; dead cells are recessed.

Usage: python make_scene.py [W] [GENERATIONS] > scene.json   (defaults 64 96)
"""
import json
import math
import sys

W = int(sys.argv[1]) if len(sys.argv) > 1 else 64
G = int(sys.argv[2]) if len(sys.argv) > 2 else 96

R = 0.25                 # wall radius in metres
H = 0.012                # course height
TAN = 2 * math.pi * R / W * 0.9
PROUD, RECESSED = 0.05, 0.018

row = [0] * W
row[0] = 1
rows = []
for _ in range(G):
    rows.append(row)
    row = [row[(i - 1) % W] ^ (row[i] | row[(i + 1) % W]) for i in range(W)]

materials = {
    "live": {"inputs": {"base_color": [0.90, 0.66, 0.36], "specular_roughness": 0.4}},
    "dead": {"inputs": {"base_color": [0.07, 0.11, 0.18], "specular_roughness": 0.6}},
}
nodes = []
for t, cells in enumerate(rows):
    for i, on in enumerate(cells):
        theta = 2 * math.pi * i / W
        depth = PROUD if on else RECESSED
        r = R + depth / 2
        nodes.append({
            "type": "mesh",
            "name": f"b{t}_{i}",
            "translation": [round(r * math.cos(theta), 5), round((t + 0.5) * H, 5), round(r * math.sin(theta), 5)],
            "rotation": {"euler": [0, round(math.pi / 2 - theta, 5), 0]},
            "geometry": {"std": "box", "size": [round(TAN, 5), round(H * 0.9, 5), depth]},
            "materials": ["live" if on else "dead"],
        })

top = G * H
nodes += [
    {"type": "mesh", "name": "floor", "geometry": {"std": "grid", "size": [40, 40]},
     "materials": [{"inputs": {"base_color": [0.012, 0.014, 0.02], "specular_roughness": 0.6}}]},
    {"type": "light", "shape": "rectangle", "translation": [-1.9, top * 0.55, 1.1], "target": [0.0, top * 0.5, 0.0],
     "energy": 110, "size": 1.0, "color": [1.0, 0.82, 0.62]},
    {"type": "light", "shape": "rectangle", "translation": [1.4, top + 1.0, 1.6], "target": [0.0, top * 0.5, 0.0],
     "energy": 40, "size": 1.8, "color": [0.55, 0.7, 1.0]},
    # the single seed cell (i = 0) is at +X; the camera looks at it from slightly above
    {"type": "camera", "translation": [2.35, top * 0.72, 1.05], "target": [0.0, top * 0.48, 0.0]},
]

scene = {
    "$schema": "https://raw.githubusercontent.com/voyager-so/vork/v0.6.1/packages/ts/scene.schema.json",
    "version": 2,
    "materials": materials,
    "nodes": nodes,
    "render": {"resolution": [1000, 1400], "duration": 1},
}
json.dump(scene, sys.stdout, separators=(",", ":"))
