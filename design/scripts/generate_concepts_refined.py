#!/usr/bin/env python3
"""
Refine and craft world-class versions of the 3 logo concepts for MyLAN.
"""
import os
import math

OUT_DIR = "design/branding"

# =========================================================================
# CONCEPT A: "Nexus M" (Letterform Monogram)
# An iconic, architectural 'M' built with precise network conduit paths.
# Let's create an 'M' whose lines are continuous, perfectly proportioned,
# with chamfered 45-degree apex and integrated gateway node.
# =========================================================================

# Variant A-v4: Continuous precision monoline circuit M with filled outline
# Left stem, central chevron descending to y=152, right stem, with smooth chamfers/radii
# and a focal gateway node nested in the central chevron.
concept_a_v4 = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-a">
  <title id="title-a">MyLAN Logo - Nexus M</title>
  <!-- Main architectural M formed by precision polygonal geometry -->
  <path fill="#000000" d="
    M 40 52
    L 76 52
    L 76 136
    L 128 84
    L 180 136
    L 180 52
    L 216 52
    L 216 204
    L 180 204
    L 180 176
    L 128 124
    L 76 176
    L 76 204
    L 40 204
    Z
  " />
  <!-- Central Local Host / Router Gateway Node -->
  <circle cx="128" cy="180" r="16" fill="#000000" />
  <!-- Connecting data conduit between M and core node -->
  <rect x="122" y="140" width="12" height="28" fill="#000000" rx="3" />
</svg>
"""

# Variant A-v5: A bold, modern geometric M with integrated negative-space circuit routes
# Think Stripe / Vercel / Linear quality:
concept_a_v5 = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-a">
  <title id="title-a">MyLAN Logo - Nexus M</title>
  <!-- Three precision vertical pillars with 45-deg angled cross-links -->
  <!-- Left Pillar -->
  <path fill="#000000" d="M 40 48 L 76 48 L 76 208 L 40 208 Z" />
  <!-- Right Pillar -->
  <path fill="#000000" d="M 180 48 L 216 48 L 216 208 L 180 208 Z" />
  <!-- Center Diagonal Beams meeting at a crisp apex -->
  <path fill="#000000" d="
    M 76 68
    L 128 120
    L 180 68
    L 180 104
    L 128 156
    L 76 104
    Z
  " />
  <!-- Floating bottom node / anchor -->
  <polygon fill="#000000" points="128,172 146,190 128,208 110,190" />
</svg>
"""

# Variant A-v6: Folded Ribbon / Network Loop M (One continuous monoline stroke converted to clean shape)
# Stem width = 28. Perfectly balanced.
concept_a_v6 = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-a">
  <title id="title-a">MyLAN Logo - Nexus M</title>
  <path fill="#000000" d="
    M 44 204
    L 44 72
    A 20 20 0 0 1 64 52
    A 20 20 0 0 1 80 60
    L 128 120
    L 176 60
    A 20 20 0 0 1 192 52
    A 20 20 0 0 1 212 72
    L 212 204
    L 184 204
    L 184 84
    L 138 142
    A 16 16 0 0 1 118 142
    L 72 84
    L 72 204
    Z
  " />
  <!-- A central precision circular beacon at the base -->
  <circle cx="128" cy="188" r="16" fill="#000000" />
</svg>
"""

with open(f"{OUT_DIR}/concept-a-v4.svg", "w") as f:
    f.write(concept_a_v4)
with open(f"{OUT_DIR}/concept-a-v5.svg", "w") as f:
    f.write(concept_a_v5)
with open(f"{OUT_DIR}/concept-a-v6.svg", "w") as f:
    f.write(concept_a_v6)

# =========================================================================
# CONCEPT B: "Local Perimeter" (Abstract / Topology Boundary)
# A protective shield / polygon boundary with an authentic internal network mesh.
# "Tu red, bajo control"
# =========================================================================

# Variant B-v3: A modern rounded hexagonal crest with an embedded star/diamond network topology
concept_b_v3 = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-b">
  <title id="title-b">MyLAN Logo - Local Perimeter</title>
  <!-- Outer Shield Boundary with negative space channel -->
  <path fill="#000000" fill-rule="evenodd" d="
    M 128 28
    L 216 78
    L 216 178
    L 128 228
    L 40 178
    L 40 78
    Z
    M 128 52
    L 196 90
    L 196 166
    L 128 204
    L 60 166
    L 60 90
    Z
  " />
  <!-- Central Gateway Hub -->
  <circle cx="128" cy="128" r="22" fill="#000000" />
  <!-- Connected LAN Nodes at cardinal vertices of the inner space -->
  <!-- Top Gateway Node -->
  <circle cx="128" cy="80" r="12" fill="#000000" />
  <!-- Bottom Node -->
  <circle cx="128" cy="176" r="12" fill="#000000" />
  <!-- Left Node -->
  <circle cx="86" cy="128" r="12" fill="#000000" />
  <!-- Right Node -->
  <circle cx="170" cy="128" r="12" fill="#000000" />
  
  <!-- Interconnecting Traces / Bus lines (thickness 10) -->
  <rect x="123" y="80" width="10" height="96" fill="#000000" />
  <rect x="86" y="123" width="84" height="10" fill="#000000" />
  
  <!-- Perimeter link from Top Node to outer frame -->
  <rect x="124" y="48" width="8" height="34" fill="#000000" />
</svg>
"""

# Variant B-v4: Continuous Interlocking Circuit Shield (Solid, bold, unforgettable)
# The boundary itself weaves into a local node constellation.
concept_b_v4 = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-b">
  <title id="title-b">MyLAN Logo - Local Perimeter</title>
  <!-- Bold Protective Squircle Shield with Negative Space Network Cross -->
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
    
    <!-- Central Network Node Cutout (negative space circular hub) -->
    M 128 98
    A 30 30 0 1 0 128.1 98
    Z
    
    <!-- Negative space routing channels -->
    M 122 56 L 134 56 L 134 98 L 122 98 Z
    M 122 158 L 134 158 L 134 200 L 122 200 Z
    M 56 122 L 98 122 L 98 134 L 56 134 Z
    M 158 122 L 200 122 L 200 134 L 158 134 Z
  " />
  <!-- Solid Core Local Gateway in the center -->
  <circle cx="128" cy="128" r="16" fill="#000000" />
</svg>
"""

with open(f"{OUT_DIR}/concept-b-v3.svg", "w") as f:
    f.write(concept_b_v3)
with open(f"{OUT_DIR}/concept-b-v4.svg", "w") as f:
    f.write(concept_b_v4)

# =========================================================================
# CONCEPT C: "Subnet Scope" (Pictorial / Discovery Radar)
# Active network discovery: ARP scan, port scan, ping sweep.
# A disciplined, high-tech radar scope framing the local network beacon.
# =========================================================================

# Variant C-v4: Precision Technical Scope with Crosshairs and Staggered Scanning Arcs
concept_c_v4 = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-c">
  <title id="title-c">MyLAN Logo - Subnet Scope</title>
  <!-- Center Local Beacon Node -->
  <circle cx="128" cy="128" r="20" fill="#000000" />
  
  <!-- Outer Scope Reticle: 4 corner brackets forming a technical aperture -->
  <!-- Top-Right Reticle -->
  <path fill="#000000" d="
    M 128 32
    A 96 96 0 0 1 224 128
    L 208 128
    A 80 80 0 0 0 128 48
    Z
  " />
  <!-- Bottom-Left Reticle -->
  <path fill="#000000" d="
    M 128 224
    A 96 96 0 0 1 32 128
    L 48 128
    A 80 80 0 0 0 128 208
    Z
  " />
  
  <!-- Inner Discovery Wave (radius 52..66) sweeping top-left quadrant -->
  <path fill="#000000" d="
    M 128 64
    A 64 64 0 0 0 64 128
    L 78 128
    A 50 50 0 0 1 128 78
    Z
  " />
  
  <!-- Subnet target blips (discovered devices on the scope) -->
  <circle cx="184" cy="72" r="10" fill="#000000" />
  <circle cx="72" cy="184" r="8" fill="#000000" />
</svg>
"""

# Variant C-v5: Pure Geometric Radar Sweep - Iconic, symmetrical, timeless
concept_c_v5 = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-c">
  <title id="title-c">MyLAN Logo - Subnet Scope</title>
  <!-- Central Gateway Hub -->
  <circle cx="128" cy="128" r="22" fill="#000000" />
  
  <!-- Tier 1 Subnet Wave: Half ring (top and right) -->
  <path fill="#000000" d="
    M 128 74
    A 54 54 0 0 1 182 128
    L 198 128
    A 70 70 0 0 0 128 58
    Z
  " />
  
  <!-- Tier 2 Subnet Wave: Stepped quarter ring (top-right further out) -->
  <path fill="#000000" d="
    M 128 38
    A 90 90 0 0 1 218 128
    L 234 128
    A 106 106 0 0 0 128 22
    Z
  " />
  
  <!-- Tier 3 Counter-Wave / Perimeter Ring (bottom and left) -->
  <path fill="#000000" d="
    M 128 182
    A 54 54 0 0 1 74 128
    L 58 128
    A 70 70 0 0 0 128 198
    Z
  " />
  <path fill="#000000" d="
    M 128 218
    A 90 90 0 0 1 38 128
    L 22 128
    A 106 106 0 0 0 128 234
    Z
  " />
</svg>
"""

# Variant C-v6: Concentric Aperture with 3 Discovered Nodes (Radar + Topology)
concept_c_v6 = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-c">
  <title id="title-c">MyLAN Logo - Subnet Scope</title>
  <!-- Central Origin Beacon -->
  <circle cx="128" cy="128" r="24" fill="#000000" />
  
  <!-- Concentric Scanner Ring 1 (thickness 16) -->
  <path fill="#000000" fill-rule="evenodd" d="
    M 128 62
    A 66 66 0 1 1 62 128
    L 78 128
    A 50 50 0 1 0 128 78
    Z
  " />
  
  <!-- Outer Concentric Scanner Ring 2 (thickness 16, with 60 deg opening) -->
  <path fill="#000000" fill-rule="evenodd" d="
    M 128 24
    A 104 104 0 1 1 24 128
    L 40 128
    A 88 88 0 1 0 128 40
    Z
  " />
  
  <!-- Discovered Device Node in the scan opening -->
  <circle cx="196" cy="60" r="14" fill="#000000" />
</svg>
"""

with open(f"{OUT_DIR}/concept-c-v4.svg", "w") as f:
    f.write(concept_c_v4)
with open(f"{OUT_DIR}/concept-c-v5.svg", "w") as f:
    f.write(concept_c_v5)
with open(f"{OUT_DIR}/concept-c-v6.svg", "w") as f:
    f.write(concept_c_v6)

print("Generated refined variants v4, v5, v6.")
