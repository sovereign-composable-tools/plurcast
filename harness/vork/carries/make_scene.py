"""Pascal's triangle as terrain: height falls with the number of carries in base p.

Cell (x, y) stands for the binomial coefficient C(x+y, x). Kummer's theorem:
the power of a prime p dividing C(x+y, x) equals the number of carries when x
and y are added in base p. Zero carries means p does not divide the
coefficient; those cells are the top plateau, and each carry drops a cell one
step. For p = 2 the plateau is the Sierpinski triangle.

Top tiles are tinted by the residue C(x+y, x) mod p (always 1 for p = 2).

Usage: python make_scene.py [ROWS] [P] > scene.json
       ROWS must be a power of P. Defaults: 64 2.
"""
import json
import sys
from math import comb

N = int(sys.argv[1]) if len(sys.argv) > 1 else 64
P = int(sys.argv[2]) if len(sys.argv) > 2 else 2
K = 0
while P ** K < N:
    K += 1
assert P ** K == N, "ROWS must be a power of P"

CELL = 1.0 / N          # the whole triangle spans 1 metre on each leg
STEP = 0.084 / K        # height lost per carry; total relief stays constant
GAP = 0.86              # tile width as a fraction of the cell


def carries(x, y):
    count = carry = 0
    while x or y or carry:
        carry = 1 if x % P + y % P + carry >= P else 0
        count += carry
        x //= P
        y //= P
    return count


# Top tiles: one warm tone per nonzero residue. Lower tiles: slate blue, darker with depth.
TOPS = [(0.90, 0.66, 0.36), (0.80, 0.80, 0.74), (0.86, 0.42, 0.30), (0.62, 0.72, 0.52)]
SHALLOW = (0.16, 0.26, 0.40)
DEEP = (0.02, 0.035, 0.07)
materials = {}
for r in range(1, P):
    materials[f"top{r}"] = {"inputs": {"base_color": list(TOPS[(r - 1) % len(TOPS)]), "specular_roughness": 0.38}}
for c in range(1, K + 1):
    t = (c - 1) / max(1, K - 1)
    col = [round(SHALLOW[i] * (1 - t) + DEEP[i] * t, 4) for i in range(3)]
    materials[f"c{c}"] = {"inputs": {"base_color": col, "specular_roughness": 0.55}}

nodes = []
for x in range(N):
    for y in range(N - x):                      # x + y < N: the triangle itself
        c = carries(x, y)
        h = STEP * (K + 1 - c)
        material = f"c{c}" if c else f"top{comb(x + y, x) % P}"
        nodes.append({
            "type": "mesh",
            "name": f"t{x}_{y}",
            "translation": [round((x + 0.5) * CELL - 0.33, 5), round(h / 2, 5), round((y + 0.5) * CELL - 0.33, 5)],
            "geometry": {"std": "box", "size": [round(CELL * GAP, 5), round(h, 5), round(CELL * GAP, 5)]},
            "materials": [material],
        })

nodes += [
    {"type": "mesh", "name": "floor", "geometry": {"std": "grid", "size": [40, 40]},
     "materials": [{"inputs": {"base_color": [0.012, 0.014, 0.02], "specular_roughness": 0.6}}]},
    # low warm key raking across the terraces, cool fill from above
    {"type": "light", "shape": "rectangle", "translation": [-1.7, 0.5, 0.9], "target": [0.0, 0.05, 0.0],
     "energy": 60, "size": 0.9, "color": [1.0, 0.82, 0.62]},
    {"type": "light", "shape": "rectangle", "translation": [0.8, 1.8, -1.0], "target": [0.0, 0.0, 0.0],
     "energy": 22, "size": 1.6, "color": [0.55, 0.7, 1.0]},
    # high enough that shallow pits are not hidden behind the ridges
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
