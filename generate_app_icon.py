# -*- coding: utf-8 -*-
import math
from PIL import Image, ImageDraw, ImageFilter, ImageFont

def create_md_monitor_icon(output_path="D:/note-dev/MD_Monitor/app-icon.png"):
    size = 1024
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))

    # 1. Outer Squircle Mask & Gradient
    pad = 64
    r = 200

    mask = Image.new('L', (size, size), 0)
    mask_draw = ImageDraw.Draw(mask)
    mask_draw.rounded_rectangle([pad, pad, size - pad, size - pad], radius=r, fill=255)

    bg_layer = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    bg_draw = ImageDraw.Draw(bg_layer)
    for y in range(pad, size - pad):
        t = (y - pad) / (size - 2 * pad)
        # Deep dark cyber metallic: #0b0f17 to #161e2e
        cr = int(11 * (1 - t) + 22 * t)
        cg = int(15 * (1 - t) + 30 * t)
        cb = int(23 * (1 - t) + 46 * t)
        bg_draw.line([(pad, y), (size - pad, y)], fill=(cr, cg, cb, 255))

    img.paste(bg_layer, (0, 0), mask)

    # 2. Outer Glow & Border
    glow_layer = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    g_draw = ImageDraw.Draw(glow_layer)
    # Dual-tone neon border: Cyan to Electric Blue
    for step in range(12):
        alpha = int(200 - step * 14)
        g_draw.rounded_rectangle(
            [pad + step, pad + step, size - pad - step, size - pad - step],
            radius=max(10, r - step),
            outline=(0, 230, 204, max(10, alpha // 4)),
            width=2
        )
    g_draw.rounded_rectangle(
        [pad + 6, pad + 6, size - pad - 6, size - pad - 6],
        radius=r - 6,
        outline=(0, 255, 204, 255),
        width=10
    )
    img = Image.alpha_composite(img, glow_layer)

    # 3. Inner Monitor Screen Frame
    screen_layer = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    s_draw = ImageDraw.Draw(screen_layer)

    # Monitor outer chassis
    m_left, m_top, m_right, m_bot = 160, 180, 864, 720
    s_draw.rounded_rectangle([m_left, m_top, m_right, m_bot], radius=32, fill=(20, 26, 38, 255), outline=(48, 58, 78, 255), width=6)

    # Monitor inner display area
    d_left, d_top, d_right, d_bot = 184, 204, 840, 696
    s_draw.rounded_rectangle([d_left, d_top, d_right, d_bot], radius=20, fill=(7, 10, 16, 255))

    # Monitor Stand / Base
    s_draw.polygon([(460, 720), (564, 720), (580, 830), (444, 830)], fill=(28, 36, 52, 255))
    s_draw.rounded_rectangle([380, 820, 644, 856], radius=12, fill=(38, 48, 68, 255), outline=(0, 255, 204, 180), width=3)

    img = Image.alpha_composite(img, screen_layer)

    # 4. Display Content: Grid & Telemetry
    hud_layer = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    h_draw = ImageDraw.Draw(hud_layer)

    # Subtle radar/grid lines
    for gx in range(d_left + 40, d_right, 60):
        h_draw.line([(gx, d_top + 10), (gx, d_bot - 10)], fill=(18, 26, 38, 120), width=1)
    for gy in range(d_top + 40, d_bot, 60):
        h_draw.line([(d_left + 10, gy), (d_right - 10, gy)], fill=(18, 26, 38, 120), width=1)

    # Circular Gauge Arc on the left (Speedometer / Tachometer style)
    arc_box = [230, 280, 470, 520]
    # Arc track
    h_draw.arc(arc_box, start=135, end=405, fill=(30, 42, 60, 200), width=16)
    # Active arc
    h_draw.arc(arc_box, start=135, end=335, fill=(0, 255, 204, 255), width=16)
    # Gauge center text "MD"
    # Draw geometric "M" and "D" in center of screen
    # Draw bold M
    m_pts = [
        (540, 480), (540, 320), (600, 410), (660, 320), (660, 480)
    ]
    h_draw.line([(540, 480), (540, 320)], fill=(0, 255, 204, 255), width=20)
    h_draw.line([(540, 320), (600, 410)], fill=(0, 255, 204, 255), width=20)
    h_draw.line([(600, 410), (660, 320)], fill=(0, 255, 204, 255), width=20)
    h_draw.line([(660, 320), (660, 480)], fill=(0, 255, 204, 255), width=20)

    # Draw bold D
    d_pts_v = [(700, 320), (700, 480)]
    h_draw.line(d_pts_v, fill=(56, 139, 253, 255), width=20)
    h_draw.arc([640, 320, 770, 480], start=-90, end=90, fill=(56, 139, 253, 255), width=20)
    h_draw.line([(700, 320), (705, 320)], fill=(56, 139, 253, 255), width=20)
    h_draw.line([(700, 480), (705, 480)], fill=(56, 139, 253, 255), width=20)

    # Telemetry Pulse Line (Cardiac / Performance wave across bottom of display)
    wave_pts = [
        (d_left + 20, 590),
        (320, 590),
        (350, 570),
        (370, 630),
        (400, 530),
        (420, 640),
        (440, 590),
        (560, 590),
        (580, 550),
        (600, 620),
        (620, 590),
        (d_right - 20, 590)
    ]
    # Glow for wave
    for w_idx in range(len(wave_pts) - 1):
        p1 = wave_pts[w_idx]
        p2 = wave_pts[w_idx + 1]
        h_draw.line([p1, p2], fill=(0, 255, 204, 100), width=14)
    for w_idx in range(len(wave_pts) - 1):
        p1 = wave_pts[w_idx]
        p2 = wave_pts[w_idx + 1]
        h_draw.line([p1, p2], fill=(0, 255, 204, 255), width=6)

    # 3 Status LEDs in top right of monitor bezel
    h_draw.ellipse([760, 222, 774, 236], fill=(0, 255, 204, 255))
    h_draw.ellipse([786, 222, 800, 236], fill=(255, 183, 3, 255))
    h_draw.ellipse([812, 222, 826, 236], fill=(255, 51, 68, 255))

    img = Image.alpha_composite(img, hud_layer)

    # 5. Diagonal Glass Reflection Sheen
    sheen = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    sh_draw = ImageDraw.Draw(sheen)
    # Diagonal polygon across screen
    sh_pts = [(d_left, d_top), (d_left + 280, d_top), (d_left + 80, d_bot), (d_left, d_bot)]
    sh_draw.polygon(sh_pts, fill=(255, 255, 255, 18))
    img = Image.alpha_composite(img, sheen)

    # 6. Resize with Lanczos down to 512x512
    final_512 = img.resize((512, 512), Image.Resampling.LANCZOS)
    final_512.save(output_path, "PNG")
    print(f"Successfully generated custom app icon: {output_path}")

if __name__ == "__main__":
    create_md_monitor_icon()
