use crate::eval_config_with_vm;
use crate::AutoResult;
use auto_val::Obj;
use auto_val::{AutoPath, AutoStr, Node, Value};
use std::path::Path;
use std::path::PathBuf;

pub struct AutoConfigReader {
    args: Obj,
}

impl AutoConfigReader {
    pub fn new() -> Self {
        Self { args: Obj::new() }
    }

    pub fn skip_check(self) -> Self {
        // No-op in AutoVM mode (skip_check was an Interpreter feature)
        self
    }

    pub fn args(mut self, args: &Obj) -> Self {
        self.args = args.clone();
        self
    }

    pub fn parse(&mut self, code: impl Into<AutoStr>) -> AutoResult<AutoConfig> {
        let code = code.into();

        // Plan 091: Use AutoVM without Universe
        let result = eval_config_with_vm(code.as_str(), &self.args)?;

        Ok(AutoConfig {
            code: code.to_string(),
            root: result.to_node(),
            args: self.args.clone(),
        })
    }

    pub fn read(&mut self, path: impl Into<PathBuf>) -> AutoResult<AutoConfig> {
        let path = path.into();
        let content = std::fs::read_to_string(&path).map_err(|e| {
            format!(
                "Failed to read config file {}: {}",
                path.to_str().unwrap(),
                e
            )
        })?;
        self.parse(content)
    }
}

pub struct AutoConfig {
    pub code: String,
    pub root: Node,
    pub args: Obj,
}

impl AutoConfig {
    pub fn read(path: &Path) -> AutoResult<Self> {
        Self::from_file(path, &Obj::default())
    }

    pub fn from_file(path: &Path, args: &Obj) -> AutoResult<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            format!(
                "Failed to read config file {}: {}",
                path.to_str().unwrap(),
                e
            )
        })?;
        Self::from_code(content, args)
    }

    pub fn new(code: impl Into<String>) -> AutoResult<Self> {
        Self::from_code(code, &Obj::new())
    }

    pub fn save(&mut self, path: &AutoPath) -> AutoResult<()> {
        let contents = self.root.contents();
        std::fs::write(path.path(), contents.join("\n"))
            .map_err(|e| format!("Failed to write file: {}", e))?;
        Ok(())
    }

    pub fn from_code(code: impl Into<String>, args: &Obj) -> AutoResult<Self> {
        let code = code.into();

        // Plan 091: Use AutoVM without Universe
        let result = eval_config_with_vm(&code, args)?;

        if let Value::Node(root) = result {
            Ok(Self {
                code: code.clone(),
                args: args.clone(),
                root,
            })
        } else {
            // For empty config files (Nil result), return an empty root Node instead of error
            if matches!(result, Value::Nil) {
                return Ok(Self {
                    code: code.clone(),
                    args: args.clone(),
                    root: auto_val::Node::new("root"),
                });
            }

            Err(format!("Invalid config result: {}", result.repr()).into())
        }
    }

    pub fn name(&self) -> AutoStr {
        self.root.get_prop("name").to_astr()
    }

    pub fn version(&self) -> AutoStr {
        self.root.get_prop("version").to_astr()
    }

    pub fn list_target_names(&self) -> Vec<AutoStr> {
        self.root
            .kids_iter()
            .filter(|(_, kid)| matches!(kid, auto_val::Kid::Node(_)))
            .map(|(_, kid)| {
                if let auto_val::Kid::Node(n) = kid {
                    n.title()
                } else {
                    unreachable!()
                }
            })
            .collect()
    }

    pub fn to_xml(&self) -> AutoStr {
        AutoStr::new()
    }

    // pub fn eval(&mut self, code: impl Into<AutoStr>) -> Value {
    // self.interpreter.eval(code.into().as_str())
    // }
}

/// 后端类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BackendType {
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

impl BackendType {
    /// 从字符串解析后端类型
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
            "vscode" => Some(Self::Vscode),
            _ => None,
        }
    }

    /// 获取输出目录名
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

    /// 获取后端类型名称字符串
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

    /// 返回所有可用的后端类型
    pub fn all_variants() -> Vec<BackendType> {
        vec![
            Self::Vue, Self::Jet, Self::Tauri, Self::Gpui,
            Self::Iced, Self::Arkts, Self::Cangjie, Self::Godot,
            Self::Rust, Self::Vm, Self::Vscode,
        ]
    }
}

/// 后端配置（单后端或多后端）
#[derive(Debug, Clone, PartialEq)]
pub enum BackendConfig {
    /// 单后端：整个项目都是同一种类型
    Single(BackendType),
    /// 多前端：多个前端框架（如 vue + tauri）
    Multi(Vec<BackendType>),
    /// 前后端分离
    Split {
        front: Vec<BackendType>,
        back: BackendType,
    },
}

impl BackendConfig {
    /// 从字符串解析
    pub fn parse(s: &str) -> Option<Self> {
        BackendType::from_str(s).map(Self::Single)
    }

    /// 从 Value 解析（支持对象形式和数组形式）
    /// - 字符串: backend: "vue"
    /// - 数组: backend: ["vue", "tauri"] (多个前端框架)
    /// - 对象: backend: { front: "vue", back: "rust" }
    pub fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::Str(s) => Self::parse(&s),
            Value::Array(arr) => {
                // 数组形式：backend: ["vue", "tauri"]
                let frontends: Vec<BackendType> = arr.iter()
                    .filter_map(|v| match v {
                        Value::Str(s) => BackendType::from_str(&s),
                        _ => None,
                    })
                    .collect();
                if frontends.len() == 1 {
                    Some(Self::Single(frontends[0].clone()))
                } else if !frontends.is_empty() {
                    Some(Self::Multi(frontends))
                } else {
                    None
                }
            }
            Value::Obj(obj) => {
                let front = obj.get("front").and_then(|v| match v {
                    Value::Str(s) => BackendType::from_str(&s).map(|t| vec![t]),
                    Value::Array(arr) => Some(
                        arr.iter()
                            .filter_map(|v| match v {
                                Value::Str(s) => BackendType::from_str(&s),
                                _ => None,
                            })
                            .collect()
                    ),
                    _ => None,
                });
                let back = obj.get("back").and_then(|v| match v {
                    Value::Str(s) => BackendType::from_str(&s),
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

    /// 获取所有前端后端类型
    pub fn frontends(&self) -> Vec<&BackendType> {
        match self {
            Self::Single(t) => vec![t],
            Self::Multi(types) => types.iter().collect(),
            Self::Split { front, .. } => front.iter().collect(),
        }
    }

    /// 获取后端类型
    pub fn backend(&self) -> Option<&BackendType> {
        match self {
            Self::Single(_) => None,
            Self::Multi(_) => None,
            Self::Split { back, .. } => Some(back),
        }
    }
}
