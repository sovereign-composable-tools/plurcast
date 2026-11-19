"""A pin tumbler lock cut open, with the right key in it. Units are centimetres.

Each chamber holds a key pin (brass, resting on the key), a driver pin (steel) and a spring.
The key's cuts are computed from the key-pin lengths so that every key pin ends exactly at the
shear line, the top of the plug. That is checked before the scene is written.

Usage: python make_scene.py [BITTING like 31425] > scene.json
"""
import json, math, sys

BITTING = [int(c) for c in (sys.argv[1] if len(sys.argv) > 1 else "31425")]
SHEAR = 0.60            # top of the plug
KEYWAY_TOP, KEYWAY_BOTTOM = 0.02, -0.46
PITCH, X0, R = 0.80, 0.0, 0.15
DRIVER = 0.50
TOP = 2.15              # top of the chambers
pin_len = [0.62 + 0.06 * b for b in BITTING]                 # key pin lengths by code
cut_y = [SHEAR - L for L in pin_len]                          # where the key must hold each pin
assert all(KEYWAY_BOTTOM + 0.1 < c < KEYWAY_TOP + 0.001 for c in cut_y), cut_y
assert all(abs(c + L - SHEAR) < 1e-12 for c, L in zip(cut_y, pin_len))

M = {
    "housing": {"inputs": {"base_color": [0.55, 0.40, 0.16], "base_metalness": 1.0, "specular_roughness": 0.42}},
    "plug": {"inputs": {"base_color": [0.80, 0.62, 0.28], "base_metalness": 1.0, "specular_roughness": 0.34}},
    "void": {"inputs": {"base_color": [0.015, 0.014, 0.012], "specular_roughness": 0.9}},
    "keypin": {"inputs": {"base_color": [0.92, 0.70, 0.30], "base_metalness": 1.0, "specular_roughness": 0.22}},
    "driver": {"inputs": {"base_color": [0.72, 0.73, 0.75], "base_metalness": 1.0, "specular_roughness": 0.25}},
    "spring": {"inputs": {"base_color": [0.55, 0.56, 0.58], "base_metalness": 1.0, "specular_roughness": 0.3}},
    "key": {"inputs": {"base_color": [0.78, 0.78, 0.74], "base_metalness": 1.0, "specular_roughness": 0.3}},
    "back": {"inputs": {"base_color": [0.03, 0.035, 0.045], "specular_roughness": 0.8}},
}
N = []


def box(name, cx, cy, w, h, depth, z, mat):
    N.append({"type": "mesh", "name": name, "translation": [round(cx, 4), round(cy, 4), z],
              "geometry": {"std": "box", "size": [round(w, 4), round(h, 4), depth]}, "materials": [mat]})


XL, XR = -0.75, X0 + PITCH * (len(BITTING) - 1) + 0.75
W = XR - XL
N.append({"type": "mesh", "name": "back", "translation": [0, 0, -1.2], "rotation": {"euler": [1.5708, 0, 0]},
          "geometry": {"std": "grid", "size": [60, 60]}, "materials": ["back"]})
box("housing", (XL + XR) / 2, (SHEAR + TOP + 0.3) / 2, W, TOP + 0.3 - SHEAR, 0.6, -0.30, "housing")
box("plug", (XL + XR) / 2, (SHEAR + (-0.75)) / 2, W, SHEAR + 0.75, 0.6, -0.30, "plug")
box("shearline", (XL + XR) / 2, SHEAR, W, 0.012, 0.02, 0.011, "void")
box("keyway", (XL + XR) / 2 - 0.2, (KEYWAY_TOP + KEYWAY_BOTTOM) / 2, W + 0.4, KEYWAY_TOP - KEYWAY_BOTTOM, 0.02, 0.012, "void")

for i, b in enumerate(BITTING):
    x = X0 + PITCH * i
    box(f"chamber{i}", x, (cut_y[i] + TOP) / 2, 2 * R + 0.05, TOP - cut_y[i], 0.02, 0.014, "void")
    # key pin: a cylinder with a rounded tip resting in the key's cut
    L = pin_len[i]
    N.append({"type": "mesh", "name": f"keypin{i}", "translation": [x, round(cut_y[i] + 0.07 + (L - 0.07) / 2, 4), 0.19],
              "geometry": {"std": "cylinder", "radius": R, "height": round(L - 0.07, 4), "segments": 32}, "materials": ["keypin"]})
    N.append({"type": "mesh", "name": f"tip{i}", "translation": [x, round(cut_y[i] + 0.035, 4), 0.19],
              "rotation": {"euler": [3.14159, 0, 0]},
              "geometry": {"std": "cone", "radius": R, "height": 0.07, "segments": 32}, "materials": ["keypin"]})
    N.append({"type": "mesh", "name": f"driver{i}", "translation": [x, SHEAR + DRIVER / 2 + 0.006, 0.19],
              "geometry": {"std": "cylinder", "radius": R, "height": DRIVER - 0.012, "segments": 32}, "materials": ["driver"]})
    coils = 7
    for k in range(coils):
        y = SHEAR + DRIVER + 0.06 + (TOP - SHEAR - DRIVER - 0.1) * k / (coils - 1)
        N.append({"type": "mesh", "name": f"coil{i}_{k}", "translation": [x, round(y, 4), 0.19],
                  "rotation": {"euler": [0.16, 0, 0]},
                  "geometry": {"std": "torus", "radius": R - 0.03, "tube": 0.018, "segments": 24, "rings": 6}, "materials": ["spring"]})

# The key: blade profile with a V cut under each pin, then the shoulder and a round bow.
top = KEYWAY_TOP - 0.02
pts = [[XR - 0.25, KEYWAY_BOTTOM + 0.03], [XR - 0.05, KEYWAY_BOTTOM + 0.2], [XR - 0.25, top - 0.02]]   # tip, reversed below
blade = []
x_tip = X0 + PITCH * (len(BITTING) - 1) + 0.45
blade.append([x_tip, KEYWAY_BOTTOM + 0.04])
blade.append([x_tip + 0.12, KEYWAY_BOTTOM + 0.22])
blade.append([x_tip - 0.05, min(top, cut_y[-1] + 0.16)])
for i in reversed(range(len(BITTING))):
    x = X0 + PITCH * i
    flat = 0.07
    blade.append([x + flat, cut_y[i]])
    blade.append([x - flat, cut_y[i]])
    if i > 0:
        mid = x - PITCH / 2
        peak = min(top, max(cut_y[i], cut_y[i - 1]) + 0.20)
        blade.append([mid, peak])
blade.append([X0 - 0.32, top])
blade.append([XL - 0.9, top])
blade.append([XL - 0.9, KEYWAY_BOTTOM + 0.04])
N.append({"type": "mesh", "name": "blade", "translation": [0, 0, 0.19],
          "geometry": {"std": "extrude", "outline": [[round(a, 4), round(b, 4)] for a, b in blade], "depth": 0.16, "bevel_depth": 0.01},
          "materials": ["key"]})
N.append({"type": "mesh", "name": "bow", "translation": [XL - 1.75, (top + KEYWAY_BOTTOM) / 2, 0.19], "rotation": {"euler": [1.5708, 0, 0]},
          "geometry": {"std": "torus", "radius": 0.72, "tube": 0.2, "segments": 48, "rings": 12}, "scale": [1, 0.4, 1], "materials": ["key"]})
N += [
    {"type": "light", "shape": "rectangle", "translation": [-3.0, 5.0, 6.0], "target": [1.2, 0.8, 0], "energy": 2600, "size": 5.0, "color": [1.0, 0.95, 0.88]},
    {"type": "light", "shape": "rectangle", "translation": [6.0, 1.0, 4.0], "target": [1.2, 0.8, 0], "energy": 600, "size": 4.0, "color": [0.7, 0.8, 1.0]},
    {"type": "camera", "translation": [0.9, 0.9, 11.0], "target": [0.9, 0.75, 0.0]},
]
json.dump({
    "$schema": "https://raw.githubusercontent.com/voyager-so/vork/v0.6.1/packages/ts/scene.schema.json",
    "version": 2, "materials": M, "nodes": N,
    "render": {"resolution": [1500, 1000], "duration": 1},
}, sys.stdout, separators=(",", ":"))
