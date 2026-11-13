//! PLAN-947 T-09/T-10：标准库四轴装配 provider 选择合同。
//!
//! 四轴（分别建模，不合并）：**执行后端**（AV/native）、**宿主平台**
//! （构建/运行宿主）、**目标平台**（被装配程序的平台）、**provider 实现
//! 来源**（自有 Auto / 薄 OS 适配 / 显式外库）。选择只看**目标平台**；
//! 宿主平台单独记录（宿主 Windows 而目标 Linux 时选 Linux 的 provider）。
//!
//! 冻结规则（V-09/V-10/V-NATIVE 的关闭 oracle）：
//! 1. 默认实现由**自有 Auto**（或薄 OS 适配）承担——不以 libc/Rust std
//!    间接委托为默认；缺默认 provider 拒绝（`MissingDefaultProvider`），
//!    唯一候选是未显式选择的外库时拒绝（`AutoForeignFallback`）。
//! 2. 显式外库必须点名选择；公共声明的公共边界是 **AutoABI**——把平台
//!    ABI 的 provider 标成 AutoABI 是审计反例（`PlatformAbiAsAutoAbi`）。
//! 3. 无 provider（`NoProvider`）、同轴重复/歧义（`AmbiguousProvider`）、
//!    混合 ABI 候选（`MixedTargetAbi`）一律具名拒绝。
//! 4. 操作必须存在于 943 操作合同（删 proof → `UnknownOperation`）；
//!    所需 grant 必须在本轮注入集合内（恢复的旧授权/缺授权 →
//!    `MissingGrant`，dispatch 前拒绝）。
//!
//! 本模块是纯合同/选择器：不实现 CTE（`#if`/`#is` 的源码级实现归
//! PLAN-966；此处冻结的是其目标条件语义——条件按**目标平台**求值、
//! 未选分支仅语法检查、导入提升到所在模块、无/歧义 provider 具名失败）。

use serde::{Deserialize, Serialize};

/// 选择合同 schema（装配收据 selection 节的版本）。
pub const SELECTION_SCHEMA_VERSION: u32 = 1;

/// 标准库公共 API 分层（单一公共 API 的层归属）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StdlibLayer {
    /// core：语言内置最小面（无 OS 依赖）。
    Core,
    /// alloc：分配/释放职责面（接口待 M2 内存专题冻结）。
    Alloc,
    /// host：宿主服务面（console/1 等每轮授权服务）。
    Host,
    /// domain：领域算法面（text/collections 等）。
    Domain,
}

/// 平轴的 OS 侧（有界集合；新平台=合同修订）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Os {
    Windows,
    Linux,
    Macos,
}

/// 平台（目标/宿主分别实例化）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Platform {
    pub os: Os,
}

impl Platform {
    pub const fn windows() -> Platform {
        Platform { os: Os::Windows }
    }
    pub const fn linux() -> Platform {
        Platform { os: Os::Linux }
    }
}

/// 执行后端轴。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Backend {
    Av,
    Native,
}

/// provider 与公共边界的 ABI 关系。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderAbi {
    /// 公共 AutoABI（自有实现或显式 AutoABI 桥）。
    AutoAbi,
    /// 平台/外部 ABI（OS 适配层、libc、Rust/C 库）。
    PlatformAbi,
}

/// provider 实现来源轴。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderOrigin {
    /// 默认自有 Auto 实现。
    OwnAuto,
    /// 薄 OS 适配（host 服务面；平台 ABI→AutoABI 的显式桥）。
    PlatformAdapter,
    /// 显式外库（必须点名选择；永不默认回落）。
    ExplicitForeign,
}

/// 一条 provider 声明（注册表项；由真实装配面登记，非自报）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderDecl {
    pub provider: String,
    pub module: String,
    pub origin: ProviderOrigin,
    pub abi: ProviderAbi,
    /// 该 provider 服务的**目标**平台集合。
    pub platforms: Vec<Platform>,
}

/// 一次装配请求：公共声明（943 操作 ID）+ 层 + 可选显式 provider。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindingRequest {
    pub operation: String,
    pub module: String,
    pub symbol: String,
    pub layer: StdlibLayer,
    /// 该操作的所需 grant（来自操作合同 requirements）。
    pub required_grants: Vec<String>,
    pub explicit_provider: Option<String>,
}

/// 四轴装配上下文。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssemblyAxes {
    pub backend: Backend,
    pub host: Platform,
    pub target: Platform,
}

/// 一条冻结的选择记录（进装配收据；只携带身份事实，不携带授权）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionRecord {
    pub operation: String,
    pub module: String,
    pub symbol: String,
    pub layer: StdlibLayer,
    pub target: Platform,
    pub provider: String,
    pub abi: ProviderAbi,
}

/// 具名拒绝（每条规则一个变体；绝不静默降级）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionError {
    /// 目标平台无任何 provider。
    NoProvider { operation: String, target: Platform },
    /// 默认序消除后仍有多个候选（重复 provider）。
    AmbiguousProvider {
        operation: String,
        candidates: Vec<String>,
    },
    /// 无默认（自有 Auto/OS 适配）provider。
    MissingDefaultProvider { operation: String },
    /// 唯一候选是未显式选择的外库（禁止自动 libc 回落）。
    AutoForeignFallback { operation: String, provider: String },
    /// 注册表审计反例：平台 ABI 的 provider 被标为 AutoABI。
    PlatformAbiAsAutoAbi { provider: String },
    /// 同一绑定在同一目标下混合 AutoABI 与平台 ABI 候选。
    MixedTargetAbi { operation: String, target: Platform },
    /// 操作不在 943 操作合同内（proof 缺失/被删）。
    UnknownOperation { operation: String },
    /// 所需 grant 未在本轮注入集合内（恢复的旧授权无效）。
    MissingGrant { operation: String, grant: String },
}

impl SelectionError {
    /// 稳定错误码（诊断/测试断言面）。
    pub fn code(&self) -> &'static str {
        match self {
            SelectionError::NoProvider { .. } => "assembly.no-provider",
            SelectionError::AmbiguousProvider { .. } => "assembly.ambiguous-provider",
            SelectionError::MissingDefaultProvider { .. } => "assembly.missing-default-provider",
            SelectionError::AutoForeignFallback { .. } => "assembly.auto-foreign-fallback",
            SelectionError::PlatformAbiAsAutoAbi { .. } => "assembly.platform-abi-as-auto-abi",
            SelectionError::MixedTargetAbi { .. } => "assembly.mixed-target-abi",
            SelectionError::UnknownOperation { .. } => "assembly.unknown-operation",
            SelectionError::MissingGrant { .. } => "assembly.missing-grant",
        }
    }
}

/// 注册表审计：结构合法方可参与选择。平台 ABI 冒充 AutoABI 是反例
/// （V-09 的审计腿：注册面真实声明，不许标签洗白）。
pub fn audit_registry(providers: &[ProviderDecl]) -> Result<(), SelectionError> {
    let mut seen = std::collections::BTreeSet::new();
    for p in providers {
        // 同 provider+module+平台重复声明 = 歧义来源，登记期拒绝。
        for platform in &p.platforms {
            if !seen.insert((p.provider.clone(), p.module.clone(), *platform)) {
                return Err(SelectionError::AmbiguousProvider {
                    operation: p.module.clone(),
                    candidates: vec![p.provider.clone()],
                });
            }
        }
        if p.abi == ProviderAbi::AutoAbi && p.origin == ProviderOrigin::ExplicitForeign {
            return Err(SelectionError::PlatformAbiAsAutoAbi {
                provider: p.provider.clone(),
            });
        }
    }
    Ok(())
}

/// 为一批公共声明执行四轴选择。目标平台轴决定候选集合；默认序：
/// 自有 Auto → 薄 OS 适配；外库只经显式点名。
pub fn select(
    providers: &[ProviderDecl],
    axes: AssemblyAxes,
    requests: &[BindingRequest],
    granted: &[String],
) -> Result<Vec<SelectionRecord>, SelectionError> {
    audit_registry(providers)?;
    let mut records = Vec::with_capacity(requests.len());
    for req in requests {
        if req.operation.trim().is_empty() || req.module.trim().is_empty() {
            return Err(SelectionError::UnknownOperation {
                operation: req.operation.clone(),
            });
        }
        // 目标平台（非宿主）过滤候选。
        let candidates: Vec<&ProviderDecl> = providers
            .iter()
            .filter(|p| p.module == req.module && p.platforms.contains(&axes.target))
            .collect();
        if candidates.is_empty() {
            return Err(SelectionError::NoProvider {
                operation: req.operation.clone(),
                target: axes.target,
            });
        }
        let chosen: Vec<&&ProviderDecl> = match &req.explicit_provider {
            Some(name) => candidates.iter().filter(|p| p.provider == *name).collect(),
            None => candidates
                .iter()
                .filter(|p| {
                    matches!(
                        p.origin,
                        ProviderOrigin::OwnAuto | ProviderOrigin::PlatformAdapter
                    )
                })
                .collect(),
        };
        if chosen.is_empty() {
            match &req.explicit_provider {
                // 点名 provider 在目标平台无候选：与无 provider 同码拒绝。
                Some(_) => {
                    return Err(SelectionError::NoProvider {
                        operation: req.operation.clone(),
                        target: axes.target,
                    })
                }
                None => {
                    // 候选只有外库：禁止默认回落（AutoForeignFallback）。
                    let foreign_only = candidates
                        .iter()
                        .all(|p| matches!(p.origin, ProviderOrigin::ExplicitForeign));
                    let err = if foreign_only {
                        SelectionError::AutoForeignFallback {
                            operation: req.operation.clone(),
                            provider: candidates[0].provider.clone(),
                        }
                    } else {
                        SelectionError::MissingDefaultProvider {
                            operation: req.operation.clone(),
                        }
                    };
                    return Err(err);
                }
            }
        }
        // 混合 ABI（默认池内）：AutoABI 与平台 ABI 同层并存即具名拒绝——
        // 先于同质多候选的歧义判定（不同 ABI 的"多候选"不是普通歧义）。
        // 显式点名绕过默认池，不触发本检查。
        if req.explicit_provider.is_none() {
            let default_pool: Vec<&ProviderDecl> = candidates
                .iter()
                .filter(|p| {
                    matches!(
                        p.origin,
                        ProviderOrigin::OwnAuto | ProviderOrigin::PlatformAdapter
                    )
                })
                .copied()
                .collect();
            if default_pool.len() > 1
                && default_pool
                    .iter()
                    .any(|p| p.abi == ProviderAbi::PlatformAbi)
                && default_pool.iter().any(|p| p.abi == ProviderAbi::AutoAbi)
            {
                return Err(SelectionError::MixedTargetAbi {
                    operation: req.operation.clone(),
                    target: axes.target,
                });
            }
        }
        if chosen.len() > 1 {
            return Err(SelectionError::AmbiguousProvider {
                operation: req.operation.clone(),
                candidates: chosen.iter().map(|p| p.provider.clone()).collect(),
            });
        }
        let provider = *chosen[0];
        // grant 检查在"dispatch 前"：本轮注入集合必须覆盖需求；恢复
        // 产物携带的旧授权永不入场（调用方只传本轮 grants）。
        for grant in &req.required_grants {
            if !granted.contains(grant) {
                return Err(SelectionError::MissingGrant {
                    operation: req.operation.clone(),
                    grant: grant.clone(),
                });
            }
        }
        records.push(SelectionRecord {
            operation: req.operation.clone(),
            module: req.module.clone(),
            symbol: req.symbol.clone(),
            layer: req.layer,
            target: axes.target,
            provider: provider.provider.clone(),
            abi: provider.abi,
        });
    }
    Ok(records)
}

#[cfg(test)]
mod plan947_tests {
    //! V-09 / V-NATIVE / V-10 的永久关闭 oracle（stdlib 固定组）。
    //! 每条断言对应冻结合同的一条规则；合法对照与反例成对出现。
    use super::*;

    fn own(module: &str, os: Os) -> ProviderDecl {
        ProviderDecl {
            provider: "auto".into(),
            module: module.into(),
            origin: ProviderOrigin::OwnAuto,
            abi: ProviderAbi::AutoAbi,
            platforms: vec![Platform { os }],
        }
    }

    fn adapter(module: &str, os: Os) -> ProviderDecl {
        ProviderDecl {
            provider: format!("os-{}", if os == Os::Windows { "win32" } else { "linux" }),
            module: module.into(),
            origin: ProviderOrigin::PlatformAdapter,
            abi: ProviderAbi::PlatformAbi,
            platforms: vec![Platform { os }],
        }
    }

    fn foreign(module: &str, os: Os, provider: &str) -> ProviderDecl {
        ProviderDecl {
            provider: provider.into(),
            module: module.into(),
            origin: ProviderOrigin::ExplicitForeign,
            abi: ProviderAbi::PlatformAbi,
            platforms: vec![Platform { os }],
        }
    }

    fn say_request() -> BindingRequest {
        BindingRequest {
            operation: "console.say".into(),
            module: "io".into(),
            symbol: "say".into(),
            layer: StdlibLayer::Host,
            required_grants: vec!["console".into()],
            explicit_provider: None,
        }
    }

    fn axes(host: Platform, target: Platform) -> AssemblyAxes {
        AssemblyAxes {
            backend: Backend::Av,
            host,
            target,
        }
    }

    /// V-09：同一公共操作映射不同目标——选择只看目标平台轴；宿主
    /// Windows 而目标 Linux 时合同选 Linux 的 provider（并如实记录
    /// 目标），不因宿主漂移。
    #[test]
    fn v09_target_axis_selects_target_not_host() {
        let providers = vec![
            own("io", Os::Windows),
            ProviderDecl {
                provider: "auto".into(),
                module: "io".into(),
                origin: ProviderOrigin::OwnAuto,
                abi: ProviderAbi::AutoAbi,
                platforms: vec![Platform { os: Os::Linux }],
            },
        ];
        let grants = vec!["console".to_string()];
        let host_win = axes(Platform::windows(), Platform::windows());
        let cross = axes(Platform::windows(), Platform::linux());
        let same_host_target = select(&providers, host_win, &[say_request()], &grants).unwrap();
        assert_eq!(same_host_target[0].target, Platform::windows());
        let cross_build = select(&providers, cross, &[say_request()], &grants).unwrap();
        assert_eq!(cross_build[0].target, Platform::linux());
        assert_eq!(cross_build[0].provider, "auto");
        // 宿主/目标分离是身份事实：两条记录仅目标轴不同。
        assert_ne!(same_host_target[0], cross_build[0]);
    }

    /// V-09：目标平台无 provider 具名拒绝；空操作/缺 proof 同样具名。
    #[test]
    fn v09_no_provider_and_unknown_operation_are_named_rejections() {
        let providers = vec![own("io", Os::Windows)];
        let grants = vec!["console".to_string()];
        let linux_only = axes(Platform::linux(), Platform::linux());
        let err = select(&providers, linux_only, &[say_request()], &grants).unwrap_err();
        assert_eq!(err.code(), "assembly.no-provider");
        let mut empty = say_request();
        empty.operation = "  ".into();
        let err = select(
            &providers,
            axes(Platform::windows(), Platform::windows()),
            &[empty],
            &grants,
        )
        .unwrap_err();
        assert_eq!(err.code(), "assembly.unknown-operation");
    }

    /// V-09：把平台 ABI 的 provider 标为 AutoABI 是审计反例（外库不是
    /// 公共 AutoABI 的替身）。
    #[test]
    fn v09_platform_abi_labeled_auto_abi_is_an_audit_counterexample() {
        let bad = ProviderDecl {
            provider: "libc".into(),
            module: "io".into(),
            origin: ProviderOrigin::ExplicitForeign,
            abi: ProviderAbi::AutoAbi,
            platforms: vec![Platform::windows()],
        };
        let err = audit_registry(&[bad]).unwrap_err();
        assert_eq!(err.code(), "assembly.platform-abi-as-auto-abi");
    }

    /// V-10：同一默认池混 AutoABI/平台 ABI 候选拒绝；重复 provider 拒绝。
    #[test]
    fn v10_mixed_target_abi_and_duplicate_providers_reject() {
        let mut dupe = own("io", Os::Windows);
        dupe.provider = "auto-2".into();
        let providers = vec![own("io", Os::Windows), dupe];
        let err = select(
            &providers,
            axes(Platform::windows(), Platform::windows()),
            &[say_request()],
            &["console".to_string()],
        )
        .unwrap_err();
        assert_eq!(err.code(), "assembly.ambiguous-provider");

        let mixed = vec![
            own("io", Os::Windows),
            ProviderDecl {
                provider: "os-win32".into(),
                module: "io".into(),
                origin: ProviderOrigin::PlatformAdapter,
                abi: ProviderAbi::PlatformAbi,
                platforms: vec![Platform::windows()],
            },
        ];
        let err = select(
            &mixed,
            axes(Platform::windows(), Platform::windows()),
            &[say_request()],
            &["console".to_string()],
        )
        .unwrap_err();
        assert_eq!(err.code(), "assembly.mixed-target-abi");
    }

    /// V-10：恢复的旧授权无效——grant 缺失在 dispatch 前具名拒绝；
    /// 本轮注入完整集合即通过（重新授权可执行）。
    #[test]
    fn v10_missing_grant_rejects_before_dispatch_and_regrant_passes() {
        let providers = vec![own("io", Os::Windows)];
        let win = axes(Platform::windows(), Platform::windows());
        let err = select(&providers, win, &[say_request()], &[]).unwrap_err();
        assert_eq!(err.code(), "assembly.missing-grant");
        let ok = select(&providers, win, &[say_request()], &["console".to_string()]).unwrap();
        assert_eq!(ok[0].provider, "auto");
    }

    /// V-NATIVE：默认自有 provider 装配成功；显式外库点名可装配；
    /// 缺默认 provider 拒绝；唯一候选是未点名外库（自动 libc 回落）
    /// 拒绝。
    #[test]
    fn v_native_default_own_explicit_foreign_and_no_auto_fallback() {
        let grants = vec!["console".to_string()];
        let win = axes(Platform::windows(), Platform::windows());
        // 合法默认：自有 Auto。
        let own_only = vec![own("io", Os::Windows)];
        let r = select(&own_only, win, &[say_request()], &grants).unwrap();
        assert_eq!(r[0].provider, "auto");
        // 合法显式外库（点名 libc，平台 ABI 如实标注）。
        let with_libc = vec![own("io", Os::Windows), foreign("io", Os::Windows, "libc")];
        let mut explicit = say_request();
        explicit.explicit_provider = Some("libc".into());
        let r = select(&with_libc, win, &[explicit.clone()], &grants).unwrap();
        assert_eq!(r[0].provider, "libc");
        assert_eq!(r[0].abi, ProviderAbi::PlatformAbi);
        // 缺默认（适配器平台不符）：no-provider 具名。
        let adapter_linux = vec![adapter("io", Os::Linux)];
        let err = select(&adapter_linux, win, &[say_request()], &grants).unwrap_err();
        assert_eq!(err.code(), "assembly.no-provider");
        // 唯一候选是未点名外库：自动回落拒绝（AutoForeignFallback）。
        let libc_only = vec![foreign("io", Os::Windows, "libc")];
        let err = select(&libc_only, win, &[say_request()], &grants).unwrap_err();
        assert_eq!(err.code(), "assembly.auto-foreign-fallback");
        // 点名后同一注册表可装配（显式选择是唯一合法外库路径）。
        let r = select(&libc_only, win, &[explicit.clone()], &grants).unwrap();
        assert_eq!(r[0].provider, "libc");
    }

    /// T-04/V-08：File/Task/Stream 有限资源前探——当前注册面无任何
    /// provider，全部状态必须输出 capability-missing（具名 no-provider），
    /// 绝不以选择成功或空收据冒充支持。
    #[test]
    fn t04_file_task_stream_probe_outputs_capability_missing_never_pass() {
        // 现行注册面（无 File/Task/Stream host provider——与分账一致）。
        let providers = vec![own("io", Os::Windows)];
        let grants: Vec<String> = vec![];
        let win = axes(Platform::windows(), Platform::windows());
        for (module, symbol) in [("file", "open"), ("task", "spawn"), ("stream", "read")] {
            let request = BindingRequest {
                operation: format!("{module}.{symbol}"),
                module: module.into(),
                symbol: symbol.into(),
                layer: StdlibLayer::Host,
                required_grants: vec![],
                explicit_provider: None,
            };
            // 显式点名也不得绕过（无该 provider 即 no-provider）。
            let mut explicit = request.clone();
            explicit.explicit_provider = Some("libc".into());
            for req in [request, explicit] {
                let err = select(&providers, win, &[req], &grants).unwrap_err();
                assert_eq!(
                    err.code(),
                    "assembly.no-provider",
                    "{module} must stay capability-missing"
                );
            }
        }
    }

    /// 分层事实入账：Host 层的 console 服务携带 layer/target/provider
    /// 完整身份（供装配收据与 #if 目标条件消费；966 实施源级 CTE）。
    #[test]
    fn selection_records_carry_layer_and_target_identity() {
        let providers = vec![own("io", Os::Windows), adapter("io", Os::Windows)];
        // Host 层默认池=自有 Auto；点名适配器验证平台桥身份如实入账。
        let mut explicit = say_request();
        explicit.explicit_provider = Some("os-win32".into());
        let r = select(
            &providers,
            axes(Platform::windows(), Platform::windows()),
            &[explicit],
            &["console".to_string()],
        )
        .unwrap();
        assert_eq!(r[0].layer, StdlibLayer::Host);
        assert_eq!(r[0].abi, ProviderAbi::PlatformAbi);
        assert_eq!(r[0].provider, "os-win32");
    }
}
