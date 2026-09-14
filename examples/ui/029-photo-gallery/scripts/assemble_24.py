import json

head_file = r"D:\autostack\.wt\lang-628\auto-lang\examples\ui\029-photo-gallery\scripts\app_head.at"
out_file = r"D:\autostack\auto-lang\examples\ui\029-photo-gallery\src\front\app.at"

with open(head_file, "r", encoding="utf-8") as f:
    code = f.read()

# Only keep first 24 items in app_head.at to match the proven fast scale
def slice_arrays(text, limit=24):
    lines = text.splitlines()
    out_lines = []
    in_arr = False
    arr_lines = []
    for l in lines:
        if " = [" in l and "var p_" in l:
            in_arr = True
            out_lines.append(l)
            arr_lines = []
        elif in_arr:
            if l.strip() == "]":
                in_arr = False
                raw = " ".join(arr_lines)
                items = json.loads("[" + raw + "]")
                sliced = items[:limit]
                # format 4 per line
                for i in range(0, len(sliced), 4):
                    chunk = sliced[i:i+4]
                    formatted_chunk = []
                    for x in chunk:
                        if isinstance(x, str):
                            formatted_chunk.append(json.dumps(x, ensure_ascii=False))
                        elif isinstance(x, bool):
                            formatted_chunk.append("true" if x else "false")
                        else:
                            formatted_chunk.append(str(x))
                    row_s = ", ".join(formatted_chunk)
                    if i + 4 < len(sliced):
                        row_s += ","
                    out_lines.append("            " + row_s)
                out_lines.append("        ]")
            else:
                arr_lines.append(l.strip())
        else:
            out_lines.append(l)
    return "\n".join(out_lines)

code = slice_arrays(code, 24)

tail = '''
        // ---- 主列表与视图列表 ----
        var photos = [
            {
                id: 1,
                title: "照片",
                album: "photos",
                date: "2026-01-01",
                meta: "",
                fav: false,
                thumb: "",
                full: ""
            }
        ]
        var view_list = [
            {
                id: 1,
                title: "照片",
                album: "photos",
                date: "2026-01-01",
                meta: "",
                fav: false,
                thumb: "",
                full: ""
            }
        ]
        var view_ids = [1]
        var has_view bool = true

        // accent 五色点
        var accents = [
            { name: "coral", dot: "text-[#fb7185]" },
            { name: "ocean", dot: "text-[#38bdf8]" },
            { name: "sage", dot: "text-[#4ade80]" },
            { name: "amber", dot: "text-[#fbbf24]" },
            { name: "indigo", dot: "text-[#818cf8]" }
        ]

        // ---- 计数与文案 ----
        var view_label str = ""
        var cnt_all str = "0"
        var cnt_photos str = "0"
        var cnt_screenshots str = "0"
        var cnt_favorites str = "0"

        // ---- 查看器 ----
        var cur_id int = -1
        var cur_title str = ""
        var cur_meta str = ""
        var cur_full str = ""
        var prev_id int = -1
        var next_id int = -1
        var fav_label str = "♡ 收藏"
        var is_cur_fav bool = false
    }

    view {
        col {
            style: "w-full h-screen flex flex-col bg-background text-foreground overflow-hidden font-sans"

            // ════════════════════════════════════════════════════════
            // 模式 1：网格相册模式 (移动/平板全幅沉浸)
            // ════════════════════════════════════════════════════════
            if .mode == "grid" {
                col {
                    style: "w-full h-full flex flex-col overflow-hidden"

                    // ---- 顶部沉浸式导航栏（移动/平板风格 Segmented Control + 工具栏） ----
                    row {
                        style: "h-16 shrink-0 items-center justify-between px-6 border-b border-border bg-card/60 backdrop-blur-md gap-4"

                        // App 标题徽标
                        row {
                            style: "items-center gap-2.5"
                            text "相册" { style: "text-lg font-bold tracking-tight" }
                            text "Photos" { style: "text-xs px-2 py-0.5 rounded-full bg-primary/10 text-primary font-medium" }
                        }

                        // 平板风格分段胶囊控制条 (Segmented Album Tabs)
                        row {
                            style: "items-center p-1 bg-muted/60 rounded-xl border border-border/50 gap-1"
                            button {
                                onclick: .SelectAlbum("all")
                                style: if .album == "all" { "h-8 px-4 rounded-lg bg-background text-foreground text-xs font-semibold shadow-sm" } else { "h-8 px-4 rounded-lg bg-transparent text-muted-foreground hover:text-foreground text-xs font-medium" }
                                text "全部照片" { style: "text-xs" }
                                text .cnt_all { style: "text-[11px] ml-1.5 opacity-60" }
                            }
                            button {
                                onclick: .SelectAlbum("photos")
                                style: if .album == "photos" { "h-8 px-4 rounded-lg bg-background text-foreground text-xs font-semibold shadow-sm" } else { "h-8 px-4 rounded-lg bg-transparent text-muted-foreground hover:text-foreground text-xs font-medium" }
                                text "📷 图片" { style: "text-xs" }
                                text .cnt_photos { style: "text-[11px] ml-1.5 opacity-60" }
                            }
                            button {
                                onclick: .SelectAlbum("screenshots")
                                style: if .album == "screenshots" { "h-8 px-4 rounded-lg bg-background text-foreground text-xs font-semibold shadow-sm" } else { "h-8 px-4 rounded-lg bg-transparent text-muted-foreground hover:text-foreground text-xs font-medium" }
                                text "📱 截图" { style: "text-xs" }
                                text .cnt_screenshots { style: "text-[11px] ml-1.5 opacity-60" }
                            }
                            button {
                                onclick: .SelectAlbum("favorites")
                                style: if .album == "favorites" { "h-8 px-4 rounded-lg bg-background text-foreground text-xs font-semibold shadow-sm" } else { "h-8 px-4 rounded-lg bg-transparent text-muted-foreground hover:text-foreground text-xs font-medium" }
                                text "❤ 收藏" { style: "text-xs" }
                                text .cnt_favorites { style: "text-[11px] ml-1.5 opacity-60" }
                            }
                        }

                        // 右侧工具区：搜索、排序、网格密度调节、主题色
                        row {
                            style: "items-center gap-2.5"
                            input {
                                value: .search_q
                                oninput: .SetSearch
                                placeholder: "🔍 搜索照片…"
                                style: "w-44 h-8 text-xs px-3 bg-background border border-border/80 rounded-lg outline-none placeholder:text-muted-foreground focus:border-primary"
                            }
                            button (text: .sort_label, variant: "ghost") {
                                onclick: .ToggleSort
                                style: "h-8 px-2.5 text-xs rounded-lg border border-border text-muted-foreground hover:text-foreground"
                            }
                            // 缩放密度按钮 (2 / 3 / 4 列)
                            row {
                                style: "items-center bg-muted/60 border border-border/60 rounded-lg p-0.5 gap-0.5"
                                button (text: "2", variant: "ghost") {
                                    onclick: .SetDensity(2)
                                    style: if .density == 2 { "h-7 w-7 text-xs rounded-md bg-background text-foreground font-bold shadow-sm" } else { "h-7 w-7 text-xs rounded-md text-muted-foreground hover:text-foreground" }
                                }
                                button (text: "3", variant: "ghost") {
                                    onclick: .SetDensity(3)
                                    style: if .density == 3 { "h-7 w-7 text-xs rounded-md bg-background text-foreground font-bold shadow-sm" } else { "h-7 w-7 text-xs rounded-md text-muted-foreground hover:text-foreground" }
                                }
                                button (text: "4", variant: "ghost") {
                                    onclick: .SetDensity(4)
                                    style: if .density == 4 { "h-7 w-7 text-xs rounded-md bg-background text-foreground font-bold shadow-sm" } else { "h-7 w-7 text-xs rounded-md text-muted-foreground hover:text-foreground" }
                                }
                            }
                            if .dark_mode {
                                button {
                                    onclick: .ToggleDark
                                    style: "h-8 w-8 rounded-lg bg-transparent hover:bg-muted text-muted-foreground hover:text-foreground flex items-center justify-center"
                                    icon (name: "moon", size: 16)
                                }
                            } else {
                                button {
                                    onclick: .ToggleDark
                                    style: "h-8 w-8 rounded-lg bg-transparent hover:bg-muted text-muted-foreground hover:text-foreground flex items-center justify-center"
                                    icon (name: "sun", size: 16)
                                }
                            }
                        }
                    }

                    // 信息提示条 (状态与真实目录展示)
                    row {
                        style: "h-8 shrink-0 items-center justify-between px-6 bg-muted/20 border-b border-border/40"
                        text .view_label { style: "text-xs text-muted-foreground" }
                        text "源目录: C:\\Users\\zhaop\\Pictures\\" { style: "text-[11px] text-muted-foreground/60" }
                    }

                    // 照片网格区 (无缝现代网格设计，纯净呈现图片主体)
                    col {
                        style: "flex-1 min-h-0 overflow-y-auto p-5"
                        if .has_view {
                            if .density == 2 {
                                grid {
                                    cols: 2
                                    gap: 16
                                    for item in .view_list {
                                        col {
                                            style: "group rounded-2xl overflow-hidden bg-card border border-border/50 hover:border-primary/60 transition-all shadow-sm flex flex-col"
                                            // 图片区域（点击直开原图）
                                            button {
                                                onclick: .OpenPhoto(item.id)
                                                style: "w-full h-auto p-0 border-0 rounded-none bg-transparent block cursor-pointer"
                                                col {
                                                    style: "h-56 w-full overflow-hidden bg-muted"
                                                    image (src: item.thumb, alt: item.title, fit: "cover") {
                                                        style: "w-full h-full"
                                                    }
                                                }
                                            }
                                            // 悬浮信息条（标题 + 拍摄日期 + 收藏）
                                            row {
                                                style: "items-center justify-between px-3.5 py-2.5 bg-card"
                                                col {
                                                    style: "flex-1 min-w-0 pr-2"
                                                    text item.title { style: "text-xs font-semibold truncate" }
                                                    text item.date { style: "text-[11px] text-muted-foreground mt-0.5" }
                                                }
                                                if item.fav {
                                                    button (text: "❤", variant: "ghost") {
                                                        onclick: .ToggleFav(item.id)
                                                        style: "shrink-0 h-7 w-7 p-0 text-sm text-red-500 hover:bg-red-500/10 rounded-full"
                                                    }
                                                } else {
                                                    button (text: "♡", variant: "ghost") {
                                                        onclick: .ToggleFav(item.id)
                                                        style: "shrink-0 h-7 w-7 p-0 text-sm text-muted-foreground hover:text-foreground rounded-full"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            } else if .density == 3 {
                                grid {
                                    cols: 3
                                    gap: 14
                                    for item in .view_list {
                                        col {
                                            style: "group rounded-2xl overflow-hidden bg-card border border-border/50 hover:border-primary/60 transition-all shadow-sm flex flex-col"
                                            button {
                                                onclick: .OpenPhoto(item.id)
                                                style: "w-full h-auto p-0 border-0 rounded-none bg-transparent block cursor-pointer"
                                                col {
                                                    style: "h-48 w-full overflow-hidden bg-muted"
                                                    image (src: item.thumb, alt: item.title, fit: "cover") {
                                                        style: "w-full h-full"
                                                    }
                                                }
                                            }
                                            row {
                                                style: "items-center justify-between px-3 py-2 bg-card"
                                                col {
                                                    style: "flex-1 min-w-0 pr-2"
                                                    text item.title { style: "text-xs font-semibold truncate" }
                                                    text item.date { style: "text-[10px] text-muted-foreground mt-0.5" }
                                                }
                                                if item.fav {
                                                    button (text: "❤", variant: "ghost") {
                                                        onclick: .ToggleFav(item.id)
                                                        style: "shrink-0 h-6 w-6 p-0 text-xs text-red-500 hover:bg-red-500/10 rounded-full"
                                                    }
                                                } else {
                                                    button (text: "♡", variant: "ghost") {
                                                        onclick: .ToggleFav(item.id)
                                                        style: "shrink-0 h-6 w-6 p-0 text-xs text-muted-foreground hover:text-foreground rounded-full"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            } else if .density == 4 {
                                grid {
                                    cols: 4
                                    gap: 10
                                    for item in .view_list {
                                        col {
                                            style: "group rounded-xl overflow-hidden bg-card border border-border/50 hover:border-primary/60 transition-all shadow-sm flex flex-col"
                                            button {
                                                onclick: .OpenPhoto(item.id)
                                                style: "w-full h-auto p-0 border-0 rounded-none bg-transparent block cursor-pointer"
                                                col {
                                                    style: "h-36 w-full overflow-hidden bg-muted"
                                                    image (src: item.thumb, alt: item.title, fit: "cover") {
                                                        style: "w-full h-full"
                                                    }
                                                }
                                            }
                                            row {
                                                style: "items-center justify-between px-2.5 py-1.5 bg-card"
                                                col {
                                                    style: "flex-1 min-w-0 pr-1"
                                                    text item.title { style: "text-[11px] font-medium truncate" }
                                                    text item.date { style: "text-[10px] text-muted-foreground" }
                                                }
                                                if item.fav {
                                                    button (text: "❤", variant: "ghost") {
                                                        onclick: .ToggleFav(item.id)
                                                        style: "shrink-0 h-5 w-5 p-0 text-xs text-red-500 rounded-full"
                                                    }
                                                } else {
                                                    button (text: "♡", variant: "ghost") {
                                                        onclick: .ToggleFav(item.id)
                                                        style: "shrink-0 h-5 w-5 p-0 text-xs text-muted-foreground rounded-full"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            col {
                                style: "w-full flex flex-col items-center justify-center py-24 gap-3"
                                text "🖼️" { style: "text-5xl opacity-80" }
                                text "当前相册未找到照片" { style: "text-sm font-semibold" }
                                text "试着清空搜索关键字，或在上方切换到其他相簿" { style: "text-xs text-muted-foreground" }
                            }
                        }
                    }
                }
            } else {
                // ════════════════════════════════════════════════════════
                // 模式 2：沉浸式原画全屏查看态 (移动/平板 Lightbox)
                // ════════════════════════════════════════════════════════
                col {
                    style: "w-full h-full flex flex-col bg-black text-white relative select-none"

                    // 悬浮顶栏（返回 + 标题与参数 + 收藏）
                    row {
                        style: "h-16 shrink-0 items-center justify-between px-6 bg-gradient-to-b from-black/80 to-transparent z-10 gap-4"
                        button {
                            onclick: .BackToGrid
                            style: "h-9 px-3.5 rounded-full bg-white/10 hover:bg-white/20 text-white flex items-center gap-1.5 backdrop-blur-md border border-white/10"
                            icon (name: "chevron-left", size: 16)
                            text "返回网格" { style: "text-xs font-medium" }
                        }
                        col {
                            style: "items-center text-center"
                            text .cur_title { style: "text-sm font-semibold tracking-wide text-white truncate max-w-md" }
                            text .cur_meta { style: "text-[11px] text-white/60 mt-0.5" }
                        }
                        button {
                            onclick: .ToggleFav(.cur_id)
                            style: "h-9 px-3.5 rounded-full bg-white/10 hover:bg-white/20 text-white flex items-center gap-1.5 backdrop-blur-md border border-white/10"
                            text .fav_label { style: "text-xs font-medium" }
                        }
                    }

                    // 主图展示区（无损原图直显，contain 完整视口呈现）
                    col {
                        style: "flex-1 min-h-0 flex items-center justify-center p-4 relative"
                        image (src: .cur_full, alt: .cur_title, fit: "contain") {
                            style: "max-w-full max-h-full"
                        }
                    }

                    // 移动端风格悬浮胶囊控制底栏 (Floating Navigation Capsule)
                    row {
                        style: "h-20 shrink-0 items-center justify-center px-6 bg-gradient-to-t from-black/80 to-transparent z-10"
                        row {
                            style: "items-center px-4 py-2 rounded-full bg-white/15 backdrop-blur-xl border border-white/15 gap-4 shadow-2xl"
                            button {
                                onclick: .PrevPhoto
                                style: "h-9 px-4 rounded-full bg-white/10 hover:bg-white/25 text-white flex items-center gap-1.5 border border-white/10"
                                icon (name: "chevron-left", size: 16)
                                text "上一张" { style: "text-xs font-medium" }
                            }
                            text "•" { style: "text-white/30 text-xs" }
                            button {
                                onclick: .NextPhoto
                                style: "h-9 px-4 rounded-full bg-white/10 hover:bg-white/25 text-white flex items-center gap-1.5 border border-white/10"
                                text "下一张" { style: "text-xs font-medium" }
                                icon (name: "chevron-right", size: 16)
                            }
                        }
                    }
                }
            }
        }
    }

    on {
        .Init -> {
            var plist = []
            var i int = 0
            for i < .p_ids.len() {
                var p = {
                    id: .p_ids[i],
                    title: .p_titles[i],
                    album: .p_albums[i],
                    date: .p_dates[i],
                    meta: .p_metas[i],
                    fav: .p_favs[i],
                    thumb: .p_thumbs[i],
                    full: .p_fulls[i]
                }
                plist.push(p)
                i = i + 1
            }
            .photos = plist
            .ApplyFilter()
        }

        .SelectAlbum(a) -> {
            .album = a
            .ApplyFilter()
        }

        .SetSearch(q) -> {
            .search_q = q
            .ApplyFilter()
        }

        .ApplyFilter -> {
            var hits = []
            var used = []
            let ql str = .search_q.lower()
            var i int = 0
            for i < .p_ids.len() {
                var album_match bool = false
                if .album == "all" { album_match = true }
                if .album == "favorites" { album_match = .p_favs[i] }
                if .album == "photos" { album_match = (.p_albums[i] == "photos") }
                if .album == "screenshots" { album_match = (.p_albums[i] == "screenshots") }

                let tl = .p_tls[i]
                let tmatch = (ql == "") || tl.contains(ql)
                if album_match && tmatch {
                    hits.push(i)
                    used.push(0)
                }
                i = i + 1
            }

            var vlist = []
            var vids = []
            var n int = hits.len()
            var placed int = 0
            for placed < n {
                var best int = -1
                var best_key int = 0
                var bi int = 0
                for bi < n {
                    if used[bi] == 0 {
                        var k int = .p_keys[hits[bi]]
                        var take bool = false
                        if best == -1 { take = true }
                        else {
                            if .sort_dir == "asc" { take = k < best_key }
                            if .sort_dir == "desc" { take = k > best_key }
                        }
                        if take {
                            best = bi
                            best_key = k
                        }
                    }
                    bi = bi + 1
                }
                if best >= 0 {
                    let si = hits[best]
                    var p = {
                        id: .p_ids[si],
                        title: .p_titles[si],
                        album: .p_albums[si],
                        date: .p_dates[si],
                        meta: .p_metas[si],
                        fav: .p_favs[si],
                        thumb: .p_thumbs[si],
                        full: .p_fulls[si]
                    }
                    vlist.push(p)
                    vids.push(.p_ids[si])
                    used[best] = 1
                }
                placed = placed + 1
            }

            .view_list = vlist
            .view_ids = vids
            .has_view = (vids.len() > 0)

            // 相簿全量计数
            var c_all int = 0
            var c_pho int = 0
            var c_scr int = 0
            var c_fav int = 0
            var k int = 0
            for k < .p_ids.len() {
                c_all = c_all + 1
                let alb = .p_albums[k]
                if alb == "photos" { c_pho = c_pho + 1 }
                if alb == "screenshots" { c_scr = c_scr + 1 }
                if .p_favs[k] { c_fav = c_fav + 1 }
                k = k + 1
            }
            .cnt_all = c_all.str()
            .cnt_photos = c_pho.str()
            .cnt_screenshots = c_scr.str()
            .cnt_favorites = c_fav.str()

            var cur_label str = ""
            if .album == "all" { cur_label = "全部" }
            if .album == "photos" { cur_label = "图片" }
            if .album == "screenshots" { cur_label = "截图" }
            if .album == "favorites" { cur_label = "收藏" }

            if .search_q == "" {
                .view_label = vids.len().str() + " 项内容 · " + cur_label
            } else {
                .view_label = vids.len().str() + " 项内容 · 搜索 “" + .search_q + "” · " + cur_label
            }
        }

        .ToggleSort -> {
            if .sort_dir == "desc" {
                .sort_dir = "asc"
                .sort_label = "最早 ↑"
            } else {
                .sort_dir = "desc"
                .sort_label = "最新 ↓"
            }
            .ApplyFilter()
        }

        .SetDensity(d) -> {
            .density = d
        }

        .OpenPhoto(pid) -> {
            var pos int = -1
            var n int = .view_ids.len()
            var i int = 0
            for i < n {
                if .view_ids[i] == pid { pos = i }
                i = i + 1
            }
            if pos >= 0 {
                .mode = "view"
                .cur_id = pid
                var prev_pos int = pos - 1
                if prev_pos < 0 { prev_pos = n - 1 }
                var next_pos int = pos + 1
                if next_pos >= n { next_pos = 0 }
                .prev_id = .view_ids[prev_pos]
                .next_id = .view_ids[next_pos]

                var si int = -1
                var j int = 0
                for j < .p_ids.len() {
                    if .p_ids[j] == pid { si = j }
                    j = j + 1
                }
                if si >= 0 {
                    .cur_title = .p_titles[si]
                    .cur_full = .p_fulls[si]
                    .cur_meta = .p_metas[si]
                    .is_cur_fav = .p_favs[si]
                    if .is_cur_fav {
                        .fav_label = "❤ 已收藏"
                    } else {
                        .fav_label = "♡ 收藏"
                    }
                }
            }
        }

        .PrevPhoto -> { .OpenPhoto(.prev_id) }
        .NextPhoto -> { .OpenPhoto(.next_id) }
        .BackToGrid -> { .mode = "grid" }

        .ToggleFav(id) -> {
            var i int = 0
            for i < .p_ids.len() {
                if .p_ids[i] == id {
                    .p_favs[i] = !.p_favs[i]
                }
                i = i + 1
            }
            .ApplyFilter()
            if .mode == "view" && .cur_id == id {
                var si int = -1
                var j int = 0
                for j < .p_ids.len() {
                    if .p_ids[j] == id { si = j }
                    j = j + 1
                }
                if si >= 0 {
                    .is_cur_fav = .p_favs[si]
                    if .is_cur_fav {
                        .fav_label = "❤ 已收藏"
                    } else {
                        .fav_label = "♡ 收藏"
                    }
                }
            }
        }

        .ToggleDark -> { .dark_mode = !.dark_mode }
        .SetAccent(name) -> { .accent_color = name }
    }
}
'''

with open(out_file, "w", encoding="utf-8") as f:
    f.write(code + tail)

print("Successfully wrote 24-photo real gallery app.at!")
