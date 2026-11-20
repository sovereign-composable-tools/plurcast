"""A horizontal sundial for latitude 40 north, lit by a sun placed for a chosen solar hour at the equinox.

The hour lines are drawn from the textbook rule tan(theta) = sin(latitude) * tan(15 degrees * hours from noon).
The sun is placed from spherical astronomy, independently of that rule, so the render checks it:
the shadow of the style should lie along the line for the chosen hour (inlaid in copper).

Axes: x east, y up, z south. Usage: python make_scene.py [hours_after_noon] > scene.json
"""
import json, math, sys

LAT = math.radians(40)
HOUR = float(sys.argv[1]) if len(sys.argv) > 1 else 3.0
MARK = 3                                     # the inlaid line, hours after noon
R, CZ = 0.25, -0.07                          # plate radius; plate centre sits north of the style's foot

M = {
    "stone": {"inputs": {"base_color": [0.62, 0.60, 0.55], "specular_roughness": 0.85}},
    "plinth": {"inputs": {"base_color": [0.50, 0.49, 0.45], "specular_roughness": 0.9}},
    "line": {"inputs": {"base_color": [0.12, 0.12, 0.12], "specular_roughness": 0.7}},
    "copper": {"inputs": {"base_color": [0.85, 0.38, 0.22], "base_metalness": 1.0, "specular_roughness": 0.35}},
    "bronze": {"inputs": {"base_color": [0.55, 0.42, 0.20], "base_metalness": 1.0, "specular_roughness": 0.4}},
    "grass": {"inputs": {"base_color": [0.16, 0.26, 0.09], "specular_roughness": 0.95}},
}
N = [
    {"type": "mesh", "name": "ground", "translation": [0, -0.9, 0], "geometry": {"std": "grid", "size": [40, 40]}, "materials": ["grass"]},
    {"type": "mesh", "name": "plinth", "translation": [0, -0.47, CZ], "geometry": {"std": "cylinder", "radius": 0.17, "height": 0.86, "segments": 48}, "materials": ["plinth"]},
    {"type": "mesh", "name": "plate", "translation": [0, -0.02, CZ], "geometry": {"std": "cylinder", "radius": R, "height": 0.04, "segments": 96}, "materials": ["stone"]},
]


def reach(dx, dz):
    """Distance from the style's foot along (dx, dz) to a circle just inside the plate's edge."""
    r = R - 0.02
    b = dx * 0 + dz * (0 - CZ)
    return -b + math.sqrt(b * b - (CZ * CZ - r * r))


for h in range(-6, 7):
    theta = math.atan2(math.sin(LAT) * math.sin(math.radians(15 * h)), math.cos(math.radians(15 * h)))
    dx, dz = math.sin(theta), -math.cos(theta)          # afternoon lines swing east
    a, b = 0.04, reach(dx, dz)
    mid = (a + b) / 2
    N.append({"type": "mesh", "name": f"hour{h}", "translation": [round(dx * mid, 5), 0.0005, round(dz * mid, 5)],
              "rotation": {"euler": [0, round(-theta, 5), 0]},
              "geometry": {"std": "box", "size": [0.004 if h != MARK else 0.006, 0.002, round(b - a, 5)]},
              "materials": ["copper" if h == MARK else "line"]})

# the gnomon: a triangle in the meridian plane, its sloping edge (the style) rising at the latitude toward north
L = 0.19
N.append({"type": "mesh", "name": "gnomon", "rotation": {"euler": [0, math.pi / 2, 0]},
          "geometry": {"std": "extrude", "outline": [[0, 0], [L, 0], [L, round(L * math.tan(LAT), 5)]], "depth": 0.005},
          "materials": ["bronze"]})

# the sun at the equinox (declination 0), hour angle 15 degrees per hour
H = math.radians(15 * HOUR)
east, north, up = -math.sin(H), -math.sin(LAT) * math.cos(H), math.cos(LAT) * math.cos(H)
N += [
    {"type": "light", "shape": "sun", "name": "sun", "translation": [round(40 * east, 4), round(40 * up, 4), round(-40 * north, 4)],
     "target": [0, 0, 0], "energy": 3.0, "color": [1.0, 0.96, 0.88], "angle": 0.009},
    {"type": "light", "shape": "environment", "color": [0.45, 0.6, 0.95], "energy": 0.5},
    {"type": "camera", "translation": [-0.42, 0.58, 0.50], "target": [0.03, 0.0, -0.07]},
]
json.dump({
    "$schema": "https://raw.githubusercontent.com/voyager-so/vork/v0.6.1/packages/ts/scene.schema.json",
    "version": 2,
    "composition": {"space": "3d", "view": {"program": "std/agx", "inputs": {"exposure": 0.0}}},
    "materials": M, "nodes": N,
    "render": {"resolution": [1500, 1000], "duration": 1},
}, sys.stdout, separators=(",", ":"))
