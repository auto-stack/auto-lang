import os
import io
import base64
import json
import time
from datetime import datetime
from PIL import Image

SOURCE_DIR = r"C:\Users\zhaop\Pictures"
OUT_THUMBS_DIR = r"D:\autostack\.wt\lang-628\auto-lang\examples\ui\029-photo-gallery\src\front\thumbnails"
METADATA_FILE = r"D:\autostack\.wt\lang-628\auto-lang\examples\ui\029-photo-gallery\src\front\gallery_data.json"

os.makedirs(OUT_THUMBS_DIR, exist_ok=True)

valid_exts = {".jpg", ".jpeg", ".png", ".webp"}

entries = []
print(f"[*] Scanning {SOURCE_DIR}...")

# 1. First collect direct files in C:\Users\zhaop\Pictures
for fname in sorted(os.listdir(SOURCE_DIR)):
    ext = os.path.splitext(fname)[1].lower()
    if ext in valid_exts:
        full_path = os.path.join(SOURCE_DIR, fname)
        if os.path.isfile(full_path):
            entries.append((full_path, "photos", fname))

# 2. Collect from Screenshots (take recent 15)
screenshots_dir = os.path.join(SOURCE_DIR, "Screenshots")
if os.path.isdir(screenshots_dir):
    for fname in sorted(os.listdir(screenshots_dir), reverse=True)[:15]:
        ext = os.path.splitext(fname)[1].lower()
        if ext in valid_exts:
            full_path = os.path.join(screenshots_dir, fname)
            if os.path.isfile(full_path):
                entries.append((full_path, "screenshots", fname))

print(f"[*] Total collected candidates: {len(entries)}")

items = []
t0 = time.time()

for idx, (img_path, album_type, raw_name) in enumerate(entries):
    try:
        stat = os.stat(img_path)
        mtime = stat.st_mtime
        dt = datetime.fromtimestamp(mtime)
        date_str = dt.strftime("%Y-%m-%d")
        sort_key = int(dt.strftime("%Y%m%d"))
        size_kb = int(stat.st_size / 1024)

        # Generate thumbnail
        thumb_name = f"thumb_{idx+1:03d}.jpg"
        thumb_full_path = os.path.join(OUT_THUMBS_DIR, thumb_name)

        with Image.open(img_path) as im:
            w, h = im.size
            im.thumbnail((260, 260), Image.Resampling.LANCZOS)
            buf = io.BytesIO()
            im.convert("RGB").save(buf, format="JPEG", quality=75)
            thumb_bytes = buf.getvalue()

            with open(thumb_full_path, "wb") as f:
                f.write(thumb_bytes)

            data_url = "data:image/jpeg;base64," + base64.b64encode(thumb_bytes).decode("ascii")

        # Friendly title
        title = os.path.splitext(raw_name)[0]
        if len(title) > 20:
            title = title[:18] + "…"

        item = {
            "id": idx + 1,
            "title": title,
            "raw_name": raw_name,
            "album": album_type,
            "date": date_str,
            "sort_key": sort_key,
            "width": w,
            "height": h,
            "size_kb": size_kb,
            "full_path": img_path.replace("\\", "/"),
            "thumb_url": data_url,
            "fav": False
        }
        items.append(item)
    except Exception as e:
        print(f"[!] Skip {img_path}: {e}")

# Preset a few favorites
for idx in [0, 2, 5, 8, 12]:
    if idx < len(items):
        items[idx]["fav"] = True

with open(METADATA_FILE, "w", encoding="utf-8") as f:
    json.dump(items, f, ensure_ascii=False, indent=2)

t1 = time.time()
print(f"[+] Successfully generated {len(items)} thumbnails and metadata in {t1 - t0:.2f}s!")
