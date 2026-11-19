"""The generation ship from "turnover", partway through its flip. Units are metres, scaled down 1:100.

A spine with a shield at the front, three habitat decks, tanks, and the engine bells at the back.
The decks are flat drums across the axis, not spinning rings: in the story, weight comes from thrust,
so "down" is toward the engines and the floors lie across the ship.
Stars are small emissive spheres on a large shell; there is no sky.

Usage: python make_scene.py [ANGLE_DEGREES] > scene.json
"""
import json, math, random, sys

ANGLE = math.radians(float(sys.argv[1]) if len(sys.argv) > 1 else 62.0)
rng = random.Random(106)
M = {
    "hull": {"inputs": {"base_color": [0.42, 0.42, 0.41], "specular_roughness": 0.55}},
    "dark": {"inputs": {"base_color": [0.10, 0.10, 0.11], "specular_roughness": 0.6}},
    "shield": {"inputs": {"base_color": [0.30, 0.22, 0.14], "specular_roughness": 0.8}},
    "tank": {"inputs": {"base_color": [0.75, 0.70, 0.55], "base_metalness": 0.6, "specular_roughness": 0.35}},
    "bell": {"inputs": {"base_color": [0.20, 0.19, 0.18], "base_metalness": 1.0, "specular_roughness": 0.4}},
    "window": {"inputs": {"base_color": [1.0, 0.8, 0.5], "emission_color": [1.0, 0.72, 0.38], "emission_luminance": 60}},
    "star": {"inputs": {"base_color": [1, 1, 1], "emission_color": [1.0, 0.97, 0.92], "emission_luminance": 2500}},
    "starw": {"inputs": {"base_color": [1, 1, 1], "emission_color": [1.0, 0.8, 0.6], "emission_luminance": 2000}},
}
ship = []          # children of one node whose long axis is local +Y (front) / -Y (engines)


def cyl(name, y, radius, height, mat, seg=48):
    ship.append({"type": "mesh", "name": name, "translation": [0, y, 0],
                 "geometry": {"std": "cylinder", "radius": radius, "height": height, "segments": seg}, "materials": [mat]})


cyl("spine", 0.0, 0.10, 9.0, "dark", 16)
ship.append({"type": "mesh", "name": "shield", "translation": [0, 4.75, 0],
             "geometry": {"std": "cone", "radius": 1.3, "height": 0.6, "segments": 64}, "materials": ["shield"]})
cyl("shieldback", 4.42, 1.3, 0.08, "dark", 64)
for i, y in enumerate((2.6, 1.5, 0.4)):
    ship.append({"type": "mesh", "name": f"ring{i}", "translation": [0, y, 0],
                 "geometry": {"std": "cylinder", "radius": 1.2, "height": 0.34, "segments": 96}, "materials": ["hull"]})
    for k in range(4):
        a = k * math.pi / 2 + i * 0.5
        ship.append({"type": "mesh", "name": f"spoke{i}_{k}", "translation": [0.52 * math.cos(a), y, 0.52 * math.sin(a)],
                     "rotation": {"euler": [0, -a, 1.5708]},
                     "geometry": {"std": "cylinder", "radius": 0.025, "height": 1.0, "segments": 8}, "materials": ["dark"]})
    for k in range(48):                                   # lit windows round the outside of each ring
        if rng.random() < 0.55:
            a = k * 2 * math.pi / 48
            ship.append({"type": "mesh", "name": f"w{i}_{k}", "translation": [1.2 * math.cos(a), y + rng.choice([-0.08, 0.0, 0.08]), 1.2 * math.sin(a)],
                         "geometry": {"std": "box", "size": [0.03, 0.035, 0.03]}, "materials": ["window"]})
for k in range(6):                                        # propellant tanks round the spine
    a = k * math.pi / 3
    ship.append({"type": "mesh", "name": f"tank{k}", "translation": [0.42 * math.cos(a), -1.9, 0.42 * math.sin(a)],
                 "geometry": {"std": "capsule", "radius": 0.26, "height": 2.4, "segments": 24, "rings": 12}, "materials": ["tank"]})
cyl("thrust", -3.6, 0.6, 0.5, "dark", 32)
for k in range(4):
    a = k * math.pi / 2 + 0.785
    ship.append({"type": "mesh", "name": f"bell{k}", "translation": [0.36 * math.cos(a), -4.25, 0.36 * math.sin(a)],
                 "geometry": {"std": "cone", "radius": 0.34, "height": 0.9, "segments": 32}, "materials": ["bell"]})

N = [{"type": "node", "name": "ship", "rotation": {"euler": [0.25, 0.5, round(ANGLE, 4)]}, "children": ship}]
for i in range(2600):
    u, v = rng.uniform(-1, 1), rng.uniform(0, 2 * math.pi)
    r = 600.0
    s = math.sqrt(1 - u * u)
    size = rng.choice([0.25, 0.25, 0.3, 0.35, 0.45, 0.7])
    N.append({"type": "mesh", "name": f"s{i}", "translation": [round(r * s * math.cos(v), 1), round(r * u, 1), round(r * s * math.sin(v), 1)],
              "geometry": {"std": "sphere", "radius": size, "segments": 6, "rings": 4},
              "materials": ["starw" if rng.random() < 0.2 else "star"]})
N += [
    {"type": "light", "shape": "environment", "color": [0.004, 0.005, 0.008], "energy": 1.0},
    {"type": "light", "shape": "sun", "name": "sunlight", "rotation": {"euler": [-0.5, -1.0, 0]}, "energy": 2.2,
     "color": [1.0, 0.96, 0.9], "angle": 0.01},
    {"type": "camera", "translation": [4.0, 2.0, 20.0], "target": [0.0, 0.0, 0.0]},
]
json.dump({
    "$schema": "https://raw.githubusercontent.com/voyager-so/vork/v0.6.1/packages/ts/scene.schema.json",
    "version": 2,
    "composition": {"space": "3d", "view": {"program": "std/agx", "inputs": {"exposure": 0.0}}},
    "materials": M, "nodes": N,
    "render": {"resolution": [1500, 1000], "duration": 1},
}, sys.stdout, separators=(",", ":"))
