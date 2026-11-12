//! PLAN-935（T-05）：实例层宿主——单飞生产、集合候选与原子切换、实例页
//! 持久化（ac935 域）。
//!
//! 职责（计划 §5.2/§5.3/§5.5，沿 934 `QueryHost` 的 prepare→target→
//! revalidate→switch 纪律，但键/页/域是**实例域**）：
//! * **单飞**：同一完整实例键的一次生产生命周期 pending→completed；两个
//!   调用点请求同实例共享同一次生产（attempt 计量=1）；
//! * **集合候选**：`prepare_set` 把"模板源 → 需求实例集 + 脱糖程序"打包
//!   为一个候选（绑定宿主 revision + 模板签名/体版本依赖）；
//! * **原子切换**：`commit` 重验（取消/late/key 漂移/依赖漂移）后在一步
//!   内换入新集合——失败保旧集合完整（不存在半发布状态）；
//! * **A→B→A**：历史键可复用字节，但必须重新准入当前签名/来源/依赖
//!   （`produce` 恒经验证后回放，不以 hash 相等直接信任）；
//! * **持久化**：实例页 = [`InstancePage`]（producer/schema/algo 信封 +
//!   完整键），经 [`InstanceSink`] 落 ManagedStore 层 `S935`（项目软根，
//!   生命周期归 store 的 GC/lease，不另起第二套）。

use crate::ident::sq_hash;
use crate::instances::{
    instantiate_program_limited, InstanceRecord, InstantiateOutcome, INSTANCES_ALGO,
    INSTANCES_PRODUCER, INSTANCES_SCHEMA,
};
use crate::produce::ProduceLimits;
use ace_artifacts::codec::ChunkKind;
use ace_artifacts::identity::QueryKey as StoreQueryKey;
use ace_artifacts::store::{ManagedStore, PutRequest};
use ace_core::source::ast::Program;

/// 实例层在统一 store 的层 id（独立于 `S934`/`S796`）。
pub const INSTANCE_LAYER: &str = "S935";
/// 实例页在项目下的软根 kind。
pub const INSTANCE_ROOT_KIND: &str = "source-instances";
/// 实例页载体（不可变记录页，沿 S934 的 SnapshotRecord 载体纪律）。
pub const INSTANCE_CARRIER: ChunkKind = ChunkKind::SnapshotRecord;
/// 实例页 magic + codec 版本（新版本必须 MISS，绝不重释旧字节）。
///
/// PLAN-963：v2 = `argc + 每实参 (name, ty)`（通用多类型参数）。v1 页按
/// 既有「版本不符即 MISS」语义自然失效重产；magic 与 producer/schema/algo
/// 信封不变。
pub const INSTANCE_PAGE_MAGIC: &[u8; 6] = b"IN935P";
pub const INSTANCE_PAGE_CODEC_VERSION: u8 = 2;
/// 单页字节上限（沿 512KiB 页锚）。
pub const INSTANCE_MAX_PAGE_BYTES: usize = 512 * 1024;

/// 一个实例的持久页（ac935 域；与进程内 [`InstanceRecord`] 同构 + 信封）。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct InstancePage {
    pub producer: &'static str,
    pub schema: &'static str,
    pub algo: &'static str,
    pub key: String,
    pub logical: String,
    pub template: String,
    /// 声明序 `(类型参数名, 类型码)`（0=I32, 1=Bool, 2=Unit；单参即 935 面）。
    pub args: Vec<(String, u8)>,
    pub concrete_name: String,
    pub sig_version: String,
    pub body_version: String,
    pub demands: u64,
}

impl From<&InstanceRecord> for InstancePage {
    fn from(r: &InstanceRecord) -> Self {
        InstancePage {
            producer: INSTANCES_PRODUCER,
            schema: INSTANCES_SCHEMA,
            algo: INSTANCES_ALGO,
            key: r.key.clone(),
            logical: r.logical.clone(),
            template: r.template.clone(),
            args: r
                .args
                .iter()
                .map(|a| (a.param.clone(), ty_code(a.ty)))
                .collect(),
            concrete_name: r.concrete_name.clone(),
            sig_version: r.sig_version.clone(),
            body_version: r.body_version.clone(),
            demands: r.demands as u64,
        }
    }
}

/// substitution 类型码（页编码面；与 [`crate::facts::PortableTy`] 一一对应）。
fn ty_code(ty: crate::facts::PortableTy) -> u8 {
    match ty {
        crate::facts::PortableTy::I32 => 0,
        crate::facts::PortableTy::Bool => 1,
        crate::facts::PortableTy::Unit => 2,
    }
}

/// 实例页信封/内容错误。
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum InstancePageError {
    TooLarge {
        bytes: usize,
    },
    BadMagic,
    BadVersion {
        found: u8,
    },
    BadEnvelope {
        producer: String,
        schema: String,
        algo: String,
    },
    Truncated,
}

impl std::fmt::Display for InstancePageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstancePageError::TooLarge { bytes } => {
                write!(f, "instance page exceeds the 512KiB anchor ({bytes}B)")
            }
            InstancePageError::BadMagic => write!(f, "not an instance page (bad magic)"),
            InstancePageError::BadVersion { found } => {
                write!(f, "instance page codec version {found} is not this reader's version")
            }
            InstancePageError::BadEnvelope { producer, schema, algo } => write!(
                f,
                "instance page envelope mismatch (producer={producer}, schema={schema}, algo={algo})"
            ),
            InstancePageError::Truncated => write!(f, "instance page truncated"),
        }
    }
}

fn put_str(out: &mut Vec<u8>, s: &str) {
    out.extend_from_slice(&(s.len() as u32).to_le_bytes());
    out.extend_from_slice(s.as_bytes());
}

fn get_str(b: &[u8], pos: &mut usize) -> Result<String, InstancePageError> {
    if *pos + 4 > b.len() {
        return Err(InstancePageError::Truncated);
    }
    let n = u32::from_le_bytes([b[*pos], b[*pos + 1], b[*pos + 2], b[*pos + 3]]) as usize;
    *pos += 4;
    if *pos + n > b.len() {
        return Err(InstancePageError::Truncated);
    }
    let s =
        String::from_utf8(b[*pos..*pos + n].to_vec()).map_err(|_| InstancePageError::Truncated)?;
    *pos += n;
    Ok(s)
}

/// 编码一个实例页（确定性字节：同页同字节）。
pub fn encode_instance_page(p: &InstancePage) -> Result<Vec<u8>, InstancePageError> {
    let mut out = Vec::with_capacity(256);
    out.extend_from_slice(INSTANCE_PAGE_MAGIC);
    out.push(INSTANCE_PAGE_CODEC_VERSION);
    put_str(&mut out, p.producer);
    put_str(&mut out, p.schema);
    put_str(&mut out, p.algo);
    put_str(&mut out, &p.key);
    put_str(&mut out, &p.logical);
    put_str(&mut out, &p.template);
    out.extend_from_slice(&(p.args.len() as u32).to_le_bytes());
    for (name, code) in &p.args {
        put_str(&mut out, name);
        out.push(*code);
    }
    put_str(&mut out, &p.concrete_name);
    put_str(&mut out, &p.sig_version);
    put_str(&mut out, &p.body_version);
    out.extend_from_slice(&p.demands.to_le_bytes());
    if out.len() > INSTANCE_MAX_PAGE_BYTES {
        return Err(InstancePageError::TooLarge { bytes: out.len() });
    }
    Ok(out)
}

/// 解码并做信封准入（producer/schema/algo 必须 == 本 crate 常量——他域
/// 字节不重释）。
pub fn decode_instance_page(b: &[u8]) -> Result<InstancePage, InstancePageError> {
    if b.len() < INSTANCE_PAGE_MAGIC.len() + 1 {
        return Err(InstancePageError::Truncated);
    }
    if &b[..6] != INSTANCE_PAGE_MAGIC {
        return Err(InstancePageError::BadMagic);
    }
    if b[6] != INSTANCE_PAGE_CODEC_VERSION {
        return Err(InstancePageError::BadVersion { found: b[6] });
    }
    let mut pos = 7usize;
    let producer = get_str(b, &mut pos)?;
    let schema = get_str(b, &mut pos)?;
    let algo = get_str(b, &mut pos)?;
    if producer != INSTANCES_PRODUCER || schema != INSTANCES_SCHEMA || algo != INSTANCES_ALGO {
        return Err(InstancePageError::BadEnvelope {
            producer,
            schema,
            algo,
        });
    }
    let key = get_str(b, &mut pos)?;
    let logical = get_str(b, &mut pos)?;
    let template = get_str(b, &mut pos)?;
    if pos + 4 > b.len() {
        return Err(InstancePageError::Truncated);
    }
    let argc = u32::from_le_bytes([b[pos], b[pos + 1], b[pos + 2], b[pos + 3]]) as usize;
    pos += 4;
    // 防御性上限：单页内实参个数不可能超过 64（声明序位数；越界即截断/损坏）。
    if argc > 64 {
        return Err(InstancePageError::Truncated);
    }
    let mut args = Vec::with_capacity(argc);
    for _ in 0..argc {
        let name = get_str(b, &mut pos)?;
        if pos >= b.len() {
            return Err(InstancePageError::Truncated);
        }
        let code = b[pos];
        pos += 1;
        args.push((name, code));
    }
    let concrete_name = get_str(b, &mut pos)?;
    let sig_version = get_str(b, &mut pos)?;
    let body_version = get_str(b, &mut pos)?;
    if pos + 8 > b.len() {
        return Err(InstancePageError::Truncated);
    }
    let demands = u64::from_le_bytes([
        b[pos],
        b[pos + 1],
        b[pos + 2],
        b[pos + 3],
        b[pos + 4],
        b[pos + 5],
        b[pos + 6],
        b[pos + 7],
    ]);
    Ok(InstancePage {
        producer: INSTANCES_PRODUCER,
        schema: INSTANCES_SCHEMA,
        algo: INSTANCES_ALGO,
        key,
        logical,
        template,
        args,
        concrete_name,
        sig_version,
        body_version,
        demands,
    })
}

/// 实例页 sink（内存实现=确定性验证；ManagedStore 实现=生产持久化）。
pub trait InstanceSink {
    fn put_page(&mut self, page: &InstancePage) -> Result<String, String>;
    fn get_page(&self, artifact: &str) -> Option<InstancePage>;
    fn get_by_key(&self, key: &str) -> Option<InstancePage>;
}

/// 内存 sink（键→页 + 工件 id→页）。
#[derive(Default)]
pub struct MemoryInstanceSink {
    by_key: std::collections::BTreeMap<String, (String, InstancePage)>,
}

impl InstanceSink for MemoryInstanceSink {
    fn put_page(&mut self, page: &InstancePage) -> Result<String, String> {
        let artifact = format!("inst:{}", page.key);
        self.by_key
            .insert(page.key.clone(), (artifact.clone(), page.clone()));
        Ok(artifact)
    }
    fn get_page(&self, artifact: &str) -> Option<InstancePage> {
        self.by_key
            .values()
            .find(|(a, _)| a == artifact)
            .map(|(_, p)| p.clone())
    }
    fn get_by_key(&self, key: &str) -> Option<InstancePage> {
        self.by_key.get(key).map(|(_, p)| p.clone())
    }
}

/// 统一 store 后端的实例页 sink（层 `S935`，项目软根；生命周期归 store）。
pub struct ManagedInstanceSink {
    store: ManagedStore,
    project: Option<String>,
}

impl ManagedInstanceSink {
    pub fn open(
        root: &std::path::Path,
        project: Option<&str>,
    ) -> Result<Self, ace_artifacts::store::StoreError> {
        Ok(ManagedInstanceSink {
            store: ManagedStore::open(root)?,
            project: project.map(|s| s.to_string()),
        })
    }
}

fn instance_store_key(key: &str) -> StoreQueryKey {
    StoreQueryKey {
        layer: INSTANCE_LAYER,
        components: vec![("instance-key".to_string(), key.to_string())],
    }
}

impl InstanceSink for ManagedInstanceSink {
    fn put_page(&mut self, page: &InstancePage) -> Result<String, String> {
        let payload = encode_instance_page(page).map_err(|e| e.to_string())?;
        let req = PutRequest {
            layer: INSTANCE_LAYER,
            key: instance_store_key(&page.key),
            kind: INSTANCE_CARRIER,
            payload,
            edges: Vec::new(),
            project: self.project.clone(),
        };
        let art = self.store.put_artifact(req).map_err(|e| e.to_string())?;
        if let Some(project) = &self.project {
            self.store
                .register_root(project, &art, INSTANCE_ROOT_KIND)
                .map_err(|e| e.to_string())?;
        }
        Ok(art)
    }
    fn get_page(&self, artifact: &str) -> Option<InstancePage> {
        self.store
            .get_artifact_by_id(artifact)
            .ok()
            .flatten()
            .and_then(|a| decode_instance_page(&a.payload).ok())
    }
    fn get_by_key(&self, key: &str) -> Option<InstancePage> {
        self.store
            .get_artifact(INSTANCE_LAYER, &instance_store_key(key))
            .ok()
            .flatten()
            .and_then(|a| decode_instance_page(&a.payload).ok())
    }
}

// ---------------------------------------------------------------------------
// 宿主：单飞生产 + 集合候选 + 原子切换
// ---------------------------------------------------------------------------

/// 宿主 revision（每次成功 commit +1；候选绑定其 prepare 时的基面）。
pub type InstanceRevision = u64;

/// 一次 `prepare_set` 产出的集合候选（未安装）。
#[derive(Clone, Debug)]
pub struct InstanceSetCandidate {
    /// 脱糖具体程序（事实路径的真实输入）。
    pub program: Program,
    pub instances: Vec<InstanceRecord>,
    /// 候选基面（prepare 时的宿主 revision）。
    pub base: InstanceRevision,
    /// 集合指纹（全部实例键的有序合成——漂移检测）。
    pub set_fp: String,
    pub target: Option<u64>,
}

/// 已安装的当前实例集合（旧集合在成功 commit 前一直完整存活）。
#[derive(Clone, Debug)]
pub struct CurrentInstanceSet {
    pub revision: InstanceRevision,
    pub set_fp: String,
    pub instances: Vec<InstanceRecord>,
    pub artifacts: Vec<String>,
    pub target: Option<u64>,
}

/// commit 拒因（任何变体都保持旧集合不动）。
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum InstanceCommitError {
    Cancelled,
    LateResult {
        current: InstanceRevision,
        candidate: InstanceRevision,
    },
    /// 当前源重推导出的集合与候选不同（模板签名/体/调用需求漂移）。
    SetChanged {
        expected: String,
        current: String,
    },
    Sink(String),
}

/// 实例宿主：进程内单飞 + 页 sink + 原子集合切换。
pub struct InstanceHost<S: InstanceSink> {
    revision: InstanceRevision,
    current: Option<CurrentInstanceSet>,
    sink: S,
    /// 完整键 → 页工件（单飞结果缓存；A→B→A 复用须再验证当前源）。
    produced: std::collections::BTreeMap<String, String>,
    attempts: u64,
    productions: u64,
}

impl<S: InstanceSink> InstanceHost<S> {
    pub fn new(sink: S) -> Self {
        InstanceHost {
            revision: 0,
            current: None,
            sink,
            produced: std::collections::BTreeMap::new(),
            attempts: 0,
            productions: 0,
        }
    }

    pub fn revision(&self) -> InstanceRevision {
        self.revision
    }
    pub fn current(&self) -> Option<&CurrentInstanceSet> {
        self.current.as_ref()
    }
    /// 真实生产次数（attempt 计量——同键复用不计数）。
    pub fn productions(&self) -> u64 {
        self.productions
    }
    /// 生产请求次数（含命中复用的请求）。
    pub fn attempts(&self) -> u64 {
        self.attempts
    }

    /// 单飞：请求一个实例键。首次（或页缺失）真生产并落页；同键后续
    /// 请求共享同一次生产（productions 不增）。**复用不是信任**：命中页
    /// 的信封/键已被 sink 读回验证（解码即验证）。
    pub fn produce(
        &mut self,
        page: &InstancePage,
    ) -> Result<(InstancePage, bool /*produced now*/), String> {
        self.attempts += 1;
        if let Some(art) = self.produced.get(&page.key) {
            if let Some(p) = self.sink.get_page(art) {
                if p == *page {
                    return Ok((p, false));
                }
            }
        }
        // 也查持久层（跨宿主/进程恢复后的 A→B→A 复用面）。
        if let Some(p) = self.sink.get_by_key(&page.key) {
            if p == *page {
                let art = self.sink.put_page(&p)?; // re-pin under this host
                self.produced.insert(page.key.clone(), art);
                return Ok((p, false));
            }
        }
        let art = self.sink.put_page(page)?;
        self.produced.insert(page.key.clone(), art.clone());
        self.productions += 1;
        Ok((page.clone(), true))
    }

    /// Phase 1 —— 对当前（模板）源 prepare 一个集合候选：真推导（求解器
    /// discovery），绑定基面 revision + 集合指纹；不安装。
    pub fn prepare_set(
        &mut self,
        prog: &Program,
        package: &str,
        module: &str,
        capabilities: &[String],
    ) -> Result<InstanceSetCandidate, Vec<ace_core::atom_text::Diagnostic>> {
        let out: InstantiateOutcome = instantiate_program_limited(
            prog,
            package,
            module,
            capabilities,
            &ProduceLimits::default(),
        )?;
        let set_fp = set_fingerprint(&out.instances);
        Ok(InstanceSetCandidate {
            program: out.program,
            instances: out.instances,
            base: self.revision,
            set_fp,
            target: None,
        })
    }

    /// Phase 2 —— 附后端 target 句柄。
    pub fn attach_target(candidate: &mut InstanceSetCandidate, target: u64) {
        candidate.target = Some(target);
    }

    /// Phase 3/4 —— 重验 + 一步切换。重验内容：取消；late（宿主 revision
    /// 已前进）；**当前源重推导**的集合指纹与候选一致（签名/体/需求漂移
    /// 即拒）。成功：全部实例页落 sink（soft-root），集合原子换入，旧集
    /// 合让位（其页工件由 store GC/lease 按 root 生命周期回收——宿主不
    /// 主动删历史页）。
    pub fn commit(
        &mut self,
        candidate: InstanceSetCandidate,
        current_prog: &Program,
        package: &str,
        module: &str,
        capabilities: &[String],
        cancel: Option<&std::sync::atomic::AtomicBool>,
    ) -> Result<&CurrentInstanceSet, InstanceCommitError> {
        use std::sync::atomic::Ordering;
        if cancel.is_some_and(|c| c.load(Ordering::Relaxed)) {
            return Err(InstanceCommitError::Cancelled);
        }
        if self.revision != candidate.base {
            return Err(InstanceCommitError::LateResult {
                current: self.revision,
                candidate: candidate.base,
            });
        }
        // 当前源重推导（真验证，不是 hash 对 hash）。
        let now = instantiate_program_limited(
            current_prog,
            package,
            module,
            capabilities,
            &ProduceLimits::default(),
        )
        .map_err(|ds| InstanceCommitError::SetChanged {
            expected: candidate.set_fp.clone(),
            current: format!("rejected: {:?}", ds.first().map(|d| d.code.clone())),
        })?;
        let now_fp = set_fingerprint(&now.instances);
        if now_fp != candidate.set_fp {
            return Err(InstanceCommitError::SetChanged {
                expected: candidate.set_fp,
                current: now_fp,
            });
        }
        // 落页 + 换入（一步）。
        let mut artifacts = Vec::new();
        for inst in &candidate.instances {
            let page = InstancePage::from(inst);
            let (_p, _fresh) = self.produce(&page).map_err(InstanceCommitError::Sink)?;
            let art = self
                .produced
                .get(&inst.key)
                .cloned()
                .ok_or_else(|| InstanceCommitError::Sink("page not registered".into()))?;
            artifacts.push(art);
        }
        self.revision += 1;
        self.current = Some(CurrentInstanceSet {
            revision: self.revision,
            set_fp: candidate.set_fp,
            instances: candidate.instances,
            artifacts,
            target: candidate.target,
        });
        Ok(self.current.as_ref().expect("just set"))
    }
}

/// 集合指纹：全部实例完整键的域分离合成（序确定）。
pub fn set_fingerprint(instances: &[InstanceRecord]) -> String {
    let parts: Vec<Vec<u8>> = instances
        .iter()
        .map(|i| i.key.as_bytes().to_vec())
        .collect();
    let refs: Vec<&[u8]> = parts.iter().map(|p| p.as_slice()).collect();
    sq_hash("ac935/instance-set/v1", &refs)
}
