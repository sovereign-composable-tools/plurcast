#!/usr/bin/env bash
#
# generate.sh — create a unique visual fingerprint from a nostr pubkey
#
# Takes a 64-char hex pubkey and deterministically generates an image
# using the bytes as parameters for multi-scale interference patterns.
#
# Usage:
#   ./generate.sh <hex_pubkey> [output.png]
#
# The same pubkey always produces the same image.

set -euo pipefail

PUBKEY="${1:-}"
OUTPUT="${2:-/tmp/pubkey_art.png}"

if [[ -z "$PUBKEY" || ${#PUBKEY} -lt 64 ]]; then
    echo "usage: $0 <64-char-hex-pubkey> [output.png]"
    exit 1
fi

FILTER=$(node -e "
const pk = '${PUBKEY}';
const b = (i) => parseInt(pk.substr(i*2, 2), 16);
const n = (i, lo, hi) => lo + (b(i) / 255) * (hi - lo);
const f = (v, d) => v.toFixed(d || 4);

// Center
const cx = Math.floor(384 + b(0) / 255 * 256);
const cy = Math.floor(384 + b(1) / 255 * 256);

// Structure
const sym = 3 + (b(2) % 8);          // 3-10 fold symmetry
const twist = n(3, 0, 0.006);        // spiral twist

// Frequencies — three scales
const f1 = n(4, 0.02, 0.06);         // large shapes
const f2 = n(5, 0.06, 0.15);         // medium detail
const f3 = n(6, 0.15, 0.35);         // fine texture

// Phases
const p1 = n(7, 0, 6.28);
const p2 = n(8, 0, 6.28);

// Radial
const rf = n(9, 0.015, 0.05);

// Color
const hue1 = n(10, 0, 6.28);
const hue2 = n(11, 0, 6.28);
const sat = n(12, 30, 55);
const colorSpeed = n(13, 0.003, 0.015);

// Mix weights
const w2 = n(14, 0.2, 0.6);          // medium layer weight
const w3 = n(15, 0.05, 0.2);         // fine layer weight
const wr = n(16, 0.15, 0.4);         // radial weight

const r = \`sqrt((X-\${cx})*(X-\${cx})+(Y-\${cy})*(Y-\${cy}))\`;
const theta = \`atan2(Y-\${cy},X-\${cx})\`;

// Layer 1: n-fold symmetric interference (normalized to ±1)
let waves1 = [];
for (let k = 0; k < sym; k++) {
    const a = 6.28318 * k / sym;
    const ca = f(Math.cos(a), 5);
    const sa = f(Math.sin(a), 5);
    waves1.push(\`sin((X-\${cx})*\${f(f1)}*\${ca}+(Y-\${cy})*\${f(f1)}*\${sa}+\${f(p1)}+\${f(twist)}*\${r})\`);
}
const L1 = \`(\${waves1.join('+')})/\${f(sym)}\`;

// Layer 2: different symmetry at medium frequency
const sym2 = 2 + (b(17) % 5);
let waves2 = [];
for (let k = 0; k < sym2; k++) {
    const a = 6.28318 * k / sym2 + n(18, 0, 3.14);
    const ca = f(Math.cos(a), 5);
    const sa = f(Math.sin(a), 5);
    waves2.push(\`sin((X-\${cx})*\${f(f2)}*\${ca}+(Y-\${cy})*\${f(f2)}*\${sa}+\${f(p2)})\`);
}
const L2 = \`(\${waves2.join('+')})/\${f(sym2)}\`;

// Layer 3: fine texture — just 2 waves for subtle grain
const a3a = n(19, 0, 3.14);
const a3b = a3a + n(20, 0.5, 2.5);
const L3 = \`(sin((X-\${cx})*\${f(f3)}*\${f(Math.cos(a3a),5)}+(Y-\${cy})*\${f(f3)}*\${f(Math.sin(a3a),5)})+sin((X-\${cx})*\${f(f3)}*\${f(Math.cos(a3b),5)}+(Y-\${cy})*\${f(f3)}*\${f(Math.sin(a3b),5)}))/2\`;

// Radial rings
const LR = \`sin(\${r}*\${f(rf)})\`;

// Combine: weighted sum, all layers ±1, result ±1
const totalW = f(1 + w2 + w3 + wr);
const field = \`(\${L1}+\${f(w2)}*\${L2}+\${f(w3)}*\${L3}+\${f(wr)}*\${LR})/\${totalW}\`;

// Map to luminance: ±1 → 0-255 with soft S-curve via sin mapping
// sin(field * pi/2) gives S-curve contrast boost
const lumRaw = \`128+127*sin((\${field})*1.5708)\`;
const lumFinal = \`clip(\${lumRaw},0,255)\`;

// Color: hue rotates with angle and radius
const cAngle = \`\${f(hue1)}+\${theta}*0.5+\${r}*\${f(colorSpeed)}\`;
const cbVal = \`clip(128+\${Math.floor(sat)}*sin(\${cAngle}),16,240)\`;
const crVal = \`clip(128+\${Math.floor(sat)}*cos(\${cAngle}+\${f(hue2-hue1)}),16,240)\`;

console.log(\`lum='\${lumFinal}':cb='\${cbVal}':cr='\${crVal}'\`);
")

ffmpeg -f lavfi -i "color=black:s=1024x1024:d=1,geq=${FILTER}" \
    -frames:v 1 -update 1 -y "$OUTPUT" 2>/dev/null

echo "$OUTPUT"
