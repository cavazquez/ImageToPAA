#!/usr/bin/env python3
"""Generate synthetic PNG fixtures (no third-party art). Idempotent."""
from __future__ import annotations

import math
import struct
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "fixtures" / "sources"


def write_png(path: Path, width: int, height: int, rgba: bytes) -> None:
    assert len(rgba) == width * height * 4

    def chunk(tag: bytes, data: bytes) -> bytes:
        return (
            struct.pack(">I", len(data))
            + tag
            + data
            + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
        )

    raw = b"".join(
        b"\x00" + rgba[y * width * 4 : (y + 1) * width * 4] for y in range(height)
    )
    ihdr = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", ihdr) + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b"")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(png)


def opaque_blocks(w: int = 16, h: int = 16) -> bytes:
    out = bytearray()
    for y in range(h):
        for x in range(w):
            if (x // 4 + y // 4) % 2 == 0:
                out.extend((220, 40, 40, 255))
            else:
                out.extend((40, 40, 220, 255))
    return bytes(out)


def opaque_gradient(w: int = 16, h: int = 8) -> bytes:
    out = bytearray()
    for y in range(h):
        for x in range(w):
            out.extend((x * 15, y * 30, 128, 255))
    return bytes(out)


def binary_alpha(w: int = 16, h: int = 16) -> bytes:
    out = bytearray()
    cx, cy = (w - 1) / 2, (h - 1) / 2
    r = min(w, h) * 0.35
    for y in range(h):
        for x in range(w):
            d = math.hypot(x - cx, y - cy)
            a = 255 if d <= r else 0
            # coloured fringe under transparent pixels (halo detector)
            rgb = (255, 0, 0) if a else (0, 0, 0)
            out.extend((*rgb, a))
    return bytes(out)


def soft_radial(w: int = 16, h: int = 16) -> bytes:
    out = bytearray()
    cx, cy = (w - 1) / 2, (h - 1) / 2
    r = min(w, h) * 0.5
    for y in range(h):
        for x in range(w):
            d = math.hypot(x - cx, y - cy)
            a = int(max(0.0, 1.0 - d / r) * 255)
            out.extend((30, 180, 90, a))
    return bytes(out)


def soft_diagonal(w: int = 8, h: int = 16) -> bytes:
    out = bytearray()
    for y in range(h):
        for x in range(w):
            a = int((x + y) / (w + h - 2) * 255)
            # black under transparency
            out.extend((0, 0, 0, a))
    return bytes(out)


def main() -> None:
    cases = {
        "opaque_blocks_16.png": (16, 16, opaque_blocks()),
        "opaque_gradient_16x8.png": (16, 8, opaque_gradient()),
        "binary_alpha_16.png": (16, 16, binary_alpha()),
        "soft_radial_16.png": (16, 16, soft_radial()),
        "soft_diagonal_8x16.png": (8, 16, soft_diagonal()),
        "spaces in name 8.png": (8, 8, opaque_blocks(8, 8)),
    }
    for name, (w, h, data) in cases.items():
        path = OUT / name
        write_png(path, w, h, data)
        print(f"wrote {path.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
