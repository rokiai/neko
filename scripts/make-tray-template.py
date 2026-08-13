#!/usr/bin/env python3
"""Draw the macOS menu bar template icon (black glyph + alpha) for Neko.

macOS renders status bar images as template images: only the alpha channel is
used, and AppKit recolors the glyph for light/dark menu bars. The full-color
app icon cannot be reused here, it turns into an unreadable smudge at 18pt.

Usage: python3 scripts/make-tray-template.py
Requires Pillow. Output: src-tauri/icons/tray-template.png (36x36 = 18pt @2x).
"""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "src-tauri" / "icons" / "tray-template.png"

SIZE = 36
SUPERSAMPLE = 16


def draw_cat(draw: ImageDraw.ImageDraw, s: int) -> None:
    def box(x0: float, y0: float, x1: float, y1: float) -> list[float]:
        return [x0 * s, y0 * s, x1 * s, y1 * s]

    def poly(points: list[tuple[float, float]]) -> list[tuple[float, float]]:
        return [(x * s, y * s) for x, y in points]

    draw.polygon(poly([(6.2, 18.0), (8.6, 4.2), (17.4, 12.0)]), fill=255)
    draw.polygon(poly([(29.8, 18.0), (27.4, 4.2), (18.6, 12.0)]), fill=255)
    draw.ellipse(box(5.2, 11.0, 30.8, 31.6), fill=255)

    draw.ellipse(box(11.6, 19.0, 15.4, 23.6), fill=0)
    draw.ellipse(box(20.6, 19.0, 24.4, 23.6), fill=0)
    draw.polygon(poly([(16.6, 25.2), (19.4, 25.2), (18.0, 27.2)]), fill=0)


def main() -> None:
    canvas = Image.new("L", (SIZE * SUPERSAMPLE, SIZE * SUPERSAMPLE), 0)
    draw_cat(ImageDraw.Draw(canvas), SUPERSAMPLE)
    alpha = canvas.resize((SIZE, SIZE), Image.LANCZOS)

    icon = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    icon.putalpha(alpha)
    OUT.parent.mkdir(parents=True, exist_ok=True)
    icon.save(OUT, format="PNG", optimize=True)
    print(f"wrote {OUT.relative_to(ROOT)} ({SIZE}x{SIZE})")


if __name__ == "__main__":
    main()
