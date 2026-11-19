"""The ridge between Station 9 and Salt Creek, seen from the warm side at low sun.
Terrain is a flat grid displaced by dunes.slang (a custom vork program).

Usage: python make_scene.py [SUN_ROT] [SUN_ELEV] [EXPOSURE] > scene.json
"""
import json, os, subprocess, sys

SUN_ROT = float(sys.argv[1]) if len(sys.argv) > 1 else 1.2
SUN_ELEV = float(sys.argv[2]) if len(sys.argv) > 2 else 0.07
EXPOSURE = float(sys.argv[3]) if len(sys.argv) > 3 else -1.8

VORK = os.path.join(os.environ["LOCALAPPDATA"], "Programs", "vork", "vork.exe")
sky = json.loads(subprocess.run(
    [VORK, "std", "sky", json.dumps({"sun_elevation": SUN_ELEV, "sun_rotation": SUN_ROT})],
    capture_output=True, text=True, check=True, env={**os.environ, "VORK_NO_UPDATE_NOTIFIER": "1"}).stdout)
M = dict(sky["materials"])
M["dunes"] = {"program": "dunes", "inputs": {"height": 1.0}}
N = list(sky["nodes"])
N.append({"type": "mesh", "name": "land", "translation": [0, 0, -40],
          "geometry": {"std": "grid", "size": [240, 160], "segments": [900, 600]}, "materials": ["dunes"]})
N.append({"type": "camera", "translation": [0.0, 6.5, 36.0], "target": [0.0, 5.0, -18.0]})

json.dump({
    "$schema": "https://raw.githubusercontent.com/voyager-so/vork/v0.6.1/packages/ts/scene.schema.json",
    "version": 2,
    "composition": {"space": "3d", "view": {"program": "std/agx", "inputs": {"exposure": EXPOSURE}}},
    "programs": {"dunes": {"file": "dunes.slang"}},
    "materials": M, "nodes": N,
    "render": {"resolution": [1500, 1000], "duration": 1},
}, sys.stdout, separators=(",", ":"))
