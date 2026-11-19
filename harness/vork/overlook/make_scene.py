"""From the ridge after dark: the frozen plain, Station 9, and how far the lines under the glass go.
Dunes from ../ridge/dunes.slang in the foreground; beyond the boundary a flat glass plane with a
branching network of emissive lines spreading from the station.

Usage: python make_scene.py [EXPOSURE] > scene.json
"""
import json, math, os, random, subprocess, sys

EXPOSURE = float(sys.argv[1]) if len(sys.argv) > 1 else -0.2
rng = random.Random(99)
VORK = os.path.join(os.environ["LOCALAPPDATA"], "Programs", "vork", "vork.exe")
sky = json.loads(subprocess.run(
    [VORK, "std", "sky", json.dumps({"sun_elevation": -0.06, "sun_rotation": 0.15})],
    capture_output=True, text=True, check=True, env={**os.environ, "VORK_NO_UPDATE_NOTIFIER": "1"}).stdout)
M = dict(sky["materials"])
M.update({
    "dunes": {"program": "dunes", "inputs": {"height": 1.0}},
    "glass": {"inputs": {"base_color": [0.008, 0.014, 0.022], "specular_roughness": 0.07}},
    "hull": {"inputs": {"base_color": [0.05, 0.05, 0.055], "specular_roughness": 0.6}},
    "vein": {"inputs": {"base_color": [0.2, 0.1, 0.04], "emission_color": [1.0, 0.36, 0.08], "emission_luminance": 7}},
})
N = [n for n in sky["nodes"] if n.get("shape") != "sun"]
N.append({"type": "mesh", "name": "land", "translation": [0, 0, -40],
          "geometry": {"std": "grid", "size": [240, 160], "segments": [900, 600]}, "materials": ["dunes"]})
GY, EDGE = 3.4, -84.0          # glass level, and the boundary: a straight line, thirty paces past the crest
N.append({"type": "mesh", "name": "glass", "translation": [0, GY, EDGE - 1500],
          "geometry": {"std": "grid", "size": [6000, 3000]}, "materials": ["glass"]})

SX, SZ = 30.0, -330.0
N.append({"type": "mesh", "name": "plinth", "translation": [SX, GY + 0.3, SZ],
          "geometry": {"std": "cylinder", "radius": 3.2, "height": 0.6, "segments": 32}, "materials": ["hull"]})
N.append({"type": "mesh", "name": "drum", "translation": [SX, GY + 2.6, SZ],
          "geometry": {"std": "cylinder", "radius": 1.9, "height": 4.0, "segments": 32}, "materials": ["hull"]})
N.append({"type": "mesh", "name": "mast", "translation": [SX, GY + 9.0, SZ],
          "geometry": {"std": "cylinder", "radius": 0.12, "height": 9.0, "segments": 8}, "materials": ["hull"]})

count = 0
STEP = 7.0


def vein(x, z, heading, length, width, depth):
    global count
    for _ in range(int(length / STEP)):
        heading += rng.uniform(-0.3, 0.3)
        nx, nz = x + STEP * math.cos(heading), z + STEP * math.sin(heading)
        if nz > EDGE - 1.0:          # the lines stop at the boundary
            return
        N.append({"type": "mesh", "name": f"v{count}",
                  "translation": [round((x + nx) / 2, 2), GY + 0.02, round((z + nz) / 2, 2)],
                  "rotation": {"euler": [0, round(-heading, 4), 0]},
                  "geometry": {"std": "box", "size": [STEP * 1.05, 0.02, round(width, 3)]}, "materials": ["vein"]})
        count += 1
        x, z = nx, nz
        if depth < 3 and rng.random() < 0.035:
            vein(x, z, heading + rng.choice([-1, 1]) * rng.uniform(0.5, 1.1), length * 0.6, width * 0.7, depth + 1)


for i in range(8):
    a = i * 2 * math.pi / 8 + rng.uniform(-0.2, 0.2)
    vein(SX + 3.2 * math.cos(a), SZ + 3.2 * math.sin(a), a, rng.uniform(250, 520), 0.55, 0)
print(count, "segments", file=sys.stderr)
N.append({"type": "camera", "translation": [0.0, 17.0, -30.0], "target": [14.0, 4.0, -260.0]})

json.dump({
    "$schema": "https://raw.githubusercontent.com/voyager-so/vork/v0.6.1/packages/ts/scene.schema.json",
    "version": 2,
    "composition": {"space": "3d", "view": {"program": "std/agx", "inputs": {"exposure": EXPOSURE}}},
    "programs": {"dunes": {"file": "../ridge/dunes.slang"}},
    "materials": M, "nodes": N,
    "render": {"resolution": [1500, 1000], "duration": 1},
}, sys.stdout, separators=(",", ":"))
