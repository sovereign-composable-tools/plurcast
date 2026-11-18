"""An abelian sandpile, rendered as an actual pile.

GRAINS grains are dropped on one cell of an empty grid. Any cell holding four
or more gives one grain to each of its four neighbours, until nothing can
topple. Every cell then holds 0, 1, 2 or 3 grains. Each cell is drawn as a
stack that many chips high.

Usage: python make_scene.py [GRAINS] > scene.json    (default 12000)
"""
import json
import sys
from collections import Counter

GRAINS = int(sys.argv[1]) if len(sys.argv) > 1 else 12000

# Topple in bulk: a cell with n >= 4 grains sends n // 4 to each neighbour.
# The final state does not depend on the order (that is the "abelian" part).
grid = {(0, 0): GRAINS}
unstable = [(0, 0)]
while unstable:
    cell = unstable.pop()
    n = grid.get(cell, 0)
    if n < 4:
        continue
    share = n // 4
    grid[cell] = n % 4
    x, y = cell
    for nb in ((x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)):
        grid[nb] = grid.get(nb, 0) + share
        if grid[nb] >= 4:
            unstable.append(nb)

assert sum(grid.values()) == GRAINS and max(grid.values()) <= 3
radius = max(max(abs(x), abs(y)) for x, y in grid)
print(f"grains {GRAINS}, radius {radius}, cells {len(grid)}, heights {sorted(Counter(grid.values()).items())}",
      file=sys.stderr)

CELL = 0.5 / radius          # the pile spans about 1 metre
CHIP = CELL * 0.85           # thickness of one grain
PLATE = CELL * 0.08          # empty cells inside the pile get a thin plate
COLOURS = {
    0: (0.03, 0.045, 0.08),
    1: (0.07, 0.12, 0.20),
    2: (0.18, 0.36, 0.50),
    3: (0.92, 0.70, 0.40),
}
materials = {f"h{h}": {"inputs": {"base_color": list(c), "specular_roughness": 0.45}} for h, c in COLOURS.items()}

nodes = []
for (x, y), h in grid.items():
    height = PLATE + CHIP * h
    nodes.append({
        "type": "mesh",
        "name": f"s{x}_{y}",
        "translation": [round(x * CELL, 5), round(height / 2, 5), round(y * CELL, 5)],
        "geometry": {"std": "box", "size": [round(CELL * 0.9, 5), round(height, 5), round(CELL * 0.9, 5)]},
        "materials": [f"h{h}"],
    })

nodes += [
    {"type": "mesh", "name": "floor", "geometry": {"std": "grid", "size": [40, 40]},
     "materials": [{"inputs": {"base_color": [0.012, 0.014, 0.02], "specular_roughness": 0.6}}]},
    {"type": "light", "shape": "rectangle", "translation": [-1.8, 0.45, 1.0], "target": [0.0, 0.0, 0.0],
     "energy": 60, "size": 0.9, "color": [1.0, 0.82, 0.62]},
    {"type": "light", "shape": "rectangle", "translation": [0.8, 1.9, -1.0], "target": [0.0, 0.0, 0.0],
     "energy": 26, "size": 1.6, "color": [0.55, 0.7, 1.0]},
    {"type": "camera", "translation": [-0.22, 1.72, -0.50], "target": [0.0, 0.0, 0.01]},
]

scene = {
    "$schema": "https://raw.githubusercontent.com/voyager-so/vork/v0.6.1/packages/ts/scene.schema.json",
    "version": 2,
    "materials": materials,
    "nodes": nodes,
    "render": {"resolution": [1200, 1200], "duration": 1},
}
json.dump(scene, sys.stdout, separators=(",", ":"))
