#!/usr/bin/env python3
"""
Generate vector SVG concepts for MyLAN logo exploration.
Canvas: 256x256 integer grid.
Solid black, no live text, no filters, no raster.
"""
import os
import math

OUT_DIR = "design/branding"
os.makedirs(OUT_DIR, exist_ok=True)

# -------------------------------------------------------------------------
# Concept A: Nexus M (Letterform)
# Bold architectural M with precision data conduits and central nexus node.
# -------------------------------------------------------------------------
concept_a_svg = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-a">
  <title id="title-a">MyLAN Logo Concept A - Nexus M</title>
  <!-- Left Pillar -->
  <path fill="#000000" d="M 36 56 L 68 56 L 68 200 L 36 200 Z" />
  <!-- Right Pillar -->
  <path fill="#000000" d="M 188 56 L 220 56 L 220 200 L 188 200 Z" />
  <!-- Central Chevron / Routing apex -->
  <path fill="#000000" d="M 82 56 L 128 116 L 174 56 L 198 56 L 138 134 L 138 148 L 118 148 L 118 134 L 58 56 Z" />
  <!-- Nexus Hub Node (Gateway core) -->
  <circle cx="128" cy="184" r="16" fill="#000000" />
</svg>
"""

# Let's also explore a continuous-path version of Concept A (A2) with integrated conduits:
# An M formed from an interlocking geometric ribbon where the lines flow like a network trace:
concept_a2_svg = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-a">
  <title id="title-a">MyLAN Logo Concept A - Nexus M</title>
  <g fill="#000000">
    <!-- Left vertical bar with chamfered top -->
    <path d="M 36 68 L 56 48 L 72 48 L 72 208 L 36 208 Z" />
    <!-- Right vertical bar with chamfered top -->
    <path d="M 184 48 L 200 48 L 220 68 L 220 208 L 184 208 Z" />
    <!-- Center chevron -->
    <path d="M 86 64 L 128 118 L 170 64 L 184 82 L 138 140 L 138 152 L 118 152 L 118 140 L 72 82 Z" />
    <!-- Core Host Node -->
    <circle cx="128" cy="188" r="16" />
    <!-- Bridge link from chevron to node -->
    <rect x="123" y="152" width="10" height="24" rx="2" />
  </g>
</svg>
"""

# And an even sleeker, pure modern geometric M (A3) with balanced angles and negative space:
concept_a3_svg = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-a">
  <title id="title-a">MyLAN Logo Concept A - Nexus M</title>
  <path fill="#000000" fill-rule="evenodd" d="
    M 36 48 L 72 48 L 72 152 L 128 88 L 184 152 L 184 48 L 220 48 L 220 208 L 184 208 L 128 144 L 72 208 L 36 208 Z
    M 128 180 A 14 14 0 1 1 127.9 180 Z
  " />
</svg>
"""

with open(f"{OUT_DIR}/concept-a-v1.svg", "w") as f:
    f.write(concept_a_svg)
with open(f"{OUT_DIR}/concept-a-v2.svg", "w") as f:
    f.write(concept_a2_svg)
with open(f"{OUT_DIR}/concept-a-v3.svg", "w") as f:
    f.write(concept_a3_svg)

# -------------------------------------------------------------------------
# Concept B: Local Perimeter (Abstract / Topology Boundary)
# Heavy squircle perimeter shielding an internal local star/bus constellation.
# "Tu red, bajo control" - enclosed private domain.
# -------------------------------------------------------------------------
# Let's build a squircle outer frame with 4 connected internal subnet nodes:
concept_b_svg = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-b">
  <title id="title-b">MyLAN Logo Concept B - Local Perimeter</title>
  <!-- Outer perimeter squircle with clean rounded corners -->
  <path fill="#000000" fill-rule="evenodd" d="
    M 80 32
    L 176 32
    A 48 48 0 0 1 224 80
    L 224 176
    A 48 48 0 0 1 176 224
    L 80 224
    A 48 48 0 0 1 32 176
    L 32 80
    A 48 48 0 0 1 80 32 Z

    M 84 56
    A 28 28 0 0 0 56 84
    L 56 172
    A 28 28 0 0 0 84 200
    L 172 200
    A 28 28 0 0 0 200 172
    L 200 84
    A 28 28 0 0 0 172 56
    L 84 56 Z
  " />
  <!-- Internal Constellation / Central Gateway with 4 subnet branches -->
  <!-- Center Core Node -->
  <circle cx="128" cy="128" r="22" fill="#000000" />
  <!-- Cross Connectors -->
  <rect x="121" y="80" width="14" height="96" rx="4" fill="#000000" />
  <rect x="80" y="121" width="96" height="14" rx="4" fill="#000000" />
  <!-- Subnet satellite nodes -->
  <circle cx="128" cy="76" r="12" fill="#000000" />
  <circle cx="128" cy="180" r="12" fill="#000000" />
  <circle cx="76" cy="128" r="12" fill="#000000" />
  <circle cx="180" cy="128" r="12" fill="#000000" />
</svg>
"""

# Let's also explore a geometric hexagon perimeter (B2) with interlocking topology:
concept_b2_svg = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-b">
  <title id="title-b">MyLAN Logo Concept B - Hex Perimeter</title>
  <!-- Hexagonal outer boundary shielding a private local network -->
  <path fill="#000000" fill-rule="evenodd" d="
    M 128 28
    L 214 78
    L 214 178
    L 128 228
    L 42 178
    L 42 78 Z

    M 128 56
    L 66 92
    L 66 164
    L 128 200
    L 190 164
    L 190 92 Z
  " />
  <!-- Central Gateway Hub -->
  <circle cx="128" cy="128" r="24" fill="#000000" />
  <!-- Radiating 3-way distribution to vertices (Y-network) -->
  <path fill="#000000" d="
    M 121 128 L 121 68 L 135 68 L 135 128 Z
    M 128 124 L 176 152 L 169 164 L 121 136 Z
    M 128 124 L 80 152 L 87 164 L 135 136 Z
  " />
  <circle cx="128" cy="68" r="10" fill="#000000" />
  <circle cx="176" cy="154" r="10" fill="#000000" />
  <circle cx="80" cy="154" r="10" fill="#000000" />
</svg>
"""

with open(f"{OUT_DIR}/concept-b-v1.svg", "w") as f:
    f.write(concept_b_svg)
with open(f"{OUT_DIR}/concept-b-v2.svg", "w") as f:
    f.write(concept_b2_svg)

# -------------------------------------------------------------------------
# Concept C: Subnet Scope (Pictorial / Discovery Lens)
# Concentric active discovery radar / scanning aperture framing central host beacon.
# -------------------------------------------------------------------------
# Let's build a scanning aperture with concentric arcs and focal node:
concept_c_svg = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-c">
  <title id="title-c">MyLAN Logo Concept C - Subnet Scope</title>
  <!-- Central host beacon -->
  <circle cx="128" cy="128" r="20" fill="#000000" />

  <!-- Arc 1: Inner scan ring (r=52, stroke 16) -> drawn as path -->
  <path fill="#000000" fill-rule="evenodd" d="
    M 128 68
    A 60 60 0 1 1 68 128
    L 84 128
    A 44 44 0 1 0 128 84
    Z
  " />

  <!-- Arc 2: Middle discovery wave (r=86, stroke 16) -->
  <path fill="#000000" fill-rule="evenodd" d="
    M 128 34
    A 94 94 0 0 1 222 128
    L 206 128
    A 78 78 0 0 0 128 50
    Z
  " />

  <!-- Arc 3: Outer perimeter bracket (r=118, stroke 16) -->
  <path fill="#000000" fill-rule="evenodd" d="
    M 128 10
    A 118 118 0 0 1 246 128
    L 230 128
    A 102 102 0 0 0 128 26
    Z
  " />

  <!-- Lower left pulse arc -->
  <path fill="#000000" fill-rule="evenodd" d="
    M 50 128
    A 78 78 0 0 0 128 206
    L 128 222
    A 94 94 0 0 1 34 128
    Z
  " />
</svg>
"""

# Let's also build an elegant aperture radar (C2) with quadrant sweep and focal target:
concept_c2_svg = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-c">
  <title id="title-c">MyLAN Logo Concept C - Subnet Scope</title>
  <!-- Central Local Host Node -->
  <circle cx="128" cy="128" r="24" fill="#000000" />

  <!-- First Concentric Discovery Wave (Radius 58..76, 3 quadrants) -->
  <path fill="#000000" fill-rule="evenodd" d="
    M 128 52
    A 76 76 0 1 1 52 128
    L 70 128
    A 58 58 0 1 0 128 70
    Z
  " />

  <!-- Second Concentric Discovery Wave (Radius 94..112, diagonal sweep from 45 deg to 225 deg) -->
  <path fill="#000000" fill-rule="evenodd" d="
    M 128 16
    A 112 112 0 0 1 240 128
    L 222 128
    A 94 94 0 0 0 128 34
    Z
  " />
  <path fill="#000000" fill-rule="evenodd" d="
    M 128 222
    A 94 94 0 0 0 34 128
    L 16 128
    A 112 112 0 0 1 128 240
    Z
  " />
</svg>
"""

# And an exceptionally balanced Radar Scope (C3) - concentric stepped sweep:
concept_c3_svg = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-c">
  <title id="title-c">MyLAN Logo Concept C - Subnet Scope</title>
  <!-- Center Core Node -->
  <circle cx="128" cy="128" r="22" fill="#000000" />

  <!-- Inner ring: 270 degree sweep -->
  <path fill="#000000" d="
    M 128 64
    A 64 64 0 1 1 64 128
    L 82 128
    A 46 46 0 1 0 128 82
    Z
  " />

  <!-- Outer ring: Staggered pulse quadrant -->
  <path fill="#000000" d="
    M 128 26
    A 102 102 0 0 1 230 128
    L 212 128
    A 84 84 0 0 0 128 44
    Z
  " />
  <path fill="#000000" d="
    M 44 128
    A 84 84 0 0 0 128 212
    L 128 230
    A 102 102 0 0 1 26 128
    Z
  " />
</svg>
"""

with open(f"{OUT_DIR}/concept-c-v1.svg", "w") as f:
    f.write(concept_c_svg)
with open(f"{OUT_DIR}/concept-c-v2.svg", "w") as f:
    f.write(concept_c2_svg)
with open(f"{OUT_DIR}/concept-c-v3.svg", "w") as f:
    f.write(concept_c3_svg)

print("Generated initial SVG concept variations.")
