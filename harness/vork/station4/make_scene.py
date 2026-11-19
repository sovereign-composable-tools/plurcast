"""Station 4 on the far dunes at dusk, one lamp lit (station world). Terrain from ../ridge/dunes.slang.

Usage: python make_scene.py [EXPOSURE] > scene.json
"""
import json, math, os, subprocess, sys

EXPOSURE = float(sys.argv[1]) if len(sys.argv) > 1 else -0.6
VORK = os.path.join(os.environ["LOCALAPPDATA"], "Programs", "vork", "vork.exe")
sky = json.loads(subprocess.run(
    [VORK, "std", "sky", json.dumps({"sun_elevation": -0.035, "sun_rotation": 0.9})],
    capture_output=True, text=True, check=True, env={**os.environ, "VORK_NO_UPDATE_NOTIFIER": "1"}).stdout)
M = dict(sky["materials"])
M.update({
    "dunes": {"program": "dunes", "inputs": {"height": 1.0}},
    "hull": {"inputs": {"base_color": [0.10, 0.10, 0.11], "specular_roughness": 0.6}},
    "dead": {"inputs": {"base_color": [0.03, 0.03, 0.03], "specular_roughness": 0.3}},
    "lamp": {"inputs": {"base_color": [1.0, 0.7, 0.3], "emission_color": [1.0, 0.55, 0.2], "emission_luminance": 40000}},
})
N = [n for n in sky["nodes"] if n.get("shape") != "sun"]
N.append({"type": "mesh", "name": "land", "translation": [0, 0, -40],
          "geometry": {"std": "grid", "size": [240, 160], "segments": [900, 600]}, "materials": ["dunes"]})

# The station stands on the main ridge (the swell in dunes.slang peaks near world z = -58).
SX, SZ, BASE = 14.0, -58.0, 6.0
N.append({"type": "mesh", "name": "drum", "translation": [SX, BASE + 5.0, SZ],
          "geometry": {"std": "cylinder", "radius": 1.3, "height": 10.0, "segments": 32}, "materials": ["hull"]})
N.append({"type": "mesh", "name": "cap", "translation": [SX, BASE + 10.3, SZ],
          "geometry": {"std": "cone", "radius": 1.6, "height": 0.6, "segments": 32}, "materials": ["hull"]})
N.append({"type": "mesh", "name": "mast", "translation": [SX, BASE + 14.0, SZ],
          "geometry": {"std": "cylinder", "radius": 0.06, "height": 7.5, "segments": 8}, "materials": ["hull"]})
for k in range(3):
    a = 0.4 + k * 2 * math.pi / 3
    h = BASE + 13.2 + 1.1 * k
    x, z = SX + 1.3 * math.cos(a), SZ + 1.3 * math.sin(a)
    lit = k == 1
    N.append({"type": "mesh", "name": f"arm{k}", "translation": [(SX + x) / 2, h, (SZ + z) / 2],
              "rotation": {"euler": [0, -a, 0]},
              "geometry": {"std": "box", "size": [1.3, 0.05, 0.05]}, "materials": ["hull"]})
    N.append({"type": "mesh", "name": f"lamp{k}", "translation": [x, h - 0.2, z],
              "geometry": {"std": "sphere", "radius": 0.2}, "materials": ["lamp" if lit else "dead"]})
    if lit:
        N.append({"type": "light", "shape": "point", "name": "glow", "translation": [x, h - 0.2, z],
                  "energy": 550, "radius": 0.2, "color": [1.0, 0.55, 0.2]})
N.append({"type": "camera", "translation": [0.0, 6.5, 36.0], "target": [4.0, 8.0, -18.0]})

json.dump({
    "$schema": "https://raw.githubusercontent.com/voyager-so/vork/v0.6.1/packages/ts/scene.schema.json",
    "version": 2,
    "composition": {"space": "3d", "view": {"program": "std/agx", "inputs": {"exposure": EXPOSURE}}},
    "programs": {"dunes": {"file": "../ridge/dunes.slang"}},
    "materials": M, "nodes": N,
    "render": {"resolution": [1500, 1000], "duration": 1},
}, sys.stdout, separators=(",", ":"))
