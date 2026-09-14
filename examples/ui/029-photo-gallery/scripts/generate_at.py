import json

with open(r"D:\autostack\.wt\lang-628\auto-lang\examples\ui\029-photo-gallery\src\front\gallery_data.json", encoding="utf-8") as f:
    items = json.load(f)

p_ids = [item["id"] for item in items]
p_titles = [item["title"] for item in items]
p_tls = [item["title"].lower() for item in items]
p_albums = [item["album"] for item in items]
p_dates = [item["date"] for item in items]
p_keys = [item["sort_key"] for item in items]
p_favs = [item["fav"] for item in items]
p_fulls = [item["full_path"] for item in items]
p_metas = [f"{item['width']}×{item['height']} · {item['size_kb']}KB · {item['date']}" for item in items]
# 使用本地绝对路径文件引用！
p_thumbs = [f"D:/autostack/.wt/lang-628/auto-lang/examples/ui/029-photo-gallery/src/front/thumbnails/thumb_{item['id']:03d}.jpg" for item in items]

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

code = []
code.append("// app.at — 029-photo-gallery 现代相册（Plan 628）")
code.append("//")
code.append("// 移动端/平板风格真实图库：")
code.append("// 1. 真实相册数据源：直连 C:\\Users\\zhaop\\Pictures\\ 真实照片与屏幕截图")
code.append("// 2. 真实缩略图与原图呈现：秒开高清本地缩略图 + 全屏无损原画直接渲染")
code.append("// 3. 平板/手机端 UI/UX：取消生硬的左侧 PC 侧栏，采用沉浸式分段导航栏 + 纯净无边框照片网格 + 悬浮胶囊控制条")
code.append("")
code.append("widget App {")
code.append("    msg {")
code.append("        Init,")
code.append("        SelectAlbum(str), SetSearch(str), ApplyFilter,")
code.append("        ToggleSort, SetDensity(int),")
code.append("        OpenPhoto(int), PrevPhoto, NextPhoto, BackToGrid,")
code.append("        ToggleFav(int),")
code.append("        ToggleDark, SetAccent(str)")
code.append("    }")
code.append("")
code.append("    model {")
code.append("        // ---- 主题契约变量 ----")
code.append("        var dark_mode bool = true")
code.append("        var accent_color str = \"indigo\"")
code.append("")
code.append("        // ---- 视图形态 ----")
code.append("        var mode str = \"grid\"               // \"grid\" | \"view\"")
code.append("")
code.append("        // ---- 过滤 / 排序 / 密度 ----")
code.append("        var album str = \"all\"               // all | photos | screenshots | favorites")
code.append("        var search_q str = \"\"")
code.append("        var sort_dir str = \"desc\"           // \"desc\"=最新在前 | \"asc\"")
code.append("        var sort_label str = \"最新 ↓\"")
code.append("        var density int = 3                 // 网格列数：2 | 3 | 4")
code.append("")
code.append(f"        // ---- 真实照片种子列表（{len(items)} 张，来自 C:\\Users\\zhaop\\Pictures）----")
code.append(fmt_arr("p_ids", p_ids))
code.append(fmt_arr("p_titles", p_titles, is_str=True))
code.append(fmt_arr("p_tls", p_tls, is_str=True))
code.append(fmt_arr("p_albums", p_albums, is_str=True))
code.append(fmt_arr("p_dates", p_dates, is_str=True))
code.append(fmt_arr("p_keys", p_keys))
code.append(fmt_arr("p_favs", p_favs, is_bool=True))
code.append(fmt_arr("p_fulls", p_fulls, is_str=True))
code.append(fmt_arr("p_metas", p_metas, is_str=True))
code.append(fmt_arr("p_thumbs", p_thumbs, is_str=True))
code.append("")

with open(r"D:\autostack\.wt\lang-628\auto-lang\examples\ui\029-photo-gallery\scripts\app_head.at", "w", encoding="utf-8") as f:
    f.write("\n".join(code) + "\n")

print("Regenerated app_head.at with local file paths")
