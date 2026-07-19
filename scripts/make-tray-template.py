#!/usr/bin/env python3
"""Make trayTemplate-source transparent and regenerate trayTemplate icons."""

from __future__ import annotations

import base64
import re
from collections import deque
from pathlib import Path

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "resources" / "trayTemplate-source.png"
OUT_1X = ROOT / "resources" / "trayTemplate.png"
OUT_2X = ROOT / "resources" / "ivan.p@example.net"
TRAY_TS = ROOT / "src" / "main" / "lib" / "tray.ts"


def flood_bg_mask(rgb: np.ndarray, tol: int = 36) -> np.ndarray:
    h, w = rgb.shape[:2]
    mask = np.zeros((h, w), dtype=bool)

    def is_bg(y: int, x: int) -> bool:
        r, g, b = map(int, rgb[y, x])
        if min(r, g, b) >= 255 - tol:
            return True
        if min(r, g, b) >= 220 and max(r, g, b) - min(r, g, b) <= 20:
            return True
        return False

    q: deque[tuple[int, int]] = deque()
    for x in range(w):
        for y in (0, h - 1):
            if is_bg(y, x):
                mask[y, x] = True
                q.append((y, x))
    for y in range(h):
        for x in (0, w - 1):
            if not mask[y, x] and is_bg(y, x):
                mask[y, x] = True
                q.append((y, x))

    while q:
        y, x = q.popleft()
        for dy, dx in ((-1, 0), (1, 0), (0, -1), (0, 1)):
            ny, nx = y + dy, x + dx
            if 0 <= ny < h and 0 <= nx < w and not mask[ny, nx] and is_bg(ny, nx):
                mask[ny, nx] = True
                q.append((ny, nx))

    for _ in range(2):
        ys, xs = np.where(mask)
        extra: list[tuple[int, int]] = []
        for y, x in zip(ys.tolist(), xs.tolist()):
            for dy in (-1, 0, 1):
                for dx in (-1, 0, 1):
                    ny, nx = y + dy, x + dx
                    if 0 <= ny < h and 0 <= nx < w and not mask[ny, nx]:
                        r, g, b = map(int, rgb[ny, nx])
                        if min(r, g, b) >= 200 and max(r, g, b) - min(r, g, b) <= 24:
                            extra.append((ny, nx))
        for y, x in extra:
            mask[y, x] = True
    return mask


def to_transparent_art(im: Image.Image) -> Image.Image:
    arr = np.array(im.convert("RGBA"))
    rgb = arr[:, :, :3].astype(np.int16)
    bg = flood_bg_mask(rgb)

    lum = (0.299 * rgb[:, :, 0] + 0.587 * rgb[:, :, 1] + 0.114 * rgb[:, :, 2]).astype(
        np.float32
    )
    alpha = np.clip(255.0 - lum, 0, 255).astype(np.uint8)
    # Drop watermark / light noise; keep cup silhouette
    alpha[lum >= 200] = 0
    alpha[bg] = 0

    out = np.zeros_like(arr)
    out[:, :, 3] = alpha
    print(
        f"bg={100 * bg.mean():.1f}% opaque={100 * (alpha > 10).mean():.1f}% "
        f"corner={tuple(out[0, 0])} center={tuple(out[out.shape[0] // 2, out.shape[1] // 2])}"
    )
    return Image.fromarray(out, "RGBA")


def tight_crop(im: Image.Image, pad_ratio: float = 0.1) -> Image.Image:
    a = np.array(im.split()[-1])
    ys, xs = np.where(a > 10)
    if len(xs) == 0:
        return im
    y0, y1 = int(ys.min()), int(ys.max())
    x0, x1 = int(xs.min()), int(xs.max())
    side = max(y1 - y0 + 1, x1 - x0 + 1)
    pad = int(side * pad_ratio)
    cy = (y0 + y1) // 2
    cx = (x0 + x1) // 2
    half = side // 2 + pad
    left = max(0, cx - half)
    top = max(0, cy - half)
    right = min(im.width, cx + half)
    bottom = min(im.height, cy + half)
    crop = im.crop((left, top, right, bottom))
    s = max(crop.size)
    canvas = Image.new("RGBA", (s, s), (0, 0, 0, 0))
    canvas.paste(crop, ((s - crop.width) // 2, (s - crop.height) // 2))
    return canvas


def resize_template(im: Image.Image, size: int) -> Image.Image:
    scaled = im.resize((size, size), Image.Resampling.LANCZOS)
    arr = np.array(scaled)
    a = arr[:, :, 3].astype(np.float32)
    a = np.clip((a - 40) * (255.0 / (255 - 40)), 0, 255)
    a = np.where(a < 48, 0, np.where(a > 200, 255, a)).astype(np.uint8)
    arr[:, :, 0:3] = 0
    arr[:, :, 3] = a
    return Image.fromarray(arr, "RGBA")


def main() -> None:
    art = tight_crop(to_transparent_art(Image.open(SRC)))

    hi = art.resize((512, 512), Image.Resampling.LANCZOS)
    hi_arr = np.array(hi)
    hi_arr[:, :, 0:3] = 0
    Image.fromarray(hi_arr, "RGBA").save(SRC, format="PNG", optimize=True)

    icon1x = resize_template(art, 16)
    icon2x = resize_template(art, 32)
    icon1x.save(OUT_1X, format="PNG", optimize=True)
    icon2x.save(OUT_2X, format="PNG", optimize=True)
    print(f"saved {SRC.name} {OUT_1X.name} {OUT_2X.name}")

    b64 = base64.b64encode(OUT_1X.read_bytes()).decode("ascii")
    text = TRAY_TS.read_text()
    new_block = (
        "const EMBEDDED_TRAY_TEMPLATE_PNG = Buffer.from(\n"
        f"  '{b64}',\n"
        "  'base64'\n"
        ")"
    )
    new_text, n = re.subn(
        r"const EMBEDDED_TRAY_TEMPLATE_PNG = Buffer\.from\(\n  '[^']*',\n  'base64'\n\)",
        new_block,
        text,
        count=1,
    )
    if n != 1:
        raise SystemExit(f"embed replace failed n={n}")
    new_text = new_text.replace(
        "Embedded 16×16 template (black/alpha pause block)",
        "Embedded 16×16 template (black/alpha cup silhouette)",
    )
    TRAY_TS.write_text(new_text)
    print("updated tray.ts embed")
    print("1x opaque px", int((np.array(icon1x)[:, :, 3] > 10).sum()))
    print("2x opaque px", int((np.array(icon2x)[:, :, 3] > 10).sum()))


if __name__ == "__main__":
    main()
