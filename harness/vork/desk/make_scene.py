"""The keeper's desk at Salt Creek: the log sheet (the 2D piece, used as a texture), a lamp, a pencil, a mug.

Needs sheet.png beside it: crop of ../logsheet/log.png (ffmpeg -vf crop=1060:1380:70:60).
Usage: python make_scene.py > scene.json
"""
import json, math, sys

SW, SH = 0.212, 0.276          # sheet, metres
LX, LZ = -0.19, -0.11          # lamp position
nodes = [
    {"type": "mesh", "name": "desk", "geometry": {"std": "grid", "size": [6, 6]}, "materials": ["wood"]},
    {"type": "mesh", "name": "sheet", "translation": [0.03, 0.0008, 0.02], "rotation": {"euler": [0, 0.10, 0]},
     "geometry": {"std": "grid", "size": [SW, SH]}, "materials": ["paper"]},
    {"type": "mesh", "name": "under", "translation": [0.045, 0.0004, 0.012], "rotation": {"euler": [0, -0.05, 0]},
     "geometry": {"std": "grid", "size": [SW, SH]}, "materials": ["blank"]},
    # lamp: brass font, collar, glass chimney suggested by a dark ring, flame
    {"type": "mesh", "name": "font", "translation": [LX, 0.022, LZ],
     "geometry": {"std": "cylinder", "radius": 0.045, "height": 0.044, "segments": 48}, "materials": ["brass"]},
    {"type": "mesh", "name": "collar", "translation": [LX, 0.054, LZ],
     "geometry": {"std": "cylinder", "radius": 0.018, "height": 0.02, "segments": 32}, "materials": ["brass"]},
    {"type": "mesh", "name": "flame", "translation": [LX, 0.083, LZ], "scale": [0.6, 1.0, 0.6],
     "geometry": {"std": "capsule", "radius": 0.008, "height": 0.034}, "materials": ["flame"]},
    {"type": "light", "shape": "point", "name": "lamp", "translation": [LX, 0.086, LZ], "energy": 1.5,
     "radius": 0.012, "color": [1.0, 0.58, 0.24]},
    # pencil, lying across the corner of the sheet
    {"type": "mesh", "name": "pencil", "translation": [0.165, 0.0045, 0.11], "rotation": {"euler": [1.5708, 0.5, 0]},
     "geometry": {"std": "cylinder", "radius": 0.0037, "height": 0.15, "segments": 6}, "materials": ["pencil"]},
    # mug
    {"type": "mesh", "name": "mug", "translation": [0.215, 0.04, -0.12],
     "geometry": {"std": "cylinder", "radius": 0.038, "height": 0.08, "segments": 48}, "materials": ["tin"]},
    {"type": "mesh", "name": "tea", "translation": [0.215, 0.0805, -0.12],
     "geometry": {"std": "cylinder", "radius": 0.035, "height": 0.001, "segments": 48}, "materials": ["tea"]},
    {"type": "camera", "translation": [0.03, 0.56, 0.56], "target": [0.01, 0.02, -0.03]},
]
json.dump({
    "$schema": "https://raw.githubusercontent.com/voyager-so/vork/v0.6.1/packages/ts/scene.schema.json",
    "version": 2,
    "composition": {"space": "3d", "view": {"program": "std/agx", "inputs": {"exposure": 0.3}}},
    "images": {"log": {"file": "sheet.png"}},
    "materials": {
        "wood": {"inputs": {"base_color": [0.10, 0.058, 0.032], "specular_roughness": 0.5}},
        "paper": {"inputs": {"base_color": {"image": "log"}, "specular_roughness": 0.85}},
        "blank": {"inputs": {"base_color": [0.80, 0.76, 0.66], "specular_roughness": 0.85}},
        "brass": {"inputs": {"base_color": [0.80, 0.58, 0.26], "base_metalness": 1.0, "specular_roughness": 0.3}},
        "flame": {"inputs": {"base_color": [1.0, 0.7, 0.3], "emission_color": [1.0, 0.6, 0.22], "emission_luminance": 3000}},
        "pencil": {"inputs": {"base_color": [0.55, 0.12, 0.08], "specular_roughness": 0.4}},
        "tin": {"inputs": {"base_color": [0.55, 0.56, 0.58], "base_metalness": 1.0, "specular_roughness": 0.4}},
        "tea": {"inputs": {"base_color": [0.05, 0.02, 0.01], "specular_roughness": 0.05}},
    },
    "nodes": nodes,
    "render": {"resolution": [1500, 1000], "duration": 1},
}, sys.stdout, separators=(",", ":"))
