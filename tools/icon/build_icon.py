# build_icon.py - Builds every MyNoX icon (app, extension, overlay, docs) from art/MyNoX Download Manager.png.
# Run from the repository root: python -I tools/icon/build_icon.py

import base64
import re
from pathlib import Path

from PIL import Image, ImageChops, ImageFilter

ROOT = Path(__file__).resolve().parents[2]
SRC = ROOT / "art" / "MyNoX Download Manager.png"
APP = ROOT / "crates" / "app" / "assets" / "icons" / "app"
EXT = ROOT / "extension" / "icons"
VIDEO_JS = ROOT / "extension" / "video.js"
DOCS = ROOT / "docs" / "images"
SIZES = [16, 20, 24, 32, 48, 64, 128, 256]
EDGE = (6, 26, 92)  # thin navy edge round the white arrow, so it shows on light backgrounds


def mark() -> Image.Image:
    # The logo cropped to its visible pixels, with an edge round its white arrow.
    src = Image.open(SRC).convert("RGBA")
    src = src.crop(src.split()[3].point(lambda v: 255 if v > 16 else 0).getbbox())
    r, g, b, a = src.split()
    white = ImageChops.multiply(ImageChops.multiply(r.point(lambda v: 255 if v > 225 else 0), g.point(lambda v: 255 if v > 225 else 0)), b.point(lambda v: 255 if v > 225 else 0))
    white = ImageChops.multiply(white, a.point(lambda v: 255 if v > 128 else 0))
    ring = white.filter(ImageFilter.MaxFilter(9))
    edge = Image.new("RGBA", src.size, EDGE + (0,))
    edge.putalpha(ring)
    out = Image.alpha_composite(edge, src)
    return out


def icon(size: int, mk: Image.Image) -> Image.Image:
    # The mark centred on a transparent square, as wide as the square allows.
    pad = 0 if size <= 32 else round(size * 0.04)
    w = size - 2 * pad
    h = round(mk.height * w / mk.width)
    out = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    out.alpha_composite(mk.resize((w, h), Image.LANCZOS), (pad, (size - h) // 2))
    return out


def main() -> None:
    mk = mark()
    icons = {s: icon(s, mk) for s in SIZES}
    for s, im in icons.items():
        im.save(APP / f"mynox-{s}.png")
    icons[256].save(APP / "mynox.ico", sizes=[(s, s) for s in SIZES], append_images=[icons[s] for s in SIZES if s != 256])
    for s in (16, 32, 48, 128):
        icon(s, mk).save(EXT / f"icon-{s}.png")
    # The extension's Download video button carries the 48 px icon inline (pages can't load extension files).
    data = "data:image/png;base64," + base64.b64encode((EXT / "icon-48.png").read_bytes()).decode()
    js = VIDEO_JS.read_text(encoding="utf-8")
    js, n = re.subn(r'const LOGO = "data:image/png;base64,[A-Za-z0-9+/=]+";', f'const LOGO = "{data}";', js)
    assert n == 1, "LOGO not found in extension/video.js"
    VIDEO_JS.write_text(js, encoding="utf-8", newline="")
    icon(320, mk).save(DOCS / "logo.png")
    print("wrote", ", ".join(map(str, SIZES)), "+ mynox.ico + extension icons + video button + docs logo")


if __name__ == "__main__":
    main()
