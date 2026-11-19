"""The keeper's receiver log at Salt Creek (station world). A 2D canvas piece: typed rows on a paper sheet.
Fourteen silent cycles (212 to 225), as the keeper counts them; Kef arrives on the last and says eleven.

Usage: python make_scene.py > scene.json
"""
import json, sys

W, H = 1200, 1500
FONT = "C:/Windows/Fonts/cour.ttf"
BOLD = "C:/Windows/Fonts/courbd.ttf"
INK, FAINT, RED = "#1d1b18", "#6b655c", "#8a2a1c"
P3 = "\u2022 \u2022 \u2022"
G = "   "
rows = [
    (207, f"hum{G}{P3}{G}{P3}", ""),
    (208, f"hum{G}{P3}{G}{P3}", ""),
    (209, f"hum{G}{P3}{G}{P3}", "faint"),
    (210, f"hum{G}{P3}{G}• •", "one short"),
    (211, f"hum{G}• •", "second group missing"),
    (212, "\u2014", "nothing on the band"),
    (213, "\u2014", ""),
    (214, "\u2014", "checked the mast. mast is fine"),
    (215, "\u2014", ""),
    (216, "\u2014", ""),
    (217, "\u2014", "walkers say three. it is six"),
    (218, "\u2014", ""),
    (219, "\u2014", ""),
    (220, "\u2014", "lamp oil"),
    (221, "\u2014", ""),
    (222, "\u2014", ""),
    (223, "\u2014", ""),
    (224, "\u2014", ""),
    (225, "\u2014", "a walker in. she says eleven"),
]
nodes = [
    {"type": "path", "name": "desk", "d": f"M0 0H{W}V{H}H0Z", "fill": "#2a2622"},
    {"type": "path", "name": "sheet", "d": f"M70 60H{W-70}V{H-60}H70Z", "fill": "#e9e2d0",
     "filter": "drop-shadow(0px 6px 14px rgba(0,0,0,0.5))"},
]


def text(name, x, y, content, size=26, fill=INK, font=FONT):
    nodes.append({"type": "text", "name": name, "translation": [x, y, 0], "content": content,
                  "font": font, "font-size": size, "fill": fill})


text("head", 130, 120, "SALT CREEK RELAY", 40, INK, BOLD)
text("sub", 130, 176, "receiver log  \u00b7  band 9", 26, FAINT)
text("c1", 130, 250, "cycle", 22, FAINT)
text("c2", 290, 250, "received", 22, FAINT)
text("c3", 670, 250, "note", 22, FAINT)
nodes.append({"type": "path", "name": "rule", "d": f"M130 286H{W-130}", "fill": "none", "stroke": FAINT, "stroke-width": 1.5})
y = 310
for i, (cyc, rec, note) in enumerate(rows):
    text(f"n{i}", 130, y, str(cyc))
    text(f"r{i}", 290, y, rec)
    if note:
        text(f"t{i}", 670, y, note, 22)
    y += 52
nodes.append({"type": "path", "name": "rule2", "d": f"M130 {y+4}H{W-130}", "fill": "none", "stroke": FAINT, "stroke-width": 1.5})
text("last1", 130, y + 30, "226", 26, RED)
text("last2", 290, y + 30, "retuned low. below the band.", 26, RED)
text("last3", 290, y + 82, f"hum{G}{P3}{G}{P3}", 26, RED)

json.dump({
    "$schema": "https://raw.githubusercontent.com/voyager-so/vork/v0.6.1/packages/ts/scene.schema.json",
    "version": 2,
    "composition": {"space": "2d"},
    "nodes": nodes,
    "render": {"resolution": [W, H], "duration": 1},
}, sys.stdout, separators=(",", ":"))
