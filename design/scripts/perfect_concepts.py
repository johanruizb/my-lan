#!/usr/bin/env python3
"""
Generate perfected candidate logos for MyLAN.
Zero comments in 'd' attributes.
Pure math, clean viewBox 0 0 256 256.
"""
import os

OUT_DIR = "design/branding"
os.makedirs(OUT_DIR, exist_ok=True)

# -------------------------------------------------------------------------
# CONCEPT A: "Nexus M" (Letterform Monogram)
# Pure architectural geometric M with integrated router vertex and conduit channels.
# -------------------------------------------------------------------------
concept_a_v7 = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-a">
  <title id="title-a">MyLAN Logo - Nexus M</title>
  <!-- Architectural M with precision 45-deg angles and central nexus aperture -->
  <path fill="#000000" fill-rule="evenodd" d="
    M 40 48
    L 76 48
    L 76 132
    L 128 84
    L 180 132
    L 180 48
    L 216 48
    L 216 208
    L 180 208
    L 180 168
    L 128 120
    L 76 168
    L 76 208
    L 40 208
    Z
    M 128 152
    A 16 16 0 1 1 127.9 152
    Z
  " />
</svg>
"""

# Concept A variant 8: Bold solid M where the chevron extends to the bottom and holds a router node
concept_a_v8 = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-a">
  <title id="title-a">MyLAN Logo - Nexus M</title>
  <!-- Symmetrical geometric M with continuous routing lines -->
  <path fill="#000000" d="
    M 38 48
    L 76 48
    L 76 116
    L 128 168
    L 180 116
    L 180 48
    L 218 48
    L 218 208
    L 180 208
    L 180 156
    L 128 208
    L 76 156
    L 76 208
    L 38 208
    Z
  " />
  <!-- Central Local Gateway Hub in the upper valley -->
  <circle cx="128" cy="92" r="18" fill="#000000" />
</svg>
"""

# Concept A variant 9: M with integrated circuit routes and center nexus
concept_a_v9 = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-a">
  <title id="title-a">MyLAN Logo - Nexus M</title>
  <!-- Left Pillar -->
  <rect x="40" y="48" width="34" height="160" rx="4" fill="#000000" />
  <!-- Right Pillar -->
  <rect x="182" y="48" width="34" height="160" rx="4" fill="#000000" />
  <!-- Center Diagonal Chevron -->
  <path fill="#000000" d="
    M 74 62
    L 128 116
    L 182 62
    L 182 98
    L 128 152
    L 74 98
    Z
  " />
  <!-- Center Nexus Node Hub -->
  <circle cx="128" cy="184" r="18" fill="#000000" />
  <circle cx="128" cy="184" r="7" fill="#ffffff" />
</svg>
"""

# -------------------------------------------------------------------------
# CONCEPT B: "Subnet Shield" (Abstract / Topology Boundary)
# The local perimeter protecting all discovered subnet devices.
# -------------------------------------------------------------------------
# Shield with 3-node network constellation inside (pure fill-rule="evenodd")
concept_b_v5 = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-b">
  <title id="title-b">MyLAN Logo - Subnet Shield</title>
  <!-- Outer Shield Boundary -->
  <path fill="#000000" fill-rule="evenodd" d="
    M 128 30
    C 180 30 218 46 218 96
    C 218 164 154 208 128 226
    C 102 208 38 164 38 96
    C 38 46 76 30 128 30
    Z
    
    M 128 54
    C 88 54 60 66 60 102
    C 60 152 110 188 128 202
    C 146 188 196 152 196 102
    C 196 66 168 54 128 54
    Z
  " />
  <!-- Central Gateway Hub -->
  <circle cx="128" cy="116" r="22" fill="#000000" />
  <!-- Local Subnet Satellite Nodes -->
  <circle cx="128" cy="74" r="11" fill="#000000" />
  <circle cx="86" cy="148" r="11" fill="#000000" />
  <circle cx="170" cy="148" r="11" fill="#000000" />
  
  <!-- Network Bus Lines -->
  <rect x="123" y="74" width="10" height="44" fill="#000000" />
  <path fill="#000000" d="M 124 116 L 86 148 L 93 156 L 131 124 Z" />
  <path fill="#000000" d="M 132 116 L 170 148 L 163 156 L 125 124 Z" />
</svg>
"""

# Concept B variant 6: Geometric Hexagonal Shield with interwoven LAN paths
concept_b_v6 = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-b">
  <title id="title-b">MyLAN Logo - Subnet Shield</title>
  <!-- Outer Hexagonal Boundary -->
  <path fill="#000000" fill-rule="evenodd" d="
    M 128 28
    L 218 80
    L 218 176
    L 128 228
    L 38 176
    L 38 80
    Z
    
    M 128 54
    L 194 92
    L 194 164
    L 128 202
    L 62 164
    L 62 92
    Z
  " />
  <!-- Central Local Gateway Hub -->
  <circle cx="128" cy="128" r="24" fill="#000000" />
  <circle cx="128" cy="128" r="10" fill="#ffffff" />
  
  <!-- 4 Orthogonal Satellite Nodes -->
  <circle cx="128" cy="80" r="12" fill="#000000" />
  <circle cx="128" cy="176" r="12" fill="#000000" />
  <circle cx="84" cy="128" r="12" fill="#000000" />
  <circle cx="172" cy="128" r="12" fill="#000000" />
  
  <!-- Bus Conduits -->
  <rect x="123" y="80" width="10" height="96" fill="#000000" />
  <rect x="84" y="123" width="88" height="10" fill="#000000" />
</svg>
"""

# -------------------------------------------------------------------------
# CONCEPT C: "Discovery Scope" (Pictorial / Local Radar)
# Active network discovery: ARP scan, port scan, ping sweep.
# -------------------------------------------------------------------------
# C variant 7: Technical Reticle Viewfinder with Concentric Scanning Pulses
concept_c_v7 = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-c">
  <title id="title-c">MyLAN Logo - Discovery Scope</title>
  <!-- Central Local Beacon Node -->
  <circle cx="128" cy="128" r="22" fill="#000000" />
  
  <!-- Viewfinder Frame Brackets (thickness 18, radius 16) -->
  <!-- Top-Left -->
  <path fill="#000000" d="M 36 104 L 36 56 A 20 20 0 0 1 56 36 L 104 36 L 104 54 L 56 54 A 2 2 0 0 0 54 56 L 54 104 Z" />
  <!-- Top-Right -->
  <path fill="#000000" d="M 220 104 L 220 56 A 20 20 0 0 0 200 36 L 152 36 L 152 54 L 200 54 A 2 2 0 0 1 202 56 L 202 104 Z" />
  <!-- Bottom-Left -->
  <path fill="#000000" d="M 36 152 L 36 200 A 20 20 0 0 0 56 220 L 104 220 L 104 202 L 56 202 A 2 2 0 0 1 54 200 L 54 152 Z" />
  <!-- Bottom-Right -->
  <path fill="#000000" d="M 220 152 L 220 200 A 20 20 0 0 1 200 220 L 152 220 L 152 202 L 200 202 A 2 2 0 0 0 202 200 L 202 152 Z" />
  
  <!-- Inner Discovery Wave (radius 50..66) in upper-right quadrant -->
  <path fill="#000000" d="
    M 128 70
    A 58 58 0 0 1 186 128
    L 170 128
    A 42 42 0 0 0 128 86
    Z
  " />
  <!-- Outer Discovery Wave (radius 78..94) in upper-right quadrant -->
  <path fill="#000000" d="
    M 128 44
    A 84 84 0 0 1 212 128
    L 196 128
    A 68 68 0 0 0 128 60
    Z
  " />
  
  <!-- Discovered Device Node in lower-left sector -->
  <circle cx="86" cy="170" r="12" fill="#000000" />
</svg>
"""

# C variant 8: Circular Precision Scope with Concentric Pulses
concept_c_v8 = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-c">
  <title id="title-c">MyLAN Logo - Discovery Scope</title>
  <!-- Central Origin Node -->
  <circle cx="128" cy="128" r="22" fill="#000000" />
  
  <!-- Outer Circular Scope Frame with 4 Crosshair Breaks -->
  <path fill="#000000" d="M 138 28 A 100 100 0 0 1 228 118 L 212 118 A 84 84 0 0 0 138 44 Z" />
  <path fill="#000000" d="M 228 138 A 100 100 0 0 1 138 228 L 138 212 A 84 84 0 0 0 212 138 Z" />
  <path fill="#000000" d="M 118 228 A 100 100 0 0 1 28 138 L 44 138 A 84 84 0 0 0 118 212 Z" />
  <path fill="#000000" d="M 28 118 A 100 100 0 0 1 118 28 L 118 44 A 84 84 0 0 0 44 118 Z" />
  
  <!-- Wave 1 (Inner Scan Arc) -->
  <path fill="#000000" d="M 128 72 A 56 56 0 0 1 184 128 L 168 128 A 40 40 0 0 0 128 88 Z" />
  <!-- Wave 2 (Outer Scan Arc) -->
  <path fill="#000000" d="M 128 48 A 80 80 0 0 1 208 128 L 192 128 A 64 64 0 0 0 128 64 Z" />
  
  <!-- Target Blip (Detected Subnet Device) -->
  <circle cx="86" cy="170" r="11" fill="#000000" />
</svg>
"""

with open(f"{OUT_DIR}/concept-a-v7.svg", "w") as f:
    f.write(concept_a_v7)
with open(f"{OUT_DIR}/concept-a-v8.svg", "w") as f:
    f.write(concept_a_v8)
with open(f"{OUT_DIR}/concept-a-v9.svg", "w") as f:
    f.write(concept_a_v9)
with open(f"{OUT_DIR}/concept-b-v5.svg", "w") as f:
    f.write(concept_b_v5)
with open(f"{OUT_DIR}/concept-b-v6.svg", "w") as f:
    f.write(concept_b_v6)
with open(f"{OUT_DIR}/concept-c-v7.svg", "w") as f:
    f.write(concept_c_v7)
with open(f"{OUT_DIR}/concept-c-v8.svg", "w") as f:
    f.write(concept_c_v8)

print("Generated perfected candidates v7, v8, v9, b-v5, b-v6, c-v7, c-v8.")
