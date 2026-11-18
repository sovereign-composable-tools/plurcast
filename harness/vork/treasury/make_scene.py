"""The treasury: rule30's wallet balance, two sats, as two coins on a table.

Usage: python make_scene.py > scene.json
"""
import json, sys

R, T = 0.10, 0.008          # coin radius and half-thickness (scene units, not real size)
profile = [[0, T * 0.55], [R * 0.30, T * 0.55], [R * 0.33, T * 0.8], [R * 0.36, T * 0.55],
           [R * 0.80, T * 0.55], [R * 0.84, T], [R * 0.96, T], [R, T * 0.6],
           [R, -T * 0.6], [R * 0.96, -T], [R * 0.84, -T], [R * 0.80, -T * 0.55], [0, -T * 0.55]]
coin = {"std": "revolve", "profile": [[round(a, 5), round(b, 5)] for a, b in profile], "segments": 96}

scene = {
    "$schema": "https://raw.githubusercontent.com/voyager-so/vork/v0.6.1/packages/ts/scene.schema.json",
    "version": 2,
    "composition": {"space": "3d", "view": {"program": "std/agx", "inputs": {"exposure": 0.0}}},
    "geometry": {"coin": coin},
    "materials": {
        "gold": {"inputs": {"base_color": [0.95, 0.66, 0.22], "base_metalness": 1.0, "specular_roughness": 0.28}},
        "table": {"inputs": {"base_color": [0.09, 0.055, 0.035], "specular_roughness": 0.55}},
    },
    "nodes": [
        {"type": "mesh", "name": "table", "geometry": {"std": "grid", "size": [20, 20]}, "materials": ["table"]},
        {"type": "mesh", "name": "coin1", "translation": [0, T, 0], "geometry": "coin", "materials": ["gold"]},
        # the second coin leans on the rim of the first
        {"type": "mesh", "name": "coin2", "translation": [0.185, 0.0185, 0.03],
         "rotation": {"euler": [0.0, 0.0, -0.085]}, "geometry": "coin", "materials": ["gold"]},
        {"type": "light", "shape": "rectangle", "translation": [-0.5, 0.35, -0.6], "target": [0.05, 0, 0],
         "energy": 9, "size": 0.5, "color": [1.0, 0.85, 0.7]},
        {"type": "light", "shape": "rectangle", "translation": [0.7, 0.6, 0.5], "target": [0.05, 0, 0],
         "energy": 2.0, "size": 0.9, "color": [0.6, 0.72, 1.0]},
        {"type": "camera", "translation": [0.12, 0.26, 0.62], "target": [0.09, 0.01, 0.0]},
    ],
    "render": {"resolution": [1500, 1000], "duration": 1},
}
json.dump(scene, sys.stdout, separators=(",", ":"))
