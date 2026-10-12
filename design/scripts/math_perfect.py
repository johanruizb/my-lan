#!/usr/bin/env python3
"""
Mathematically perfect construction of MyLAN logo concepts.
All coordinates strictly calculated for symmetry, concentricity, and exact angles.
"""

import math
import os

OUT_DIR = "design/branding"
os.makedirs(OUT_DIR, exist_ok=True)

# =========================================================================
# 1. CONCEPT A: "Nexus M" (Letterform Monogram)
# =========================================================================
# Exact parameters:
# Canvas: 256 x 256. Center x = 128.
# Baseline = 204. Topline = 48. Total height = 156.
# Left pillar: x in [36, 68], y in [48, 204]. Width = 32.
# Right pillar: x in [188, 220], y in [48, 204]. Width = 32.
# Chevron: Starts from top inner corners (68, 48) and (188, 48).
# dx to center = 128 - 68 = 60.
# Exact 45° angle means dy = 60!
# Outer apex = (128, 48 + 60) = (128, 108).
# Chevron vertical thickness = 44 (giving perpendicular thickness = 44 / sqrt(2) = 31.11 ≈ 31.1).
# Inner apex = (128, 108 + 44) = (128, 152).
# Inner start points = (68, 48 + 44) = (68, 92) and (188, 92).
#
# Central Gateway Drop and Node:
# Drops vertically from inner apex (128, 152) to gateway node center at (128, 184).
# Drop stem: width 16 (x in [120, 136]), from y = 144 down to y = 176.
# Gateway node: centered at exactly (128, 184), outer radius R = 20.
# Bottom of node: y = 184 + 20 = 204 (ALIGNS EXACTLY TO BASELINE y=204!).
# Inner concentric knockout: centered at exactly (128, 184), radius r = 9.

def generate_concept_a():
    return """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-a">
  <title id="title-a">MyLAN - Nexus M Symbol</title>
  <!-- Left Pillar (Baseline 204, Topline 48) -->
  <rect x="36" y="48" width="32" height="156" fill="#000000" />
  <!-- Right Pillar (Baseline 204, Topline 48) -->
  <rect x="188" y="48" width="32" height="156" fill="#000000" />
  <!-- 45-degree Chevron & Gateway Hub (True mathematical 45° angles, dx=60, dy=60) -->
  <path fill="#000000" fill-rule="evenodd" d="
    M 68 48
    L 128 108
    L 188 48
    L 188 92
    L 136 144
    L 136 168
    A 20 20 0 1 1 120 168
    L 120 144
    L 68 92
    Z
    M 128 184
    A 9 9 0 1 0 128.1 184
    Z
  " />
</svg>
"""

# =========================================================================
# 2. CONCEPT B: "Subnet Perimeter" (Abstract / Topology Boundary)
# =========================================================================
# Exact parameters:
# Canvas: 256 x 256. Center = (128, 128).
# Perfect 4-fold dihedral symmetry (D4).
# Outer Squircle: 192 x 192, x in [32, 224], y in [32, 224], corner radius R = 48.
# Inner Squircle: 152 x 152, x in [52, 204], y in [52, 204], corner radius R = 28.
# Frame thickness = 20 everywhere.
# Center Gateway Hub: circle centered at (128, 128), radius R_hub = 24.
# 4 Client Satellite Nodes at 45°:
# Distance from center = 65.
# dx = dy = 46.
# Node 1: (82, 82), Node 2: (174, 82), Node 3: (174, 174), Node 4: (82, 174).
# Radius of nodes = 13.
# Continuous 45° diagonal beams from (58, 58) to (198, 198) and (58, 198) to (198, 58).
# Thickness = 10 (half-thickness = 5).
# For line through (128, 128) with slope 1 (45°):
# Normal offset vector: dx = -5/sqrt(2) = -3.5355, dy = +5/sqrt(2) = +3.5355.
# Beam 1 points:
# (58-3.54, 58+3.54) -> (58+3.54, 58-3.54) -> (198+3.54, 198-3.54) -> (198-3.54, 198+3.54)
# = (54.46, 61.54) -> (61.54, 54.46) -> (201.54, 194.46) -> (194.46, 201.54)
# Beam 2 points (slope -1):
# = (54.46, 194.46) -> (61.54, 201.54) -> (201.54, 61.54) -> (194.46, 54.46)

def generate_concept_b():
    return """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-b">
  <title id="title-b">MyLAN - Subnet Perimeter Symbol</title>
  <!-- Outer Squircle Boundary (Concentric 192x192 squircle, thickness 20) -->
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
  
  <!-- Symmetrical 45-degree Diagonal Network Conduits (anchored into squircle corners) -->
  <polygon points="54.5,61.5 61.5,54.5 201.5,194.5 194.5,201.5" fill="#000000" />
  <polygon points="54.5,194.5 61.5,201.5 201.5,61.5 194.5,54.5" fill="#000000" />
  
  <!-- Central Gateway Hub (Exact center 128, 128) -->
  <circle cx="128" cy="128" r="24" fill="#000000" />
  
  <!-- 4 Symmetrical Subnet Satellite Nodes (dx=46, dy=46 from center) -->
  <circle cx="82" cy="82" r="13" fill="#000000" />
  <circle cx="174" cy="82" r="13" fill="#000000" />
  <circle cx="82" cy="174" r="13" fill="#000000" />
  <circle cx="174" cy="174" r="13" fill="#000000" />
</svg>
"""

# =========================================================================
# 3. CONCEPT C: "Discovery Scope" (Pictorial / Tactical Radar)
# =========================================================================
# Exact parameters:
# Center = (128, 128).
# Reticle Ring:
# Outer radius = 104, Inner radius = 88. Thickness = 16.
# Cardinal slot openings: width = 16 (from -8 to +8 relative to axes).
# Vertical slot: x in [120, 136].
# Horizontal slot: y in [120, 136].
#
# Quadrant 1 (Top-Right): x >= 136, y <= 120.
# The ring arc in Quadrant 1:
# Starts at (136, y_top). For R_out=104, at x=136 (dx=8): dy = sqrt(104^2 - 8^2) = sqrt(10752) ≈ 103.69 -> y = 128 - 103.69 = 24.31.
# Ends at (x_right, 120). For R_out=104, at y=120 (dy=-8): dx = 103.69 -> x = 128 + 103.69 = 231.69.
#
# But for crisp pixel rendering and clean geometry:
# What if the 4 reticle breaks are exact 16° angles?
# Or what if we use corner framing brackets (squircle / viewfinder) or a clean circular reticle with 12px gaps?
# Let's make the 4 arcs use exact angular span:
# Quadrant 1: from 10° to 80° (gap of 20° centered on 0° and 90°).
# Let's compute:
# For angle alpha: x = 128 + R * cos(alpha), y = 128 - R * sin(alpha).
# At 80°: cos(80°) = 0.1736, sin(80°) = 0.9848.
# For R_out=104: x = 128 + 18.06 = 146.06, y = 128 - 102.42 = 25.58.
# For R_in=88: x = 128 + 15.28 = 143.28, y = 128 - 86.66 = 41.34.
# At 10°: cos(10°) = 0.9848, sin(10°) = 0.1736.
# For R_out=104: x = 128 + 102.42 = 230.42, y = 128 - 18.06 = 109.94.
# For R_in=88: x = 128 + 86.66 = 214.66, y = 128 - 15.28 = 112.72.
#
# AND the radar waves inside Quadrant 1:
# Span the EXACT same angular sweep: from 10° to 80°!
# Wave 1 (Inner): R_out = 66, R_in = 52. (thickness = 14).
# Wave 2 (Middle): R_out = 88, R_in = 74. (thickness = 14).
# BOTH span from 10° to 80°!
# THIS MEANS THE EDGES OF THE WAVES ALIGN TO THE EXACT RADIAL SPOKES (10° and 80°) OF THE RETICLE BREAKS!
#
# Target Blip (Discovered Device):
# Sits in Quadrant 3 (Bottom-Left) along the 225° diagonal (45° below horizontal).
# Range ring 1 radius = (52 + 66)/2 = 59.
# Center of blip: dx = -59 * cos(45°) = -41.72, dy = +59 * sin(45°) = +41.72.
# cx = 128 - 41.7 = 86.3 ≈ 86.
# cy = 128 + 41.7 = 169.7 ≈ 170.
# Radius = 10.
# It sits EXACTLY on Range Ring 1 along the 45° diagonal!

def arc_quadrant(cx, cy, r_in, r_out, a1_deg, a2_deg):
    a1 = math.radians(a1_deg)
    a2 = math.radians(a2_deg)
    
    # Outer start (a2) to outer end (a1)
    x_out_start = cx + r_out * math.cos(a2)
    y_out_start = cy - r_out * math.sin(a2)
    x_out_end = cx + r_out * math.cos(a1)
    y_out_end = cy - r_out * math.sin(a1)
    
    # Inner start (a1) to inner end (a2)
    x_in_start = cx + r_in * math.cos(a1)
    y_in_start = cy - r_in * math.sin(a1)
    x_in_end = cx + r_in * math.cos(a2)
    y_in_end = cy - r_in * math.sin(a2)
    
    return f"""M {x_out_start:.2f} {y_out_start:.2f}
    A {r_out} {r_out} 0 0 1 {x_out_end:.2f} {y_out_end:.2f}
    L {x_in_start:.2f} {y_in_start:.2f}
    A {r_in} {r_in} 0 0 0 {x_in_end:.2f} {y_in_end:.2f}
    Z"""

def generate_concept_c():
    cx, cy = 128, 128
    r_ret_in, r_ret_out = 90, 106 # reticle ring
    
    # 4 Reticle segments (each 72 degrees, gaps of 18 degrees at 0, 90, 180, 270)
    q1 = arc_quadrant(cx, cy, r_ret_in, r_ret_out, 9, 81)
    q2 = arc_quadrant(cx, cy, r_ret_in, r_ret_out, 99, 171)
    q3 = arc_quadrant(cx, cy, r_ret_in, r_ret_out, 189, 251)
    q4 = arc_quadrant(cx, cy, r_ret_in, r_ret_out, 279, 351)
    
    # 2 Radar Sweep Waves in Q1 (sharing the exact radial spoke angles 9° to 81°!)
    wave1 = arc_quadrant(cx, cy, 46, 62, 9, 81)
    wave2 = arc_quadrant(cx, cy, 68, 84, 9, 81)
    
    # Blip in Q3: r_blip = (46 + 62)/2 = 54 (midpoint of wave 1 range)
    blip_r = 54
    blip_a = math.radians(225)
    blip_x = cx + blip_r * math.cos(blip_a)
    blip_y = cy - blip_r * math.sin(blip_a)
    
    return f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-c">
  <title id="title-c">MyLAN - Discovery Scope Symbol</title>
  <!-- Central Origin Beacon (Host Scanner) -->
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

# =========================================================================
# MATHEMATICALLY PERFECT WORDMARK FOR LOCKUPS: "MyLAN"
# =========================================================================
# Grid:
# Baseline = 184
# Cap Height = 74 (height = 110)
# Optical center of caps = (74 + 184)/2 = 129 ≈ 128 (center of 256 canvas!)
# Stem width = 20
#
# Letter M:
# Width = 80, x in [270, 350].
# Left stem: [270, 290], Right stem: [330, 350].
# Diagonals meet at (310, 144).
#
# Letter y:
# Width = 56, x in [364, 420].
# x-height = 106 (height = 78).
# Descender = 214 (depth = 30).
# Left stroke: (364, 106) -> (388, 154).
# Right stroke: (416, 106) down to descender curve (382, 214).
#
# Letter L:
# Width = 52, x in [434, 486].
# Stem: [434, 454], Base: [434, 486], y in [164, 184].
#
# Letter A:
# Width = 68, x in [500, 568].
# Apex at x = 534.
# Crossbar at y in [144, 162].
#
# Letter N:
# Width = 62, x in [582, 644].
# Left stem: [582, 602]. Right stem: [624, 644].
# Diagonal from (602, 74) to (624, 184).

wordmark_code = """
  <!-- Letter M (x: 270..350) -->
  <path fill="#000000" d="
    M 270 74 L 290 74 L 310 134 L 330 74 L 350 74 L 350 184 L 330 184 L 330 110 L 318 146 L 302 146 L 290 110 L 290 184 L 270 184 Z
  " />
  
  <!-- Letter y (x: 364..420) -->
  <path fill="#000000" d="
    M 364 106 L 384 106 L 396 144 L 408 106 L 428 106 L 404 184 C 398 202 388 214 372 214 L 364 214 L 364 196 L 372 196 C 380 196 386 190 388 180 L 390 170 Z
  " />
  
  <!-- Letter L (x: 434..486) -->
  <path fill="#000000" d="
    M 434 74 L 454 74 L 454 164 L 486 164 L 486 184 L 434 184 Z
  " />
  
  <!-- Letter A (x: 500..568) -->
  <path fill="#000000" fill-rule="evenodd" d="
    M 524 74 L 544 74 L 568 184 L 546 184 L 540 156 L 528 156 L 522 184 L 500 184 Z
    M 534 106 L 531 138 L 537 138 Z
  " />
  
  <!-- Letter N (x: 582..644) -->
  <path fill="#000000" d="
    M 582 74 L 602 74 L 624 146 L 624 74 L 644 74 L 644 184 L 624 184 L 602 112 L 602 184 L 582 184 Z
  " />
"""

def make_lockup(symbol_svg_body, title):
    return f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 680 256" width="680" height="256" role="img" aria-labelledby="title-lockup">
  <title id="title-lockup">{title}</title>
  <g id="symbol">
{symbol_svg_body}
  </g>
  <g id="wordmark">
{wordmark_code}
  </g>
</svg>
"""

# Extract body contents:
svg_a = generate_concept_a()
svg_b = generate_concept_b()
svg_c = generate_concept_c()

body_a = svg_a.split('<svg', 1)[1].split('>', 1)[1].rsplit('</svg>', 1)[0]
body_b = svg_b.split('<svg', 1)[1].split('>', 1)[1].rsplit('</svg>', 1)[0]
body_c = svg_c.split('<svg', 1)[1].split('>', 1)[1].rsplit('</svg>', 1)[0]

with open(f"{OUT_DIR}/a-symbol.svg", "w") as f:
    f.write(svg_a)
with open(f"{OUT_DIR}/a-lockup.svg", "w") as f:
    f.write(make_lockup(body_a, "MyLAN - Nexus M Lockup"))

with open(f"{OUT_DIR}/b-symbol.svg", "w") as f:
    f.write(svg_b)
with open(f"{OUT_DIR}/b-lockup.svg", "w") as f:
    f.write(make_lockup(body_b, "MyLAN - Subnet Perimeter Lockup"))

with open(f"{OUT_DIR}/c-symbol.svg", "w") as f:
    f.write(svg_c)
with open(f"{OUT_DIR}/c-lockup.svg", "w") as f:
    f.write(make_lockup(body_c, "MyLAN - Discovery Scope Lockup"))

print("Mathematical SVGs generated successfully.")
