"""The Rybin counterexample to Goemans' cost conjecture for unsplittable flow, on an index card.
Graph, capacities, costs, demands and the split flow are taken from the Isabelle AFP entry
(Dinitz_Garg_Goemans_Counterexample, theory file read 2026-11-19) and re-checked by brute force
in this script before anything is drawn.

Usage: python make_scene.py > scene.json
"""
import json, math, sys
from itertools import product

CAP = dict(S_T1=10, S_T2=6, S_U=24, U_V=14, U_T3=10, V_T1=5, V_W=9, W_T2=4, W_T3=5)
COST = dict(S_T1=2, S_T2=3, S_U=0, U_V=0, U_T3=2, V_T1=0, V_W=0, W_T2=0, W_T3=0)
DEM = {1: 15, 2: 10, 3: 15}
ROUTES = {1: {"P": ["S_T1"], "F": ["S_U", "U_V", "V_T1"]},
          2: {"P": ["S_T2"], "F": ["S_U", "U_V", "V_W", "W_T2"]},
          3: {"P": ["S_U", "U_T3"], "F": ["S_U", "U_V", "V_W", "W_T3"]}}
SPLIT = {(1, "P"): 10, (1, "F"): 5, (2, "P"): 6, (2, "F"): 4, (3, "P"): 10, (3, "F"): 5}

load = {a: 0 for a in CAP}
for (k, r), f in SPLIT.items():
    for a in ROUTES[k][r]:
        load[a] += f
assert load == CAP and sum(COST[a] * load[a] for a in CAP) == 58
feasible = []
for ch in product("PF", repeat=3):
    l = {a: 0 for a in CAP}
    for k, r in zip((1, 2, 3), ch):
        for a in ROUTES[k][r]:
            l[a] += DEM[k]
    if all(l[a] <= CAP[a] + 15 for a in CAP):
        feasible.append(sum(COST[a] * l[a] for a in CAP))
assert min(feasible) == 60

W, H = 1500, 1000
HAND, HANDB = "C:/Windows/Fonts/segoepr.ttf", "C:/Windows/Fonts/segoeprb.ttf"
INK, RED, BLUE = "#1f2a44", "#b03a2e", "#9db7d6"
POS = dict(S=(190, 470), U=(520, 470), V=(850, 470), W=(1180, 470), T1=(685, 200), T3=(850, 680), T2=(1390, 470))
NAMES = dict(S="s", U="u", V="v", W="w", T1="t1", T2="t2", T3="t3")
nodes = []


def path(name, d, **kw):
    n = {"type": "path", "name": name, "d": d, "fill": "none"}
    n.update(kw)
    nodes.append(n)


def text(name, x, y, content, size=30, fill=INK, font=HAND):
    nodes.append({"type": "text", "name": name, "translation": [x, y, 0], "content": content,
                  "font": font, "font-size": size, "fill": fill})


path("desk", f"M0 0H{W}V{H}H0Z", fill="#3a332b")
path("card", f"M60 50H{W-60}V{H-50}H60Z", fill="#f6f1e3", filter="drop-shadow(0px 8px 16px rgba(0,0,0,0.45))")
path("redline", f"M60 150H{W-60}", stroke="#d98a8a", **{"stroke-width": 2})
for i in range(12):
    y = 215 + i * 62
    path(f"rule{i}", f"M60 {y}H{W-60}", stroke=BLUE, **{"stroke-width": 1.2})
text("title", 100, 74, "unsplittable flow: the cheap answer needs splitting", 34, INK, HANDB)

R = 30


def arrow(name, a, b, label, paid, ctrl=None, lpos=None):
    (x1, y1), (x2, y2) = POS[a], POS[b]
    col = RED if paid else INK
    if ctrl:
        cx, cy = ctrl
        d1 = math.atan2(cy - y1, cx - x1); d2 = math.atan2(y2 - cy, x2 - cx)
        sx, sy = x1 + R * math.cos(d1), y1 + R * math.sin(d1)
        ex, ey = x2 - (R + 4) * math.cos(d2), y2 - (R + 4) * math.sin(d2)
        path(name, f"M{sx:.1f} {sy:.1f}Q{cx} {cy} {ex:.1f} {ey:.1f}", stroke=col, **{"stroke-width": 3})
        ang = d2
        mx, my = 0.25 * sx + 0.5 * cx + 0.25 * ex, 0.25 * sy + 0.5 * cy + 0.25 * ey
    else:
        ang = math.atan2(y2 - y1, x2 - x1)
        sx, sy = x1 + R * math.cos(ang), y1 + R * math.sin(ang)
        ex, ey = x2 - (R + 4) * math.cos(ang), y2 - (R + 4) * math.sin(ang)
        path(name, f"M{sx:.1f} {sy:.1f}L{ex:.1f} {ey:.1f}", stroke=col, **{"stroke-width": 3})
        mx, my = (sx + ex) / 2, (sy + ey) / 2
    h = 16
    p1 = (ex - h * math.cos(ang - 0.4), ey - h * math.sin(ang - 0.4))
    p2 = (ex - h * math.cos(ang + 0.4), ey - h * math.sin(ang + 0.4))
    path(name + "_h", f"M{ex:.1f} {ey:.1f}L{p1[0]:.1f} {p1[1]:.1f}L{p2[0]:.1f} {p2[1]:.1f}Z", fill=col)
    lx, ly = lpos if lpos else (mx, my)
    text(name + "_l", lx, ly, label, 26, col)


def lab(a):
    return f"{CAP[a]}" + (f"  @{COST[a]}" if COST[a] else "")


arrow("S_U", "S", "U", lab("S_U"), False, lpos=(330, 418))
arrow("U_V", "U", "V", lab("U_V"), False, lpos=(665, 418))
arrow("V_W", "V", "W", lab("V_W"), False, lpos=(1000, 418))
arrow("W_T2", "W", "T2", lab("W_T2"), False, lpos=(1272, 418))
arrow("S_T1", "S", "T1", lab("S_T1"), True, lpos=(330, 270))
arrow("V_T1", "V", "T1", lab("V_T1"), False, lpos=(800, 290))
arrow("U_T3", "U", "T3", lab("U_T3"), True, lpos=(545, 580))
arrow("W_T3", "W", "T3", lab("W_T3"), False, lpos=(1050, 570))
arrow("S_T2", "S", "T2", lab("S_T2"), True, ctrl=(790, 1140), lpos=(1165, 690))

for k, (x, y) in POS.items():
    term = k.startswith("T")
    path("n" + k, f"M{x-R} {y}a{R} {R} 0 1 0 {2*R} 0a{R} {R} 0 1 0 {-2*R} 0Z", fill="#f6f1e3", stroke=INK,
         **{"stroke-width": 4 if term else 3})
    text("t" + k, x - (15 if term else 9), y - 24, NAMES[k], 28, INK, HANDB)
text("d1", POS["T1"][0] + 44, POS["T1"][1] - 40, "wants 15", 24)
text("d2", POS["T2"][0] - 118, POS["T2"][1] + 44, "wants 10", 24)
text("d3", POS["T3"][0] + 44, POS["T3"][1] - 6, "wants 15", 24)

text("key", 100, 838, "numbers are capacity.  red arcs charge @ per unit.  overload allowed: 15", 24)
text("ans1", 100, 884, "split every delivery two ways:  58.", 28, INK, HANDB)
text("ans2", 800, 884, "send each one whole:  60 at best.", 28, RED, HANDB)

json.dump({
    "$schema": "https://raw.githubusercontent.com/voyager-so/vork/v0.6.1/packages/ts/scene.schema.json",
    "version": 2, "composition": {"space": "2d"}, "nodes": nodes,
    "render": {"resolution": [W, H], "duration": 1},
}, sys.stdout, separators=(",", ":"))
