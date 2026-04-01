"""
Генерация тестового спрайт-листа НЛО: 4 кадра 60×40 px → файл 240×40 px (RGBA).
Кадры отличаются цветом огней снизу (анимация вращения/мигания).
"""
import struct
import zlib


def png_chunk(tag: bytes, data: bytes) -> bytes:
    c = struct.pack(">I", len(data)) + tag + data
    return c + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)


def save_png(path: str, width: int, height: int, pixels: list[list[tuple]]):
    """pixels[y][x] = (r, g, b, a)"""
    raw = b""
    for row in pixels:
        raw += b"\x00"  # filter type None
        for r, g, b, a in row:
            raw += bytes([r, g, b, a])
    compressed = zlib.compress(raw, 9)

    with open(path, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n")
        f.write(png_chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)))
        f.write(png_chunk(b"IDAT", compressed))
        f.write(png_chunk(b"IEND", b""))


def make_frame(frame_idx: int, w: int = 60, h: int = 40) -> list[list[tuple]]:
    """
    Рисует один кадр НЛО:
    - корпус: эллипс серебристого цвета
    - купол: меньший эллипс сверху, голубоватый
    - огни снизу: 5 кружков, цвет зависит от frame_idx
    """
    # Цвета огней для каждого кадра (циклическая анимация)
    light_colors = [
        [(255, 80,  80,  255), (255, 200, 80,  255), (80,  255, 80,  255), (80,  200, 255, 255), (200, 80,  255, 255)],
        [(200, 80,  255, 255), (255, 80,  80,  255), (255, 200, 80,  255), (80,  255, 80,  255), (80,  200, 255, 255)],
        [(80,  200, 255, 255), (200, 80,  255, 255), (255, 80,  80,  255), (255, 200, 80,  255), (80,  255, 80,  255)],
        [(80,  255, 80,  255), (80,  200, 255, 255), (200, 80,  255, 255), (255, 80,  80,  255), (255, 200, 80,  255)],
    ]

    pixels = [[(0, 0, 0, 0)] * w for _ in range(h)]

    cx = w // 2
    # --- корпус НЛО (широкий эллипс) ---
    body_cx, body_cy = cx, 26
    body_rx, body_ry = 27, 10
    for y in range(h):
        for x in range(w):
            dx = (x - body_cx) / body_rx
            dy = (y - body_cy) / body_ry
            if dx*dx + dy*dy <= 1.0:
                # Градиент: светлее сверху
                t = 1.0 - (y - (body_cy - body_ry)) / (2 * body_ry)
                t = max(0.0, min(1.0, t))
                c = int(160 + 60 * t)
                pixels[y][x] = (c, c, c + 20, 255)

    # --- купол (меньший эллипс сверху) ---
    dome_cx, dome_cy = cx, 18
    dome_rx, dome_ry = 14, 10
    for y in range(h):
        for x in range(w):
            dx = (x - dome_cx) / dome_rx
            dy = (y - dome_cy) / dome_ry
            if dx*dx + dy*dy <= 1.0 and y <= dome_cy:
                t = 1.0 - abs(x - dome_cx) / dome_rx
                r = int(80  + 60 * t)
                g = int(160 + 60 * t)
                b = int(220 + 30 * t)
                pixels[y][x] = (r, g, b, 220)

    # --- огни снизу (5 кружков) ---
    light_xs = [cx - 20, cx - 10, cx, cx + 10, cx + 20]
    light_y   = 33
    light_r   = 3
    for i, lx in enumerate(light_xs):
        col = light_colors[frame_idx][i]
        for y in range(h):
            for x in range(w):
                dx = x - lx
                dy = y - light_y
                if dx*dx + dy*dy <= light_r * light_r:
                    pixels[y][x] = col

    return pixels


def main():
    frames = 4
    fw, fh = 60, 40
    total_w = fw * frames

    all_pixels = [[(0, 0, 0, 0)] * total_w for _ in range(fh)]

    for f in range(frames):
        frame = make_frame(f, fw, fh)
        for y in range(fh):
            for x in range(fw):
                all_pixels[y][f * fw + x] = frame[y][x]

    out = "assets/sprites/ufo.png"
    save_png(out, total_w, fh, all_pixels)
    print(f"Saved: {out}  ({total_w}x{fh}, {frames} frames of {fw}x{fh})")


if __name__ == "__main__":
    main()
