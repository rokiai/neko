#!/usr/bin/env python3
"""Prepare supplied desktop artwork while preserving its transparent padding."""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageChops, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "resources" / "app-icon-artwork.png"
OUT = ROOT / "resources" / "app-icon-source.png"

CANVAS_SIZE = 1254
MASK_INSET = 40
MASK_RADIUS = 200


def main() -> None:
    artwork = Image.open(SOURCE).convert("RGBA").resize(
        (CANVAS_SIZE, CANVAS_SIZE), Image.LANCZOS
    )
    mask = Image.new("L", (CANVAS_SIZE, CANVAS_SIZE))
    ImageDraw.Draw(mask).rounded_rectangle(
        (MASK_INSET, MASK_INSET, CANVAS_SIZE - MASK_INSET, CANVAS_SIZE - MASK_INSET),
        radius=MASK_RADIUS,
        fill=255,
    )
    artwork.putalpha(ImageChops.multiply(artwork.getchannel("A"), mask))
    artwork.save(OUT, format="PNG", optimize=True)
    print(f"wrote {OUT.relative_to(ROOT)} ({CANVAS_SIZE}x{CANVAS_SIZE})")


if __name__ == "__main__":
    main()
