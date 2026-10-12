#!/usr/bin/env python3
"""
Build production-ready Master SVG files for MyLAN Discovery Scope Kit:
- final-symbol.svg (standard master, black)
- final-symbol-small.svg (16px/32px optical cut)
- final-symbol-color.svg (Electric Cyan on Deep Void)
- final-symbol-dark.svg (Electric Cyan for dark UI)
- final-horizontal.svg (symbol + wordmark lockup)
- final-horizontal-color.svg
- final-stacked.svg (stacked vertical lockup)
- final-wordmark.svg (standalone wordmark)
"""

import math
import os

OUT_DIR = "design/branding/kit"
os.makedirs(OUT_DIR, exist_ok=True)

# -------------------------------------------------------------------------
# Exact Mathematical Geometry for Discovery Scope
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

# Standard Symbol Master (Black)
final_symbol_svg = f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-symbol">
  <title id="title-symbol">MyLAN - Discovery Scope Symbol</title>
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

# Small-Size Optical Cut (Optimized for 16px - 32px favicon/app dock)
# Wider single pulse wave, bolder center node, omission of tiny secondary blip
small_wave = arc_quadrant(cx, cy, 50, 78, 9, 81)
final_symbol_small_svg = f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-small">
  <title id="title-small">MyLAN - Discovery Scope (Small Size Cut)</title>
  <!-- Bolder Central Beacon -->
  <circle cx="128" cy="128" r="26" fill="#000000" />
  
  <!-- 4 Reticle Quadrants -->
  <path fill="#000000" d="{q1}" />
  <path fill="#000000" d="{q2}" />
  <path fill="#000000" d="{q3}" />
  <path fill="#000000" d="{q4}" />
  
  <!-- High-Contrast Single Unified Sweep Wave -->
  <path fill="#000000" d="{small_wave}" />
</svg>
"""

# Color Symbol Master (Electric Cyan on Dark Tile)
final_symbol_color_svg = f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-color">
  <title id="title-color">MyLAN - Discovery Scope (Electric Cyan on Void Tile)</title>
  <rect width="256" height="256" rx="56" fill="#090D16" />
  <!-- Central Local Origin Beacon -->
  <circle cx="128" cy="128" r="22" fill="#06B6D4" />
  
  <!-- 4 Symmetrical Reticle Quadrants -->
  <path fill="#06B6D4" d="{q1}" opacity="0.9" />
  <path fill="#06B6D4" d="{q2}" opacity="0.9" />
  <path fill="#06B6D4" d="{q3}" opacity="0.9" />
  <path fill="#06B6D4" d="{q4}" opacity="0.9" />
  
  <!-- Radar Sweep Waves -->
  <path fill="#06B6D4" d="{wave1}" opacity="0.75" />
  <path fill="#06B6D4" d="{wave2}" opacity="0.5" />
  
  <!-- Discovered Host Blip -->
  <circle cx="{blip_x:.2f}" cy="{blip_y:.2f}" r="10" fill="#38BDF8" />
</svg>
"""

# Color Symbol Transparent (for dark backgrounds)
final_symbol_dark_svg = f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-labelledby="title-dark">
  <title id="title-dark">MyLAN - Discovery Scope Symbol (Dark Theme)</title>
  <!-- Central Local Origin Beacon -->
  <circle cx="128" cy="128" r="22" fill="#06B6D4" />
  
  <!-- 4 Symmetrical Reticle Quadrants -->
  <path fill="#06B6D4" d="{q1}" />
  <path fill="#06B6D4" d="{q2}" />
  <path fill="#06B6D4" d="{q3}" />
  <path fill="#06B6D4" d="{q4}" />
  
  <!-- Radar Sweep Waves -->
  <path fill="#06B6D4" d="{wave1}" opacity="0.8" />
  <path fill="#06B6D4" d="{wave2}" opacity="0.6" />
  
  <!-- Discovered Host Blip -->
  <circle cx="{blip_x:.2f}" cy="{blip_y:.2f}" r="10" fill="#38BDF8" />
</svg>
"""

# -------------------------------------------------------------------------
# Wordmark Geometry
# -------------------------------------------------------------------------
wordmark_paths = """
  <!-- Letter M (x: 270..350, width 80) -->
  <path fill="#000000" d="
    M 270 74 L 290 74 L 310 134 L 330 74 L 350 74 L 350 184 L 330 184 L 330 110 L 318 146 L 302 146 L 290 110 L 290 184 L 270 184 Z
  " />
  
  <!-- Letter y (x: 364..420, width 56) -->
  <path fill="#000000" d="
    M 364 106 L 384 106 L 396 144 L 408 106 L 428 106 L 404 184 C 398 202 388 214 372 214 L 364 214 L 364 196 L 372 196 C 380 196 386 190 388 180 L 390 170 Z
  " />
  
  <!-- Letter L (x: 434..486, width 52) -->
  <path fill="#000000" d="
    M 434 74 L 454 74 L 454 164 L 486 164 L 486 184 L 434 184 Z
  " />
  
  <!-- Letter A (x: 500..568, width 68) -->
  <path fill="#000000" fill-rule="evenodd" d="
    M 524 74 L 544 74 L 568 184 L 546 184 L 540 156 L 528 156 L 522 184 L 500 184 Z
    M 534 106 L 531 138 L 537 138 Z
  " />
  
  <!-- Letter N (x: 582..644, width 62) -->
  <path fill="#000000" d="
    M 582 74 L 602 74 L 624 146 L 624 74 L 644 74 L 644 184 L 624 184 L 602 112 L 602 184 L 582 184 Z
  " />
"""

# Horizontal Lockup Master
final_horizontal_svg = f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 680 256" width="680" height="256" role="img" aria-labelledby="title-h">
  <title id="title-h">MyLAN - Horizontal Lockup</title>
  <g id="symbol">
    <circle cx="128" cy="128" r="22" fill="#000000" />
    <path fill="#000000" d="{q1}" />
    <path fill="#000000" d="{q2}" />
    <path fill="#000000" d="{q3}" />
    <path fill="#000000" d="{q4}" />
    <path fill="#000000" d="{wave1}" />
    <path fill="#000000" d="{wave2}" />
    <circle cx="{blip_x:.2f}" cy="{blip_y:.2f}" r="10" fill="#000000" />
  </g>
  <g id="wordmark">
{wordmark_paths}
  </g>
</svg>
"""

# Horizontal Lockup Color (on dark)
final_horizontal_color_svg = f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 680 256" width="680" height="256" role="img" aria-labelledby="title-hc">
  <title id="title-hc">MyLAN - Horizontal Lockup (Electric Cyan)</title>
  <g id="symbol">
    <circle cx="128" cy="128" r="22" fill="#06B6D4" />
    <path fill="#06B6D4" d="{q1}" />
    <path fill="#06B6D4" d="{q2}" />
    <path fill="#06B6D4" d="{q3}" />
    <path fill="#06B6D4" d="{q4}" />
    <path fill="#06B6D4" d="{wave1}" opacity="0.8" />
    <path fill="#06B6D4" d="{wave2}" opacity="0.6" />
    <circle cx="{blip_x:.2f}" cy="{blip_y:.2f}" r="10" fill="#38BDF8" />
  </g>
  <g id="wordmark" fill="#FFFFFF">
    <path d="M 270 74 L 290 74 L 310 134 L 330 74 L 350 74 L 350 184 L 330 184 L 330 110 L 318 146 L 302 146 L 290 110 L 290 184 L 270 184 Z" />
    <path d="M 364 106 L 384 106 L 396 144 L 408 106 L 428 106 L 404 184 C 398 202 388 214 372 214 L 364 214 L 364 196 L 372 196 C 380 196 386 190 388 180 L 390 170 Z" />
    <path d="M 434 74 L 454 74 L 454 164 L 486 164 L 486 184 L 434 184 Z" />
    <path fill-rule="evenodd" d="M 524 74 L 544 74 L 568 184 L 546 184 L 540 156 L 528 156 L 522 184 L 500 184 Z M 534 106 L 531 138 L 537 138 Z" />
    <path d="M 582 74 L 602 74 L 624 146 L 624 74 L 644 74 L 644 184 L 624 184 L 602 112 L 602 184 L 582 184 Z" />
  </g>
</svg>
"""

# Stacked Lockup Master (256 x 340)
# Scaled symbol at top (center 128, 100, scale 0.75), wordmark centered at bottom (width 374 -> scaled to 200, scale 0.53)
final_stacked_svg = f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 320" width="256" height="320" role="img" aria-labelledby="title-s">
  <title id="title-s">MyLAN - Stacked Lockup</title>
  <!-- Symbol scaled and centered at top -->
  <g id="symbol" transform="translate(128, 104) scale(0.72) translate(-128, -128)">
    <circle cx="128" cy="128" r="22" fill="#000000" />
    <path fill="#000000" d="{q1}" />
    <path fill="#000000" d="{q2}" />
    <path fill="#000000" d="{q3}" />
    <path fill="#000000" d="{q4}" />
    <path fill="#000000" d="{wave1}" />
    <path fill="#000000" d="{wave2}" />
    <circle cx="{blip_x:.2f}" cy="{blip_y:.2f}" r="10" fill="#000000" />
  </g>
  <!-- Wordmark scaled and centered at bottom -->
  <g id="wordmark" transform="translate(128, 252) scale(0.52) translate(-457, -135)">
{wordmark_paths}
  </g>
</svg>
"""

# Standalone Wordmark (420 x 180)
final_wordmark_svg = f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 420 180" width="420" height="180" role="img" aria-labelledby="title-w">
  <title id="title-w">MyLAN - Wordmark</title>
  <g id="wordmark" transform="translate(-250, -54)">
{wordmark_paths}
  </g>
</svg>
"""

files_to_write = {
    "final-symbol.svg": final_symbol_svg,
    "final-symbol-small.svg": final_symbol_small_svg,
    "final-symbol-color.svg": final_symbol_color_svg,
    "final-symbol-dark.svg": final_symbol_dark_svg,
    "final-horizontal.svg": final_horizontal_svg,
    "final-horizontal-color.svg": final_horizontal_color_svg,
    "final-stacked.svg": final_stacked_svg,
    "final-wordmark.svg": final_wordmark_svg,
}

for fname, content in files_to_write.items():
    fpath = os.path.join(OUT_DIR, fname)
    with open(fpath, "w") as f:
        f.write(content)

print(f"Wrote {len(files_to_write)} master files to {OUT_DIR}.")
