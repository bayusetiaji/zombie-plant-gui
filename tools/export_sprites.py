"""Export the pixel maps in src/pixel_art.rs as real PNG files.

The game renders the maps directly, so editing a character in pixel_art.rs and
running this script keeps the PNG assets in sync. Python standard library only.

    python tools/export_sprites.py
"""

from __future__ import annotations

import re
import struct
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "src" / "pixel_art.rs"
OUT = ROOT / "assets" / "sprites"
SIZE = 16


def chunk(tag: bytes, data: bytes) -> bytes:
    return (
        struct.pack(">I", len(data))
        + tag
        + data
        + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
    )


def write_png(path: Path, width: int, height: int, rgba: bytes) -> None:
    raw = b"".join(
        b"\x00" + rgba[y * width * 4 : (y + 1) * width * 4] for y in range(height)
    )
    png = b"\x89PNG\r\n\x1a\n"
    png += chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
    png += chunk(b"IDAT", zlib.compress(raw, 9))
    png += chunk(b"IEND", b"")
    path.write_bytes(png)


def parse() -> tuple[dict[str, tuple[int, int, int, int]], dict[str, list[str]]]:
    text = SOURCE.read_text(encoding="utf-8")

    palette: dict[str, tuple[int, int, int, int]] = {}
    for ch, values in re.findall(
        r"\('(.)',\s*Color::new\(([^)]*)\)\)", text
    ):
        nums = [float(n.strip()) for n in values.split(",")]
        rgba = tuple(round(min(1.0, max(0.0, n)) * 255) for n in nums[:4])
        palette[ch] = rgba  # type: ignore[assignment]

    rows_for = text.split("fn rows_for", 1)[1]
    sprites: dict[str, list[str]] = {}
    for name, body in re.findall(r"Sprite::(\w+)\s*=>\s*&\[(.*?)\]", rows_for, re.S):
        rows = re.findall(r'"([^"]*)"', body)
        if len(rows) != SIZE:
            raise ValueError(f"{name}: expected {SIZE} rows, got {len(rows)}")
        sprites[name] = [row.ljust(SIZE, ".")[:SIZE] for row in rows]

    missing = {ch for rows in sprites.values() for row in rows for ch in row} - set(
        palette
    )
    if missing:
        raise ValueError(f"unknown palette characters: {sorted(missing)}")
    return palette, sprites


def pixels(
    palette: dict[str, tuple[int, int, int, int]], rows: list[str]
) -> bytes:
    out = bytearray()
    for row in rows:
        for ch in row:
            out.extend(palette[ch])
    return bytes(out)


def scale_nearest(rgba: bytes, factor: int) -> bytes:
    out = bytearray()
    for y in range(SIZE):
        line = rgba[y * SIZE * 4 : (y + 1) * SIZE * 4]
        scaled_row = b"".join(line[x * 4 : x * 4 + 4] * factor for x in range(SIZE))
        for _ in range(factor):
            out.extend(scaled_row)
    return bytes(out)


def main() -> None:
    palette, sprites = parse()
    OUT.mkdir(parents=True, exist_ok=True)

    for name, rows in sprites.items():
        rgba = pixels(palette, rows)
        write_png(OUT / f"{name.lower()}.png", SIZE, SIZE, rgba)

    # A scaled contact sheet makes it easy to eyeball every sprite at once.
    factor = 8
    cell = SIZE * factor
    cols = 3
    rows_count = (len(sprites) + cols - 1) // cols
    sheet = bytearray(cell * cols * cell * rows_count * 4)
    for i, rows in enumerate(sprites.values()):
        big = scale_nearest(pixels(palette, rows), factor)
        ox = (i % cols) * cell
        oy = (i // cols) * cell
        for y in range(cell):
            start = ((oy + y) * cell * cols + ox) * 4
            sheet[start : start + cell * 4] = big[y * cell * 4 : (y + 1) * cell * 4]
    write_png(
        OUT / "preview_8x.png",
        cell * cols,
        cell * rows_count,
        bytes(sheet),
    )
    print(f"Wrote {len(sprites)} sprites + preview_8x.png to {OUT}")


if __name__ == "__main__":
    main()
