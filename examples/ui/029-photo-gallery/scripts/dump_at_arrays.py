import json

with open(r"D:\autostack\.wt\lang-628\auto-lang\examples\ui\029-photo-gallery\src\front\gallery_data.json", encoding="utf-8") as f:
    items = json.load(f)

print(f"Loaded {len(items)} items")

# Generate .at array definitions
lines = []
lines.append(f"        // ---- 真实照片种子列表（{len(items)} 张，来自 C:\\Users\\zhaop\\Pictures）----")
p_ids = [item["id"] for item in items]
p_titles = [item["title"] for item in items]
p_tls = [item["title"].lower() for item in items]
p_albums = [item["album"] for item in items]
p_dates = [item["date"] for item in items]
p_keys = [item["sort_key"] for item in items]
p_favs = [item["fav"] for item in items]
p_fulls = [item["full_path"] for item in items]
p_metas = [f"{item['width']}×{item['height']} · {item['size_kb']}KB · {item['date']}" for item in items]
p_thumbs = [item["thumb_url"] for item in items]

def fmt_arr(name, arr, is_str=False, is_bool=False):
    s = f"        var {name} = [\n"
    for i in range(0, len(arr), 4):
        chunk = arr[i:i+4]
        if is_str:
            row_s = ", ".join(json.dumps(x, ensure_ascii=False) for x in chunk)
        elif is_bool:
            row_s = ", ".join("true" if x else "false" for x in chunk)
        else:
            row_s = ", ".join(str(x) for x in chunk)
        if i + 4 < len(arr):
            row_s += ","
        s += f"            {row_s}\n"
    s += "        ]"
    return s

with open(r"D:\autostack\.wt\lang-628\auto-lang\examples\ui\029-photo-gallery\src\front\data_gen.py", "w", encoding="utf-8") as out:
    out.write("# helper to inspect format\n")

print(f"Generated arrays successfully.")
