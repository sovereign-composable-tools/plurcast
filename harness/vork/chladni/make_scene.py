"""Sand on a vibrating plate (a Chladni figure), idealised. The pattern is drawn by plate.slang.

Usage: python make_scene.py [N] [M] > scene.json
"""
import json, sys

N_ = float(sys.argv[1]) if len(sys.argv) > 1 else 3
M_ = float(sys.argv[2]) if len(sys.argv) > 2 else 5
HALF = 0.25
json.dump({
    "$schema": "https://raw.githubusercontent.com/voyager-so/vork/v0.6.1/packages/ts/scene.schema.json",
    "version": 2,
    "programs": {"plate": {"file": "plate.slang"}},
    "materials": {
        "plate": {"program": "plate", "inputs": {"n": N_, "m": M_, "half": HALF}},
        "edge": {"inputs": {"base_color": [0.78, 0.57, 0.26], "base_metalness": 1.0, "specular_roughness": 0.35}},
        "post": {"inputs": {"base_color": [0.05, 0.05, 0.055], "specular_roughness": 0.4}},
        "bench": {"inputs": {"base_color": [0.045, 0.03, 0.022], "specular_roughness": 0.6}},
    },
    "nodes": [
        {"type": "mesh", "name": "bench", "geometry": {"std": "grid", "size": [10, 10]}, "materials": ["bench"]},
        {"type": "mesh", "name": "post", "translation": [0, 0.06, 0],
         "geometry": {"std": "cylinder", "radius": 0.012, "height": 0.12}, "materials": ["post"]},
        {"type": "mesh", "name": "slab", "translation": [0, 0.1215, 0],
         "geometry": {"std": "box", "size": [2 * HALF, 0.003, 2 * HALF]}, "materials": ["edge"]},
        {"type": "mesh", "name": "top", "translation": [0, 0.1232, 0],
         "geometry": {"std": "grid", "size": [2 * HALF, 2 * HALF], "segments": [500, 500]}, "materials": ["plate"]},
        {"type": "light", "shape": "rectangle", "translation": [-0.5, 0.75, 0.3], "target": [0, 0.12, 0],
         "energy": 26, "size": 0.5, "color": [1.0, 0.92, 0.82]},
        {"type": "light", "shape": "rectangle", "translation": [0.7, 0.4, -0.5], "target": [0, 0.12, 0],
         "energy": 6, "size": 0.8, "color": [0.6, 0.72, 1.0]},
        {"type": "camera", "translation": [0.10, 0.86, 0.50], "target": [0, 0.10, 0.02]},
    ],
    "render": {"resolution": [1200, 1200], "duration": 1},
}, sys.stdout, separators=(",", ":"))
