#!/usr/bin/env python3
"""
Test iterations for Concept A (Unified Nexus M) and Concept B (Subnet Shield with Ethernet Bus).
"""
import os

OUT_DIR = "design/branding"

# Concept A - Unified: Monogram M whose central diagonals flow into a vertical drop stem with an anchor hub
concept_a_v10 = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-a">
  <title id="title-a">MyLAN Logo - Nexus M</title>
  <!-- Unified Monogram M with integrated central router drop and node -->
  <path fill="#000000" d="
    M 38 48
    L 74 48
    L 74 102
    L 122 150
    L 122 172
    A 16 16 0 1 0 134 172
    L 134 150
    L 182 102
    L 182 48
    L 218 48
    L 218 208
    L 182 208
    L 182 142
    L 128 196
    L 74 142
    L 74 208
    L 38 208
    Z
  " />
</svg>
"""

# Let's fix coordinates for A-v10 so the geometry is crisp:
# Left pillar: x 38..74, y 48..208
# Right pillar: x 182..218, y 48..208
# Diagonals from (74, 48) and (182, 48) meeting at center vertex (128, 140).
# From vertex (128, 140), a vertical conduit drops to a circular gateway node at (128, 184).
concept_a_v11 = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-a">
  <title id="title-a">MyLAN Logo - Nexus M</title>
  <!-- Left Pillar -->
  <rect x="38" y="48" width="34" height="160" rx="3" fill="#000000" />
  <!-- Right Pillar -->
  <rect x="184" y="48" width="34" height="160" rx="3" fill="#000000" />
  <!-- Center Chevron descending from top corners -->
  <path fill="#000000" d="
    M 72 48
    L 128 114
    L 184 48
    L 184 84
    L 136 140
    L 136 164
    A 20 20 0 1 1 120 164
    L 120 140
    L 72 84
    Z
  " />
  <!-- Negative space core in the node -->
  <circle cx="128" cy="180" r="8" fill="#ffffff" />
</svg>
"""

# Concept B - Subnet Shield with Ethernet / LAN Bus Architecture
# A solid geometric shield with negative-space LAN bus:
# A horizontal bus bar connecting 3 local client nodes.
concept_b_v7 = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-b">
  <title id="title-b">MyLAN Logo - Subnet Shield</title>
  <!-- Solid Shield Silhouette -->
  <path fill="#000000" fill-rule="evenodd" d="
    M 128 28
    C 182 28 220 44 220 94
    C 220 166 156 210 128 228
    C 100 210 36 166 36 94
    C 36 44 74 28 128 28
    Z
    
    <!-- Negative space Ethernet Bus / Subnet Architecture inside shield -->
    <!-- Gateway Hub (Top Center) -->
    M 128 66
    A 16 16 0 1 0 128.1 66
    Z
    
    <!-- Central Drop line from Gateway to Bus -->
    M 122 82 L 134 82 L 134 130 L 122 130 Z
    
    <!-- Horizontal Subnet Bus Bar -->
    M 72 126 L 184 126 L 184 136 L 72 136 Z
    
    <!-- Left Client Drop Line and Node -->
    M 78 136 L 86 136 L 86 156 L 78 156 Z
    M 82 154
    A 12 12 0 1 0 82.1 154
    Z
    
    <!-- Middle Client Drop Line and Node -->
    M 124 136 L 132 136 L 132 164 L 124 164 Z
    M 128 162
    A 12 12 0 1 0 128.1 162
    Z
    
    <!-- Right Client Drop Line and Node -->
    M 170 136 L 178 136 L 178 156 L 170 156 Z
    M 174 154
    A 12 12 0 1 0 174.1 154
    Z
  " />
  
  <!-- Solid Core inside Gateway Hub -->
  <circle cx="128" cy="82" r="7" fill="#000000" />
</svg>
"""

# Concept B variant 8: Modern Squircle Perimeter with negative-space LAN Bus
concept_b_v8 = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-b">
  <title id="title-b">MyLAN Logo - Subnet Perimeter</title>
  <!-- Outer Squircle Tile with rounded corners -->
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
    
    <!-- Gateway Router Node at top -->
    M 128 64
    A 18 18 0 1 0 128.1 64
    Z
    
    <!-- Drop Trunk from Router to Bus -->
    M 122 82 L 134 82 L 134 132 L 122 132 Z
    
    <!-- Horizontal LAN Bus -->
    M 68 126 L 188 126 L 188 138 L 68 138 Z
    
    <!-- Left Device Drop -->
    M 74 138 L 84 138 L 84 160 L 74 160 Z
    M 79 160
    A 14 14 0 1 0 79.1 160
    Z
    
    <!-- Center Device Drop -->
    M 123 138 L 133 138 L 133 168 L 123 168 Z
    M 128 168
    A 14 14 0 1 0 128.1 168
    Z
    
    <!-- Right Device Drop -->
    M 172 138 L 182 138 L 182 160 L 172 160 Z
    M 177 160
    A 14 14 0 1 0 177.1 160
    Z
  " />
  <!-- Center Core dot inside router -->
  <circle cx="128" cy="82" r="8" fill="#000000" />
</svg>
"""

with open(f"{OUT_DIR}/concept-a-v11.svg", "w") as f:
    f.write(concept_a_v11)
with open(f"{OUT_DIR}/concept-b-v7.svg", "w") as f:
    f.write(concept_b_v7)
with open(f"{OUT_DIR}/concept-b-v8.svg", "w") as f:
    f.write(concept_b_v8)

print("Generated a-v11, b-v7, b-v8.")
