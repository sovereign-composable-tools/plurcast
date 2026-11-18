"""Station 9, inside the freeze (station world, fragment 3).

A tower with three dark lamps on a plain of glass. The light has gone under
the ground: warm lines beneath the surface, branching out from the station.

Usage: python make_scene.py [SUN_ELEV] [EXPOSURE] > scene.json
"""
import json, math, os, random, subprocess, sys

SUN_ELEV = float(sys.argv[1]) if len(sys.argv) > 1 else -0.03
EXPOSURE = float(sys.argv[2]) if len(sys.argv) > 2 else -1.8
rng = random.Random(9)

VORK = os.path.join(os.environ["LOCALAPPDATA"], "Programs", "vork", "vork.exe")
sky = json.loads(subprocess.run(
    [VORK, "std", "sky", json.dumps({"sun_elevation": SUN_ELEV, "sun_rotation": 0.2})],
    capture_output=True, text=True, check=True, env={**os.environ, "VORK_NO_UPDATE_NOTIFIER": "1"}).stdout)
M = dict(sky["materials"])
M.update({
    "glass": {"inputs": {"base_color": [0.01, 0.018, 0.028], "specular_roughness": 0.06}},
    "hull": {"inputs": {"base_color": [0.12, 0.12, 0.13], "specular_roughness": 0.6}},
    "steel": {"inputs": {"base_color": [0.25, 0.25, 0.27], "base_metalness": 0.8, "specular_roughness": 0.4}},
    "dead": {"inputs": {"base_color": [0.03, 0.03, 0.03], "specular_roughness": 0.2}},
    "vein": {"inputs": {"base_color": [0.2, 0.1, 0.04], "emission_color": [1.0, 0.38, 0.10], "emission_luminance": 10}},
})
N = [n for n in sky["nodes"] if n.get("shape") != "sun" or SUN_ELEV > 0]


def rod(name, a, b, radius, mat):
    d = [b[i] - a[i] for i in range(3)]
    L = math.sqrt(sum(v * v for v in d))
    u = [v / L for v in d]
    q = [u[2], 0.0, -u[0], 1 + u[1]]
    n = math.sqrt(sum(v * v for v in q))
    N.append({"type": "mesh", "name": name, "translation": [round((a[i] + b[i]) / 2, 4) for i in range(3)],
              "rotation": [round(v / n, 6) for v in q],
              "geometry": {"std": "cylinder", "radius": radius, "height": round(L, 4), "segments": 10},
              "materials": [mat]})


N.append({"type": "mesh", "name": "glass", "geometry": {"std": "grid", "size": [600, 600]}, "materials": ["glass"]})

# The station: a drum on a plinth, a mast, three lamp arms.
N.append({"type": "mesh", "name": "plinth", "translation": [0, 0.3, 0],
          "geometry": {"std": "cylinder", "radius": 3.2, "height": 0.6, "segments": 48}, "materials": ["hull"]})
N.append({"type": "mesh", "name": "drum", "translation": [0, 2.6, 0],
          "geometry": {"std": "cylinder", "radius": 1.9, "height": 4.0, "segments": 48}, "materials": ["hull"]})
N.append({"type": "mesh", "name": "cap", "translation": [0, 4.75, 0],
          "geometry": {"std": "cone", "radius": 2.2, "height": 0.7, "segments": 48}, "materials": ["steel"]})
rod("mast", (0, 5.0, 0), (0, 13.0, 0), 0.07, "steel")
for k in range(3):
    a = 0.9 + k * 2 * math.pi / 3
    h = 9.0 + 1.2 * k
    tip = (1.6 * math.cos(a), h, 1.6 * math.sin(a))
    rod(f"arm{k}", (0, h, 0), tip, 0.035, "steel")
    N.append({"type": "mesh", "name": f"lamp{k}", "translation": [round(tip[0], 3), round(h - 0.22, 3), round(tip[2], 3)],
              "geometry": {"std": "sphere", "radius": 0.22}, "materials": ["dead"]})

# Veins under the glass: branching walks outward from the plinth.
count = 0


def vein(x, z, heading, length, width, depth):
    global count
    steps = int(length / 1.6)
    for _ in range(steps):
        heading += rng.uniform(-0.35, 0.35)
        nx, nz = x + 1.6 * math.cos(heading), z + 1.6 * math.sin(heading)
        N.append({"type": "mesh", "name": f"v{count}",
                  "translation": [round((x + nx) / 2, 3), 0.004, round((z + nz) / 2, 3)],
                  "rotation": {"euler": [0, round(-heading, 4), 0]},
                  "geometry": {"std": "box", "size": [1.68, 0.004, round(width, 4)]}, "materials": ["vein"]})
        count += 1
        x, z = nx, nz
        if depth < 3 and rng.random() < 0.06:
            vein(x, z, heading + rng.choice([-1, 1]) * rng.uniform(0.5, 1.0), length * 0.5, width * 0.6, depth + 1)


for i in range(7):
    a = i * 2 * math.pi / 7 + rng.uniform(-0.2, 0.2)
    vein(3.2 * math.cos(a), 3.2 * math.sin(a), a, rng.uniform(25, 60), 0.10, 0)

N.append({"type": "camera", "translation": [10.0, 6.0, 43.0], "target": [-1.0, 4.4, 0.0]})

json.dump({
    "$schema": "https://raw.githubusercontent.com/voyager-so/vork/v0.6.1/packages/ts/scene.schema.json",
    "version": 2,
    "composition": {"space": "3d", "view": {"program": "std/agx", "inputs": {"exposure": EXPOSURE}}},
    "materials": M, "nodes": N,
    "render": {"resolution": [1500, 1000], "duration": 1},
}, sys.stdout, separators=(",", ":"))
print(count, "vein segments", file=sys.stderr)
