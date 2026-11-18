"""Salt Creek relay at dusk (station world, fragment 2).

A hut with one lit window, a guyed mast with a receiver dish, and the creek
with its salt banks. No formula; a place.

Usage: python make_scene.py [SUN_ROT] [SUN_ELEV] [EXPOSURE] > scene.json
"""
import json, math, os, random, subprocess, sys

SUN_ROT = float(sys.argv[1]) if len(sys.argv) > 1 else 0.45
SUN_ELEV = float(sys.argv[2]) if len(sys.argv) > 2 else 0.03
EXPOSURE = float(sys.argv[3]) if len(sys.argv) > 3 else -1.6
rng = random.Random(14)

VORK = os.path.join(os.environ["LOCALAPPDATA"], "Programs", "vork", "vork.exe")
sky = json.loads(subprocess.run(
    [VORK, "std", "sky", json.dumps({"sun_elevation": SUN_ELEV, "sun_rotation": SUN_ROT})],
    capture_output=True, text=True, check=True, env={**os.environ, "VORK_NO_UPDATE_NOTIFIER": "1"}).stdout)

M = dict(sky["materials"])
M.update({
    "earth": {"inputs": {"base_color": [0.16, 0.11, 0.08], "specular_roughness": 0.95}},
    "salt": {"inputs": {"base_color": [0.62, 0.60, 0.56], "specular_roughness": 0.8}},
    "water": {"inputs": {"base_color": [0.01, 0.02, 0.03], "specular_roughness": 0.03}},
    "wall": {"inputs": {"base_color": [0.22, 0.20, 0.18], "specular_roughness": 0.85}},
    "roof": {"inputs": {"base_color": [0.10, 0.09, 0.09], "specular_roughness": 0.6}},
    "steel": {"inputs": {"base_color": [0.30, 0.30, 0.31], "base_metalness": 0.8, "specular_roughness": 0.45}},
    "lamp": {"inputs": {"base_color": [0.9, 0.6, 0.3], "emission_color": [1.0, 0.62, 0.28], "emission_luminance": 900}},
    "stone": {"inputs": {"base_color": [0.20, 0.18, 0.16], "specular_roughness": 0.85}},
})
N = list(sky["nodes"])


def box(name, pos, size, mat, yaw=0.0):
    N.append({"type": "mesh", "name": name, "translation": [round(v, 4) for v in pos],
              "rotation": {"euler": [0, round(yaw, 4), 0]},
              "geometry": {"std": "box", "size": [round(v, 4) for v in size]}, "materials": [mat]})


def rod(name, a, b, radius, mat="steel"):
    """A cylinder from point a to point b (cylinders are built along Y)."""
    d = [b[i] - a[i] for i in range(3)]
    L = math.sqrt(sum(v * v for v in d))
    u = [v / L for v in d]
    # quaternion rotating (0,1,0) onto u
    w = 1 + u[1]
    q = [u[2], 0.0, -u[0], w]
    n = math.sqrt(sum(v * v for v in q))
    q = [round(v / n, 6) for v in q]
    N.append({"type": "mesh", "name": name, "translation": [round((a[i] + b[i]) / 2, 4) for i in range(3)],
              "rotation": q, "geometry": {"std": "cylinder", "radius": radius, "height": round(L, 4), "segments": 10},
              "materials": [mat]})


# Ground, creek and salt banks. The creek runs along x at z = +4 (between camera and hut).
N.append({"type": "mesh", "name": "earth", "geometry": {"std": "grid", "size": [400, 400]}, "materials": ["earth"]})
box("creek", (0, 0.004, 4.0), (400, 0.008, 2.6), "water")
for side, zc in (("n", 2.45), ("s", 5.55)):
    box(f"bank_{side}", (0, 0.012, zc), (400, 0.024, 0.55), "salt")
    for i in range(160):
        x = rng.uniform(-30, 30)
        s = rng.uniform(0.08, 0.35)
        box(f"crust_{side}{i}", (x, 0.02, zc + rng.uniform(-0.6, 0.6)), (s * 2, 0.03, s), "salt", rng.uniform(0, 3.14))

# The hut.
HX, HZ = -1.6, -2.0
box("hut", (HX, 1.1, HZ), (3.4, 2.2, 2.6), "wall")
box("roof", (HX, 2.26, HZ), (3.9, 0.12, 3.1), "roof")
box("window", (HX + 0.75, 1.35, HZ + 1.305), (0.7, 0.55, 0.02), "lamp")
box("door", (HX - 0.8, 0.9, HZ + 1.305), (0.75, 1.8, 0.03), "roof")
box("step", (HX - 0.8, 0.06, HZ + 1.6), (1.0, 0.12, 0.5), "stone")
N.append({"type": "light", "shape": "point", "name": "spill", "translation": [HX + 0.75, 1.35, HZ + 1.7],
          "energy": 14, "radius": 0.15, "color": [1.0, 0.62, 0.28]})

# The mast, guyed three ways, with cross-arms and the receiver dish.
MX, MZ, MH = 2.6, -3.2, 11.0
rod("mast", (MX, 0, MZ), (MX, MH, MZ), 0.06)
for k in range(3):
    a = 1.25 + k * 2 * math.pi / 3
    for j, h in enumerate((MH * 0.55, MH * 0.95)):
        rod(f"guy{k}_{j}", (MX, h, MZ), (MX + 5.5 * math.cos(a), 0, MZ + 5.5 * math.sin(a)), 0.012)
    box(f"anchor{k}", (MX + 5.5 * math.cos(a), 0.1, MZ + 5.5 * math.sin(a)), (0.35, 0.2, 0.35), "stone", a)
for j, h in enumerate((MH * 0.7, MH * 0.82, MH * 0.94)):
    rod(f"arm{j}", (MX - 0.9 + 0.2 * j, h, MZ), (MX + 0.9 - 0.2 * j, h, MZ), 0.02)
N.append({"type": "mesh", "name": "dish", "translation": [MX + 0.35, MH * 0.45, MZ + 0.3],
          "rotation": {"euler": [1.15, 0.6, 0]},
          "geometry": {"std": "cone", "radius": 0.75, "height": 0.3, "segments": 40}, "materials": ["steel"]})
rod("feed", (MX, MH * 0.45, MZ), (MX + 0.35, MH * 0.45, MZ + 0.3), 0.03)
rod("cable", (MX, 0.3, MZ), (HX + 1.7, 2.0, HZ - 0.4), 0.015, "roof")

# Loose stones on the near ground.
for i in range(260):
    x, z = rng.uniform(-25, 25), rng.uniform(-30, 12)
    if 2.0 < z < 6.0 or (abs(x - HX) < 2.2 and abs(z - HZ) < 2.0):
        continue
    r = rng.choice([0.04, 0.06, 0.1, 0.16])
    N.append({"type": "mesh", "name": f"p{i}", "translation": [round(x, 3), round(r * 0.3, 3), round(z, 3)],
              "scale": [r, r * 0.5, round(r * rng.uniform(0.6, 1.0), 3)],
              "geometry": {"std": "sphere", "radius": 1, "segments": 16, "rings": 8}, "materials": ["stone"]})

N.append({"type": "camera", "translation": [-6.0, 3.2, 24.0], "target": [0.8, 4.2, -2.5]})

json.dump({
    "$schema": "https://raw.githubusercontent.com/voyager-so/vork/v0.6.1/packages/ts/scene.schema.json",
    "version": 2,
    "composition": {"space": "3d", "view": {"program": "std/agx", "inputs": {"exposure": EXPOSURE}}},
    "materials": M, "nodes": N,
    "render": {"resolution": [1500, 1000], "duration": 1},
}, sys.stdout, separators=(",", ":"))
