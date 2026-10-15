//! PLAN-786: AutoMan 展示字段（name/title/title_zh）统一只读入口。
//!
//! 一次 [`auto_project_model::parse_document`] 解析同时服务三个字段；每个字段
//! 独立做准入判定（模型 DTO 直读 / 具名 legacy 兼容 / 无值），判定与结果对
//! 单测/收据可观察，不进入用户 stdout。locale 选择（title_zh→title 链）与
//! name/目录兜底留在调用者（vue.rs）。
//!
//! 生产不做双臂逐次判等选路：Model 路径只运行模型解析 + 对唯一声明行的
//! 单行形态谓词（复算一次冻结旧行归一，非全文件对拍）；其余输入全部走
//! 本模块内的兼容臂——vue.rs 原行扫描（parse_pac_scalar/parse_pac_name）的
//! 冻结搬迁，仅此一处。与旧行为的对拍 oracle 只存在于测试模块。
//!
//! 准入矩阵与每类输入的冻结旧输出见
//! `docs/plans/attachments/786/t01-frozen-oracle.md`。只读契约：不写文件、
//! 不读 env、不执行 VM（本模块不依赖 auto-lang）。

use auto_project_model::{parse_document, ProjectDescriptor, SourceInfo};

/// 每字段读取路径分类（单测/收据可观察）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReadPath {
    /// 模型 DTO 直读：解析 Ok、gate 行集合==provenance 行集合、无重复键，
    /// 且（有值时）声明行的旧行归一与模型值逐字相等。
    Model(ModelState),
    /// legacy 兼容臂：旧行扫描唯一冻结副本，具名原因。
    Compat(CompatReason),
}

/// Model 路径的细分类（同属旧行为等价，分类仅为审计粒度）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ModelState {
    /// 文件中无该 key 的门槛行（真实缺席；赋值形态/前缀相似 key 归此）。
    Absent,
    /// title 族声明归一为空——旧行为跳空续扫无果 → None（name 不会处此态）。
    DeclaredEmpty,
    /// 声明的静态值，与旧行归一逐字相等。
    Value,
}

/// 兼容臂触发原因（具名、有限、各自绑定冻结矩阵行）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CompatReason {
    /// 静态解析 Err（坏输入）：旧行扫描照常返回首匹配或 None。
    ParseError,
    /// gate 行集合 ≠ provenance 行集合：存在模型看不见的门槛行（块注释/
    /// 跨行字符串/参数表/结构化值内部/嵌套节点体），或模型可见而行扫描
    /// 不可见（冒号前空格等门槛不过形态）。
    HiddenOrPhantomLine,
    /// 重复声明：旧行扫描取首匹配，模型 last-wins。
    DuplicateKey,
    /// 声明值非干净静态字面量（单引号/bare/插值/f-string/调用/on-off），
    /// 或 name 的 None+单行形态（旧值非空必返回）。
    UnsupportedValue,
    /// 值虽静态但行形态与旧行归一不等：转义、引号内边缘空白、行内注释、
    /// 非 Str 的 astr 差异（如 nil）。
    LineFormMismatch,
    /// 开启括号字符总数（`[`+`{`）超过 [`MODEL_PARSE_MAX_BRACKET_DEPTH`]——
    /// 超出模型递归解析的安全包络（R1 F786-R1-01：512 层嵌套数组使
    /// parse_document 进程级栈溢出 abort，catch_unwind 不可捕获；上界
    /// 论证见 [`bracket_open_count`]）。超限输入不做模型解析，三字段整体
    /// 走兼容臂，旧行为逐值保留。
    DeepNesting,
}

/// 一次读取的全部结果：三字段值 + 各自路径分类。
pub(crate) struct DisplayRead {
    pub(crate) name: Option<String>,
    pub(crate) title: Option<String>,
    pub(crate) title_zh: Option<String>,
    /// 路径分类是审计观测面（单测/收据读取，见 T01 冻结表 §3）；
    /// 生产只消费三字段值，非测试构建不读取。
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) name_path: ReadPath,
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) title_path: ReadPath,
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) title_zh_path: ReadPath,
}

/// 模型递归解析的括号包络。实测（R1 F786-R1-01）：512 层嵌套使
/// `parse_document` 在 Windows 默认主线程栈上 STATUS_STACK_OVERFLOW abort
/// （catch_unwind 不可捕获）；128/32 层通过。真实受管 pac.at 括号开启数
/// 远低于此。64 = 已验证通过点之下的成对余量；超限输入整体走兼容臂
/// （旧行为逐值保留），不是语法拒绝。
const MODEL_PARSE_MAX_BRACKET_DEPTH: usize = 64;

/// 统计**开启括号字符总数**（`[` + `{`），不做任何词法判定（不识别
/// 字符串/注释/转义）。R4 终形上界论证：
///
/// 解析器的结构下钻（scan_value_positions 的 scan_object/scan_array、
/// parse_node 子节点、AtomParser 结构化字面量）按**嵌套层**递归——元素
/// 迭代、下钻才递归，且每下钻一层必消费一个被解析为 opener 的 `[`/`{`
/// 字符。故 **真实递归深度 ≤ 文本中 `[`+`{` 字符总数**：
/// - 闭括号（无论是否在字符串内）不参与计数，不可能削减上界（R4
///   F786-R4-02：字符串 `]` 逐层抵消开括号，使 max-depth 型上界失效）；
/// - 注释/字符串上下文只可能**虚增**计数（其中的 `[`/`{` 被算入，
///   解析器却不会下钻），过计数仅多走兼容臂，值行为不变（R4
///   F786-R4-01：字符串内 `/*`、`//`、`#` 使任何"识别注释"的筛子
///   藏住后续真实括号）；
/// - 零词法判定 ⇒ 不存在"筛子与某条解析路径的字符串/注释语义不一致"
///   这一反例族（R1 遇 LF 断 / R2 跨 LF / R3 行界重入同根）。
fn bracket_open_count(content: &str) -> usize {
    content.chars().filter(|c| matches!(c, '[' | '{')).count()
}

/// 读取 pac.at 文本的展示三字段（只读，无副作用）。
pub(crate) fn read(content: &str) -> DisplayRead {
    // 开启括号总数前置筛：超包络输入不做模型解析（进程级栈溢出不可
    // 捕获），三字段整体走兼容臂——值行为与旧实现逐值一致，仅路径分类不同。
    if bracket_open_count(content) > MODEL_PARSE_MAX_BRACKET_DEPTH {
        let reason = ReadPath::Compat(CompatReason::DeepNesting);
        return DisplayRead {
            name: legacy_scan(content, "name", false),
            title: legacy_scan(content, "title", true),
            title_zh: legacy_scan(content, "title_zh", true),
            name_path: reason,
            title_path: reason,
            title_zh_path: reason,
        };
    }
    let parsed = parse_document(content, SourceInfo::inline());
    let (name, name_path) = read_field(
        content,
        &parsed,
        "name",
        false, // 旧 parse_pac_name：首匹配即返回，空值也是 Some("")
        |d| d.identity.name.as_ref(),
    );
    let (title, title_path) = read_field(
        content,
        &parsed,
        "title",
        true, // 旧 parse_pac_scalar：空值跳过并继续扫描
        |d| d.display.title.as_ref(),
    );
    let (title_zh, title_zh_path) = read_field(content, &parsed, "title_zh", true, |d| {
        d.display.title_zh.as_ref()
    });
    DisplayRead {
        name,
        title,
        title_zh,
        name_path,
        title_path,
        title_zh_path,
    }
}

/// 旧行扫描的行门槛（冻结）：trim 后以 `key:` 开头。
fn gate_line(line: &str, key_with_colon: &str) -> bool {
    line.trim().starts_with(key_with_colon)
}

/// 兼容臂：旧 parse_pac_scalar/parse_pac_name 行循环的逐字搬迁（唯一副本）。
/// `skip_empty` 区分两族空值语义：title 族跳空续扫，name 首匹配即返回。
fn legacy_scan(content: &str, key: &str, skip_empty: bool) -> Option<String> {
    let key_with_colon = format!("{key}:");
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with(&key_with_colon) {
            if let Some(colon_pos) = line.find(':') {
                let value = line[colon_pos + 1..].trim();
                let value = value.trim_end_matches(',');
                let value = value.trim_matches('"').trim_matches('\'');
                if !skip_empty || !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

/// 单行形态谓词：对（唯一）声明行复算一次旧行归一。与 [`legacy_scan`] 的
/// 值处理相同，但只看一行——用于 Model 准入，不是双臂对拍。
fn legacy_line_value(line: &str) -> Option<String> {
    let colon_pos = line.find(':')?;
    let value = line[colon_pos + 1..].trim();
    let value = value.trim_end_matches(',');
    let value = value.trim_matches('"').trim_matches('\'');
    Some(value.to_string())
}

/// 单字段准入判定与读取。`model_value` 从共享 descriptor 取该字段的模型值。
fn read_field(
    content: &str,
    parsed: &Result<ProjectDescriptor, auto_project_model::ProjectModelError>,
    key: &str,
    skip_empty: bool,
    model_value: impl Fn(&ProjectDescriptor) -> Option<&String>,
) -> (Option<String>, ReadPath) {
    let descriptor = match parsed {
        Ok(d) => d,
        Err(_) => {
            return (
                legacy_scan(content, key, skip_empty),
                ReadPath::Compat(CompatReason::ParseError),
            )
        }
    };
    let key_with_colon = format!("{key}:");
    let gate: Vec<usize> = content
        .lines()
        .enumerate()
        .filter(|(_, line)| gate_line(line, &key_with_colon))
        .map(|(i, _)| i + 1)
        .collect();
    let prov: Vec<usize> = descriptor
        .provenance
        .iter()
        .filter(|(field, _)| *field == key)
        .map(|(_, line)| *line)
        .collect();
    if gate != prov {
        return (
            legacy_scan(content, key, skip_empty),
            ReadPath::Compat(CompatReason::HiddenOrPhantomLine),
        );
    }
    if descriptor.duplicates.iter().any(|d| d.key == key) {
        return (
            legacy_scan(content, key, skip_empty),
            ReadPath::Compat(CompatReason::DuplicateKey),
        );
    }
    match model_value(descriptor) {
        Some(value) => {
            let line = content.lines().nth(gate[0] - 1).expect("gate line exists");
            let admitted = legacy_line_value(line.trim()).as_deref() == Some(value.as_str());
            if admitted {
                (Some(value.clone()), ReadPath::Model(ModelState::Value))
            } else {
                (
                    legacy_scan(content, key, skip_empty),
                    ReadPath::Compat(CompatReason::LineFormMismatch),
                )
            }
        }
        None => {
            if gate.is_empty() {
                return (None, ReadPath::Model(ModelState::Absent));
            }
            let line = content.lines().nth(gate[0] - 1).expect("gate line exists");
            let legacy_of_line = legacy_line_value(line.trim());
            if skip_empty && legacy_of_line.is_none_or(|v| v.is_empty()) {
                // title 族：旧行为跳过空值、后续无门槛行 → None
                (None, ReadPath::Model(ModelState::DeclaredEmpty))
            } else {
                // 旧行归一非空而模型为 None：要么值被分类为 unsupported
                // （单引号/bare/插值…），要么是静态值的归一差异（引号内仅
                // 空白被模型 trim 掉、nil 的 astr 差异）——按证据区分命名。
                let reason = if descriptor
                    .unsupported
                    .iter()
                    .any(|u| u.key.as_deref() == Some(key))
                {
                    CompatReason::UnsupportedValue
                } else {
                    CompatReason::LineFormMismatch
                };
                (
                    legacy_scan(content, key, skip_empty),
                    ReadPath::Compat(reason),
                )
            }
        }
    }
}
