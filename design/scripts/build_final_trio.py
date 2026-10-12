#!/usr/bin/env python3
"""
Build the 3 perfected final SVG concepts for MyLAN, plus their vector lockups.
100% pure paths, no XML comments in attributes, no text tags, integer grid.
"""

import os

OUT_DIR = "design/branding"
os.makedirs(OUT_DIR, exist_ok=True)

# =========================================================================
# 1. CONCEPT A: "Nexus M" (Letterform Monogram)
# =========================================================================
# Symmetrical geometric M where outer stems represent infrastructure,
# and central chevron routes down into an integrated gateway hub.
concept_a_symbol = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-a">
  <title id="title-a">MyLAN - Nexus M Symbol</title>
  <g fill="#000000">
    <!-- Left Pillar -->
    <rect x="36" y="48" width="36" height="160" rx="2" />
    <!-- Right Pillar -->
    <rect x="184" y="48" width="36" height="160" rx="2" />
    <!-- Central Routing Chevron & Drop Stem -->
    <path d="
      M 72 48
      L 128 112
      L 184 48
      L 184 84
      L 136 138
      L 136 160
      A 20 20 0 1 1 120 160
      L 120 138
      L 72 84
      Z
    " />
    <!-- Negative space circular core inside gateway hub -->
    <circle cx="128" cy="178" r="8" fill="#ffffff" />
  </g>
</svg>
"""

# Let's also create the true single-path version of A (using evenodd so white hole is transparent):
concept_a_symbol_evenodd = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-a">
  <title id="title-a">MyLAN - Nexus M Symbol</title>
  <path fill="#000000" fill-rule="evenodd" d="
    M 36 48 L 72 48 L 72 84 L 120 138 L 120 160
    A 20 20 0 0 0 110 178
    A 20 20 0 0 0 128 198
    A 20 20 0 0 0 146 178
    A 20 20 0 0 0 136 160
    L 136 138 L 184 84 L 184 48 L 220 48 L 220 208 L 184 208 L 184 102
    L 128 166 L 72 102 L 72 208 L 36 208 Z
    M 128 170
    A 8 8 0 1 1 127.9 170
    Z
  " />
</svg>
"""

# Let's keep `concept_a_symbol_v2`:
# Even better: The chevron with the hub as a distinct group:
concept_a_clean = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-a">
  <title id="title-a">MyLAN - Nexus M Symbol</title>
  <!-- Left Pillar -->
  <rect x="36" y="48" width="36" height="160" rx="2" fill="#000000" />
  <!-- Right Pillar -->
  <rect x="184" y="48" width="36" height="160" rx="2" fill="#000000" />
  <!-- Routing Chevron & Gateway Hub with real transparent hole -->
  <path fill="#000000" fill-rule="evenodd" d="
    M 72 48
    L 128 112
    L 184 48
    L 184 86
    L 136 140
    L 136 160
    A 20 20 0 1 1 120 160
    L 120 140
    L 72 86
    Z
    M 128 170
    A 8 8 0 1 0 128.1 170
    Z
  " />
</svg>
"""

# =========================================================================
# 2. CONCEPT B: "Subnet Perimeter" (Abstract / Topology Boundary)
# =========================================================================
# Squircle protective boundary framing an internal 4-point star constellation.
# Represents "Tu red, bajo control" - local network autonomy.
concept_b_clean = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-b">
  <title id="title-b">MyLAN - Subnet Perimeter Symbol</title>
  <!-- Outer Perimeter Boundary (thickness 20) -->
  <path fill="#000000" fill-rule="evenodd" d="
    M 76 32
    L 180 32
    A 44 44 0 0 1 224 76
    L 224 180
    A 44 44 0 0 1 180 224
    L 76 224
    A 44 44 0 0 1 32 180
    L 32 76
    A 44 44 0 0 1 76 32
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
  
  <!-- Central Gateway Hub -->
  <circle cx="128" cy="128" r="22" fill="#000000" />
  
  <!-- 4 Satellite Client Nodes (Corners) -->
  <circle cx="82" cy="82" r="12" fill="#000000" />
  <circle cx="174" cy="82" r="12" fill="#000000" />
  <circle cx="82" cy="174" r="12" fill="#000000" />
  <circle cx="174" cy="174" r="12" fill="#000000" />
  
  <!-- 4 Diagonal Bus Conduits (thickness 10) -->
  <path fill="#000000" d="M 82 82 L 174 174 L 167 181 L 75 89 Z" />
  <path fill="#000000" d="M 174 82 L 82 174 L 89 181 L 181 89 Z" />
  
  <!-- Corner Anchorage Links to Perimeter Frame -->
  <rect x="52" y="77" width="30" height="10" fill="#000000" />
  <rect x="174" y="77" width="30" height="10" fill="#000000" />
  <rect x="52" y="169" width="30" height="10" fill="#000000" />
  <rect x="174" y="169" width="30" height="10" fill="#000000" />
</svg>
"""

# =========================================================================
# 3. CONCEPT C: "Discovery Scope" (Pictorial / Local Radar)
# =========================================================================
# Tactical circular radar reticle with cardinal crosshair breaks,
# central origin beacon, concentric scanning pulses, and discovered host blip.
concept_c_clean = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-c">
  <title id="title-c">MyLAN - Discovery Scope Symbol</title>
  <!-- Central Origin Beacon (Host Scanner) -->
  <circle cx="128" cy="128" r="22" fill="#000000" />
  
  <!-- Outer Circular Scope Frame with 4 Cardinal Reticle Breaks (thickness 16) -->
  <path fill="#000000" d="M 138 28 A 100 100 0 0 1 228 118 L 212 118 A 84 84 0 0 0 138 44 Z" />
  <path fill="#000000" d="M 228 138 A 100 100 0 0 1 138 228 L 138 212 A 84 84 0 0 0 212 138 Z" />
  <path fill="#000000" d="M 118 228 A 100 100 0 0 1 28 138 L 44 138 A 84 84 0 0 0 118 212 Z" />
  <path fill="#000000" d="M 28 118 A 100 100 0 0 1 118 28 L 118 44 A 84 84 0 0 0 44 118 Z" />
  
  <!-- Discovery Pulse Wave 1 (Inner Arc, radius 56..72) in Upper-Right Sector -->
  <path fill="#000000" d="M 128 72 A 56 56 0 0 1 184 128 L 168 128 A 40 40 0 0 0 128 88 Z" />
  <!-- Discovery Pulse Wave 2 (Middle Arc, radius 78..94) in Upper-Right Sector -->
  <path fill="#000000" d="M 128 48 A 80 80 0 0 1 208 128 L 192 128 A 64 64 0 0 0 128 64 Z" />
  
  <!-- Target Blip (Discovered Local Host Device) -->
  <circle cx="86" cy="170" r="11" fill="#000000" />
</svg>
"""

# =========================================================================
# OUTLINED WORDMARK FOR LOCKUPS: "MyLAN"
# Clean geometric vector letterforms. Cap height = 120 (y 68..188), Baseline = y 188.
# M: width 72, gap 12, y: width 54, gap 16, L: width 48, gap 14, A: width 64, gap 14, N: width 58.
# =========================================================================
wordmark_paths = """
  <!-- Letter M (x: 270..346) -->
  <path fill="#000000" d="
    M 270 68 L 292 68 L 308 132 L 324 68 L 346 68 L 346 188 L 326 188 L 326 106 L 314 154 L 302 154 L 290 106 L 290 188 L 270 188 Z
  " />
  
  <!-- Letter y (x: 358..408) -->
  <path fill="#000000" d="
    M 358 104 L 378 104 L 388 144 L 398 104 L 418 104 L 394 186 C 390 200 380 208 366 208 L 360 208 L 360 190 L 366 190 C 374 190 378 184 380 176 L 382 168 Z
  " />
  
  <!-- Letter L (x: 434..480) -->
  <path fill="#000000" d="
    M 434 68 L 454 68 L 454 168 L 484 168 L 484 188 L 434 188 Z
  " />
  
  <!-- Letter A (x: 494..556) -->
  <path fill="#000000" fill-rule="evenodd" d="
    M 515 68 L 535 68 L 560 188 L 538 188 L 532 158 L 518 158 L 512 188 L 490 188 Z
    M 525 102 L 521 140 L 529 140 Z
  " />
  
  <!-- Letter N (x: 570..626) -->
  <path fill="#000000" d="
    M 570 68 L 590 68 L 610 144 L 610 68 L 630 68 L 630 188 L 610 188 L 590 112 L 590 188 L 570 188 Z
  " />
"""

def make_lockup(symbol_inner, title):
    return f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 680 256" width="680" height="256" role="img" aria-labelledby="title-lockup">
  <title id="title-lockup">{title}</title>
  <g id="symbol">
{symbol_inner}
  </g>
  <g id="wordmark">
{wordmark_paths}
  </g>
</svg>
"""

# Extract inner symbol parts for lockup:
# Symbol A inner:
a_inner = """    <rect x="36" y="48" width="36" height="160" rx="2" fill="#000000" />
    <rect x="184" y="48" width="36" height="160" rx="2" fill="#000000" />
    <path fill="#000000" fill-rule="evenodd" d="
      M 72 48 L 128 112 L 184 48 L 184 86 L 136 140 L 136 160
      A 20 20 0 1 1 120 160 L 120 140 L 72 86 Z
      M 128 170 A 8 8 0 1 0 128.1 170 Z
    " />"""

# Symbol B inner:
b_inner = """    <path fill="#000000" fill-rule="evenodd" d="
      M 76 32 L 180 32 A 44 44 0 0 1 224 76 L 224 180 A 44 44 0 0 1 180 224 L 76 224 A 44 44 0 0 1 32 180 L 32 76 A 44 44 0 0 1 76 32 Z
      M 80 52 A 28 28 0 0 0 52 80 L 52 176 A 28 28 0 0 0 80 204 L 176 204 A 28 28 0 0 0 204 176 L 204 80 A 28 28 0 0 0 176 52 L 80 52 Z
    " />
    <circle cx="128" cy="128" r="22" fill="#000000" />
    <circle cx="82" cy="82" r="12" fill="#000000" />
    <circle cx="174" cy="82" r="12" fill="#000000" />
    <circle cx="82" cy="174" r="12" fill="#000000" />
    <circle cx="174" cy="174" r="12" fill="#000000" />
    <path fill="#000000" d="M 82 82 L 174 174 L 167 181 L 75 89 Z" />
    <path fill="#000000" d="M 174 82 L 82 174 L 89 181 L 181 89 Z" />
    <rect x="52" y="77" width="30" height="10" fill="#000000" />
    <rect x="174" y="77" width="30" height="10" fill="#000000" />
    <rect x="52" y="169" width="30" height="10" fill="#000000" />
    <rect x="174" y="169" width="30" height="10" fill="#000000" />"""

# Symbol C inner:
c_inner = """    <circle cx="128" cy="128" r="22" fill="#000000" />
    <path fill="#000000" d="M 138 28 A 100 100 0 0 1 228 118 L 212 118 A 84 84 0 0 0 138 44 Z" />
    <path fill="#000000" d="M 228 138 A 100 100 0 0 1 138 228 L 138 212 A 84 84 0 0 0 212 138 Z" />
    <path fill="#000000" d="M 118 228 A 100 100 0 0 1 28 138 L 44 138 A 84 84 0 0 0 118 212 Z" />
    <path fill="#000000" d="M 28 118 A 100 100 0 0 1 118 28 L 118 44 A 84 84 0 0 0 44 118 Z" />
    <path fill="#000000" d="M 128 72 A 56 56 0 0 1 184 128 L 168 128 A 40 40 0 0 0 128 88 Z" />
    <path fill="#000000" d="M 128 48 A 80 80 0 0 1 208 128 L 192 128 A 64 64 0 0 0 128 64 Z" />
    <circle cx="86" cy="170" r="11" fill="#000000" />"""

with open(f"{OUT_DIR}/a-symbol.svg", "w") as f:
    f.write(concept_a_clean)
with open(f"{OUT_DIR}/a-lockup.svg", "w") as f:
    f.write(make_lockup(a_inner, "MyLAN - Nexus M Lockup"))

with open(f"{OUT_DIR}/b-symbol.svg", "w") as f:
    f.write(concept_b_clean)
with open(f"{OUT_DIR}/b-lockup.svg", "w") as f:
    f.write(make_lockup(b_inner, "MyLAN - Subnet Perimeter Lockup"))

with open(f"{OUT_DIR}/c-symbol.svg", "w") as f:
    f.write(concept_c_clean)
with open(f"{OUT_DIR}/c-lockup.svg", "w") as f:
    f.write(make_lockup(c_inner, "MyLAN - Discovery Scope Lockup"))

print("Wrote symbols and lockups for A, B, C.")
