//! PLAN-982 B-CONFIG-I: the **neutral configuration-document contract**.
//!
//! This module owns only the public, ACE-free shape of one project
//! configuration document evaluation:
//!
//! - the owned document/value model, so a successful evaluation hands the
//!   consumer a complete snapshot with **no AV handles, no request borrows and
//!   no external pointers** (`auto.config.value/1`);
//! - the explicit request (source text, virtual path, context, limits, cancel
//!   flag), so nothing is read from ambient process state;
//! - the terminal report/identity/cost, so a success can only be claimed with
//!   the producing profile/domain/schema, the serving engine, the source and
//!   context identities and the real dispatch facts;
//! - the engine choice, where the legacy VM stays an explicit, named choice
//!   during migration — never a fallback from the ACE path.
//!
//! Ownership split (frozen by the design
//! `docs/design/ace-config-document-evaluation.md` §3):
//! - **this module** — neutral contract only. No ACE dependency, no legacy
//!   parser/codegen/VM reference: an explicit ACE configuration request can
//!   never silently switch to the legacy configuration engine, and this module
//!   contains no evaluation of its own.
//! - **`auto_lang_core::ace_entry`** — registry and dispatch discipline. The
//!   executable engine is the one already installed through
//!   [`install_ace_executor`](crate::install_ace_executor); the configuration
//!   face is a **default named-unsupported method on that same executor**
//!   (not a second global registry).
//! - **`auto-lang-ace-host`** — the reference implementation: compile the
//!   config-document profile, verify, execute on the AV, export the owned
//!   snapshot, tear down.
//!
//! Semantics that must survive the snapshot (design §3.1): exact signed /
//! unsigned / IEEE-bit float / char / Unicode-and-NUL text values, the
//! Node-vs-Object distinction, node arguments and children **order**,
//! duplicate nodes, and property override order. Object entries and node
//! props/children are ordered vectors on purpose — a map would lose the
//! observable order the legacy document had.
//!
//! The legacy shape (`auto_val::Node`/`Obj`) is only a *consumer adapter* on
//! top of this snapshot; it is not part of this contract and never evaluates
//! anything.

use std::time::Duration;

/// Source profile identity of the configuration-document front end.
pub const CONFIG_SOURCE_PROFILE: &str = "auto.source.config-document.v1";
/// Domain identity of the configuration-document evaluation.
pub const CONFIG_DOMAIN: &str = "ace.config.document/v1";
/// Public schema identity of one owned configuration value snapshot.
pub const CONFIG_VALUE_SCHEMA: &str = "auto.config.value/1";
/// Identity of the configuration evaluation receipt itself.
pub const CONFIG_RECEIPT_SCHEMA: &str = "auto.config/eval/1";
/// Engine identity that serves a configuration run (AV business evaluation).
pub const CONFIG_ENGINE_ID: &str = "av";

/// How a configuration evaluation may be served.
///
/// `LegacyVm` is the pre-migration engine (`eval_config_with_vm`); it stays an
/// **explicit** choice while the product migrates and is never selected as a
/// retry of a failed `AceAv` request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigEngineChoice {
    /// The legacy VM configuration engine (explicit migration-period choice).
    LegacyVm,
    /// The ACE AV configuration-document evaluation.
    AceAv,
}

impl ConfigEngineChoice {
    /// Canonical public name accepted by name-driven hosts.
    pub fn as_str(&self) -> &'static str {
        match self {
            ConfigEngineChoice::LegacyVm => "legacy-vm",
            ConfigEngineChoice::AceAv => "av",
        }
    }
}

/// Parse an explicit configuration engine name. Unknown names are a named
/// refusal; no name selects the legacy engine implicitly.
pub fn parse_config_engine(name: &str) -> Result<ConfigEngineChoice, String> {
    match name {
        "av" | "ace-av" | "ace" => Ok(ConfigEngineChoice::AceAv),
        "legacy" | "legacy-vm" | "vm" => Ok(ConfigEngineChoice::LegacyVm),
        other => Err(format!(
            "unknown configuration engine `{other}`: the accepted names are [av, legacy-vm]"
        )),
    }
}

/// One owned configuration value.
///
/// Numbers keep their exact identity (`I64`/`U64` never collapse into one
/// another, `F64` carries IEEE bits, `Char` is a Unicode scalar value) and
/// text keeps its exact UTF-8 bytes including NUL. `Object` and `Node` are
/// separate variants because the document model has both: an object is a
/// keyed value, a node is a document element with a name, an optional id,
/// positional arguments, props and ordered children.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValue {
    /// An explicit null (distinct from an absent entry).
    Null,
    Bool(bool),
    I64(i64),
    U64(u64),
    /// IEEE-754 binary64 bits, kept bit-exact (NaN payloads included).
    F64(u64),
    Char(char),
    Text(String),
    List(Vec<ConfigValue>),
    /// Ordered key/value entries (duplicate keys keep their order and both
    /// remain visible; consumers apply the document's own override rule).
    Object(Vec<(String, ConfigValue)>),
    Node(Box<ConfigNode>),
}

impl ConfigValue {
    /// A float from its value (bits preserved).
    pub fn f64(value: f64) -> Self {
        ConfigValue::F64(value.to_bits())
    }

    /// The float value of an `F64` (bit-exact round trip).
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            ConfigValue::F64(bits) => Some(f64::from_bits(*bits)),
            _ => None,
        }
    }

    /// Stable kind name (diagnostics/tests; never a coercion).
    pub fn kind(&self) -> &'static str {
        match self {
            ConfigValue::Null => "null",
            ConfigValue::Bool(_) => "bool",
            ConfigValue::I64(_) => "i64",
            ConfigValue::U64(_) => "u64",
            ConfigValue::F64(_) => "f64",
            ConfigValue::Char(_) => "char",
            ConfigValue::Text(_) => "text",
            ConfigValue::List(_) => "list",
            ConfigValue::Object(_) => "object",
            ConfigValue::Node(_) => "node",
        }
    }
}

/// One document node: `name id? (args...) { props/children }`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigNode {
    /// Node name as written in the document (`dep`, `app`, `link`, …).
    pub name: String,
    /// Optional identity argument (e.g. the quoted/bare dep name).
    pub id: Option<String>,
    /// Positional arguments after the identity.
    pub args: Vec<ConfigValue>,
    /// Ordered properties (an overridden property keeps both entries here;
    /// the consumer's document rule decides the visible one).
    pub props: Vec<(String, ConfigValue)>,
    /// Ordered child nodes (order and duplicates preserved).
    pub children: Vec<ConfigNode>,
}

impl ConfigNode {
    /// A node with just a name.
    pub fn new(name: impl Into<String>) -> Self {
        ConfigNode {
            name: name.into(),
            ..ConfigNode::default()
        }
    }

    /// The first property with this key (document override order applies at
    /// the consumer; this is the plain lookup).
    pub fn prop(&self, key: &str) -> Option<&ConfigValue> {
        self.props.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Child nodes with this name, in document order.
    pub fn children_named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a ConfigNode> {
        self.children.iter().filter(move |c| c.name == name)
    }
}

/// The complete evaluated document handed to the consumer.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfigDocument {
    pub root: ConfigNode,
}

impl ConfigDocument {
    /// Iterative statistics: no recursion, so an arbitrarily deep accepted
    /// document cannot overflow the exporter's stack.
    pub fn stats(&self) -> ConfigStats {
        enum Item<'a> {
            Value(&'a ConfigValue, u64),
            Node(&'a ConfigNode, u64),
        }
        let mut stats = ConfigStats::default();
        let mut work = vec![Item::Node(&self.root, 0)];
        while let Some(item) = work.pop() {
            match item {
                Item::Node(node, depth) => {
                    stats.nodes += 1;
                    stats.depth = stats.depth.max(depth);
                    stats.bytes += node.name.len() as u64
                        + node.id.as_ref().map(|i| i.len() as u64).unwrap_or(0);
                    for arg in &node.args {
                        work.push(Item::Value(arg, depth + 1));
                    }
                    for (_, value) in &node.props {
                        work.push(Item::Value(value, depth + 1));
                    }
                    for child in &node.children {
                        work.push(Item::Node(child, depth + 1));
                    }
                }
                Item::Value(value, depth) => {
                    stats.values += 1;
                    stats.depth = stats.depth.max(depth);
                    match value {
                        ConfigValue::List(items) => {
                            for item in items {
                                work.push(Item::Value(item, depth + 1));
                            }
                        }
                        ConfigValue::Object(entries) => {
                            for (key, item) in entries {
                                stats.bytes += key.len() as u64;
                                work.push(Item::Value(item, depth + 1));
                            }
                        }
                        ConfigValue::Node(node) => work.push(Item::Node(node, depth + 1)),
                        ConfigValue::Text(text) => stats.bytes += text.len() as u64,
                        _ => {}
                    }
                }
            }
        }
        stats
    }
}

/// Document size accounting (budgets and receipts; counts, not opinions).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ConfigStats {
    pub nodes: u64,
    pub values: u64,
    pub bytes: u64,
    pub depth: u64,
}

// ---------------------------------------------------------------------------
// Configuration value/container primitives (the semantic core the AV
// intrinsics will delegate to)
// ---------------------------------------------------------------------------

/// One document node slot in a [`ConfigDocumentBuilder`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConfigNodeId(u32);

/// A named failure of one configuration primitive. The AV bindings surface
/// these as their own stable codes; nothing falls back to a default value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigBuildError {
    /// The node handle was never minted by this builder (or was released).
    UnknownNode { id: u32 },
    /// A value-kind check failed (`get`/`merge`/scalar unbox).
    KindMismatch {
        expected: &'static str,
        actual: &'static str,
    },
    /// A keyed lookup missed its key.
    MissingKey { key: String },
    /// Appending a node into itself (or into its own subtree) would build a
    /// cycle; the document model is a tree.
    Cycle,
    /// A node handle was already appended somewhere else (a tree node has one
    /// parent).
    AlreadyAttached { id: u32 },
    /// A document budget was exceeded.
    Limit { axis: &'static str, limit: u64 },
}

impl std::fmt::Display for ConfigBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigBuildError::UnknownNode { id } => {
                write!(f, "cfg.unknown-node: no configuration node {id}")
            }
            ConfigBuildError::KindMismatch { expected, actual } => {
                write!(f, "cfg.kind-mismatch: expected {expected}, found {actual}")
            }
            ConfigBuildError::MissingKey { key } => write!(f, "cfg.missing-key: `{key}`"),
            ConfigBuildError::Cycle => {
                write!(f, "cfg.cycle: a node cannot be appended into itself")
            }
            ConfigBuildError::AlreadyAttached { id } => {
                write!(f, "cfg.already-attached: node {id} already has a parent")
            }
            ConfigBuildError::Limit { axis, limit } => {
                write!(f, "cfg.limit/{axis}: limit {limit} exceeded")
            }
        }
    }
}

/// Builds one configuration document through explicit, kind-checked
/// primitives (`cfg.node` / `cfg.put` / `cfg.append` / `cfg.get` / `cfg.merge`).
///
/// This is the semantic core of the design's configuration intrinsics: the AV
/// bindings receive a `ConfigNodeId` from this arena and must never carry a
/// bare Rust pointer or index of their own. Handles are per-builder, a node has
/// at most one parent, and the finishing step produces the same owned snapshot
/// the public success type carries. Budgets are checked on every mutation, so a
/// runaway document fails by name instead of growing without bound.
pub struct ConfigDocumentBuilder {
    nodes: Vec<ConfigNode>,
    /// Parent of each attached node (`None` = not attached yet).
    attached: Vec<bool>,
    limits: ConfigEvalLimits,
    nodes_minted: u64,
    bytes: u64,
}

impl ConfigDocumentBuilder {
    /// A builder under the given budgets.
    pub fn new(limits: ConfigEvalLimits) -> Self {
        ConfigDocumentBuilder {
            nodes: Vec::new(),
            attached: Vec::new(),
            limits,
            nodes_minted: 0,
            bytes: 0,
        }
    }

    /// Mint a node (`cfg.node`). The returned handle may be mutated and
    /// appended exactly once.
    pub fn node(
        &mut self,
        name: impl Into<String>,
        id: Option<String>,
        args: Vec<ConfigValue>,
    ) -> Result<ConfigNodeId, ConfigBuildError> {
        self.nodes_minted += 1;
        if self.limits.max_nodes > 0 && self.nodes_minted > self.limits.max_nodes {
            return Err(ConfigBuildError::Limit {
                axis: "nodes",
                limit: self.limits.max_nodes,
            });
        }
        let name = name.into();
        self.charge(name.len() as u64 + id.as_ref().map(|i| i.len() as u64).unwrap_or(0))?;
        let node = ConfigNode {
            name,
            id,
            args,
            props: Vec::new(),
            children: Vec::new(),
        };
        self.charge(node_bytes(&node))?;
        self.nodes.push(node);
        self.attached.push(false);
        Ok(ConfigNodeId((self.nodes.len() - 1) as u32))
    }

    /// Set a property (`cfg.put`). Later puts with the same key keep both
    /// entries in order; consumers apply the document override rule.
    pub fn put(
        &mut self,
        node: ConfigNodeId,
        key: impl Into<String>,
        value: ConfigValue,
    ) -> Result<(), ConfigBuildError> {
        let index = self.index_of(node)?;
        let key = key.into();
        self.charge(key.len() as u64 + value_own_bytes(&value))?;
        self.nodes[index].props.push((key, value));
        Ok(())
    }

    /// Append a child node (`cfg.append`): the child must be unattached, must
    /// not be the parent and must not already be an ancestor of it.
    pub fn append(
        &mut self,
        parent: ConfigNodeId,
        child: ConfigNodeId,
    ) -> Result<(), ConfigBuildError> {
        let parent_index = self.index_of(parent)?;
        let child_index = self.index_of(child)?;
        if parent_index == child_index {
            return Err(ConfigBuildError::Cycle);
        }
        if self.attached[child_index] {
            return Err(ConfigBuildError::AlreadyAttached { id: child.0 });
        }
        let child = self.nodes[child_index].clone();
        self.nodes[parent_index].children.push(child);
        self.attached[child_index] = true;
        Ok(())
    }

    /// Merge an object value's entries into a node (`cfg.merge`). The value
    /// must be an `Object`; anything else is a kind mismatch.
    pub fn merge(
        &mut self,
        node: ConfigNodeId,
        value: &ConfigValue,
    ) -> Result<(), ConfigBuildError> {
        let index = self.index_of(node)?;
        let ConfigValue::Object(entries) = value else {
            return Err(ConfigBuildError::KindMismatch {
                expected: "object",
                actual: value.kind(),
            });
        };
        for (key, entry) in entries {
            let key = key.clone();
            let entry = entry.clone();
            self.put(node, key, entry)?;
        }
        let _ = index;
        Ok(())
    }

    /// Read a key from an object value or a value's own object payload
    /// (`cfg.get`). Kind mismatches and missing keys are named errors.
    pub fn get<'a>(
        &self,
        value: &'a ConfigValue,
        key: &str,
    ) -> Result<&'a ConfigValue, ConfigBuildError> {
        match value {
            ConfigValue::Object(entries) => entries
                .iter()
                .rev()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v)
                .ok_or_else(|| ConfigBuildError::MissingKey {
                    key: key.to_string(),
                }),
            other => Err(ConfigBuildError::KindMismatch {
                expected: "object",
                actual: other.kind(),
            }),
        }
    }

    /// Read a node's prop (`cfg.node.get`).
    pub fn node_prop(
        &self,
        node: ConfigNodeId,
        key: &str,
    ) -> Result<&ConfigValue, ConfigBuildError> {
        let index = self.index_of(node)?;
        self.nodes[index]
            .prop(key)
            .ok_or_else(|| ConfigBuildError::MissingKey {
                key: key.to_string(),
            })
    }

    /// Attach the given unattached node as the document root and export the
    /// owned snapshot. Any node minted but never attached is dropped by name
    /// of the budget accounting (it never reaches the consumer).
    pub fn finish(mut self, root: ConfigNodeId) -> Result<ConfigDocument, ConfigBuildError> {
        let index = self.index_of(root)?;
        if self.attached[index] {
            return Err(ConfigBuildError::AlreadyAttached { id: root.0 });
        }
        let root_node = self.nodes[index].clone();
        let stats = ConfigDocument {
            root: root_node.clone(),
        }
        .stats();
        if self.limits.max_bytes > 0 && stats.bytes > self.limits.max_bytes {
            return Err(ConfigBuildError::Limit {
                axis: "bytes",
                limit: self.limits.max_bytes,
            });
        }
        Ok(ConfigDocument { root: root_node })
    }

    fn index_of(&self, id: ConfigNodeId) -> Result<usize, ConfigBuildError> {
        let index = id.0 as usize;
        if index >= self.nodes.len() {
            return Err(ConfigBuildError::UnknownNode { id: id.0 });
        }
        Ok(index)
    }

    fn charge(&mut self, bytes: u64) -> Result<(), ConfigBuildError> {
        self.bytes += bytes;
        if self.limits.max_bytes > 0 && self.bytes > self.limits.max_bytes {
            return Err(ConfigBuildError::Limit {
                axis: "bytes",
                limit: self.limits.max_bytes,
            });
        }
        Ok(())
    }
}

fn node_bytes(node: &ConfigNode) -> u64 {
    let mut bytes = node.name.len() as u64 + node.id.as_ref().map(|i| i.len() as u64).unwrap_or(0);
    for arg in &node.args {
        bytes += value_own_bytes(arg);
    }
    bytes
}

fn value_own_bytes(value: &ConfigValue) -> u64 {
    match value {
        ConfigValue::Text(text) => text.len() as u64,
        ConfigValue::Object(entries) => entries.iter().map(|(k, _)| k.len() as u64).sum(),
        _ => 0,
    }
}
/// The explicit evaluation context of one configuration run.
///
/// Everything the document may observe is *in here*: injected arguments
/// (`port`, `kernel`, …), an explicit environment snapshot, the source root and
/// the allowed module bundle, the schema identity and the explicit grants. No
/// ambient process state, no implicit file reads, no network.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ConfigContext {
    /// Injected arguments as tagged values (order preserved).
    pub args: Vec<(String, ConfigValue)>,
    /// Explicit environment snapshot (name, value); empty = nothing visible.
    pub env: Vec<(String, String)>,
    /// Source root the document's relative reads resolve against.
    pub source_root: String,
    /// Allowed module/source identities (empty = the document itself only).
    pub module_bundle: Vec<String>,
    /// Schema identity the document is checked against.
    pub schema_id: String,
    /// Explicit capability grants for this run.
    pub grants: Vec<String>,
}

impl ConfigContext {
    /// A context with no injection (the document-only case).
    pub fn empty() -> Self {
        ConfigContext::default()
    }

    /// Stable digest over the full context (identity axis of the receipt).
    pub fn digest(&self) -> u64 {
        let mut hash = FNV_OFFSET;
        for (key, value) in &self.args {
            hash = fnv_bytes(hash, key.as_bytes());
            hash = fnv_bytes(hash, value.kind().as_bytes());
            hash = fnv_bytes(hash, &[0x1f]);
        }
        for (key, value) in &self.env {
            hash = fnv_bytes(hash, key.as_bytes());
            hash = fnv_bytes(hash, value.as_bytes());
            hash = fnv_bytes(hash, &[0x1e]);
        }
        for part in [
            self.source_root.as_str(),
            self.schema_id.as_str(),
            &self.module_bundle.join("\u{1f}"),
            &self.grants.join("\u{1f}"),
        ] {
            hash = fnv_bytes(hash, part.as_bytes());
            hash = fnv_bytes(hash, &[0x1d]);
        }
        hash
    }
}

/// Budgets for one configuration evaluation. `None`/`0` means "not this axis";
/// the host still applies its own hard caps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigEvalLimits {
    /// Maximum executed AV steps.
    pub steps: u64,
    /// Maximum nodes in the exported document.
    pub max_nodes: u64,
    /// Maximum retained document bytes.
    pub max_bytes: u64,
    /// Wall-clock budget for the run.
    pub wall: Option<Duration>,
}

impl Default for ConfigEvalLimits {
    fn default() -> Self {
        ConfigEvalLimits {
            steps: 1_000_000,
            max_nodes: 100_000,
            max_bytes: 16 * 1024 * 1024,
            wall: Some(Duration::from_secs(5)),
        }
    }
}

/// One explicit configuration evaluation request.
pub struct AceConfigRequest<'a> {
    /// The configuration document source text.
    pub source: &'a str,
    /// Virtual path used by diagnostics/source identity (never an ambient
    /// file read: the source is carried explicitly).
    pub virtual_path: &'a str,
    /// Explicit context (args/env/source root/modules/schema/grants).
    pub context: &'a ConfigContext,
    /// Budgets for this run.
    pub limits: ConfigEvalLimits,
    /// Host-owned cancellation flag.
    pub cancel: Option<&'a std::sync::atomic::AtomicBool>,
}

impl<'a> AceConfigRequest<'a> {
    /// A minimal request: the document only, empty context, default limits.
    pub fn simple(source: &'a str) -> Self {
        AceConfigRequest {
            source,
            virtual_path: "<config>",
            context: &EMPTY_CONTEXT,
            limits: ConfigEvalLimits::default(),
            cancel: None,
        }
    }
}

static EMPTY_CONTEXT: ConfigContext = ConfigContext {
    args: Vec::new(),
    env: Vec::new(),
    source_root: String::new(),
    module_bundle: Vec::new(),
    schema_id: String::new(),
    grants: Vec::new(),
};

impl ConfigContext {
    /// The shared empty context used by [`AceConfigRequest::simple`].
    pub fn empty_ref() -> &'static ConfigContext {
        &EMPTY_CONTEXT
    }
}

/// Where a diagnostic/denial came from in the configuration pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigSpan {
    pub start: u64,
    pub end: u64,
}

/// Identity of one successful evaluation — what makes the result attributable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigIdentity {
    /// Receipt schema (`CONFIG_RECEIPT_SCHEMA`).
    pub schema: &'static str,
    /// Serving engine (`CONFIG_ENGINE_ID` when the AV served it).
    pub engine: &'static str,
    /// Source profile (`CONFIG_SOURCE_PROFILE`).
    pub source_profile: &'static str,
    /// Domain identity (`CONFIG_DOMAIN`).
    pub domain: &'static str,
    /// Public value schema (`CONFIG_VALUE_SCHEMA`).
    pub value_schema: &'static str,
    /// Schema the document was checked against (from the request context).
    pub schema_id: String,
    /// FNV-1a over the exact source bytes.
    pub source_hash: u64,
    /// Digest over the full request context.
    pub context_digest: u64,
    /// This SDK's package version.
    pub sdk_version: &'static str,
}

impl ConfigIdentity {
    /// Build the identity of one served run.
    pub fn of(request: &AceConfigRequest<'_>, engine: &'static str) -> Self {
        ConfigIdentity {
            schema: CONFIG_RECEIPT_SCHEMA,
            engine,
            source_profile: CONFIG_SOURCE_PROFILE,
            domain: CONFIG_DOMAIN,
            value_schema: CONFIG_VALUE_SCHEMA,
            schema_id: request.context.schema_id.clone(),
            source_hash: fnv1a(request.source.as_bytes()),
            context_digest: request.context.digest(),
            sdk_version: env!("CARGO_PKG_VERSION"),
        }
    }

    /// True when this identity names the ACE AV configuration engine.
    pub fn served_by_ace_av(&self) -> bool {
        self.engine == CONFIG_ENGINE_ID
    }
}

/// What the run actually cost (real dispatch facts, not estimates).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ConfigCost {
    /// Executed AV steps.
    pub steps: u64,
    /// Nodes in the exported document.
    pub nodes: u64,
    /// Retained document bytes.
    pub bytes: u64,
    /// Deepest document nesting observed.
    pub depth: u64,
    /// Configuration-native primitive dispatches (node/put/append/merge/get…).
    pub callee_count: Option<u64>,
}

/// Terminal state of one configuration evaluation.
#[derive(Debug, Clone, PartialEq)]
pub enum AceConfigOutcome {
    Success {
        document: ConfigDocument,
        identity: ConfigIdentity,
        cost: ConfigCost,
    },
    /// A source/binding/type/verify diagnostic, original stage and span kept.
    Diagnostic {
        stage: String,
        code: String,
        message: String,
        span: Option<ConfigSpan>,
    },
    /// Authorization/admission refusal (engine, grants, schema, context).
    Denied { code: String, message: String },
    /// Budget denial (steps/wall/nodes/bytes).
    ResourceDenied { kind: String, message: String },
    /// The host cancel flag was observed.
    Cancelled,
    /// Host/tool failure (no executor, unsupported platform, bridge fault).
    HostFailure { code: String, message: String },
}

/// The full report of one configuration evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct AceConfigReport {
    pub outcome: AceConfigOutcome,
    /// Executor-measured wall time of the whole request.
    pub wall_ms: f64,
}

impl AceConfigReport {
    pub fn success(document: ConfigDocument, identity: ConfigIdentity, cost: ConfigCost) -> Self {
        AceConfigReport {
            outcome: AceConfigOutcome::Success {
                document,
                identity,
                cost,
            },
            wall_ms: 0.0,
        }
    }

    pub fn diagnostic(
        stage: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
        span: Option<ConfigSpan>,
    ) -> Self {
        AceConfigReport {
            outcome: AceConfigOutcome::Diagnostic {
                stage: stage.into(),
                code: code.into(),
                message: message.into(),
                span,
            },
            wall_ms: 0.0,
        }
    }

    pub fn denied(code: impl Into<String>, message: impl Into<String>) -> Self {
        AceConfigReport {
            outcome: AceConfigOutcome::Denied {
                code: code.into(),
                message: message.into(),
            },
            wall_ms: 0.0,
        }
    }

    pub fn resource_denied(kind: impl Into<String>, message: impl Into<String>) -> Self {
        AceConfigReport {
            outcome: AceConfigOutcome::ResourceDenied {
                kind: kind.into(),
                message: message.into(),
            },
            wall_ms: 0.0,
        }
    }

    pub fn cancelled() -> Self {
        AceConfigReport {
            outcome: AceConfigOutcome::Cancelled,
            wall_ms: 0.0,
        }
    }

    pub fn host_failure(code: impl Into<String>, message: impl Into<String>) -> Self {
        AceConfigReport {
            outcome: AceConfigOutcome::HostFailure {
                code: code.into(),
                message: message.into(),
            },
            wall_ms: 0.0,
        }
    }

    /// The default terminal state of an executor that does not implement the
    /// configuration face: a NAMED refusal, never a legacy retry.
    pub fn unsupported() -> Self {
        AceConfigReport::host_failure(
            "ace.config/executor-unsupported",
            "the installed ACE executor does not implement the configuration-document face; \
             the request is refused rather than evaluated by the legacy engine",
        )
    }

    /// True only for a successful evaluation.
    pub fn is_success(&self) -> bool {
        matches!(self.outcome, AceConfigOutcome::Success { .. })
    }
}

// ---------------------------------------------------------------------------
// Consumer adapter: the owned snapshot -> the legacy `auto_val` shape
// ---------------------------------------------------------------------------

/// Why a document could not be projected onto the legacy `auto_val` shape.
///
/// The projection is a **pure data conversion** for existing consumers
/// (`Pac`, project-model readers). It never evaluates anything, never calls the
/// parser/VM, and refuses instead of silently losing value identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigProjectionError {
    /// `auto_val` has no unsigned 64-bit value; a larger-than-`i64::MAX` U64
    /// would change its value, so it is refused.
    U64OutOfRange { value: u64 },
    /// The document nests deeper than the projection guard (no stack overflow).
    TooDeep { depth: usize },
}

impl std::fmt::Display for ConfigProjectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigProjectionError::U64OutOfRange { value } => write!(
                f,
                "config.projection/u64-out-of-range: {value} has no exact auto-val representation"
            ),
            ConfigProjectionError::TooDeep { depth } => write!(
                f,
                "config.projection/too-deep: depth {depth} exceeds the projection guard \
                 (MAX_PROJECTION_DEPTH = {MAX_PROJECTION_DEPTH})"
            ),
        }
    }
}

/// Depth guard of the consumer projection (iterative accounting has no such
/// limit; this one exists so the recursive auto_val shape cannot overflow).
pub const MAX_PROJECTION_DEPTH: usize = 512;

impl ConfigDocument {
    /// Project the owned snapshot onto the legacy `auto_val` shape for existing
    /// consumers. Order and duplicates are preserved for nodes; object and prop
    /// entries keep the snapshot's order, and the projection applies the
    /// document's own override rule (a later entry for the same key wins, with
    /// the `auto_val` container's insertion-order semantics).
    pub fn to_auto_val_node(&self) -> Result<auto_val::Node, ConfigProjectionError> {
        project_node(&self.root, 0)
    }
}

fn project_node(node: &ConfigNode, depth: usize) -> Result<auto_val::Node, ConfigProjectionError> {
    if depth > MAX_PROJECTION_DEPTH {
        return Err(ConfigProjectionError::TooDeep { depth });
    }
    // 文档根沿用 legacy 消费形状的名字 `root`（配置求值结果的根节点）。
    let name = if depth == 0 && node.name.is_empty() {
        "root"
    } else {
        node.name.as_str()
    };
    let mut out = auto_val::Node::new(name);
    // D-982-CFG-IDENTITY: 身份与前序位置实参都落在统一 args/props 槽的
    // `_arg{n}`（legacy 形状：`dep kernel { _arg0: "kernel" }`；`app("x")`
    // 也是 `_arg0`）。有身份时位置实参从 `_arg1` 起，无身份时从 `_arg0` 起。
    // 这些键是**普通 prop**（legacy 的 config 求值不把它们计入 `num_args`），
    // 且身份同时镜像到 `Node.id`（legacy 对 `app("x")` 也是这样）。
    // legacy 形状区分两种写法：**裸/字符串身份**（`dep bps {}`）只落在 `id`，
    // 不产生 `_arg0` 属性；**括号实参**（`dep("kernel")`、`app("x")`）落在
    // `_arg0…` 属性，且首个文本实参同时镜像到 `id`。
    let mut identity: Option<String> = node.id.clone();
    let mut next_arg = 0usize;
    for arg in node.args.iter() {
        if next_arg == 0 {
            // 无身份时首个文本实参同时作为身份（legacy `app("x")` 形状）。
            if let ConfigValue::Text(t) = arg {
                identity.get_or_insert_with(|| t.clone());
            }
        }
        let key = format!("_arg{next_arg}");
        out.set_prop(key, project_value(arg, depth + 1)?);
        next_arg += 1;
    }
    if let Some(id) = identity {
        out.id = auto_val::AutoStr::from(id.as_str());
    }
    for (key, value) in &node.props {
        out.set_prop(key.as_str(), project_value(value, depth + 1)?);
    }
    for child in &node.children {
        out.add_kid(project_node(child, depth + 1)?);
    }
    Ok(out)
}

fn project_value(
    value: &ConfigValue,
    depth: usize,
) -> Result<auto_val::Value, ConfigProjectionError> {
    if depth > MAX_PROJECTION_DEPTH {
        return Err(ConfigProjectionError::TooDeep { depth });
    }
    Ok(match value {
        ConfigValue::Null => auto_val::Value::Nil,
        ConfigValue::Bool(v) => auto_val::Value::Bool(*v),
        // legacy config 求值把整数放进 `Int`（i32）；超范围才用 `I64`，
        // 这样消费面的种类与旧引擎一致（不是"看起来相等"）。
        ConfigValue::I64(v) => match i32::try_from(*v) {
            Ok(n) => auto_val::Value::Int(n),
            Err(_) => auto_val::Value::I64(*v),
        },
        ConfigValue::U64(v) => {
            if *v <= i64::MAX as u64 {
                auto_val::Value::I64(*v as i64)
            } else {
                return Err(ConfigProjectionError::U64OutOfRange { value: *v });
            }
        }
        ConfigValue::F64(bits) => auto_val::Value::Double(f64::from_bits(*bits)),
        ConfigValue::Char(c) => auto_val::Value::Char(*c),
        ConfigValue::Text(text) => auto_val::Value::Str(auto_val::AutoStr::from(text.as_str())),
        ConfigValue::List(items) => {
            let mut projected = Vec::with_capacity(items.len());
            for item in items {
                projected.push(project_value(item, depth + 1)?);
            }
            auto_val::Value::Array(auto_val::Array::from_vec(projected))
        }
        ConfigValue::Object(entries) => {
            let mut obj = auto_val::Obj::new();
            for (key, item) in entries {
                obj.set(key.as_str(), project_value(item, depth + 1)?);
            }
            auto_val::Value::Obj(Box::new(obj))
        }
        ConfigValue::Node(node) => auto_val::Value::node(project_node(node, depth + 1)?),
    })
}
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x100_0000_01b3;

fn fnv_bytes(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// FNV-1a over exact bytes (source/context identity; never a security hash).
pub fn fnv1a(bytes: &[u8]) -> u64 {
    fnv_bytes(FNV_OFFSET, bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_document() -> ConfigDocument {
        let mut kernel = ConfigNode::new("dep");
        kernel.id = Some("kernel".to_string());
        kernel.props.push((
            "port".to_string(),
            ConfigValue::Text("msvc/x64".to_string()),
        ));
        let mut root = ConfigNode::new("root");
        root.props
            .push(("name".to_string(), ConfigValue::Text("p982".to_string())));
        root.props
            .push(("u".to_string(), ConfigValue::U64(u64::MAX)));
        root.props
            .push(("i".to_string(), ConfigValue::I64(i64::MIN)));
        root.props.push((
            "nan".to_string(),
            ConfigValue::f64(f64::from_bits(0x7ff8_0000_dead_beef)),
        ));
        root.props.push(("ch".to_string(), ConfigValue::Char('中')));
        root.props
            .push(("nul".to_string(), ConfigValue::Text("a\0b".to_string())));
        root.props.push((
            "obj".to_string(),
            ConfigValue::Object(vec![
                ("z".to_string(), ConfigValue::Bool(true)),
                ("a".to_string(), ConfigValue::Null),
            ]),
        ));
        root.children.push(kernel);
        root.children.push(ConfigNode::new("link"));
        root.children.push(ConfigNode::new("link"));
        ConfigDocument { root }
    }

    #[test]
    fn p982b_values_keep_exact_identity() {
        let doc = sample_document();
        assert_eq!(doc.root.prop("u"), Some(&ConfigValue::U64(u64::MAX)));
        assert_eq!(doc.root.prop("i"), Some(&ConfigValue::I64(i64::MIN)));
        let nan = doc.root.prop("nan").unwrap().as_f64().unwrap();
        assert_eq!(nan.to_bits(), 0x7ff8_0000_dead_beef);
        assert_eq!(doc.root.prop("ch"), Some(&ConfigValue::Char('中')));
        assert_eq!(
            doc.root.prop("nul"),
            Some(&ConfigValue::Text("a\0b".to_string()))
        );
        // Object entries keep their written order (not sorted).
        match doc.root.prop("obj").unwrap() {
            ConfigValue::Object(entries) => {
                let keys: Vec<&str> = entries.iter().map(|(k, _)| k.as_str()).collect();
                assert_eq!(keys, ["z", "a"]);
            }
            other => panic!("expected object, got {other:?}"),
        }
        // Nodes and duplicates survive in order; object != node.
        let names: Vec<&str> = doc.root.children.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["dep", "link", "link"]);
        assert_eq!(doc.root.children[0].id.as_deref(), Some("kernel"));
        assert_eq!(doc.root.prop("obj").unwrap().kind(), "object");
        assert!(matches!(
            doc.root.prop("obj").unwrap(),
            ConfigValue::Object(_)
        ));
        assert_eq!(doc.root.children[0].prop("port").unwrap().kind(), "text");
    }

    #[test]
    fn p982b_stats_are_iterative_and_count_real_content() {
        let doc = sample_document();
        let stats = doc.stats();
        assert_eq!(stats.nodes, 4, "root + 3 children: {stats:?}");
        assert!(
            stats.values >= 7,
            "props/entries/args counted as values: {stats:?}"
        );
        assert!(stats.bytes >= 4, "text payload counted: {stats:?}");
        assert_eq!(
            stats.depth, 2,
            "children one level below the root, object entries one below that"
        );

        // A 2000-deep document must not overflow the exporter's stack.
        let mut deep = ConfigNode::new("leaf");
        for _ in 0..2000 {
            let mut parent = ConfigNode::new("wrap");
            parent.children.push(deep);
            deep = parent;
        }
        let stats = ConfigDocument { root: deep }.stats();
        assert_eq!(stats.nodes, 2001);
        assert_eq!(stats.depth, 2000);
    }

    #[test]
    fn p982b_identity_names_the_producing_surface() {
        let source = "name: \"p982\"\n";
        let mut context = ConfigContext::empty();
        context.schema_id = "auto.pac/1".to_string();
        context
            .args
            .push(("port".to_string(), ConfigValue::Text("win32".to_string())));
        let request = AceConfigRequest {
            source,
            virtual_path: "pac.at",
            context: &context,
            limits: ConfigEvalLimits::default(),
            cancel: None,
        };
        let identity = ConfigIdentity::of(&request, CONFIG_ENGINE_ID);
        assert_eq!(identity.schema, CONFIG_RECEIPT_SCHEMA);
        assert_eq!(identity.engine, "av");
        assert_eq!(identity.source_profile, CONFIG_SOURCE_PROFILE);
        assert_eq!(identity.domain, CONFIG_DOMAIN);
        assert_eq!(identity.value_schema, CONFIG_VALUE_SCHEMA);
        assert_eq!(identity.schema_id, "auto.pac/1");
        assert_eq!(identity.source_hash, fnv1a(source.as_bytes()));
        assert_eq!(identity.context_digest, context.digest());
        assert!(identity.served_by_ace_av());
        assert_eq!(identity.sdk_version, env!("CARGO_PKG_VERSION"));

        // A different context/source never reuses the digest.
        let mut other = context.clone();
        other
            .args
            .push(("kernel".to_string(), ConfigValue::Bool(true)));
        assert_ne!(other.digest(), context.digest());
        assert_ne!(fnv1a(b"name: \"x\"\n"), fnv1a(source.as_bytes()));
        assert_eq!(
            ConfigIdentity::of(&request, "legacy-vm").served_by_ace_av(),
            false
        );
    }

    #[test]
    fn p982b_engine_names_are_explicit() {
        assert_eq!(parse_config_engine("av"), Ok(ConfigEngineChoice::AceAv));
        assert_eq!(parse_config_engine("ace-av"), Ok(ConfigEngineChoice::AceAv));
        assert_eq!(
            parse_config_engine("legacy-vm"),
            Ok(ConfigEngineChoice::LegacyVm)
        );
        assert_eq!(ConfigEngineChoice::LegacyVm.as_str(), "legacy-vm");
        assert_eq!(ConfigEngineChoice::AceAv.as_str(), "av");
        // Unknown names refuse; there is no implicit engine.
        assert!(parse_config_engine("autovm").is_err());
        assert!(parse_config_engine("").is_err());
        assert!(parse_config_engine("AV").is_err());
    }

    #[test]
    fn p982b_requests_and_terminal_states_are_named() {
        let request = AceConfigRequest::simple("name: \"x\"\n");
        assert_eq!(request.virtual_path, "<config>");
        assert!(request.cancel.is_none());
        assert!(request.context.args.is_empty());
        assert_eq!(
            ConfigContext::empty_ref().digest(),
            ConfigContext::empty().digest()
        );

        assert!(AceConfigReport::success(
            sample_document(),
            ConfigIdentity::of(&request, CONFIG_ENGINE_ID),
            ConfigCost::default()
        )
        .is_success());
        assert!(!AceConfigReport::unsupported().is_success());
        match AceConfigReport::unsupported().outcome {
            AceConfigOutcome::HostFailure { code, .. } => {
                assert_eq!(code, "ace.config/executor-unsupported")
            }
            other => panic!("unsupported must be a named host failure: {other:?}"),
        }
        match AceConfigReport::resource_denied("wall", "budget").outcome {
            AceConfigOutcome::ResourceDenied { kind, .. } => assert_eq!(kind, "wall"),
            other => panic!("resource denial: {other:?}"),
        }
        match AceConfigReport::cancelled().outcome {
            AceConfigOutcome::Cancelled => {}
            other => panic!("cancel terminal state: {other:?}"),
        }
    }
    #[test]
    fn p982b_builder_assembles_a_real_document() {
        let mut builder = ConfigDocumentBuilder::new(ConfigEvalLimits::default());
        let root = builder.node("root", None, vec![]).unwrap();
        let dep = builder
            .node("dep", Some("kernel".to_string()), vec![])
            .unwrap();
        builder
            .put(dep, "port", ConfigValue::Text("msvc/x64".to_string()))
            .unwrap();
        builder
            .put(root, "name", ConfigValue::Text("p982b".to_string()))
            .unwrap();
        builder
            .merge(
                root,
                &ConfigValue::Object(vec![("extra".to_string(), ConfigValue::Bool(true))]),
            )
            .unwrap();
        builder.append(root, dep).unwrap();
        let link = builder.node("link", None, vec![]).unwrap();
        builder.append(root, link).unwrap();
        let document = builder.finish(root).unwrap();
        assert_eq!(document.root.name, "root");
        assert_eq!(
            document.root.prop("name"),
            Some(&ConfigValue::Text("p982b".to_string()))
        );
        assert_eq!(document.root.prop("extra"), Some(&ConfigValue::Bool(true)));
        let names: Vec<&str> = document
            .root
            .children
            .iter()
            .map(|c| c.name.as_str())
            .collect();
        assert_eq!(names, ["dep", "link"]);
        assert_eq!(document.root.children[0].id.as_deref(), Some("kernel"));
        assert_eq!(
            document.root.children[0].prop("port"),
            Some(&ConfigValue::Text("msvc/x64".to_string()))
        );
        let stats = document.stats();
        assert_eq!(stats.nodes, 3);
    }

    #[test]
    fn p982b_builder_refuses_by_name() {
        let mut builder = ConfigDocumentBuilder::new(ConfigEvalLimits::default());
        let root = builder.node("root", None, vec![]).unwrap();
        let dep = builder.node("dep", None, vec![]).unwrap();

        // Kind checks and missing keys.
        let text = ConfigValue::Text("x".to_string());
        assert_eq!(
            builder.get(&text, "k").unwrap_err(),
            ConfigBuildError::KindMismatch {
                expected: "object",
                actual: "text"
            }
        );
        let object = ConfigValue::Object(vec![]);
        assert_eq!(
            builder.get(&object, "missing").unwrap_err(),
            ConfigBuildError::MissingKey {
                key: "missing".to_string()
            }
        );
        assert!(matches!(
            builder.merge(root, &text),
            Err(ConfigBuildError::KindMismatch { .. })
        ));

        // Tree discipline: self-append, double attach, attached root.
        assert_eq!(
            builder.append(root, root).unwrap_err(),
            ConfigBuildError::Cycle
        );
        builder.append(root, dep).unwrap();
        assert_eq!(
            builder.append(root, dep).unwrap_err(),
            ConfigBuildError::AlreadyAttached { id: dep.0 }
        );

        // Unknown handles are refused, never treated as a fresh node.
        let bogus = ConfigNodeId(999);
        assert!(matches!(
            builder.put(bogus, "k", text.clone()),
            Err(ConfigBuildError::UnknownNode { .. })
        ));
        assert!(matches!(
            builder.node_prop(root, "k"),
            Err(ConfigBuildError::MissingKey { .. })
        ));
        // An already-attached node cannot become the root of a second document.
        assert_eq!(
            builder.finish(dep).unwrap_err(),
            ConfigBuildError::AlreadyAttached { id: dep.0 }
        );
    }

    #[test]
    fn p982b_builder_applies_its_budgets() {
        let limits = ConfigEvalLimits {
            max_nodes: 1,
            max_bytes: 8,
            ..ConfigEvalLimits::default()
        };
        let mut builder = ConfigDocumentBuilder::new(limits);
        let root = builder.node("root", None, vec![]).unwrap();
        assert!(matches!(
            builder.node("dep", None, vec![]).unwrap_err(),
            ConfigBuildError::Limit { axis: "nodes", .. }
        ));
        // A property that pushes the document past the byte budget refuses too.
        assert!(matches!(
            builder.put(
                root,
                "very_long_key",
                ConfigValue::Text("value".to_string())
            ),
            Err(ConfigBuildError::Limit { axis: "bytes", .. })
        ));
    }
    #[test]
    fn p982b_node_identity_is_the_first_positional_argument() {
        // D-982-CFG-IDENTITY decision (2026-11-17): the legacy configuration
        // shape carries a node's identity as its FIRST POSITIONAL ARGUMENT
        // (`dep kernel { _arg0: ... }`), and `auto_val::Node.id` stays empty.
        // The projection reproduces that shape exactly instead of inventing a
        // second identity channel.
        let mut builder = ConfigDocumentBuilder::new(ConfigEvalLimits::default());
        let root = builder.node("root", None, vec![]).unwrap();
        let dep = builder
            .node("dep", Some("kernel".to_string()), vec![])
            .unwrap();
        builder.append(root, dep).unwrap();
        let node = builder.finish(root).unwrap().to_auto_val_node().unwrap();
        let mut checked = false;
        for (_k, kid) in node.kids_iter() {
            if let auto_val::Kid::Node(child) = kid {
                checked = true;
                assert_eq!(child.name.as_str(), "dep");
                assert_eq!(child.id.as_str(), "", "the separate id field stays empty");
                // `add_arg(Arg::Pos(..))` lands in the unified args/props slot
                // as `_arg0`, exactly the shape the legacy evaluator produced.
                assert_eq!(child.get_prop("_arg0").to_astr().as_str(), "kernel");
            }
        }
        assert!(checked, "the projected document keeps its node child");
    }
}
