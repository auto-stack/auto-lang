//! Plan 527 对拍审计台 —— Tailwind v3.4 清单 → parse → applied 分类的常驻门。
//!
//! 数据流:
//!   tests/fixtures/tailwind-v34-utilities.txt (8859 类 × 15 docs-families)
//!     → `StyleClass::parse_single`          → mapped / missing
//!     → `IcedStyle::from_style(单类)` 机械差分 → iced applied / parsed-only
//!     → 未映射且命中白名单                   → unsupported(显式豁免,有理由)
//!
//! 常驻断言: 白名单外零 missing —— 任何清单类从在册退化为静默丢弃即红。
//! 家族级 applied 断言随 T3/T4/T5 逐步硬化的入。
//!
//! 覆盖率表: `STYLE_COVERAGE_REGEN=1 cargo test -p auto-lang --test style_parity
//! regen_coverage_table --features ui-iced --nocapture` 重新生成
//! docs/style-coverage.md(与断言同源分类,人工复核后 commit)。
//!
//! 诊断: `STYLE_AUDIT_DUMP=1 … audit_no_silent_drop --nocapture` 打印全量分类。

#![cfg(feature = "ui-iced")]

use std::collections::BTreeMap;

use auto_lang::ui::style::{IcedStyle, Style, StyleClass};

const MANIFEST: &str = include_str!("fixtures/tailwind-v34-utilities.txt");

// ===========================================================================
// Unsupported 白名单 —— 每行有家族级理由,与 KNOWN-DEBT-AND-RISKS.md 互链(T9)
// ===========================================================================
// 匹配规则(按序,首个命中): ("=", 精确类名) / ("prefix:", 前缀,要求以 '-' 结尾)。
// reason 带 [T3]/[T4]/[T5] 标记 = 临时基线条目,对应任务补全解析臂后必须移除;
// 无标记 = 永久不做/受限台账(原生无语义或宿主上限,登记 KNOWN-DEBT)。
// 语义口径:
//   - "grid 系" = iced 无 grid 布局,renderer 仅以等分 Row 模拟 grid-cols-N;
//   - "无背景图宿主" = VM 轨无 background-image 原语(纯色/渐变之外);
//   - "宿主接管" = 能力属于渲染宿主/OS 层,样式 IR 无对应字段。
const UNSUPPORTED: &[(&str, &str, &str)] = &[
    // ---- layout:永久 ----
    ("=", "box-border", "box-sizing 无对应:iced 布局语义内建(不影响宽高计算)"),
    ("=", "box-content", "box-sizing 无对应:iced 布局语义内建"),
    ("=", "container", "响应式容器断点类:VM 桌面窗口无断点容器语义"),
    ("=", "table", "表格显示模式:无表格布局宿主(tables 家族降级)"),
    ("prefix:", "table-", "表格显示模式/布局(table-caption…table-fixed):无表格布局宿主"),
    ("=", "inline-table", "表格显示模式:inline-table 无表格布局宿主"),
    ("=", "inline-grid", "grid 系:inline 网格无对应"),
    ("prefix:", "float-", "float/clear:原生无浮动语义(文档流模型不同)"),
    ("prefix:", "clear-", "float/clear:原生无浮动语义"),
    ("=", "flow-root", "display 细分:无 BFC 概念"),
    ("=", "contents", "display:contents:无布局穿透语义"),
    ("=", "list-item", "display:list-item:列表 marker 渲染属文本块职责(未接)"),
    ("=", "isolate", "isolation:无合成层隔离概念"),
    ("=", "isolation-auto", "isolation:无合成层隔离概念"),
    ("prefix:", "object-", "object-position:Image 无内容定位原语(object-fit 除外,见上)"),
    ("=", "visible", "visibility:visible = 默认态,无操作语义"),
    ("=", "invisible", "visibility:hidden(保位隐藏):无原生,renderer 未接 skip-paint"),
    ("prefix:", "overscroll-", "overscroll-behavior:桌面滚动宿主无滚动链语义"),
    ("prefix:", "columns-", "CSS multi-column 文本分栏:cosmic-text 上限,无分栏布局"),
    ("prefix:", "break-before-", "分页/分片控制:无 print/fragmentation 宿主"),
    ("prefix:", "break-after-", "分页/分片控制:无 print/fragmentation 宿主"),
    ("prefix:", "break-inside-", "分页/分片控制:无 print/fragmentation 宿主"),
    ("=", "decoration-slice", "box-decoration-break:无分段渲染概念"),
    ("=", "decoration-clone", "box-decoration-break:无分段渲染概念"),
    ("prefix:", "aspect-", "aspect-ratio:无原生(Plan 527 详细设计 T3 裁定直接入白名单)"),
    // ---- layout:永久(续) ----
    ("=", "static", "position static = 文档流默认位:双轨均为无操作语义"),
    ("prefix:", "inset-", "inset 百分比/auto/full 档:无容器查询语义(数值/px/负值档已接,T3)"),
    ("prefix:", "-inset-", "inset 负分数档:无容器查询语义(数值档已接,T3)"),
    ("prefix:", "top-", "top 百分比/auto/full 档:无容器查询语义(数值/px/负值档已接,T3)"),
    ("prefix:", "right-", "right 百分比/auto/full 档:无容器查询语义(数值/px/负值档已接,T3)"),
    ("prefix:", "bottom-", "bottom 百分比/auto/full 档:无容器查询语义(数值/px/负值档已接,T3)"),
    ("prefix:", "left-", "left 百分比/auto/full 档:无容器查询语义(数值/px/负值档已接,T3)"),
    ("prefix:", "-top-", "top 负分数档:无容器查询语义(数值档已接,T3)"),
    ("prefix:", "-right-", "right 负分数档:无容器查询语义(数值档已接,T3)"),
    ("prefix:", "-bottom-", "bottom 负分数档:无容器查询语义(数值档已接,T3)"),
    ("prefix:", "-left-", "left 负分数档:无容器查询语义(数值档已接,T3)"),
    ("prefix:", "w-", "w-min/max/fit:内容尺寸查询无宿主(分数/数值/vh 档已接,T3)"),
    ("prefix:", "h-", "h-min/max/fit:内容尺寸查询无宿主(分数/vh/数值档已接,T3)"),
    ("prefix:", "min-w-", "min-w full/min/max/fit:百分比与内容尺寸无宿主(数值/px 档已接;T3 收紧此前未知命名值误落 0.0)"),
    ("prefix:", "min-h-", "min-h full/min/max/fit:同 min-w(svh/lvh/dvh ≈ screen 已接,T3)"),
    ("prefix:", "max-w-", "max-w min/max/fit/prose:内容尺寸/ch 单位无宿主(none/full/分数/数值档已接,T3)"),
    ("prefix:", "max-h-", "max-h min/max/fit/screen:内容尺寸/视口高查询无宿主(none/分数/数值档已接,T3)"),
    // ---- flexbox:永久 ----
    ("prefix:", "justify-items-", "grid 系:grid 布局仅 grid-cols-N 模拟"),
    ("prefix:", "justify-self-", "grid 系:无 per-item 网格对齐"),
    ("=", "order-first", "order 降级:VM 按源码序渲染(Plan 412 §5),first/last/none 无增量语义"),
    ("=", "order-last", "order 降级:VM 按源码序渲染"),
    ("=", "order-none", "order 降级:VM 按源码序渲染"),
    ("=", "col-auto", "grid 系:col/row 自动放置无对应"),
    ("=", "row-auto", "grid 系:col/row 自动放置无对应"),
    ("=", "col-start-auto", "grid 系:自动放置无对应"),
    ("=", "col-end-auto", "grid 系:自动放置无对应"),
    ("=", "row-start-auto", "grid 系:自动放置无对应"),
    ("=", "row-end-auto", "grid 系:自动放置无对应"),
    ("=", "grid-cols-none", "grid 系:模拟仅数值列(grid-cols-N)"),
    ("=", "grid-rows-none", "grid 系:模拟仅数值行(grid-rows-N)"),
    ("prefix:", "auto-cols-", "grid 系:隐式轨道尺寸无对应"),
    ("prefix:", "auto-rows-", "grid 系:隐式轨道尺寸无对应"),
    ("prefix:", "grid-flow-", "grid 系:自动流向无对应"),
    // ---- spacing:永久 ----
    ("=", "space-x-reverse", "per-child margin 反转:VM gap 语义无方向反转"),
    ("=", "space-y-reverse", "per-child margin 反转:VM gap 语义无方向反转"),
    ("prefix:", "-space-x-", "负 spacing:iced 间距不可负"),
    ("prefix:", "-space-y-", "负 spacing:iced 间距不可负"),
    // ---- typography:永久 ----
    ("=", "list-disc", "列表 marker 渲染:文本块职责未接(仅 list-none 已接)"),
    ("=", "list-decimal", "列表 marker 渲染:文本块职责未接"),
    ("=", "list-inside", "列表 marker 位置:无 marker 渲染"),
    ("=", "list-outside", "列表 marker 位置:无 marker 渲染"),
    ("prefix:", "indent-", "text-indent:cosmic-text 无段落首行缩进"),
    ("prefix:", "-indent-", "text-indent:cosmic-text 无段落首行缩进"),
    ("prefix:", "align-", "vertical-align:无行内盒排版"),
    ("=", "whitespace-normal", "whitespace 细分:cosmic-text 换行策略仅 word/none(nowrap 已接)"),
    ("=", "whitespace-pre", "whitespace 细分:pre 保留空白无原生"),
    ("=", "whitespace-pre-line", "whitespace 细分:无原生"),
    ("=", "whitespace-pre-wrap", "whitespace 细分:无原生"),
    ("=", "whitespace-break-spaces", "whitespace 细分:无原生"),
    ("=", "break-normal", "word-break 细分:默认词断行即 normal,无增量语义"),
    ("=", "break-all", "word-break:cosmic-text 无字符断行策略"),
    ("prefix:", "wrap-", "overflow-wrap:cosmic-text 断行策略上限"),
    ("=", "content-none", "伪元素内容:无伪元素宿主"),
    ("prefix:", "hyphens-", "连字符断词:cosmic-text 上限"),
    ("=", "uppercase", "text-transform:渲染层无字形变换(字符串预处理超出样式 IR 职责)"),
    ("=", "lowercase", "text-transform:渲染层无字形变换"),
    ("=", "capitalize", "text-transform:渲染层无字形变换"),
    ("=", "normal-case", "text-transform:默认态无操作语义"),
    ("=", "overline", "上划线:iced 文本装饰仅 underline/strikethrough"),
    ("prefix:", "decoration-", "装饰线型/粗细(wavy/dotted/粗细档):iced 装饰线无样式参数"),
    ("prefix:", "underline-offset-", "下划线偏移:iced 装饰线无偏移参数"),
    ("=", "subpixel-antialiased", "字体平滑:宿主接管(antialiased 同理,见下)"),
    ("=", "antialiased", "字体平滑:宿主接管,无样式字段"),
    ("=", "normal-nums", "font-variant-numeric:cosmic-text 无数字变体"),
    ("=", "ordinal", "font-variant-numeric:cosmic-text 上限"),
    ("=", "slashed-zero", "font-variant-numeric:cosmic-text 上限"),
    ("=", "lining-nums", "font-variant-numeric:cosmic-text 上限"),
    ("=", "oldstyle-nums", "font-variant-numeric:cosmic-text 上限"),
    ("=", "proportional-nums", "font-variant-numeric:cosmic-text 上限"),
    ("=", "tabular-nums", "font-variant-numeric:cosmic-text 上限"),
    ("=", "diagonal-fractions", "font-variant-numeric:cosmic-text 上限"),
    ("=", "stacked-fractions", "font-variant-numeric:cosmic-text 上限"),
    ("=", "text-inherit", "颜色继承(currentColor/inherit):样式 IR 无继承链"),
    ("=", "text-current", "颜色继承(currentColor/inherit):样式 IR 无继承链"),
    // ---- typography:永久(cosmic-text 上限;T5 核销 tracking/leading/
    // line-clamp/text-start/end/ellipsis/clip 档) ----
    ("=", "text-justify", "两端对齐:cosmic-text 无 justify 排版"),
    // ---- backgrounds:永久(无背景图宿主) ----
    ("=", "bg-fixed", "背景图附件/裁剪/原点/位置/重复/尺寸:VM 轨无 background-image 原语"),
    ("=", "bg-local", "背景图附件:无 background-image 原语"),
    ("=", "bg-scroll", "背景图附件:无 background-image 原语"),
    ("=", "bg-clip-border", "背景裁剪:无 background-image 原语"),
    ("=", "bg-clip-padding", "背景裁剪:无 background-image 原语"),
    ("=", "bg-clip-content", "背景裁剪:无 background-image 原语"),
    ("=", "bg-clip-text", "背景裁剪:无 background-image 原语"),
    ("=", "bg-none", "背景图置空:无 background-image 原语"),
    ("=", "bg-origin-border", "背景原点:无 background-image 原语"),
    ("=", "bg-origin-padding", "背景原点:无 background-image 原语"),
    ("=", "bg-origin-content", "背景原点:无 background-image 原语"),
    ("=", "bg-bottom", "背景位置:无 background-image 原语"),
    ("=", "bg-center", "背景位置:无 background-image 原语"),
    ("=", "bg-left", "背景位置:无 background-image 原语"),
    ("=", "bg-left-bottom", "背景位置:无 background-image 原语"),
    ("=", "bg-left-top", "背景位置:无 background-image 原语"),
    ("=", "bg-right", "背景位置:无 background-image 原语"),
    ("=", "bg-right-bottom", "背景位置:无 background-image 原语"),
    ("=", "bg-right-top", "背景位置:无 background-image 原语"),
    ("=", "bg-top", "背景位置:无 background-image 原语"),
    ("=", "bg-repeat", "背景重复:无 background-image 原语"),
    ("=", "bg-no-repeat", "背景重复:无 background-image 原语"),
    ("=", "bg-repeat-x", "背景重复:无 background-image 原语"),
    ("=", "bg-repeat-y", "背景重复:无 background-image 原语"),
    ("=", "bg-repeat-round", "背景重复:无 background-image 原语"),
    ("=", "bg-repeat-space", "背景重复:无 background-image 原语"),
    ("=", "bg-auto", "背景尺寸:无 background-image 原语"),
    ("=", "bg-cover", "背景尺寸:无 background-image 原语"),
    ("=", "bg-contain", "背景尺寸:无 background-image 原语"),
    ("=", "bg-inherit", "颜色继承:样式 IR 无继承链"),
    ("=", "bg-current", "颜色继承:样式 IR 无继承链"),
    // ---- backgrounds:颜色继承(Plan 527 T4 核销 from/via/to 色板+pct 档) ----
    ("=", "from-inherit", "颜色继承:样式 IR 无继承链"),
    ("=", "from-current", "颜色继承:样式 IR 无继承链"),
    ("=", "via-inherit", "颜色继承:样式 IR 无继承链"),
    ("=", "via-current", "颜色继承:样式 IR 无继承链"),
    ("=", "to-inherit", "颜色继承:样式 IR 无继承链"),
    ("=", "to-current", "颜色继承:样式 IR 无继承链"),
    // ---- borders:永久 ----
    ("prefix:", "divide-", "子项分隔线:渲染层未接(divide width/color/style)"),
    ("=", "border-solid", "边框样式:iced border 无 style 参数(solid 默认)"),
    ("=", "border-dashed", "边框样式:iced border 无 style 参数"),
    ("=", "border-dotted", "边框样式:iced border 无 style 参数"),
    ("=", "border-double", "边框样式:iced border 无 style 参数"),
    ("=", "border-hidden", "边框样式:iced border 无 style 参数(hidden ≈ none 已接)"),
    ("=", "border-inherit", "颜色继承:样式 IR 无继承链"),
    ("=", "border-current", "颜色继承:样式 IR 无继承链"),
    ("=", "ring-inherit", "颜色继承:样式 IR 无继承链"),
    ("=", "ring-current", "颜色继承:样式 IR 无继承链"),
    ("=", "shadow-inherit", "颜色继承:样式 IR 无继承链"),
    ("=", "shadow-current", "颜色继承:样式 IR 无继承链"),
    ("prefix:", "ring-offset-", "ring 外扩 offset:渲染层未接"),
    ("=", "outline", "outline 组合样式:focus 环由宿主接管(outline-none 已接)"),
    ("prefix:", "outline-", "outline 颜色/样式:focus 环宿主接管"),
    // ---- borders:单侧宽度/颜色分档(PLAN-050 C2 模拟边界;全侧 border/border-{color} 已接) ----
    ("prefix:", "border-x-", "单侧宽度/颜色分档:renderer 1px 填充条模拟(PLAN-050 C2)无宽/色分档能力"),
    ("prefix:", "border-y-", "单侧宽度/颜色分档:renderer 1px 填充条模拟(PLAN-050 C2)无宽/色分档能力"),
    ("prefix:", "border-t-", "单侧宽度/颜色分档:renderer 1px 填充条模拟(PLAN-050 C2)无宽/色分档能力"),
    ("prefix:", "border-r-", "单侧宽度/颜色分档:renderer 1px 填充条模拟(PLAN-050 C2)无宽/色分档能力"),
    ("prefix:", "border-b-", "单侧宽度/颜色分档:renderer 1px 填充条模拟(PLAN-050 C2)无宽/色分档能力"),
    ("prefix:", "border-l-", "单侧宽度/颜色分档:renderer 1px 填充条模拟(PLAN-050 C2)无宽/色分档能力"),
    // ---- effects:永久 ----
    ("=", "shadow-inner", "内阴影:iced shadow 无 inner"),
    ("prefix:", "mix-blend-", "混合模式:无混合原语"),
    ("prefix:", "bg-blend-", "混合模式:无混合原语"),
    // ---- filters:永久(声明冻结,Plan 518 G8 先例) ----
    ("=", "filter", "滤镜系:无滤镜渲染管道(声明冻结,Plan 518 G8 先例)"),
    ("=", "filter-none", "滤镜系:无滤镜渲染管道"),
    ("prefix:", "blur-", "滤镜系:无滤镜渲染管道"),
    ("=", "blur", "滤镜系:无滤镜渲染管道"),
    ("prefix:", "saturate-", "滤镜系:无滤镜渲染管道"),
    ("prefix:", "drop-shadow-", "滤镜系:无滤镜渲染管道"),
    ("=", "drop-shadow", "滤镜系:无滤镜渲染管道"),
    ("prefix:", "brightness-", "滤镜系:无滤镜渲染管道"),
    ("prefix:", "contrast-", "滤镜系:无滤镜渲染管道"),
    ("prefix:", "grayscale-", "滤镜系:无滤镜渲染管道"),
    ("prefix:", "invert-", "滤镜系:无滤镜渲染管道"),
    ("prefix:", "sepia-", "滤镜系:无滤镜渲染管道"),
    ("prefix:", "hue-rotate-", "滤镜系:无滤镜渲染管道"),
    ("prefix:", "-brightness-", "滤镜系:无滤镜渲染管道(负值)"),
    ("prefix:", "-contrast-", "滤镜系:无滤镜渲染管道(负值)"),
    ("prefix:", "-grayscale-", "滤镜系:无滤镜渲染管道(负值)"),
    ("prefix:", "-invert-", "滤镜系:无滤镜渲染管道(负值)"),
    ("prefix:", "-sepia-", "滤镜系:无滤镜渲染管道(负值)"),
    ("prefix:", "-hue-rotate-", "滤镜系:无滤镜渲染管道(负值)"),
    ("=", "backdrop-filter", "backdrop 滤镜系:G8 词汇冻结(blur/saturate 之外不收)"),
    ("=", "backdrop-filter-none", "backdrop 滤镜系:G8 词汇冻结"),
    ("=", "backdrop-blur-none", "backdrop 滤镜系:G8 词汇冻结(刻度外档)"),
    ("prefix:", "backdrop-opacity-", "backdrop 滤镜系:G8 词汇冻结"),
    ("prefix:", "backdrop-brightness-", "backdrop 滤镜系:G8 词汇冻结"),
    ("prefix:", "backdrop-contrast-", "backdrop 滤镜系:G8 词汇冻结"),
    ("prefix:", "backdrop-grayscale-", "backdrop 滤镜系:G8 词汇冻结"),
    ("prefix:", "backdrop-invert-", "backdrop 滤镜系:G8 词汇冻结"),
    ("prefix:", "backdrop-sepia-", "backdrop 滤镜系:G8 词汇冻结"),
    ("prefix:", "backdrop-hue-rotate-", "backdrop 滤镜系:G8 词汇冻结"),
    ("prefix:", "-backdrop-hue-rotate-", "backdrop 滤镜系:G8 词汇冻结(负值)"),
    ("prefix:", "backdrop-saturate-", "backdrop 滤镜系:G8 词汇冻结(0 档不收;50..200 已接)"),
    // ---- tables:永久 ----
    ("=", "border-collapse", "表格边框模型:无表格布局宿主"),
    ("=", "border-separate", "表格边框模型:无表格布局宿主"),
    ("prefix:", "border-spacing-", "表格间距:无表格布局宿主"),
    ("=", "caption-top", "表标题位置:无表格布局宿主"),
    ("=", "caption-bottom", "表标题位置:无表格布局宿主"),
    // ---- transitions:永久 ----
    ("=", "transition", "transition 属性集:无动画宿主(duration/transition-colors 已存 IR,渲染 no-op)"),
    ("=", "transition-none", "transition 属性集:无动画宿主"),
    ("=", "transition-all", "transition 属性集:无动画宿主"),
    ("=", "transition-opacity", "transition 属性集:无动画宿主"),
    ("=", "transition-shadow", "transition 属性集:无动画宿主"),
    ("=", "transition-transform", "transition 属性集:无动画宿主"),
    ("prefix:", "delay-", "过渡延迟:无动画宿主"),
    ("prefix:", "ease-", "缓动函数:无动画宿主"),
    ("prefix:", "animate-", "关键帧动画:无动画宿主"),
    // ---- transforms:永久 ----
    ("=", "transform", "变换管道:仅 rotate 已接(renderer);scale/translate/skew/origin 无变换矩阵管道"),
    ("=", "transform-gpu", "变换管道:无变换矩阵管道"),
    ("=", "transform-none", "变换管道:无变换矩阵管道"),
    ("prefix:", "scale-", "变换管道:无变换矩阵管道(rotate 例外已接)"),
    ("prefix:", "translate-", "变换管道:无变换矩阵管道"),
    ("prefix:", "-translate-", "变换管道:无变换矩阵管道(负值)"),
    ("prefix:", "skew-", "变换管道:无变换矩阵管道"),
    ("prefix:", "-skew-", "变换管道:无变换矩阵管道(负值)"),
    ("prefix:", "origin-", "变换原点:无变换矩阵管道"),
    ("prefix:", "-rotate-", "变换管道:负旋转未接(rotate-N 正值已接)"),
    // ---- interactivity:永久 ----
    ("prefix:", "appearance-", "控件外观重置:宿主接管"),
    ("prefix:", "cursor-", "光标交互细分:宿主接管(cursor-pointer 已接)"),
    ("prefix:", "caret-", "插入符颜色:文本编辑器职责未接"),
    ("prefix:", "pointer-events-", "命中测试:宿主接管"),
    ("prefix:", "resize-", "尺寸调整柄:窗口 resize 宿主职责"),
    ("=", "resize", "尺寸调整柄:窗口 resize 宿主职责"),
    ("prefix:", "scroll-", "scroll 行为/margin/padding:无锚点滚动宿主"),
    ("prefix:", "touch-", "触摸手势:桌面宿主无对应"),
    ("prefix:", "select-", "文本选择性:cosmic-text selection 由编辑器路径接管"),
    ("prefix:", "will-change-", "渲染提示:无合成层管道"),
    ("=", "accent-inherit", "颜色继承:样式 IR 无继承链"),
    ("=", "accent-current", "颜色继承:样式 IR 无继承链"),
    // ---- svg:永久 ----
    ("prefix:", "fill-", "SVG 填充:VM 图标走字体/着色管道,非 SVG 原语"),
    ("prefix:", "stroke-", "SVG 描边:VM 图标走字体/着色管道"),
    // ---- accessibility:永久 ----
    ("=", "sr-only", "屏幕阅读器语义:无辅助功能宿主"),
    ("=", "not-sr-only", "屏幕阅读器语义:无辅助功能宿主"),
    ("prefix:", "forced-color-", "强制色彩模式:OS 层接管"),
];

/// 三家族内允许 parsed-only 的类(「解析了但 iced 适配器 no-op」的显式豁免
/// 台账,Plan 527 目标 2 闸门的第二本账)。匹配规则同 UNSUPPORTED。
/// 家族断言按组激活:布局组 T3、视觉组 T4、文本组 T5。
const PARSED_ONLY_ALLOWED: &[(&str, &str, &str)] = &[
    // 布局组
    ("=", "flex-initial", "CSS 默认(flex:0 1 auto):iced 默认即收缩不生长,无操作语义"),
    ("=", "flex-none", "flex:none:iced 子元素默认不伸缩,无操作语义"),
    ("=", "grow-0", "flex-grow:0 = CSS 默认:iced 默认不生长"),
    ("=", "shrink", "flex-shrink:1 = CSS 默认:iced 默认收缩语义近似"),
    ("=", "shrink-0", "flex-shrink:0:iced 无 per-child 收缩控制,默认行为近似"),
    ("=", "flex-nowrap", "flex-wrap:nowrap = 默认:VM 单行降级一致"),
    // 视觉组(T4 激活断言)
    ("prefix:", "backdrop-", "G8 声明冻结:渲染端视觉 no-op(渲染分期,KNOWN-DEBT planned-debt)"),
    // 文本组(T5 激活断言)
    ("=", "list-none", "列表 marker 移除:文本块无 marker 渲染,无操作语义"),
    ("=", "antialiased", "字体平滑宿主接管,无操作语义"),
    ("=", "break-words", "cosmic-text 默认词断行,无操作语义"),
    ("=", "no-underline", "装饰重置语义:单类差分不可见(仅对先序 underline 有意义)"),
];

fn pattern_hits(kind: &str, pat: &str, class: &str) -> bool {
    if kind == "=" {
        class == pat
    } else {
        class.starts_with(pat) && pat.ends_with('-')
    }
}

fn parsed_only_allowed(class: &str) -> Option<&'static str> {
    PARSED_ONLY_ALLOWED
        .iter()
        .find(|(kind, pat, _)| pattern_hits(kind, pat, class))
        .map(|(_, _, reason)| *reason)
}

fn unsupported_reason(class: &str) -> Option<&'static str> {
    for (kind, pat, reason) in UNSUPPORTED {
        if pattern_hits(kind, pat, class) {
            return Some(reason);
        }
    }
    None
}

// ===========================================================================
// 清单解析与分类
// ===========================================================================

struct Entry {
    family: &'static str,
    class: String,
}

fn manifest_entries() -> Vec<Entry> {
    let mut family = "";
    let mut out = Vec::new();
    for line in MANIFEST.lines() {
        if let Some(rest) = line.strip_prefix("# family ") {
            family = rest.trim();
        } else if !line.is_empty() && !line.starts_with('#') {
            out.push(Entry { family, class: line.to_string() });
        }
    }
    out
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Status {
    /// mapped + iced 适配器真消费(单类 from_style 差分非默认)
    Applied,
    /// mapped 但 iced 适配器 no-op(渲染层不消费,coverage 表可见,非静默)
    ParsedOnly,
    /// 未映射 + 白名单显式豁免(不做/受限台账)
    Unsupported,
    /// 未映射且不在白名单 —— 审计红
    Missing,
}

fn classify(class: &str) -> Status {
    match StyleClass::parse_single(class) {
        Ok(c) => {
            let style = Style { classes: vec![c], hover_classes: Vec::new(), variant_classes: Vec::new() };
            if IcedStyle::from_style(&style) == IcedStyle::default() {
                Status::ParsedOnly
            } else {
                Status::Applied
            }
        }
        Err(_) => match unsupported_reason(class) {
            Some(_) => Status::Unsupported,
            None => Status::Missing,
        },
    }
}

// ===========================================================================
// 常驻断言
// ===========================================================================

/// 白名单外零静默丢弃 —— Plan 527 目标 1 的机器闸门。
#[test]
fn audit_no_silent_drop() {
    let entries = manifest_entries();
    assert!(!entries.is_empty(), "manifest fixture 解析为空 —— 格式回归");

    let mut missing_by_family: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    let mut status_count: BTreeMap<Status, usize> = BTreeMap::new();
    for e in &entries {
        let st = classify(&e.class);
        *status_count.entry(st).or_default() += 1;
        if st == Status::Missing {
            missing_by_family.entry(e.family).or_default().push(e.class.clone());
        }
    }

    if std::env::var("STYLE_AUDIT_DUMP").is_ok() {
        eprintln!("=== 全量分类统计 ===");
        for (st, n) in &status_count {
            eprintln!("{:?}: {}", st, n);
        }
        eprintln!("=== unsupported 白名单外的 missing(按家族) ===");
        for (fam, list) in &missing_by_family {
            eprintln!("[{}] {} 个: {}", fam, list.len(), list.join(" "));
        }
    }

    let total_missing: usize = missing_by_family.values().map(Vec::len).sum();
    if total_missing > 0 {
        let mut msg = format!(
            "白名单外清单类静默丢弃 {} 个 —— 补解析臂或登记 UNSUPPORTED 白名单:\n",
            total_missing
        );
        for (fam, list) in &missing_by_family {
            let preview: Vec<&str> = list.iter().take(40).map(String::as_str).collect();
            msg.push_str(&format!(
                "[{}] {} 个(前 40): {}\n",
                fam,
                list.len(),
                preview.join(" ")
            ));
        }
        panic!("{}", msg);
    }
}

// ===========================================================================
// 家族 applied 断言(Plan 527 目标 2:三家族非白名单类必须 iced applied)
// ===========================================================================

/// 家族组闸门:组内清单类必须 Applied 或 UNSUPPORTED(显式豁免);
/// ParsedOnly 仅限 PARSED_ONLY_ALLOWED 在册;Missing 一律违例。
/// 按组激活:布局组 T3、视觉组 T4、文本组 T5。
fn assert_family_group_applied(families: &[&str], group_name: &str) {
    let mut offenders: Vec<String> = Vec::new();
    for e in manifest_entries() {
        if !families.contains(&e.family) {
            continue;
        }
        let st = classify(&e.class);
        let bad = match st {
            Status::Applied | Status::Unsupported => false,
            Status::ParsedOnly => parsed_only_allowed(&e.class).is_none(),
            Status::Missing => true,
        };
        if bad {
            offenders.push(format!("[{}] {} ({:?})", e.family, e.class, st));
        }
    }
    assert!(
        offenders.is_empty(),
        "{} 家族组 applied 断言违例 {} 个(应为 applied/unsupported,或登记 PARSED_ONLY_ALLOWED):\n{}",
        group_name,
        offenders.len(),
        offenders.join("\n")
    );
}

/// T3 激活:布局组(layout/flexbox/spacing/sizing)。
#[test]
fn audit_layout_family_applied() {
    assert_family_group_applied(&["layout", "flexbox", "spacing", "sizing"], "布局");
}

/// T4 激活:视觉组(backgrounds/borders/effects/filters)。
#[test]
fn audit_visual_family_applied() {
    assert_family_group_applied(
        &["backgrounds", "borders", "effects", "filters"],
        "视觉",
    );
}

/// T5 激活:文本组(typography)。
#[test]
fn audit_text_family_applied() {
    assert_family_group_applied(&["typography"], "文本");
}

// ===========================================================================
// 覆盖率表(docs/style-coverage.md,与断言同源生成)
// ===========================================================================

/// docs-family → 计划三家族聚合(coverage 表行序)。
const FAMILY_GROUPS: &[(&str, &[&str])] = &[
    ("布局 (layout/flexbox/spacing/sizing)", &["layout", "flexbox", "spacing", "sizing"]),
    ("视觉 (backgrounds/borders/effects/filters)", &["backgrounds", "borders", "effects", "filters"]),
    ("文本 (typography)", &["typography"]),
    (
        "其他 (tables/transitions/transforms/interactivity/svg/accessibility)",
        &["tables", "transitions", "transforms", "interactivity", "svg", "accessibility"],
    ),
];

#[test]
fn regen_coverage_table() {
    if std::env::var("STYLE_COVERAGE_REGEN").is_err() {
        return; // 常规跑跳过;表按需再生
    }
    let mut fam_stats: BTreeMap<&str, [usize; 4]> = BTreeMap::new(); // applied, parsed-only, unsupported, missing
    let mut total = [0usize; 4];
    for e in manifest_entries() {
        let idx = match classify(&e.class) {
            Status::Applied => 0,
            Status::ParsedOnly => 1,
            Status::Unsupported => 2,
            Status::Missing => 3,
        };
        fam_stats.entry(e.family).or_default()[idx] += 1;
        total[idx] += 1;
    }

    let mut md = String::new();
    md.push_str("# Style Coverage —— Tailwind v3.4 清单 × VM 三后端覆盖矩阵\n\n");
    md.push_str("> GENERATED by `tests/style_parity.rs`(`STYLE_COVERAGE_REGEN=1` 再生,");
    md.push_str("与 `audit_no_silent_drop` 断言同源分类)—— **不要手改**。\n>\n");
    md.push_str("> 状态定义: **applied** = 解析为 StyleClass 且 iced 适配器单类差分非默认");
    md.push_str("(字段被消费);**parsed-only** = 解析成功但 iced 适配器 no-op");
    md.push_str("(渲染层不消费——coverage 表可见,非静默丢弃);**unsupported** =");
    md.push_str("显式白名单豁免(不做/受限台账,理由见 `UNSUPPORTED` 常量表,");
    md.push_str("与 KNOWN-DEBT-AND-RISKS.md 互链);**missing** = 清单在册但解析失败");
    md.push_str("(常驻断言下恒为 0)。\n>\n> headless 金标 = mapped(解析成功即无渲染歧义);");
    md.push_str("gpui 尽力对齐 iced 金标,差异登记台账(feature 不入默认门禁)。\n\n");
    let grand_total: usize = total.iter().sum();
    md.push_str(&format!(
        "总计: {} 类 — applied {} / parsed-only {} / unsupported {} / missing {}\n\n",
        grand_total, total[0], total[1], total[2], total[3]
    ));
    md.push_str("| 家族 | applied | parsed-only | unsupported | missing |\n|---|---|---|---|---|\n");
    for (group, fams) in FAMILY_GROUPS {
        md.push_str(&format!("| **{}** | | | | |\n", group));
        for fam in *fams {
            let s = fam_stats.get(*fam).copied().unwrap_or([0, 0, 0, 0]);
            md.push_str(&format!(
                "| └ {} | {} | {} | {} | {} |\n",
                fam, s[0], s[1], s[2], s[3]
            ));
        }
    }
    md.push_str("\n## Unsupported 白名单(与断言同源)\n\n");
    md.push_str("| 模式 | 理由 |\n|---|---|\n");
    for (kind, pat, reason) in UNSUPPORTED {
        md.push_str(&format!("| `{}{}` | {} |\n", if *kind == "=" { "" } else { "prefix " }, pat, reason));
    }

    md.push_str("
## Parsed-only 豁免台账(三家族内允许 no-op 的类)

");
    md.push_str("| 模式 | 理由 |
|---|---|
");
    for (kind, pat, reason) in PARSED_ONLY_ALLOWED {
        md.push_str(&format!("| `{}{}` | {} |
", if *kind == "=" { "" } else { "prefix " }, pat, reason));
    }

    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/style-coverage.md");
    std::fs::write(path, md).expect("写 docs/style-coverage.md");
    eprintln!("coverage table -> {}", path);
}
