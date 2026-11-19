"""The receiver needle leaving band 9 and settling below the scale. 2D canvas animation.
Builds on make_scene.py: same dial, but the needle is its own node with a rotation track, and the
pencil marks fade in when the story reaches them.

Usage: python make_movie.py > movie.json   (then vork render movie.json --out needle.mp4)
"""
import json, math, subprocess, sys

SECONDS, FPS = 10, 30
scene = json.loads(subprocess.run([sys.executable, "make_scene.py"], capture_output=True, text=True, check=True).stdout)
CX, CY, R = 650, 720, 470
A1, A12 = 205.0, -25.0
band = lambda b: A1 + (A12 - A1) * (b - 1) / 11.0
A9, AS = band(9), A1 + 26

nodes = [n for n in scene["nodes"] if n["name"] not in ("needle", "hub", "hub2")]
by = {n["name"]: n for n in scene["nodes"]}
# needle drawn pointing along +x from the hub, placed at the hub, turned by a track
tip, half, tail = R - 30, 14, 90
nodes.append({"type": "path", "name": "needle", "translation": [CX, CY, 0],
              "d": f"M0 {-half}L{tip} 0L0 {half}L{-tail} 0Z", "fill": "#161412",
              "filter": "drop-shadow(4px 6px 5px rgba(0,0,0,0.35))"})
nodes += [by["hub"], by["hub2"]]


def smooth(u):
    u = max(0.0, min(1.0, u))
    return u * u * (3 - 2 * u)


frames = SECONDS * FPS
rot = []
for f in range(frames + 1):
    t = f / FPS
    if t < 1.5:                       # steady on 9
        a = A9 + 0.25 * math.sin(t * 9)
    elif t < 3.0:                     # falters
        u = (t - 1.5) / 1.5
        a = A9 + (0.25 + 2.2 * u) * math.sin(t * 9 + 4 * u)
    elif t < 7.5:                     # goes down the scale and off the end
        u = smooth((t - 3.0) / 4.5)
        a = A9 + (AS - A9) * u + 1.2 * (1 - u) * math.sin(t * 7)
    else:                             # rests on nothing printed
        a = AS + 0.5 * math.exp(-(t - 7.5) * 2.5) * math.sin((t - 7.5) * 14)
    phi = -math.radians(a)            # canvas y runs down
    rot.append([f"{f}/{FPS}", [0, 0, round(math.sin(phi / 2), 6), round(math.cos(phi / 2), 6)]])
tracks = [{"path": ["nodes", "needle", "rotation"], "keys": rot}]
for name, t0 in (("x9", 3.4), ("scratch", 8.2), ("nine", 8.8)):
    tracks.append({"path": ["nodes", name, "opacity"], "keys": [[0, 0], [t0, 0], [t0 + 0.35, 1], [SECONDS, 1]]})
scene["nodes"] = nodes
scene["animation"] = [{"name": "needle", "tracks": tracks}]
scene["render"] = {"resolution": [900, 900], "duration": SECONDS, "fps": FPS}
scene["composition"] = {"space": "2d", "viewBox": [0, 0, 1300, 1300]}
json.dump(scene, sys.stdout, separators=(",", ":"))
