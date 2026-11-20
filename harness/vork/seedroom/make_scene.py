"""The seed room on the ship, for the story "the count": a wall of drawers, one pulled open under a headlamp.

Usage: python make_scene.py > scene.json
"""
import json, math, random, sys

rng = random.Random(212)
COLS, ROWS = 16, 8
DW, DH, GAP = 0.30, 0.18, 0.012            # drawer front, metres
OPEN = (9, 3)                               # column, row of the open drawer
PULL = 0.34
M = {
    "front": {"inputs": {"base_color": [0.34, 0.37, 0.40], "base_metalness": 0.7, "specular_roughness": 0.45}},
    "frame": {"inputs": {"base_color": [0.10, 0.11, 0.12], "specular_roughness": 0.6}},
    "handle": {"inputs": {"base_color": [0.75, 0.75, 0.72], "base_metalness": 1.0, "specular_roughness": 0.25}},
    "label": {"inputs": {"base_color": [0.82, 0.78, 0.66], "specular_roughness": 0.8}},
    "inside": {"inputs": {"base_color": [0.16, 0.17, 0.18], "specular_roughness": 0.7}},
    "barley": {"inputs": {"base_color": [0.72, 0.56, 0.28], "specular_roughness": 0.55}},
    "floor": {"inputs": {"base_color": [0.07, 0.075, 0.08], "specular_roughness": 0.35}},
    "paper": {"inputs": {"base_color": [0.80, 0.76, 0.66], "specular_roughness": 0.85}},
    "cover": {"inputs": {"base_color": [0.20, 0.08, 0.06], "specular_roughness": 0.6}},
}
N = []


def box(name, pos, size, mat, rot=None):
    n = {"type": "mesh", "name": name, "translation": [round(v, 4) for v in pos],
         "geometry": {"std": "box", "size": [round(v, 4) for v in size]}, "materials": [mat]}
    if rot:
        n["rotation"] = {"euler": rot}
    N.append(n)


W = COLS * (DW + GAP)
H = ROWS * (DH + GAP)
box("carcass", (0, H / 2 + 0.1, -0.30), (W + 0.1, H + 0.1, 0.56), "frame")
N.append({"type": "mesh", "name": "floor", "geometry": {"std": "grid", "size": [30, 30]}, "materials": ["floor"]})
for c in range(COLS):
    for r in range(ROWS):
        x = -W / 2 + (c + 0.5) * (DW + GAP)
        y = 0.1 + (r + 0.5) * (DH + GAP)
        z = PULL if (c, r) == OPEN else 0.0
        box(f"f{c}_{r}", (x, y, z), (DW, DH, 0.02), "front")
        box(f"h{c}_{r}", (x, y - 0.045, z + 0.02), (0.09, 0.012, 0.02), "handle")
        box(f"l{c}_{r}", (x, y + 0.035, z + 0.011), (0.11, 0.035, 0.002), "label")
        if (c, r) == OPEN:
            # the drawer body: bottom, two sides, back; open top
            box("d_bottom", (x, y - DH / 2 + 0.01, z - 0.22), (DW - 0.02, 0.008, 0.44), "inside")
            box("d_left", (x - DW / 2 + 0.01, y - 0.02, z - 0.22), (0.008, DH - 0.05, 0.44), "inside")
            box("d_right", (x + DW / 2 - 0.01, y - 0.02, z - 0.22), (0.008, DH - 0.05, 0.44), "inside")
            box("d_back", (x, y - 0.02, z - 0.44), (DW - 0.02, DH - 0.05, 0.008), "inside")
            box("bed", (x, y - DH / 2 + 0.04, z - 0.225), (DW - 0.03, 0.055, 0.42), "barley")
            for i in range(3200):              # loose grains lying on the bed
                gx = x + rng.uniform(-DW / 2 + 0.02, DW / 2 - 0.02)
                gz = z - rng.uniform(0.02, 0.43)
                gy = y - DH / 2 + 0.068 + rng.uniform(0, 0.004)
                N.append({"type": "mesh", "name": f"g{i}", "translation": [round(gx, 4), round(gy, 4), round(gz, 4)],
                          "scale": [0.0045, 0.0028, 0.0022], "rotation": {"euler": [0, round(rng.uniform(0, 3.14), 2), round(rng.uniform(-0.25, 0.25), 2)]},
                          "geometry": {"std": "sphere", "radius": 1, "segments": 8, "rings": 5}, "materials": ["barley"]})
            ox, oy = x, y

# the ledger, open on a stool beside the drawer
box("stool", (ox + 0.55, 0.26, 0.55), (0.34, 0.52, 0.34), "frame")
box("ledger_l", (ox + 0.47, 0.532, 0.55), (0.15, 0.02, 0.24), "paper", [0, 0.2, 0.05])
box("ledger_r", (ox + 0.63, 0.532, 0.55), (0.15, 0.02, 0.24), "paper", [0, 0.2, -0.05])
box("ledger_c", (ox + 0.55, 0.521, 0.55), (0.34, 0.012, 0.26), "cover", [0, 0.2, 0])

N += [
    {"type": "light", "shape": "spot", "name": "headlamp", "translation": [ox - 0.10, oy + 0.85, 1.00],
     "target": [ox + 0.15, oy - 0.10, PULL - 0.05], "energy": 60, "spot_size": 1.0, "spot_blend": 0.5, "radius": 0.02,
     "color": [1.0, 0.93, 0.82]},
    {"type": "light", "shape": "rectangle", "translation": [-3.5, 2.6, 2.5], "target": [0, 0.8, 0], "energy": 9, "size": 2.0,
     "color": [0.45, 0.6, 1.0]},
    {"type": "light", "shape": "environment", "color": [0.01, 0.014, 0.02], "energy": 1.0},
    {"type": "camera", "translation": [ox - 0.62, oy + 0.72, 1.30], "target": [ox + 0.16, oy - 0.14, 0.22]},
]
json.dump({
    "$schema": "https://raw.githubusercontent.com/voyager-so/vork/v0.6.1/packages/ts/scene.schema.json",
    "version": 2, "materials": M, "nodes": N,
    "render": {"resolution": [1500, 1000], "duration": 1},
}, sys.stdout, separators=(",", ":"))
