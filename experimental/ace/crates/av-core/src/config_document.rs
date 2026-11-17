//! PLAN-982（B-CONFIG-I）：配置文档**宿主侧求值原语**（`config.document/1`）。
//!
//! 设计（`docs/design/ace-config-document-evaluation.md` §3.1/§3.2）：表达式、
//! `if`/`for`/模板由共享 ACE 指令执行；本模块只承载**值和容器**原语（建节点/
//! 放值/追加/收尾），不解释 AST、不选业务分支。语义与 SDK 侧已冻结的
//! `ConfigDocumentBuilder`（`crates/auto-lang-core/src/ace_config.rs`）同纪律：
//! - 每份文档从 root 容器开始，语句按源顺序；
//! - 句柄带种类（node/obj/list/value），每次读写都核 kind，错用 = 具名负码；
//! - 节点/属性/实参/孩子顺序与重复保留；重复 key 的写入按旧 set_prop 语义
//!   **原位替换**（可观察顺序不变）；
//! - 一父树纪律：节点重复 attach、把自己当孩子/实参、收尾时存在环 → 具名拒；
//! - 预算逐次计费（值节点数/owned 字节），超限 = 具名 `HostError::Resource`；
//! - 退出前导出为**不借运行帧**的 owned 快照（节点/对象/列表/值）。
//!
//! 会话是**单线程显式打开**的：宿主在求值前 `open_session`，求值后
//! `close_session` 取快照；没有会话时任何配置原语 = 具名拒绝（不是空操作）。

use crate::av::CancelFlag;
use crate::contract::Value;
use crate::host::{AdmissionError, HostError};
use crate::mir::{MirHostImport, MirModule, MirType};

/// 配置原语类别/能力键（与 auto-lang-core 的 `ace_config` 侧同一字符串）。
pub const CATEGORY: &str = "config.document/1";
pub const CAPABILITY: &str = "config.document/1";
pub const MAX_CONFIG_IMPORTS: usize = 24;
pub const MAX_CALL_BYTES: usize = 65_536;
pub const MAX_RUN_BYTES: usize = 1_048_576;

/// 原语负码（确定性；从不 panic、不静默当成 0）。
pub const ERR_NO_SESSION: i32 = -1;
pub const ERR_INVALID_HANDLE: i32 = -2;
pub const ERR_KIND: i32 = -3;
pub const ERR_BUDGET: i32 = -4;
pub const ERR_ATTACHED: i32 = -5;
pub const ERR_CYCLE: i32 = -6;
pub const ERR_FINISHED: i32 = -7;
pub const ERR_INVALID_VALUE: i32 = -8;
pub const ERR_DUPLICATE_ID: i32 = -9;

/// 值种类（句柄/参数种类；也决定源级重载）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ValKind {
    I32,
    Bool,
    Str,
    Char,
    Node,
    Obj,
    List,
}

impl ValKind {
    fn of_word(word: &str) -> Option<ValKind> {
        Some(match word {
            "i32" => ValKind::I32,
            "bool" => ValKind::Bool,
            "str" => ValKind::Str,
            "char" => ValKind::Char,
            "node" => ValKind::Node,
            "obj" => ValKind::Obj,
            "list" => ValKind::List,
            _ => return None,
        })
    }

    fn mir(self) -> MirType {
        match self {
            ValKind::I32 | ValKind::Node | ValKind::Obj | ValKind::List => MirType::I32,
            ValKind::Bool => MirType::Bool,
            ValKind::Str => MirType::StrView,
            ValKind::Char => MirType::Char,
        }
    }
}

/// 写入目标容器种类。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PutTarget {
    Node,
    Obj,
    List,
}

/// 一条原语的结构（由符号解析得到；符号是唯一真源）。
#[derive(Clone, Copy, Debug)]
pub enum ConfigSpec {
    /// `root()` → 根容器句柄。
    Root,
    /// `node(name)` → 新节点句柄。
    Node,
    ObjNew,
    ListNew,
    /// `id.name(node, id)`／`id.text(node, id)`。
    IdName,
    IdText,
    /// 位置实参 `arg.<kind>(node, value)`。
    ArgPos(ValKind),
    /// 具名实参 `arg-named.<kind>(node, key, value)`。
    ArgNamed(ValKind),
    /// 属性写入 `put.<kind>`／`obj-put.<kind>`／`list-push.<kind>`。
    Put(PutTarget, ValKind),
    /// 子节点追加 `append(parent, child)`。
    Append,
    /// `super-put(dst_node, src_obj, key)`：把 `src_obj` 的同名字段**原样搬进**
    /// `dst_node`（任何值种类）。物化侧用它实现"对象字段进文档"而不需要
    /// Str 返回面（`heap: kernel.heap`）。
    SuperPut,
    /// 收尾 `finish()`。
    Finish,
}

impl ConfigSpec {
    /// 精确参数表（与 auto-lang-core `config_ops` 目录同一契约）。
    pub fn params(self) -> Vec<MirType> {
        match self {
            ConfigSpec::Root | ConfigSpec::ObjNew | ConfigSpec::ListNew | ConfigSpec::Finish => {
                Vec::new()
            }
            ConfigSpec::Node => vec![MirType::StrView],
            ConfigSpec::IdName | ConfigSpec::IdText => vec![MirType::I32, MirType::StrView],
            ConfigSpec::ArgPos(k) => vec![MirType::I32, k.mir()],
            ConfigSpec::ArgNamed(k) => vec![MirType::I32, MirType::StrView, k.mir()],
            ConfigSpec::Put(PutTarget::List, k) => vec![MirType::I32, k.mir()],
            ConfigSpec::Put(_, k) => vec![MirType::I32, MirType::StrView, k.mir()],
            ConfigSpec::Append => vec![MirType::I32, MirType::I32],
            ConfigSpec::SuperPut => vec![MirType::I32, MirType::I32, MirType::StrView],
        }
    }

    pub fn result(self) -> MirType {
        MirType::I32
    }
}

/// 该符号是否是配置文档原语。
pub fn is_config_symbol(sym: &str) -> bool {
    sym.starts_with("auto.config.document.")
}

/// 符号 → 结构（未知/拼写错误 = None，绝不静默接受）。
pub fn spec_of(sym: &str) -> Option<ConfigSpec> {
    let rest = sym.strip_prefix("auto.config.document.")?;
    let rest = rest.strip_suffix(".v1")?;
    // 形如 `<op>`／`<op>.<kind>`；`obj-put`/`list-push`/`arg-named` 是连字符名字。
    if rest == "super-put" {
        return Some(ConfigSpec::SuperPut);
    }
    if let Some(rest) = rest.strip_prefix("obj-put.") {
        return Some(ConfigSpec::Put(PutTarget::Obj, ValKind::of_word(rest)?));
    }
    if let Some(rest) = rest.strip_prefix("list-push.") {
        return Some(ConfigSpec::Put(PutTarget::List, ValKind::of_word(rest)?));
    }
    if let Some(rest) = rest.strip_prefix("arg-named.") {
        return Some(ConfigSpec::ArgNamed(ValKind::of_word(rest)?));
    }
    if let Some(rest) = rest.strip_prefix("arg.") {
        return Some(ConfigSpec::ArgPos(ValKind::of_word(rest)?));
    }
    if let Some(rest) = rest.strip_prefix("put.") {
        return Some(ConfigSpec::Put(PutTarget::Node, ValKind::of_word(rest)?));
    }
    Some(match rest {
        "root" => ConfigSpec::Root,
        "node" => ConfigSpec::Node,
        "obj-new" => ConfigSpec::ObjNew,
        "list-new" => ConfigSpec::ListNew,
        "id.name" => ConfigSpec::IdName,
        "id.text" => ConfigSpec::IdText,
        "append" => ConfigSpec::Append,
        "finish" => ConfigSpec::Finish,
        _ => return None,
    })
}

/// 降级侧声明：符号必须可解析，签名由目录给出（与源级重载同一行）。
pub fn declaration(symbol: &str, sig: u32) -> Result<MirHostImport, AdmissionError> {
    spec_of(symbol).ok_or_else(|| AdmissionError {
        code: "config.unknown-symbol",
        message: "unknown configuration document primitive".into(),
    })?;
    Ok(MirHostImport {
        category: CATEGORY.into(),
        symbol: symbol.into(),
        sig,
        capability: CAPABILITY.into(),
        max_call_bytes: MAX_CALL_BYTES as u32,
    })
}

/// 准入校验：类别/能力/预算/签名类型逐项核（不是只看符号前缀）。
pub fn validate_import(module: &MirModule, h: &MirHostImport) -> Result<(), AdmissionError> {
    let spec = spec_of(&h.symbol).ok_or_else(|| AdmissionError {
        code: "config.unknown-symbol",
        message: "unknown configuration document primitive".into(),
    })?;
    let sig = module.sig_of(h.sig).ok_or_else(|| AdmissionError {
        code: "config.signature",
        message: "configuration primitive signature is not a function type".into(),
    })?;
    let want = spec.params();
    let ok = h.category == CATEGORY
        && h.capability == CAPABILITY
        && h.max_call_bytes == MAX_CALL_BYTES as u32
        && sig.params.len() == want.len()
        && sig
            .params
            .iter()
            .zip(want.iter())
            .all(|(a, b)| module.types.get(*a as usize) == Some(b))
        && sig
            .result
            .checked_sub(0)
            .and_then(|_| module.types.get(sig.result as usize))
            .map(|t| *t == spec.result())
            .unwrap_or(false);
    if ok {
        Ok(())
    } else {
        Err(AdmissionError {
            code: "config.signature",
            message: "configuration primitive category/signature/budget tuple mismatch".into(),
        })
    }
}

// ---------------------------------------------------------------------------
// 会话与构建器
// ---------------------------------------------------------------------------

/// 求值预算（宿主请求预算；不是语言永久上限）。
#[derive(Clone, Copy, Debug)]
pub struct DocumentLimits {
    pub value_nodes: usize,
    pub owned_bytes: usize,
}

impl Default for DocumentLimits {
    fn default() -> Self {
        Self {
            value_nodes: 65_536,
            owned_bytes: 16 * 1024 * 1024,
        }
    }
}

/// 节点身份（裸名/字符串两形，均保原语义）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OwnedId {
    Name(String),
    Text(String),
}

/// owned 值（不借运行帧/句柄）。
#[derive(Clone, Debug, PartialEq)]
pub enum OwnedValue {
    I32(i32),
    Bool(bool),
    Char(char),
    Text(String),
    Node(Box<OwnedNode>),
    Obj(Vec<(String, OwnedValue)>),
    List(Vec<OwnedValue>),
}

/// owned 节点（参数/属性/孩子顺序与重复保留）。
#[derive(Clone, Debug, PartialEq)]
pub struct OwnedNode {
    pub name: String,
    pub id: Option<OwnedId>,
    pub args: Vec<OwnedValue>,
    pub props: Vec<(String, OwnedValue)>,
    pub children: Vec<OwnedNode>,
}

/// 一份文档的 owned 快照。
#[derive(Clone, Debug, PartialEq)]
pub struct OwnedDocument {
    pub root: OwnedNode,
    pub cost: DocumentCost,
}

/// 真实代价事实（不是自报布尔）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DocumentCost {
    pub nodes: usize,
    pub values: usize,
    pub owned_bytes: usize,
    /// Configuration primitive dispatches executed (real count, not an estimate).
    pub callees: u64,
}

enum Entry {
    Node(Box<NodeData>),
    Obj(Box<ObjData>),
    List(Box<ListData>),
    Value(StoredValue),
}

struct NodeData {
    name: String,
    id: Option<OwnedId>,
    args: Vec<u32>,
    props: Vec<(String, u32)>,
    children: Vec<u32>,
    attached: bool,
}

struct ObjData {
    fields: Vec<(String, u32)>,
}

struct ListData {
    items: Vec<u32>,
}

#[derive(Clone, Copy)]
enum StoredValue {
    I32(i32),
    Bool(bool),
    Char(char),
    Text(u32),
    Node(u32),
    Obj(u32),
    List(u32),
}

struct Session {
    entries: Vec<Entry>,
    root: Option<u32>,
    finished: bool,
    limits: DocumentLimits,
    nodes: usize,
    values: usize,
    owned_bytes: usize,
    callees: u64,
    texts: Vec<String>,
    /// 宿主注入的根 prop（legacy 语义：args 先播为根属性）。
    seeded: std::collections::BTreeSet<String>,
    /// 播种值在 root 建立前先备好（句柄与 root 无关）。
    pending_seed: Vec<(String, u32)>,
}

impl Session {
    fn handle(&mut self, entry: Entry) -> u32 {
        self.entries.push(entry);
        self.entries.len() as u32
    }

    fn get(&self, handle: u32) -> Result<&Entry, HostError> {
        if handle == 0 || handle as usize > self.entries.len() {
            return Err(HostError::Io(format!("config: invalid handle {handle}")));
        }
        Ok(&self.entries[handle as usize - 1])
    }

    fn get_mut(&mut self, handle: u32) -> Result<&mut Entry, HostError> {
        if handle == 0 || handle as usize > self.entries.len() {
            return Err(HostError::Io(format!("config: invalid handle {handle}")));
        }
        Ok(&mut self.entries[handle as usize - 1])
    }

    fn charge_value(&mut self) -> Result<(), HostError> {
        self.values += 1;
        if self.values > self.limits.value_nodes {
            return Err(HostError::Resource);
        }
        Ok(())
    }

    fn charge_bytes(&mut self, n: usize) -> Result<(), HostError> {
        self.owned_bytes += n;
        if self.owned_bytes > self.limits.owned_bytes {
            return Err(HostError::Resource);
        }
        Ok(())
    }

    fn text(&mut self, bytes: &[u8]) -> Result<u32, HostError> {
        let s = std::str::from_utf8(bytes)
            .map_err(|_| HostError::Io("config: text argument is not UTF-8".into()))?
            .to_string();
        self.charge_bytes(s.len())?;
        self.texts.push(s);
        Ok((self.texts.len() - 1) as u32)
    }

    fn text_of(&self, id: u32) -> String {
        self.texts
            .get(id as usize)
            .cloned()
            .expect("config: text handle out of range (internal invariant)")
    }

    /// 值的 owned 形态（带深度守卫；收尾后导出用）。
    fn owned_value(&self, handle: u32, depth: usize) -> Result<OwnedValue, HostError> {
        if depth > 2048 {
            return Err(HostError::Resource);
        }
        match self.get(handle)? {
            Entry::Value(v) => match v {
                StoredValue::I32(n) => Ok(OwnedValue::I32(*n)),
                StoredValue::Bool(b) => Ok(OwnedValue::Bool(*b)),
                StoredValue::Char(c) => Ok(OwnedValue::Char(*c)),
                StoredValue::Text(t) => Ok(OwnedValue::Text(self.text_of(*t))),
                StoredValue::Node(n) => {
                    Ok(OwnedValue::Node(Box::new(self.owned_node(*n, depth + 1)?)))
                }
                StoredValue::Obj(o) => {
                    let fields = match self.get(*o)? {
                        Entry::Obj(data) => data.fields.clone(),
                        _ => {
                            return Err(HostError::Io("config: object handle kind changed".into()))
                        }
                    };
                    let mut out = Vec::with_capacity(fields.len());
                    for (k, v) in fields {
                        out.push((k, self.owned_value(v, depth + 1)?));
                    }
                    Ok(OwnedValue::Obj(out))
                }
                StoredValue::List(l) => {
                    let items = match self.get(*l)? {
                        Entry::List(data) => data.items.clone(),
                        _ => return Err(HostError::Io("config: list handle kind changed".into())),
                    };
                    let mut out = Vec::with_capacity(items.len());
                    for v in items {
                        out.push(self.owned_value(v, depth + 1)?);
                    }
                    Ok(OwnedValue::List(out))
                }
            },
            _ => Err(HostError::Io(
                "config: value position holds a container handle, not a value".into(),
            )),
        }
    }

    fn owned_node(&self, handle: u32, depth: usize) -> Result<OwnedNode, HostError> {
        if depth > 2048 {
            return Err(HostError::Resource);
        }
        let data = match self.get(handle)? {
            Entry::Node(d) => d,
            _ => return Err(HostError::Io("config: node handle expected".into())),
        };
        let mut args = Vec::with_capacity(data.args.len());
        for a in &data.args {
            args.push(self.owned_value(*a, depth + 1)?);
        }
        let mut props = Vec::with_capacity(data.props.len());
        for (k, v) in &data.props {
            props.push((k.clone(), self.owned_value(*v, depth + 1)?));
        }
        let mut children = Vec::with_capacity(data.children.len());
        for c in &data.children {
            children.push(self.owned_node(*c, depth + 1)?);
        }
        Ok(OwnedNode {
            name: data.name.clone(),
            id: data.id.clone(),
            args,
            props,
            children,
        })
    }

    /// 环检查（迭代）：节点→孩子与节点→实参里的节点值边。
    fn has_cycle(&self, root: u32) -> bool {
        let mut color: Vec<u8> = vec![0; self.entries.len()];
        let mut stack: Vec<(u32, usize)> = vec![(root, 0)];
        while let Some((h, idx)) = stack.pop() {
            let children: Vec<u32> = match self.entries.get(h as usize - 1) {
                Some(Entry::Node(d)) => {
                    let mut edges = d.children.clone();
                    for a in &d.args {
                        if let Some(Entry::Value(StoredValue::Node(n))) =
                            self.entries.get(*a as usize - 1)
                        {
                            edges.push(*n);
                        }
                    }
                    edges
                }
                _ => Vec::new(),
            };
            if idx == 0 {
                if color[h as usize - 1] == 1 {
                    return true;
                }
                if color[h as usize - 1] == 2 {
                    continue;
                }
                color[h as usize - 1] = 1;
            }
            if idx < children.len() {
                stack.push((h, idx + 1));
                stack.push((children[idx], 0));
            } else {
                color[h as usize - 1] = 2;
            }
        }
        false
    }
}

// 会话是显式单线程生命周期（宿主在一次求值内打开/关闭），不是全局注册表。
thread_local! {
    static SESSION: std::cell::RefCell<Option<Session>> = const { std::cell::RefCell::new(None) };
}

/// 打开一次配置文档求值会话（已打开 = 具名拒绝，不覆盖别人的状态）。
pub fn open_session(limits: DocumentLimits) -> Result<(), AdmissionError> {
    SESSION.with(|slot| {
        let mut guard = slot.borrow_mut();
        if guard.is_some() {
            return Err(AdmissionError {
                code: "config.session",
                message: "a configuration document session is already open".into(),
            });
        }
        *guard = Some(Session {
            entries: Vec::new(),
            root: None,
            finished: false,
            limits,
            nodes: 0,
            values: 0,
            owned_bytes: 0,
            callees: 0,
            texts: Vec::new(),
            seeded: std::collections::BTreeSet::new(),
            pending_seed: Vec::new(),
        });
        Ok(())
    })
}

/// 宿主注入一个根属性（legacy 语义：请求 args 先播成根 prop，且**文档里同名
/// `key: value` 不覆盖它**——宿主注入的值赢）。必须在 `open_session` 之后、
/// 执行之前调用；root 尚未建立时先备好，`cfg_root()` 一建立就挂上。
pub fn seed_root_prop(key: &str, value: &OwnedValue) -> Result<(), HostError> {
    SESSION.with(|slot| {
        let mut guard = slot.borrow_mut();
        let Some(session) = guard.as_mut() else {
            return Err(HostError::Io("config: no open session".into()));
        };
        if session.finished {
            return Err(HostError::Io(format!(
                "config: session already finished ({ERR_FINISHED})"
            )));
        }
        let handle = seed_value(session, value)?;
        session.seeded.insert(key.to_string());
        session.pending_seed.push((key.to_string(), handle));
        Ok(())
    })
}

/// 把 owned 值建成会话条目（递归；预算照计）。
fn seed_value(session: &mut Session, value: &OwnedValue) -> Result<u32, HostError> {
    match value {
        OwnedValue::I32(n) => {
            session.charge_value()?;
            session.charge_bytes(4)?;
            Ok(session.handle(Entry::Value(StoredValue::I32(*n))))
        }
        OwnedValue::Bool(b) => {
            session.charge_value()?;
            Ok(session.handle(Entry::Value(StoredValue::Bool(*b))))
        }
        OwnedValue::Char(c) => {
            session.charge_value()?;
            Ok(session.handle(Entry::Value(StoredValue::Char(*c))))
        }
        OwnedValue::Text(t) => {
            session.charge_value()?;
            session.charge_bytes(t.len())?;
            let idx = session.text(t.as_bytes())?;
            Ok(session.handle(Entry::Value(StoredValue::Text(idx))))
        }
        OwnedValue::Obj(fields) => {
            session.charge_value()?;
            let mut built: Vec<(String, u32)> = Vec::with_capacity(fields.len());
            for (k, v) in fields {
                let h = seed_value(session, v)?;
                built.push((k.clone(), h));
            }
            let mut obj = ObjData { fields: Vec::new() };
            for (k, v) in built {
                obj_set(&mut obj, k, v);
            }
            Ok(session.handle(Entry::Obj(Box::new(obj))))
        }
        OwnedValue::List(items) => {
            session.charge_value()?;
            let mut built = Vec::with_capacity(items.len());
            for v in items {
                built.push(seed_value(session, v)?);
            }
            Ok(session.handle(Entry::List(Box::new(ListData { items: built }))))
        }
        OwnedValue::Node(n) => build_seed_node(session, n),
    }
}

fn build_seed_node(session: &mut Session, node: &OwnedNode) -> Result<u32, HostError> {
    session.nodes += 1;
    session.charge_bytes(node.name.len())?;
    let mut props: Vec<(String, u32)> = Vec::with_capacity(node.props.len());
    for (k, v) in &node.props {
        let h = seed_value(session, v)?;
        props.push((k.clone(), h));
    }
    let mut args = Vec::with_capacity(node.args.len());
    for a in &node.args {
        args.push(seed_value(session, a)?);
    }
    let mut children = Vec::with_capacity(node.children.len());
    for c in &node.children {
        children.push(build_seed_node(session, c)?);
    }
    let id = node.id.as_ref().map(|i| match i {
        OwnedId::Name(n) => OwnedId::Name(n.clone()),
        OwnedId::Text(t) => OwnedId::Text(t.clone()),
    });
    Ok(session.handle(Entry::Node(Box::new(NodeData {
        name: node.name.clone(),
        id,
        args,
        props,
        children,
        attached: true,
    }))))
}
/// 该线程是否有打开的配置会话。
pub fn session_is_open() -> bool {
    SESSION.with(|slot| slot.borrow().is_some())
}

/// 关闭会话并取 owned 快照（未 `finish`/未建 root ⇒ 具名拒绝 + 清状态）。
pub fn close_session() -> Result<OwnedDocument, AdmissionError> {
    SESSION.with(|slot| {
        let mut guard = slot.borrow_mut();
        let Some(session) = guard.take() else {
            return Err(AdmissionError {
                code: "config.session",
                message: "no configuration document session is open".into(),
            });
        };
        let Some(root) = session.root else {
            return Err(AdmissionError {
                code: "config.session",
                message: "the configuration document session produced no root container".into(),
            });
        };
        if !session.finished {
            return Err(AdmissionError {
                code: "config.session",
                message: "the configuration document session was not finished".into(),
            });
        }
        if session.has_cycle(root) {
            return Err(AdmissionError {
                code: "config.cycle",
                message: "the configuration document contains a cycle".into(),
            });
        }
        let root_node = session.owned_node(root, 0).map_err(|e| AdmissionError {
            code: "config.export",
            message: format!("{e:?}"),
        })?;
        Ok(OwnedDocument {
            root: root_node,
            cost: DocumentCost {
                nodes: session.nodes,
                values: session.values,
                owned_bytes: session.owned_bytes,
                callees: session.callees,
            },
        })
    })
}

/// 丢弃会话（失败路径；不产出文档）。
pub fn abandon_session() {
    SESSION.with(|slot| {
        *slot.borrow_mut() = None;
    });
}

/// 原语执行：`PlatformCall` 的配置分支（会话内同步、无重入）。
pub fn dispatch(symbol: &str, args: &[Value], cancel: &CancelFlag) -> Result<i32, HostError> {
    if crate::av::CancelCheck::poll(cancel) {
        return Err(HostError::Cancelled);
    }
    let spec = spec_of(symbol)
        .ok_or_else(|| HostError::Io(format!("config: unknown primitive `{symbol}`")))?;
    // 参数个数/类型：MIR 已校验；这里兜底核（不静默转换）。
    if args.len() != spec.params().len() {
        return Err(HostError::Io("config: arity mismatch".into()));
    }
    SESSION.with(|slot| {
        let mut guard = slot.borrow_mut();
        let Some(session) = guard.as_mut() else {
            return Err(HostError::Io("config: no open session".into()));
        };
        session.callees += 1;
        if session.finished && !matches!(spec, ConfigSpec::Finish) {
            return Err(HostError::Io(format!(
                "config: session already finished ({ERR_FINISHED})"
            )));
        }
        match spec {
            ConfigSpec::Root => {
                if let Some(r) = session.root {
                    return Ok(r as i32);
                }
                session.nodes += 1;
                let h = session.handle(Entry::Node(Box::new(NodeData {
                    name: String::new(),
                    id: None,
                    args: Vec::new(),
                    props: Vec::new(),
                    children: Vec::new(),
                    attached: true,
                })));
                session.root = Some(h);
                // 注入的根属性在此挂上（legacy：args 先播，文档同名键不覆盖）。
                let mut seeds = std::mem::take(&mut session.pending_seed);
                for (key, value) in seeds.drain(..) {
                    set_prop(session, h, key, value)?;
                }
                Ok(h as i32)
            }
            ConfigSpec::Node => {
                let name = text_arg(&args[0])?;
                session.charge_bytes(name.len())?;
                session.nodes += 1;
                Ok(session.handle(Entry::Node(Box::new(NodeData {
                    name,
                    id: None,
                    args: Vec::new(),
                    props: Vec::new(),
                    children: Vec::new(),
                    attached: false,
                }))) as i32)
            }
            ConfigSpec::ObjNew => {
                session.charge_value()?;
                Ok(session.handle(Entry::Obj(Box::new(ObjData { fields: Vec::new() }))) as i32)
            }
            ConfigSpec::ListNew => {
                session.charge_value()?;
                Ok(session.handle(Entry::List(Box::new(ListData { items: Vec::new() }))) as i32)
            }
            ConfigSpec::IdName | ConfigSpec::IdText => {
                let node = i32_arg(&args[0])? as u32;
                let id = text_arg(&args[1])?;
                session.charge_bytes(id.len())?;
                match session.get_mut(node)? {
                    Entry::Node(d) => {
                        if d.id.is_some() {
                            return Err(HostError::Io(format!(
                                "config: node already has an id ({ERR_DUPLICATE_ID})"
                            )));
                        }
                        d.id = Some(if matches!(spec, ConfigSpec::IdText) {
                            OwnedId::Text(id)
                        } else {
                            OwnedId::Name(id)
                        });
                        Ok(0)
                    }
                    _ => Err(HostError::Io("config: id requires a node handle".into())),
                }
            }
            ConfigSpec::ArgPos(kind) => {
                let node = i32_arg(&args[0])? as u32;
                let value = store_value(session, kind, &args[1])?;
                match session.get_mut(node)? {
                    Entry::Node(d) => {
                        d.args.push(value);
                        Ok(0)
                    }
                    _ => Err(HostError::Io(
                        "config: argument requires a node handle".into(),
                    )),
                }
            }
            ConfigSpec::ArgNamed(kind) => {
                let node = i32_arg(&args[0])? as u32;
                let key = text_arg(&args[1])?;
                session.charge_bytes(key.len())?;
                let value = store_value(session, kind, &args[2])?;
                match session.get_mut(node)? {
                    Entry::Node(d) => {
                        d.args.push(value);
                        // 具名实参额外记成属性（旧实现同形：命名实参也进 props）。
                        set_prop(session, node, key, value)?;
                        Ok(0)
                    }
                    _ => Err(HostError::Io(
                        "config: argument requires a node handle".into(),
                    )),
                }
            }
            ConfigSpec::Put(target, kind) => {
                let handle = i32_arg(&args[0])? as u32;
                match target {
                    PutTarget::List => {
                        let value = store_value(session, kind, &args[1])?;
                        match session.get_mut(handle)? {
                            Entry::List(d) => {
                                d.items.push(value);
                                Ok(0)
                            }
                            _ => Err(HostError::Io(
                                "config: list-push requires a list handle".into(),
                            )),
                        }
                    }
                    PutTarget::Node | PutTarget::Obj => {
                        let key = text_arg(&args[1])?;
                        session.charge_bytes(key.len())?;
                        let value = store_value(session, kind, &args[2])?;
                        match target {
                            PutTarget::Node => {
                                // 宿主注入赢：root 上被 args 播过的键不被文档覆盖。
                                if Some(handle) == session.root && session.seeded.contains(&key) {
                                    return Ok(0);
                                }
                                match session.get(handle)? {
                                    Entry::Node(_) => set_prop(session, handle, key, value)?,
                                    _ => {
                                        return Err(HostError::Io(
                                            "config: put requires a node handle".into(),
                                        ))
                                    }
                                }
                                Ok(0)
                            }
                            _ => match session.get_mut(handle)? {
                                Entry::Obj(d) => {
                                    obj_set(d, key, value);
                                    Ok(0)
                                }
                                _ => Err(HostError::Io(
                                    "config: obj-put requires an object handle".into(),
                                )),
                            },
                        }
                    }
                }
            }
            ConfigSpec::SuperPut => {
                // `cfg_super_put(dst, src_obj, key)`：把源对象的字段**克隆**进
                // 目标节点（任何值种类；缺键具名报错；根级仍受 seeded 覆盖规则）。
                let dst = i32_arg(&args[0])? as u32;
                let src = i32_arg(&args[1])? as u32;
                let key = text_arg(&args[2])?;
                session.charge_bytes(key.len())?;
                let value_handle = {
                    let handle = match session.get(src)? {
                        Entry::Obj(d) => *d
                            .fields
                            .iter()
                            .find(|(k, _)| *k == key)
                            .map(|(_, h)| h)
                            .ok_or_else(|| {
                                HostError::Io(format!("config: object has no field `{key}`"))
                            })?,
                        _ => {
                            return Err(HostError::Io(
                                "config: super-put requires an object source".into(),
                            ))
                        }
                    };
                    handle
                };
                session.charge_value()?;
                if Some(dst) == session.root && session.seeded.contains(&key) {
                    return Ok(0);
                }
                match session.get_mut(dst)? {
                    Entry::Node(d) => {
                        // 克隆 = 新条目（共享句柄会让两个文档位置别名）。
                        d.props.push((key, value_handle));
                        Ok(0)
                    }
                    _ => Err(HostError::Io(
                        "config: super-put requires a node destination".into(),
                    )),
                }
            }
            ConfigSpec::Append => {
                let parent = i32_arg(&args[0])? as u32;
                let child = i32_arg(&args[1])? as u32;
                if parent == child {
                    return Err(HostError::Io(format!("config: self-append ({ERR_CYCLE})")));
                }
                let (child_name, attached) = match session.get(child)? {
                    Entry::Node(d) => (d.name.clone(), d.attached),
                    _ => return Err(HostError::Io("config: append requires node handles".into())),
                };
                if attached {
                    return Err(HostError::Io(format!(
                        "config: child `{child_name}` is already attached ({ERR_ATTACHED})"
                    )));
                }
                match session.get_mut(child)? {
                    Entry::Node(d) => d.attached = true,
                    _ => unreachable!(),
                }
                match session.get_mut(parent)? {
                    Entry::Node(d) => {
                        d.children.push(child);
                        Ok(0)
                    }
                    _ => Err(HostError::Io("config: append parent must be a node".into())),
                }
            }
            ConfigSpec::Finish => {
                if session.root.is_none() {
                    return Err(HostError::Io("config: finish without a root".into()));
                }
                session.finished = true;
                Ok(0)
            }
        }
    })
}

fn set_prop(session: &mut Session, handle: u32, key: String, value: u32) -> Result<(), HostError> {
    match session.get_mut(handle)? {
        Entry::Node(d) => {
            // 旧 set_prop 语义：同名属性原位替换，可观察顺序不变。
            if let Some(slot) = d.props.iter_mut().find(|(k, _)| *k == key) {
                slot.1 = value;
            } else {
                d.props.push((key, value));
            }
            Ok(())
        }
        _ => Err(HostError::Io(
            "config: property write requires a node".into(),
        )),
    }
}

fn obj_set(obj: &mut ObjData, key: String, value: u32) {
    if let Some(slot) = obj.fields.iter_mut().find(|(k, _)| *k == key) {
        slot.1 = value;
    } else {
        obj.fields.push((key, value));
    }
}

/// 把一个已求值实参登记成值句柄（标量装箱 / 容器引用）。
fn store_value(session: &mut Session, kind: ValKind, v: &Value) -> Result<u32, HostError> {
    session.charge_value()?;
    let stored = match kind {
        ValKind::I32 => StoredValue::I32(i32_arg(v)?),
        ValKind::Bool => match v {
            Value::Bool(b) => StoredValue::Bool(*b),
            _ => return Err(HostError::Io("config: bool argument expected".into())),
        },
        ValKind::Char => match v {
            Value::Char(c) => StoredValue::Char(*c),
            _ => return Err(HostError::Io("config: char argument expected".into())),
        },
        ValKind::Str => {
            let bytes = str_bytes(v)?;
            let t = session.text(bytes)?;
            StoredValue::Text(t)
        }
        ValKind::Node => StoredValue::Node(i32_arg(v)? as u32),
        ValKind::Obj => StoredValue::Obj(i32_arg(v)? as u32),
        ValKind::List => StoredValue::List(i32_arg(v)? as u32),
    };
    Ok(session.handle(Entry::Value(stored)))
}

fn i32_arg(v: &Value) -> Result<i32, HostError> {
    match v {
        Value::I32(n) => Ok(*n),
        _ => Err(HostError::Io("config: i32 argument expected".into())),
    }
}

fn text_arg(v: &Value) -> Result<String, HostError> {
    let bytes = str_bytes(v)?;
    std::str::from_utf8(bytes)
        .map(|s| s.to_string())
        .map_err(|_| HostError::Io("config: text argument is not UTF-8".into()))
}

fn str_bytes<'a>(v: &'a Value) -> Result<&'a [u8], HostError> {
    match v {
        Value::Str(h) => Ok(h.bytes()),
        Value::StrView(sv) => {
            let raw = match &sv.src {
                crate::contract::StrViewSrc::Share(h) => h.bytes(),
                crate::contract::StrViewSrc::Static { buf } => buf.as_ref(),
            };
            crate::owned_text::view_bytes(raw, sv)
                .map_err(|t| HostError::Io(format!("config: invalid str view: {t:?}")))
        }
        _ => Err(HostError::Io("config: text argument expected".into())),
    }
}
