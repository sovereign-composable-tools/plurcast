"""Kef's cairn on the freeze boundary (station world, fragment 1).

Warm, pebbled ground on one side; flat dark glass on the other; a straight
edge between them, and a cairn of seven stones stacked on the warm side.
Not a formula piece: the only arithmetic is a seeded scatter of pebbles.

Usage: python make_scene.py [SUN_ROTATION] [SUN_ELEVATION] | (sky is merged by vork std sky)
"""
import json
import math
import random
import subprocess
import os
import sys

SUN_ROT = float(sys.argv[1]) if len(sys.argv) > 1 else 2.6
SUN_ELEV = float(sys.argv[2]) if len(sys.argv) > 2 else 0.14
EXPOSURE = float(sys.argv[3]) if len(sys.argv) > 3 else -2.5
rng = random.Random(9)

VORK = os.path.join(os.environ["LOCALAPPDATA"], "Programs", "vork", "vork.exe")
sky = json.loads(subprocess.run(
    [VORK, "std", "sky", json.dumps({"sun_elevation": SUN_ELEV, "sun_rotation": SUN_ROT})],
    capture_output=True, text=True, check=True, env={**os.environ, "VORK_NO_UPDATE_NOTIFIER": "1"}).stdout)

materials = dict(sky["materials"])
materials.update({
    "earth": {"inputs": {"base_color": [0.30, 0.19, 0.11], "specular_roughness": 0.9}},
    "glass": {"inputs": {"base_color": [0.015, 0.03, 0.045], "specular_roughness": 0.04}},
    "stone_a": {"inputs": {"base_color": [0.20, 0.18, 0.16], "specular_roughness": 0.8}},
    "stone_b": {"inputs": {"base_color": [0.13, 0.12, 0.12], "specular_roughness": 0.85}},
    "stone_c": {"inputs": {"base_color": [0.26, 0.20, 0.14], "specular_roughness": 0.75}},
})
STONES = ["stone_a", "stone_b", "stone_c"]


def stone(name, pos, size, yaw, tilt, mat):
    return {"type": "mesh", "name": name, "translation": [round(v, 4) for v in pos],
            "scale": [round(v, 4) for v in size],
            "rotation": {"euler": [round(tilt, 3), round(yaw, 3), round(rng.uniform(-0.06, 0.06), 3)]},
            "geometry": {"std": "sphere", "radius": 1, "segments": 40, "rings": 20},
            "materials": [mat]}


nodes = list(sky["nodes"])
# Ground: fluid side is x < 0, frozen side is x > 0. The glass sits a hair lower.
nodes.append({"type": "mesh", "name": "earth", "translation": [-30, 0, 0],
              "geometry": {"std": "grid", "size": [60, 120]}, "materials": ["earth"]})
nodes.append({"type": "mesh", "name": "glass", "translation": [30, -0.004, 0],
              "geometry": {"std": "grid", "size": [60, 120]}, "materials": ["glass"]})

# The cairn: seven stones, each smaller, each a little off centre.
CX, CZ = -0.32, 0.0
y = 0.0
for i in range(9):
    w = 0.24 * (0.86 ** i) * rng.uniform(0.85, 1.1)
    h = w * rng.uniform(0.32, 0.48)
    d = w * rng.uniform(0.6, 0.9)
    nodes.append({"type": "mesh", "name": f"cairn{i}",
                  "translation": [round(CX + rng.uniform(-0.025, 0.025), 4), round(y + h / 2, 4), round(CZ + rng.uniform(-0.025, 0.025), 4)],
                  "rotation": {"euler": [round(rng.uniform(-0.05, 0.05), 3), round(rng.uniform(0, math.pi), 3), round(rng.uniform(-0.05, 0.05), 3)]},
                  "geometry": {"std": "rounded_box", "size": [round(2 * w, 4), round(h, 4), round(2 * d, 4)],
                               "radius": round(h * rng.uniform(0.12, 0.25), 4), "segments": 4},
                  "materials": [STONES[i % 3]]})
    y += h * 0.97

# Pebbles on the warm side only, denser near the camera, none within the cairn's footprint.
count = 0
while count < 900:
    x = -rng.uniform(0.02, 9.0)
    z = rng.uniform(-12.0, 5.0)
    if math.hypot(x - CX, z - CZ) < 0.36:
        continue
    r = rng.choice([0.012, 0.018, 0.025, 0.035, 0.05, 0.08])
    nodes.append(stone(f"p{count}", (x, r * 0.35, z), (r, r * 0.5, r * rng.uniform(0.6, 1.0)),
                       rng.uniform(0, math.pi), 0.0, rng.choice(STONES)))
    count += 1

nodes.append({"type": "camera", "translation": [2.5, 0.38, 1.3], "target": [-0.30, 0.22, 0.0]})

scene = {
    "$schema": "https://raw.githubusercontent.com/voyager-so/vork/v0.6.1/packages/ts/scene.schema.json",
    "version": 2,
    "composition": {"space": "3d", "view": {"program": "std/agx", "inputs": {"exposure": EXPOSURE}}},
    "materials": materials,
    "nodes": nodes,
    "render": {"resolution": [1500, 1000], "duration": 1},
}
json.dump(scene, sys.stdout, separators=(",", ":"))
