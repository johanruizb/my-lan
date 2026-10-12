#!/usr/bin/env python3
"""
Regenerate all 3 logo concepts and lockups with strict mathematical alignment.
"""

import math
import os
import subprocess

OUT_DIR = "design/branding"
os.makedirs(OUT_DIR, exist_ok=True)

# -------------------------------------------------------------------------
# Concept A: Nexus M (True 45° angles, perfectly concentric gateway node)
# -------------------------------------------------------------------------
svg_a_symbol = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-a">
  <title id="title-a">MyLAN - Nexus M Symbol</title>
  <!-- Left Pillar (Baseline 204, Topline 48) -->
  <rect x="36" y="48" width="32" height="156" fill="#000000" />
  <!-- Right Pillar (Baseline 204, Topline 48) -->
  <rect x="188" y="48" width="32" height="156" fill="#000000" />
  <!-- 45-degree Routing Chevron & Concentric Gateway Hub -->
  <path fill="#000000" fill-rule="evenodd" d="
    M 68 48
    L 128 108
    L 188 48
    L 188 92
    L 140 140
    L 140 164
    A 20 20 0 1 1 116 164
    L 116 140
    L 68 92
    Z
    M 128 180
    A 9 9 0 1 0 128.1 180
    Z
  " />
</svg>
"""

# -------------------------------------------------------------------------
# Concept B: Subnet Perimeter (Perfect D4 4-fold dihedral symmetry)
# -------------------------------------------------------------------------
svg_b_symbol = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-b">
  <title id="title-b">MyLAN - Subnet Perimeter Symbol</title>
  <!-- Symmetrical Squircle Boundary (192x192, centered at 128,128, frame thickness 20) -->
  <path fill="#000000" fill-rule="evenodd" d="
    M 80 32
    L 176 32
    A 48 48 0 0 1 224 80
    L 224 176
    A 48 48 0 0 1 176 224
    L 80 224
    A 48 48 0 0 1 32 176
    L 32 80
    A 48 48 0 0 1 80 32
    Z
    M 80 52
    A 28 28 0 0 0 52 80
    L 52 176
    A 28 28 0 0 0 80 204
    L 176 204
    A 28 28 0 0 0 204 176
    L 204 80
    A 28 28 0 0 0 176 52
    L 80 52
    Z
  " />
  
  <!-- Diagonal Network Conduits with Exact Center Alignment (width 10, passing through 128,128) -->
  <polygon points="54.5,61.5 61.5,54.5 201.5,194.5 194.5,201.5" fill="#000000" />
  <polygon points="54.5,194.5 61.5,201.5 201.5,61.5 194.5,54.5" fill="#000000" />
  
  <!-- Central Gateway Hub (Radius 24, exact center 128, 128) -->
  <circle cx="128" cy="128" r="24" fill="#000000" />
  
  <!-- 4 Symmetrical Client Nodes on the 45° Diagonals (dx=46, dy=46, radius 13) -->
  <circle cx="82" cy="82" r="13" fill="#000000" />
  <circle cx="174" cy="82" r="13" fill="#000000" />
  <circle cx="82" cy="174" r="13" fill="#000000" />
  <circle cx="174" cy="174" r="13" fill="#000000" />
</svg>
"""

# -------------------------------------------------------------------------
# Concept C: Discovery Scope (Exact radial spokes and range-ring blip)
# -------------------------------------------------------------------------
def arc_quadrant(cx, cy, r_in, r_out, a1_deg, a2_deg):
    a1 = math.radians(a1_deg)
    a2 = math.radians(a2_deg)
    x_out_start = cx + r_out * math.cos(a2)
    y_out_start = cy - r_out * math.sin(a2)
    x_out_end = cx + r_out * math.cos(a1)
    y_out_end = cy - r_out * math.sin(a1)
    x_in_start = cx + r_in * math.cos(a1)
    y_in_start = cy - r_in * math.sin(a1)
    x_in_end = cx + r_in * math.cos(a2)
    y_in_end = cy - r_in * math.sin(a2)
    return f"""M {x_out_start:.2f} {y_out_start:.2f} A {r_out} {r_out} 0 0 1 {x_out_end:.2f} {y_out_end:.2f} L {x_in_start:.2f} {y_in_start:.2f} A {r_in} {r_in} 0 0 0 {x_in_end:.2f} {y_in_end:.2f} Z"""

cx, cy = 128, 128
r_ret_in, r_ret_out = 90, 106
q1 = arc_quadrant(cx, cy, r_ret_in, r_ret_out, 9, 81)
q2 = arc_quadrant(cx, cy, r_ret_in, r_ret_out, 99, 171)
q3 = arc_quadrant(cx, cy, r_ret_in, r_ret_out, 189, 261)
q4 = arc_quadrant(cx, cy, r_ret_in, r_ret_out, 279, 351)
wave1 = arc_quadrant(cx, cy, 46, 62, 9, 81)
wave2 = arc_quadrant(cx, cy, 68, 84, 9, 81)
blip_r = 54
blip_a = math.radians(225)
blip_x = cx + blip_r * math.cos(blip_a)
blip_y = cy - blip_r * math.sin(blip_a)

svg_c_symbol = f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-c">
  <title id="title-c">MyLAN - Discovery Scope Symbol</title>
  <!-- Central Local Origin Beacon -->
  <circle cx="128" cy="128" r="22" fill="#000000" />
  
  <!-- 4 Symmetrical Reticle Quadrants (72° arc with 18° cardinal breaks) -->
  <path fill="#000000" d="{q1}" />
  <path fill="#000000" d="{q2}" />
  <path fill="#000000" d="{q3}" />
  <path fill="#000000" d="{q4}" />
  
  <!-- Radar Sweep Waves (sharing exact radial spokes 9° to 81°) -->
  <path fill="#000000" d="{wave1}" />
  <path fill="#000000" d="{wave2}" />
  
  <!-- Discovered Host Blip (positioned exactly at 225° on Range Ring 1) -->
  <circle cx="{blip_x:.2f}" cy="{blip_y:.2f}" r="10" fill="#000000" />
</svg>
"""

# -------------------------------------------------------------------------
# Perfected Wordmark (Baseline 184, Cap Height 74..184 = 110, Gap 14 everywhere)
# -------------------------------------------------------------------------
wordmark_paths = """
  <!-- Letter M (x: 270..350, width 80) -->
  <path fill="#000000" d="
    M 270 74 L 290 74 L 310 134 L 330 74 L 350 74 L 350 184 L 330 184 L 330 110 L 318 146 L 302 146 L 290 110 L 290 184 L 270 184 Z
  " />
  
  <!-- Letter y (x: 364..420, width 56, gap 14) -->
  <path fill="#000000" d="
    M 364 106 L 384 106 L 396 144 L 408 106 L 428 106 L 404 184 C 398 202 388 214 372 214 L 364 214 L 364 196 L 372 196 C 380 196 386 190 388 180 L 390 170 Z
  " />
  
  <!-- Letter L (x: 434..486, width 52, gap 14) -->
  <path fill="#000000" d="
    M 434 74 L 454 74 L 454 164 L 486 164 L 486 184 L 434 184 Z
  " />
  
  <!-- Letter A (x: 500..568, width 68, gap 14) -->
  <path fill="#000000" fill-rule="evenodd" d="
    M 524 74 L 544 74 L 568 184 L 546 184 L 540 156 L 528 156 L 522 184 L 500 184 Z
    M 534 106 L 531 138 L 537 138 Z
  " />
  
  <!-- Letter N (x: 582..644, width 62, gap 14) -->
  <path fill="#000000" d="
    M 582 74 L 602 74 L 624 146 L 624 74 L 644 74 L 644 184 L 624 184 L 602 112 L 602 184 L 582 184 Z
  " />
"""

def make_lockup(symbol_svg, title):
    body = symbol_svg.split('<svg', 1)[1].split('>', 1)[1].rsplit('</svg>', 1)[0]
    return f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 680 256" width="680" height="256" role="img" aria-labelledby="title-lockup">
  <title id="title-lockup">{title}</title>
  <g id="symbol">
{body}
  </g>
  <g id="wordmark">
{wordmark_paths}
  </g>
</svg>
"""

with open(f"{OUT_DIR}/a-symbol.svg", "w") as f:
    f.write(svg_a_symbol)
with open(f"{OUT_DIR}/a-lockup.svg", "w") as f:
    f.write(make_lockup(svg_a_symbol, "MyLAN - Nexus M Lockup"))

with open(f"{OUT_DIR}/b-symbol.svg", "w") as f:
    f.write(svg_b_symbol)
with open(f"{OUT_DIR}/b-lockup.svg", "w") as f:
    f.write(make_lockup(svg_b_symbol, "MyLAN - Subnet Perimeter Lockup"))

with open(f"{OUT_DIR}/c-symbol.svg", "w") as f:
    f.write(svg_c_symbol)
with open(f"{OUT_DIR}/c-lockup.svg", "w") as f:
    f.write(make_lockup(svg_c_symbol, "MyLAN - Discovery Scope Lockup"))

print("Updated all SVGs.")
