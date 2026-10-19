//! 渲染/后端选择的轻量单源规则（PLAN-809 T-03）。
//!
//! `BackendName` 是 `auto_lang::config::BackendType` 的轻等价镜像
//! （from_str/output_dir 语义逐字对齐），让本 crate 不依赖 auto-lang；
//! auto-man adapter 在门面层做两者间的转换（转换是纯枚举映射）。
//! `RenderSelection` 承载 pac.at `render:`/`backend:` 声明的归一结果，
//! 解析规则与 `BackendConfig::from_value` 逐字对齐（Str/Array/Obj 三形态、
//! 单元素数组折叠 Single、未知名丢弃、空集→无声明）。

use auto_val::{AutoStr, Node, Value};

/// 轻量后端名（`auto_lang::config::BackendType` 的语义镜像）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BackendName {
    Vue,
    Jet,
    Tauri,
    Gpui,
    Iced,
    Arkts,
    Cangjie,
    Godot,
    Rust,
    Vm,
    Vscode,
}

impl BackendName {
    /// 与 `BackendType::from_str` 逐字对齐（trim + lowercase；`ark` 别名、
    /// `remote` 按 vm 前端解析——PLAN-690 T-04）。
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "vue" => Some(Self::Vue),
            "jet" => Some(Self::Jet),
            "tauri" => Some(Self::Tauri),
            "gpui" => Some(Self::Gpui),
            "iced" => Some(Self::Iced),
            "arkts" | "ark" => Some(Self::Arkts),
            "cangjie" => Some(Self::Cangjie),
            "godot" => Some(Self::Godot),
            "rust" => Some(Self::Rust),
            "vm" => Some(Self::Vm),
            "remote" => Some(Self::Vm),
            "vscode" => Some(Self::Vscode),
            _ => None,
        }
    }

    /// 输出目录名（与 `BackendType::output_dir` 逐字对齐；注意 rust → "back"）。
    pub fn output_dir(&self) -> &'static str {
        match self {
            Self::Vue => "vue",
            Self::Jet => "jet",
            Self::Tauri => "tauri",
            Self::Gpui => "gpui",
            Self::Iced => "iced",
            Self::Arkts => "arkts",
            Self::Cangjie => "cangjie",
            Self::Godot => "godot",
            Self::Rust => "back",
            Self::Vm => "vm",
            Self::Vscode => "vscode",
        }
    }

    /// 类型名字符串（与 `BackendType::as_str` 逐字对齐；rust → "rust"）。
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Vue => "vue",
            Self::Jet => "jet",
            Self::Tauri => "tauri",
            Self::Gpui => "gpui",
            Self::Iced => "iced",
            Self::Arkts => "arkts",
            Self::Cangjie => "cangjie",
            Self::Godot => "godot",
            Self::Rust => "rust",
            Self::Vm => "vm",
            Self::Vscode => "vscode",
        }
    }
}

/// `render:`/`backend:` 声明的归一选择（`BackendConfig` 语义的轻承载）。
#[derive(Debug, Clone, PartialEq)]
pub enum RenderSelection {
    /// 无有效声明（Pac.backend_config = None 的臂）。
    Absent,
    /// 单后端。
    Single(BackendName),
    /// 多前端。
    Multi(Vec<BackendName>),
    /// 前后端分离。
    Split {
        front: Vec<BackendName>,
        back: BackendName,
    },
}

impl RenderSelection {
    /// 与 `BackendConfig::from_value` 逐字对齐的 Value 解析。
    pub fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::Str(s) => BackendName::from_str(&s).map(Self::Single),
            Value::Array(arr) => {
                let frontends: Vec<BackendName> = arr
                    .iter()
                    .filter_map(|v| match v {
                        Value::Str(s) => BackendName::from_str(&s),
                        _ => None,
                    })
                    .collect();
                if frontends.len() == 1 {
                    Some(Self::Single(frontends[0]))
                } else if !frontends.is_empty() {
                    Some(Self::Multi(frontends))
                } else {
                    None
                }
            }
            Value::Obj(obj) => {
                let front = obj.get("front").and_then(|v| match v {
                    Value::Str(s) => BackendName::from_str(&s).map(|t| vec![t]),
                    Value::Array(arr) => Some(
                        arr.iter()
                            .filter_map(|v| match v {
                                Value::Str(s) => BackendName::from_str(&s),
                                _ => None,
                            })
                            .collect(),
                    ),
                    _ => None,
                });
                let back = obj.get("back").and_then(|v| match v {
                    Value::Str(s) => BackendName::from_str(&s),
                    _ => None,
                });
                match (front, back) {
                    (Some(f), Some(b)) => Some(Self::Split { front: f, back: b }),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// 从 pac 根 props 读 render/backend 对：render 非空优先，回退 backend
    /// （`Pac::new` 的 effective_value 选择规则）。
    pub fn from_root_props(root: &Node) -> Option<Self> {
        let render_value = root.get_prop("render");
        let backend_value = root.get_prop("backend");
        let effective = if !render_value.to_astr().is_empty() {
            render_value
        } else {
            backend_value
        };
        Self::from_value(&effective)
    }

    /// 前端后端列表（`Pac::frontend_types` 的规则半；legacy 回退由
    /// [`frontend_types_with_legacy`] 承担）。
    pub fn frontends(&self) -> Vec<BackendName> {
        match self {
            Self::Single(t) => vec![*t],
            Self::Multi(types) => types.clone(),
            Self::Split { front, .. } => front.clone(),
            Self::Absent => Vec::new(),
        }
    }

    /// 分离模式的后端（`Pac::backend_type` 规则）。
    pub fn backend(&self) -> Option<BackendName> {
        match self {
            Self::Split { back, .. } => Some(*back),
            _ => None,
        }
    }

    /// 是否多前端分离（`Pac::has_multi_backend` 规则）。
    pub fn has_multi_backend(&self) -> bool {
        matches!(self, Self::Split { front, .. } if front.len() > 1)
    }
}

/// `Pac::frontend_types` 全规则：声明前端列表；无声明时回退解析 legacy
/// backend 字符串（`BackendType::from_str(&self.backend)` 臂）。
pub fn frontend_types_with_legacy(
    selection: &RenderSelection,
    legacy_backend: &AutoStr,
) -> Vec<BackendName> {
    match selection {
        RenderSelection::Absent => BackendName::from_str(legacy_backend)
            .map(|t| vec![t])
            .unwrap_or_default(),
        _ => selection.frontends(),
    }
}

/// `Pac::default_frontend` 全规则：首个前端；无声明回退 legacy 字符串。
pub fn default_frontend_with_legacy(
    selection: &RenderSelection,
    legacy_backend: &AutoStr,
) -> Option<BackendName> {
    frontend_types_with_legacy(selection, legacy_backend)
        .into_iter()
        .next()
}

/// `Pac::output_dirs` 规则：前端目录 + 分离后端目录。
pub fn output_dirs(selection: &RenderSelection) -> Vec<(BackendName, AutoStr)> {
    let mut dirs = Vec::new();
    for frontend in selection.frontends() {
        dirs.push((frontend, frontend.output_dir().into()));
    }
    if let Some(back) = selection.backend() {
        dirs.push((back, back.output_dir().into()));
    }
    dirs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_name_from_str_alignment() {
        assert_eq!(BackendName::from_str("vue"), Some(BackendName::Vue));
        assert_eq!(BackendName::from_str(" Vue "), Some(BackendName::Vue));
        assert_eq!(BackendName::from_str("ark"), Some(BackendName::Arkts));
        assert_eq!(BackendName::from_str("arkts"), Some(BackendName::Arkts));
        assert_eq!(BackendName::from_str("remote"), Some(BackendName::Vm));
        assert_eq!(BackendName::from_str("rust-tauri"), None);
        assert_eq!(BackendName::from_str(""), None);
        // output_dir / as_str 对齐（rust 的两义）
        assert_eq!(BackendName::Rust.output_dir(), "back");
        assert_eq!(BackendName::Rust.as_str(), "rust");
    }

    #[test]
    fn single_element_array_folds_to_single() {
        let arr = Value::Array(auto_val::Array::from_vec(vec![Value::Str("vue".into())]));
        assert_eq!(
            RenderSelection::from_value(&arr),
            Some(RenderSelection::Single(BackendName::Vue))
        );
        let two = Value::Array(auto_val::Array::from_vec(vec![
            Value::Str("vue".into()),
            Value::Str("tauri".into()),
        ]));
        assert!(matches!(
            RenderSelection::from_value(&two),
            Some(RenderSelection::Multi(v)) if v.len() == 2
        ));
    }

    #[test]
    fn legacy_fallback_matches_pac_arm() {
        // 无声明 + legacy "rust" → [Rust]（Pac::frontend_types 回退臂）
        let sel = RenderSelection::Absent;
        assert_eq!(
            frontend_types_with_legacy(&sel, &"rust".into()),
            vec![BackendName::Rust]
        );
        // 无声明 + 未知 legacy → 空
        assert!(frontend_types_with_legacy(&sel, &"nope".into()).is_empty());
        // Split 前端列表
        let split = RenderSelection::Split {
            front: vec![BackendName::Vue],
            back: BackendName::Rust,
        };
        assert_eq!(split.frontends(), vec![BackendName::Vue]);
        assert_eq!(split.backend(), Some(BackendName::Rust));
        assert_eq!(
            output_dirs(&split),
            vec![
                (BackendName::Vue, "vue".into()),
                (BackendName::Rust, "back".into()),
            ]
        );
    }
}
