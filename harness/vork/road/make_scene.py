"""The walkers' road in ground mist: a line of marker posts going out of sight. Station world.
First use of world fog (mist.slang bound to the sky light's medium).

Usage: python make_scene.py [DENSITY] [EXPOSURE] > scene.json
"""
import json, math, os, random, subprocess, sys

DENSITY = float(sys.argv[1]) if len(sys.argv) > 1 else 0.014
EXPOSURE = float(sys.argv[2]) if len(sys.argv) > 2 else -3.1
rng = random.Random(4)
VORK = os.path.join(os.environ["LOCALAPPDATA"], "Programs", "vork", "vork.exe")
sky = json.loads(subprocess.run(
    [VORK, "std", "sky", json.dumps({"sun_elevation": 0.16, "sun_rotation": 0.35})],
    capture_output=True, text=True, check=True, env={**os.environ, "VORK_NO_UPDATE_NOTIFIER": "1"}).stdout)
M = dict(sky["materials"])
M.update({
    "mist": {"program": "mist", "inputs": {"density": DENSITY}},
    "earth": {"inputs": {"base_color": [0.20, 0.14, 0.10], "specular_roughness": 0.95}},
    "post": {"inputs": {"base_color": [0.10, 0.085, 0.07], "specular_roughness": 0.8}},
    "band": {"inputs": {"base_color": [0.75, 0.72, 0.62], "specular_roughness": 0.6}},
    "stone": {"inputs": {"base_color": [0.18, 0.16, 0.14], "specular_roughness": 0.85}},
})
N = []
for n in sky["nodes"]:
    if n.get("shape") == "environment":
        n = dict(n, medium="mist")
    N.append(n)
N.append({"type": "mesh", "name": "earth", "geometry": {"std": "grid", "size": [800, 800]}, "materials": ["earth"]})

# Posts every 14 m along a road that bends slowly to the left.
for i in range(26):
    z = -6 - 14 * i
    x = 2.2 + 0.012 * (z ** 2) / 10 * 0.35 * -1 if False else 2.2 - 0.0009 * z * z
    lean = rng.uniform(-0.05, 0.05)
    h = rng.uniform(2.3, 2.7)
    N.append({"type": "mesh", "name": f"post{i}", "translation": [round(x, 3), h / 2, z],
              "rotation": {"euler": [round(rng.uniform(-0.03, 0.03), 3), 0, round(lean, 3)]},
              "geometry": {"std": "cylinder", "radius": 0.07, "height": round(h, 3), "segments": 12}, "materials": ["post"]})
    N.append({"type": "mesh", "name": f"band{i}", "translation": [round(x - lean * h * 0.35, 3), round(h * 0.86, 3), z],
              "geometry": {"std": "cylinder", "radius": 0.078, "height": 0.16, "segments": 12}, "materials": ["band"]})
for i in range(500):
    x, z = rng.uniform(-40, 40), rng.uniform(-200, 8)
    r = rng.choice([0.04, 0.07, 0.11, 0.18])
    N.append({"type": "mesh", "name": f"p{i}", "translation": [round(x, 2), round(r * 0.3, 3), round(z, 2)],
              "scale": [r, r * 0.5, round(r * rng.uniform(0.6, 1.0), 3)],
              "geometry": {"std": "sphere", "radius": 1, "segments": 12, "rings": 6}, "materials": ["stone"]})
N.append({"type": "camera", "translation": [0.0, 1.6, 6.0], "target": [-1.5, 1.9, -60.0]})

json.dump({
    "$schema": "https://raw.githubusercontent.com/voyager-so/vork/v0.6.1/packages/ts/scene.schema.json",
    "version": 2,
    "composition": {"space": "3d", "view": {"program": "std/agx", "inputs": {"exposure": EXPOSURE}}},
    "programs": {"mist": {"file": "mist.slang"}},
    "materials": M, "nodes": N,
    "render": {"resolution": [1500, 1000], "duration": 1},
}, sys.stdout, separators=(",", ":"))
