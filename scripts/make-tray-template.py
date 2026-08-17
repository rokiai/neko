#!/usr/bin/env python3
"""Sync the supplied macOS menu-bar logo representations without altering them.

Usage: python3 scripts/make-tray-template.py
Requires Pillow. Outputs 1x, 2x, and 3x source-identical PNG representations.
"""

from __future__ import annotations

from pathlib import Path
from shutil import copyfile

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
ASSETS = (
    ("tray-template-1x.png", "tray-template.png", 22),
    ("tray-template-2x.png", "tray-template@2x.png", 44),
    ("tray-template-3x.png", "tray-template@3x.png", 66),
)


def main() -> None:
    output_dir = ROOT / "src-tauri" / "icons"
    output_dir.mkdir(parents=True, exist_ok=True)
    for source_name, output_name, size in ASSETS:
        source = ROOT / "resources" / source_name
        image = Image.open(source).convert("RGBA")
        if image.size != (size, size):
            raise ValueError(f"expected {source_name} to be {size}x{size}, got {image.size}")
        if image.getchannel("A").getbbox() is None:
            raise ValueError(f"tray logo source has no visible pixels: {source}")
        output = output_dir / output_name
        copyfile(source, output)
        print(f"wrote {output.relative_to(ROOT)} ({size}x{size})")


if __name__ == "__main__":
    main()
