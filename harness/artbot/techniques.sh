#!/usr/bin/env bash
# techniques.sh — reusable ffmpeg art generation techniques
# Usage: ./techniques.sh <technique> <output.png> [seed]
#
# Techniques:
#   sierpinski-zoom    — variable-resolution Sierpinski, coarse center, fine edges
#   threshold-cells    — organic cell/vein structures from sin-of-sin interference
#   edge-of-chaos      — order-to-chaos transition (variable threshold)
#   node-halos         — glowing network nodes with texture overlay
#   quasicrystal       — 5-fold symmetric interference pattern
#   moire-rings        — concentric rings dissolving into wave texture
#   mandelbrot-deep    — deep zoom with period coloring
#   bitwise-triptych   — AND, MUL, SIN side by side
#   symmetry-spiral    — competing symmetries create spiral arm structure
#   flow-ribbons       — multi-scale flow lines with spiral distortion, warm-to-cool radial
#
# Seed (optional): hex string used to vary parameters deterministically.
# If omitted, uses current timestamp.

set -euo pipefail

TECHNIQUE="${1:?Usage: techniques.sh <technique> <output.png> [seed]}"
OUTPUT="${2:?Usage: techniques.sh <technique> <output.png> [seed]}"
SEED="${3:-$(date +%s)}"
SIZE="1024x1024"

# Extract variation parameters from seed (first 8 hex chars → integers)
SEED_HEX=$(echo -n "$SEED" | md5sum | cut -c1-16)
S1=$(printf '%d' "0x${SEED_HEX:0:2}")  # 0-255
S2=$(printf '%d' "0x${SEED_HEX:2:2}")
S3=$(printf '%d' "0x${SEED_HEX:4:2}")
S4=$(printf '%d' "0x${SEED_HEX:6:2}")

# Float helper (no bc on Windows)
fmath() { awk "BEGIN {printf \"%.6f\", $1}"; }

case "$TECHNIQUE" in

sierpinski-zoom)
    # Variable-resolution Sierpinski: coarse blocks at center, fine at edges
    # Warm amber core, deep purple edges
    ffmpeg -f lavfi -i "color=c=black:s=$SIZE:d=1" -vf "format=gbrp,geq=\
r='clip( \
  ( 255 - 200*min(1, bitand(floor(X/max(1,16-floor(hypot(X-512,Y-512)/40))), floor(Y/max(1,16-floor(hypot(X-512,Y-512)/40))))/max(1,16-floor(hypot(X-512,Y-512)/40)) ) ) \
  * (0.3 + 0.7*exp(-hypot(X-512,Y-512)/400)) \
, 0, 255)'\
:g='clip( \
  ( 200 - 180*min(1, bitand(floor(X/max(1,16-floor(hypot(X-512,Y-512)/40))), floor(Y/max(1,16-floor(hypot(X-512,Y-512)/40))))/max(1,16-floor(hypot(X-512,Y-512)/40)) ) ) \
  * (0.15 + 0.5*exp(-hypot(X-512,Y-512)/350)) \
, 0, 255)'\
:b='clip( \
  ( 120 - 80*min(1, bitand(floor(X/max(1,16-floor(hypot(X-512,Y-512)/40))), floor(Y/max(1,16-floor(hypot(X-512,Y-512)/40))))/max(1,16-floor(hypot(X-512,Y-512)/40)) ) ) \
  * (0.1 + 0.3*exp(-hypot(X-512,Y-512)/500)) \
  + 30*min(1,hypot(X-512,Y-512)/400) \
, 0, 255)'" \
    -frames:v 1 -update 1 "$OUTPUT" 2>/dev/null
    ;;

threshold-cells)
    # Organic cell/vein structures — sin-of-sin interference, hard threshold
    # Dark green on black, like fluorescent microscopy
    F1=$(fmath "0.02 + $S1 / 12800")
    F2=$(fmath "0.03 + $S2 / 12800")
    F3=$(fmath "0.015 + $S3 / 25600")
    ffmpeg -f lavfi -i "color=c=black:s=$SIZE:d=1" -vf "format=gbrp,geq=\
r='clip(15*lt(abs(sin(X*$F1+sin(Y*$F2)*3)+sin(Y*$F3+sin(X*$F1)*2.5)+sin((X+Y)*0.01+sin((X-Y)*0.02)*2)),0.5), 0, 255)'\
:g='clip(80*lt(abs(sin(X*$F1+sin(Y*$F2)*3)+sin(Y*$F3+sin(X*$F1)*2.5)+sin((X+Y)*0.01+sin((X-Y)*0.02)*2)),0.5)+20*lt(abs(sin(X*$F1+sin(Y*$F2)*3)+sin(Y*$F3+sin(X*$F1)*2.5)+sin((X+Y)*0.01+sin((X-Y)*0.02)*2)),0.3), 0, 255)'\
:b='clip(10*lt(abs(sin(X*$F1+sin(Y*$F2)*3)+sin(Y*$F3+sin(X*$F1)*2.5)+sin((X+Y)*0.01+sin((X-Y)*0.02)*2)),0.5), 0, 255)'" \
    -frames:v 1 -update 1 "$OUTPUT" 2>/dev/null
    ;;

edge-of-chaos)
    # Variable threshold: left = ordered cells, right = fragmented chaos
    # Teal to green color shift
    ffmpeg -f lavfi -i "color=c=black:s=$SIZE:d=1" -vf "format=gbrp,geq=\
r='clip(15*lt(abs(sin(X*0.025+sin(Y*0.03)*3)+sin(Y*0.015+sin(X*0.025)*2.5)+sin((X+Y)*0.01+sin((X-Y)*0.02)*2)),0.15+0.7*X/1024), 0, 255)'\
:g='clip((50+30*X/1024)*lt(abs(sin(X*0.025+sin(Y*0.03)*3)+sin(Y*0.015+sin(X*0.025)*2.5)+sin((X+Y)*0.01+sin((X-Y)*0.02)*2)),0.15+0.7*X/1024), 0, 255)'\
:b='clip((40-20*X/1024)*lt(abs(sin(X*0.025+sin(Y*0.03)*3)+sin(Y*0.015+sin(X*0.025)*2.5)+sin((X+Y)*0.01+sin((X-Y)*0.02)*2)),0.15+0.7*X/1024), 0, 255)'" \
    -frames:v 1 -update 1 "$OUTPUT" 2>/dev/null
    ;;

node-halos)
    # Glowing network nodes with quadratic residue texture
    # Dark blue with warm node centers
    P1=$((7 + S1 % 20))
    P2=$((11 + S2 % 20))
    ffmpeg -f lavfi -i "color=c=black:s=$SIZE:d=1" -vf "format=gbrp,geq=\
r='clip( \
  60*exp(-hypot(X-200,Y-300)/80) + 50*exp(-hypot(X-700,Y-200)/100) + 40*exp(-hypot(X-500,Y-700)/90) + 30*exp(-hypot(X-300,Y-600)/70) + 45*exp(-hypot(X-800,Y-600)/85) \
  + 8*mod(floor(X/4)*floor(X/4)+floor(Y/4)*floor(Y/4),$P1)/$P1 \
, 0, 255)'\
:g='clip( \
  30*exp(-hypot(X-200,Y-300)/80) + 25*exp(-hypot(X-700,Y-200)/100) + 20*exp(-hypot(X-500,Y-700)/90) + 15*exp(-hypot(X-300,Y-600)/70) + 22*exp(-hypot(X-800,Y-600)/85) \
  + 4*mod(floor(X/4)*floor(X/4)+floor(Y/4)*floor(Y/4),$P2)/$P2 \
, 0, 255)'\
:b='clip( \
  15*exp(-hypot(X-200,Y-300)/120) + 12*exp(-hypot(X-700,Y-200)/150) + 10*exp(-hypot(X-500,Y-700)/130) + 8*exp(-hypot(X-300,Y-600)/100) + 11*exp(-hypot(X-800,Y-600)/125) \
  + 12*mod(floor(X/4)*floor(X/4)+floor(Y/4)*floor(Y/4),$P1)/$P1 \
  + 15 \
, 0, 255)'" \
    -frames:v 1 -update 1 "$OUTPUT" 2>/dev/null
    ;;

quasicrystal)
    # 5-fold symmetric interference — Penrose tiling-like
    # Cool blue-purple palette
    FREQ=$(fmath "0.04 + $S1 / 6400")
    ffmpeg -f lavfi -i "color=c=black:s=$SIZE:d=1" -vf "format=gbrp,geq=\
r='clip(128 + 100*(\
  cos(X*$FREQ*cos(0)+Y*$FREQ*sin(0)) \
  *cos(X*$FREQ*cos(1.2566)+Y*$FREQ*sin(1.2566)) \
  *cos(X*$FREQ*cos(2.5133)+Y*$FREQ*sin(2.5133)) \
  *cos(X*$FREQ*cos(3.7699)+Y*$FREQ*sin(3.7699)) \
  *cos(X*$FREQ*cos(5.0265)+Y*$FREQ*sin(5.0265)) \
)*8, 0, 255)'\
:g='clip(100 + 80*(\
  cos(X*$FREQ*cos(0)+Y*$FREQ*sin(0)) \
  *cos(X*$FREQ*cos(1.2566)+Y*$FREQ*sin(1.2566)) \
  *cos(X*$FREQ*cos(2.5133)+Y*$FREQ*sin(2.5133)) \
  *cos(X*$FREQ*cos(3.7699)+Y*$FREQ*sin(3.7699)) \
  *cos(X*$FREQ*cos(5.0265)+Y*$FREQ*sin(5.0265)) \
)*8, 0, 255)'\
:b='clip(140 + 115*(\
  cos(X*$FREQ*cos(0)+Y*$FREQ*sin(0)) \
  *cos(X*$FREQ*cos(1.2566)+Y*$FREQ*sin(1.2566)) \
  *cos(X*$FREQ*cos(2.5133)+Y*$FREQ*sin(2.5133)) \
  *cos(X*$FREQ*cos(3.7699)+Y*$FREQ*sin(3.7699)) \
  *cos(X*$FREQ*cos(5.0265)+Y*$FREQ*sin(5.0265)) \
)*8, 0, 255)'" \
    -frames:v 1 -update 1 "$OUTPUT" 2>/dev/null
    ;;

moire-rings)
    # Concentric rings fading into wave texture at edges
    # Cool blue palette
    ffmpeg -f lavfi -i "color=c=black:s=$SIZE:d=1" -vf "format=gbrp,geq=\
r='clip( \
  25*pow(sin(hypot(X-512,Y-512)/6),2)*exp(-hypot(X-512,Y-512)/400) \
  + 20*(sin(X*0.05+0.3)*sin(Y*0.04+0.7)*sin(X*0.03-Y*0.02))*min(1,hypot(X-512,Y-512)/250) \
, 0, 255)'\
:g='clip( \
  40*pow(sin(hypot(X-512,Y-512)/6),2)*exp(-hypot(X-512,Y-512)/350) \
  + 30*(sin(X*0.05+0.3)*sin(Y*0.04+0.7)*sin(X*0.03-Y*0.02))*min(1,hypot(X-512,Y-512)/200) \
, 0, 255)'\
:b='clip( \
  80*pow(sin(hypot(X-512,Y-512)/6),2)*exp(-hypot(X-512,Y-512)/300) \
  + 50*(sin(X*0.05+0.3)*sin(Y*0.04+0.7)*sin(X*0.03-Y*0.02))*min(1,hypot(X-512,Y-512)/180) \
  + 15 \
, 0, 255)'" \
    -frames:v 1 -update 1 "$OUTPUT" 2>/dev/null
    ;;

mandelbrot-deep)
    # Deep zoom with period coloring, dark palette
    # Seed varies the zoom location slightly
    SX=$(fmath "-0.745 + $S1 / 500000")
    SY=$(fmath "0.186 + $S2 / 500000")
    SCALE=$(fmath "0.0003 + $S3 / 2000000")
    ffmpeg -f lavfi -i "mandelbrot=s=$SIZE:start_x=$SX:start_y=$SY:start_scale=$SCALE:inner=period:maxiter=500" \
    -vf "eq=brightness=-0.4:contrast=2.0:gamma=0.3,hue=H=200:s=0.3,colorbalance=rs=-0.3:gs=-0.1:bs=0.2" \
    -frames:v 1 -update 1 "$OUTPUT" 2>/dev/null
    ;;

bitwise-triptych)
    # Three panels: AND (sierpinski), MUL (hyperbolic), SIN (lattice)
    W=341  # ~1024/3
    ffmpeg -f lavfi -i "color=c=black:s=$SIZE:d=1" -vf "format=gbrp,geq=\
r='clip(if(lt(X,$W), \
  (200-bitand(floor(X/2),floor(Y/2))*3)*sin(X*0.003+1.5)*sin(Y*0.002+0.5)*0.5+80, \
  if(lt(X,$W*2), \
  mod(floor((X-$W)/2)*floor(Y/2),256)*0.6, \
  128+127*sin(sqrt(pow(X-853,2)+pow(Y-512,2))*0.02)*sin(sqrt(pow(X-750,2)+pow(Y-300,2))*0.025) \
)), 0, 255)'\
:g='clip(if(lt(X,$W), \
  (150-bitand(floor(X/2),floor(Y/2))*2)*sin(X*0.002+1)*sin(Y*0.003+0.3)*0.4+40, \
  if(lt(X,$W*2), \
  mod(floor((X-$W)/2)*floor(Y/2),256)*0.3, \
  80+80*sin(sqrt(pow(X-853,2)+pow(Y-512,2))*0.02)*sin(sqrt(pow(X-750,2)+pow(Y-300,2))*0.025) \
)), 0, 255)'\
:b='clip(if(lt(X,$W), \
  (100-bitand(floor(X/2),floor(Y/2))*1.5)*sin(X*0.004+0.7)*sin(Y*0.001+0.9)*0.3+30, \
  if(lt(X,$W*2), \
  mod(floor((X-$W)/2)*floor(Y/2),256)*0.15, \
  120+120*sin(sqrt(pow(X-853,2)+pow(Y-512,2))*0.02)*sin(sqrt(pow(X-750,2)+pow(Y-300,2))*0.025) \
)), 0, 255)'" \
    -frames:v 1 -update 1 "$OUTPUT" 2>/dev/null
    ;;

symmetry-spiral)
    # Competing symmetries: 6-fold at center, 5-fold at edges
    # Incommensurate angular frequencies create spiral arm structure
    # Dark purple on black
    N1=6
    N2=$((5 + S1 % 3))  # 5, 6, or 7-fold outer symmetry
    RFADE=$((320 + S2 % 160))  # crossfade radius 320-480
    RFREQ=$(fmath "0.025 + $S3 / 25600")
    ffmpeg -f lavfi -i "color=c=black:s=$SIZE:d=1" -vf "format=gbrp,geq=\
r='clip( \
  (1-min(1,hypot(X-512,Y-512)/$RFADE)) * 50 * lt(abs(sin(atan2(Y-512,X-512)*$N1+hypot(X-512,Y-512)*$RFREQ)), 0.4) \
  + min(1,hypot(X-512,Y-512)/$RFADE) * 45 * lt(abs(sin(atan2(Y-512,X-512)*$N2+hypot(X-512,Y-512)*$RFREQ*0.85)), 0.35) \
  + 10*exp(-hypot(X-512,Y-512)/130) \
, 0, 255)'\
:g='clip( \
  (1-min(1,hypot(X-512,Y-512)/$RFADE)) * 30 * lt(abs(sin(atan2(Y-512,X-512)*$N1+hypot(X-512,Y-512)*$RFREQ)), 0.4) \
  + min(1,hypot(X-512,Y-512)/$RFADE) * 25 * lt(abs(sin(atan2(Y-512,X-512)*$N2+hypot(X-512,Y-512)*$RFREQ*0.85)), 0.35) \
  + 15*exp(-hypot(X-512,Y-512)/110) \
, 0, 255)'\
:b='clip( \
  (1-min(1,hypot(X-512,Y-512)/$RFADE)) * 80 * lt(abs(sin(atan2(Y-512,X-512)*$N1+hypot(X-512,Y-512)*$RFREQ)), 0.4) \
  + min(1,hypot(X-512,Y-512)/$RFADE) * 70 * lt(abs(sin(atan2(Y-512,X-512)*$N2+hypot(X-512,Y-512)*$RFREQ*0.85)), 0.35) \
  + 6*exp(-hypot(X-512,Y-512)/160) \
  + 8 \
, 0, 255)'" \
    -frames:v 1 -update 1 "$OUTPUT" 2>/dev/null
    ;;

flow-ribbons)
    # Multi-scale flow lines with spiral distortion and radial color shift
    # Warm amber center (breath) fading to cool blue edges (wind)
    # Four octaves of threshold ribbons + atan2 spiral warp
    F1=$(fmath "0.018 + $S1 / 25600")
    F2=$(fmath "0.023 + $S2 / 25600")
    SPIRAL=$(fmath "0.4 + $S3 / 512")    # spiral distortion 0.4-0.9
    RFOCUS=$((380 + S4 % 100))            # radial focus 380-480
    ffmpeg -f lavfi -i "nullsrc=s=$SIZE:d=1" -vf "format=yuv444p,geq=\
lum=clip((255*lt(abs(sin(X*$F1+sin(Y*0.015+sin(X*0.004)*5)*3+atan2(Y-512\,X-512)*$SPIRAL))\,0.2)*0.55\
+255*lt(abs(sin(Y*$F2+sin(X*0.018+sin(Y*0.006)*4)*2.5+atan2(Y-512\,X-512)*0.3))\,0.16)*0.4\
+255*lt(abs(sin(X*0.07+Y*0.05+sin(X*0.012+Y*0.009)*6))\,0.13)*0.25\
+255*lt(abs(sin(X*0.18+sin(Y*0.14)*2))\,0.1)*0.12\
)*(0.15+0.85*exp(-pow(hypot(X-512\,Y-512)/$RFOCUS\,2)))\,0\,255)\
:cb=clip(128+25*(1-exp(-pow(hypot(X-512\,Y-512)/350\,2)))-20*exp(-pow(hypot(X-512\,Y-512)/300\,2))\,0\,255)\
:cr=clip(128+25*exp(-pow(hypot(X-512\,Y-512)/280\,2))-15*(1-exp(-pow(hypot(X-512\,Y-512)/400\,2)))\,0\,255)" \
    -frames:v 1 -update 1 "$OUTPUT" 2>/dev/null
    ;;

*)
    echo "Unknown technique: $TECHNIQUE" >&2
    echo "Available: sierpinski-zoom threshold-cells edge-of-chaos node-halos quasicrystal moire-rings mandelbrot-deep bitwise-triptych symmetry-spiral flow-ribbons" >&2
    exit 1
    ;;
esac

echo "Generated: $OUTPUT (technique: $TECHNIQUE, seed: $SEED)"
