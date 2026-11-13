//! Immutable evidence captured at assembly time. Local paths are deliberately
//! excluded from the serialized identity; they remain in LayerSelection.
use std::path::Path;

use super::model::{fnv1a64, AssemblyContext, AssemblyTarget, Environment, LayerSelection};
use super::provider_selection::{SelectionRecord, SELECTION_SCHEMA_VERSION};
use super::reference::ReferenceProof;
include!(concat!(env!("OUT_DIR"), "/assembly_build_inputs.rs"));

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SourceInput {
    pub module: String,
    pub source_id: String,
    pub role: String,
    pub content_hash: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProviderInput {
    pub source_id: String,
    pub content_hash: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Payload {
    schema_version: u32,
    target: AssemblyTarget,
    environment: Environment,
    platform: String,
    consumer: String,
    features: Vec<String>,
    sources: Vec<SourceInput>,
    providers: Vec<ProviderInput>,
    references: Vec<ReferenceProof>,
    /// PLAN-947：四轴 provider 选择收据（schema 5 增量节；空=旧 schema 4
    /// 线型逐字节不变）。只携带身份事实，永不携带授权/地址。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    selection: Vec<SelectionRecord>,
}

/// 不可变装配收据（schema 4）。双重身份（PLAN-738 rv3 §5.8.3/T-12）：
/// - `fingerprint`：**共同装配身份**——消费者中立投影（consumer 名与
///   `consumer_input` 角色源剔除）。同 fixture/同 target/同 features/同
///   来源闭包下，CLI actual、编译会话、生成收据三个消费者必须相等；
/// - `consumer_fingerprint`：**消费者收据身份**——全量 payload（含
///   consumer 名与消费者输入），供新鲜度门/收据对拍使用。
/// 旧（≤schema 3）单指纹语义 = 现.consumer_fingerprint。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssemblyManifest {
    #[serde(flatten)]
    payload: Payload,
    fingerprint: String,
    consumer_fingerprint: String,
}

impl AssemblyManifest {
    /// Read schema 4 with its original field ordering and FNV projections.
    /// This is not admission as an ACE producer or a current host grant.
    pub fn decode_v4(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > 4 * 1024 * 1024 {
            return Err("STDASSEMBLY.MANIFEST_SIZE".into());
        }
        let manifest: Self = serde_json::from_slice(bytes)
            .map_err(|e| format!("STDASSEMBLY.MANIFEST_CODEC: {e}"))?;
        // PLAN-947 兼容规则：schema 4 = 无选择节（旧线型）；schema 5 =
        // 携带四轴选择收据（selection 非空且版本匹配）。交叉形态拒绝。
        match (
            manifest.schema_version(),
            manifest.payload.selection.is_empty(),
        ) {
            (4, true) => {}
            (5, false) if !manifest.payload.selection.is_empty() => {}
            (4, _) | (5, _) => return Err("STDASSEMBLY.MANIFEST_SCHEMA".into()),
            _ => return Err("STDASSEMBLY.MANIFEST_SCHEMA".into()),
        }
        if manifest.fingerprint != assembly_identity(&manifest.payload)
            || manifest.consumer_fingerprint
                != format!(
                    "{:016x}",
                    fnv1a64(&serde_json::to_string(&manifest.payload).unwrap())
                )
        {
            return Err("STDASSEMBLY.MANIFEST_FINGERPRINT".into());
        }
        Ok(manifest)
    }
    pub fn freeze(
        context: AssemblyContext,
        consumer: &str,
        project_root: &Path,
        stdlib_root: &Path,
        selections: &[LayerSelection],
        references: Vec<ReferenceProof>,
    ) -> Self {
        Self::freeze_opt(
            context,
            consumer,
            project_root,
            stdlib_root,
            selections,
            references,
            Vec::new(),
            4,
        )
    }

    /// PLAN-947：携带四轴 provider 选择收据的装配冻结（schema 5）。选择
    /// 记录参与共同/消费者双指纹；恢复重授权由选择合同的 grant 前置
    /// 检查承担（收据本身不带授权）。
    pub fn freeze_with_selection(
        context: AssemblyContext,
        consumer: &str,
        project_root: &Path,
        stdlib_root: &Path,
        selections: &[LayerSelection],
        references: Vec<ReferenceProof>,
        selection: Vec<SelectionRecord>,
    ) -> Self {
        Self::freeze_opt(
            context,
            consumer,
            project_root,
            stdlib_root,
            selections,
            references,
            selection,
            5,
        )
    }

    fn freeze_opt(
        context: AssemblyContext,
        consumer: &str,
        project_root: &Path,
        stdlib_root: &Path,
        selections: &[LayerSelection],
        references: Vec<ReferenceProof>,
        selection: Vec<SelectionRecord>,
        schema_version: u32,
    ) -> Self {
        let mut sources = Vec::new();
        for selected in selections {
            let public = portable_source_id(
                Path::new(&selected.public_file),
                project_root,
                stdlib_root,
                &selected.module,
            );
            sources.push(SourceInput {
                module: selected.module.clone(),
                source_id: public,
                role: "public".into(),
                content_hash: selected.public_hash,
            });
            if let (Some(path), Some(hash)) = (&selected.context_file, selected.context_hash) {
                sources.push(SourceInput {
                    module: selected.module.clone(),
                    source_id: portable_source_id(
                        Path::new(path),
                        project_root,
                        stdlib_root,
                        &selected.module,
                    ),
                    role: "selected_target".into(),
                    content_hash: hash,
                });
            }
        }
        for reference in &references {
            if !sources.iter().any(|s| s.source_id == reference.declaration) {
                sources.push(SourceInput {
                    module: format!("auto.{}", reference.module),
                    source_id: reference.declaration.clone(),
                    role: "native_declaration".into(),
                    content_hash: reference.declaration_hash,
                });
            }
        }
        sources.sort_by(|a, b| {
            (&a.module, &a.role, &a.source_id).cmp(&(&b.module, &b.role, &b.source_id))
        });
        sources.dedup_by(|a, b| a.source_id == b.source_id && a.role == b.role);
        let mut references = references;
        references.sort_by(|a, b| {
            (&a.module, &a.symbol, a.native_id, &a.callee, &a.producer).cmp(&(
                &b.module,
                &b.symbol,
                b.native_id,
                &b.callee,
                &b.producer,
            ))
        });
        references.dedup_by(|a, b| {
            a.module == b.module
                && a.symbol == b.symbol
                && a.native_id == b.native_id
                && a.callee == b.callee
                && a.producer == b.producer
        });
        let mut selection = selection;
        selection.sort_by(|a, b| {
            (&a.operation, &a.module, &a.symbol).cmp(&(&b.operation, &b.module, &b.symbol))
        });
        selection.dedup_by(|a, b| a.operation == b.operation);
        let payload = Payload {
            schema_version,
            selection,
            target: context.target,
            environment: context.environment,
            platform: env!("AUTO_ASSEMBLY_TARGET").into(),
            consumer: consumer.into(),
            features: enabled_features(),
            sources,
            providers: provider_inputs(context.target),
            references,
        };
        let fingerprint = assembly_identity(&payload);
        let consumer_fingerprint = format!(
            "{:016x}",
            fnv1a64(&serde_json::to_string(&payload).unwrap())
        );
        Self {
            payload,
            fingerprint,
            consumer_fingerprint,
        }
    }

    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    pub fn consumer_fingerprint(&self) -> &str {
        &self.consumer_fingerprint
    }

    pub fn schema_version(&self) -> u32 {
        self.payload.schema_version
    }

    pub fn consumer(&self) -> &str {
        &self.payload.consumer
    }

    /// P738-R5-03 对拍 getter：可比装配面字段的值级拷贝（serde JSON 字符串
    /// ——稳定比较，避免暴露内部枚举）。
    pub fn target_json(&self) -> String {
        serde_json::to_string(&self.payload.target).unwrap()
    }

    pub fn environment_json(&self) -> String {
        serde_json::to_string(&self.payload.environment).unwrap()
    }

    pub fn features_json(&self) -> String {
        serde_json::to_string(&self.payload.features).unwrap()
    }

    pub fn providers_json(&self) -> String {
        serde_json::to_string(&self.payload.providers).unwrap()
    }
    pub fn sources(&self) -> &[SourceInput] {
        &self.payload.sources
    }
    /// PLAN-947：四轴选择收据（schema 4 = 空）。
    pub fn selection(&self) -> &[SelectionRecord] {
        &self.payload.selection
    }
    /// 选择节合同版本（无选择节=0；有=SELECTION_SCHEMA_VERSION）。
    pub fn selection_schema(&self) -> u32 {
        if self.payload.selection.is_empty() {
            0
        } else {
            SELECTION_SCHEMA_VERSION
        }
    }
    pub fn references(&self) -> &[ReferenceProof] {
        &self.payload.references
    }

    /// Consumer-owned inputs are frozen by value before artifact creation.
    /// The builder consumes the old snapshot, preserving immutable receipts.
    pub fn with_consumer_input(mut self, source_id: &str, text: &str) -> Self {
        self.payload.sources.push(SourceInput {
            module: self.payload.consumer.clone(),
            source_id: source_id.into(),
            role: "consumer_input".into(),
            content_hash: fnv1a64(text),
        });
        self.payload.sources.sort_by(|a, b| {
            (&a.module, &a.role, &a.source_id).cmp(&(&b.module, &b.role, &b.source_id))
        });
        self.consumer_fingerprint = format!(
            "{:016x}",
            fnv1a64(&serde_json::to_string(&self.payload).unwrap())
        );
        self
    }
}

/// 共同装配身份投影：剔除 consumer 名与 `consumer_input` 角色源后哈希。
/// 消费者元数据（名/业务输入）不得伪造装配差异，也不得掩盖真实差异。
fn assembly_identity(payload: &Payload) -> String {
    let mut neutral = payload.clone();
    neutral.consumer = String::new();
    neutral
        .sources
        .retain(|source| source.role != "consumer_input");
    format!(
        "{:016x}",
        fnv1a64(&serde_json::to_string(&neutral).unwrap())
    )
}

pub fn enabled_features() -> Vec<String> {
    env!("AUTO_ASSEMBLY_FEATURES")
        .split(',')
        .filter(|f| !f.is_empty())
        .map(str::to_string)
        .collect()
}

/// These are compiled producer inputs, not claims read from the provider catalog.
/// Public .at files enter only through the actual selection/reference closure.
pub fn provider_inputs(target: AssemblyTarget) -> Vec<ProviderInput> {
    let common = [
        (
            "stdlib/contracts/operations.toml",
            super::contracts::OPERATIONS,
        ),
        (
            "stdlib/contracts/compatibility.toml",
            super::contracts::COMPATIBILITY,
        ),
        (
            "stdlib/assembly-providers.json",
            include_str!("../../../../stdlib/assembly-providers.json"),
        ),
        ("Cargo.lock", include_str!("../../../../Cargo.lock")),
        (
            "crates/auto-macros/src/lib.rs",
            include_str!("../../../auto-macros/src/lib.rs"),
        ),
    ];
    let mut inputs: Vec<_> = common
        .into_iter()
        .map(|(name, text)| ProviderInput {
            source_id: name.into(),
            content_hash: fnv1a64(text),
        })
        .collect();
    let selected: &[(&str, &str)] = match target {
        AssemblyTarget::Vm => &[
            ("vm/native.rs", include_str!("../vm/native.rs")),
            (
                "vm/native_catalog.rs",
                include_str!("../vm/native_catalog.rs"),
            ),
            ("vm/io.rs", include_str!("../vm/io.rs")),
            ("vm/ffi/stdlib.rs", include_str!("../vm/ffi/stdlib.rs")),
            (
                "vm/ffi/http_stream.rs",
                include_str!("../vm/ffi/http_stream.rs"),
            ),
            (
                "vm/ffi/http_server.rs",
                include_str!("../vm/ffi/http_server.rs"),
            ),
            (
                "vm/ffi/async_http.rs",
                include_str!("../vm/ffi/async_http.rs"),
            ),
            (
                "vm/ffi/stdlib/crash_trace.rs",
                include_str!("../vm/ffi/stdlib/crash_trace.rs"),
            ),
            (
                "vm/ffi/stdlib/file_io.rs",
                include_str!("../vm/ffi/stdlib/file_io.rs"),
            ),
            (
                "vm/ffi/stdlib/http_client.rs",
                include_str!("../vm/ffi/stdlib/http_client.rs"),
            ),
            (
                "vm/ffi/stdlib/image.rs",
                include_str!("../vm/ffi/stdlib/image.rs"),
            ),
            (
                "vm/ffi/stdlib/zip_io.rs",
                include_str!("../vm/ffi/stdlib/zip_io.rs"),
            ),
            (
                "vm/ffi/stdlib/math.rs",
                include_str!("../vm/ffi/stdlib/math.rs"),
            ),
            (
                "vm/ffi/stdlib/sqlite.rs",
                include_str!("../vm/ffi/stdlib/sqlite.rs"),
            ),
            (
                "vm/ffi/stdlib/strings_json.rs",
                include_str!("../vm/ffi/stdlib/strings_json.rs"),
            ),
            (
                "vm/ffi/stdlib/sys_env.rs",
                include_str!("../vm/ffi/stdlib/sys_env.rs"),
            ),
            // PLAN-884 T-04：基础设施域文件入 Vm provider 输入（迁移源不得缺列）。
            (
                "vm/ffi/stdlib/request_root.rs",
                include_str!("../vm/ffi/stdlib/request_root.rs"),
            ),
            (
                "vm/ffi/stdlib/storage.rs",
                include_str!("../vm/ffi/stdlib/storage.rs"),
            ),
            (
                "vm/ffi/stdlib/exit_audit.rs",
                include_str!("../vm/ffi/stdlib/exit_audit.rs"),
            ),
            (
                "vm/ffi/stdlib/http_async.rs",
                include_str!("../vm/ffi/stdlib/http_async.rs"),
            ),
            (
                "vm/ffi/stdlib/rust_stdlib_dispatch.rs",
                include_str!("../vm/ffi/stdlib/rust_stdlib_dispatch.rs"),
            ),
            (
                "vm/ffi/http_server/server_metadata.rs",
                include_str!("../vm/ffi/http_server/server_metadata.rs"),
            ),
            (
                "vm/ffi/http_server/value_json.rs",
                include_str!("../vm/ffi/http_server/value_json.rs"),
            ),
            (
                "vm/native/natives_chrono.rs",
                include_str!("../vm/native/natives_chrono.rs"),
            ),
            (
                "vm/native/natives_collections.rs",
                include_str!("../vm/native/natives_collections.rs"),
            ),
            (
                "vm/native/natives_formats.rs",
                include_str!("../vm/native/natives_formats.rs"),
            ),
            (
                "vm/native/natives_fs.rs",
                include_str!("../vm/native/natives_fs.rs"),
            ),
            (
                "vm/native/natives_host.rs",
                include_str!("../vm/native/natives_host.rs"),
            ),
            (
                "vm/native/natives_int.rs",
                include_str!("../vm/native/natives_int.rs"),
            ),
            (
                "vm/native/natives_io.rs",
                include_str!("../vm/native/natives_io.rs"),
            ),
            (
                "vm/native/natives_rand.rs",
                include_str!("../vm/native/natives_rand.rs"),
            ),
            (
                "vm/native/natives_strings.rs",
                include_str!("../vm/native/natives_strings.rs"),
            ),
            (
                "vm/native/natives_sys.rs",
                include_str!("../vm/native/natives_sys.rs"),
            ),
            (
                "vm/native/natives_test.rs",
                include_str!("../vm/native/natives_test.rs"),
            ),
            (
                "vm/native/natives_web.rs",
                include_str!("../vm/native/natives_web.rs"),
            ),
            (
                "trans/rust/call_emit.rs",
                include_str!("../trans/rust/call_emit.rs"),
            ),
            // PLAN-884 T-05：call_inner 主题拆分文件入 Vm provider 输入。
            (
                "trans/rust/call_emit/call_bare_builtins.rs",
                include_str!("../trans/rust/call_emit/call_bare_builtins.rs"),
            ),
            (
                "trans/rust/call_emit/call_bina_methods.rs",
                include_str!("../trans/rust/call_emit/call_bina_methods.rs"),
            ),
            (
                "trans/rust/call_emit/call_dot_methods.rs",
                include_str!("../trans/rust/call_emit/call_dot_methods.rs"),
            ),
            (
                "trans/rust/call_emit/call_generic_fallback.rs",
                include_str!("../trans/rust/call_emit/call_generic_fallback.rs"),
            ),
            (
                "trans/rust/call_emit/call_macro_shapes.rs",
                include_str!("../trans/rust/call_emit/call_macro_shapes.rs"),
            ),
            (
                "trans/rust/call_emit/call_module_methods.rs",
                include_str!("../trans/rust/call_emit/call_module_methods.rs"),
            ),
            (
                "trans/rust/call_emit/call_tag_ctor.rs",
                include_str!("../trans/rust/call_emit/call_tag_ctor.rs"),
            ),
            (
                "trans/rust/call_emit/call_task.rs",
                include_str!("../trans/rust/call_emit/call_task.rs"),
            ),
            (
                "trans/rust/call_emit/call_value_ctors.rs",
                include_str!("../trans/rust/call_emit/call_value_ctors.rs"),
            ),
            // PLAN-884 T-06：主题算法模块入 Vm provider 输入。
            (
                "trans/rust/semantics.rs",
                include_str!("../trans/rust/semantics.rs"),
            ),
            (
                "trans/rust/semantics/walk.rs",
                include_str!("../trans/rust/semantics/walk.rs"),
            ),
            (
                "trans/rust/ast_scans.rs",
                include_str!("../trans/rust/ast_scans.rs"),
            ),
            (
                "trans/rust/merged_fixes.rs",
                include_str!("../trans/rust/merged_fixes.rs"),
            ),
            (
                "trans/rust/post_fixes.rs",
                include_str!("../trans/rust/post_fixes.rs"),
            ),
            (
                "trans/rust/decl_emit.rs",
                include_str!("../trans/rust/decl_emit.rs"),
            ),
            (
                "trans/rust/expr_emit.rs",
                include_str!("../trans/rust/expr_emit.rs"),
            ),
            (
                "trans/rust/dynamic_json.rs",
                include_str!("../trans/rust/dynamic_json.rs"),
            ),
            (
                "trans/rust/project_fixups.rs",
                include_str!("../trans/rust/project_fixups.rs"),
            ),
            (
                "trans/rust/stmt_emit.rs",
                include_str!("../trans/rust/stmt_emit.rs"),
            ),
            ("trans/rust/ty.rs", include_str!("../trans/rust/ty.rs")),
            // PLAN-1014 T-03（plan884 V-09 漏登记修复）：byte_lists.rs 与
            // res_obs.rs 落库时未入 Vm provider 指纹闭包——域目录新增源码
            // 必须同步登记，漏登=静默不入指纹（该红此前被 ACE 守卫红掩盖）。
            (
                "trans/rust/byte_lists.rs",
                include_str!("../trans/rust/byte_lists.rs"),
            ),
            (
                "trans/rust/res_obs.rs",
                include_str!("../trans/rust/res_obs.rs"),
            ),
        ],
        AssemblyTarget::Rust => &[
            ("a2r_std.rs", include_str!("../a2r_std.rs")),
            ("trans/rust.rs", include_str!("../trans/rust.rs")),
        ],
        AssemblyTarget::C => &[("trans/c.rs", include_str!("../trans/c.rs"))],
    };
    inputs.extend(selected.iter().map(|(name, text)| ProviderInput {
        source_id: format!("crates/auto-lang/src/{name}"),
        content_hash: fnv1a64(text),
    }));
    if target == AssemblyTarget::Rust {
        for (name, source) in [
            ("Cargo.toml", include_str!("../../../a2r-std/Cargo.toml")),
            ("src/lib.rs", include_str!("../../../a2r-std/src/lib.rs")),
            ("src/json.rs", include_str!("../../../a2r-std/src/json.rs")),
            ("src/str.rs", include_str!("../../../a2r-std/src/str.rs")),
            ("src/hash.rs", include_str!("../../../a2r-std/src/hash.rs")),
            ("src/fs.rs", include_str!("../../../a2r-std/src/fs.rs")),
            ("src/http.rs", include_str!("../../../a2r-std/src/http.rs")),
            ("src/sse.rs", include_str!("../../../a2r-std/src/sse.rs")),
        ] {
            inputs.push(ProviderInput {
                source_id: format!("crates/a2r-std/{name}"),
                content_hash: fnv1a64(source),
            });
        }
    }
    if matches!(target, AssemblyTarget::Vm | AssemblyTarget::Rust) {
        inputs.push(ProviderInput {
            source_id: "crates/a2r-std/src/zip.rs".to_string(),
            content_hash: fnv1a64(include_str!("../../../a2r-std/src/zip.rs")),
        });
    }
    inputs.sort_by(|a, b| a.source_id.cmp(&b.source_id));
    for &(source_id, content_hash) in BUILD_INPUTS {
        if !inputs.iter().any(|input| input.source_id == source_id) {
            inputs.push(ProviderInput {
                source_id: source_id.into(),
                content_hash,
            });
        }
    }
    inputs.sort_by(|a, b| a.source_id.cmp(&b.source_id));
    inputs
}

fn portable_source_id(path: &Path, project: &Path, stdlib: &Path, module: &str) -> String {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if let Ok(relative) = canonical.strip_prefix(
        stdlib
            .canonicalize()
            .unwrap_or_else(|_| stdlib.to_path_buf()),
    ) {
        return format!(
            "stdlib/auto/{}",
            relative.to_string_lossy().replace('\\', "/")
        );
    }
    if let Ok(relative) = canonical.strip_prefix(
        project
            .canonicalize()
            .unwrap_or_else(|_| project.to_path_buf()),
    ) {
        return format!("project/{}", relative.to_string_lossy().replace('\\', "/"));
    }
    format!(
        "dependency/{module}/{}",
        path.file_name().unwrap_or_default().to_string_lossy()
    )
}

// ---- PLAN-814 R3-03/R4：SDK build 输入 owner 闭包负控 ----
// 身份源=build.rs 具名清单（IDENTITY_SOURCE_CRATES=SDK 编译闭包内 workspace
// path 依赖）；UI/Man/旧包/生成器源码不得进入 SDK assembly 身份，直接
// path 依赖不得缺席（防清单与 Cargo.toml 漂移——新增依赖时该断言强制
// 同步 build.rs 清单）。
#[cfg(test)]
mod plan814_build_input_boundary {
    use super::BUILD_INPUTS;

    /// 禁入：不进 SDK rlib 的 crate 源码（UI/Man 实现、旧兼容包、生成器）。
    #[test]
    fn ui_man_sources_never_enter_sdk_identity() {
        let forbidden = [
            "crates/auto-lang/",
            "crates/auto-man/",
            "crates/auto-ui-compiler/",
            "crates/auto-ui-runtime/",
            "crates/auto-ui-dialect/",
            "crates/a2r-actor-tests/",
            "dependency/auto-down/",
        ];
        let offenders: Vec<&str> = BUILD_INPUTS
            .iter()
            .filter(|(id, _)| forbidden.iter().any(|p| id.starts_with(p)))
            .map(|(id, _)| *id)
            .collect();
        assert!(
            offenders.is_empty(),
            "SDK assembly identity must not include non-SDK sources: {offenders:?}"
        );
    }

    /// 覆盖：SDK Cargo.toml 声明的每个 workspace path 依赖（normal/build）
    /// 都在身份清单内（经 build.rs 的 AUTO_IDENTITY_SOURCE_CRATES 注入；
    /// 传递闭包由清单维护时的同源计算保证，本断言钉直接面）。
    #[test]
    fn declared_path_deps_all_in_identity_sources() {
        let identity: Vec<&str> = env!("AUTO_IDENTITY_SOURCE_CRATES").split(',').collect();
        let manifest = include_str!("../../Cargo.toml");
        let mut in_deps = false;
        let mut missing = Vec::new();
        for line in manifest.lines() {
            let s = line.trim();
            if s.starts_with('[') {
                in_deps = s == "[dependencies]" || s == "[build-dependencies]";
                continue;
            }
            if !in_deps {
                continue;
            }
            if let Some((name, rest)) = s.split_once('=') {
                if rest.contains("path") {
                    let name = name.trim();
                    if !identity.contains(&name) {
                        missing.push(name.to_string());
                    }
                }
            }
        }
        assert!(
            missing.is_empty(),
            "path deps missing from IDENTITY_SOURCE_CRATES (update build.rs): {missing:?}"
        );
    }
    #[test]
    fn assembly_contract_source_is_in_sdk_identity_without_backend_sources() {
        assert!(BUILD_INPUTS
            .iter()
            .any(|(id, _)| *id == "experimental/ace/crates/semantics-contract/src/assembly.rs"));
        let offenders: Vec<_> = BUILD_INPUTS
            .iter()
            .filter(|(id, _)| {
                id.starts_with("experimental/ace/")
                    && !id.starts_with("experimental/ace/crates/semantics-contract/")
            })
            .collect();
        assert!(
            offenders.is_empty(),
            "SDK identity contains non-contract ACE sources: {offenders:?}"
        );
    }
}

/// PLAN-884 V-09：迁移源不得静默缺列——stdlib 域目录（及 http_server/
/// native/ trans/rust 主题目录）中的每个 .rs 文件都必须出现在 Vm 目标的
/// provider 输入闭包里；新增域文件而漏登记会在此红，而不是静默不入指纹。
#[cfg(test)]
mod plan884_provider_input_coverage {
    use super::provider_inputs;
    use crate::stdlib_assembly::manifest::AssemblyTarget;

    #[test]
    fn domain_dir_sources_all_covered_by_vm_provider_inputs() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let covered: std::collections::HashSet<String> = provider_inputs(AssemblyTarget::Vm)
            .into_iter()
            .map(|p| p.source_id)
            .map(|id| {
                id.strip_prefix("crates/auto-lang/src/")
                    .unwrap_or(&id)
                    .to_string()
            })
            .collect();
        let mut missing = Vec::new();
        for dir in [
            "vm/ffi/stdlib",
            "vm/ffi/http_server",
            "vm/native",
            "trans/rust",
        ] {
            let path = std::path::Path::new(manifest_dir).join("src").join(dir);
            let mut stack = vec![(path, dir.to_string())];
            while let Some((p, rel_dir)) = stack.pop() {
                for entry in std::fs::read_dir(&p).expect("read domain dir") {
                    let entry = entry.expect("entry");
                    let ep = entry.path();
                    if ep.is_dir() {
                        let sub = ep.file_name().unwrap().to_string_lossy().to_string();
                        stack.push((ep, format!("{rel_dir}/{sub}")));
                        continue;
                    }
                    if ep.extension().and_then(|e| e.to_str()) == Some("rs") {
                        let rel =
                            format!("{}/{}", rel_dir, ep.file_name().unwrap().to_string_lossy());
                        if !covered.contains(&rel) {
                            missing.push(rel);
                        }
                    }
                }
            }
        }
        assert!(
            missing.is_empty(),
            "PLAN-884 V-09: 域目录源码未入 Vm provider 指纹闭包（漏登记=静默不入指纹）: {missing:?}"
        );
    }
}

#[cfg(test)]
mod plan738_manifest {
    use super::*;
    #[test]
    fn schema4_read_preserves_original_common_and_consumer_projections() {
        // Pre-extension wire shape and independently fixed FNV answers. A new
        // field/order/hash algorithm must not silently change this reader.
        let legacy = br#"{"schema_version":4,"target":"vm","environment":"native","platform":"test-x64","consumer":"legacy","features":[],"sources":[],"providers":[],"references":[],"fingerprint":"133f7fdee4742800","consumer_fingerprint":"93ff2a3f7992bab7"}"#;
        let golden = AssemblyManifest::decode_v4(legacy).unwrap();
        assert_eq!(serde_json::to_vec(&golden).unwrap(), legacy);
        let manifest = AssemblyManifest::freeze(
            Default::default(),
            "reader944",
            Path::new("."),
            Path::new("stdlib"),
            &[],
            vec![],
        );
        let bytes = serde_json::to_vec(&manifest).unwrap();
        let read = AssemblyManifest::decode_v4(&bytes).unwrap();
        assert_eq!(serde_json::to_vec(&read).unwrap(), bytes);
        let business = manifest
            .clone()
            .with_consumer_input("project/main.at", "fn main() {}");
        assert_eq!(manifest.fingerprint(), business.fingerprint());
        assert_ne!(
            manifest.consumer_fingerprint(),
            business.consumer_fingerprint()
        );
        assert!(AssemblyManifest::decode_v4(&serde_json::to_vec(&business).unwrap()).is_ok());
        for (key, value) in [
            ("schema_version", serde_json::json!(5)),
            ("fingerprint", serde_json::json!("forged")),
            ("consumer", serde_json::json!("other")),
        ] {
            let mut value0 = serde_json::to_value(&manifest).unwrap();
            value0[key] = value;
            assert!(
                AssemblyManifest::decode_v4(&serde_json::to_vec(&value0).unwrap()).is_err(),
                "{key}"
            );
        }
    }
    /// PLAN-947 T-10/V-10：schema 5 选择收据——合法冻结可读回、选择参与
    /// 双指纹（篡改拒）、schema 4 旧线型逐字节兼容、交叉形态拒绝。
    #[test]
    fn plan947_selection_receipt_schema5_and_legacy_compat() {
        use super::super::provider_selection::{
            select, AssemblyAxes, Backend, BindingRequest, Os, Platform, ProviderAbi, ProviderDecl,
            ProviderOrigin, StdlibLayer,
        };
        let providers = vec![ProviderDecl {
            provider: "auto".into(),
            module: "io".into(),
            origin: ProviderOrigin::OwnAuto,
            abi: ProviderAbi::AutoAbi,
            platforms: vec![Platform { os: Os::Windows }],
        }];
        let request = BindingRequest {
            operation: "console.say".into(),
            module: "io".into(),
            symbol: "say".into(),
            layer: StdlibLayer::Host,
            required_grants: vec!["console".into()],
            explicit_provider: None,
        };
        let axes = AssemblyAxes {
            backend: Backend::Av,
            host: Platform { os: Os::Windows },
            target: Platform { os: Os::Windows },
        };
        let selection = select(&providers, axes, &[request], &["console".into()]).unwrap();
        let manifest = AssemblyManifest::freeze_with_selection(
            Default::default(),
            "plan947",
            Path::new("."),
            Path::new("stdlib"),
            &[],
            vec![],
            selection,
        );
        assert_eq!(manifest.schema_version(), 5);
        assert_eq!(manifest.selection_schema(), 1);
        assert_eq!(manifest.selection().len(), 1);
        assert_eq!(manifest.selection()[0].provider, "auto");
        let bytes = serde_json::to_vec(&manifest).unwrap();
        let read = AssemblyManifest::decode_v4(&bytes).unwrap();
        assert_eq!(serde_json::to_vec(&read).unwrap(), bytes);
        // 篡改选择记录破坏双指纹（防伪造/漂移）。
        let mut forged: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        forged["selection"][0]["provider"] = serde_json::json!("libc");
        assert!(AssemblyManifest::decode_v4(&serde_json::to_vec(&forged).unwrap()).is_err());
        // schema 4 旧线型（无选择节）逐字节兼容且 selection 为空。
        let legacy = AssemblyManifest::freeze(
            Default::default(),
            "legacy947",
            Path::new("."),
            Path::new("stdlib"),
            &[],
            vec![],
        );
        assert_eq!(legacy.schema_version(), 4);
        assert!(legacy.selection().is_empty());
        let legacy_bytes = serde_json::to_vec(&legacy).unwrap();
        let legacy_read = AssemblyManifest::decode_v4(&legacy_bytes).unwrap();
        assert_eq!(serde_json::to_vec(&legacy_read).unwrap(), legacy_bytes);
        // 交叉形态拒绝：schema 4 携带选择节 / schema 5 无选择节。
        let mut v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        v["schema_version"] = serde_json::json!(4);
        assert!(AssemblyManifest::decode_v4(&serde_json::to_vec(&v).unwrap()).is_err());
        let mut no_selection: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        no_selection["selection"] = serde_json::json!([]);
        // 空选择节 + schema 5：指纹不再匹配，同样拒绝。
        assert!(AssemblyManifest::decode_v4(&serde_json::to_vec(&no_selection).unwrap()).is_err());
    }

    #[test]
    fn captured_sources_are_immutable_and_portable() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("m.at");
        std::fs::write(&source, "pub fn value() int { return 1 }").unwrap();
        let mut session = crate::compile::CompileSession::new();
        session.add_source_dir(root.path().to_path_buf());
        session.resolve_uses("use m").unwrap();
        let freeze = || {
            AssemblyManifest::freeze(
                Default::default(),
                "compiler",
                root.path(),
                &root.path().join("stdlib"),
                &session.layer_selections,
                Vec::new(),
            )
        };
        let before = freeze();
        std::fs::write(&source, "pub fn value() int { return 2 }").unwrap();
        assert_eq!(before.fingerprint(), freeze().fingerprint());
        let json = serde_json::to_string(&before).unwrap();
        assert!(!json.contains(&root.path().to_string_lossy().to_string()));
        assert!(json.contains("project/m.at"));
        assert_eq!(
            before.sources()[0].content_hash,
            fnv1a64("pub fn value() int { return 1 }")
        );
    }
}
