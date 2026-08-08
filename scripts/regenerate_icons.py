"""Render the source favicon into a full-size transparent desktop icon.

Usage (inside a temporary tooling venv):
  pip install Pillow CairoSVG
  python scripts/regenerate_icons.py
  npm run tauri -- icon src-tauri/icons/icon.png
"""

from io import BytesIO
from pathlib import Path

import cairosvg
from PIL import Image


SOURCE = Path("src-tauri/icons/icon.svg")
DESTINATION = Path("src-tauri/icons/icon.png")
CANVAS_SIZE = 1024
MAX_CONTENT_SIZE = 900


def main() -> None:
    rendered = cairosvg.svg2png(
        url=str(SOURCE), output_width=CANVAS_SIZE, output_height=CANVAS_SIZE
    )
    image = Image.open(BytesIO(rendered)).convert("RGBA")
    bounds = image.getchannel("A").getbbox()
    if bounds is None:
        raise RuntimeError("The SVG source icon has no visible pixels")

    content = image.crop(bounds)
    content.thumbnail((MAX_CONTENT_SIZE, MAX_CONTENT_SIZE), Image.Resampling.NEAREST)

    icon = Image.new("RGBA", (CANVAS_SIZE, CANVAS_SIZE), (0, 0, 0, 0))
    position = ((CANVAS_SIZE - content.width) // 2, (CANVAS_SIZE - content.height) // 2)
    icon.alpha_composite(content, position)
    icon.save(DESTINATION)


if __name__ == "__main__":
    main()
