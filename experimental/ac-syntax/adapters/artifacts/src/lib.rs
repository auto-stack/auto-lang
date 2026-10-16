//! ac-syntax-artifact-adapter：PLAN-787 T-06 —— 真实语法产物接入 783 探针 store。
//!
//! - A01 `grammar-tables`：语法程序 canonical 二进制（GrammarId 绑定，装载时
//!   身份重验，不匹配 = 受控 miss）；
//! - A02 `tokens-checkpoints`：token 流二进制（可再装载供增量词法）；
//! - A03 `tolerant-cst`：CST 按节点区间**分页**（每页独立 chunk，manifest 记
//!   目录/根/页映射）——重启后 `CstView` 按 manifest 小范围查询，**按需取页**，
//!   禁调 parser 且不反序列化整树；
//! - 依赖边：A03 --required--> A02 --required--> A01（required-data 参与探针
//!   GC pin，GC 算法本身不改）；
//! - 来源隔离：QueryKey 不含工程路径（内容身份）；工程归属经 roots/put 的
//!   project 字段——两工程同源同 grammar 共享字节块而诊断/根各自独立。

use ac_artifact_probe::schema;
use ac_artifact_probe::store::{ManagedStore, PutRequest, StoreError};
use ac_artifact_probe::identity::QueryKey;
use ac_syntax_rt::cst::{CstNode, CstTree};
use ac_syntax_rt::lexer::CompiledScanner;
use ac_syntax_rt::program::Budget;
use ac_syntax_rt::snapshot::PageStore;
use std::rc::Rc;
use ac_syntax_rt::lexer::Token;
use ac_syntax_rt::program::{GrammarProgram, RtError};
use ac_syntax_rt::snapshot::Snapshot;
use ac_syntax_rt::{PARSER_ABI_VERSION, RUNTIME_VERSION, SCANNER_ABI_VERSION};
use std::path::Path;

/// 每页最大节点数（CST 分页粒度；页 chunk 远小于探针 64MiB 上限）。
pub const CST_PAGE_NODES: usize = 512;
/// 流式构建页容量（put 次数控制：O(节点/4K)；单子树查询 payload 仍 ≤512KiB）。
pub const STREAM_PAGE_NODES: usize = 4096;
/// 页缓存受管预算（V-11：hot ≤8MiB——超出按 LRU 淘汰，可再取）。
pub const CST_HOT_BUDGET: usize = 8 << 20;

pub struct SyntaxArtifactStore {
    store: ManagedStore,
    project: String,
}

fn layer(id: &str) -> &'static schema::LayerSpec {
    schema::layer(id).expect("syntax layers registered")
}

fn grammar_key(p: &GrammarProgram) -> QueryKey {
    QueryKey::new(layer("A01"), vec![
        ("grammar-id".into(), p.grammar_id().as_str().to_string()),
        ("grammar-revision".into(), p.revision.to_string()),
        ("scanner-abi".into(), SCANNER_ABI_VERSION.to_string()),
    ])
}

fn tokens_key(snap: &Snapshot, p: &GrammarProgram) -> QueryKey {
    QueryKey::new(layer("A02"), vec![
        ("source-snapshot".into(), snap.id().as_str().to_string()),
        ("scanner-abi".into(), SCANNER_ABI_VERSION.to_string()),
        ("lexer-version".into(), RUNTIME_VERSION.to_string()),
    ])
}

fn cst_key(snap: &Snapshot, p: &GrammarProgram) -> QueryKey {
    QueryKey::new(layer("A03"), vec![
        ("source-snapshot".into(), snap.id().as_str().to_string()),
        ("grammar-id".into(), p.grammar_id().as_str().to_string()),
        ("parser-version".into(), PARSER_ABI_VERSION.to_string()),
    ])
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
        Ok(SyntaxArtifactStore { store: ManagedStore::open(root)?, project: project.to_string() })
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
            project: Some(self.project.clone()),
        })?;
        Ok(art)
    }

    /// 装载语法程序：canonical → decode → **身份重验**（name+内容+版本 hash
    /// 与 key 的 grammar-id 不符 = 受控 miss/错误，不返回可执行程序）。
    pub fn get_grammar(&self, p_for_key: &GrammarProgram) -> Result<Option<GrammarProgram>, AdapterError> {
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

    // ---- A02 tokens-checkpoints ----

    pub fn put_tokens(&self, p: &GrammarProgram, snap: &Snapshot, tokens: &[Token]) -> Result<String, AdapterError> {
        let mut payload = Vec::with_capacity(tokens.len() * 26 + 4);
        payload.extend_from_slice(&(tokens.len() as u32).to_le_bytes());
        for t in tokens {
            payload.extend_from_slice(&t.token.to_le_bytes());
            payload.extend_from_slice(&t.start.to_le_bytes());
            payload.extend_from_slice(&t.end.to_le_bytes());
            payload.extend_from_slice(&t.line.to_le_bytes());
            payload.push(t.error as u8);
        }
        let art = self.store.put_artifact(PutRequest {
            layer: "A02",
            key: tokens_key(snap, p),
            kind: ac_artifact_probe::codec::ChunkKind::TokenStream,
            payload,
            edges: vec![],
            project: Some(self.project.clone()),
        })?;
        Ok(art)
    }

    pub fn get_tokens(&self, p: &GrammarProgram, snap: &Snapshot) -> Result<Option<Vec<Token>>, AdapterError> {
        let Some(loaded) = self.store.get_artifact("A02", &tokens_key(snap, p))? else {
            return Ok(None);
        };
        decode_tokens(&loaded.payload).map(Some).map_err(|e| AdapterError::Admissions(e))
    }

    // ---- A03 tolerant-cst（分页） ----

    /// 序列化整树为页 chunk + manifest，返回 manifest artifact id。
    /// edges：cst --required--> a02 artifact（调用方先 put_tokens 拿 id）。
    pub fn put_cst(
        &self,
        p: &GrammarProgram,
        snap: &Snapshot,
        tree: &CstTree,
        required_a02: &str,
    ) -> Result<String, AdapterError> {
        let pages = serialize_pages(tree);
        // 双 id 分离：edges 目标 = **artifact id**（store liveness 按 artifacts 表）；
        // manifest 页引用 = **存储 chunk id**（块地址）。framed_chunk_id(kind, raw)
        // = put 内部 frame 后的块 id。
        let mut page_ids: Vec<String> = Vec::with_capacity(pages.len());
        let mut page_chunks: Vec<String> = Vec::with_capacity(pages.len());
        let mut edges: Vec<(String, ac_artifact_probe::identity::RefKind)> = vec![(
            required_a02.to_string(),
            ac_artifact_probe::identity::RefKind::RequiredData,
        )];
        for (idx, page) in pages.iter().enumerate() {
            // 页间依赖链（页 i required 页 i-1）：让 GC 把页集合视为连通整体。
            if idx > 0 {
                edges.push((page_ids[idx - 1].clone(), ac_artifact_probe::identity::RefKind::RequiredData));
            }
            let key = QueryKey::new(layer("A03"), vec![
                ("source-snapshot".into(), snap.id().as_str().to_string()),
                ("grammar-id".into(), p.grammar_id().as_str().to_string()),
                ("parser-version".into(), PARSER_ABI_VERSION.to_string()),
                ("page".into(), idx.to_string()),
            ]);
            let art = self.store.put_artifact(PutRequest {
                layer: "A03",
                key,
                kind: ac_artifact_probe::codec::ChunkKind::CstPage,
                payload: page.clone(),
                edges: std::mem::take(&mut edges),
                project: Some(self.project.clone()),
            })?;
            page_ids.push(art);
            page_chunks.push(
                ac_artifact_probe::codec::framed_chunk_id(
                    ac_artifact_probe::codec::ChunkKind::CstPage,
                    page,
                )
                .to_string(),
            );
            // 重新建立下一页的基线 edges（页 i 的依赖在下一轮重建）。
            edges = vec![(page_ids[idx].clone(), ac_artifact_probe::identity::RefKind::RequiredData)];

        }
        // manifest：目录 + 根 + 页映射（JSON；serde 类型用 probe 泛型 chunk）。
        let manifest = build_manifest(p, snap, tree, &page_chunks, CST_PAGE_NODES);
        let mkey = QueryKey::new(layer("A03"), vec![
            ("source-snapshot".into(), snap.id().as_str().to_string()),
            ("grammar-id".into(), p.grammar_id().as_str().to_string()),
            ("parser-version".into(), PARSER_ABI_VERSION.to_string()),
            ("page".into(), "manifest".into()),
        ]);
        let art = self.store.put_artifact(PutRequest {
            layer: "A03",
            key: mkey,
            kind: ac_artifact_probe::codec::ChunkKind::CstManifest,
            payload: serde_json::to_vec(&manifest).map_err(|e| AdapterError::Admissions(e.to_string()))?,
            edges: vec![(page_ids[0].clone(), ac_artifact_probe::identity::RefKind::RequiredData)],
            project: Some(self.project.clone()),
        })?;
        self.store.register_root(&self.project, &art, "cst-manifest");
        Ok(art)
    }

    /// 打开分页视图（跨进程恢复：新 store 实例 + manifest key → CstView）。
    pub fn open_cst(&self, p: &GrammarProgram, snap: &Snapshot) -> Result<Option<CstView>, AdapterError> {
        let key = QueryKey::new(layer("A03"), vec![
            ("source-snapshot".into(), snap.id().as_str().to_string()),
            ("grammar-id".into(), p.grammar_id().as_str().to_string()),
            ("parser-version".into(), PARSER_ABI_VERSION.to_string()),
            ("page".into(), "manifest".into()),
        ]);
        let Some(loaded) = self.store.get_artifact("A03", &key)? else {
            return Ok(None);
        };
        let manifest: CstManifest = serde_json::from_slice(&loaded.payload)
            .map_err(|e| AdapterError::Admissions(format!("bad manifest: {e}")))?;
        if manifest.snapshot != snap.id().as_str() || manifest.grammar != p.grammar_id().as_str() {
            return Err(AdapterError::Admissions("manifest identity mismatch (snapshot/grammar)".into()));
        }
        if manifest.total_len != snap.len() {
            return Err(AdapterError::Admissions("manifest total_len != snapshot len".into()));
        }
        Ok(Some(CstView { store: &self.store, manifest, hot: std::cell::RefCell::new(PageCache::new(CST_HOT_BUDGET)) }))
    }
}

fn decode_tokens(mut b: &[u8]) -> Result<Vec<Token>, String> {
    if b.len() < 4 {
        return Err("token chunk truncated".into());
    }
    let n = u32::from_le_bytes(b[..4].try_into().unwrap()) as usize;
    b = &b[4..];
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
    Ok(out)
}

// ---- CST 分页序列化 ----

/// 节点区二进制：n u32 | nodes×(kind,start,len,first_child,child_count,expected u32,
/// flags u8, pad u32×? 布局 = 32B 定长) | edges×(node u32, field u32)。
/// child 引用是**页内局部下标**（跨页孩子不存在：页边界只在 decl 子树外切）。
fn serialize_pages(tree: &CstTree) -> Vec<Vec<u8>> {
    // 计算每个节点的"安全切点"：切页只能在 decl 边界（root 直接孩子）处——
    // decl 子树整页连续。root 节点放第 0 页开头。
    let total = tree.nodes.len();
    let mut pages = Vec::new();
    let mut node_idx = 0usize;
    // 第 0 页：root + 紧随其后的 decl 子树们。
    let root_edges: Vec<u32> = tree.children(tree.root).to_vec();
    let mut i = 1usize; // root = 0
    // 收集 root 直接孩子的子树区间 [c_lo, c]；root 自身 [0,0]。
    let mut groups: Vec<(usize, usize)> = Vec::new(); // (lo, hi) 节点区间
    groups.push((0, 1));
    for &child in &root_edges {
        let lo = tree.nodes[child as usize].decl != 0;
        let _ = lo;
        // root 直接孩子区间：用 node_lo（子树区间下界）——root 叶子（trivia）区间 [i,i+1)。
        if tree.nodes[child as usize].child_count == 0 {
            groups.push((child as usize, child as usize + 1));
        } else {
            let d = tree.decls.iter().rev().find(|d| d.node == child);
            let lo = d.map(|d| d.node_lo as usize).unwrap_or(child as usize);
            groups.push((lo.min(child as usize), child as usize + 1));
        }
        i += 1;
    }
    let _ = i;
    // 打包 groups 到页（≤CST_PAGE_NODES 节点；root 组单独页保证）。
    let mut cur: Vec<(usize, usize)> = Vec::new();
    let mut cur_nodes = 0usize;
    for g in groups {
        let gn = g.1 - g.0;
        if cur_nodes > 0 && cur_nodes + gn > CST_PAGE_NODES {
            pages.push(emit_page(tree, &cur));
            cur = Vec::new();
            cur_nodes = 0;
        }
        cur.push(g);
        cur_nodes += gn;
    }
    if !cur.is_empty() {
        pages.push(emit_page(tree, &cur));
    }
    node_idx = total;
    let _ = node_idx;
    pages
}

fn emit_page(tree: &CstTree, groups: &[(usize, usize)]) -> Vec<u8> {
    // 页内节点 = groups 区间顺序拼接；局部重映射 child id（全部在页内）。
    let mut local = std::collections::HashMap::new();
    let mut nodes_out: Vec<CstNode> = Vec::new();
    for &(lo, hi) in groups {
        for g in lo..hi {
            local.insert(g as u32, nodes_out.len() as u32);
            nodes_out.push(tree.nodes[g]);
        }
    }
    // 根边重映射 + 收集页内边。
    let mut edges: Vec<(u32, u32)> = Vec::new();
    for (li, nd) in nodes_out.iter().enumerate() {
        for k in 0..nd.child_count as usize {
            let old_child = tree.children[(nd.first_child as usize) + k];
            let lf = tree.child_fields[(nd.first_child as usize) + k];
            let Some(&new_child) = local.get(&old_child) else {
                // 跨页孩子：只在切点不是 decl 边界时发生——切点策略保证不达此处。
                panic!("page split crossed a subtree");
            };
            edges.push((new_child, lf));
            let _ = li;
        }
    }
    // 页内 first_child 重写：节点 child_count=0 的无操作；root（页内首节点）
    // 的跨组边放 manifest（页内置零），其余节点的边已按节点序在 edges 里
    // 连续排布——按累计值重写。
    {
        let mut cursor = 0u32;
        let mut first = true;
        for nd in nodes_out.iter_mut() {
            if first && nd.child_count > 0 {
                // root（或任何跨组父）——其边放 manifest，页内置零。
                nd.child_count = 0;
                nd.first_child = 0;
                first = false;
                continue;
            }
            first = false;
            let c = nd.child_count;
            nd.first_child = cursor;
            cursor += c;
        }
    }
    let mut out = Vec::with_capacity(8 + nodes_out.len() * 32 + edges.len() * 8);
    out.extend_from_slice(&(nodes_out.len() as u32).to_le_bytes());
    out.extend_from_slice(&(edges.len() as u32).to_le_bytes());
    for nd in &nodes_out {
        let n2 = *nd;
        out.extend_from_slice(&n2.kind.to_le_bytes());
        out.extend_from_slice(&n2.start.to_le_bytes());
        out.extend_from_slice(&n2.len.to_le_bytes());
        out.extend_from_slice(&n2.first_child.to_le_bytes());
        out.extend_from_slice(&n2.child_count.to_le_bytes());
        out.extend_from_slice(&n2.expected.to_le_bytes());
        out.extend_from_slice(&n2.decl.to_le_bytes());
        out.push(n2.flags);
        out.extend_from_slice(&[0u8; 3]);
    }
    for (n, f) in edges {
        out.extend_from_slice(&n.to_le_bytes());
        out.extend_from_slice(&f.to_le_bytes());
    }
    out
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CstManifest {
    pub snapshot: String,
    pub grammar: String,
    pub total_len: u64,
    pub diags: Vec<DiagEntry>,
    /// 树根（全局节点 id = 0）的孩子边（全局 id + field）。
    pub root_edges: Vec<(u32, u32)>,
    /// 页表：全局节点区间 → chunk id。
    pub pages: Vec<PageEntry>,
    /// 声明目录分页（PLAN-787：百万级声明目录不能整驻 manifest——每
    /// DECL_INDEX_PAGE 条一个 CstManifest chunk，按需取）。
    pub decl_index: Vec<PageEntry>,
    pub decl_count: u64,
    /// 兼容字段：非流式小树直接内联目录。
    pub decls: Vec<DeclEntry>,
}

/// 每目录页声明数。
pub const DECL_INDEX_PAGE: usize = 50_000;

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

fn build_manifest(p: &GrammarProgram, snap: &Snapshot, tree: &CstTree, page_ids: &[String], page_nodes: usize) -> CstManifest {
    // 页区间与 serialize_pages 的分组一致：root 组（0,1)+后续组累积——重算分组。
    let root_edges: Vec<u32> = tree.children(tree.root).to_vec();
    let mut groups: Vec<(usize, usize)> = Vec::new();
    groups.push((0, 1));
    for &child in &root_edges {
        if tree.nodes[child as usize].child_count == 0 {
            groups.push((child as usize, child as usize + 1));
        } else {
            let d = tree.decls.iter().rev().find(|d| d.node == child);
            let lo = d.map(|d| d.node_lo as usize).unwrap_or(child as usize);
            groups.push((lo.min(child as usize), child as usize + 1));
        }
    }
    let mut pages: Vec<PageEntry> = Vec::new();
    let mut decls: Vec<DeclEntry> = Vec::new();
    let mut cur_first: Option<u32> = None;
    let mut cur_count = 0u32;
    let mut page_idx = 0usize;
    let mut group_global: Vec<u32> = vec![0u32; groups.len()]; // 组起始全局 id 映射
    // 先线性分配全局 id（与 serialize 顺序一致），再切页。
    let mut next = 0u32;
    for (gi, &(lo, _hi)) in groups.iter().enumerate() {
        group_global[gi] = next;
        let _ = lo;
        next += 1; // 占位——下面按组内真实节点数累计
    }
    // 重算：组内节点数 = hi-lo。
    let mut next = 0u32;
    for (gi, &(lo, hi)) in groups.iter().enumerate() {
        group_global[gi] = next;
        next += (hi - lo) as u32;
        let _ = lo;
    }
    let mut cur_groups: Vec<usize> = Vec::new();
    let mut cur_nodes = 0usize;
    let mut flush = |cur_groups: &mut Vec<usize>, cur_nodes: &mut usize, pages: &mut Vec<PageEntry>, decls: &mut Vec<DeclEntry>, page_idx: &mut usize| {
        if cur_groups.is_empty() {
            return;
        }
        let first = group_global[cur_groups[0]];
        pages.push(PageEntry { chunk: page_ids[*page_idx].clone(), first_node: first, node_count: *cur_nodes as u32 });
        // decl 目录：本页各组对应 decl。
        for &gi in cur_groups.iter() {
            if gi == 0 {
                continue; // root 组：saturating_sub 会错配第一个 root 孩子。
            }
            let child = root_edges.get(gi - 1).copied().unwrap_or(tree.root);
            // 只有 root 直接孩子本身就是声明节点才入目录（根级 trivia/error 叶跳过）。
            if let Some(d) = tree.decls.iter().rev().find(|d| d.node == child) {
                decls.push(DeclEntry { origin: d.origin, node: d.node - d.node_lo + group_global[gi], node_lo: group_global[gi], page: *page_idx as u32 });
            }
        }
        *page_idx += 1;
        cur_groups.clear();
        *cur_nodes = 0;
    };
    for (gi, &(lo, hi)) in groups.iter().enumerate() {
        let gn = hi - lo;
        if cur_nodes > 0 && cur_nodes + gn > page_nodes {
            flush(&mut cur_groups, &mut cur_nodes, &mut pages, &mut decls, &mut page_idx);
        }
        cur_groups.push(gi);
        cur_nodes += gn;
    }
    flush(&mut cur_groups, &mut cur_nodes, &mut pages, &mut decls, &mut page_idx);
    // root 边：孩子全局 id = 组起始。
    let root_edges_mapped: Vec<(u32, u32)> = root_edges
        .iter()
        .enumerate()
        .map(|(i, &c)| {
            let g = i + 1;
            let lf = tree.child_fields[tree.nodes[tree.root as usize].first_child as usize + i];
            (group_global[g], lf)
        })
        .collect();
    CstManifest {
        snapshot: snap.id().as_str().to_string(),
        grammar: p.grammar_id().as_str().to_string(),
        total_len: snap.len(),
        diags: tree.diags.iter().map(|d| DiagEntry { message: d.message.clone(), start: d.start, end: d.end }).collect(),
        root_edges: root_edges_mapped,
        pages,
        decl_index: Vec::new(),
        decl_count: decls.len() as u64,
        decls,
    }
}

// ---- 页视图（按需取页，不反序列化整树） ----

struct PageCache {
    budget: usize,
    map: std::collections::HashMap<u32, (usize, Vec<CstPageData>)>, // page_idx -> (bytes, decoded)
    order: Vec<u32>,
    bytes: usize,
}

#[derive(Clone)]
struct CstPageData {
    nodes: Vec<CstNode>,
    edges: Vec<(u32, u32)>,
}

impl PageCache {
    fn new(budget: usize) -> Self {
        PageCache { budget, map: HashMap::new(), order: Vec::new(), bytes: 0 }
    }
    fn insert(&mut self, idx: u32, data: Vec<CstPageData>) {
        let sz: usize = data.iter().map(|p| p.nodes.len() * 32 + p.edges.len() * 8).sum();
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

pub struct CstView<'s> {
    store: &'s ManagedStore,
    manifest: CstManifest,
    hot: std::cell::RefCell<PageCache>,
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
            return self.manifest.decls.get(i).cloned().ok_or_else(|| AdapterError::Admissions(format!("decl index {i} out of range")));
        }
        // 目录分页：定位条目所在索引页并按需取。
        let bi = i / DECL_INDEX_PAGE;
        let pe = self.manifest.decl_index.get(bi)
            .ok_or_else(|| AdapterError::Admissions(format!("decl index {i} out of range")))?;
        let Some(payload) = self.store.read_chunk_gated(&pe.chunk)? else {
            return Err(AdapterError::Admissions(format!("decl index page {bi} missing")));
        };
        let (_k, payload) = ac_artifact_probe::codec::unframe_chunk(&payload)
            .map_err(|e| AdapterError::Admissions(format!("decl index page not framed: {e:?}")))?;
        let page: DeclIndexPage = serde_json::from_slice(&payload)
            .map_err(|e| AdapterError::Admissions(format!("bad decl index page: {e}")))?;
        page.decls.get(i % DECL_INDEX_PAGE).cloned().ok_or_else(|| AdapterError::Admissions(format!("decl {i} missing from index page")))
    }
    pub fn debug_page_chunk(&self, idx: usize) -> Option<String> {
        self.manifest.pages.get(idx).map(|p| p.chunk.clone())
    }
    pub fn debug_decl_entries(&self) -> Vec<(u64, u32, u32, u32)> {
        self.manifest.decls.iter().map(|d| (d.origin, d.node, d.node_lo, d.page)).collect()
    }
    pub fn total_len(&self) -> u64 {
        self.manifest.total_len
    }
    pub fn diag_count(&self) -> usize {
        self.manifest.diags.len()
    }
    /// 声明绝对范围（manifest 目录直读，零页读取）。
    pub fn decl_range(&self, i: usize) -> Result<(u64, u64), AdapterError> {
        let d = self.decl_entry(i)?;
        // 目录不含 span——需读页取节点；此处给出 origin + 页内查询。
        let page = self.page(d.page)?;
        let local = d.node - self.manifest.pages[d.page as usize].first_node;
        let nd = page.get(0).and_then(|p| p.nodes.get(local as usize))
            .ok_or_else(|| AdapterError::Admissions(format!("decl node {local} missing from page {}", d.page)))?;
        Ok((d.origin + nd.start as u64, d.origin + nd.start as u64 + nd.len as u64))
    }
    /// 单声明子树重建（仅取该声明所在页；活子树 = 根 BFS 可达——页内节点
    /// 为 BFS 序，不再区间连续）。
    pub fn decl_subtree(&self, i: usize) -> Result<CstTree, AdapterError> {
        let d = self.decl_entry(i)?;
        let first = self.manifest.pages[d.page as usize].first_node;
        let page = self.page(d.page)?;
        let p0 = page.get(0).ok_or_else(|| AdapterError::Admissions("empty page".into()))?;
        let root_local = (d.node - first) as usize;
        // BFS 收集可达节点（防坏 ref 环：seen 去重）。
        let mut seen = std::collections::HashSet::new();
        let mut order = Vec::new();
        let mut stack = vec![root_local];
        seen.insert(root_local);
        while let Some(n) = stack.pop() {
            order.push(n);
            let nd = p0.nodes[n];
            for k in 0..nd.child_count as usize {
                let (c, _) = p0.edges[(nd.first_child as usize) + k];
                let cl = c as usize;
                if cl < p0.nodes.len() && seen.insert(cl) {
                    stack.push(cl);
                }
            }
        }
        order.sort_unstable(); // 页内序 = 文档序（BFS 发射保持）
        let mut remap = HashMap::new();
        let mut nodes = Vec::with_capacity(order.len());
        for &li in &order {
            remap.insert(li as u32, nodes.len() as u32);
            let mut nd = p0.nodes[li];
            nodes.push(nd);
        }
        let mut children = Vec::new();
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
                    children.push(nc);
                    fields.push(f);
                }
            }
            cursor += nd.child_count;
        }
        let root = remap[&(root_local as u32)];
        Ok(CstTree {
            nodes,
            children,
            child_fields: fields,
            root,
            decls: vec![ac_syntax_rt::cst::DeclUnit { origin: d.origin, node: root, node_lo: 0 }],
            diags: self.manifest_diags_for(d.origin),
            total_len: self.manifest.total_len,
        })
    }

    fn manifest_diags_for(&self, _origin: u64) -> Vec<ac_syntax_rt::cst::SynDiag> {
        // 诊断按声明归属过滤成本高；视图层保留全量诊断的轻量投影（消息可查）。
        self.manifest.diags.iter().map(|d| ac_syntax_rt::cst::SynDiag { message: d.message.clone(), start: d.start, end: d.end }).collect()
    }
    fn page(&self, idx: u32) -> Result<Vec<CstPageData>, AdapterError> {
        if let Some((_, v)) = self.hot.borrow().map.get(&idx) {
            return Ok(v.clone());
        }
        let pe = self.manifest.pages.get(idx as usize)
            .ok_or_else(|| AdapterError::Admissions(format!("page {idx} missing from manifest")))?;
        let Some(payload) = self.store.read_chunk_gated(&pe.chunk)? else {
            return Err(AdapterError::Admissions(format!("page chunk {} missing (CAS miss)", &pe.chunk[..8.min(pe.chunk.len())])));
        };
        let (_kind, payload) = ac_artifact_probe::codec::unframe_chunk(&payload)
            .map_err(|e| AdapterError::Admissions(format!("page chunk not framed: {e:?}")))?;
        let decoded = decode_page(&payload)?;
        self.hot.borrow_mut().insert(idx, decoded.clone());
        Ok(decoded)
    }
}

fn decode_page(b: &[u8]) -> Result<Vec<CstPageData>, AdapterError> {
    if b.len() < 8 {
        return Err(AdapterError::Admissions("page truncated".into()));
    }
    let n = u32::from_le_bytes(b[0..4].try_into().unwrap()) as usize;
    let e = u32::from_le_bytes(b[4..8].try_into().unwrap()) as usize;
    if b.len() < 8 + n * 32 + e * 8 {
        return Err(AdapterError::Admissions("page length mismatch".into()));
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
        edges.push((u32::from_le_bytes(c[0..4].try_into().unwrap()), u32::from_le_bytes(c[4..8].try_into().unwrap())));
    }
    // 边索引一致性（first_child+count 越界 = 坏 offset/ref）。
    for nd in &nodes {
        let fc = nd.first_child as usize;
        let ec = nd.child_count as usize;
        if fc + ec > e {
            return Err(AdapterError::Admissions(format!("edge range {fc}+{ec} > {e} (bad ref)")));
        }
    }
    Ok(vec![CstPageData { nodes, edges }])
}

// ---- 流式构建（V-11：64MiB 级源不驻留整树） ----

/// 分块流式 CST 生产者：每块独立 parse→分页→入库，内存中只保留
/// 目录/页 chunk 索引（字节量 O(声明数)），从不持有整棵树。
pub struct StreamingCstBuilder<'a> {
    sas: &'a SyntaxArtifactStore,
    p: &'a GrammarProgram,
    scanner: CompiledScanner,
    prefix_len: u64,
    prev_page_art: Option<String>,
    page_chunks: Vec<String>,
    decls: Vec<DeclEntry>,
    pages: Vec<PageEntry>,
    next_global: u32,
    snap_hash_parts: Vec<Vec<u8>>,
    diags: Vec<DiagEntry>,
}

impl<'a> StreamingCstBuilder<'a> {
    pub fn new(sas: &'a SyntaxArtifactStore, p: &'a GrammarProgram) -> Self {
        StreamingCstBuilder {
            sas,
            p,
            scanner: CompiledScanner::compile(p).expect("scanner"),
            prefix_len: 0,
            prev_page_art: None,
            page_chunks: Vec::new(),
            decls: Vec::new(),
            pages: Vec::new(),
            next_global: 0,
            snap_hash_parts: Vec::new(),
            diags: Vec::new(),
        }
    }

    /// 推入一个块（块间在行边界切分；块内声明**批页**——按
    /// STREAM_PAGE_NODES 活节点/页打包多声明。活子树 = 从声明根 BFS 可达节点
    /// （区间 [node_lo..node] 含回溯孤儿，实测单区间可达数百万节点）。
    pub fn push_chunk(&mut self, chunk: &str) -> Result<(), AdapterError> {
        let store_rc = Rc::new(PageStore::new());
        let snap = Snapshot::from_text(&store_rc, chunk);
        let out = ac_syntax_rt::parse::parse_snapshot(self.p, &self.scanner, &snap, &Budget::default())
            .map_err(AdapterError::Rt)?;
        let tree = &out.tree;
        let roots: Vec<(u32, u64)> = tree.decls.iter().map(|d| (d.node, d.origin)).collect();
        let mut batch: Vec<(u32, u64)> = Vec::new();
        let mut batch_nodes = 0usize;
        for &(root, origin) in roots.iter() {
            let n = live_subtree_len(tree, root);
            if batch_nodes > 0 && batch_nodes + n > STREAM_PAGE_NODES {
                self.flush_batch(tree, &batch);
                batch.clear();
                batch_nodes = 0;
            }
            batch.push((root, origin));
            batch_nodes += n;
        }
        if !batch.is_empty() {
            self.flush_batch(tree, &batch);
        }
        for dg in &tree.diags {
            self.diags.push(DiagEntry { message: dg.message.clone(), start: self.prefix_len + dg.start, end: self.prefix_len + dg.end });
        }
        let mut part = snap.id().as_str().as_bytes().to_vec();
        part.push(0);
        self.snap_hash_parts.push(part);
        self.prefix_len += chunk.len() as u64;
        Ok(())
    }

    /// 批活子树页发射 + 入库 + 目录登记。
    fn flush_batch(&mut self, tree: &CstTree, batch: &[(u32, u64)]) {
        let payload = emit_subtrees_as_page(tree, batch);
        let idx = self.page_chunks.len();
        let mut edges: Vec<(String, ac_artifact_probe::identity::RefKind)> = Vec::new();
        if let Some(pv) = &self.prev_page_art {
            edges.push((pv.clone(), ac_artifact_probe::identity::RefKind::RequiredData));
        }
        let key = QueryKey::new(layer("A03"), vec![
            ("source-snapshot".into(), self.stream_snapshot_component()),
            ("grammar-id".into(), self.p.grammar_id().as_str().to_string()),
            ("parser-version".into(), PARSER_ABI_VERSION.to_string()),
            ("page".into(), idx.to_string()),
        ]);
        let art = self.sas.inner().put_artifact(PutRequest {
            layer: "A03",
            key,
            kind: ac_artifact_probe::codec::ChunkKind::CstPage,
            payload: payload.clone(),
            edges,
            project: None,
        })
        .expect("page put");
        let chunk_id = ac_artifact_probe::codec::framed_chunk_id(
            ac_artifact_probe::codec::ChunkKind::CstPage,
            &payload,
        )
        .to_string();
        // 目录：每声明在页内的（首节点下标, 根下标）由发射器对齐返回。
        let layout = subtree_layout(tree, batch);
        let base = self.next_global;
        let mut node_count_total = 0u32;
        for (i, &(root, origin)) in batch.iter().enumerate() {
            let (first, count) = layout[i];
            self.decls.push(DeclEntry {
                origin: self.prefix_len + origin,
                node: base + first,
                node_lo: base + first,
                page: idx as u32,
            });
            node_count_total += count;
        }
        self.pages.push(PageEntry { chunk: chunk_id.clone(), first_node: base, node_count: node_count_total });
        self.next_global += node_count_total;
        self.prev_page_art = Some(art);
        self.page_chunks.push(chunk_id);
    }

    fn stream_snapshot_component(&self) -> String {
        // 流式源身份 = 各块 snapshot id 的域 hash（无单一大 Snapshot）。
        let joined: Vec<u8> = self.snap_hash_parts.concat();
        ac_artifact_probe::identity::domain_hash("snap-stream", &[&joined])
    }

    /// 收尾：写目录页 + manifest，返回 manifest artifact id。
    pub fn finish(&mut self, total_len: u64) -> Result<String, AdapterError> {
        let decl_count = self.decls.len() as u64;
        // 目录分页：每 DECL_INDEX_PAGE 条一个 chunk。
        let mut decl_index: Vec<PageEntry> = Vec::new();
        let decls = std::mem::take(&mut self.decls);
        for (bi, chunk_decls) in decls.chunks(DECL_INDEX_PAGE).enumerate() {
            let page_json = DeclIndexPage { first: (bi as u64) * DECL_INDEX_PAGE as u64, decls: chunk_decls.to_vec() };
            let payload = serde_json::to_vec(&page_json).map_err(|e| AdapterError::Admissions(e.to_string()))?;
            let key = QueryKey::new(layer("A03"), vec![
                ("source-snapshot".into(), self.stream_snapshot_component()),
                ("grammar-id".into(), self.p.grammar_id().as_str().to_string()),
                ("parser-version".into(), PARSER_ABI_VERSION.to_string()),
                ("page".into(), format!("declidx-{bi}")),
            ]);
            self.sas.inner().put_artifact(PutRequest {
                layer: "A03",
                key,
                kind: ac_artifact_probe::codec::ChunkKind::CstManifest,
                payload: payload.clone(),
                edges: self.prev_page_art.clone().map(|a| vec![(a, ac_artifact_probe::identity::RefKind::RequiredData)]).unwrap_or_default(),
                project: None,
            })?;
            let cid = ac_artifact_probe::codec::framed_chunk_id(
                ac_artifact_probe::codec::ChunkKind::CstManifest,
                &payload,
            )
            .to_string();
            decl_index.push(PageEntry { chunk: cid, first_node: (bi * DECL_INDEX_PAGE) as u32, node_count: chunk_decls.len() as u32 });
        }
        let manifest = CstManifest {
            snapshot: self.stream_snapshot_component(),
            grammar: self.p.grammar_id().as_str().to_string(),
            total_len,
            diags: std::mem::take(&mut self.diags),
            root_edges: Vec::new(), // 流式树无单根；视图经声明目录查询
            pages: std::mem::take(&mut self.pages),
            decl_index,
            decl_count,
            decls: Vec::new(),
        };
        let mkey = QueryKey::new(layer("A03"), vec![
            ("source-snapshot".into(), manifest.snapshot.clone()),
            ("grammar-id".into(), self.p.grammar_id().as_str().to_string()),
            ("parser-version".into(), PARSER_ABI_VERSION.to_string()),
            ("page".into(), "manifest".into()),
        ]);
        let payload = serde_json::to_vec(&manifest).map_err(|e| AdapterError::Admissions(e.to_string()))?;
        let art = self.sas.inner().put_artifact(PutRequest {
            layer: "A03",
            key: mkey,
            kind: ac_artifact_probe::codec::ChunkKind::CstManifest,
            payload,
            edges: self.prev_page_art.clone().map(|a| vec![(a, ac_artifact_probe::identity::RefKind::RequiredData)]).unwrap_or_default(),
            project: None,
        })?;
        self.sas.inner().register_root("stream", &art, "cst-manifest");
        Ok(art)
    }

    pub fn stream_snapshot_id(&self) -> String {
        self.stream_snapshot_component()
    }
}

/// 多区间单页发射（节点 copy + 页内边重映射；start 保持块内相对值）。
fn emit_ranges_as_page(tree: &CstTree, ranges: &[(usize, usize, u64)]) -> Vec<u8> {
    let mut local = HashMap::new();
    let mut nodes_out: Vec<CstNode> = Vec::new();
    for &(lo, hi, _) in ranges {
        for g in lo..=hi {
            local.insert(g as u32, nodes_out.len() as u32);
            nodes_out.push(tree.nodes[g]);
        }
    }
    let mut edges: Vec<(u32, u32)> = Vec::new();
    for nd in &nodes_out {
        for k in 0..nd.child_count as usize {
            let old_child = tree.children[(nd.first_child as usize) + k];
            let lf = tree.child_fields[(nd.first_child as usize) + k];
            if let Some(&nc) = local.get(&old_child) {
                edges.push((nc, lf));
            }
        }
    }
    let mut cursor = 0u32;
    for nd in nodes_out.iter_mut() {
        let c = nd.child_count;
        nd.first_child = cursor;
        cursor += c;
    }
    let mut out = Vec::with_capacity(8 + nodes_out.len() * 32 + edges.len() * 8);
    out.extend_from_slice(&(nodes_out.len() as u32).to_le_bytes());
    out.extend_from_slice(&(edges.len() as u32).to_le_bytes());
    for n2 in &nodes_out {
        out.extend_from_slice(&n2.kind.to_le_bytes());
        out.extend_from_slice(&n2.start.to_le_bytes());
        out.extend_from_slice(&n2.len.to_le_bytes());
        out.extend_from_slice(&n2.first_child.to_le_bytes());
        out.extend_from_slice(&n2.child_count.to_le_bytes());
        out.extend_from_slice(&n2.expected.to_le_bytes());
        out.extend_from_slice(&n2.decl.to_le_bytes());
        out.push(n2.flags);
        out.extend_from_slice(&[0u8; 3]);
    }
    for (n, f) in edges {
        out.extend_from_slice(&n.to_le_bytes());
        out.extend_from_slice(&f.to_le_bytes());
    }
    out
}

#[allow(dead_code)]
/// 区间单页发射（节点 copy + 页内边重映射；start 保持块内相对值）。
fn emit_range_as_page(tree: &CstTree, lo: usize, hi: usize) -> Vec<u8> {
    let mut local = HashMap::new();
    let mut nodes_out: Vec<CstNode> = Vec::new();
    for g in lo..=hi {
        local.insert(g as u32, nodes_out.len() as u32);
        nodes_out.push(tree.nodes[g]);
    }
    let mut edges: Vec<(u32, u32)> = Vec::new();
    for nd in &nodes_out {
        for k in 0..nd.child_count as usize {
            let old_child = tree.children[(nd.first_child as usize) + k];
            let lf = tree.child_fields[(nd.first_child as usize) + k];
            if let Some(&nc) = local.get(&old_child) {
                edges.push((nc, lf));
            }
            // 越界引用（孤儿区间）静默丢弃——目录查询不走孤儿。
        }
    }
    // 重写 first_child。
    let mut cursor = 0u32;
    for nd in nodes_out.iter_mut() {
        let c = nd.child_count;
        nd.first_child = cursor;
        cursor += c;
    }
    let mut out = Vec::with_capacity(8 + nodes_out.len() * 32 + edges.len() * 8);
    out.extend_from_slice(&(nodes_out.len() as u32).to_le_bytes());
    out.extend_from_slice(&(edges.len() as u32).to_le_bytes());
    for n2 in &nodes_out {
        out.extend_from_slice(&n2.kind.to_le_bytes());
        out.extend_from_slice(&n2.start.to_le_bytes());
        out.extend_from_slice(&n2.len.to_le_bytes());
        out.extend_from_slice(&n2.first_child.to_le_bytes());
        out.extend_from_slice(&n2.child_count.to_le_bytes());
        out.extend_from_slice(&n2.expected.to_le_bytes());
        out.extend_from_slice(&n2.decl.to_le_bytes());
        out.push(n2.flags);
        out.extend_from_slice(&[0u8; 3]);
    }
    for (n, f) in edges {
        out.extend_from_slice(&n.to_le_bytes());
        out.extend_from_slice(&f.to_le_bytes());
    }
    out
}

impl SyntaxArtifactStore {
    /// 流式源的视图打开（manifest key 用 stream snapshot 域 hash）。
    pub fn open_cst_stream(&self, p: &GrammarProgram, stream_id: &str, total_len: u64) -> Result<Option<CstView<'_>>, AdapterError> {
        let key = QueryKey::new(layer("A03"), vec![
            ("source-snapshot".into(), stream_id.to_string()),
            ("grammar-id".into(), p.grammar_id().as_str().to_string()),
            ("parser-version".into(), PARSER_ABI_VERSION.to_string()),
            ("page".into(), "manifest".into()),
        ]);
        let Some(loaded) = self.store.get_artifact("A03", &key)? else {
            return Ok(None);
        };
        let manifest: CstManifest = serde_json::from_slice(&loaded.payload)
            .map_err(|e| AdapterError::Admissions(format!("bad manifest: {e}")))?;
        if manifest.grammar != p.grammar_id().as_str() {
            return Err(AdapterError::Admissions("manifest grammar mismatch".into()));
        }
        if manifest.total_len != total_len {
            return Err(AdapterError::Admissions("manifest total_len mismatch".into()));
        }
        Ok(Some(CstView { store: &self.store, manifest, hot: std::cell::RefCell::new(PageCache::new(CST_HOT_BUDGET)) }))
    }
}

impl<'s> CstView<'s> {
    /// 受管页缓存当前字节（预算计量）。
    pub fn debug_hot_bytes(&self) -> usize {
        self.hot.borrow().bytes
    }
}

/// 声明根的活子树节点数（BFS 去重；回溯孤儿不计）。
fn live_subtree_len(tree: &CstTree, root: u32) -> usize {
    let mut seen = std::collections::HashSet::new();
    let mut stack = vec![root];
    seen.insert(root);
    while let Some(n) = stack.pop() {
        for &c in tree.children(n) {
            if seen.insert(c) {
                stack.push(c);
            }
        }
    }
    seen.len()
}

/// 每声明的（页内首节点下标, 活节点数）——与 emit_subtrees_as_page 的
/// 节点顺序一致（BFS 序）。
fn subtree_layout(tree: &CstTree, roots: &[(u32, u64)]) -> Vec<(u32, u32)> {
    let mut out = Vec::with_capacity(roots.len());
    let mut seen_all = std::collections::HashSet::new();
    let mut first = 0u32;
    for &(root, _) in roots {
        let mut seen = std::collections::HashSet::new();
        let mut stack = vec![root];
        seen.insert(root);
        while let Some(n) = stack.pop() {
            for &c in tree.children(n) {
                if seen.insert(c) && seen_all.insert(c) {
                    stack.push(c);
                }
            }
        }
        out.push((first, seen.len() as u32));
        first += seen.len() as u32;
    }
    out
}

/// 多活子树页发射：BFS 序节点 + 页内边重映射；start 保持块内相对值。
fn emit_subtrees_as_page(tree: &CstTree, roots: &[(u32, u64)]) -> Vec<u8> {
    let mut local = HashMap::new();
    let mut nodes_out: Vec<CstNode> = Vec::new();
    let mut edge_specs: Vec<(u32, u32, u32)> = Vec::new(); // (parent_local, old_child, field)
    let mut seen_all = std::collections::HashSet::new();
    for &(root, _) in roots {
        let mut order = Vec::new();
        let mut seen = std::collections::HashSet::new();
        let mut stack = vec![root];
        seen.insert(root);
        while let Some(n) = stack.pop() {
            order.push(n);
            for &c in tree.children(n).iter().rev() {
                if seen.insert(c) && seen_all.insert(c) {
                    stack.push(c);
                }
            }
        }
        for old_id in order {
            local.insert(old_id, nodes_out.len() as u32);
            let mut nd = tree.nodes[old_id as usize];
            let fc = nd.first_child;
            let cc = nd.child_count;
            edge_specs.reserve(cc as usize);
            for k in 0..cc as usize {
                edge_specs.push((nodes_out.len() as u32, tree.children[(fc as usize) + k], tree.child_fields[(fc as usize) + k]));
            }
            // 占位：first_child 重写需在全部 local 建立后进行。
            nd.first_child = u32::MAX;
            nodes_out.push(nd);
        }
    }
    // 重写 first_child + 生成页内边。
    let mut first_by_node = vec![0u32; nodes_out.len()];
    let mut cursor = 0u32;
    for (i, _) in nodes_out.iter().enumerate() {
        first_by_node[i] = cursor;
        cursor += 0; // 占位
    }
    // 按 parent 分组计数。
    let mut counts = vec![0u32; nodes_out.len()];
    for (p, _, _) in &edge_specs {
        counts[*p as usize] += 1;
    }
    let mut offs = vec![0u32; nodes_out.len()];
    let mut acc = 0u32;
    for i in 0..nodes_out.len() {
        offs[i] = acc;
        acc += counts[i];
    }
    let mut edges: Vec<(u32, u32)> = vec![(0, 0); edge_specs.len()];
    let mut fill = offs.clone();
    for (p, old_child, f) in edge_specs {
        edges[fill[p as usize] as usize] = (local[&old_child], f);
        fill[p as usize] += 1;
    }
    for (i, nd) in nodes_out.iter_mut().enumerate() {
        nd.first_child = offs[i];
        nd.child_count = counts[i];
    }
    let mut out = Vec::with_capacity(8 + nodes_out.len() * 32 + edges.len() * 8);
    out.extend_from_slice(&(nodes_out.len() as u32).to_le_bytes());
    out.extend_from_slice(&(edges.len() as u32).to_le_bytes());
    for n2 in &nodes_out {
        out.extend_from_slice(&n2.kind.to_le_bytes());
        out.extend_from_slice(&n2.start.to_le_bytes());
        out.extend_from_slice(&n2.len.to_le_bytes());
        out.extend_from_slice(&n2.first_child.to_le_bytes());
        out.extend_from_slice(&n2.child_count.to_le_bytes());
        out.extend_from_slice(&n2.expected.to_le_bytes());
        out.extend_from_slice(&n2.decl.to_le_bytes());
        out.push(n2.flags);
        out.extend_from_slice(&[0u8; 3]);
    }
    for (n, f) in edges {
        out.extend_from_slice(&n.to_le_bytes());
        out.extend_from_slice(&f.to_le_bytes());
    }
    out
}

