"""The one tree on the walkers' road, bare, at last light (station world).

Branching is recursive: each limb ends in two or three children. Radii follow Leonardo's rule:
the cross-sections of the children add up to the cross-section of the parent (r_parent^2 = sum r_child^2).
The script checks that sum at every fork.

Usage: python make_scene.py [SEED] > scene.json
"""
import json, math, os, random, subprocess, sys

SEED = int(sys.argv[1]) if len(sys.argv) > 1 else 7
rng = random.Random(SEED)
VORK = os.path.join(os.environ["LOCALAPPDATA"], "Programs", "vork", "vork.exe")
sky = json.loads(subprocess.run(
    [VORK, "std", "sky", json.dumps({"sun_elevation": 0.035, "sun_rotation": 0.55})],
    capture_output=True, text=True, check=True, env={**os.environ, "VORK_NO_UPDATE_NOTIFIER": "1"}).stdout)
M = dict(sky["materials"])
M.update({
    "earth": {"inputs": {"base_color": [0.17, 0.11, 0.08], "specular_roughness": 0.95}},
    "bark": {"inputs": {"base_color": [0.07, 0.055, 0.045], "specular_roughness": 0.9}},
    "stone": {"inputs": {"base_color": [0.18, 0.16, 0.14], "specular_roughness": 0.85}},
    "post": {"inputs": {"base_color": [0.10, 0.085, 0.07], "specular_roughness": 0.8}},
    "band": {"inputs": {"base_color": [0.75, 0.72, 0.62], "specular_roughness": 0.6}},
})
N = list(sky["nodes"])
N.append({"type": "mesh", "name": "earth", "geometry": {"std": "grid", "size": [800, 800]}, "materials": ["earth"]})
count = 0
worst = 0.0


def norm(v):
    L = math.sqrt(sum(c * c for c in v))
    return [c / L for c in v]


def limb(a, d, length, r0, r1, mat="bark"):
    """A tapered limb from a along d, as three short cylinders."""
    global count
    for i in range(3):
        p0 = [a[k] + d[k] * length * i / 3 for k in range(3)]
        p1 = [a[k] + d[k] * length * (i + 1) / 3 for k in range(3)]
        r = r0 + (r1 - r0) * (i + 0.5) / 3
        q = [d[2], 0.0, -d[0], 1 + d[1]]
        n = math.sqrt(sum(c * c for c in q)) or 1.0
        N.append({"type": "mesh", "name": f"b{count}", "translation": [round((p0[k] + p1[k]) / 2, 4) for k in range(3)],
                  "rotation": [round(c / n, 5) for c in q],
                  "geometry": {"std": "cylinder", "radius": round(r, 4), "height": round(length / 3 * 1.04, 4), "segments": 8},
                  "materials": [mat]})
        count += 1


def grow(a, d, length, r, depth):
    global worst
    end = [a[k] + d[k] * length for k in range(3)]
    kids = 3 if (depth < 3 and rng.random() < 0.35) else 2
    shares = [rng.uniform(0.6, 1.4) for _ in range(kids)]
    total = sum(shares)
    radii = [r * math.sqrt(s / total) for s in shares]          # Leonardo: areas add up
    worst = max(worst, abs(sum(x * x for x in radii) - r * r) / (r * r))
    limb(a, d, length, r, max(radii) if depth < 9 else r * 0.5)
    if depth >= 9 or r < 0.004:
        return
    # a frame around d, to spread the children
    ref = [0.0, 1.0, 0.0] if abs(d[1]) < 0.9 else [1.0, 0.0, 0.0]
    u = norm([d[1] * ref[2] - d[2] * ref[1], d[2] * ref[0] - d[0] * ref[2], d[0] * ref[1] - d[1] * ref[0]])
    w = [d[1] * u[2] - d[2] * u[1], d[2] * u[0] - d[0] * u[2], d[0] * u[1] - d[1] * u[0]]
    base = rng.uniform(0, 2 * math.pi)
    for i, cr in enumerate(radii):
        spread = rng.uniform(0.35, 0.75) * (1.25 - 0.5 * cr / max(radii))    # thin children turn further
        az = base + 2 * math.pi * i / kids + rng.uniform(-0.4, 0.4)
        nd = [d[k] * math.cos(spread) + (u[k] * math.cos(az) + w[k] * math.sin(az)) * math.sin(spread) for k in range(3)]
        nd[1] += 0.10                                                          # reach up a little
        nd[0] += 0.05                                                          # and lean with the wind
        grow(end, norm(nd), length * rng.uniform(0.68, 0.82), cr, depth + 1)


grow([0.0, 0.0, 0.0], norm([0.06, 1.0, 0.0]), 1.5, 0.19, 0)
print(count, "cylinders; worst area error at a fork", f"{worst:.2e}", file=sys.stderr)
assert worst < 1e-9

for i in range(8):                                  # the road's posts, passing the tree
    z = -9 - 14 * i
    x = -5.5 + 0.0005 * z * z
    N.append({"type": "mesh", "name": f"post{i}", "translation": [round(x, 2), 1.25, z],
              "geometry": {"std": "cylinder", "radius": 0.07, "height": 2.5, "segments": 10}, "materials": ["post"]})
    N.append({"type": "mesh", "name": f"band{i}", "translation": [round(x, 2), 2.15, z],
              "geometry": {"std": "cylinder", "radius": 0.078, "height": 0.16, "segments": 10}, "materials": ["band"]})
for i in range(350):
    x, z = rng.uniform(-40, 40), rng.uniform(-120, 12)
    if abs(x) < 0.5 and abs(z) < 0.5:
        continue
    s = rng.choice([0.04, 0.07, 0.11, 0.16])
    N.append({"type": "mesh", "name": f"p{i}", "translation": [round(x, 2), round(s * 0.3, 3), round(z, 2)],
              "scale": [s, s * 0.5, round(s * rng.uniform(0.6, 1.0), 3)],
              "geometry": {"std": "sphere", "radius": 1, "segments": 12, "rings": 6}, "materials": ["stone"]})
N.append({"type": "camera", "translation": [1.5, 1.5, 13.5], "target": [0.3, 3.4, 0.0]})

json.dump({
    "$schema": "https://raw.githubusercontent.com/voyager-so/vork/v0.6.1/packages/ts/scene.schema.json",
    "version": 2,
    "composition": {"space": "3d", "view": {"program": "std/agx", "inputs": {"exposure": -1.7}}},
    "materials": M, "nodes": N,
    "render": {"resolution": [1200, 1500], "duration": 1},
}, sys.stdout, separators=(",", ":"))
