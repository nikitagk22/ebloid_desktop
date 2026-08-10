#!/usr/bin/env python3
"""Generate small Windows taskbar overlay badges for unread counts."""

from pathlib import Path

from PIL import Image, ImageDraw, ImageFont


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "src-tauri" / "icons" / "badges"
FONT_CANDIDATES = (
    Path("/System/Library/Fonts/Supplemental/Arial Bold.ttf"),
    Path("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"),
    Path("C:/Windows/Fonts/arialbd.ttf"),
)


def font(size: int) -> ImageFont.FreeTypeFont | ImageFont.ImageFont:
    for candidate in FONT_CANDIDATES:
        if candidate.exists():
            return ImageFont.truetype(str(candidate), size)
    return ImageFont.load_default()


def badge(label: str, name: str) -> None:
    scale = 4
    image = Image.new("RGBA", (32 * scale, 32 * scale), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)
    draw.ellipse((1 * scale, 1 * scale, 31 * scale, 31 * scale), fill="#9fc236", outline="#1b1b1b", width=2 * scale)
    text_font = font((17 if len(label) == 1 else 12) * scale)
    bounds = draw.textbbox((0, 0), label, font=text_font)
    width = bounds[2] - bounds[0]
    height = bounds[3] - bounds[1]
    draw.text(((128 - width) / 2, (128 - height) / 2 - bounds[1]), label, fill="#151515", font=text_font)
    image.resize((32, 32), Image.Resampling.LANCZOS).save(OUTPUT / f"{name}.png")


def main() -> None:
    OUTPUT.mkdir(parents=True, exist_ok=True)
    for value in range(1, 10):
        badge(str(value), str(value))
    badge("9+", "9plus")


if __name__ == "__main__":
    main()
