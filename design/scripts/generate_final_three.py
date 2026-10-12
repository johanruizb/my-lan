#!/usr/bin/env python3
"""
Generate and perfect the 3 final logo concepts for MyLAN:
- Concept A: Nexus M (Letterform Monogram)
- Concept B: Subnet Shield (Abstract / Protected Perimeter)
- Concept C: Discovery Scope (Pictorial / Local Radar)

All built with pure mathematical SVG paths, viewBox 0 0 256 256,
single color #000000, genuine negative space (fill-rule="evenodd"),
no strokes, no text, 100% production ready.
"""

import os
import math

OUT_DIR = "design/branding"

# =========================================================================
# CONCEPT A: "Nexus M" (Letterform Monogram)
# Monogram 'M' where the diagonal pathways converge into an integrated
# router/gateway hub with negative space core.
# =========================================================================
# Construction:
# Width: from x=36 to x=220 (width 184).
# Height: from y=50 to y=206 (height 156).
# Perfectly centered: x midpoint = 128, y midpoint = 128.
# Left stem: x 36..72, y 50..206
# Right stem: x 184..220, y 50..206
# Center Hub: circle at (128, 142) with radius 28, inner hole r=12.
# Diagonals: thickness 32, connecting top-inner corners of stems to center hub.
# All united into a single seamless compound path.

concept_a = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-a">
  <title id="title-a">MyLAN Logo - Nexus M</title>
  <!-- Unified Monogram M with integrated central routing hub -->
  <path fill="#000000" fill-rule="evenodd" d="
    M 36 50
    L 72 50
    L 72 96
    L 110 128
    A 26 26 0 0 0 128 122
    A 26 26 0 0 0 146 128
    L 184 96
    L 184 50
    L 220 50
    L 220 206
    L 184 206
    L 184 140
    L 146 172
    A 26 26 0 0 0 128 178
    A 26 26 0 0 0 110 172
    L 72 140
    L 72 206
    L 36 206
    Z

    M 128 138
    A 12 12 0 1 0 128.1 138
    Z
  " />
</svg>
"""

# Let's also create an alternative refined Concept A (A-sharp):
# Pure polygonal precision with architectural cuts and floating node:
concept_a_sharp = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-a">
  <title id="title-a">MyLAN Logo - Nexus M</title>
  <g fill="#000000">
    <!-- Left Pillar -->
    <path d="M 38 52 L 74 52 L 74 204 L 38 204 Z" />
    <!-- Right Pillar -->
    <path d="M 182 52 L 218 52 L 218 204 L 182 204 Z" />
    <!-- Center Chevron routing path -->
    <path d="
      M 74 72
      L 128 122
      L 182 72
      L 182 104
      L 128 154
      L 74 104
      Z
    " />
    <!-- Gateway Hub Node in negative space valley -->
    <circle cx="128" cy="184" r="16" />
    <!-- Conduit trace connecting chevron to hub -->
    <rect x="123" y="152" width="10" height="20" rx="2" />
  </g>
</svg>
"""

# =========================================================================
# CONCEPT B: "Subnet Shield" (Abstract / Topology Boundary)
# Autonomous private perimeter ("Tu red, bajo control").
# A modern geometric shield containing an internal topological constellation
# formed by negative space routing conduits and 4 client nodes.
# =========================================================================

concept_b = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-b">
  <title id="title-b">MyLAN Logo - Subnet Shield</title>
  <!-- Outer Shield Silhouette with negative-space local network mesh inside -->
  <path fill="#000000" fill-rule="evenodd" d="
    M 128 32
    C 176 32 216 46 216 94
    C 216 162 154 206 128 224
    C 102 206 40 162 40 94
    C 40 46 80 32 128 32
    Z

    M 128 108
    A 20 20 0 1 0 128.1 108
    Z

    M 128 66
    A 11 11 0 1 0 128.1 66
    Z
    M 84 124
    A 11 11 0 1 0 84.1 124
    Z
    M 172 124
    A 11 11 0 1 0 172.1 124
    Z
    M 128 174
    A 11 11 0 1 0 128.1 174
    Z

    M 124 77 L 132 77 L 132 108 L 124 108 Z
    M 124 148 L 132 148 L 132 163 L 124 163 Z
    M 95 124 L 108 124 L 108 132 L 95 132 Z
    M 148 124 L 161 124 L 161 132 L 148 132 Z
  " />

  <!-- Solid core gateway inside the central negative cutout -->
  <circle cx="128" cy="128" r="10" fill="#000000" />
</svg>
"""

# Alternative Concept B (B-hex): Modern chamfered hexagonal boundary
concept_b_hex = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-b">
  <title id="title-b">MyLAN Logo - Subnet Shield</title>
  <!-- Hexagonal boundary with clean internal network constellation -->
  <path fill="#000000" fill-rule="evenodd" d="
    M 128 26
    L 220 78
    L 220 178
    L 128 230
    L 36 178
    L 36 78
    Z

    M 128 54
    L 200 94
    L 200 162
    L 128 202
    L 56 162
    L 56 94
    Z
  " />

  <!-- Central Gateway Node with 3 connected satellite devices -->
  <circle cx="128" cy="128" r="22" fill="#000000" />

  <!-- Top Node (Internet / Gateway) -->
  <circle cx="128" cy="84" r="12" fill="#000000" />
  <!-- Bottom-Left Node (Desktop / Server) -->
  <circle cx="90" cy="154" r="12" fill="#000000" />
  <!-- Bottom-Right Node (Mobile / IoT) -->
  <circle cx="166" cy="154" r="12" fill="#000000" />

  <!-- Connecting bus traces -->
  <rect x="123" y="84" width="10" height="46" fill="#000000" />
  <path fill="#000000" d="M 124 128 L 90 154 L 96 162 L 130 136 Z" />
  <path fill="#000000" d="M 132 128 L 166 154 L 160 162 L 126 136 Z" />
</svg>
"""

# =========================================================================
# CONCEPT C: "Discovery Scope" (Pictorial / Local Radar)
# Active network discovery: ARP scan, port scan, ping sweep.
# A high-tech radar scope composed of 4 quadrant brackets framing concentric
# sweep pulses and a luminous center beacon.
# =========================================================================

concept_c = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-c">
  <title id="title-c">MyLAN Logo - Discovery Scope</title>
  <!-- Center Core Beacon (local host scanner) -->
  <circle cx="128" cy="128" r="22" fill="#000000" />

  <!-- Reticle Bracket Top-Left -->
  <path fill="#000000" d="
    M 36 112
    L 36 56
    A 20 20 0 0 1 56 36
    L 112 36
    L 112 56
    L 56 56
    L 56 112
    Z
  " />
  <!-- Reticle Bracket Top-Right -->
  <path fill="#000000" d="
    M 220 112
    L 220 56
    A 20 20 0 0 0 200 36
    L 144 36
    L 144 56
    L 200 56
    L 200 112
    Z
  " />
  <!-- Reticle Bracket Bottom-Left -->
  <path fill="#000000" d="
    M 36 144
    L 36 200
    A 20 20 0 0 0 56 220
    L 112 220
    L 112 200
    L 56 200
    L 56 144
    Z
  " />
  <!-- Reticle Bracket Bottom-Right -->
  <path fill="#000000" d="
    M 220 144
    L 220 200
    A 20 20 0 0 1 200 220
    L 144 220
    L 144 200
    L 200 200
    L 200 144
    Z
  " />

  <!-- Inner Discovery Wave Arc (radius 56..70, upper-right sweep) -->
  <path fill="#000000" d="
    M 128 72
    A 56 56 0 0 1 184 128
    L 170 128
    A 42 42 0 0 0 128 86
    Z
  " />

  <!-- Outer Discovery Wave Arc (radius 78..92, upper-right sweep) -->
  <path fill="#000000" d="
    M 128 48
    A 80 80 0 0 1 208 128
    L 194 128
    A 66 66 0 0 0 128 62
    Z
  " />

  <!-- Counter-balance pulse blip in lower-left quadrant (discovered device) -->
  <circle cx="84" cy="172" r="10" fill="#000000" />
</svg>
"""

# Alternative Concept C (C-lens): Circular precision scope
concept_c_lens = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-c">
  <title id="title-c">MyLAN Logo - Discovery Scope</title>
  <!-- Central Origin Beacon -->
  <circle cx="128" cy="128" r="22" fill="#000000" />

  <!-- Outer Reticle Ring with 4 Crosshair Breaks -->
  <!-- Top-Right Quadrant -->
  <path fill="#000000" d="
    M 138 28
    A 100 100 0 0 1 228 118
    L 214 118
    A 86 86 0 0 0 138 42
    Z
  " />
  <!-- Bottom-Right Quadrant -->
  <path fill="#000000" d="
    M 228 138
    A 100 100 0 0 1 138 228
    L 138 214
    A 86 86 0 0 0 214 138
    Z
  " />
  <!-- Bottom-Left Quadrant -->
  <path fill="#000000" d="
    M 118 228
    A 100 100 0 0 1 28 138
    L 42 138
    A 86 86 0 0 0 118 214
    Z
  " />
  <!-- Top-Left Quadrant -->
  <path fill="#000000" d="
    M 28 118
    A 100 100 0 0 1 118 28
    L 118 42
    A 86 86 0 0 0 42 118
    Z
  " />

  <!-- Inner Discovery Pulse Arc (radius 52..66) -->
  <path fill="#000000" d="
    M 128 68
    A 60 60 0 0 1 188 128
    L 174 128
    A 46 46 0 0 0 128 82
    Z
  " />

  <!-- Discovered Node Target -->
  <circle cx="178" cy="78" r="10" fill="#000000" />
</svg>
"""

with open(f"{OUT_DIR}/concept-a.svg", "w") as f:
    f.write(concept_a_sharp)
with open(f"{OUT_DIR}/concept-a-alt.svg", "w") as f:
    f.write(concept_a)
with open(f"{OUT_DIR}/concept-b.svg", "w") as f:
    f.write(concept_b)
with open(f"{OUT_DIR}/concept-b-alt.svg", "w") as f:
    f.write(concept_b_hex)
with open(f"{OUT_DIR}/concept-c.svg", "w") as f:
    f.write(concept_c_lens)
with open(f"{OUT_DIR}/concept-c-alt.svg", "w") as f:
    f.write(concept_c)

print("Saved primary concepts A, B, C and alternatives.")
