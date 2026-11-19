"""Salt Creek after the sun has gone: the same scene as make_scene.py, later, with the lamp
flickering. A 6-second loop.

Usage: python make_night.py > night.json   (then vork render night.json --out lamp.mp4)
"""
import json, math, subprocess, sys

SECONDS, FPS = 6, 24
scene = json.loads(subprocess.run([sys.executable, "make_scene.py", "0.45", "-0.045", "0.9"],
                                  capture_output=True, text=True, check=True).stdout)
scene["nodes"] = [n for n in scene["nodes"] if n.get("shape") != "sun"]
frames = SECONDS * FPS
lum, spill = [], []
for f in range(frames + 1):
    t = 2 * math.pi * f / frames
    # a sum of whole-number harmonics, so the flicker closes on itself
    k = 1.0 + 0.10 * math.sin(3 * t) + 0.07 * math.sin(7 * t + 1.3) + 0.05 * math.sin(13 * t + 0.4) + 0.04 * math.sin(23 * t + 2.0)
    lum.append([f"{f}/{FPS}", round(260 * k, 3)])
    spill.append([f"{f}/{FPS}", round(5.0 * k, 4)])
scene["animation"] = [{"name": "flicker", "tracks": [
    {"path": ["materials", "lamp", "inputs", "emission_luminance"], "keys": lum},
    {"path": ["nodes", "spill", "energy"], "keys": spill},
]}]
scene["render"] = {"resolution": [900, 600], "duration": SECONDS, "fps": FPS}
json.dump(scene, sys.stdout, separators=(",", ":"))
