//! Plan 561: sidebar_* class-token contract — VM 端契约子集的 class 单一来源。
//!
//! 与 nav_contract.rs（Plan 482）的差异：sidebar scaffold 资产
//! （`auto-man/assets/shadcn-ui/sidebar/*.vue` + `index.ts` cva）按 Plan 548 保持
//! shadcn 原版逐字不改，因此本契约常量不是"双端嵌入同一字符串"，而是
//! **VM 侧精选子集**——只收 iced 可解析的几何/颜色 token；web-only 机制
//! （group-data/peer-data 父级选择器、`[&>svg]` 子代选择器、CSS 变量任意值、
//! transition/focus-visible/cursor/select-none）一律不进常量。防漂移锚改为
//! **逐 token 资产包含断言**（每个 parity token 必须逐字出现在对应资产文件里）。
//!
//! - VM 侧：`ui/aura_view_builder` 用这些常量构建 sidebar 族视图样式
//!   （token 必须可被 `StyleClass::parse_single` 解析）。
//! - Vue 侧：codegen 直通 scaffold 资产（Plan 548 臂），不消费本契约；
//!   本契约是 VM 端与原版视觉对拍的锚。

// ── 容器与分区 ──────────────────────────────────────────────────────

/// sidebar 容器（Sidebar.vue C-3 内层 + collapsible=none 分支的公共几何）。
/// 宽度不入契约：web 端 `w-[var(--sidebar-width)]`（16rem CSS 变量），VM 端由
/// builder 直接给固定宽（16rem = w-64 等价，见 WIDTH_VM）。
pub const SIDEBAR_BASE: &str = "flex h-full flex-col bg-sidebar text-sidebar-foreground";
/// VM 端容器宽（web `--sidebar-width: 16rem` 的刻度等价；VM 专适配件，不做资产锚）。
pub const WIDTH_VM: &str = "w-64";
/// variant=floating 的圆角/描边/阴影（VM 端取无边框阴影的圆角子集）。
pub const VARIANT_FLOATING_VM: &str = "rounded-lg border border-sidebar-border";
/// sidebar_header / sidebar_footer 分区（两资产同串）。
pub const HEADER_BASE: &str = "flex flex-col gap-2 p-2";
pub const FOOTER_BASE: &str = "flex flex-col gap-2 p-2";
/// sidebar_content 分区（去 web-only `group-data-[collapsible=icon]:overflow-hidden`）。
pub const CONTENT_BASE: &str = "flex min-h-0 flex-1 flex-col gap-2 overflow-auto";
/// sidebar_separator。
pub const SEPARATOR: &str = "mx-2 w-auto bg-sidebar-border";
/// sidebar_inset 主内容区（去 min-h-svh 与 peer-data inset 变体串）。
pub const INSET_BASE: &str = "relative flex flex-1 flex-col bg-background";

// ── 分组 ────────────────────────────────────────────────────────────

/// sidebar_group 容器。
pub const GROUP_BASE: &str = "relative flex w-full min-w-0 flex-col p-2";
/// sidebar_group_label（去 duration/transition/outline/ring/`[&>svg]`/icon-collapse 串）。
pub const GROUP_LABEL: &str =
    "flex h-8 shrink-0 items-center rounded-md px-2 text-xs font-medium text-sidebar-foreground/70";
/// sidebar_group_content。
pub const GROUP_CONTENT: &str = "w-full text-sm";
/// sidebar_group_action（去 after:/focus-visible/`[&>svg]`/icon-collapse 串；
/// `aspect-square w-5` 的 VM 等价为 `w-5 h-5`——aspect 比例类 VM 不解析）。
pub const GROUP_ACTION: &str =
    "absolute right-3 top-3.5 flex w-5 h-5 items-center justify-center rounded-md p-0 text-sidebar-foreground";

// ── 菜单 ────────────────────────────────────────────────────────────

/// sidebar_menu（ul → VM column）。
pub const MENU_BASE: &str = "flex w-full min-w-0 flex-col gap-1";
/// sidebar_menu_item（li；`group/menu-item` 为 web 命名组标记，不入契约）。
pub const MENU_ITEM: &str = "relative";
/// sidebar_menu_button 基座（cva base 的 VM 可解析子集）。
pub const MENU_BUTTON_BASE: &str =
    "flex w-full items-center gap-2 overflow-hidden rounded-md p-2 text-left text-sm";
/// size 变体（cva variants.size 逐字子集）。
pub const MENU_BUTTON_SIZE_DEFAULT: &str = "h-8 text-sm";
pub const MENU_BUTTON_SIZE_SM: &str = "h-7 text-xs";
pub const MENU_BUTTON_SIZE_LG: &str = "h-12 text-sm";
/// variant=outline（cva outline 的 `shadow-[...var]` 不可解析，VM 等价适配为边框）。
pub const MENU_BUTTON_OUTLINE_VM: &str = "bg-background border border-sidebar-border";
/// hover 反馈（builder 仅在非 active 时挂，同 nav 的 either/or 约定）。
pub const MENU_BUTTON_HOVER: &str = "hover:bg-sidebar-accent hover:text-sidebar-accent-foreground";
/// active 选中态（cva `data-[active=true]:*` 三件的 VM 等价——builder 按 active
/// 状态直接挂裸类，同 nav ITEM_ACTIVE 模式）。
pub const MENU_BUTTON_ACTIVE: &str =
    "bg-sidebar-accent text-sidebar-accent-foreground font-medium";
/// disabled（cva `disabled:opacity-50` 的裸类等价；pointer-events 为 web-only）。
pub const MENU_BUTTON_DISABLED: &str = "opacity-50";
/// sidebar_menu_action（行内动作槽；去 peer/after/focus/`[&>svg]`/icon-collapse 串，
/// aspect-square → w-5 h-5 同 group_action）。
pub const MENU_ACTION: &str =
    "absolute right-1 flex w-5 h-5 items-center justify-center rounded-md p-0 text-sidebar-foreground";
/// sidebar_menu_badge（去 peer/icon-collapse/tabular-nums/select-none/pointer-events）。
pub const MENU_BADGE: &str =
    "flex h-5 min-w-5 items-center justify-center rounded-md px-1 text-xs font-medium text-sidebar-foreground";
/// sidebar_menu_sub 子菜单缩进层级（去 translate-x-px/icon-collapse 串）。
pub const MENU_SUB: &str =
    "mx-3.5 flex min-w-0 flex-col gap-1 border-l border-sidebar-border px-2.5 py-0.5";
/// sidebar_menu_sub_button（去 -translate-x-px/outline/ring/focus/aria/`[&>*]`/icon-collapse）。
pub const MENU_SUB_BUTTON_BASE: &str =
    "flex h-7 min-w-0 items-center gap-2 overflow-hidden rounded-md px-2 text-sidebar-foreground text-sm";
/// sub_button size=sm。
pub const MENU_SUB_BUTTON_SIZE_SM: &str = "text-xs";

/// VM 专适配 token（web 端由 scaffold 资产的 var/shadow 机制表达，资产里找不到
/// 对应逐字串——资产锚测试跳过这些常量）。
#[cfg(test)]
const VM_ADAPTED: &[&str] = &["w-64", "rounded-lg", "border", "border-sidebar-border", "bg-background", "h-5"];

/// hover:/active 等状态变体前缀在 parity 提取时剥壳（Style::parse 自行分流）。
const STATE_PREFIXES: &[&str] = &["hover:"];

/// 每个常量锚定的 scaffold 资产文件（相对 auto-man/assets/shadcn-ui/sidebar/）。
/// 用于逐 token 防漂移断言。
#[cfg(test)]
const ASSET_ANCHORS: &[(&str, &[&str])] = &[
    ("Sidebar.vue", &[SIDEBAR_BASE]),
    ("SidebarHeader.vue", &[HEADER_BASE]),
    ("SidebarFooter.vue", &[FOOTER_BASE]),
    ("SidebarContent.vue", &[CONTENT_BASE]),
    ("SidebarSeparator.vue", &[SEPARATOR]),
    ("SidebarInset.vue", &[INSET_BASE]),
    ("SidebarGroup.vue", &[GROUP_BASE]),
    ("SidebarGroupLabel.vue", &[GROUP_LABEL]),
    ("SidebarGroupContent.vue", &[GROUP_CONTENT]),
    ("SidebarGroupAction.vue", &[GROUP_ACTION]),
    ("SidebarMenu.vue", &[MENU_BASE]),
    ("SidebarMenuItem.vue", &[MENU_ITEM]),
    // cva 定义在 index.ts（button 基座/尺寸/hover/active/disabled 的源串）。
    (
        "index.ts",
        &[
            MENU_BUTTON_BASE,
            MENU_BUTTON_SIZE_DEFAULT,
            MENU_BUTTON_SIZE_SM,
            MENU_BUTTON_SIZE_LG,
            MENU_BUTTON_HOVER,
            MENU_BUTTON_ACTIVE,
            MENU_BUTTON_DISABLED,
        ],
    ),
    ("SidebarMenuAction.vue", &[MENU_ACTION]),
    ("SidebarMenuBadge.vue", &[MENU_BADGE]),
    ("SidebarMenuSub.vue", &[MENU_SUB]),
    ("SidebarMenuSubButton.vue", &[MENU_SUB_BUTTON_BASE, MENU_SUB_BUTTON_SIZE_SM]),
];

/// 全部契约常量（parity 提取源）。
const ALL: &[&str] = &[
    SIDEBAR_BASE, WIDTH_VM, VARIANT_FLOATING_VM, HEADER_BASE, FOOTER_BASE, CONTENT_BASE,
    SEPARATOR, INSET_BASE, GROUP_BASE, GROUP_LABEL, GROUP_CONTENT, GROUP_ACTION, MENU_BASE,
    MENU_ITEM, MENU_BUTTON_BASE, MENU_BUTTON_SIZE_DEFAULT, MENU_BUTTON_SIZE_SM,
    MENU_BUTTON_SIZE_LG, MENU_BUTTON_OUTLINE_VM, MENU_BUTTON_HOVER, MENU_BUTTON_ACTIVE,
    MENU_BUTTON_DISABLED, MENU_ACTION, MENU_BADGE, MENU_SUB, MENU_SUB_BUTTON_BASE,
    MENU_SUB_BUTTON_SIZE_SM,
];

/// 每个 whitespace 分隔、双端承载几何/颜色的 token（剥状态前缀；含 VM 适配 token）。
pub fn parity_tokens() -> Vec<&'static str> {
    let mut tokens = Vec::new();
    for s in ALL {
        for token in s.split_whitespace() {
            let mut bare = token;
            for p in STATE_PREFIXES {
                if let Some(stripped) = bare.strip_prefix(p) {
                    bare = stripped;
                }
            }
            if !tokens.contains(&bare) {
                tokens.push(bare);
            }
        }
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "ui")]
    use crate::ui::style::StyleClass;

    /// 全部 parity token 必须可被 VM style 解析器理解（未知 token 会被
    /// `Style::parse` 静默丢弃，故逐 token 断言）——sidebar 族的双端对拍门禁。
    #[cfg(feature = "ui")]
    #[test]
    fn sidebar_contract_tokens_parse_on_vm() {
        let tokens = parity_tokens();
        assert!(!tokens.is_empty());
        for token in &tokens {
            assert!(
                StyleClass::parse_single(token).is_ok(),
                "sidebar contract token `{token}` is not parseable by the VM style parser — remove it or curate a VM equivalent"
            );
        }
    }

    /// hover/active 串经 `Style::parse` 后落位正确：hover 进 hover_classes，
    /// active 三件的裸类进主类表（builder 按状态挂载）。
    #[cfg(feature = "ui")]
    #[test]
    fn sidebar_contract_active_hover_parse() {
        let style = crate::ui::style::Style::parse(MENU_BUTTON_HOVER).unwrap();
        assert!(
            !style.hover_classes.is_empty(),
            "hover utilities must land in hover_classes"
        );
        let style = crate::ui::style::Style::parse(MENU_BUTTON_ACTIVE).unwrap();
        assert!(
            style.hover_classes.is_empty(),
            "active uses bare classes (builder mounts by state), nothing may leak into hover_classes"
        );
    }

    /// 逐 token 资产锚：每个非 VM 适配 token 必须逐字出现在其常量锚定的
    /// scaffold 资产文件里（sidebar 资产保持 shadcn 原版逐字，故锚到 token 级）。
    #[test]
    fn sidebar_contract_matches_scaffold_assets() {
        let manifest = env!("CARGO_MANIFEST_DIR");
        let dir = std::path::Path::new(manifest).join("../auto-man/assets/shadcn-ui/sidebar");
        for (file, consts) in ASSET_ANCHORS {
            let path = dir.join(file);
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("scaffold asset {} missing ({e}) — Vue/VM contract drift", path.display()));
            for c in *consts {
                for token in c.split_whitespace() {
                    let mut bare = token;
                    for p in STATE_PREFIXES {
                        if let Some(stripped) = bare.strip_prefix(p) {
                            bare = stripped;
                        }
                    }
                    if VM_ADAPTED.contains(&bare) {
                        continue;
                    }
                    assert!(
                        content.contains(bare),
                        "scaffold asset sidebar/{file} no longer contains contract token `{bare}` (from `{c}`) — update the asset or sidebar_contract.rs"
                    );
                }
            }
        }
    }
}
