"""A pinch of salt from the creek bank, close up. Salt crystallises in cubes, so the heap is cubes.
No physics: crystals are placed in shells of a cone and jittered, which reads as a heap from this distance.

Usage: python make_scene.py [COUNT] > scene.json
"""
import json, math, random, sys

COUNT = int(sys.argv[1]) if len(sys.argv) > 1 else 9000
rng = random.Random(11)
nodes = [{"type": "mesh", "name": "slate", "geometry": {"std": "grid", "size": [60, 60]}, "materials": ["slate"]}]
HEAP_R, HEAP_H = 0.44, 0.22        # scene units: 20x life size, so the camera is not inside vork's near clip
for i in range(COUNT):
    s = rng.choice([0.010, 0.014, 0.018, 0.024]) * rng.uniform(0.8, 1.2)
    if i < COUNT * 0.86:
        r = HEAP_R * math.sqrt(rng.random())
        a = rng.uniform(0, 2 * math.pi)
        top = HEAP_H * (1 - r / HEAP_R) ** 1.3
        y = s / 2 + top * rng.random() ** 0.6
    else:                      # strays around the heap
        r = HEAP_R * rng.uniform(1.0, 2.6)
        a = rng.uniform(0, 2 * math.pi)
        y = s / 2
    nodes.append({"type": "mesh", "name": f"c{i}",
                  "translation": [round(r * math.cos(a), 4), round(y, 4), round(r * math.sin(a), 4)],
                  "rotation": {"euler": [round(rng.uniform(0, 1.5), 2), round(rng.uniform(0, 1.5), 2), round(rng.uniform(0, 1.5), 2)] if y > s else [0, round(rng.uniform(0, 1.5), 2), 0]},
                  "geometry": {"std": "box", "size": [round(s, 5), round(s * rng.uniform(0.7, 1.0), 5), round(s * rng.uniform(0.8, 1.0), 5)]},
                  "materials": ["salt"]})
nodes += [
    {"type": "light", "shape": "rectangle", "translation": [-2.4, 1.0, 0.6], "target": [0, 0.08, 0],
     "energy": 200, "size": 1.2, "color": [1.0, 0.9, 0.78]},
    {"type": "light", "shape": "rectangle", "translation": [1.6, 2.0, -1.6], "target": [0, 0.08, 0],
     "energy": 48, "size": 2.4, "color": [0.6, 0.72, 1.0]},
    {"type": "camera", "translation": [0.0, 0.55, 1.45], "target": [0.0, 0.08, 0.0]},
]
json.dump({
    "$schema": "https://raw.githubusercontent.com/voyager-so/vork/v0.6.1/packages/ts/scene.schema.json",
    "version": 2,
    "composition": {"space": "3d", "view": {"program": "std/agx", "inputs": {"exposure": 0.9}}},
    "materials": {
        "slate": {"inputs": {"base_color": [0.03, 0.035, 0.04], "specular_roughness": 0.55}},
        "salt": {"inputs": {"base_color": [0.93, 0.92, 0.9], "specular_roughness": 0.18,
                            "transmission_weight": 0.3, "specular_ior": 1.54}},
    },
    "nodes": nodes,
    "render": {"resolution": [1500, 1000], "duration": 1},
}, sys.stdout, separators=(",", ":"))
