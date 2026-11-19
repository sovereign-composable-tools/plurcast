"""Station 6, the top of the mast at dusk: two lamps steady, one unlit, one of the two flickering.
A loop. Lamp brightness is keyed on the materials' emission and on a point light at each lamp.

Usage: python make_scene.py > scene.json   (then vork render scene.json --out lamps.mp4)
"""
import json, math, os, subprocess, sys

SECONDS, FPS = 8, 24
VORK = os.path.join(os.environ["LOCALAPPDATA"], "Programs", "vork", "vork.exe")
sky = json.loads(subprocess.run(
    [VORK, "std", "sky", json.dumps({"sun_elevation": -0.05, "sun_rotation": 2.6})],
    capture_output=True, text=True, check=True, env={**os.environ, "VORK_NO_UPDATE_NOTIFIER": "1"}).stdout)
M = dict(sky["materials"])
M.update({
    "steel": {"inputs": {"base_color": [0.16, 0.16, 0.17], "base_metalness": 0.7, "specular_roughness": 0.5}},
    "glass_dead": {"inputs": {"base_color": [0.05, 0.05, 0.05], "specular_roughness": 0.15}},
})
N = [n for n in sky["nodes"] if n.get("shape") != "sun"]
N.append({"type": "mesh", "name": "mast", "translation": [0, 0, 0],
          "geometry": {"std": "cylinder", "radius": 0.05, "height": 6.0, "segments": 12}, "materials": ["steel"]})
N.append({"type": "mesh", "name": "finial", "translation": [0, 3.05, 0],
          "geometry": {"std": "cone", "radius": 0.07, "height": 0.2, "segments": 12}, "materials": ["steel"]})

frames = SECONDS * FPS
tracks = []
ARMS = [(0.5, 0.35, "steady"), (0.5 + 2.2, 1.25, "flicker"), (0.5 + 4.1, 2.1, "dead")]
for k, (a, h, kind) in enumerate(ARMS):
    x, z = 1.15 * math.cos(a), 1.15 * math.sin(a)
    N.append({"type": "mesh", "name": f"arm{k}", "translation": [x / 2, h, z / 2], "rotation": {"euler": [0, -a, 0]},
              "geometry": {"std": "box", "size": [1.15, 0.045, 0.045]}, "materials": ["steel"]})
    N.append({"type": "mesh", "name": f"hood{k}", "translation": [x, h - 0.03, z],
              "geometry": {"std": "cone", "radius": 0.2, "height": 0.14, "segments": 24}, "materials": ["steel"]})
    if kind == "dead":
        N.append({"type": "mesh", "name": f"lamp{k}", "translation": [x, h - 0.2, z],
                  "geometry": {"std": "sphere", "radius": 0.14}, "materials": ["glass_dead"]})
        continue
    M[f"lamp{k}"] = {"inputs": {"base_color": [1.0, 0.7, 0.3], "emission_color": [1.0, 0.56, 0.2], "emission_luminance": 320}}
    N.append({"type": "mesh", "name": f"lamp{k}", "translation": [x, h - 0.2, z],
              "geometry": {"std": "sphere", "radius": 0.14}, "materials": [f"lamp{k}"]})
    N.append({"type": "light", "shape": "point", "name": f"glow{k}", "translation": [x, h - 0.2, z],
              "energy": 30, "radius": 0.14, "color": [1.0, 0.56, 0.2]})
    lum, en = [], []
    for f in range(frames + 1):
        t = 2 * math.pi * f / frames
        if kind == "steady":
            v = 1.0 + 0.03 * math.sin(5 * t)
        else:
            # mostly on, with two stumbles per loop; whole-number harmonics so it closes
            v = 0.75 + 0.2 * math.sin(3 * t) + 0.12 * math.sin(11 * t + 1.0) + 0.1 * math.sin(29 * t + 0.3)
            dip = max(0.0, math.sin(2 * t + 0.6)) ** 14 + max(0.0, math.sin(3 * t + 2.4)) ** 20
            v = max(0.03, v * (1.0 - 0.97 * min(1.0, dip)))
        lum.append([f"{f}/{FPS}", round(320 * v, 2)])
        en.append([f"{f}/{FPS}", round(30 * v, 3)])
    tracks.append({"path": ["materials", f"lamp{k}", "inputs", "emission_luminance"], "keys": lum})
    tracks.append({"path": ["nodes", f"glow{k}", "energy"], "keys": en})

N.append({"type": "camera", "translation": [2.4, -1.6, 4.6], "target": [0.1, 1.45, 0.0]})
json.dump({
    "$schema": "https://raw.githubusercontent.com/voyager-so/vork/v0.6.1/packages/ts/scene.schema.json",
    "version": 2,
    "composition": {"space": "3d", "view": {"program": "std/agx", "inputs": {"exposure": 0.4}}},
    "animation": [{"name": "lamps", "tracks": tracks}],
    "materials": M, "nodes": N,
    "render": {"resolution": [900, 900], "duration": SECONDS, "fps": FPS},
}, sys.stdout, separators=(",", ":"))
