#!/usr/bin/env bash
# Coprime pairs: cell (x, y), counted from 1 at the bottom left, is gold when gcd(x, y) = 1.
# 150 cells a side, enlarged 8x with nearest-neighbour so each cell is a square.
# Counted separately: 13715 of 22500 at N=150 (60.96%), 875571 of 1440000 at N=1200 (60.80%); 6/pi^2 = 60.79%.
N=${1:-150}
ffmpeg -v error -y -f lavfi -i "nullsrc=s=${N}x${N},format=rgb24,geq=r='if(eq(gcd(X+1\,${N}-Y)\,1)\,232\,20)':g='if(eq(gcd(X+1\,${N}-Y)\,1)\,196\,26)':b='if(eq(gcd(X+1\,${N}-Y)\,1)\,120\,44)',scale=1200:1200:flags=neighbor" -frames:v 1 coprime.png
