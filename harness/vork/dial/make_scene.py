"""The receiver dial at Salt Creek (station world). 2D canvas.
A printed scale of bands 1 to 12; band 9 is where the needle used to sit. Below the end of the
printed scale someone has scratched a new mark by hand, and the needle rests on it.

Usage: python make_scene.py > scene.json
"""
import json, math, sys

W = H = 1300
CX, CY, R = 650, 720, 470
SERIF, SERIFB, HAND = "C:/Windows/Fonts/georgia.ttf", "C:/Windows/Fonts/georgiab.ttf", "C:/Windows/Fonts/segoepr.ttf"
INK, RED, BRASS = "#231f1a", "#a8321f", "#b08a3c"
nodes = []


def path(name, d, **kw):
    n = {"type": "path", "name": name, "d": d, "fill": "none"}
    n.update(kw)
    nodes.append(n)


def text(name, x, y, content, size, fill=INK, font=SERIF):
    nodes.append({"type": "text", "name": name, "translation": [x, y, 0], "content": content,
                  "font": font, "font-size": size, "fill": fill})


def pt(angle_deg, r):
    a = math.radians(angle_deg)
    return CX + r * math.cos(a), CY - r * math.sin(a)


def circle(name, cx, cy, r, **kw):
    path(name, f"M{cx-r} {cy}a{r} {r} 0 1 0 {2*r} 0a{r} {r} 0 1 0 {-2*r} 0Z", **kw)


path("panel", f"M0 0H{W}V{H}H0Z", fill="#1b1a18")
circle("bezel", CX, CY, R + 70, fill=BRASS)
circle("bezel2", CX, CY, R + 52, fill="#6e5622")
circle("face", CX, CY, R + 40, fill="#ece4cf")

# Printed scale: band 1 at 150 degrees round to band 12 at 30 degrees, over the top.
A1, A12 = 205.0, -25.0


def band_angle(b):
    return A1 + (A12 - A1) * (b - 1) / 11.0


sx, sy = pt(A1, R)
ex, ey = pt(A12, R)
path("arc", f"M{sx:.1f} {sy:.1f}A{R} {R} 0 1 1 {ex:.1f} {ey:.1f}", stroke=INK, **{"stroke-width": 4})
for b in range(1, 13):
    a = band_angle(b)
    x1, y1 = pt(a, R); x2, y2 = pt(a, R - 46)
    path(f"tick{b}", f"M{x1:.1f} {y1:.1f}L{x2:.1f} {y2:.1f}", stroke=INK, **{"stroke-width": 5})
    tx, ty = pt(a, R - 92)
    text(f"num{b}", tx - (20 if b >= 10 else 11), ty - 24, str(b), 40, INK, SERIFB)
    if b < 12:
        for j in range(1, 5):
            aa = a + (band_angle(b + 1) - a) * j / 5
            x1, y1 = pt(aa, R); x2, y2 = pt(aa, R - 22)
            path(f"m{b}_{j}", f"M{x1:.1f} {y1:.1f}L{x2:.1f} {y2:.1f}", stroke=INK, **{"stroke-width": 2})
text("legend", CX - 62, CY - 250, "BAND", 44, INK, SERIFB)
text("maker", CX - 150, CY + 190, "salt creek  \u00b7  receiver no. 2", 26, "#6d6558")

# Where band 9 is printed, a small pencilled cross: it is not there any more.
a9 = band_angle(9)
x9, y9 = pt(a9, R + 20)
path("x9", f"M{x9-9:.1f} {y9-9:.1f}l18 18M{x9+9:.1f} {y9-9:.1f}l-18 18", stroke=RED, **{"stroke-width": 3})

# The hand-scratched mark, past band 1 and off the printed scale, and the needle on it.
AS = A1 + 26
x1, y1 = pt(AS, R + 6); x2, y2 = pt(AS + 1.2, R - 58)
path("scratch", f"M{x1:.1f} {y1:.1f}L{x2:.1f} {y2:.1f}", stroke=RED, **{"stroke-width": 4})
x3, y3 = pt(AS - 0.8, R - 20); x4, y4 = pt(AS + 1.6, R - 40)
tx, ty = pt(AS + 2, R - 150)
text("nine", tx - 10, ty - 30, "9", 54, RED, HAND)
nx, ny = pt(AS, R - 30)
bx1, by1 = pt(AS + 90, 14); bx2, by2 = pt(AS - 90, 14)
tailx, taily = pt(AS + 180, 90)
path("needle", f"M{bx1:.1f} {by1:.1f}L{nx:.1f} {ny:.1f}L{bx2:.1f} {by2:.1f}L{tailx:.1f} {taily:.1f}Z", fill="#161412",
     filter="drop-shadow(4px 6px 5px rgba(0,0,0,0.35))")
circle("hub", CX, CY, 34, fill=BRASS)
circle("hub2", CX, CY, 10, fill="#3a2e12")

json.dump({
    "$schema": "https://raw.githubusercontent.com/voyager-so/vork/v0.6.1/packages/ts/scene.schema.json",
    "version": 2, "composition": {"space": "2d"}, "nodes": nodes,
    "render": {"resolution": [W, H], "duration": 1},
}, sys.stdout, separators=(",", ":"))
