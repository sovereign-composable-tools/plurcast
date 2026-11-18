"""Pascal's triangle as terrain: height falls with the number of binary carries.

Cell (x, y) stands for the binomial coefficient C(x+y, x). Kummer's theorem:
the power of 2 dividing C(x+y, x) equals the number of carries when x and y
are added in binary. Zero carries (x & y == 0) means the coefficient is odd,
and those cells form the Sierpinski triangle. Here they are the top plateau;
each carry drops a cell one step.

Usage: python make_scene.py [N] > scene.json     (N a power of two, default 64)
"""
import json
import sys

N = int(sys.argv[1]) if len(sys.argv) > 1 else 64
BITS = N.bit_length() - 1
CELL = 1.0 / N          # the whole triangle spans 1 metre on each leg
STEP = 0.014            # height lost per carry
GAP = 0.86              # tile width as a fraction of the cell


def carries(x, y):
    return bin(x).count("1") + bin(y).count("1") - bin(x + y).count("1")


# One material per carry count: warm and pale on top, cooling and darkening below.
TOP = (0.90, 0.66, 0.36)       # zero carries: the odd coefficients
SHALLOW = (0.16, 0.26, 0.40)   # one carry
DEEP = (0.02, 0.035, 0.07)     # the most carries
materials = {}
for c in range(BITS + 1):
    if c == 0:
        col = list(TOP)
    else:
        t = (c - 1) / max(1, BITS - 1)
        col = [round(SHALLOW[i] * (1 - t) + DEEP[i] * t, 4) for i in range(3)]
    materials[f"c{c}"] = {"inputs": {"base_color": col, "specular_roughness": 0.55 if c else 0.38}}

nodes = []
for x in range(N):
    for y in range(N - x):                      # x + y < N: the triangle itself
        c = carries(x, y)
        h = STEP * (BITS + 1 - c)
        nodes.append({
            "type": "mesh",
            "name": f"t{x}_{y}",
            "translation": [round((x + 0.5) * CELL - 0.33, 5), round(h / 2, 5), round((y + 0.5) * CELL - 0.33, 5)],
            "geometry": {"std": "box", "size": [round(CELL * GAP, 5), round(h, 5), round(CELL * GAP, 5)]},
            "materials": [f"c{c}"],
        })

nodes += [
    {"type": "mesh", "name": "floor", "geometry": {"std": "grid", "size": [40, 40]},
     "materials": [{"inputs": {"base_color": [0.012, 0.014, 0.02], "specular_roughness": 0.6}}]},
    # low raking key from the apex side, cool fill from above
    {"type": "light", "shape": "rectangle", "translation": [-1.7, 0.5, 0.9], "target": [0.0, 0.05, 0.0],
     "energy": 60, "size": 0.9, "color": [1.0, 0.82, 0.62]},
    {"type": "light", "shape": "rectangle", "translation": [0.8, 1.8, -1.0], "target": [0.0, 0.0, 0.0],
     "energy": 22, "size": 1.6, "color": [0.55, 0.7, 1.0]},
    {"type": "camera", "translation": [-0.80, 1.62, -0.84], "target": [-0.05, 0.0, -0.04]},
]

scene = {
    "$schema": "https://raw.githubusercontent.com/voyager-so/vork/v0.6.1/packages/ts/scene.schema.json",
    "version": 2,
    "materials": materials,
    "nodes": nodes,
    "render": {"resolution": [1200, 1200], "duration": 1},
}
json.dump(scene, sys.stdout, separators=(",", ":"))
