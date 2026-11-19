"""A walker's map of the country between Salt Creek and Station 9 (station world). 2D canvas.
Geography follows story/world.md: Salt Creek on the warm side, the ridge between, the freeze
boundary about thirty paces south of the ridge, Station 9 inside the freeze.

Usage: python make_scene.py > scene.json
"""
import json, math, random, sys

W, H = 1200, 1500
rng = random.Random(30)
FONT, ITAL, BOLD = "C:/Windows/Fonts/georgia.ttf", "C:/Windows/Fonts/georgiai.ttf", "C:/Windows/Fonts/georgiab.ttf"
INK, SOFT, COLD, WARM = "#2b2620", "#7a7064", "#5d7f96", "#a5562c"
nodes = []


def path(name, d, **kw):
    n = {"type": "path", "name": name, "d": d, "fill": "none"}
    n.update(kw)
    nodes.append(n)


def text(name, x, y, content, size=24, fill=INK, font=FONT, **kw):
    n = {"type": "text", "name": name, "translation": [x, y, 0], "content": content, "font": font, "font-size": size, "fill": fill}
    n.update(kw)
    nodes.append(n)


def wobble(points, amp):
    return [(x + rng.uniform(-amp, amp), y + rng.uniform(-amp, amp)) for x, y in points]


def poly(points):
    return "M" + " L".join(f"{x:.1f} {y:.1f}" for x, y in points)


path("paper", f"M0 0H{W}V{H}H0Z", fill="#e6dcc4")
path("border", f"M40 40H{W-40}V{H-40}H40Z", stroke=INK, **{"stroke-width": 2})

# The freeze: everything south of the boundary line, tinted. Boundary is nearly straight ("too clean").
BY = 860
path("frozen", f"M42 {BY}H{W-42}V{H-42}H42Z", fill="#cfd8d6")
path("boundary", f"M42 {BY}H{W-42}", stroke=COLD, **{"stroke-width": 3, "stroke-dasharray": [14, 8]})
text("blabel", 760, BY + 12, "the boundary  (too clean)", 22, COLD, ITAL)
text("frozenlabel", 120, 1130, "F R O Z E N", 44, "#9fb0b1", BOLD)
text("warmlabel", 120, 300, "W A R M", 44, "#cdbf9f", BOLD)

# The ridge: a wobbling line with hachures on its south face, about 30 paces north of the boundary.
RY = 790
ridge = wobble([(60 + i * 18, RY + 14 * math.sin(i / 6.0)) for i in range(61)], 3)
path("ridge", poly(ridge), stroke=INK, **{"stroke-width": 2.5})
for i, (x, y) in enumerate(ridge[::2]):
    path(f"h{i}", f"M{x:.1f} {y+4:.1f} l{rng.uniform(-3,3):.1f} {rng.uniform(14,26):.1f}", stroke=SOFT, **{"stroke-width": 1.2})
text("ridgelabel", 150, RY - 52, "the ridge", 28, INK, ITAL)
text("paces", 880, RY + 30, "thirty paces", 18, SOFT, ITAL)

# Salt Creek: a wandering stream across the warm side, and the relay beside it.
creek = wobble([(60 + i * 20, 430 + 40 * math.sin(i / 5.0) + 0.9 * i) for i in range(55)], 4)
path("creek", poly(creek), stroke="#6f8f9c", **{"stroke-width": 4})
path("creek2", poly([(x, y + 7) for x, y in creek]), stroke="#d9d3c0", **{"stroke-width": 2})
SX, SY = 700, 400
path("relay", f"M{SX-11} {SY-11}h22v22h-22Z", fill=INK)
path("mast", f"M{SX+24} {SY+11}v-46 M{SX+16} {SY-24}h16 M{SX+18} {SY-33}h12", stroke=INK, **{"stroke-width": 2})
text("saltlabel", SX + 52, SY - 30, "Salt Creek relay", 28, INK, BOLD)
text("saltnote", SX + 52, SY + 6, "receiver. one lamp.", 20, SOFT, ITAL)

# The cairn, on the last warm ground, and Station 9 two cycles' walk into the freeze.
CX, CY = 470, BY - 16
path("cairn", f"M{CX-10} {CY+8}h20 M{CX-7} {CY+2}h14 M{CX-4} {CY-4}h8", stroke=WARM, **{"stroke-width": 4})
text("cairnlabel", CX + 22, CY - 34, "cairn", 22, WARM, ITAL)
NX, NY = 380, 1290
path("st9", f"M{NX-14} {NY}a14 14 0 1 0 28 0a14 14 0 1 0 -28 0Z", stroke=INK, **{"stroke-width": 2.5})
path("st9dot", f"M{NX-4} {NY}a4 4 0 1 0 8 0a4 4 0 1 0 -8 0Z", fill=INK)
text("st9label", NX + 34, NY - 78, "Station 9", 28, INK, BOLD)
text("st9note", NX + 34, NY - 44, "quiet. lamps dark.", 20, SOFT, ITAL)
# lines under the glass, radiating from the station
for k in range(7):
    a = k * 2 * math.pi / 7 + 0.3
    pts, x, y = [(NX + 16 * math.cos(a), NY + 16 * math.sin(a))], NX + 16 * math.cos(a), NY + 16 * math.sin(a)
    for _ in range(rng.randint(5, 9)):
        a += rng.uniform(-0.4, 0.4)
        x, y = x + 26 * math.cos(a), y + 26 * math.sin(a)
        if y < BY + 20 or y > H - 70 or x < 70 or x > W - 70:
            break
        pts.append((x, y))
    if len(pts) > 1:
        path(f"vein{k}", poly(pts), stroke=WARM, **{"stroke-width": 1.6})

# The walker's route: relay -> over the ridge -> cairn -> Station 9.
route = wobble([(SX, SY + 20), (640, 560), (560, 700), (500, RY - 6), (CX + 4, CY + 14), (440, 1000), (405, 1150), (NX + 6, NY - 22)], 3)
path("route", poly(route), stroke=INK, **{"stroke-width": 1.8, "stroke-dasharray": [3, 9], "stroke-linecap": "round"})
text("routelabel", 455, 1040, "two cycles", 18, SOFT, ITAL)

# Furniture: title, north, scale.
text("title", 80, 78, "between Salt Creek and Station 9", 34, INK, BOLD)
text("subtitle", 80, 124, "as walked. not to scale past the cairn.", 20, SOFT, ITAL)
path("north", f"M{W-110} 250v-90 M{W-122} 184l12 -24l12 24", stroke=INK, **{"stroke-width": 2})
text("n", W - 119, 260, "N", 22, INK, BOLD)

json.dump({
    "$schema": "https://raw.githubusercontent.com/voyager-so/vork/v0.6.1/packages/ts/scene.schema.json",
    "version": 2, "composition": {"space": "2d"}, "nodes": nodes,
    "render": {"resolution": [W, H], "duration": 1},
}, sys.stdout, separators=(",", ":"))
