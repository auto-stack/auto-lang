//! PLAN-1043（T-02）：平铺 provider 实现来源——文件后缀分类、候选清单与
//! backend×provider×target 三轴选择（2026-11-13 用户确定：公共 `io.at` +
//! `io.at.at`/`io.c.at`/`io.rs.at`/`io.py.at` 四候选，最后 `.at` 是源码扩展、
//! 前一段 at/c/rs/py 是实现来源；平台不扩后缀轴）。
//!
//! 本模块是纯数据模型 + 纯函数：不搜盘（宿主提供候选文件名闭包）、不
//! 解析、不绑定。缺/不相容/无默认一律具名拒，绝不静默回落其他 provider。
//! 旧 `.vm.at` 目标层选择仍归 legacy `auto_lang_core::stdlib_assembly`
//! 旧配方（PLAN-738/947），与本合同互不混用；退役归 986/987。
use std::collections::BTreeMap;
use std::collections::BTreeSet;

/// 选择/装配配方身份（进入绑定 identity 与 store 根）。
pub const PROVIDER_SOURCE_SCHEMA: &str = "ac.source.assembly.provider-sources/v1";

/// 实现来源（文件后缀的第二段）。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ProviderKind {
    /// 纯 Auto 实现（`.at.at`）——可移植单源，任何后端可消费。
    At,
    /// C 适配 Auto 源（`.c.at`）——AV/C 配置与原 A2C 共用。
    C,
    /// Rust 适配 Auto 源（`.rs.at`）——AV/Rust 配置与 A2R 共用。
    Rs,
    /// Python 适配 Auto 源（`.py.at`）——A2PY 消费。
    Py,
}

impl ProviderKind {
    pub fn suffix(self) -> &'static str {
        match self {
            ProviderKind::At => ".at.at",
            ProviderKind::C => ".c.at",
            ProviderKind::Rs => ".rs.at",
            ProviderKind::Py => ".py.at",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            ProviderKind::At => "at",
            ProviderKind::C => "c",
            ProviderKind::Rs => "rs",
            ProviderKind::Py => "py",
        }
    }
}

/// 执行/生成后端轴（与 provider 轴正交；后端决定消费方式，provider 决定
/// 实现来源文件）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AssemblyBackend {
    Av,
    /// 原生机器码（共享 HIR/MIR→COFF/PE）。
    Ac,
    /// Checked HIR/A2X→Rust 工程。
    A2R,
    /// Checked HIR/A2X→Python 工程（PLAN-1042 producer）。
    A2Py,
    /// 原 C 生成路径（a2x-c）。
    A2C,
}

impl AssemblyBackend {
    pub fn label(self) -> &'static str {
        match self {
            AssemblyBackend::Av => "av",
            AssemblyBackend::Ac => "ac",
            AssemblyBackend::A2R => "a2r",
            AssemblyBackend::A2Py => "a2py",
            AssemblyBackend::A2C => "a2c",
        }
    }
    /// provider×backend 相容表（设计§8：AV/C 与 A2C 共用 C 适配源，AV/Rust
    /// 与 A2R 共用 Rust 适配源；Python 适配源只有 A2PY 消费；纯 Auto 单源
    /// 全后端可用）。不相容组合在装配前具名拒。
    pub fn accepts(self, kind: ProviderKind) -> bool {
        match kind {
            ProviderKind::At => true,
            ProviderKind::C => matches!(self, AssemblyBackend::Av | AssemblyBackend::A2C),
            ProviderKind::Rs => matches!(self, AssemblyBackend::Av | AssemblyBackend::A2R),
            ProviderKind::Py => matches!(self, AssemblyBackend::A2Py),
        }
    }
}

/// 目标平台轴（与 legacy 947 `Os` 同域；Mac 仅可选输入，不是产品验收平台）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TargetOs {
    Windows,
    Linux,
    Macos,
}

impl TargetOs {
    pub fn label(self) -> &'static str {
        match self {
            TargetOs::Windows => "windows",
            TargetOs::Linux => "linux",
            TargetOs::Macos => "macos",
        }
    }
    /// `sys` namespace 的 OS 判别常量值（T-01 冻结：Windows=1/Linux=2/Mac=3；
    /// 与公共常量拼写共同组成同一判别域，0 保留为"未定"）。
    pub fn discriminant(self) -> i32 {
        match self {
            TargetOs::Windows => 1,
            TargetOs::Linux => 2,
            TargetOs::Macos => 3,
        }
    }
    pub fn from_discriminant(v: i32) -> Option<TargetOs> {
        Some(match v {
            1 => TargetOs::Windows,
            2 => TargetOs::Linux,
            3 => TargetOs::Macos,
            _ => return None,
        })
    }
}

/// 文件名分类结果（最长保留后缀；`io.at.at` 永不出现在 Public 类）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProviderFileKind {
    /// 公共源（裸 `.at`）。
    Public,
    /// 实现来源候选。
    Provider(ProviderKind),
}

/// 单个文件名 → (类别, 模块基名)。非 `.at` 结尾或无法剥离 → None。
/// 顺序即最长后缀优先：`.at.at`/`.c.at`/`.rs.at`/`.py.at` 先于裸 `.at`。
/// 旧目标层 `.vm.at` 不属本合同（legacy 旧配方专用）→ None：新发现面
/// 既不把它当实现候选，也不把 `io.vm.at` 误列成公共模块 `io.vm`。
pub fn classify_provider_file_name(name: &str) -> Option<(ProviderFileKind, String)> {
    if name.ends_with(".vm.at") {
        return None;
    }
    for kind in [
        ProviderKind::At,
        ProviderKind::C,
        ProviderKind::Rs,
        ProviderKind::Py,
    ] {
        if let Some(base) = name.strip_suffix(kind.suffix()) {
            if base.is_empty() {
                return None;
            }
            return Some((ProviderFileKind::Provider(kind), base.to_string()));
        }
    }
    let base = name.strip_suffix(".at")?;
    if base.is_empty() {
        return None;
    }
    Some((ProviderFileKind::Public, base.to_string()))
}

/// 一个平铺候选目录的清单（宿主扫描提供；本层不读盘）。
#[derive(Clone, Debug, Default)]
pub struct ProviderCandidates {
    /// 模块基名 → 存在的实现来源 kind 集。
    pub modules: BTreeMap<String, BTreeSet<ProviderKind>>,
}

impl ProviderCandidates {
    /// 从一组文件名构建（忽略不可分类名字——非本合同的文件不是候选）。
    pub fn from_names<'a>(names: impl IntoIterator<Item = &'a str>) -> ProviderCandidates {
        let mut out = ProviderCandidates::default();
        for name in names {
            if let Some((ProviderFileKind::Provider(kind), base)) =
                classify_provider_file_name(name)
            {
                out.modules.entry(base).or_default().insert(kind);
            }
        }
        out
    }
    /// 候选存在性读集（进入装配身份：增删候选文件即失效，未选 body 内容
    /// 不入身份）。
    pub fn existence_set(&self) -> BTreeMap<String, BTreeSet<ProviderKind>> {
        self.modules.clone()
    }
}

/// 显式 provider 选择：`Auto`（默认序）或点名 kind。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProviderChoice {
    Auto,
    Kind(ProviderKind),
}

impl ProviderChoice {
    pub fn label(self) -> String {
        match self {
            ProviderChoice::Auto => "auto".into(),
            ProviderChoice::Kind(k) => k.label().into(),
        }
    }
}

/// PLAN-1043（T-04）：一次装配的显式上下文——backend/provider 选择、
/// 目标平台与候选存在性读集。全部进入绑定 identity 与 store 根（恢复
/// 重验）；`available` 只记存在性（增删候选即失效），未选 body 内容不入
/// 身份（仅改未选文件不重编本目标）。
#[derive(Clone, Debug)]
pub struct PackageAssemblyContext {
    pub backend: AssemblyBackend,
    pub provider: ProviderChoice,
    pub target: TargetOs,
    /// 模块基名 → 发现的候选来源 kind 集（宿主扫描提供）。
    pub available: BTreeMap<String, BTreeSet<ProviderKind>>,
}

impl PackageAssemblyContext {
    pub fn new(
        backend: AssemblyBackend,
        provider: ProviderChoice,
        target: TargetOs,
    ) -> PackageAssemblyContext {
        PackageAssemblyContext {
            backend,
            provider,
            target,
            available: BTreeMap::new(),
        }
    }
    /// 候选存在性读集（进身份；目录形状变化→失效）。
    pub fn availability_id(&self) -> String {
        let mut parts = Vec::new();
        for (module, kinds) in &self.available {
            let mut ks: Vec<&str> = kinds.iter().map(|k| k.label()).collect();
            ks.sort_unstable();
            parts.push(format!("{module}:{}", ks.join("+")));
        }
        parts.join(";")
    }
    /// 装配身份串（backend/provider/target/配方/候选存在集）。
    pub fn identity(&self) -> String {
        format!(
            "{}|{}|{}|{}|{}",
            PROVIDER_SOURCE_SCHEMA,
            self.backend.label(),
            self.provider.label(),
            self.target.label(),
            self.availability_id()
        )
    }
    /// sys namespace 事实表（T-01 冻结拼写；`current`=目标、`host`=宿主
    /// 另名，Windows/Linux/Mac 为同域 i32 常量）。
    pub fn sys_facts(&self) -> BTreeMap<String, i32> {
        let mut facts = BTreeMap::new();
        facts.insert("current".into(), self.target.discriminant());
        facts.insert("host".into(), host_os().discriminant());
        facts.insert("Windows".into(), TargetOs::Windows.discriminant());
        facts.insert("Linux".into(), TargetOs::Linux.discriminant());
        facts.insert("Mac".into(), TargetOs::Macos.discriminant());
        facts
    }
}

/// 宿主 OS（编译进程运行所在的 OS；仅作 `sys.host` 事实，不参与选择）。
pub fn host_os() -> TargetOs {
    if cfg!(target_os = "windows") {
        TargetOs::Windows
    } else if cfg!(target_os = "macos") {
        TargetOs::Macos
    } else {
        TargetOs::Linux
    }
}

/// 选定的逻辑 source 集合（冻结产物：公共 + 至多一份所选实现）。
#[derive(Clone, Debug)]
pub struct ProviderSelection {
    pub backend: AssemblyBackend,
    pub provider: Option<ProviderKind>,
    pub target: TargetOs,
    pub recipe: &'static str,
}

/// 具名拒绝（调用前；绝不静默回落）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProviderSourceError {
    /// 点名的 provider 候选文件不存在（附现存 kind 标签）。
    ProviderMissing {
        module: String,
        kind: ProviderKind,
        present: Vec<&'static str>,
    },
    /// 默认序（Auto→at）无可选实现而其他来源存在（附现存 kind 标签）。
    MissingDefaultProvider {
        module: String,
        present: Vec<&'static str>,
    },
    /// 后端不消费该来源（如 A2R 选 c、AV 选 py）。
    ProviderBackend {
        module: String,
        kind: ProviderKind,
        backend: AssemblyBackend,
    },
}

impl ProviderSourceError {
    /// 稳定诊断码（进入绑定/测试断言）。
    pub fn code(&self) -> &'static str {
        match self {
            ProviderSourceError::ProviderMissing { .. } => "assembly.provider-missing",
            ProviderSourceError::MissingDefaultProvider { .. } => {
                "assembly.missing-default-provider"
            }
            ProviderSourceError::ProviderBackend { .. } => "assembly.provider-backend",
        }
    }
    pub fn message(&self) -> String {
        match self {
            ProviderSourceError::ProviderMissing {
                module,
                kind,
                present,
            } => format!(
                "module `{module}` has no `{}` provider candidate (present: {})",
                kind.label(),
                if present.is_empty() {
                    "none".into()
                } else {
                    present.join(",")
                }
            ),
            ProviderSourceError::MissingDefaultProvider { module, present } => format!(
                "module `{module}` needs an explicit provider: no `.at.at` portable body exists (present: {})",
                present.join(",")
            ),
            ProviderSourceError::ProviderBackend {
                module,
                kind,
                backend,
            } => format!(
                "backend `{}` does not consume the `{}` provider source (module `{module}`)",
                backend.label(),
                kind.label()
            ),
        }
    }
}

/// 三轴选择：候选（模块基名级）× backend × 显式选择 → 所选来源。
///
/// 规则（T-01 冻结）：
/// - 点名 kind：候选必须存在；backend 必须相容；否则具名拒。
/// - `Auto`：默认序 = 纯 Auto（at）。at 缺而其他来源存在 →
///   `MissingDefaultProvider`（点名才能用，绝不静默回落 c/rs/py）。
/// - 模块无任何实现候选 = 纯公共体单源（合法；必需实现缺口由绑定的
///   `pub spec fn` 装配检查负责）。
/// - target 只进选择身份（平台选择由 `#in` 表达，不扩后缀轴）。
pub fn select_provider_source(
    module: &str,
    candidates: &BTreeSet<ProviderKind>,
    backend: AssemblyBackend,
    choice: ProviderChoice,
    target: TargetOs,
) -> Result<ProviderSelection, ProviderSourceError> {
    let present_labels = || {
        let mut v: Vec<&'static str> = candidates.iter().map(|k| k.label()).collect();
        v.sort_unstable();
        v
    };
    let kind = match choice {
        ProviderChoice::Kind(k) => {
            if !candidates.contains(&k) {
                return Err(ProviderSourceError::ProviderMissing {
                    module: module.into(),
                    kind: k,
                    present: present_labels(),
                });
            }
            if !backend.accepts(k) {
                return Err(ProviderSourceError::ProviderBackend {
                    module: module.into(),
                    kind: k,
                    backend,
                });
            }
            Some(k)
        }
        ProviderChoice::Auto => {
            if candidates.contains(&ProviderKind::At) {
                if !backend.accepts(ProviderKind::At) {
                    return Err(ProviderSourceError::ProviderBackend {
                        module: module.into(),
                        kind: ProviderKind::At,
                        backend,
                    });
                }
                Some(ProviderKind::At)
            } else if candidates.is_empty() {
                None
            } else {
                return Err(ProviderSourceError::MissingDefaultProvider {
                    module: module.into(),
                    present: present_labels(),
                });
            }
        }
    };
    Ok(ProviderSelection {
        backend,
        provider: kind,
        target,
        recipe: PROVIDER_SOURCE_SCHEMA,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(kinds: &[ProviderKind]) -> BTreeSet<ProviderKind> {
        kinds.iter().copied().collect()
    }

    #[test]
    fn classification_keeps_longest_suffix_and_never_lists_at_at_as_public() {
        assert_eq!(
            classify_provider_file_name("io.at.at"),
            Some((ProviderFileKind::Provider(ProviderKind::At), "io".into()))
        );
        assert_eq!(
            classify_provider_file_name("io.c.at"),
            Some((ProviderFileKind::Provider(ProviderKind::C), "io".into()))
        );
        assert_eq!(
            classify_provider_file_name("io.rs.at"),
            Some((ProviderFileKind::Provider(ProviderKind::Rs), "io".into()))
        );
        assert_eq!(
            classify_provider_file_name("io.py.at"),
            Some((ProviderFileKind::Provider(ProviderKind::Py), "io".into()))
        );
        assert_eq!(
            classify_provider_file_name("io.at"),
            Some((ProviderFileKind::Public, "io".into()))
        );
        assert_eq!(classify_provider_file_name("io.vm.at"), None);
        assert_eq!(classify_provider_file_name("io.txt"), None);
        assert_eq!(classify_provider_file_name(".at"), None);
        // 嵌套路径/多段模块名按文件名分类（发现层负责目录语义）。
        assert_eq!(
            classify_provider_file_name("platform.linux.io.at.at"),
            Some((
                ProviderFileKind::Provider(ProviderKind::At),
                "platform.linux.io".into()
            ))
        );
    }

    #[test]
    fn auto_selects_portable_body_and_never_falls_back_to_foreign() {
        // at 存在 → 默认选 at
        let s = select_provider_source(
            "io",
            &set(&[ProviderKind::At, ProviderKind::C, ProviderKind::Rs]),
            AssemblyBackend::Av,
            ProviderChoice::Auto,
            TargetOs::Linux,
        )
        .unwrap();
        assert_eq!(s.provider, Some(ProviderKind::At));
        assert_eq!(s.target, TargetOs::Linux);
        // at 缺而 c/rs 在 → 具名拒（不静默回落）
        let e = select_provider_source(
            "io",
            &set(&[ProviderKind::C, ProviderKind::Rs]),
            AssemblyBackend::Av,
            ProviderChoice::Auto,
            TargetOs::Windows,
        )
        .unwrap_err();
        assert_eq!(e.code(), "assembly.missing-default-provider");
        assert!(e.message().contains("explicit provider"));
        // 无任何候选 = 纯公共单源
        let s = select_provider_source(
            "io",
            &set(&[]),
            AssemblyBackend::A2R,
            ProviderChoice::Auto,
            TargetOs::Windows,
        )
        .unwrap();
        assert_eq!(s.provider, None);
    }

    #[test]
    fn explicit_choice_checks_presence_and_backend_compatibility() {
        let e = select_provider_source(
            "io",
            &set(&[ProviderKind::C]),
            AssemblyBackend::Av,
            ProviderChoice::Kind(ProviderKind::Rs),
            TargetOs::Windows,
        )
        .unwrap_err();
        assert_eq!(e.code(), "assembly.provider-missing");
        // A2R 不消费 c 源；A2C 消费
        let e = select_provider_source(
            "io",
            &set(&[ProviderKind::C]),
            AssemblyBackend::A2R,
            ProviderChoice::Kind(ProviderKind::C),
            TargetOs::Linux,
        )
        .unwrap_err();
        assert_eq!(e.code(), "assembly.provider-backend");
        let s = select_provider_source(
            "io",
            &set(&[ProviderKind::C]),
            AssemblyBackend::A2C,
            ProviderChoice::Kind(ProviderKind::C),
            TargetOs::Linux,
        )
        .unwrap();
        assert_eq!(s.provider, Some(ProviderKind::C));
        // py 只有 A2PY 消费（1042 交付前装配层仍可表达选择）
        assert!(select_provider_source(
            "io",
            &set(&[ProviderKind::Py]),
            AssemblyBackend::Av,
            ProviderChoice::Kind(ProviderKind::Py),
            TargetOs::Windows
        )
        .is_err());
        assert!(select_provider_source(
            "io",
            &set(&[ProviderKind::Py]),
            AssemblyBackend::A2Py,
            ProviderChoice::Kind(ProviderKind::Py),
            TargetOs::Windows
        )
        .is_ok());
    }

    #[test]
    fn candidate_existence_set_records_directory_shape() {
        let c = ProviderCandidates::from_names([
            "io.at",
            "io.at.at",
            "io.c.at",
            "io.rs.at",
            "io.py.at",
            "math.at",
            "readme.md",
        ]);
        let set = c.existence_set();
        assert_eq!(set.len(), 1, "only io has provider candidates");
        assert!(set["io"].contains(&ProviderKind::Py));
        // 增删候选改变存在集（未选 body 内容不在其中）
        let mut names = vec![
            "io.at",
            "io.at.at",
            "io.c.at",
            "io.rs.at",
            "io.py.at",
            "math.at",
            "readme.md",
        ];
        names.retain(|n| *n != "io.py.at");
        assert_ne!(
            ProviderCandidates::from_names(names).existence_set(),
            set,
            "removing a candidate must change the existence read-set"
        );
    }

    #[test]
    fn os_discriminants_form_one_domain() {
        assert_eq!(TargetOs::Windows.discriminant(), 1);
        assert_eq!(TargetOs::Linux.discriminant(), 2);
        assert_eq!(TargetOs::Macos.discriminant(), 3);
        for os in [TargetOs::Windows, TargetOs::Linux, TargetOs::Macos] {
            assert_eq!(TargetOs::from_discriminant(os.discriminant()), Some(os));
        }
        assert_eq!(TargetOs::from_discriminant(0), None);
    }
}
