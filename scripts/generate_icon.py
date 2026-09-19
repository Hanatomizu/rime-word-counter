#!/usr/bin/env python3
"""生成应用图标源图（1024×1024 PNG）。

设计：强调色圆角方块 + 三根递增的白色柱子，与界面左上角的 Logo 一致。
只在需要改图标时手动运行，产物提交到仓库：

    python3 scripts/generate_icon.py
    npx tauri icon src-tauri/icons/source.png   # 再生成各平台尺寸

依赖：Pillow
"""

from pathlib import Path

from PIL import Image, ImageDraw

# 与 ui/src/styles/tokens.css 保持一致
PRIMARY = (94, 106, 210, 255)
ON_PRIMARY = (255, 255, 255, 255)

SIZE = 1024
# 先按 4 倍绘制再缩小，得到干净的抗锯齿边缘
SCALE = 4


def main() -> None:
    canvas = SIZE * SCALE
    image = Image.new("RGBA", (canvas, canvas), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)

    # 圆角方块（半径约 22%，接近 macOS/iOS 的 squircle 观感）
    radius = int(canvas * 0.22)
    draw.rounded_rectangle([(0, 0), (canvas - 1, canvas - 1)], radius=radius, fill=PRIMARY)

    # 三根递增的柱子，底部对齐
    bar_width = int(canvas * 0.148)
    gap = int(canvas * 0.059)
    bottom = int(canvas * 0.781)
    heights = [0.234, 0.391, 0.562]
    left = (canvas - (bar_width * 3 + gap * 2)) // 2

    for index, ratio in enumerate(heights):
        height = int(canvas * ratio)
        x0 = left + index * (bar_width + gap)
        draw.rounded_rectangle(
            [(x0, bottom - height), (x0 + bar_width, bottom)],
            radius=bar_width // 2,
            fill=ON_PRIMARY,
        )

    output = Path(__file__).resolve().parent.parent / "src-tauri" / "icons" / "source.png"
    output.parent.mkdir(parents=True, exist_ok=True)
    image.resize((SIZE, SIZE), Image.LANCZOS).save(output, "PNG")
    print(f"wrote {output}")


if __name__ == "__main__":
    main()
