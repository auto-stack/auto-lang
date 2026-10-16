//! ac-syntax-artifact-adapter：PLAN-787 T-06 —— 真实语法产物接入 783 探针 store。
//!
//! - A01 `grammar-tables`：语法程序 canonical 二进制（GrammarId 绑定，装载时
//!   身份重验，不匹配 = 受控 miss）；
//! - A02 `tokens-checkpoints`：token 流 + **完整 scanner checkpoint**（mode/
//!   未闭合状态/行，F787-R1-05）+ 源页（内容寻址 SourcePage，F787-R1-06）；
//!   键含 grammar 身份——跨 grammar 真读拒绝；
//! - A03 `tolerant-cst`：内存内容块（`CstBlock`）即持久页（同构序列化），
//!   manifest 记目录/根边/页映射/源页表——重启后 `CstView` 按 manifest 小范围
//!   查询，**按需取页**，禁调 parser 且不反序列化整树；
//! - required-data 图（F787-R1-06）：manifest --required--> 全部数据页 + 全部
//!   声明目录页 + A02（A02 --required--> A01 + 源页）。GC 从当前根出发的
//!   required 闭包覆盖整棵可达树；
//! - 当前根/lease 生命周期：每工程仅一个 `cst-manifest` 硬根（发布时替换），
//!   历史版本靠 `CstView` 持有的 store lease 保活，视图释放后可被 GC 回收；
//! - 来源隔离：QueryKey 不含工程路径（内容身份）；工程归属经 roots/put 的
//!   project 字段——两工程同源同 grammar 共享字节块而诊断/根各自独立。

use ac_artifact_probe::identity::QueryKey;
use ac_artifact_probe::schema;
use ac_artifact_probe::store::{ManagedStore, PutRequest, StoreError};
use ac_syntax_rt::cst::{CstBlock, CstNode, CstTree, UNGRAFTED_DECL};
use ac_syntax_rt::lexer::{CompiledScanner, MidToken, ScanCheckpoint, Token};
use ac_syntax_rt::program::{Budget, GrammarProgram, RtError};
use ac_syntax_rt::snapshot::Snapshot;
use ac_syntax_rt::{PARSER_ABI_VERSION, RUNTIME_VERSION, SCANNER_ABI_VERSION};
use std::path::Path;
use std::rc::Rc;

/// 页缓存受管预算（V-11：hot ≤8MiB——超出按 LRU 淘汰，可再取）。
pub const CST_HOT_BUDGET: usize = 8 << 20;

/// store 源页聚合粒度（快照 16KiB 页的聚合上限：256KiB/源页——内容寻址
/// 共享单位，兼顾大负载的 put 次数）。
pub const SRC_STORE_PAGE_BYTES: usize = 256 * 1024;

pub struct SyntaxArtifactStore {
    store: ManagedStore,
    project: String,
    /// 本会话已发布的最高快照修订（F787-R3-02：修订比较的适用域=**会话**——
    /// 迟到任务=同一编辑会话内低修订后到；跨会话/重启的 revision=0 是新
    /// 文档发布，不是迟到任务。发布元数据不进内容工件（同文本跨修订必须
    /// 幂等复用）。
    published_revision: std::cell::Cell<Option<u32>>,
}

fn layer(id: &str) -> &'static schema::LayerSpec {
    schema::layer(id).expect("syntax layers registered")
}

fn grammar_key(p: &GrammarProgram) -> QueryKey {
    QueryKey::new(
        layer("A01"),
        vec![
            ("grammar-id".into(), p.grammar_id().as_str().to_string()),
            ("grammar-revision".into(), p.revision.to_string()),
            ("scanner-abi".into(), SCANNER_ABI_VERSION.to_string()),
        ],
    )
}

/// A02 token 产物键：**含 grammar 身份**（F787-R1-05）——同名不同内容语法包
/// 的 token 流键不同（跨 grammar 真读 = miss，绝不串用）。
fn tokens_key(snap: &Snapshot, p: &GrammarProgram) -> QueryKey {
    QueryKey::new(
        layer("A02"),
        vec![
            ("source-snapshot".into(), snap.id().as_str().to_string()),
            ("grammar-id".into(), p.grammar_id().as_str().to_string()),
            ("scanner-abi".into(), SCANNER_ABI_VERSION.to_string()),
            ("lexer-version".into(), RUNTIME_VERSION.to_string()),
        ],
    )
}

/// 内容寻址源页键（跨快照/跨工程按内容共享）。
fn source_page_key(hash: &str) -> QueryKey {
    QueryKey::new(layer("A02"), vec![("page-hash".into(), hash.to_string())])
}

#[derive(Debug)]
pub enum AdapterError {
    Store(StoreError),
    Rt(RtError),
    /// 身份/内容校验失败（grammar 不匹配/坏 manifest/坏 offset/ref）——受控 miss
    /// 或明确错误，绝不恢复带信任标志的树。
    Admissions(String),
}

impl From<StoreError> for AdapterError {
    fn from(e: StoreError) -> Self {
        AdapterError::Store(e)
    }
}
impl From<RtError> for AdapterError {
    fn from(e: RtError) -> Self {
        AdapterError::Rt(e)
    }
}
impl std::fmt::Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdapterError::Store(e) => write!(f, "store: {e}"),
            AdapterError::Rt(e) => write!(f, "runtime: {e}"),
            AdapterError::Admissions(m) => write!(f, "admissions: {m}"),
        }
    }
}

impl SyntaxArtifactStore {
    pub fn open(root: &Path, project: &str) -> Result<Self, AdapterError> {
        Ok(SyntaxArtifactStore {
            store: ManagedStore::open(root)?,
            project: project.to_string(),
            published_revision: std::cell::Cell::new(None),
        })
    }

    pub fn inner(&self) -> &ManagedStore {
        &self.store
    }

    // ---- A01 grammar-tables ----

    /// 发布语法程序（canonical 二进制；同 key 幂等）。
    pub fn put_grammar(&self, p: &GrammarProgram) -> Result<String, AdapterError> {
        let payload = p.canonical_bytes();
        let art = self.store.put_artifact(PutRequest {
            layer: "A01",
            key: grammar_key(p),
            kind: ac_artifact_probe::codec::ChunkKind::SyntaxGrammar,
            payload,
            edges: vec![],
            project: None,
        })?;
        Ok(art)
    }

    /// 装载语法程序：canonical → decode → **身份重验**（name+内容+版本 hash
    /// 与 key 的 grammar-id 不符 = 受控 miss/错误，不返回可执行程序）。
    pub fn get_grammar(
        &self,
        p_for_key: &GrammarProgram,
    ) -> Result<Option<GrammarProgram>, AdapterError> {
        let Some(loaded) = self.store.get_artifact("A01", &grammar_key(p_for_key))? else {
            return Ok(None);
        };
        let prog = ac_syntax_rt::program_codec::decode_canonical(&loaded.payload)
            .map_err(|e| AdapterError::Admissions(format!("grammar chunk undecodable: {e}")))?;
        if prog.grammar_id() != p_for_key.grammar_id() {
            return Err(AdapterError::Admissions(
                "grammar identity mismatch: chunk content hash != key grammar-id".into(),
            ));
        }
        Ok(Some(prog))
    }

    /// 按（grammar-id, revision）装载语法程序（跨进程恢复：调用方从交接文件
    /// 获得身份，不编译 .at 源——程序本体从 store 反序列化）。装载后身份重验。
    pub fn get_grammar_by_id(
        &self,
        grammar_id: &str,
        revision: u64,
    ) -> Result<Option<GrammarProgram>, AdapterError> {
        let key = QueryKey::new(
            layer("A01"),
            vec![
                ("grammar-id".into(), grammar_id.to_string()),
                ("grammar-revision".into(), revision.to_string()),
                ("scanner-abi".into(), SCANNER_ABI_VERSION.to_string()),
            ],
        );
        let Some(loaded) = self.store.get_artifact("A01", &key)? else {
            return Ok(None);
        };
        let prog = ac_syntax_rt::program_codec::decode_canonical(&loaded.payload)
            .map_err(|e| AdapterError::Admissions(format!("grammar chunk undecodable: {e}")))?;
        if prog.grammar_id().as_str() != grammar_id {
            return Err(AdapterError::Admissions(
                "grammar identity mismatch: chunk content hash != requested grammar-id".into(),
            ));
        }
        Ok(Some(prog))
    }

    // ---- 源页（F787-R1-06：源字节持久化，可从 store 重建原文）----

    /// 发布快照源字节（内容寻址，跨快照/工程共享）。快照页（16KiB）按
    /// `SRC_STORE_PAGE_BYTES` 聚合为 store 源页——大负载下控制 put 次数
    /// （probe 每次提交有全目录扫描成本），聚合粒度仍是内容寻址共享单位。
    /// 返回 (artifact ids, chunk ids)，均按文档序。
    pub fn put_source_pages(
        &self,
        snap: &Snapshot,
    ) -> Result<(Vec<String>, Vec<String>), AdapterError> {
        let mut arts = Vec::new();
        let mut chunks = Vec::new();
        let mut buf: Vec<u8> = Vec::with_capacity(SRC_STORE_PAGE_BYTES);
        let flush = |buf: &mut Vec<u8>,
                     arts: &mut Vec<String>,
                     chunks: &mut Vec<String>|
         -> Result<(), AdapterError> {
            if buf.is_empty() {
                return Ok(());
            }
            let hash = ac_artifact_probe::identity::domain_hash("src-page", &[&buf[..]]);
            let art = self.store.put_artifact(PutRequest {
                layer: "A02",
                key: source_page_key(&hash),
                kind: ac_artifact_probe::codec::ChunkKind::SourcePage,
                payload: buf.clone(),
                edges: vec![],
                project: None,
            })?;
            let chunk = ac_artifact_probe::codec::framed_chunk_id(
                ac_artifact_probe::codec::ChunkKind::SourcePage,
                buf,
            )
            .to_string();
            arts.push(art);
            chunks.push(chunk);
            buf.clear();
            Ok(())
        };
        for pref in snap.page_refs() {
            if buf.len() + pref.bytes().len() > SRC_STORE_PAGE_BYTES && !buf.is_empty() {
                flush(&mut buf, &mut arts, &mut chunks)?;
            }
            buf.extend_from_slice(pref.bytes());
        }
        flush(&mut buf, &mut arts, &mut chunks)?;
        Ok((arts, chunks))
    }

    // ---- A02 tokens-checkpoints ----

    /// 发布 token 流 + EOF 终态 checkpoint（完整 mode/未闭合状态/行）。
    /// edges：A02 --required--> A01（语法表）+ 全部源页（F787-R1-05/06）。
    pub fn put_tokens(
        &self,
        p: &GrammarProgram,
        snap: &Snapshot,
        tokens: &[Token],
    ) -> Result<String, AdapterError> {
        let a01 = self.put_grammar(p)?;
        let (src_arts, _src_chunks) = self.put_source_pages(snap)?;
        let cp = derive_final_checkpoint(p, snap, tokens);
        let mut payload = Vec::with_capacity(tokens.len() * 25 + 4 + 4 + 40);
        payload.extend_from_slice(&TOKENS_CODEC_VERSION.to_le_bytes());
        payload.extend_from_slice(&(tokens.len() as u32).to_le_bytes());
        for t in tokens {
            payload.extend_from_slice(&t.token.to_le_bytes());
            payload.extend_from_slice(&t.start.to_le_bytes());
            payload.extend_from_slice(&t.end.to_le_bytes());
            payload.extend_from_slice(&t.line.to_le_bytes());
            payload.push(t.error as u8);
        }
        encode_checkpoint(&mut payload, &cp);
        let mut edges = vec![(a01, ac_artifact_probe::identity::RefKind::RequiredData)];
        edges.extend(
            src_arts
                .into_iter()
                .map(|a| (a, ac_artifact_probe::identity::RefKind::RequiredData)),
        );
        let art = self.store.put_artifact(PutRequest {
            layer: "A02",
            key: tokens_key(snap, p),
            kind: ac_artifact_probe::codec::ChunkKind::TokenStream,
            payload,
            edges,
            project: None,
        })?;
        Ok(art)
    }

    pub fn get_tokens(
        &self,
        p: &GrammarProgram,
        snap: &Snapshot,
    ) -> Result<Option<Vec<Token>>, AdapterError> {
        let Some((tokens, _cp)) = self.get_tokens_with_checkpoint(p, snap)? else {
            return Ok(None);
        };
        Ok(Some(tokens))
    }

    /// token 流 + checkpoint 往返（F787-R1-05：完整 scanner 状态可再装载）。
    pub fn get_tokens_with_checkpoint(
        &self,
        p: &GrammarProgram,
        snap: &Snapshot,
    ) -> Result<Option<(Vec<Token>, ScanCheckpoint)>, AdapterError> {
        let Some(loaded) = self.store.get_artifact("A02", &tokens_key(snap, p))? else {
            return Ok(None);
        };
        let (tokens, cp) = decode_tokens_v2(&loaded.payload).map_err(AdapterError::Admissions)?;
        Ok(Some((tokens, cp)))
    }

    // ---- A03 tolerant-cst（分页 = 内存内容块同构） ----

    /// 序列化整树为页 chunk + manifest，返回 manifest artifact id。
    /// 页 = 内容块（块内边为局部下标，同 wire 格式）；manifest required
    /// A02 + 全部页（F787-R1-06：当前根的 required 闭包覆盖全树）。
    pub fn put_cst(
        &self,
        p: &GrammarProgram,
        snap: &Snapshot,
        tree: &CstTree,
        required_a02: &str,
    ) -> Result<String, AdapterError> {
        let (src_arts, src_chunks) = self.put_source_pages(snap)?;
        let mut page_arts: Vec<String> = Vec::with_capacity(tree.blocks.len());
        let mut pages: Vec<PageEntry> = Vec::with_capacity(tree.blocks.len());
        for (bi, blk) in tree.blocks.iter().enumerate() {
            let payload = encode_block(blk);
            let mut comps = cst_key_base(snap, p);
            comps.push(("page".into(), bi.to_string()));
            let key = QueryKey::new(layer("A03"), comps);
            let art = self.store.put_artifact(PutRequest {
                layer: "A03",
                key,
                kind: ac_artifact_probe::codec::ChunkKind::CstPage,
                payload: payload.clone(),
                edges: vec![],
                project: None,
            })?;
            let chunk = ac_artifact_probe::codec::framed_chunk_id(
                ac_artifact_probe::codec::ChunkKind::CstPage,
                &payload,
            )
            .to_string();
            page_arts.push(art);
            pages.push(PageEntry {
                chunk,
                first_node: tree.bases[bi],
                node_count: blk.nodes.len() as u32,
            });
        }
        let decls: Vec<DeclEntry> = tree
            .decls
            .iter()
            .map(|d| DeclEntry {
                origin: d.origin,
                node: d.node,
                node_lo: d.node_lo,
                page: tree.block_of(d.node),
            })
            .collect();
        let manifest = CstManifest {
            snapshot: snap.id().as_str().to_string(),
            grammar: p.grammar_id().as_str().to_string(),
            total_len: snap.len(),
            diags: tree
                .diags
                .iter()
                .map(|d| DiagEntry {
                    message: d.message.clone(),
                    start: d.start,
                    end: d.end,
                })
                .collect(),
            root_edges: tree.root_edges.clone(),
            pages,
            decl_index: Vec::new(),
            decl_count: decls.len() as u64,
            decls,
            source_pages: src_chunks,
        };
        let mut edges: Vec<(String, ac_artifact_probe::identity::RefKind)> = vec![(
            required_a02.to_string(),
            ac_artifact_probe::identity::RefKind::RequiredData,
        )];
        edges.extend(
            src_arts
                .into_iter()
                .map(|a| (a, ac_artifact_probe::identity::RefKind::RequiredData)),
        );
        edges.extend(
            page_arts
                .iter()
                .cloned()
                .map(|a| (a, ac_artifact_probe::identity::RefKind::RequiredData)),
        );
        let mut mcomps = cst_key_base(snap, p);
        mcomps.push(("page".into(), "manifest".into()));
        let mkey = QueryKey::new(layer("A03"), mcomps);
        let art = self.store.put_artifact(PutRequest {
            layer: "A03",
            key: mkey,
            kind: ac_artifact_probe::codec::ChunkKind::CstManifest,
            payload: serde_json::to_vec(&manifest)
                .map_err(|e| AdapterError::Admissions(e.to_string()))?,
            edges,
            // 版本工件存储不带工程软根（F787-R2-02：软根会永久钉住历史版本）；
            // 当前性由下方仲裁后的 cst-manifest 硬根表达。
            project: None,
        })?;
        self.publish_if_current(&art, snap.revision())?;
        Ok(art)
    }

    /// 当前发布仲裁（F787-R3-02/03）：分离"存版本工件"与"当前发布"。
    /// 迟到判定域=**本会话**（published_revision 仅内存、不进内容工件）：
    /// 同会话低修订后到 = 迟到（只入库不发布）；跨会话/重启无历史 → 可发布。
    /// 原子性顺序：**先注册新根，成功后定点删除被替换的旧根行**——注册失败
    /// （探针 trigger 注入等）时旧当前根完好保留；删除失败仅多余保活，无丢失。
    fn publish_if_current(&self, artifact: &str, revision: u32) -> Result<(), AdapterError> {
        if let Some(max) = self.published_revision.get() {
            if revision < max {
                return Ok(()); // 同会话迟到任务：不发布、不替换当前根。
            }
        }
        let previous: Vec<String> = self.store.root_artifacts(&self.project, "cst-manifest")?;
        self.store
            .register_root(&self.project, artifact, "cst-manifest")?;
        for old in previous {
            if old != artifact {
                self.store
                    .remove_root(&self.project, "cst-manifest", &old)?;
            }
        }
        self.published_revision.set(Some(
            revision.max(self.published_revision.get().unwrap_or(0)),
        ));
        Ok(())
    }

    /// 本 store 的工程名（流式 builder 的当前根按工程隔离——F787-R2-02）。
    pub fn project(&self) -> &str {
        &self.project
    }

    /// 打开分页视图（跨进程恢复：新 store 实例 + manifest key → CstView）。
    /// 视图持有 store lease——读期间 GC 不回收；drop 后可回收（旧版本）。
    pub fn open_cst(
        &self,
        p: &GrammarProgram,
        snap: &Snapshot,
    ) -> Result<Option<CstView<'_>>, AdapterError> {
        let mut comps = cst_key_base(snap, p);
        comps.push(("page".into(), "manifest".into()));
        let key = QueryKey::new(layer("A03"), comps);
        let Some(loaded) = self.store.get_artifact("A03", &key)? else {
            return Ok(None);
        };
        self.view_from_manifest(
            &loaded.payload,
            loaded.artifact_id,
            Some((snap.id().as_str(), p)),
            None,
        )
    }

    /// 流式源的视图打开（manifest key 用 stream snapshot 域 hash）。
    pub fn open_cst_stream(
        &self,
        p: &GrammarProgram,
        stream_id: &str,
        total_len: u64,
    ) -> Result<Option<CstView<'_>>, AdapterError> {
        let key = QueryKey::new(
            layer("A03"),
            vec![
                ("source-snapshot".into(), stream_id.to_string()),
                ("grammar-id".into(), p.grammar_id().as_str().to_string()),
                ("parser-version".into(), PARSER_ABI_VERSION.to_string()),
                ("page".into(), "manifest".into()),
            ],
        );
        let Some(loaded) = self.store.get_artifact("A03", &key)? else {
            return Ok(None);
        };
        // 身份强校验（F787-R2-05）：payload 的 snapshot/grammar 必须与请求的
        // stream id / grammar 一致——有效 key 下错误身份受控拒绝，不旁路。
        self.view_from_manifest(
            &loaded.payload,
            loaded.artifact_id,
            Some((stream_id, p)),
            Some(total_len),
        )
    }

    fn view_from_manifest(
        &self,
        payload: &[u8],
        artifact_id: String,
        identity: Option<(&str, &GrammarProgram)>,
        declared_len: Option<u64>,
    ) -> Result<Option<CstView<'_>>, AdapterError> {
        let manifest: CstManifest = serde_json::from_slice(payload)
            .map_err(|e| AdapterError::Admissions(format!("bad manifest: {e}")))?;
        if let Some((snap_id, p)) = identity {
            if manifest.snapshot != snap_id || manifest.grammar != p.grammar_id().as_str() {
                return Err(AdapterError::Admissions(
                    "manifest identity mismatch (snapshot/grammar)".into(),
                ));
            }
        }
        if let Some(len) = declared_len {
            if manifest.total_len != len {
                return Err(AdapterError::Admissions(
                    "manifest total_len mismatch".into(),
                ));
            }
        }
        validate_manifest(&manifest)?;
        // 视图 lease（F787-R1-06）：manifest artifact 在读期间保活；视图 drop
        // （lease 释放）后，非当前根的旧版本可被 GC 回收。
        let lease = self
            .store
            .acquire_lease("cst-view", 3600, Some(&artifact_id))?;
        // manifest 载入计入读取口径（视图创建即已读取；framed≈payload+头部）。
        let mbytes = payload.len() + 64;
        Ok(Some(CstView {
            store: &self.store,
            manifest,
            declared_total_len: declared_len,
            hot: std::cell::RefCell::new(PageCache::new(CST_HOT_BUDGET)),
            max_chunk_read: std::cell::Cell::new(mbytes),
            total_chunk_read: std::cell::Cell::new(mbytes as u64),
            manifest_read_bytes: std::cell::Cell::new(mbytes as u64),
            _lease: Some(lease),
        }))
    }
}

/// A03 manifest/page 键的公共分量。
fn cst_key_base(snap: &Snapshot, p: &GrammarProgram) -> Vec<(String, String)> {
    vec![
        ("source-snapshot".into(), snap.id().as_str().to_string()),
        ("grammar-id".into(), p.grammar_id().as_str().to_string()),
        ("parser-version".into(), PARSER_ABI_VERSION.to_string()),
    ]
}

// ---- checkpoint 编解码（F787-R1-05：完整 mode/未闭合状态/行） ----

const TOKENS_CODEC_VERSION: u32 = 2;

fn encode_checkpoint(out: &mut Vec<u8>, cp: &ScanCheckpoint) {
    out.extend_from_slice(&cp.offset.to_le_bytes());
    out.extend_from_slice(&cp.mode.to_le_bytes());
    match &cp.mid {
        None => out.push(0),
        Some(MidToken::Str {
            start,
            start_line,
            quote,
            escape_pending,
        }) => {
            out.push(1);
            out.extend_from_slice(&start.to_le_bytes());
            out.extend_from_slice(&start_line.to_le_bytes());
            out.push(*quote);
            out.push(*escape_pending as u8);
        }
        Some(MidToken::Block {
            start,
            start_line,
            depth,
        }) => {
            out.push(2);
            out.extend_from_slice(&start.to_le_bytes());
            out.extend_from_slice(&start_line.to_le_bytes());
            out.extend_from_slice(&depth.to_le_bytes());
        }
    }
    out.extend_from_slice(&cp.line.to_le_bytes());
}

fn decode_checkpoint(b: &[u8]) -> Result<(ScanCheckpoint, usize), String> {
    if b.len() < 13 {
        return Err("checkpoint truncated".into());
    }
    let offset = u64::from_le_bytes(b[0..8].try_into().unwrap());
    let mode = u32::from_le_bytes(b[8..12].try_into().unwrap());
    let tag = b[12];
    let mut i = 13usize;
    let mid = match tag {
        0 => None,
        1 => {
            if b.len() < i + 14 {
                return Err("mid-str truncated".into());
            }
            let start = u64::from_le_bytes(b[i..i + 8].try_into().unwrap());
            let start_line = u32::from_le_bytes(b[i + 8..i + 12].try_into().unwrap());
            let quote = b[i + 12];
            let escape_pending = b[i + 13] != 0;
            i += 14;
            Some(MidToken::Str {
                start,
                start_line,
                quote,
                escape_pending,
            })
        }
        2 => {
            if b.len() < i + 16 {
                return Err("mid-block truncated".into());
            }
            let start = u64::from_le_bytes(b[i..i + 8].try_into().unwrap());
            let start_line = u32::from_le_bytes(b[i + 8..i + 12].try_into().unwrap());
            let depth = u32::from_le_bytes(b[i + 12..i + 16].try_into().unwrap());
            i += 16;
            Some(MidToken::Block {
                start,
                start_line,
                depth,
            })
        }
        k => return Err(format!("bad mid tag {k}")),
    };
    if b.len() < i + 4 {
        return Err("checkpoint line truncated".into());
    }
    let line = u32::from_le_bytes(b[i..i + 4].try_into().unwrap());
    i += 4;
    Ok((
        ScanCheckpoint {
            offset,
            mode,
            mid,
            line,
        },
        i,
    ))
}

fn derive_final_checkpoint(
    p: &GrammarProgram,
    snap: &Snapshot,
    tokens: &[Token],
) -> ScanCheckpoint {
    let offset = snap.len();
    let mut line = 1u32;
    let mut mode = 0u32;
    let mut mid = None;
    if let Some(t) = tokens.last() {
        let spelling = snap.read_bytes(t.start, t.end).unwrap_or_default();
        let in_tok_newlines = spelling.iter().filter(|&&b| b == b'\n').count() as u32;
        line = t.line + in_tok_newlines;
        if t.error {
            // 未闭合：按语法包的 matcher 族重放拼写，还原真实 mid 状态。
            let block_rules: Vec<&ac_syntax_rt::program::ScanRule> = p
                .scanner
                .rules
                .iter()
                .filter(|r| {
                    matches!(
                        r.matcher,
                        ac_syntax_rt::program::Matcher::BlockComment { .. }
                    )
                })
                .collect();
            let str_rules: Vec<&ac_syntax_rt::program::ScanRule> = p
                .scanner
                .rules
                .iter()
                .filter(|r| matches!(r.matcher, ac_syntax_rt::program::Matcher::StringLit { .. }))
                .collect();
            let starts_with = |lit: &str| spelling.starts_with(lit.as_bytes());
            if block_rules.len() == 1 {
                if let ac_syntax_rt::program::Matcher::BlockComment { start, end, nested } =
                    &block_rules[0].matcher
                {
                    if starts_with(start) {
                        // 重放嵌套计数：每个未配对 opener 计一层。
                        let sb = start.as_bytes();
                        let eb = end.as_bytes();
                        let mut depth: u32 = 0;
                        let mut i = 0usize;
                        while i < spelling.len() {
                            if spelling[i..].starts_with(sb) {
                                depth += 1;
                                i += sb.len();
                                continue;
                            }
                            if spelling[i..].starts_with(eb) && depth > 0 {
                                depth -= 1;
                                i += eb.len();
                                continue;
                            }
                            i += 1;
                        }
                        let _ = nested;
                        mid = Some(MidToken::Block {
                            start: t.start,
                            start_line: t.line,
                            depth: depth.max(1),
                        });
                        mode = block_rules[0].mode;
                    }
                }
            }
            if mid.is_none() && str_rules.len() == 1 {
                if let ac_syntax_rt::program::Matcher::StringLit { quote, escape } =
                    &str_rules[0].matcher
                {
                    if spelling.first() == Some(quote) {
                        let mut pending = false;
                        for &b in spelling.iter().skip(1) {
                            if pending {
                                pending = false;
                                continue;
                            }
                            if *escape && b == b'\\' {
                                pending = true;
                                continue;
                            }
                            if b == *quote {
                                // 闭合出现在拼写内 = 不是未闭合（防御：按 token
                                // error 标记之外的状态不构造 mid）。
                                pending = false;
                                break;
                            }
                        }
                        mid = Some(MidToken::Str {
                            start: t.start,
                            start_line: t.line,
                            quote: *quote,
                            escape_pending: pending,
                        });
                        mode = str_rules[0].mode;
                    }
                }
            }
        } else if t.token != u32::MAX {
            if let Some(r) = p.scanner.rules.iter().find(|r| r.token == t.token) {
                mode = r.next_mode;
            }
        }
    }
    ScanCheckpoint {
        offset,
        mode,
        mid,
        line,
    }
}

fn decode_tokens_v2(mut b: &[u8]) -> Result<(Vec<Token>, ScanCheckpoint), String> {
    if b.len() < 8 {
        return Err("token chunk truncated".into());
    }
    let ver = u32::from_le_bytes(b[0..4].try_into().unwrap());
    if ver != TOKENS_CODEC_VERSION {
        return Err(format!(
            "token chunk version {ver} != {TOKENS_CODEC_VERSION}"
        ));
    }
    let n = u32::from_le_bytes(b[4..8].try_into().unwrap()) as usize;
    b = &b[8..];
    if b.len() < n * 25 {
        return Err(format!("token chunk length mismatch: {} < {}", b.len(), n * 25).into());
    }
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let c = &b[i * 25..(i + 1) * 25];
        out.push(Token {
            token: u32::from_le_bytes(c[0..4].try_into().unwrap()),
            start: u64::from_le_bytes(c[4..12].try_into().unwrap()),
            end: u64::from_le_bytes(c[12..20].try_into().unwrap()),
            line: u32::from_le_bytes(c[20..24].try_into().unwrap()),
            error: c[24] != 0,
        });
    }
    b = &b[n * 25..];
    let (cp, _) = decode_checkpoint(b)?;
    Ok((out, cp))
}

// ---- CST 块 <-> 页 wire 编解码（块内边 = 局部下标） ----

/// 块 → 页 payload：[u32 n][u32 e][nodes×32B][edges×(child u32, field u32)]。
fn encode_block(blk: &CstBlock) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + blk.nodes.len() * 32 + blk.edges.len() * 8);
    out.extend_from_slice(&(blk.nodes.len() as u32).to_le_bytes());
    out.extend_from_slice(&(blk.edges.len() as u32).to_le_bytes());
    for nd in &blk.nodes {
        out.extend_from_slice(&nd.kind.to_le_bytes());
        out.extend_from_slice(&nd.start.to_le_bytes());
        out.extend_from_slice(&nd.len.to_le_bytes());
        out.extend_from_slice(&nd.first_child.to_le_bytes());
        out.extend_from_slice(&nd.child_count.to_le_bytes());
        out.extend_from_slice(&nd.expected.to_le_bytes());
        out.extend_from_slice(&nd.decl.to_le_bytes());
        out.push(nd.flags);
        out.extend_from_slice(&[0u8; 3]);
    }
    for k in 0..blk.edges.len() {
        out.extend_from_slice(&blk.edges[k].to_le_bytes());
        out.extend_from_slice(&blk.fields[k].to_le_bytes());
    }
    out
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CstManifest {
    pub snapshot: String,
    pub grammar: String,
    pub total_len: u64,
    pub diags: Vec<DiagEntry>,
    /// 树根的孩子边（全局 id + field）——根节点自身不在页内（零边）。
    pub root_edges: Vec<(u32, u32)>,
    /// 页表：全局节点区间 → chunk id（页 = 内存内容块同构）。
    pub pages: Vec<PageEntry>,
    /// 声明目录分页（PLAN-787：百万级声明目录不能整驻 manifest——每
    /// DECL_INDEX_PAGE 条一个 CstManifest chunk，按需取）。
    pub decl_index: Vec<PageEntry>,
    pub decl_count: u64,
    /// 兼容字段：非流式小树直接内联目录。
    pub decls: Vec<DeclEntry>,
    /// 源页 chunk id（文档序）——重建原文用（F787-R1-06）。
    pub source_pages: Vec<String>,
}

/// 每目录页声明数（F787-R2/3-09：目录页 framed 约 60B/条——2048 条 ≈ 120KiB，
/// 单声明查询增量读取（目录页+数据页 ≈ 285KiB）满足小范围 payload ≤512KiB
/// 合同的总量口径）。
pub const DECL_INDEX_PAGE: usize = 2048;


#[derive(serde::Serialize, serde::Deserialize)]
pub struct DeclIndexPage {
    pub first: u64,
    pub decls: Vec<DeclEntry>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct DiagEntry {
    pub message: String,
    pub start: u64,
    pub end: u64,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PageEntry {
    pub chunk: String,
    pub first_node: u32,
    pub node_count: u32,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct DeclEntry {
    pub origin: u64,
    pub node: u32,
    pub node_lo: u32,
    pub page: u32,
}

/// manifest 结构校验（F787-R1-07：坏 ref 在**读入口**受控拒绝，不 panic）。
fn validate_manifest(m: &CstManifest) -> Result<(), AdapterError> {
    let total = m.pages.iter().map(|p| p.node_count as u64).sum::<u64>();
    // 页表区间连续性：first_node 必须按序衔接（页=块的全局布局）。
    let mut expect = 0u64;
    for (i, p) in m.pages.iter().enumerate() {
        if p.first_node as u64 != expect {
            return Err(AdapterError::Admissions(format!(
                "page {i} first_node {} breaks the contiguous node layout (expected {expect})",
                p.first_node
            )));
        }
        expect += p.node_count as u64;
    }
    for d in &m.decls {
        let Some(pe) = m.pages.get(d.page as usize) else {
            return Err(AdapterError::Admissions(format!(
                "decl ref page {} out of range ({} pages)",
                d.page,
                m.pages.len()
            )));
        };
        // 下界先判后算（F787-R2-05：d.node < first_node 时无符号相减溢出 panic）。
        if d.node < pe.first_node || d.node - pe.first_node >= pe.node_count {
            return Err(AdapterError::Admissions(format!(
                "decl node {} outside page {} range [{},{})",
                d.node,
                d.page,
                pe.first_node,
                pe.first_node as u64 + pe.node_count as u64
            )));
        }
        if d.node_lo > d.node {
            return Err(AdapterError::Admissions(format!(
                "decl node_lo {} > node {}",
                d.node_lo, d.node
            )));
        }
        // 越源拒绝（F787-R3-05）：声明 origin 必须落在源长度内——有效
        // frame/hash 下的错偏移不得作为成功查询返回。
        if d.origin > m.total_len {
            return Err(AdapterError::Admissions(format!(
                "decl origin {} beyond source total_len {}",
                d.origin, m.total_len
            )));
        }
    }
    for &(root, _f) in &m.root_edges {
        if root as u64 >= total {
            return Err(AdapterError::Admissions(format!(
                "root edge node {root} outside tree ({total} nodes)"
            )));
        }
    }
    if m.decl_count
        != (m.decls.len() as u64
            + m.decl_index
                .iter()
                .map(|p| p.node_count as u64)
                .sum::<u64>())
    {
        return Err(AdapterError::Admissions(
            "decl_count does not match inline decls + index pages".into(),
        ));
    }
    Ok(())
}

// ---- 页视图（按需取页，不反序列化整树） ----

struct PageCache {
    budget: usize,
    map: std::collections::HashMap<u32, (usize, Vec<CstPageData>)>, // page_idx -> (bytes, decoded)
    order: Vec<u32>,
    bytes: usize,
    /// 单页最大解码字节（查询 payload 上界观测）。
    max_page: usize,
}

#[derive(Clone)]
struct CstPageData {
    nodes: Vec<CstNode>,
    edges: Vec<(u32, u32)>,
}

impl PageCache {
    fn new(budget: usize) -> Self {
        PageCache {
            budget,
            map: std::collections::HashMap::new(),
            order: Vec::new(),
            bytes: 0,
            max_page: 0,
        }
    }
    fn insert(&mut self, idx: u32, data: Vec<CstPageData>) {
        let sz: usize = data
            .iter()
            .map(|p| p.nodes.len() * 32 + p.edges.len() * 8)
            .sum();
        if sz > self.max_page {
            self.max_page = sz;
        }
        self.map.insert(idx, (sz, data));
        self.order.push(idx);
        self.bytes += sz;
        while self.bytes > self.budget && self.order.len() > 1 {
            let evict = self.order.remove(0);
            if let Some((s, _)) = self.map.remove(&evict) {
                self.bytes -= s;
            }
        }
    }
}

use std::collections::HashMap;

/// 分页只读视图。持有 store lease（manifest artifact）——读期间 GC 不回收
/// 本版本；drop（lease 释放）后旧版本可被 GC 回收（F787-R1-06）。
pub struct CstView<'s> {
    store: &'s ManagedStore,
    manifest: CstManifest,
    /// 流式视图的 total_len 由调用方声明（与 manifest 一致性在 open 时校验）。
    declared_total_len: Option<u64>,
    hot: std::cell::RefCell<PageCache>,
    /// 单次 chunk 读取峰值（framed 字节；数据页/目录页/源页统一口径——
    /// F787-R2-09：小范围查询 payload 合同的完整观测）。
    max_chunk_read: std::cell::Cell<usize>,
    /// 视图生命周期累计读取字节（含创建时的 manifest——F787-R3-06：小查询
    /// 的**全部**读取量，不只单页最大值）。
    total_chunk_read: std::cell::Cell<u64>,
    /// 创建时 manifest 实际读取字节（与后续查询读取分列的入口成本）。
    manifest_read_bytes: std::cell::Cell<u64>,
    _lease: Option<ac_artifact_probe::store::Lease>,
}

impl<'s> CstView<'s> {
    pub fn decl_count(&self) -> usize {
        if self.manifest.decl_count > 0 || !self.manifest.decl_index.is_empty() {
            self.manifest.decl_count as usize
        } else {
            self.manifest.decls.len()
        }
    }

    fn decl_entry(&self, i: usize) -> Result<DeclEntry, AdapterError> {
        if !self.manifest.decls.is_empty() {
            return self
                .manifest
                .decls
                .get(i)
                .cloned()
                .ok_or_else(|| AdapterError::Admissions(format!("decl index {i} out of range")));
        }
        // 目录分页：定位条目所在索引页并按需取。
        let bi = i / DECL_INDEX_PAGE;
        let pe = self
            .manifest
            .decl_index
            .get(bi)
            .ok_or_else(|| AdapterError::Admissions(format!("decl index {i} out of range")))?;
        let Some(payload) = self.store.read_chunk_gated(&pe.chunk)? else {
            return Err(AdapterError::Admissions(format!(
                "decl index page {bi} missing"
            )));
        };
        let (kind, payload) = ac_artifact_probe::codec::unframe_chunk(&payload)
            .map_err(|e| AdapterError::Admissions(format!("decl index page not framed: {e:?}")))?;
        if kind != ac_artifact_probe::codec::ChunkKind::CstManifest {
            return Err(AdapterError::Admissions(format!(
                "decl index page chunk kind {kind:?} != CstManifest"
            )));
        }
        let page: DeclIndexPage = serde_json::from_slice(payload)
            .map_err(|e| AdapterError::Admissions(format!("bad decl index page: {e}")))?;
        page.decls
            .get(i % DECL_INDEX_PAGE)
            .cloned()
            .ok_or_else(|| AdapterError::Admissions(format!("decl {i} missing from index page")))
    }
    pub fn debug_page_chunk(&self, idx: usize) -> Option<String> {
        self.manifest.pages.get(idx).map(|p| p.chunk.clone())
    }
    pub fn debug_decl_entries(&self) -> Vec<(u64, u32, u32, u32)> {
        self.manifest
            .decls
            .iter()
            .map(|d| (d.origin, d.node, d.node_lo, d.page))
            .collect()
    }
    pub fn total_len(&self) -> u64 {
        self.declared_total_len.unwrap_or(self.manifest.total_len)
    }
    pub fn diag_count(&self) -> usize {
        self.manifest.diags.len()
    }
    /// 声明绝对范围（manifest 目录直读，零页读取）。
    pub fn decl_range(&self, i: usize) -> Result<(u64, u64), AdapterError> {
        let d = self.decl_entry(i)?;
        let pe = self.manifest.pages.get(d.page as usize).ok_or_else(|| {
            AdapterError::Admissions(format!("decl page {} out of range", d.page))
        })?;
        if d.node < pe.first_node {
            return Err(AdapterError::Admissions(format!(
                "decl node {} below page {} base {}",
                d.node, d.page, pe.first_node
            )));
        }
        let page = self.page(d.page)?;
        let local = (d.node - pe.first_node) as usize;
        let nd = page
            .get(0)
            .and_then(|p| p.nodes.get(local))
            .ok_or_else(|| {
                AdapterError::Admissions(format!("decl node {local} missing from page {}", d.page))
            })?;
        let lo = d.origin + nd.start as u64;
        let hi = lo + nd.len as u64;
        // 读取 admission（F787-R3-05）：绝对 span 不得越出源长度。
        if hi > self.total_len() {
            return Err(AdapterError::Admissions(format!(
                "decl span [{lo},{hi}) beyond source total_len {}",
                self.total_len()
            )));
        }
        Ok((lo, hi))
    }
    /// 单声明子树重建（仅取该声明所在页；页内节点为块内局部结构）。
    pub fn decl_subtree(&self, i: usize) -> Result<CstTree, AdapterError> {
        let d = self.decl_entry(i)?;
        let Some(pe) = self.manifest.pages.get(d.page as usize) else {
            return Err(AdapterError::Admissions(format!(
                "decl page {} out of range",
                d.page
            )));
        };
        if d.node < pe.first_node || d.node - pe.first_node >= pe.node_count {
            return Err(AdapterError::Admissions(format!(
                "decl node {} outside page {} node range",
                d.node, d.page
            )));
        }
        let page = self.page(d.page)?;
        let p0 = page
            .get(0)
            .ok_or_else(|| AdapterError::Admissions("empty page".into()))?;
        let root_local = (d.node - pe.first_node) as usize;
        if root_local >= p0.nodes.len() {
            return Err(AdapterError::Admissions(format!(
                "decl root local {root_local} outside page nodes {}",
                p0.nodes.len()
            )));
        }
        // BFS 收集可达节点。树不变量：每个节点恰一个父——边目标重复出现
        // （环/共享孩子）即非树，受控拒绝（F787-R2-05：不得装回环边后
        // 返回 Ok，否则 canonical/outline 等无环遍历无限增长）。
        let mut seen = std::collections::HashSet::new();
        let mut order = Vec::new();
        let mut stack = vec![root_local];
        seen.insert(root_local);
        while let Some(n) = stack.pop() {
            order.push(n);
            let nd = p0.nodes[n];
            for k in 0..nd.child_count as usize {
                let idx = nd.first_child as usize + k;
                let Some(&(c, _)) = p0.edges.get(idx) else {
                    return Err(AdapterError::Admissions(format!(
                        "edge slot {idx} missing from page {}",
                        d.page
                    )));
                };
                let cl = c as usize;
                if cl >= p0.nodes.len() {
                    return Err(AdapterError::Admissions(format!(
                        "edge target {cl} outside page nodes ({})",
                        p0.nodes.len()
                    )));
                }
                if !seen.insert(cl) {
                    return Err(AdapterError::Admissions(format!(
                        "page {} is not a tree: node {cl} referenced twice (cycle/shared child)",
                        d.page
                    )));
                }
                stack.push(cl);
            }
        }
        order.sort_unstable(); // 页内序 = 文档序
        let mut remap = HashMap::new();
        let mut nodes = Vec::with_capacity(order.len());
        for &li in &order {
            remap.insert(li as u32, nodes.len() as u32);
            let nd = p0.nodes[li];
            nodes.push(nd);
        }
        let mut edges = Vec::new();
        let mut fields = Vec::new();
        let mut cursor = 0u32;
        for &li in &order {
            let nd = p0.nodes[li];
            let new_id = remap[&(li as u32)] as usize;
            nodes[new_id].first_child = cursor;
            nodes[new_id].decl = 1; // 子树坐标系：唯一声明（原树 decl 索引越界）
            for k in 0..nd.child_count as usize {
                let (c, f) = p0.edges[(nd.first_child as usize) + k];
                if let Some(&nc) = remap.get(&c) {
                    edges.push(nc);
                    fields.push(f);
                }
            }
            cursor += nd.child_count;
        }
        let root = remap[&(root_local as u32)];
        // 子树坐标系：上述重写已把全部节点 decl 置 1（唯一声明）——块内
        // 声明映射只需一个槽位指向 decls[0]。
        let _original_ordinal = p0.nodes[root_local].decl;
        let bd: Vec<u32> = vec![0u32];
        Ok(CstTree {
            blocks: vec![Rc::new(CstBlock {
                nodes,
                edges,
                fields,
            })],
            bases: vec![0],
            root,
            root_edges: Vec::new(),
            decls: vec![ac_syntax_rt::cst::DeclUnit {
                origin: d.origin,
                node: root,
                node_lo: 0,
            }],
            block_decl: vec![bd],
            diags: self.manifest_diags_for(d.origin),
            total_len: self.total_len(),
        })
    }

    /// 从 store 重建源文本（F787-R1-06：源页按文档序拼接）。
    pub fn source_bytes(&self) -> Result<Vec<u8>, AdapterError> {
        let mut out = Vec::with_capacity(self.total_len() as usize);
        self.for_each_source_page(|chunk| {
            out.extend_from_slice(chunk);
            Ok(())
        })?;
        if out.len() as u64 != self.total_len() {
            return Err(AdapterError::Admissions(format!(
                "rebuilt source {} != declared total {}",
                out.len(),
                self.total_len()
            )));
        }
        Ok(out)
    }

    /// 源页流式遍历（零整源驻留——预算负载的重建比对走此路径）。
    pub fn for_each_source_page<F: FnMut(&[u8]) -> Result<(), AdapterError>>(
        &self,
        mut f: F,
    ) -> Result<(), AdapterError> {
        for (i, chunk) in self.manifest.source_pages.iter().enumerate() {
            let Some(payload) = self.store.read_chunk_gated(chunk)? else {
                return Err(AdapterError::Admissions(format!(
                    "source page {i} missing (CAS miss)"
                )));
            };
            let (kind, payload) = ac_artifact_probe::codec::unframe_chunk(&payload)
                .map_err(|e| AdapterError::Admissions(format!("source page not framed: {e:?}")))?;
            if kind != ac_artifact_probe::codec::ChunkKind::SourcePage {
                return Err(AdapterError::Admissions(format!(
                    "source page {i} kind {kind:?} != SourcePage"
                )));
            }
            f(payload)?;
        }
        Ok(())
    }

    /// manifest 驻留受管字节估算（索引计入受管预算的口径，F787-R1-08）。
    pub fn manifest_bytes(&self) -> usize {
        let m = &self.manifest;
        m.pages.iter().map(|p| p.chunk.len() + 16).sum::<usize>()
            + m.decl_index
                .iter()
                .map(|p| p.chunk.len() + 16)
                .sum::<usize>()
            + m.source_pages.iter().map(|c| c.len() + 16).sum::<usize>()
            + m.decls.iter().map(|_| 24).sum::<usize>()
            + m.diags.iter().map(|d| d.message.len() + 32).sum::<usize>()
            + m.root_edges.len() * 8
            + std::mem::size_of::<CstManifest>()
    }

    /// gated chunk 读取 + 计量（数据/目录/源页统一走此口径；F787-R3-06：
    /// 单次峰值 + 累计总量双口径）。
    fn read_chunk_metered(&self, chunk: &str) -> Result<Option<Vec<u8>>, AdapterError> {
        let out = self.store.read_chunk_gated(chunk)?;
        if let Some(b) = &out {
            let n = b.len();
            if n > self.max_chunk_read.get() {
                self.max_chunk_read.set(n);
            }
            self.total_chunk_read
                .set(self.total_chunk_read.get() + n as u64);
        }
        Ok(out)
    }

    fn manifest_diags_for(&self, _origin: u64) -> Vec<ac_syntax_rt::cst::SynDiag> {
        // 诊断按声明归属过滤成本高；视图层保留全量诊断的轻量投影（消息可查）。
        self.manifest
            .diags
            .iter()
            .map(|d| ac_syntax_rt::cst::SynDiag {
                message: d.message.clone(),
                start: d.start,
                end: d.end,
            })
            .collect()
    }
    fn page(&self, idx: u32) -> Result<Vec<CstPageData>, AdapterError> {
        if let Some((_, v)) = self.hot.borrow().map.get(&idx) {
            return Ok(v.clone());
        }
        let pe =
            self.manifest.pages.get(idx as usize).ok_or_else(|| {
                AdapterError::Admissions(format!("page {idx} missing from manifest"))
            })?;
        let Some(payload) = self.read_chunk_metered(&pe.chunk)? else {
            return Err(AdapterError::Admissions(format!(
                "page chunk {} missing (CAS miss)",
                &pe.chunk[..8.min(pe.chunk.len())]
            )));
        };
        let (kind, payload) = ac_artifact_probe::codec::unframe_chunk(&payload)
            .map_err(|e| AdapterError::Admissions(format!("page chunk not framed: {e:?}")))?;
        if kind != ac_artifact_probe::codec::ChunkKind::CstPage {
            return Err(AdapterError::Admissions(format!(
                "page chunk kind {kind:?} != CstPage"
            )));
        }
        let decoded = decode_page(payload, pe)?;
        self.hot.borrow_mut().insert(idx, decoded.clone());
        Ok(decoded)
    }
}

/// 页解码 + 结构校验（F787-R1-07：边目标越界/坏区间在真实读入口受控拒绝）。
fn decode_page(b: &[u8], pe: &PageEntry) -> Result<Vec<CstPageData>, AdapterError> {
    if b.len() < 8 {
        return Err(AdapterError::Admissions("page truncated".into()));
    }
    let n = u32::from_le_bytes(b[0..4].try_into().unwrap()) as usize;
    let e = u32::from_le_bytes(b[4..8].try_into().unwrap()) as usize;
    if b.len() < 8 + n * 32 + e * 8 {
        return Err(AdapterError::Admissions("page length mismatch".into()));
    }
    if n != pe.node_count as usize {
        return Err(AdapterError::Admissions(format!(
            "page node count {n} != manifest node_count {}",
            pe.node_count
        )));
    }
    let mut nodes = Vec::with_capacity(n);
    for i in 0..n {
        let c = &b[8 + i * 32..8 + (i + 1) * 32];
        nodes.push(CstNode {
            kind: u32::from_le_bytes(c[0..4].try_into().unwrap()),
            start: u32::from_le_bytes(c[4..8].try_into().unwrap()),
            len: u32::from_le_bytes(c[8..12].try_into().unwrap()),
            first_child: u32::from_le_bytes(c[12..16].try_into().unwrap()),
            child_count: u32::from_le_bytes(c[16..20].try_into().unwrap()),
            expected: u32::from_le_bytes(c[20..24].try_into().unwrap()),
            decl: u32::from_le_bytes(c[24..28].try_into().unwrap()),
            flags: c[28],
        });
    }
    let mut edges = Vec::with_capacity(e);
    for i in 0..e {
        let c = &b[8 + n * 32 + i * 8..8 + n * 32 + (i + 1) * 8];
        edges.push((
            u32::from_le_bytes(c[0..4].try_into().unwrap()),
            u32::from_le_bytes(c[4..8].try_into().unwrap()),
        ));
    }
    // 边索引一致性（first_child+count 越界 = 坏 offset/ref）+ 孩子目标越界。
    for nd in &nodes {
        let fc = nd.first_child as usize;
        let ec = nd.child_count as usize;
        if fc + ec > e {
            return Err(AdapterError::Admissions(format!(
                "edge range {fc}+{ec} > {e} (bad ref)"
            )));
        }
    }
    for k in 0..e {
        if edges[k].0 as usize >= n {
            return Err(AdapterError::Admissions(format!(
                "edge target {} outside page nodes ({n})",
                edges[k].0
            )));
        }
    }
    Ok(vec![CstPageData { nodes, edges }])
}

// ---- 流式构建（V-11：64MiB 级源不驻留整树；F787-R1-08 目录在线落盘） ----

/// 分块流式 CST 生产者：每块独立 parse→内容块页→入库 + 源页持久化 + 声明
/// 目录**在线分页落盘**（缓冲 ≤DECL_INDEX_PAGE 条，不在内存累积百万级目录）。
pub struct StreamingCstBuilder<'a> {
    sas: &'a SyntaxArtifactStore,
    p: &'a GrammarProgram,
    scanner: CompiledScanner,
    budget: Budget,
    prefix_len: u64,
    page_arts: Vec<String>,
    page_chunks: Vec<String>,
    pages: Vec<PageEntry>,
    src_page_chunks: Vec<String>,
    /// 源页 artifact ids（manifest required 闭包用——F787-R2-04）。
    src_page_arts: Vec<String>,
    decls_buf: Vec<DeclEntry>,
    decl_index: Vec<PageEntry>,
    decl_index_arts: Vec<String>,
    decl_count: u64,
    next_global: u32,
    snap_hash_parts: Vec<Vec<u8>>,
    diags: Vec<DiagEntry>,
    working_set_peak: usize,
}

impl<'a> StreamingCstBuilder<'a> {
    pub fn new(sas: &'a SyntaxArtifactStore, p: &'a GrammarProgram) -> Self {
        sas.put_grammar(p).expect("grammar publish");
        StreamingCstBuilder {
            sas,
            p,
            scanner: CompiledScanner::compile(p).expect("scanner"),
            budget: Budget::default(),
            prefix_len: 0,
            page_arts: Vec::new(),
            page_chunks: Vec::new(),
            pages: Vec::new(),
            src_page_chunks: Vec::new(),
            src_page_arts: Vec::new(),
            decls_buf: Vec::new(),
            decl_index: Vec::new(),
            decl_index_arts: Vec::new(),
            decl_count: 0,
            next_global: 0,
            snap_hash_parts: Vec::new(),
            diags: Vec::new(),
            working_set_peak: 0,
        }
    }

    /// 构建预算（巨声明/深嵌套受控 ResourceExhausted，不是语法错误）。
    pub fn with_budget(mut self, budget: Budget) -> Self {
        self.budget = budget;
        self
    }

    /// 受管工作集字节（目录缓冲 + 页表/源页表/诊断；峰值计量用）。
    pub fn managed_bytes(&self) -> usize {
        self.decls_buf.capacity() * std::mem::size_of::<DeclEntry>()
            + self.pages.capacity() * std::mem::size_of::<PageEntry>()
            + self.page_arts.capacity() * 70
            + self.page_chunks.capacity() * 70
            + self.src_page_chunks.capacity() * 70
            + self.decl_index.capacity() * std::mem::size_of::<PageEntry>()
            + self.decl_index_arts.capacity() * 70
            + self.diags.capacity() * std::mem::size_of::<DiagEntry>()
            + self.snap_hash_parts.capacity() * 80
            + self.src_page_arts.capacity() * 70
    }

    pub fn working_set_peak(&self) -> usize {
        self.working_set_peak
    }

    pub fn decl_count(&self) -> u64 {
        self.decl_count
    }

    /// 受管峰值采样（F787-R2-09）：常驻目录 + 调用点仍驻留的暂存量
    /// （parse 树/页 payload/目录页 JSON 等）——不只统计常驻容器。
    fn touch_peak_with(&mut self, transient: usize) {
        let b = self.managed_bytes() + transient;
        if b > self.working_set_peak {
            self.working_set_peak = b;
        }
    }

    /// 推入一个块（块间在行边界切分）；块内 parse 出的内容块逐块入库，
    /// 声明目录缓冲满页即落盘。
    pub fn push_chunk(&mut self, chunk: &str) -> Result<(), AdapterError> {
        let store_rc = Rc::new(ac_syntax_rt::snapshot::PageStore::new());
        let snap = Snapshot::from_text(&store_rc, chunk);
        // 身份时序（F787-R2-03）：本块 snapshot hash **先**入 parts 再构造任何
        // 页/目录页键——否则首块键恒为空前缀 hash，第二份文档在共享 store 里
        // 必然 Conflict（"新文档必须空 store"不能成为隐藏条件）。
        let mut part = snap.id().as_str().as_bytes().to_vec();
        part.push(0);
        self.snap_hash_parts.push(part);
        // 源页持久化（内容寻址，跨块去重；artifact ids 保留供 manifest 闭包）。
        let (arts, chunks) = self.sas.put_source_pages(&snap)?;
        self.src_page_arts.extend(arts);
        self.src_page_chunks.extend(chunks);
        let out = ac_syntax_rt::parse::parse_snapshot(self.p, &self.scanner, &snap, &self.budget)
            .map_err(AdapterError::Rt)?;
        // 受管峰值（F787-R3-06）：parse 树/词法产物仍驻留；此基线在整个块
        // 处理期间有效（树直到块结束才释放），数据页编码处合并 payload 计入。
        let transient_base: usize = out
            .tree
            .blocks
            .iter()
            .map(|b| b.nodes.len() * 40)
            .sum::<usize>()
            + out.tokens.len() * std::mem::size_of::<Token>();
        self.touch_peak_with(transient_base);
        let tree = &out.tree;
        // 块 → 数据页；块内声明 → 目录条目（全局 id = next_global + 局部）。
        for (bi, blk) in tree.blocks.iter().enumerate() {
            let payload = encode_block(blk);
            // 同驻留合计（F787-R3-06）：parse 树 + 页 payload 同时在堆上。
            self.touch_peak_with(transient_base + payload.len());
            let idx = self.pages.len();
            let key = QueryKey::new(
                layer("A03"),
                vec![
                    ("source-snapshot".into(), self.stream_snapshot_component()),
                    (
                        "grammar-id".into(),
                        self.p.grammar_id().as_str().to_string(),
                    ),
                    ("parser-version".into(), PARSER_ABI_VERSION.to_string()),
                    ("page".into(), idx.to_string()),
                ],
            );
            let art = self.sas.inner().put_artifact(PutRequest {
                layer: "A03",
                key,
                kind: ac_artifact_probe::codec::ChunkKind::CstPage,
                payload: payload.clone(),
                edges: vec![],
                project: None,
            })?;
            let chunk_id = ac_artifact_probe::codec::framed_chunk_id(
                ac_artifact_probe::codec::ChunkKind::CstPage,
                &payload,
            )
            .to_string();
            let base = self.next_global;
            for di in 0..tree.block_decl[bi].len() {
                if tree.block_decl[bi][di] == UNGRAFTED_DECL {
                    continue;
                }
                let du = &tree.decls[tree.block_decl[bi][di] as usize];
                self.decls_buf.push(DeclEntry {
                    origin: self.prefix_len + du.origin,
                    node: base + (du.node - tree.bases[bi]),
                    node_lo: base + (du.node_lo - tree.bases[bi]),
                    page: idx as u32,
                });
                self.decl_count += 1;
            }
            self.page_arts.push(art);
            self.pages.push(PageEntry {
                chunk: chunk_id,
                first_node: base,
                node_count: blk.nodes.len() as u32,
            });
            self.next_global += blk.nodes.len() as u32;
        }
        while self.decls_buf.len() >= DECL_INDEX_PAGE {
            // 严格整页切分：一次 flush 只出 DECL_INDEX_PAGE 条，余量留待下页
            // （目录页与 decl_count/页号对齐的前提）。
            let rest = self.decls_buf.split_off(DECL_INDEX_PAGE);
            let page_decls = std::mem::replace(&mut self.decls_buf, rest);
            self.write_decl_page(page_decls)?;
        }
        for dg in &tree.diags {
            self.diags.push(DiagEntry {
                message: dg.message.clone(),
                start: self.prefix_len + dg.start,
                end: self.prefix_len + dg.end,
            });
        }
        self.prefix_len += chunk.len() as u64;
        self.touch_peak_with(0);
        Ok(())
    }

    /// 声明目录页在线落盘（严格整页；edges 挂最近数据页——belt-and-braces；
    /// manifest 统一 required 全部目录页）。
    fn write_decl_page(&mut self, taken: Vec<DeclEntry>) -> Result<(), AdapterError> {
        let bi = self.decl_index.len();
        let first = (bi as u64) * DECL_INDEX_PAGE as u64;
        let count = taken.len() as u32;
        let cap = taken.capacity() * std::mem::size_of::<DeclEntry>();
        let page = DeclIndexPage {
            first,
            decls: taken,
        };
        let payload =
            serde_json::to_vec(&page).map_err(|e| AdapterError::Admissions(e.to_string()))?;
        // 目录页瞬态计入受管峰值（F787-R3-06：JSON + 条目缓冲）。
        self.touch_peak_with(payload.len() + cap);
        let key = QueryKey::new(
            layer("A03"),
            vec![
                ("source-snapshot".into(), self.stream_snapshot_component()),
                (
                    "grammar-id".into(),
                    self.p.grammar_id().as_str().to_string(),
                ),
                ("parser-version".into(), PARSER_ABI_VERSION.to_string()),
                ("page".into(), format!("declidx-{bi}")),
            ],
        );
        let mut edges: Vec<(String, ac_artifact_probe::identity::RefKind)> = Vec::new();
        if let Some(last) = self.page_arts.last() {
            edges.push((
                last.clone(),
                ac_artifact_probe::identity::RefKind::RequiredData,
            ));
        }
        let art = self.sas.inner().put_artifact(PutRequest {
            layer: "A03",
            key,
            kind: ac_artifact_probe::codec::ChunkKind::CstManifest,
            payload: payload.clone(),
            edges,
            project: None,
        })?;
        let cid = ac_artifact_probe::codec::framed_chunk_id(
            ac_artifact_probe::codec::ChunkKind::CstManifest,
            &payload,
        )
        .to_string();
        self.decl_index_arts.push(art);
        self.decl_index.push(PageEntry {
            chunk: cid,
            first_node: first as u32,
            node_count: count,
        });
        Ok(())
    }

    fn stream_snapshot_component(&self) -> String {
        // 流式源身份 = 各块 snapshot id 的域 hash（无单一大 Snapshot）。
        let joined: Vec<u8> = self.snap_hash_parts.concat();
        ac_artifact_probe::identity::domain_hash("snap-stream", &[&joined])
    }

    /// 收尾：写 manifest（required 全部数据页+目录页；含源页表），返回
    /// manifest artifact id。当前根替换（F787-R1-06）。
    pub fn finish(&mut self, total_len: u64) -> Result<String, AdapterError> {
        if !self.decls_buf.is_empty() {
            let taken = std::mem::take(&mut self.decls_buf);
            self.write_decl_page(taken)?;
        }
        let manifest = CstManifest {
            snapshot: self.stream_snapshot_component(),
            grammar: self.p.grammar_id().as_str().to_string(),
            total_len,
            diags: std::mem::take(&mut self.diags),
            root_edges: Vec::new(), // 流式树无单根；视图经声明目录查询
            pages: std::mem::take(&mut self.pages),
            decl_index: std::mem::take(&mut self.decl_index),
            decl_count: self.decl_count,
            decls: Vec::new(),
            source_pages: std::mem::take(&mut self.src_page_chunks),
        };
        let mkey = QueryKey::new(
            layer("A03"),
            vec![
                ("source-snapshot".into(), manifest.snapshot.clone()),
                ("grammar-id".into(), manifest.grammar.clone()),
                ("parser-version".into(), PARSER_ABI_VERSION.to_string()),
                ("page".into(), "manifest".into()),
            ],
        );
        // required 闭包（F787-R2-04）：数据页 + 目录页 + **源页 artifact** +
        // **A01 语法表**——当前根的 GC 闭包覆盖可重开的全部数据（源可重建、
        // grammar 可再装载）。
        let mut edges: Vec<(String, ac_artifact_probe::identity::RefKind)> = Vec::new();
        edges.extend(
            self.page_arts
                .iter()
                .cloned()
                .map(|a| (a, ac_artifact_probe::identity::RefKind::RequiredData)),
        );
        edges.extend(
            self.decl_index_arts
                .iter()
                .cloned()
                .map(|a| (a, ac_artifact_probe::identity::RefKind::RequiredData)),
        );
        edges.extend(
            self.src_page_arts
                .iter()
                .cloned()
                .map(|a| (a, ac_artifact_probe::identity::RefKind::RequiredData)),
        );
        // A01 语法（new() 已发布，幂等重取 artifact id）。
        let grammar_art = self.sas.put_grammar(self.p)?;
        edges.push((
            grammar_art,
            ac_artifact_probe::identity::RefKind::RequiredData,
        ));
        let payload =
            serde_json::to_vec(&manifest).map_err(|e| AdapterError::Admissions(e.to_string()))?;
        self.touch_peak_with(payload.len() + edges.len() * 70);
        let art = self.sas.inner().put_artifact(PutRequest {
            layer: "A03",
            key: mkey,
            kind: ac_artifact_probe::codec::ChunkKind::CstManifest,
            payload,
            edges,
            project: None,
        })?;
        // 当前根按**本工程**注册（F787-R2-02）；原子顺序同 put_cst
        // （F787-R3-03：先注册新根，成功后定点删除旧根行——注册失败旧根保留）。
        let previous: Vec<String> = self
            .sas
            .inner()
            .root_artifacts(self.sas.project(), "cst-manifest")?;
        self.sas
            .inner()
            .register_root(self.sas.project(), &art, "cst-manifest")?;
        for old in previous {
            if old != art {
                self.sas
                    .inner()
                    .remove_root(self.sas.project(), "cst-manifest", &old)?;
            }
        }
        Ok(art)
    }

    pub fn stream_snapshot_id(&self) -> String {
        self.stream_snapshot_component()
    }
}

impl<'s> CstView<'s> {
    /// 受管页缓存当前字节（预算计量）。
    pub fn debug_hot_bytes(&self) -> usize {
        self.hot.borrow().bytes
    }
    /// 单页最大解码字节（CST 数据页解码口径）。
    pub fn debug_max_page_payload(&self) -> usize {
        self.hot.borrow().max_page
    }
    /// 视图生命周期内单次 chunk 读取峰值（framed 字节；数据/目录/源页统一，
    /// F787-R2-09：小范围查询 payload 合同的完整观测口径）。
    pub fn debug_max_chunk_read(&self) -> usize {
        self.max_chunk_read.get()
    }
    /// 视图生命周期累计读取字节（含 manifest 载入——F787-R3-06）。
    pub fn debug_total_chunk_read(&self) -> u64 {
        self.total_chunk_read.get()
    }
    /// 视图创建时 manifest **实际**读取字节（一次性入口成本，与后续查询读取
    /// 分列；F787-R3-06：不用估算器冒充实读）。
    pub fn debug_manifest_read(&self) -> u64 {
        self.manifest_read_bytes.get()
    }
}
