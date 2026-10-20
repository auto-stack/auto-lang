//! # Render Support Registry (Plan 280)
//!
//! Static registry that maps AURA tag names to their support level in the
//! iced backend. Used by MCP tools to annotate snapshot output and provide
//! diagnostic information about rendering issues.

/// Support level for an AURA tag in the iced backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportLevel {
    /// Fully supported with all props and events
    Full,
    /// Supported but some props are ignored
    Partial,
    /// Tag is recognized but renders as a fallback (e.g., Column instead of Grid)
    Fallback,
    /// Tag is completely unrecognized
    Unsupported,
}

impl SupportLevel {
    /// Plan 435 P3:从 schema backends.iced 字符串解析级别;
    /// unknown/none/未识别返回 None(调用方保留静态表值)。
    pub fn parse_name(s: &str) -> Option<SupportLevel> {
        match s {
            "full" => Some(SupportLevel::Full),
            "partial" => Some(SupportLevel::Partial),
            "fallback" => Some(SupportLevel::Fallback),
            "unsupported" => Some(SupportLevel::Unsupported),
            _ => None,
        }
    }
}

/// Per-tag support info
#[derive(Debug, Clone)]
pub struct TagSupport {
    pub level: SupportLevel,
    /// Props that the iced backend ignores for this tag
    pub ignored_props: &'static [&'static str],
    /// Human-readable description of the limitation
    pub note: &'static str,
}

impl TagSupport {
    const fn full() -> Self {
        TagSupport {
            level: SupportLevel::Full,
            ignored_props: &[],
            note: "",
        }
    }

    const fn partial(ignored: &'static [&'static str], note: &'static str) -> Self {
        TagSupport {
            level: SupportLevel::Partial,
            ignored_props: ignored,
            note,
        }
    }

    const fn fallback(ignored: &'static [&'static str], note: &'static str) -> Self {
        TagSupport {
            level: SupportLevel::Fallback,
            ignored_props: ignored,
            note,
        }
    }
}

/// Look up the render support level for an AURA tag.
///
/// Plan 435 P3 数据流翻转:**级别以 docs/schema/aura.at 的 backends.iced 为权威**
/// (三级折叠解析:精确→别名→折叠键);本文件静态表降级为详情来源
/// (ignored_props/note,这些不在 schema 里)与 schema 缺失时的回退。
/// 围栏测试保证静态表与 schema 级别一致(schema 即从本表提取,再生成闭环),
/// 以及反向:schema 有 iced 级别的元素必须在静态表有臂(P6-3/D3)。
pub fn get_support(tag: &str) -> TagSupport {
    // F-759-R1（PLAN-759 落地后审核）:详情查询先折叠到 schema canonical——
    // 别名/kebab 形态此前直落 unknown 臂,与 canonical 详情不一致
    // （command-input 返回空 note vs command_input 返回错误详情）。
    // schema 缺失(solo/解析失败)时保留原词查询。
    let schema = auto_ui_compiler::aura::default_schema_cached();
    let canonical = schema.and_then(|s| s.resolve_tag(tag).map(|(c, _)| c));
    let mut support = get_support_details(canonical.unwrap_or(tag));
    if let (Some(schema), Some(canonical)) = (schema, canonical) {
        if let Some(meta) = schema.meta.get(canonical) {
            if let Some(level) = SupportLevel::parse_name(&meta.backends.iced) {
                support.level = level;
                // Plan 435 P6-3(D3):overlay 生效时,静态兜底臂的
                // "unknown tag" note 与覆盖后的级别自相矛盾
                // (Full + unknown),清空。
                if support.note.starts_with("unknown tag") {
                    support.note = "";
                }
            }
        }
    }
    support
}

/// 静态详情表(原 get_support 主体):每 tag 的级别 + ignored_props + note。
fn get_support_details(tag: &str) -> TagSupport {
    match tag {
        // ── Core layout (Full) ──
        "col" | "column" => TagSupport::full(),
        "row" => TagSupport::full(),
        // Plan 463 T5: taskbar —— 桌面 shell 底栏（Iced 映射 row 语义,Full）。
        "taskbar" => TagSupport::full(),
        // Plan 462: virtual_window —— iced 实现于 ui/iced/virtual_window.rs,Full
        // (P6-3 围栏要求 schema 元素的 iced 级别在静态详情表有臂)。
        "virtual_window" => TagSupport::full(),
        // Plan 497: window_thumbnail —— iced 实现于 renderer 渲染臂(快照
        // 缓存直绘 + fallback icon),Full(P6-3 同上)。
        "window_thumbnail" => TagSupport::full(),
        // PLAN-012 W3: workspace_preview —— iced 渲染臂(宿主合成 tile 直绘),
        // Full(P6-3 同 window_thumbnail)。
        "workspace_preview" => TagSupport::full(),
        // PLAN-760: remote-frame —— iced 渲染臂(注册表最新 DisplayList →
        // displaylist_element + mouse_area 输入包装),Full(P6-3 同上)。
        "remote-frame" => TagSupport::full(),
        "center" => TagSupport::full(),
        "container" | "div" => TagSupport::full(),

        // ── Core text (Full) ──
        "text" | "label" | "h1" | "h2" | "h3" | "p" | "span" => TagSupport::full(),

        // ── Core widgets (Full) ──
        "checkbox" | "check" => TagSupport::full(),
        "progress" => TagSupport::full(),
        "spacer" => TagSupport::full(),
        // Plan 423 P3: disabled/disabled-if 已实现(iced on_press=None + 灰
        // 样式),消 Plan 402 的 "always clickable" 警告。
        "button" | "btn" => TagSupport::full(),
        "input" => TagSupport::partial(
            &["type", "maxlength", "min", "max", "step", "pattern"],
            "basic text input only; props like type/maxlength are ignored",
        ),
        "textarea" => TagSupport::partial(
            &["rows", "cols", "maxlength", "resize"],
            "limited styling; most configuration props ignored",
        ),
        "divider" | "hr" => TagSupport::partial(
            &["style", "class"],
            "hardcoded appearance; custom style/class props ignored",
        ),
        "img" | "image" => TagSupport::partial(
            &["src", "alt", "width", "height", "fit"],
            "placeholder only; no actual image loading",
        ),
        // os-007（P534-D4）：avatar 家族——容器转换子件组合（image/fallback
        // 臂实装），src/alt 直传 avatar-image 形态。
        "avatar" => TagSupport::full(),
        "avatarimage" | "avatar-image" => TagSupport::full(),
        "avatarfallback" | "avatar-fallback" => TagSupport::full(),

        // ── Fallback: known AURA tags not supported by iced ──
        "grid" => TagSupport::partial(
            &["rows", "rowspan", "colspan"],
            "decomposed into a Column of Rows of `cols` cells (iced has no native grid); rows/rowspan/colspan ignored",
        ),
        "grid-item" => TagSupport::full(),
        // G4 (411 P2-B): the table below mirrors what the view_builder
        // ACTUALLY converts today — the old "not implemented" rows here
        // produced the 60 false-positive `unknown tag` errors in
        // autoui_check (scroll/aside/header/icon/badge/table/nav-link are
        // all real conversions now; see aura_view_builder tag dispatch).
        // PLAN-656: scroll-pane 语义——axis 三值/scrollbar 策略/controller/
        // onscroll 双端生效（hidden=Scrollbar::hidden() 结构性无命中区）；
        // legacy direction 映射 axis（axis 优先）。
        "scroll" | "scrollable" => TagSupport::full(),
        // PLAN-656 T-06: capability-test 专用 synthetic managed content
        //（logical extent 自绘 widget；非 public widget）。
        "scroll_test_content" => TagSupport::full(),
        "aside" | "main" | "header" | "nav" | "section" | "footer" | "article" => {
            TagSupport::full()
        }
        // Plan 482: nav 组件族 —— nav-item/nav-group 全支持（契约类三态 +
        // icon/desc/badge 槽 + to/onclick 双模式）；nav 容器另支持 search:*
        // 集成搜索行；nav-link 已被 nav-item 取代（保留兼容）。
        "nav-item" | "nav_item" => TagSupport::full(),
        "nav-group" | "nav_group" => TagSupport::full(),
        "nav-link" | "nav_link" => TagSupport::partial(
            &["exact", "disabled"],
            "DEPRECATED (Plan 482): renders as a link-styled button (label + optional icon); use nav-item instead",
        ),
        // Plan 561: sidebar 组件族（shadcn 契约子集）—— 走 aura_view_builder
        // 契约类构建臂 + 通用 Column/Button 渲染路径；分区/分组折叠/菜单层级/
        // menu_button 三态 + to: 路由全支持。root 的 side 放置与
        // collapsible=icon 折叠轨道属 VM 子集外（sidebar-family-and-nav-retirement §3.3）。
        "sidebar" => TagSupport::partial(
            &["side"],
            "Plan 561: contract subset — renders via contract Column (w-64); side placement and collapsible=icon rail flatten (VM subset)",
        ),
        "sidebar_provider" | "sidebar-provider" => TagSupport::full(),
        "sidebar_header" | "sidebar-header" => TagSupport::full(),
        "sidebar_footer" | "sidebar-footer" => TagSupport::full(),
        "sidebar_content" | "sidebar-content" => TagSupport::full(),
        "sidebar_separator" | "sidebar-separator" => TagSupport::full(),
        "sidebar_inset" | "sidebar-inset" => TagSupport::full(),
        "sidebar_group" | "sidebar-group" => TagSupport::full(),
        "sidebar_group_label" | "sidebar-group-label" => TagSupport::full(),
        "sidebar_group_content" | "sidebar-group-content" => TagSupport::full(),
        "sidebar_group_action" | "sidebar-group-action" => TagSupport::full(),
        "sidebar_menu" | "sidebar-menu" => TagSupport::full(),
        "sidebar_menu_item" | "sidebar-menu-item" => TagSupport::full(),
        "sidebar_menu_action" | "sidebar-menu-action" => TagSupport::full(),
        "sidebar_menu_badge" | "sidebar-menu-badge" => TagSupport::full(),
        "sidebar_menu_button" | "sidebar-menu-button" => TagSupport::partial(
            &["tooltip"],
            "Plan 561: contract subset — active/hover/disabled states + to: routing; tooltip deferred (P548-D2 debt)",
        ),
        "sidebar_menu_sub" | "sidebar-menu-sub" => TagSupport::full(),
        "sidebar_menu_sub_item" | "sidebar-menu-sub-item" => TagSupport::full(),
        "sidebar_menu_sub_button" | "sidebar-menu-sub-button" => TagSupport::full(),
        // PLAN-009 P1: native terminal component (auto-term engine grid).
        "terminal" | "Terminal" => TagSupport::full(),
        "badge" | "chip" => TagSupport::partial(
            &["class"],
            "shadcn-style badge with variant colors; custom class limited",
        ),
        "table" | "thead" | "tbody" | "tfoot" | "tr" | "td" | "th" => TagSupport::partial(
            &["style", "align", "colspan", "rowspan"],
            "renders a real table; header/divider/padding details not pixel-aligned (411 P2-A④)",
        ),
        "code-block" | "codeblock" | "code_block" | "code" | "code-pane" => TagSupport::partial(
            &["language"],
            "renders code text with a language label; Prism palette not aligned (411 P2-A①)",
        ),
        "code_editor" | "codeEditor" | "codeeditor" => TagSupport::full(),
        // Plan 442 A4: svg 子树序列化为 SVG 文档,经 resvg(svg::Handle 缓存)
        // 渲染;单色 currentColor 文档走画时着色。动态属性/动画不支持。
        "svg" => TagSupport::partial(
            &[],
            "serialized SVG document rendered via resvg; literal attrs only (viewBox/d/fill/...), dynamic attrs and animation unsupported",
        ),
        // Plan 019 批次七: feature "autodown" 下经 autodown-core crate 真渲染
        // (parse_blocks -> panel tree -> View);无 feature 时维持 D-GAP-3 textarea 降级。
        // Plan 040: 契约扩展 props（streaming/placeholder_block_id/
        // placeholder_height/scroll_sync/placeholder）。043/044/045 收口后：
        // scroll_sync 与 placeholder_*（ghost）真消费；table_col_widths
        //（列宽状态 Map）+ oncolresize（落定事件）只读臂真消费；streaming
        // 恒按 final、编辑臂 placeholder（空态文案）读取后忽略（豁免保留）。
        "autodown_editor" | "autodowneditor" | "autodown" | "markdown_editor" | "markdown" => {
            TagSupport::partial(
                &["content", "final"],
                "true rendering via autodown-core parse_blocks under feature `autodown` (plan 019); textarea degradation otherwise (D-GAP-3); scroll_sync consumed since PLAN-043, ghost placeholder_* since PLAN-044, table_col_widths/oncolresize since PLAN-045; streaming stays final-only on VM v1",
            )
        }
        "square" => TagSupport::full(),
        "preview-card" | "previewcard" | "preview_card"
        | "component-card" | "componentcard" | "component_card"
        | "category-section" | "category_section" => TagSupport::partial(
            &["style"],
            "widgets-gallery demo component; custom style limited",
        ),
        "toast-provider" | "toast_provider" | "toaster" => TagSupport::partial(
            &[],
            "toast overlay host; toast styling simplified",
        ),
        "list" | "list-item" => TagSupport::fallback(
            &["style"],
            "list component not implemented — renders as Column",
        ),
        "select" | "dropdown" => TagSupport::fallback(
            &["value", "options", "placeholder", "style"],
            "select/dropdown not implemented — renders as Column",
        ),
        "radio" => TagSupport::fallback(
            &["value", "group", "checked", "style"],
            "radio button not implemented",
        ),
        // PLAN-661 T-04：iced 侧 iced::widget::slider 真渲染（renderer
        // Slider 臂 + SliderChangeHandler 派发，T-02/T-03 落地）——
        // fallback（"not implemented" 占位）升级 partial。
        "slider" => TagSupport::partial(
            &["value", "min", "max", "step", "style"],
            "iced::widget::slider native rendering (PLAN-661 T-02/T-03: SliderChangeHandler dispatch + aura convert_slider arm)",
        ),
        "toggle" | "switch" => TagSupport::fallback(
            &["checked", "style"],
            "toggle/switch not implemented",
        ),
        "card" => TagSupport::fallback(
            &["style"],
            "card not implemented — renders as Column",
        ),
        "alert" | "toast" | "notification" => TagSupport::fallback(
            &["variant", "title", "message", "style"],
            "alert/toast not implemented",
        ),
        // PLAN-641 T-03：tabs 族升级为真折叠（convert_tabs）——有状态
        // View::Tabs（选中态受控 + onselect/onclick 事件 + variant 形态）。
        // 未登记 props 静默忽略（position/orientation VM 子集外）。
        "tabs" | "tab" | "tabs-list" | "tabs-trigger" | "tabs-content" => TagSupport::partial(
            &["style", "active", "value", "defaultvalue", "default", "variant", "text", "label", "onselect", "onclick"],
            "PLAN-641: folds into stateful Tabs (labels/contents/selected); variant default|enclosed; click switches via onselect handler receiving selected index; position/orientation not supported on VM track",
        ),
        "accordion" | "accordion-item" | "accordion-trigger" | "accordion-content" => {
            TagSupport::fallback(
                &["style", "open", "value"],
                "accordion not implemented — renders as Column",
            )
        }
        "form" | "field" | "form-item" => TagSupport::fallback(
            &["action", "method", "style"],
            "form not implemented — renders as Column",
        ),
        "modal" | "dialog" | "sheet" | "popover" | "overlay" => TagSupport::fallback(
            &["open", "style", "placement"],
            "modal/dialog not implemented — renders as Column",
        ),
        // F-759-R1（PLAN-759 落地后审核校准）:carousel/command 族不再统一
        // fallback——aura_view_builder 对各子件有真实转换臂（command_input
        // →convert_input、command_item→convert_button、carousel_previous/
        // next→Button、carousel_content→Row 等）,级别/ignored/文案按逐
        // 子件实际消费面登记;仅真正缺渲染面的子件（command_empty 恒渲染
        // Empty）保留降级。缺高阶交互（palette 过滤/键盘导航/autoplay）不
        // 等于子件的 input/button/样式属性不被消费。
        // carousel 根:静态 VM 布局(Column),align/loop/orientation 全忽略。
        "carousel" => TagSupport::partial(
            &["align", "loop", "orientation"],
            "static VM layout — renders as Column of slides; no autoplay/loop/swipe; orientation has no effect (root is always a Column)",
        ),
        // carousel_content:真 Row,但无滚动/吸附轨道语义。
        "carousel_content" => TagSupport::partial(
            &[],
            "renders as plain Row (slide track); no scroll/snap behavior",
        ),
        "carousel_item" => TagSupport::full(),
        "carousel_previous" | "carousel_next" => TagSupport::full(),
        // command 根:真 Column(class/style 消费);palette 状态属 app/VM 逻辑。
        "command" => TagSupport::partial(
            &["query", "placeholder"],
            "renders as styled Column; palette state (query binding) is app logic — root query/placeholder props ignored (place them on command_input)",
        ),
        // command_empty:唯一真降级——恒渲染 Empty,空态文案/子树不呈现。
        "command_empty" => TagSupport::fallback(
            &[],
            "renders as Empty — empty-state text/children are not displayed on the iced track (visibility filtering is app logic)",
        ),
        "command_group" => TagSupport::full(),
        // command_input:与 input 臂同口径(placeholder/text/value/class/style
        // /width/password/onchange/onenter 消费;type 仅 password 义 honored)。
        "command_input" => TagSupport::partial(
            &["type", "maxlength", "min", "max", "step", "pattern"],
            "renders as native text input (placeholder/value/style consumed, same as input); palette filtering & keyboard navigation are app logic, not widget behavior",
        ),
        // command_item:真 ghost Button;convert_button 不读 value。
        "command_item" => TagSupport::partial(
            &["value"],
            "renders as ghost Button (children/text, style, onclick consumed); no palette selection highlight — selection state is app logic",
        ),
        "command_list" => TagSupport::partial(
            &[],
            "renders as plain Column; no built-in scrolling or scroll-to-selected",
        ),
        "command_separator" => TagSupport::partial(
            &["style", "class"],
            "hardcoded 1px divider; custom style/class props ignored",
        ),
        "command_shortcut" => TagSupport::full(),
        "navigation" | "breadcrumb" => TagSupport::fallback(
            &["style", "items"],
            "navigation component not implemented — renders as Column",
        ),
        // Plan 484: 原生 chart tag 退役(裸名由 official 包 Auto 组件承接,
        // 不再落到此表);Plan 563: canvas 升 full——状态驱动画布
        // (AbstractView::Canvas → iced canvas::Program 直绘,场景数据
        // 契约见 schema 描述)。
        "canvas" => TagSupport::full(),
        // Plan 484: hover 命中区——iced mouse_area 事件转发,与 col/row 同级
        "mouse-area" => TagSupport::full(),
        // Plan 547: 后端中立图片面——iced ImageSurface widget 直渲(媒体
        // 票据只查 worker 已发布 rendition,几何/事件齐备)。
        "imagesurface" | "image-surface" | "image_surface" | "ImageSurface" => TagSupport::full(),
        // PLAN-617 T-19: `video` 由 fallback 提升为 partial —— iced 端有真实渲染面
        // （`ui/mpv/widget.rs`：libmpv → 持久纹理 → 自定义 shader widget）。
        // 仍标 partial 而非 full，因为有三项**真实限制**（不谎报能力）：
        // ① 原生解码需要运行库（`libmpv-2.dll`），本机没有时走诚实降级面板；
        // ② 帧由应用既有的 tick 驱动，应用不声明 tick 就停在首帧；
        // ③ 浏览器专有的透传属性（`poster`/`preload`/`playsinline`/`autoplay`）
        //    在 iced 端无对应语义，故列入 ignored。
        // 上行事件（ontimeupdate 等）由渲染面按帧采集，见 `ui/mpv/widget.rs`。
        "video" | "Video" => TagSupport::partial(
            &["poster", "preload", "playsinline", "autoplay", "controls"],
            "原生命中播放面（libmpv）；需运行库与 tick 驱动，缺失时走降级面板",
        ),
        // `audio`/`media` 仍无渲染面（本计划不做音频播放，见 §1 N3）。
        "audio" | "media" => TagSupport::fallback(
            &["src", "controls", "autoplay", "style"],
            "媒体/音频组件未实现（PLAN-617 只做 video）",
        ),
        "skeleton" | "loading" => TagSupport::fallback(
            &["style", "variant"],
            "skeleton/loading not implemented",
        ),
        "tooltip" => TagSupport::fallback(
            &["content", "placement", "style"],
            "tooltip not implemented",
        ),

        // ── Unknown tags ──
        _ => TagSupport::fallback(
            &[],
            "unknown tag — no handler in view builder, renders as Column fallback",
        ),
    }
}

/// Check if a tag is fully supported.
pub fn is_full(tag: &str) -> bool {
    get_support(tag).level == SupportLevel::Full
}

/// Check if a tag has any level of issue (not Full).
pub fn has_issue(tag: &str) -> bool {
    get_support(tag).level != SupportLevel::Full
}

#[cfg(test)]
mod tests {
    use super::*;

    /// F-759-R1:kebab 别名与 canonical 必须返回同一份详情
    /// （级别/ignored/note 三围一致;get_support 经 schema 折叠归一）。
    #[test]
    fn command_carousel_alias_forms_share_details() {
        let pairs = [
            ("command_input", "command-input"),
            ("command_item", "command-item"),
            ("command_empty", "command-empty"),
            ("command_group", "command-group"),
            ("command_list", "command-list"),
            ("command_separator", "command-separator"),
            ("command_shortcut", "command-shortcut"),
            ("carousel_content", "carousel-content"),
            ("carousel_item", "carousel-item"),
            ("carousel_previous", "carousel-previous"),
            ("carousel_next", "carousel-next"),
        ];
        for (canon, alias) in pairs {
            let a = get_support(canon);
            let b = get_support(alias);
            assert_eq!(a.level, b.level, "{canon} vs {alias}: level");
            assert_eq!(
                a.ignored_props, b.ignored_props,
                "{canon} vs {alias}: ignored"
            );
            assert_eq!(a.note, b.note, "{canon} vs {alias}: note");
        }
    }

    /// F-759-R1:诊断的 ignored_props 不得包含实际被消费的 props。
    /// command_input 真转原生 Input、command_item 真转 Button——
    /// placeholder/value/style 被消费,列入 ignored 即误报。
    #[test]
    fn command_family_ignores_do_not_overclaim() {
        let si = get_support("command_input");
        for consumed in ["placeholder", "value", "style", "text", "class"] {
            assert!(
                !si.ignored_props.contains(&consumed),
                "command_input 实际消费 {consumed},不得列入 ignored(当前 {:?})",
                si.ignored_props
            );
        }
        let item = get_support("command_item");
        for consumed in ["text", "label", "style", "onclick", "class"] {
            assert!(
                !item.ignored_props.contains(&consumed),
                "command_item 实际消费 {consumed},不得列入 ignored(当前 {:?})",
                item.ignored_props
            );
        }
        // 逐子件校准:不再统一 fallback;唯一真降级是 command_empty
        // （恒渲染 Empty,空态文案不呈现）。
        for tag in [
            "command_input",
            "command_item",
            "command",
            "command_list",
            "command_separator",
            "carousel",
            "carousel_content",
        ] {
            assert_ne!(
                get_support(tag).level,
                SupportLevel::Fallback,
                "{tag} 有真实转换臂,不得标 fallback"
            );
        }
        assert_eq!(get_support("command_empty").level, SupportLevel::Fallback);
        // 有真实转换臂且声明 props 全消费的子件应为 Full。
        for tag in [
            "command_group",
            "command_shortcut",
            "carousel_item",
            "carousel_previous",
            "carousel_next",
        ] {
            assert_eq!(get_support(tag).level, SupportLevel::Full);
        }
    }

    /// F-759-R1:诊断与实际 View 对照——生产构建管线真实转出原生 Input,
    /// placeholder/value/style 全落位（759 落地后审核 render_probe 的仓内化;
    /// 修复前该形态被诊断为"忽略这些属性、渲染为 Column"）。
    #[test]
    fn command_input_builds_native_input_matching_diagnostics() {
        use crate::ui::interpreter::DynamicMessage;
        use crate::ui::view::View;

        fn find_inputs<'a>(
            view: &'a View<DynamicMessage>,
            out: &mut Vec<(&'a str, &'a str, bool)>,
        ) {
            match view {
                View::Input {
                    placeholder,
                    value,
                    style,
                    ..
                } => out.push((placeholder, value, style.is_some())),
                View::Column { children, .. } | View::Row { children, .. } => {
                    for c in children {
                        find_inputs(c, out);
                    }
                }
                View::Container { child, .. } => find_inputs(child, out),
                _ => {}
            }
        }

        let source = r#"widget App {
            model { var query str = "hello" }
            view { col { command-input (placeholder: "Search", value: .query) {} } }
        }"#;
        let component =
            crate::build_dynamic_component(source, None).expect("build command-input fixture");
        let (view, _, _) = component.view_with_debug_gated(false);
        let mut inputs = Vec::new();
        find_inputs(&view, &mut inputs);
        assert!(
            inputs
                .iter()
                .any(|(p, v, styled)| *p == "Search" && *v == "hello" && *styled),
            "command-input 须转出原生 Input(placeholder/value/style 落位),实际: {:?}",
            inputs
        );
    }
}
