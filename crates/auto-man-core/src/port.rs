//! Port 数据结构与 port 声明解析（PLAN-809 T-03 自 auto-man/src/port.rs
//! 迁入；`Port::default()` 的 `env::consts::OS/ARCH` 是编译期目标平台常量
//! 而非运行期环境观测，逐字保留 = 零行为漂移。Port 节点完整解析见
//! `port_from_node`。）

use auto_toolchain_core::config::{
    CompilerConfig, CompilerKind, CompilerLocation, ExecutableType, FlagFormat, FlagMappings,
};
use auto_val::{AutoStr, Kid, Node, Value, ValueKey};

use crate::node_ext::NodeExt;

#[derive(Debug, Clone, PartialEq)]
pub struct Port {
    pub name: AutoStr,
    pub builder: AutoStr,
    pub platform: AutoStr,
    pub at: AutoStr,

    // BPBE Architecture Additions
    pub os: Option<AutoStr>,
    pub arch: Option<AutoStr>,
    pub toolchain: Option<AutoStr>,
    pub exports: Vec<AutoStr>,
    pub sdks: Vec<Node>,

    /// 编译器配置 (可选)
    pub compiler: Option<CompilerConfig>,
}

impl Default for Port {
    fn default() -> Self {
        match std::env::consts::OS {
            "windows" => {
                let compiler = CompilerConfig::msvc_default();
                Self {
                    name: "windows_ninja".into(),
                    builder: "ninja".into(),
                    platform: "windows".into(),
                    at: "build".into(),
                    os: Some("windows".into()),
                    arch: Some(std::env::consts::ARCH.into()),
                    toolchain: Some("msvc".into()),
                    exports: vec![],
                    sdks: vec![],
                    compiler: Some(compiler),
                }
            }
            _ => {
                let compiler = CompilerConfig::gcc_default();
                Self {
                    name: "linux_ninja".into(),
                    builder: "ninja".into(),
                    platform: "linux".into(),
                    at: "build".into(),
                    os: Some("linux".into()),
                    arch: Some(std::env::consts::ARCH.into()),
                    toolchain: Some("gcc".into()),
                    exports: vec![],
                    sdks: vec![],
                    compiler: Some(compiler),
                }
            }
        }
    }
}

/// port 节点 → Port 的纯解析（含完整编译器配置）。
/// （PLAN-809 T-03 自 auto-man/src/pac.rs `Pac::merge_port` 的解析段
/// 逐字迁入；props 合并/targets 收集等状态应用留在 auto-man 侧。）
pub fn port_from_node(port_node: &Node) -> Port {
    let mut port = Port::default();
    port.name = port_node.main_arg().to_astr();

    // Use "ninja" as default builder if not specified
    let builder_val = port_node.get_prop("builder").to_astr();
    port.builder = if builder_val.is_empty() {
        "ninja".into()
    } else {
        builder_val
    };

    port.platform = port_node.get_prop("platform").to_astr();
    port.at = port_node.get_prop("at").to_astr();

    // BPBE Architecture Fields
    if port_node.has_prop("os") {
        port.os = Some(port_node.get_prop("os").to_astr());
    }
    if port_node.has_prop("arch") {
        port.arch = Some(port_node.get_prop("arch").to_astr());
    }
    if port_node.has_prop("toolchain") {
        port.toolchain = Some(port_node.get_prop("toolchain").to_astr());
    }

    let exports_val = port_node.get_prop_of("exports");
    if let Value::Array(arr) = exports_val {
        port.exports = arr.values.iter().map(|v| v.to_astr()).collect();
    }

    port.sdks = port_node.get_nodes("sdk");

    // 解析完整的编译器配置
    if let Some(Kid::Node(compiler_node)) = port_node.get_kid(&ValueKey::Str("compiler".into())) {
        parse_compiler_into(&mut port, compiler_node);
    }

    port
}

fn parse_compiler_into(port: &mut Port, compiler_node: &Node) {
    // 解析编译器类型
    let kind_str = compiler_node.get_prop("kind").to_astr();
    let Some(kind) = CompilerKind::from_str(kind_str.as_str()) else {
        return;
    };
    // 解析 location
    let location = if let Some(Kid::Node(location_node)) =
        compiler_node.get_kid(&ValueKey::Str("location".into()))
    {
        let location_type = location_node.main_arg().to_astr();
        match location_type.as_str() {
            "Env" => CompilerLocation::Env,
            "Dir" => {
                let dir_str = location_node.get_prop("path").to_astr();
                CompilerLocation::Dir(auto_val::AutoPath::new(dir_str.as_str()))
            }
            "Executable" => {
                let exec_type_str = location_node.get_prop("type").to_astr();
                let exec_type = ExecutableType::from_str(exec_type_str.as_str());
                if let Some(exec_type) = exec_type {
                    let path =
                        auto_val::AutoPath::new(location_node.get_prop("path").to_astr().as_str());
                    CompilerLocation::Executable(exec_type, path)
                } else {
                    CompilerLocation::Env
                }
            }
            _ => CompilerLocation::Env,
        }
    } else {
        CompilerLocation::Env
    };

    // 解析 executables
    let mut executables = std::collections::HashMap::new();
    if let Some(Kid::Node(execs_node)) = compiler_node.get_kid(&ValueKey::Str("executables".into()))
    {
        if execs_node.has_prop("cc") {
            let cc = auto_val::AutoPath::new(execs_node.get_prop("cc").to_astr().as_str());
            executables.insert(ExecutableType::Compiler, cc);
        }
        if execs_node.has_prop("ar") {
            let ar = auto_val::AutoPath::new(execs_node.get_prop("ar").to_astr().as_str());
            executables.insert(ExecutableType::Archiver, ar);
        }
        if execs_node.has_prop("link") {
            let link = auto_val::AutoPath::new(execs_node.get_prop("link").to_astr().as_str());
            executables.insert(ExecutableType::Linker, link);
        }
        if execs_node.has_prop("as") {
            let as_cmd = auto_val::AutoPath::new(execs_node.get_prop("as").to_astr().as_str());
            executables.insert(ExecutableType::Assembler, as_cmd);
        }
    }

    // 解析 flags
    let flags = if let Some(Kid::Node(flags_node)) =
        compiler_node.get_kid(&ValueKey::Str("flags".into()))
    {
        let parse_flag_format = |flag_name: &str| -> FlagFormat {
            let flag_value = flags_node.get_str_or(flag_name, "-");
            if flag_value.contains('(') {
                let left_paren = flag_value.find('(').unwrap_or(0);
                let right_paren = flag_value.find(')').unwrap_or(flag_value.len());
                let format_type = &flag_value[0..left_paren];
                let format_arg = &flag_value[left_paren + 1..right_paren];

                match format_type {
                    "Prefix" => FlagFormat::Prefix(format_arg.to_string()),
                    "Postfix" => FlagFormat::Postfix(format_arg.to_string()),
                    "Both" => {
                        let args: Vec<&str> = format_arg.split(", ").collect();
                        if args.len() == 2 {
                            FlagFormat::Both(args[0].to_string(), args[1].to_string())
                        } else {
                            FlagFormat::Prefix(args[0].to_string())
                        }
                    }
                    _ => FlagFormat::Prefix(format_arg.to_string()),
                }
            } else {
                FlagFormat::Prefix(flag_value.as_str().to_string())
            }
        };

        FlagMappings {
            include: parse_flag_format("include"),
            define: parse_flag_format("define"),
            library: parse_flag_format("library"),
            library_path: parse_flag_format("library_path"),
            output: parse_flag_format("output"),
            compile_only: parse_flag_format("compile_only"),
        }
    } else {
        // 使用默认 flags
        match kind {
            CompilerKind::MSVC => FlagMappings::msvc_default(),
            CompilerKind::GCC => FlagMappings::gcc_default(),
            CompilerKind::Clang => FlagMappings::clang_default(),
            CompilerKind::IAR => FlagMappings::iar_default(),
            CompilerKind::GHS => FlagMappings::ghs_default(),
            CompilerKind::Targeting => FlagMappings::targeting_default(),
            CompilerKind::Hightec => FlagMappings::hightec_default(),
            CompilerKind::TICLang => FlagMappings::ticlang_default(),
        }
    };

    // 解析 default_cflags
    let default_cflags = if let Some(Kid::Node(cflags_node)) =
        compiler_node.get_kid(&ValueKey::Str("default_cflags".into()))
    {
        cflags_node.get_str_vec_or("value")
    } else {
        Vec::new()
    };

    // 创建编译器配置
    let compiler_config = CompilerConfig {
        kind,
        name: port.name.clone(),
        executables,
        flags,
        location,
        default_cflags,
    };

    port.compiler = Some(compiler_config);
}
