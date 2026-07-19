#!/usr/bin/env python3
"""Remove near-white edge-connected background from icon.png (true alpha)."""

from __future__ import annotations

from collections import deque
from pathlib import Path

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "resources" / "icon.png"
BUILD = ROOT / "build"


def is_bg_like(rgb: np.ndarray, y: int, x: int, tol: int = 28) -> bool:
    r, g, b = map(int, rgb[y, x])
    if r >= 255 - tol and g >= 255 - tol and b >= 255 - tol:
        return True
    if min(r, g, b) >= 230 and max(r, g, b) - min(r, g, b) <= 12:
        return True
    return False


def remove_edge_background(im: Image.Image) -> Image.Image:
    arr = np.array(im.convert("RGBA"))
    h, w = arr.shape[:2]
    rgb = arr[:, :, :3].astype(np.int16)
    mask = np.zeros((h, w), dtype=bool)
    q: deque[tuple[int, int]] = deque()

    for x in range(w):
        for y in (0, h - 1):
            if is_bg_like(rgb, y, x):
                mask[y, x] = True
                q.append((y, x))
    for y in range(h):
        for x in (0, w - 1):
            if is_bg_like(rgb, y, x) and not mask[y, x]:
                mask[y, x] = True
                q.append((y, x))

    while q:
        y, x = q.popleft()
        for dy, dx in ((-1, 0), (1, 0), (0, -1), (0, 1)):
            ny, nx = y + dy, x + dx
            if 0 <= ny < h and 0 <= nx < w and not mask[ny, nx] and is_bg_like(rgb, ny, nx):
                mask[ny, nx] = True
                q.append((ny, nx))

    # Clean anti-aliased fringe next to already-removed bg
    for _ in range(3):
        ys, xs = np.where(mask)
        extra: list[tuple[int, int]] = []
        for y, x in zip(ys.tolist(), xs.tolist()):
            for dy in (-1, 0, 1):
                for dx in (-1, 0, 1):
                    ny, nx = y + dy, x + dx
                    if 0 <= ny < h and 0 <= nx < w and not mask[ny, nx]:
                        r, g, b = map(int, rgb[ny, nx])
                        if min(r, g, b) >= 220 and max(r, g, b) - min(r, g, b) <= 18:
                            extra.append((ny, nx))
        if not extra:
            break
        for y, x in extra:
            mask[y, x] = True

    arr[:, :, 3] = np.where(mask, 0, 255)
    # Zero RGB on fully transparent pixels (avoids halo when scaling)
    arr[mask, 0:3] = 0

    print(f"bg removed: {int(mask.sum())} px ({100 * mask.mean():.1f}%)")
    print(f"corner alpha: {arr[0, 0, 3]}, center alpha: {arr[h // 2, w // 2, 3]}")
    white_kept = int(
        (
            (arr[:, :, 0] > 240)
            & (arr[:, :, 1] > 240)
            & (arr[:, :, 2] > 240)
            & (arr[:, :, 3] > 0)
        ).sum()
    )
    print(f"white fur pixels kept: {white_kept}")
    return Image.fromarray(arr, "RGBA")


def write_ico(src: Image.Image, path: Path) -> None:
    sizes = [16, 24, 32, 48, 64, 128, 256]
    imgs = [src.resize((s, s), Image.Resampling.LANCZOS) for s in sizes]
    imgs[-1].save(
        path,
        format="ICO",
        sizes=[(s, s) for s in sizes],
        append_images=imgs[:-1],
    )


def main() -> None:
    out = remove_edge_background(Image.open(SRC))
    out.save(SRC, format="PNG", optimize=True)
    print(f"saved {SRC} ({SRC.stat().st_size} bytes)")

    BUILD.mkdir(parents=True, exist_ok=True)
    icon1024 = out if out.size == (1024, 1024) else out.resize((1024, 1024), Image.Resampling.LANCZOS)
    icon1024.save(BUILD / "icon.png", format="PNG", optimize=True)
    write_ico(icon1024, BUILD / "icon.ico")
    print(f"saved {BUILD / 'icon.png'} and icon.ico")


if __name__ == "__main__":
    main()
