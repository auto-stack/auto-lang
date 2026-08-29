//! UI AST Nodes - First-class AST nodes for UI declarations
//!
//! These nodes are only parsed when the scenario is UI (contextual keywords).
//! They represent widget, msg, model, view, and on blocks as first-class citizens.

use super::{Body, Expr, Name, Stmt, Type};
use super::route::RoutesBlock;
use auto_val::AutoStr;

// ============================================================================
// Widget Declaration
// ============================================================================

/// Widget declaration: the core UI component
///
/// ```auto
/// widget Counter {
///     msg Msg { Inc, Dec }
///     model { count int = 0 }
///     computed { doubleCount => .count * 2 }
///     view { ... }
///     on { .Inc => { .count += 1 } }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct WidgetDecl {
    /// Widget name (e.g., "Counter")
    pub name: Name,

    /// Message type declarations (msg blocks)
    pub messages: Vec<MsgDecl>,

    /// State/model declaration
    pub model: Option<ModelBlock>,

    /// Computed properties
    pub computed: Option<ComputedBlock>,

    /// View tree declaration
    pub view: Option<ViewBlock>,

    /// Event handlers
    pub on: Option<OnBlock>,

    /// Key bindings (Plan 275)
    pub bind: Option<BindBlock>,

    /// Props for reusable components
    pub props: Vec<PropDecl>,

    /// Routes block for router widgets (Plan 105)
    pub routes: Option<RoutesBlock>,

    /// Lifecycle methods (aboutToAppear, aboutToDisappear, etc.)
    pub lifecycle: Vec<LifecycleMethod>,

    /// Raw native CSS from a widget-level `style { ... }` block, captured
    /// verbatim by the lexer (never tokenized/parsed). Emitted into the
    /// component's `<style scoped>` by the Vue backend.
    pub style: Option<String>,

    /// External TS/Vue imports declared in a widget-level `use { ... }` block.
    /// Emitted as ES import statements into the component's `<script setup>`
    /// by the Vue backend; other backends ignore them.
    pub ext_imports: Vec<ExtImport>,

    /// Reactive watchers declared in a widget-level `watch { ... }` block.
    /// The Vue backend emits them as Vue `watch(source, handler)` calls;
    /// other backends ignore them.
    pub watch: Vec<WatchDecl>,

    /// Per-instance setup preamble (Plan 426): a `setup { ... }` block whose
    /// statements run synchronously at component setup, BEFORE the first
    /// render. The Vue backend emits them at `<script setup>` top level
    /// (state/computed definitions first need these bindings); the
    /// interpreter path is future work.
    pub setup: Option<SetupBlock>,

    /// Member names declared in a widget-level `expose { ... }` block
    /// (without the leading dot). The Vue backend emits them as
    /// `defineExpose({ ... })` in `<script setup>` so a parent holding a
    /// template ref on this component can call imperative methods or read
    /// exposed refs; other backends ignore them.
    pub expose: Vec<Name>,

    /// Plan 451: `actions { ... }` 块——widget 内的动作注册表 + menubar/
    /// toolbar 声明（auto-atom 外挂配置的 DSL 化形态）。vm 渲染器经
    /// `action_config()` 消费（DSL 源优先，外挂文件兜底）；其他后端目前
    /// 忽略（vue 侧消费见 Plan 451 P2）。
    pub actions: Option<ActionsBlock>,
}

/// Plan 451: `actions { ... }` 声明块。
///
/// ```auto
/// actions {
///     action (id: "file.new", handler: .ActNew, title: "新建", shortcut: "Ctrl+N")
///     menubar { menu (id: "file", title: "文件") { item (action: "file.new")  sep } }
///     toolbar { item (action: "file.new") }
/// }
/// ```
///
/// `enabled_if`/`checked_if` 为条件表达式。两种拼写（Plan 451 P2 起等价）：
/// 引号字符串（与 auto-atom 文件形态同源，如 ".store.tab_count > 0"）或
/// 裸 DSL 表达式（`enabled_if: .store.tab_count > 0`，解析期经 view if
/// 条件同文法校验）。运行时经 `eval_condition_with` 对合并根 state 求值；
/// vue 侧经 convert_condition 转译为模板表达式。
#[derive(Debug, Clone)]
pub struct ActionsBlock {
    pub actions: Vec<ActionEntry>,
    pub menubar: Option<MenubarBlock>,
    pub toolbar: Option<ToolbarBlock>,
}

/// 单个 action 声明：`action (id: "...", handler: .ActXxx, ...)`。
#[derive(Debug, Clone)]
pub struct ActionEntry {
    /// 点分动作 id（OS 用户键位层的跨版本契约，如 "file.new"）。
    pub id: String,
    /// 指向本 widget `on{}` 事件的 handler（含前导点，如 ".ActNew"）；
    /// 解析期校验必须命中（Plan 451 编译期校验承诺）。
    pub handler: String,
    pub title: Option<String>,
    pub icon: Option<String>,
    pub shortcut: Option<String>,
    /// 条件表达式字符串（引号原样）。
    pub enabled_if: Option<String>,
    pub checked_if: Option<String>,
}

/// `menubar { ... }` 子块。
#[derive(Debug, Clone)]
pub struct MenubarBlock {
    pub menus: Vec<MenuEntry>,
}

/// 单个菜单：`menu (id: "file", title: "文件") { ... }`。id 是开合内部消息
/// `__menubar_toggle(id)` 的键，须稳定。
#[derive(Debug, Clone)]
pub struct MenuEntry {
    pub id: String,
    pub title: String,
    pub items: Vec<MenuItemEntry>,
}

/// 菜单/工具栏条目：action 引用或分隔线。
#[derive(Debug, Clone)]
pub enum MenuItemEntry {
    /// 引用 action id（须存在于注册表，转换期校验）。
    Action(String),
    /// `sep` 分隔线。
    Sep,
}

/// `toolbar { ... }` 子块。
#[derive(Debug, Clone)]
pub struct ToolbarBlock {
    pub items: Vec<MenuItemEntry>,
}

/// Plan 426: `setup { ... }` 前导槽——每实例同步执行、先于首渲染的通用
/// setup 语句槽(不止无参 composable 调用)。
///
/// ```auto
/// widget App {
///     setup {
///         let t = useT() refs: [locale]
///         let ready = dom.prefers_dark()
///     }
///     view { col { text t("hi") } }
/// }
/// ```
///
/// 语句集 = handler 语句子集(let/表达式);`await` 在 MVP 明确拒绝
/// (async setup 需要 Suspense 边界,另立任务)。`refs: [...]` 是 let 语句
/// 后缀标注:列出的字段在 script 侧访问时注入 `.value`(复用 composable
/// kind 的 facade_ref_fields 机制——composable 返回普通对象时 ref 字段
/// 不自动解包)。
#[derive(Debug, Clone)]
pub struct SetupBlock {
    /// Statements (let / expression statements).
    pub body: crate::ast::Body,
    /// refs annotations: let binding name → ref field names.
    pub ref_annotations: Vec<(String, Vec<String>)>,
}

/// A single watcher inside a widget-level `watch { ... }` block.
///
/// ```auto
/// widget SlashMenu {
///     watch {
///         .filtered_items -> { .selected_index = 0 }
///         .ratio, .viewport_h.immediate -> { ... }
///         .items.deep -> { ... }
///     }
/// }
/// ```
///
/// Each source is a single field name (model field, prop, or computed)
/// written dot-prefixed in the DSL; the body uses the same conventions as
/// `on` handlers (`.field` reads/writes state, template refs via `.value!`).
#[derive(Debug, Clone)]
pub struct WatchDecl {
    /// Watched source field names (without the leading dot). More than one
    /// source emits a multi-source watch (`watch([a, b], ...)` in Vue).
    pub sources: Vec<Name>,

    /// `.immediate` modifier: run the handler once on setup.
    pub immediate: bool,

    /// `.deep` modifier: deep-watch the source.
    pub deep: bool,

    /// Handler body (same statement conventions as `on` handlers).
    pub body: Body,
}

/// Kind of an external import declared in a widget `use { ... }` block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtImportKind {
    /// `fn: name from "path"` — a plain TS function/constant, callable from
    /// `on` handlers and `computed` expressions.
    Fn,
    /// PLAN-051 C4: `use.web.fn name from "path"` — 显式 fn 别名声明（点分
    /// kind 形态）。语义同 Fn；VM 装载器对 .at 目标做导出校验（typo 编译期
    /// 报错，不再静默落 stub）。裸/空格旧形式仍为 Fn（不校验）。
    ExplicitFn,
    /// `component: Name from "path"` — an external Vue component,
    /// instantiable in the `view` tree with props and events.
    Component,
    /// `composable: useXxx from "path"` — a Vue composable, called once at
    /// `<script setup>` top level; its return value is bound to a local
    /// const (`useXxx` → `xxx`) reachable from `on` handlers.
    Composable,
    /// Plan 435 P4:`package: name from "dir"` — 声明一个 .at 组件包引用,
    /// 包内全部 widget 注册为可用 tag(ComponentRegistry,source=Package)。
    /// path 为目录(相对源文件解析),内含可选 package.at 清单。
    Package,
}

/// One external import entry inside a widget-level `use { ... }` block.
///
/// ```auto
/// widget Icon(language: str) {
///     use {
///         fn: getLanguageIconUrl from "src/front/utils/codeBlockLanguage.ts"
///         component: FancyBadge from "src/front/components/FancyBadge.vue"
///         component: Smile from "lucide-vue-next"
///         composable: useClock from "src/front/composables/useClock.ts"
///     }
/// }
/// ```
///
/// `path` is either an npm package specifier (paired with pac.at
/// `npm_deps:`) or a project-root-relative file path; the build copies
/// local files into the generated Vue project under `src/ext/` and
/// rewrites the import specifier to `@/ext/...`.
#[derive(Debug, Clone)]
pub struct ExtImport {
    pub kind: ExtImportKind,
    /// Imported symbol names (comma-separated in the DSL).
    pub symbols: Vec<Name>,
    /// Import source: npm package specifier or project-relative file path.
    pub path: AutoStr,
    /// Plan 022: 可选调用参数（仅 composable 消费）。语法：
    /// `composable: useX(.arg) from "..."` → call_args = [Expr::Dot(...)]。
    /// fn/component 声明忽略此字段。空 Vec = 无参调用（向后兼容）。
    pub call_args: Vec<Expr>,
    /// Plan 408 P12 §10.4: composable 的 ref 字段标注。语法：
    /// `composable: useX(refs: [currentSecretary, badgeCount]) from "..."`。
    /// 列出的字段在 script 表达式里访问时加 `.value`（因为 composable 返回
    /// 普通对象时，ref 字段不会自动 unwrap）。reactive composable 无需标注。
    pub ref_fields: Vec<Name>,
}

// ============================================================================
// Store Declaration (Plan 351 / Design 18 — Rung 4 shared store)
// ============================================================================

/// A module-level shared store: a view-less widget (`model` + `msg` + `on`)
/// whose state is shared across widgets/routes via `use store:`.
#[derive(Debug, Clone)]
pub struct StoreDecl {
    /// Store name (e.g., "CounterStore")
    pub name: Name,

    /// Message type declarations
    pub messages: Vec<MsgDecl>,

    /// State/model declaration
    pub model: Option<ModelBlock>,

    /// Computed properties (Plan 367 P2-2)
    pub computed: Option<ComputedBlock>,

    /// Event handlers
    pub on: Option<OnBlock>,

    /// Reactive watchers declared in a store-level `watch { ... }` block
    /// (Plan 012 Batch G / gap 12 — same syntax as the widget-level block).
    /// The Vue backend emits them as module-level `watch(...)` calls in the
    /// store composable (the refs are module-level singletons); other
    /// backends ignore them.
    pub watch: Vec<WatchDecl>,
}

// ============================================================================
// View Fragment (Plan 367 P2-3: reusable inline view snippets)
// ============================================================================

/// A view fragment declaration: `view fn NoteItem(note: Note, active: bool) { ... }`
///
/// Plan 367 P2-3: the fragment **inline-expands** at call sites (parameter
/// substitution into the body).
///
/// Plan 425: `component fn` left this AST — it now sugars to a `WidgetDecl`
/// at parse time (parse_component_fn_decl), carrying its msg/model/computed/
/// on/use/watch/style blocks there. This struct keeps only the inline
/// `view fn` surface: params + a single body node.
#[derive(Debug, Clone)]
pub struct ViewFragmentDecl {
    pub name: Name,
    pub params: Vec<(Name, String)>,
    pub body: ViewNode,
}

// ============================================================================
// Lifecycle Method
// ============================================================================

/// Lifecycle method declaration
///
/// ```auto
/// lifecycle {
///     aboutToAppear => { ... }
///     aboutToDisappear => { ... }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct LifecycleMethod {
    /// Method name (e.g., "aboutToAppear")
    pub name: String,

    /// Method body
    pub body: Vec<super::Stmt>,
}

// ============================================================================
// Message Declaration
// ============================================================================

/// Message declaration: defines message types for MVU pattern
///
/// ```auto
/// msg { Inc, Dec, Set(int) }
/// ```
///
/// Plan 448 A: the declaration carries no type name — event lookup always
/// resolves by variant (`.Inc` / `App.Inc` take the tail segment) and every
/// backend derives the enum name from the widget name. A legacy
/// `msg Msg { ... }` still parses with the name discarded.
#[derive(Debug, Clone)]
pub struct MsgDecl {
    /// Message variants
    pub variants: Vec<MsgVariant>,
}

/// Message variant
#[derive(Debug, Clone)]
pub struct MsgVariant {
    /// Variant name (e.g., "Inc")
    pub name: Name,

    /// True when the variant name was written quoted in source
    /// (`msg Msg { "update"(str) }`): a CONTRACTUAL emit name (lowercase /
    /// hyphenated / coloned) that must always be declared in defineEmits
    /// and always get its trailing emit() — even when no template binding
    /// references the handler (bridge-originated emits, self-call relay).
    pub quoted: bool,

    /// Payload types (empty = no payload). Plan 043 M5 #1: was
    /// `Option<Type>` (single param only); now a `Vec<Type>` so variants like
    /// `Complete(str, int)` / `RunSmart(int, str, []str)` parse. An empty vec
    /// means a unit variant; a single-element vec means `Set(int)`; multiple
    /// elements mean a tuple-style payload.
    pub payload: Vec<Type>,

    /// Plan 435 挂账②:命名参数形态 `Change(value: str)` 的参数名,
    /// 与 payload 逐位平行;位置形态(`Set(int)`)对应位为 None。
    /// 名字不参与生成(handler 形参来自 on 块),供 P4 组件包 API 文档面使用。
    pub payload_names: Vec<Option<String>>,
}

// ============================================================================
// Model Block
// ============================================================================

/// Decorator for model fields (Plan 05-Nav Task 0)
///
/// Supports HarmonyOS state management decorators:
/// - `#[Consume("key")]` - Consume value from ancestor
/// - `#[Provide("key")]` - Provide value to descendants
///
/// ```auto
/// model {
///     #[Provide("pathStack")] pathStack NavPathStack = NavPathStack()
///     #[Consume("pathStack")] pathStack NavPathStack
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Decorator {
    /// Decorator name (e.g., "Consume", "Provide")
    pub name: Name,

    /// Decorator arguments (e.g., ["pathStack"])
    pub args: Vec<String>,
}

/// Model block: defines state variables
///
/// ```auto
/// model {
///     count int = 0
///     name str = ""
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ModelBlock {
    /// State variable declarations
    pub fields: Vec<ModelField>,
}

/// Model field: a single state variable
#[derive(Debug, Clone)]
pub struct ModelField {
    /// Field name
    pub name: Name,

    /// Field type
    pub ty: Type,

    /// Initial value expression
    pub init: Expr,

    /// Whether the field is mutable (Plan 130: var keyword)
    /// Default is false (immutable, like `let`)
    /// When true, allows modification in `on` handlers
    pub mutable: bool,

    /// Whether this field is the primary property for shorthand syntax (Plan 119)
    /// When true, allows: `Text "Hello" {}` instead of `Text (text: "Hello") {}`
    pub is_primary: bool,

    /// Decorators for state management (Plan 05-Nav Task 0)
    /// E.g., `#[Consume("pathStack")]`, `#[Provide("pathStack")]`
    pub decorators: Vec<Decorator>,
}

// ============================================================================
// Computed Block
// ============================================================================

/// Computed block: defines computed/derived properties
///
/// ```auto
/// computed {
///     activeCount => .todos.filter(|t| !t.done).len
///     filteredTodos => match .filter {
///         Filter::All => .todos
///         Filter::Active => .todos.filter(|t| !t.done)
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ComputedBlock {
    /// Computed property declarations
    pub properties: Vec<ComputedProperty>,
}

/// Computed property: a derived value
#[derive(Debug, Clone)]
pub struct ComputedProperty {
    /// Property name
    pub name: Name,

    /// Computation expression
    pub expr: Expr,
}

// ============================================================================
// View Block
// ============================================================================

/// View block: defines the UI structure
///
/// ```auto
/// view {
///     col {
///         button + { onclick: .Inc }
///         h2 > Count: ${.count}
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ViewBlock {
    /// Root node of the view tree
    pub root: ViewNode,
}

/// View node: element or text in the view tree
#[derive(Debug, Clone)]
pub enum ViewNode {
    /// Element node with tag, props, events, and children
    Element {
        /// Tag name (e.g., "col", "button", "h2")
        tag: String,

        /// Properties (key-value pairs)
        props: Vec<ViewProp>,

        /// Event handlers
        events: Vec<ViewEvent>,

        /// Child nodes
        children: Vec<ViewNode>,

        /// Source span: (byte_offset, byte_length) in the .at file
        span: Option<(usize, usize)>,
    },

    /// Text node (literal or with interpolations)
    Text(ViewText),

    /// For loop: for item in .list { body }
    ForLoop {
        /// Loop variable name (e.g., "todo")
        var: String,

        /// Optional index variable (e.g., Some("i") for `for i, item in ...`)
        index: Option<String>,

        /// Iterable expression (e.g., ".todos")
        iterable: String,

        /// Loop body nodes
        body: Vec<ViewNode>,

        /// Source span: (byte_offset, byte_length) in the .at file
        span: Option<(usize, usize)>,
    },

    /// Conditional: if condition { then_body } else { else_body }
    Conditional {
        /// Condition expression as string (e.g., ".todos.len > 0")
        condition: String,

        /// Body when condition is true
        then_body: Vec<ViewNode>,

        /// Optional else body
        else_body: Option<Vec<ViewNode>>,

        /// Source span: (byte_offset, byte_length) in the .at file
        span: Option<(usize, usize)>,
    },

    /// Component instantiation: TodoItem (todo: .todo, onToggle: .Toggle)
    Component {
        /// Component name (e.g., "TodoItem")
        name: String,

        /// Properties passed to component
        props: Vec<ViewProp>,

        /// Event handlers
        events: Vec<ViewEvent>,

        /// Source span: (byte_offset, byte_length) in the .at file
        span: Option<(usize, usize)>,
    },

    /// Router outlet: renders the matched child route (Plan 105)
    Outlet,

    /// Navigation link: anchor with routing (Plan 105)
    Link {
        /// Target path for router-link (e.g., "/user/123")
        to: String,

        /// Text content (optional, for shorthand form)
        text: String,

        /// href for external links (optional)
        href: String,

        /// Child content
        children: Vec<ViewNode>,

        /// Source span: (byte_offset, byte_length) in the .at file
        span: Option<(usize, usize)>,
    },
}

/// View property
#[derive(Debug, Clone)]
pub struct ViewProp {
    /// Property name
    pub name: String,

    /// Property value
    pub value: ViewPropValue,
}

/// View property value - can be an expression or a style binding
#[derive(Debug, Clone)]
pub enum ViewPropValue {
    /// Regular expression value
    Expr(Expr),

    /// Style binding: { completed: todo.done, editing: todo.editing }
    StyleBinding(Vec<StyleBindingEntry>),
}

/// A single style binding entry
#[derive(Debug, Clone)]
pub struct StyleBindingEntry {
    /// Style name (e.g., "completed")
    pub style_name: String,

    /// Condition expression (e.g., todo.done)
    pub condition: Expr,
}

/// View event handler
#[derive(Debug, Clone)]
pub struct ViewEvent {
    /// Event name (e.g., "onclick")
    pub name: String,

    /// Handler pattern (e.g., ".Inc" or "Msg::Inc")
    pub handler: String,

    /// Optional parameters for the handler (e.g., ["todo.id"] for .Delete(todo.id))
    pub params: Vec<String>,

    /// Plan 448 B1: inline lambda body for `onclick: () => { ... }`. The
    /// handler field stays empty; extraction mints an anonymous event name
    /// (`__evt_<event>_<n>`) and injects these statements as the matching
    /// on-handler, so simple callbacks skip the msg/on round trip entirely.
    pub inline: Option<Vec<Stmt>>,
}

/// View text content
#[derive(Debug, Clone)]
pub enum ViewText {
    /// Literal text
    Literal(String),

    /// Interpolated text with ${...} placeholders
    Interpolated {
        /// Template string
        template: String,

        /// Extracted state references
        bindings: Vec<String>,
    },
}

// ============================================================================
// On Block
// ============================================================================

/// On block: defines event handlers
///
/// ```auto
/// on {
///     .Inc => { .count += 1 }
///     .Dec => { .count -= 1 }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct OnBlock {
    /// Event handlers
    pub handlers: Vec<OnHandler>,
}

/// Event handler
#[derive(Debug, Clone)]
pub struct OnHandler {
    /// Pattern to match (e.g., ".Inc", "Msg::Dec")
    pub pattern: String,

    /// Parameter names for the handler (e.g., ["text"] for .AddItem(text))
    pub params: Vec<String>,

    /// Handler body
    pub body: Body,

    /// Plan 028 F9: `on stream sse(url[, "event"]) -> { … }` — when set, this
    /// handler is a platform stream subscription, not a msg pattern. The
    /// backend opens the stream and dispatches each (pre-parsed) event into
    /// the handler body; `params` carries the event binding (["ev"]).
    pub stream: Option<StreamSubscription>,
}

/// Plan 028 F8/F9: platform stream subscription declaration.
#[derive(Debug, Clone)]
pub struct StreamSubscription {
    /// Protocol kind — "sse" for now (http polling etc. later).
    pub kind: String,
    /// Endpoint URL.
    pub url: String,
    /// Named-event filter (None → default message events). Relay streams use
    /// named events ("run_event"); EventSource needs addEventListener for them.
    pub event: Option<String>,
}

/// Key binding block (Plan 275)
#[derive(Debug, Clone)]
pub struct BindBlock {
    /// Key bindings
    pub bindings: Vec<KeyBinding>,
}

/// A single key binding: "key" -> .Handler
#[derive(Debug, Clone)]
pub struct KeyBinding {
    /// Key string (e.g., "1", "+", "Enter", "Escape")
    pub key: String,
    /// Handler pattern (e.g., ".Digit1")
    pub handler: String,
}

// ============================================================================
// Prop Declaration
// ============================================================================

/// Prop declaration for reusable components
///
/// ```auto
/// widget Button(text str, disabled bool = false) { ... }
/// ```
#[derive(Debug, Clone)]
pub struct PropDecl {
    /// Prop name
    pub name: Name,

    /// Prop type
    pub ty: Type,

    /// Default value (if any)
    pub default: Option<Expr>,
}

// ============================================================================
// Helper Implementations
// ============================================================================

impl ViewNode {
    /// Create a new element node
    pub fn element(tag: impl Into<String>) -> Self {
        ViewNode::Element {
            tag: tag.into(),
            props: Vec::new(),
            events: Vec::new(),
            children: Vec::new(),
            span: None,
        }
    }

    /// Create a text node
    pub fn text(content: impl Into<String>) -> Self {
        ViewNode::Text(ViewText::Literal(content.into()))
    }

    /// Add a property
    pub fn with_prop(mut self, name: impl Into<String>, value: Expr) -> Self {
        if let ViewNode::Element { props, .. } = &mut self {
            props.push(ViewProp {
                name: name.into(),
                value: ViewPropValue::Expr(value),
            });
        }
        self
    }

    /// Add an event handler
    pub fn with_event(mut self, name: impl Into<String>, handler: impl Into<String>) -> Self {
        if let ViewNode::Element { events, .. } = &mut self {
            events.push(ViewEvent {
                name: name.into(),
                handler: handler.into(),
                params: Vec::new(),
                inline: None,
            });
        }
        self
    }

    /// Add an event handler with parameters
    pub fn with_event_params(mut self, name: impl Into<String>, handler: impl Into<String>, params: Vec<String>) -> Self {
        if let ViewNode::Element { events, .. } = &mut self {
            events.push(ViewEvent {
                name: name.into(),
                handler: handler.into(),
                params,
                inline: None,
            });
        }
        self
    }

    /// Add a child node
    pub fn with_child(mut self, child: ViewNode) -> Self {
        if let ViewNode::Element { children, .. } = &mut self {
            children.push(child);
        }
        self
    }
}

// ============================================================================
// Godot Scene Declaration (Plan 306)
// ============================================================================
//
// Describes a Godot scene tree that the TscnGenerator emits as a `.tscn` file.
// `scene` is a top-level contextual keyword (Plan 306). It is parsed in any
// scenario but only meaningful when targeting Godot.

/// A Godot scene declaration that generates a `.tscn` file.
///
/// ```auto
/// scene Player : Area2D {
///     script = "player.gd"
///     z_index = 10
///
///     node AnimatedSprite2D {
///         ...
///     }
///
///     connect body_entered from "." to "." method "_on_body_entered"
/// }
/// ```
#[derive(Debug, Clone)]
pub struct SceneDecl {
    /// Scene (root node) name, e.g. "Player"
    pub name: Name,
    /// Godot root node type, e.g. "Area2D", "Control", "Node"
    pub node_type: Name,
    /// Root node properties (`key = value`)
    pub props: Vec<SceneProp>,
    /// Attached script path, e.g. `"player.gd"` (emits an ext_resource)
    pub script: Option<AutoStr>,
    /// Child nodes and scene instances, in declaration order
    pub children: Vec<SceneNode>,
    /// Signal connections (`connect signal from ... to ... method ...`)
    pub connections: Vec<SceneConnection>,
    /// Plan 306: Signal declarations (`signal name(params)`), emitted into the
    /// attached .gd as `signal name(param: Type)` lines. Not scene (.tscn) data.
    pub signals: Vec<SceneSignal>,
}

/// A single `name = value` property on a scene node (or inside a sub-resource).
#[derive(Debug, Clone)]
pub struct SceneProp {
    pub name: Name,
    pub value: SceneValue,
}

/// The value of a scene property.
///
/// Most values are ordinary expressions (`5`, `Vector2(1, 2)`, `load("res://x")`),
/// but a typed inline resource such as `CapsuleShape2D { radius = 5.0 }` is
/// emitted as its own `[sub_resource]` section and referenced via `SubResource("N")`.
#[derive(Debug, Clone)]
pub enum SceneValue {
    /// A regular expression value.
    Expr(Expr),
    /// An inline typed sub-resource: `TypeName { props }` → `[sub_resource]`.
    SubResource(SceneSubResource),
}

/// An inline typed sub-resource value, e.g. `CapsuleShape2D { radius = 5.0 }`.
///
/// Emits a `[sub_resource type="CapsuleShape2D" id="N"]` section and is
/// referenced elsewhere as `SubResource("N")`.
#[derive(Debug, Clone)]
pub struct SceneSubResource {
    /// Godot resource type, e.g. "CapsuleShape2D", "SpriteFrames".
    pub res_type: Name,
    /// The sub-resource's own properties (`key = value`).
    pub props: Vec<SceneProp>,
}

/// A child of a scene node — either a typed node or an instance of another scene.
#[derive(Debug, Clone)]
pub enum SceneNode {
    /// `node Type ["Name"] { props; children }`
    Node {
        node_type: Name,
        /// Optional explicit instance name; defaults to node_type
        name: Option<AutoStr>,
        props: Vec<SceneProp>,
        children: Vec<SceneNode>,
    },
    /// `instance Name "res://path.tscn"`
    Instance {
        name: AutoStr,
        path: AutoStr,
    },
}

/// A signal connection: `connect signal from <path> to <path> method <name>`
#[derive(Debug, Clone)]
pub struct SceneConnection {
    pub signal: AutoStr,
    pub from: AutoStr,
    pub to: AutoStr,
    pub method: AutoStr,
}

/// A Godot signal declared inside a scene: `signal name(p1 T1, p2 T2)` or `signal name`.
///
/// Emitted into the attached .gd as `signal name(p1: T1, p2: T2)`.
#[derive(Debug, Clone)]
pub struct SceneSignal {
    /// Signal name, e.g. "health_changed".
    pub name: AutoStr,
    /// Typed parameters; empty for a param-less signal (`signal game_over`).
    pub params: Vec<SceneSignalParam>,
}

/// A parameter of a scene signal: `name Type` → `name: Type`.
#[derive(Debug, Clone)]
pub struct SceneSignalParam {
    pub name: Name,
    pub ty: Type,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use auto_val::AutoStr;

    #[test]
    fn test_view_node_element() {
        let node = ViewNode::element("col")
            .with_child(ViewNode::text("Hello"));

        match node {
            ViewNode::Element { tag, children, .. } => {
                assert_eq!(tag, "col");
                assert_eq!(children.len(), 1);
            }
            _ => panic!("Expected Element node"),
        }
    }

    #[test]
    fn test_view_node_text() {
        let node = ViewNode::text("Hello World");

        match node {
            ViewNode::Text(ViewText::Literal(s)) => {
                assert_eq!(s, "Hello World");
            }
            _ => panic!("Expected Text node"),
        }
    }

    #[test]
    fn test_widget_decl() {
        let widget = WidgetDecl {
            name: AutoStr::from("Counter"),
            messages: vec![],
            model: None,
            view: None,
            on: None,
            bind: None,
            props: vec![],
            computed: None,
            routes: None,
            lifecycle: vec![],
            style: None,
            ext_imports: Vec::new(),
            watch: Vec::new(),
            setup: None,
            actions: None,
            expose: Vec::new(),
        };

        assert_eq!(widget.name.as_str(), "Counter");
    }

    #[test]
    fn test_msg_decl() {
        let msg = MsgDecl {
            variants: vec![
                MsgVariant {
                    name: AutoStr::from("Inc"),
                    quoted: false,
                    payload: vec![],
                payload_names: vec![],
                },
                MsgVariant {
                    name: AutoStr::from("Set"),
                    quoted: false,
                    payload: vec![Type::Int],
                 payload_names: vec![None],
                },
            ],
        };

        assert_eq!(msg.variants.len(), 2);
    }
}
