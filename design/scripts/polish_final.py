#!/usr/bin/env python3
"""
Polish and finalize:
- Fix top junction in A-symbol (completely seamless, no sliver)
- Refine wordmark proportions for MyLAN (clean, modern geometric sans)
- Generate final SVGs for Phase 6 concept checkpoint
"""

import os

OUT_DIR = "design/branding"

# -------------------------------------------------------------------------
# Concept A: Nexus M (Refined)
# -------------------------------------------------------------------------
concept_a_svg = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-a">
  <title id="title-a">MyLAN - Nexus M Symbol</title>
  <!-- Unified solid construction: Left pillar, Right pillar, and Routing Chevron -->
  <!-- Left Pillar -->
  <rect x="36" y="48" width="36" height="160" fill="#000000" />
  <!-- Right Pillar -->
  <rect x="184" y="48" width="36" height="160" fill="#000000" />
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

# -------------------------------------------------------------------------
# Concept B: Subnet Perimeter (Refined)
# -------------------------------------------------------------------------
concept_b_svg = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-b">
  <title id="title-b">MyLAN - Subnet Perimeter Symbol</title>
  <!-- Protective Squircle Boundary -->
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
  <!-- 4 Satellite Client Nodes -->
  <circle cx="82" cy="82" r="12" fill="#000000" />
  <circle cx="174" cy="82" r="12" fill="#000000" />
  <circle cx="82" cy="174" r="12" fill="#000000" />
  <circle cx="174" cy="174" r="12" fill="#000000" />
  <!-- Diagonal Bus Conduits -->
  <path fill="#000000" d="M 82 82 L 174 174 L 167 181 L 75 89 Z" />
  <path fill="#000000" d="M 174 82 L 82 174 L 89 181 L 181 89 Z" />
  <!-- Anchorage to Boundary -->
  <rect x="52" y="77" width="30" height="10" fill="#000000" />
  <rect x="174" y="77" width="30" height="10" fill="#000000" />
  <rect x="52" y="169" width="30" height="10" fill="#000000" />
  <rect x="174" y="169" width="30" height="10" fill="#000000" />
</svg>
"""

# -------------------------------------------------------------------------
# Concept C: Discovery Scope (Refined)
# -------------------------------------------------------------------------
concept_c_svg = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-c">
  <title id="title-c">MyLAN - Discovery Scope Symbol</title>
  <!-- Central Local Origin Beacon -->
  <circle cx="128" cy="128" r="22" fill="#000000" />
  <!-- Outer Scope Reticle with 4 Cardinal Breaks -->
  <path fill="#000000" d="M 138 28 A 100 100 0 0 1 228 118 L 212 118 A 84 84 0 0 0 138 44 Z" />
  <path fill="#000000" d="M 228 138 A 100 100 0 0 1 138 228 L 138 212 A 84 84 0 0 0 212 138 Z" />
  <path fill="#000000" d="M 118 228 A 100 100 0 0 1 28 138 L 44 138 A 84 84 0 0 0 118 212 Z" />
  <path fill="#000000" d="M 28 118 A 100 100 0 0 1 118 28 L 118 44 A 84 84 0 0 0 44 118 Z" />
  <!-- Wave 1 (Inner Discovery Pulse Arc) -->
  <path fill="#000000" d="M 128 72 A 56 56 0 0 1 184 128 L 168 128 A 40 40 0 0 0 128 88 Z" />
  <!-- Wave 2 (Outer Discovery Pulse Arc) -->
  <path fill="#000000" d="M 128 48 A 80 80 0 0 1 208 128 L 192 128 A 64 64 0 0 0 128 64 Z" />
  <!-- Target Blip (Detected Subnet Host) -->
  <circle cx="86" cy="170" r="11" fill="#000000" />
</svg>
"""

# -------------------------------------------------------------------------
# Polished Outlined Wordmark: "MyLAN"
# Clean geometric grotesk proportions, balanced kerning.
# Baseline = 184, Cap Height = 74..184 (height 110).
# -------------------------------------------------------------------------
wordmark_paths = """
  <!-- Letter M (x: 272..352, width 80) -->
  <path fill="#000000" d="
    M 272 74
    L 294 74
    L 312 134
    L 330 74
    L 352 74
    L 352 184
    L 332 184
    L 332 108
    L 318 152
    L 306 152
    L 292 108
    L 292 184
    L 272 184
    Z
  " />
  
  <!-- Letter y (x: 366..420, width 54, x-height: 104..184, descender to 206) -->
  <path fill="#000000" d="
    M 366 104
    L 386 104
    L 396 142
    L 406 104
    L 426 104
    L 402 184
    C 398 198 388 206 374 206
    L 366 206
    L 366 188
    L 374 188
    C 380 188 384 184 386 176
    L 388 166
    Z
  " />
  
  <!-- Letter L (x: 442..492, width 50) -->
  <path fill="#000000" d="
    M 442 74
    L 462 74
    L 462 166
    L 496 166
    L 496 184
    L 442 184
    Z
  " />
  
  <!-- Letter A (x: 508..576, width 68) -->
  <path fill="#000000" fill-rule="evenodd" d="
    M 531 74
    L 553 74
    L 576 184
    L 554 184
    L 548 156
    L 536 156
    L 530 184
    L 508 184
    Z
    M 542 104
    L 539 138
    L 545 138
    Z
  " />
  
  <!-- Letter N (x: 590..650, width 60) -->
  <path fill="#000000" d="
    M 590 74
    L 610 74
    L 630 144
    L 630 74
    L 650 74
    L 650 184
    L 630 184
    L 610 114
    L 610 184
    L 590 184
    Z
  " />
"""

def make_lockup(symbol_inner, title):
    return f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 690 256" width="690" height="256" role="img" aria-labelledby="title-lockup">
  <title id="title-lockup">{title}</title>
  <g id="symbol">
{symbol_inner}
  </g>
  <g id="wordmark">
{wordmark_paths}
  </g>
</svg>
"""

a_inner = """    <rect x="36" y="48" width="36" height="160" fill="#000000" />
    <rect x="184" y="48" width="36" height="160" fill="#000000" />
    <path fill="#000000" fill-rule="evenodd" d="
      M 72 48 L 128 112 L 184 48 L 184 86 L 136 140 L 136 160
      A 20 20 0 1 1 120 160 L 120 140 L 72 86 Z
      M 128 170 A 8 8 0 1 0 128.1 170 Z
    " />"""

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

c_inner = """    <circle cx="128" cy="128" r="22" fill="#000000" />
    <path fill="#000000" d="M 138 28 A 100 100 0 0 1 228 118 L 212 118 A 84 84 0 0 0 138 44 Z" />
    <path fill="#000000" d="M 228 138 A 100 100 0 0 1 138 228 L 138 212 A 84 84 0 0 0 212 138 Z" />
    <path fill="#000000" d="M 118 228 A 100 100 0 0 1 28 138 L 44 138 A 84 84 0 0 0 118 212 Z" />
    <path fill="#000000" d="M 28 118 A 100 100 0 0 1 118 28 L 118 44 A 84 84 0 0 0 44 118 Z" />
    <path fill="#000000" d="M 128 72 A 56 56 0 0 1 184 128 L 168 128 A 40 40 0 0 0 128 88 Z" />
    <path fill="#000000" d="M 128 48 A 80 80 0 0 1 208 128 L 192 128 A 64 64 0 0 0 128 64 Z" />
    <circle cx="86" cy="170" r="11" fill="#000000" />"""

with open(f"{OUT_DIR}/a-symbol.svg", "w") as f:
    f.write(concept_a_svg)
with open(f"{OUT_DIR}/a-lockup.svg", "w") as f:
    f.write(make_lockup(a_inner, "MyLAN - Nexus M Lockup"))

with open(f"{OUT_DIR}/b-symbol.svg", "w") as f:
    f.write(concept_b_svg)
with open(f"{OUT_DIR}/b-lockup.svg", "w") as f:
    f.write(make_lockup(b_inner, "MyLAN - Subnet Perimeter Lockup"))

with open(f"{OUT_DIR}/c-symbol.svg", "w") as f:
    f.write(concept_c_svg)
with open(f"{OUT_DIR}/c-lockup.svg", "w") as f:
    f.write(make_lockup(c_inner, "MyLAN - Discovery Scope Lockup"))

print("Saved polished final SVGs.")
