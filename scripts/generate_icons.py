"""Generate polished tray icons for TinyWhispr"""
from PIL import Image, ImageDraw, ImageFilter
import struct, io, math

SIZES = [16, 24, 32, 48, 64, 128, 256]
TRAY_SIZES = [16, 24, 32, 48]


def draw_mic(draw, s, color, glow=False):
    """Draw a clean microphone icon at size s"""
    cx = s / 2
    fill = (*color, 255)

    # Proportions scaled to size
    bw = s * 0.16          # body half-width
    bt = s * 0.10          # body top
    bb = s * 0.44          # body bottom
    br = bw                # corner radius for rounded top

    # Mic capsule (pill shape)
    draw.rounded_rectangle(
        [cx - bw, bt, cx + bw, bb],
        radius=int(bw + 0.5),
        fill=fill
    )

    # Arc around capsule
    aw = s * 0.28    # arc half-width
    at = s * 0.22    # arc top
    ab = s * 0.54    # arc bottom
    lw = max(round(s / 11), 1)

    # Draw the U-shaped arc as a semicircle
    draw.arc(
        [cx - aw, at, cx + aw, ab + (ab - at) * 0.4],
        start=0, end=180,
        fill=fill, width=lw
    )

    # Vertical stand
    st = ab + lw
    sb = s * 0.72
    draw.line([cx, st, cx, sb], fill=fill, width=lw)

    # Horizontal base
    base_hw = s * 0.18
    draw.line([cx - base_hw, sb, cx + base_hw, sb], fill=fill, width=lw)


def draw_waves(draw, s, color):
    """Draw sound wave arcs emanating from right side of mic"""
    cx = s * 0.45
    cy = s * 0.30
    lw = max(round(s / 11), 1)

    wave_configs = [
        (s * 0.22, 240),  # small wave, full alpha
        (s * 0.36, 180),  # medium wave
    ]
    if s >= 32:
        wave_configs.append((s * 0.50, 120))  # large wave

    for radius, alpha in wave_configs:
        draw.arc(
            [cx, cy - radius * 0.5, cx + radius * 1.4, cy + radius * 0.5],
            start=-45, end=45,
            fill=(*color, alpha), width=lw
        )


def draw_dots(draw, s, color):
    """Draw processing indicator dots"""
    cx = s * 0.5
    cy = s * 0.35
    r = s * 0.34
    dot_r = max(s * 0.06, 1.2)

    # 3 dots in arc formation (right side)
    for i, (angle, alpha) in enumerate([(-40, 255), (10, 180), (60, 100)]):
        rad = math.radians(angle)
        x = cx + r * math.cos(rad)
        y = cy - r * math.sin(rad)
        draw.ellipse(
            [x - dot_r, y - dot_r, x + dot_r, y + dot_r],
            fill=(*color, alpha)
        )


def create_icon(size, state):
    # Render at 4x for anti-aliasing, then downscale
    render_size = size * 4
    img = Image.new('RGBA', (render_size, render_size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    if state == 'idle':
        draw_mic(draw, render_size, (210, 210, 210))
    elif state == 'recording':
        draw_mic(draw, render_size, (239, 68, 68))
        draw_waves(draw, render_size, (239, 68, 68))
    elif state == 'processing':
        draw_mic(draw, render_size, (96, 165, 250))
        draw_dots(draw, render_size, (59, 130, 246))

    # Downscale with high-quality resampling
    img = img.resize((size, size), Image.LANCZOS)
    return img


def create_ico(images):
    """Create multi-resolution ICO"""
    output = io.BytesIO()
    n = len(images)
    output.write(struct.pack('<HHH', 0, 1, n))

    data_list = []
    for img in images:
        buf = io.BytesIO()
        img.save(buf, format='PNG')
        data_list.append(buf.getvalue())

    offset = 6 + 16 * n
    for i, img in enumerate(images):
        w, h = img.size
        d = data_list[i]
        output.write(struct.pack('<BBBBHHII',
            w if w < 256 else 0, h if h < 256 else 0,
            0, 0, 1, 32, len(d), offset))
        offset += len(d)

    for d in data_list:
        output.write(d)
    return output.getvalue()


def main():
    icon_dir = "F:/Projects/tinywhispr/src-tauri/icons"

    for state, name in [('idle', 'icon'), ('recording', 'icon-recording'), ('processing', 'icon-processing')]:
        tray_imgs = [create_icon(s, state) for s in TRAY_SIZES]

        # PNG at 32x32 for Tauri tray
        tray_imgs[2].save(f"{icon_dir}/{name}.png")

        # ICO with all tray sizes
        ico = create_ico(tray_imgs)
        with open(f"{icon_dir}/{name}.ico", 'wb') as f:
            f.write(ico)

        print(f"  {name}: done ({len(TRAY_SIZES)} sizes)")

    # App icon (128x128) for installer/taskbar
    app_icon = create_icon(128, 'idle')
    app_icon.save(f"{icon_dir}/icon.png")

    # Also a 256x256 for high-DPI
    large = create_icon(256, 'idle')
    large.save(f"{icon_dir}/icon-large.png")

    # Square 32x32 ICO with just idle for the app icon
    all_sizes = [create_icon(s, 'idle') for s in [16, 24, 32, 48, 64, 128, 256]]
    ico = create_ico(all_sizes)
    with open(f"{icon_dir}/icon.ico", 'wb') as f:
        f.write(ico)

    print("  icon.ico: app icon (7 sizes)")
    print("Done!")


if __name__ == '__main__':
    main()
