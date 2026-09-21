//! Rust/GPUI Code Generator
//!
//! Generates Rust code implementing the `Component` trait from AURA widgets.
//!
//! ## Output Format
//!
//! ```ignore
//! // Auto-generated from Auto language
//! // DO NOT EDIT - changes will be overwritten
//!
//! use auto_ui::prelude::*;
//!
//! #[derive(Clone, Copy, Debug, PartialEq)]
//! pub enum Msg {
//!     Inc,
//!     Dec,
//! }
//!
//! #[derive(Debug)]
//! pub struct Counter {
//!     pub count: i32,
//! }
//!
//! impl Counter {
//!     pub fn new() -> Self {
//!         Self {
//!             count: 0,
//!         }
//!     }
//! }
//!
//! impl Component for Counter {
//!     type Msg = Msg;
//!
//!     fn on(&mut self, msg: Self::Msg) {
//!         match msg {
//!             Msg::Inc => {
//!                 self.count += 1;
//!             }
//!             Msg::Dec => {
//!                 self.count -= 1;
//!             }
//!         }
//!     }
//!
//!     fn view(&self) -> View<Self::Msg> {
//!         View::col()
//!             .child(View::button("+").on_click(|_| Msg::Inc))
//!             .child(View::text(&format!("Count: {}", self.count)))
//!             .build()
//!     }
//! }
//! ```
//!
//! Based on auto-ui/trans/rust_gen.rs, adapted for AuraWidget input.

use super::{BackendGenerator, GenError, GenResult};
use crate::aura::{AuraEvent, AuraMsgVariant, AuraNode, AuraPropValue, AuraTextContent, AuraWidget, LogicPayload};

/// Plan 371 L1: Semantic info about a child component, collected via
/// cross-file pre-scan. Used to replace hardcoded special-case logic (Init
/// forwarding, prop writeback) with .at-source-driven general code.
#[derive(Debug, Clone, Default)]
pub struct ComponentSemantics {
    /// Prop names that are WRITTEN by the component's handlers (e.g. EditorPanel
    /// `.Save -> { .note.title = .edit_title }` → writes "note").
    /// Drives prop-writeback generation in the parent's child-forwarding arm.
    pub written_props: Vec<String>,
}

/// Rust/GPUI code generator
pub struct RustGenerator {
    /// Current widget name
    current_widget: Option<String>,

    /// PLAN-627: 当前 widget 的 `#[api]` 函数名清单（use back.api 两形态
    /// 抽取）——限定名调用 `api.X(...)` 的改写门（head 方法名 ∈ 此清单
    /// 才落裸名发射，防用户同名对象误伤）。
    api_imports: Vec<String>,

    /// Collected message variants
    message_variants: Vec<AuraMsgVariant>,

    /// Whether we need imports
    needs_imports: bool,

    /// Indent level
    indent: usize,

    /// Child component names referenced in the current widget's view tree
    child_components: Vec<String>,

    /// Plan 371 L3: child component names that appear INSIDE a for-loop in the
    /// view tree. These CANNOT be persistent fields (multiple instances), so
    /// they keep the old temp-construction pattern. Single-instance children
    /// (not in this set) become persistent struct fields.
    loop_child_components: std::collections::HashSet<String>,

    /// Plan 371 Task 22c: map of component name -> its own scalar state fields
    /// (name, rust_type), collected across all widgets parsed in the same
    /// compile unit, so a parent using a child can hoist+sync its state.
    component_state_fields: std::collections::HashMap<String, Vec<(String, String)>>,

    /// Plan 371 L1: map of component name -> its semantics (written props),
    /// collected cross-file. Drives general prop-writeback generation,
    /// replacing the old "constructor_args.contains(\"note\")" heuristic.
    component_semantics: std::collections::HashMap<String, ComponentSemantics>,

    /// Loop variables in scope (for generating correct references)
    loop_vars: Vec<String>,

    /// Maps input event variant name to field names for input text parsing
    /// Multiple inputs can share the same event (e.g., main input + edit input both fire EditInputChanged)
    input_fields: std::collections::HashMap<String, Vec<String>>,
    code_editor_sources: std::collections::HashMap<String, Vec<String>>,

    /// State var types for lookup during handler generation
    state_types: std::collections::HashMap<String, String>,

    /// Prop names for lookup during handler generation (to add self. prefix)
    prop_names: std::collections::HashSet<String>,

    /// Prop types for checking if a prop needs Value index access
    prop_types: std::collections::HashMap<String, String>,

    /// Prop names whose type is a user-defined type alias (e.g., Note).
    /// These need serde_json::Value bracket access (self.note["field"]).
    value_prop_names: std::collections::HashSet<String>,

    /// Computed property method names (for adding () in dot access)
    computed_names: std::collections::HashSet<String>,

    /// Loop variables that iterate over Value-type collections (need ["field"] access)
    value_loop_vars: std::collections::HashSet<String>,
    /// PLAN-026 T-03 配套: handler 局部 List<int> 变量(db 层惯例发射
    /// Vec<i64>;元素消费统一窄化 as i32,与 i32 模型字段/字面语境对齐)。
    handler_int_list_vars: std::cell::RefCell<std::collections::HashSet<String>>,

    /// Local variables in handler bodies that hold serde_json::Value results
    /// (from API function calls like `let note = create_note(...)`)
    value_locals: std::collections::HashSet<String>,
    /// PLAN-039 T-12（批次 E）：无类型集合局部（`var scored = []`）——
    /// Vec<Value> 格（原生 push/pop/len；元素索引/字段访问降链）。
    array_locals: std::collections::HashSet<String>,
    /// PLAN-039 T-13（批次 E，E-D2）：局部记录字面量的子字段形状
    /// （局部名 → 子字段 → int/str/bool）——`row.score` 访问器选型。
    local_record_shapes:
        std::collections::HashMap<String, std::collections::HashMap<String, String>>,
    /// PLAN-039 T-14（E-D2）：循环变量 → 迭代集合名映射（元素形状查询）。
    loop_var_collections: std::collections::HashMap<String, String>,
    /// PLAN-039 T-14（E-D3 第一轨）：显式声明局部型（名字 → "int"|"str"|
    /// "bool"）——后续 Asn 赋值的强转判据（`var card_id int = 0` 后
    /// `card_id = .col0_cards[i]` 的 __at_num 株）。
    declared_locals: std::collections::HashMap<String, String>,
    /// PLAN-039 T-13（E-D2）：无类型集合局部的元素形状（数组名 → 子字段
    /// 表）——`scored[i].score` 元素字段访问器选型（push 实参/字面量推）。
    array_element_shapes:
        std::collections::HashMap<String, std::collections::HashMap<String, String>>,

    /// Whether the widget has an .Init lifecycle handler
    has_init: bool,

    /// Info about the API function called in .Init handler (async init generation)
    init_api_info: Option<InitApiInfo>,

    /// PLAN-039 D5（§5.1 定案记录）：outlet 折平策略——generate_rust 读
    /// widget.routes 装配，Outlet 臂消费。
    outlet_route: OutletRoute,
}

/// PLAN-039 D5（§5.1 定案记录）：`routes{"/"->use X}` + `outlet` 三态。
#[derive(Clone, Debug, PartialEq)]
enum OutletRoute {
    /// 无 routes 块——outlet 维持 View::empty（防御形态，现行为）。
    None,
    /// 单路由 "/" 无参——outlet 折平为持久子件 X 直用。
    Fold(String),
    /// 多路由/带参路由——outlet 位响亮拒（compile_error 带 P039 债指针）。
    Reject,
}

// Plan 346: Thread-local store for the root widget's state field names + types.
// Populated when the root widget (App) is generated; read by child widgets
// (EditorPanel, etc.) so their structs include parent state fields for unified
// state access (mirrors VM path's Plan 320 override_state_obj_id).
thread_local! {
    static ROOT_STATE_FIELDS: std::cell::RefCell<Vec<(String, String)>> =
        std::cell::RefCell::new(Vec::new());
    /// Plan 374: store composable names (e.g. {"store" => "NotesStore"}).
    pub static STORE_NAMES: std::cell::RefCell<std::collections::HashMap<String, String>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
    /// Plan 374: store computed property names (for adding () in dot access).
    pub static STORE_COMPUTED_NAMES: std::cell::RefCell<std::collections::HashSet<String>> =
        std::cell::RefCell::new(std::collections::HashSet::new());
    /// PLAN-039 T-12（批次 E）：store 字段类型表（store 名 → 字段 → Rust 型）。
    /// app.at 与 *_store.at 分文件编译（collect_at_files 字母序 app 在前），
    /// 生成 app widget 时 state_types 只含自身字段——`.store.<field>.<sub>`
    /// 多级链要判定中间级是否 Value（klondike waste_card 记录字面量株）
    /// 需要跨 widget 的字段型视图。build 入口 collect_store_decls 后预填。
    pub static STORE_FIELD_TYPES: std::cell::RefCell<
        std::collections::HashMap<String, std::collections::HashMap<String, String>>,
    > = std::cell::RefCell::new(std::collections::HashMap::new());
    /// PLAN-039 T-13（批次 E，E-D2 记录形状注册表）：store 记录字段的
    /// 子字段型表（store 名 → 记录字段名 → 子字段 → int/str/bool）——
    /// 记录字面量默认值推断（`waste_card = { id: 0, rank: 0, svg_src: "" }`
    /// → id:int/rank:int/svg_src:str）。字段访问按表型选访问器
    /// （as_i64/as_bool/as_str），未知子字段回落启发式（E-D2 动态容差）。
    pub static STORE_RECORD_SHAPES: std::cell::RefCell<
        std::collections::HashMap<
            String,
            std::collections::HashMap<String, std::collections::HashMap<String, String>>,
        >,
    > = std::cell::RefCell::new(std::collections::HashMap::new());
    /// PLAN-039 T-13（批次 E，E-D5-A 配套）：返回类型化 user 型的 api 桩
    /// 函数名集（rust_ui.rs parse_api_module 后注册）——此类调用的赋值
    /// 局部是 typed（CardsResult 等），scan 收格与类型格判定不得收 Value。
    pub static API_TYPED_FNS: std::cell::RefCell<std::collections::HashSet<String>> =
        std::cell::RefCell::new(std::collections::HashSet::new());
    /// PLAN-039 T-14（批次 E，组件传型）：子组件 prop 型表（组件名 →
    /// prop → "int"|"str"|"bool"）——构造参数按目标 prop 型强转
    /// （`scard["rank"].as_str()…` 传给 `rank: i32` 的 as_i64 收口 =
    /// showcase_cards 记录集合元素形状跨文件不可见时的正路修复）。
    pub static CHILD_PROP_TYPES: std::cell::RefCell<
        std::collections::HashMap<String, std::collections::HashMap<String, String>>,
    > = std::cell::RefCell::new(std::collections::HashMap::new());
    /// PLAN-039 T-14（批次 E，组件传型）：子组件 Msg 型表（组件名 →
    /// Msg 型名或 "()"）——无 msg 块的子组件（CardSuit/CourtBadge）的
    /// 父包装变体此前恒发 `{Child}Msg` 而该 enum 不存在（E0425 株）。
    /// 生成期自注册 + rust_ui.rs 预扫双保险（app.at 先编译）。
    pub static CHILD_MSG_TYPES: std::cell::RefCell<std::collections::HashMap<String, String>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
    /// Plan 374: widget prop declaration order (widget_name → ordered prop names).
    /// Used to ensure constructor args are emitted in the correct order.
    pub static WIDGET_PROP_ORDERS: std::cell::RefCell<std::collections::HashMap<String, Vec<String>>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

/// Detected Init handler pattern: `self.state_var = api_func()`
struct InitApiInfo {
    /// State variable being assigned (e.g., "notes")
    state_var: String,
}

impl RustGenerator {
    /// Create a new Rust generator
    pub fn new() -> Self {
        Self {
            current_widget: None,
            api_imports: Vec::new(),
            message_variants: Vec::new(),
            needs_imports: true,
            indent: 0,
            child_components: Vec::new(),
            loop_child_components: std::collections::HashSet::new(),
            component_state_fields: std::collections::HashMap::new(),
            component_semantics: std::collections::HashMap::new(),
            loop_vars: Vec::new(),
            input_fields: std::collections::HashMap::new(),
            code_editor_sources: std::collections::HashMap::new(),
            state_types: std::collections::HashMap::new(),
            prop_names: std::collections::HashSet::new(),
            prop_types: std::collections::HashMap::new(),
            value_prop_names: std::collections::HashSet::new(),
            computed_names: std::collections::HashSet::new(),
            value_loop_vars: std::collections::HashSet::new(),
            handler_int_list_vars: std::cell::RefCell::new(std::collections::HashSet::new()),
            value_locals: std::collections::HashSet::new(),
            array_locals: std::collections::HashSet::new(),
            local_record_shapes: std::collections::HashMap::new(),
            loop_var_collections: std::collections::HashMap::new(),
            declared_locals: std::collections::HashMap::new(),
            array_element_shapes: std::collections::HashMap::new(),
            has_init: false,
            init_api_info: None,
            outlet_route: OutletRoute::None,
        }
    }

    /// Plan 374: Register a store composable name.
    pub fn register_store(&mut self, alias: &str, store_name: &str) {
        STORE_NAMES.with(|sn| {
            sn.borrow_mut().insert(alias.to_string(), store_name.to_string());
        });
    }

    /// PLAN-039 T-12（批次 E）：预填 store 字段类型表（build 入口在
    /// collect_store_decls 后、逐文件编译前调用）——推断口径与
    /// generate_rust 的 state_types 填充同型（Unknown 时按初始式
    /// Array→Vec<Value>/Object→Value/…），保证 `.store.x.y` 降链判定
    /// 与 store 本体生成物的字段型一字不差。
    /// PLAN-039 T-13（E-D2）：记录字面量初始式同时登记子字段形状表
    /// （默认值字面量推 int/str/bool）——字段访问按表型选访问器。
    pub fn prime_store_field_types(&mut self, decl: &crate::ast::ui::StoreDecl) {
        let mut fields = std::collections::HashMap::new();
        let mut shapes = std::collections::HashMap::new();
        if let Some(model) = &decl.model {
            for field in &model.fields {
                let ty = if matches!(field.ty, crate::ast::Type::Unknown) {
                    match &field.init {
                        crate::ast::Expr::Array(_) => "Vec<serde_json::Value>".to_string(),
                        crate::ast::Expr::Object(_) => "serde_json::Value".to_string(),
                        crate::ast::Expr::Str(_) => "String".to_string(),
                        crate::ast::Expr::Int(_) => "i32".to_string(),
                        crate::ast::Expr::Float(_, _) | crate::ast::Expr::Double(_, _) => {
                            "f64".to_string()
                        }
                        crate::ast::Expr::Bool(_) => "bool".to_string(),
                        _ => "serde_json::Value".to_string(),
                    }
                } else {
                    self.auto_type_to_rust(&field.ty)
                };
                // E-D2：记录字面量子字段形状（waste_card = { id: 0, … }）。
                if ty == "serde_json::Value" {
                    if let Some(shape) = self.object_literal_shape(&field.init) {
                        shapes.insert(field.name.as_str().to_string(), shape);
                    }
                }
                fields.insert(field.name.as_str().to_string(), ty);
            }
        }
        let store_name = decl.name.as_str().to_string();
        STORE_FIELD_TYPES.with(|m| {
            m.borrow_mut().insert(store_name.clone(), fields);
        });
        STORE_RECORD_SHAPES.with(|m| {
            m.borrow_mut().insert(store_name, shapes);
        });
    }

    /// PLAN-039 T-13（批次 E，E-D5-A 配套）：注册返回类型化 user 型的
    /// api 桩函数名（rust_ui.rs parse 后调用）——此类调用的赋值局部是
    /// typed，scan 收格不收 Value（kanban `let r = board_cards(..)` 的
    /// r.cards 直达字段株）。
    pub fn register_api_typed_fns(&mut self, fns: std::collections::HashSet<String>) {
        API_TYPED_FNS.with(|m| {
            let mut m = m.borrow_mut();
            m.clear();
            m.extend(fns);
        });
    }

    /// PLAN-039 T-12：store 字段（跨 widget 视图）的 Rust 型——查
    /// STORE_FIELD_TYPES，按 STORE_NAMES 注册的 store 名限域。
    fn store_field_rust_type(&self, field: &str) -> Option<String> {
        STORE_NAMES.with(|sn| {
            let names: Vec<String> = sn.borrow().values().cloned().collect();
            STORE_FIELD_TYPES.with(|m| {
                let m = m.borrow();
                names.iter().find_map(|store| {
                    m.get(store).and_then(|f| f.get(field)).cloned()
                })
            })
        })
    }

    /// PLAN-039 T-13：store 字段是否 Value 型（多级点链降链判据）。
    fn store_field_is_value(&self, field: &str) -> bool {
        self.store_field_rust_type(field)
            .map_or(false, |ty| ty == "serde_json::Value")
    }

    /// PLAN-039 T-13（批次 E，E-D2）：记录字面量的子字段形状——默认值
    /// 字面量推型（Int/I64→int、Bool→bool、其余含 Str→str 容错）。
    /// 非记录字面量返回 None。
    fn object_literal_shape(
        &self,
        expr: &crate::ast::Expr,
    ) -> Option<std::collections::HashMap<String, String>> {
        if let crate::ast::Expr::Object(pairs) = expr {
            let shape: std::collections::HashMap<String, String> = pairs
                .iter()
                .filter_map(|p| {
                    let key = match &p.key {
                        crate::ast::Key::NamedKey(n) => n.as_str().to_string(),
                        crate::ast::Key::StrKey(s) => s.to_string(),
                        _ => return None,
                    };
                    // PLAN-039 T-14（E-D2）：非字面量值按声明/态型推
                    // （`score: score` 局部 int → int——此前恒 str，
                    // launcher scored 行收集株的字段访问器错源）。
                    let kind = match p.value.as_ref() {
                        crate::ast::Expr::Int(_) | crate::ast::Expr::I64(_) => "int",
                        crate::ast::Expr::Bool(_) => "bool",
                        crate::ast::Expr::Str(_) | crate::ast::Expr::CStr(_)
                        | crate::ast::Expr::FStr(_) => "str",
                        crate::ast::Expr::Ident(n) => {
                            let nm = n.as_str();
                            let nm = nm.strip_prefix('.').unwrap_or(nm);
                            if let Some(k) = self.declared_locals.get(nm) {
                                k.as_str()
                            } else if let Some(ty) = self.state_types.get(nm) {
                                match ty.as_str() {
                                    "i32" | "i64" | "u32" | "u64" | "f32" | "f64" => "int",
                                    "bool" => "bool",
                                    _ => "str",
                                }
                            } else {
                                "str"
                            }
                        }
                        crate::ast::Expr::Bina(_, op, _)
                            if matches!(
                                op,
                                auto_val::Op::Add
                                    | auto_val::Op::Sub
                                    | auto_val::Op::Mul
                                    | auto_val::Op::Div
                                    | auto_val::Op::Mod
                            ) =>
                        {
                            "int"
                        }
                        _ => "str",
                    };
                    Some((key, kind.to_string()))
                })
                .collect();
            if shape.is_empty() {
                None
            } else {
                Some(shape)
            }
        } else {
            None
        }
    }

    /// PLAN-039 T-13（批次 E，E-D2）：store 记录字段的子字段形状表查询
    /// （按 STORE_NAMES 注册的 store 名限域）。None = 非记录字段/未知
    /// 形状 → 访问器回落启发式（动态容差）。
    fn store_record_shape(
        &self,
        field: &str,
    ) -> Option<std::collections::HashMap<String, String>> {
        STORE_NAMES.with(|sn| {
            let names: Vec<String> = sn.borrow().values().cloned().collect();
            STORE_RECORD_SHAPES.with(|m| {
                let m = m.borrow();
                names
                    .iter()
                    .find_map(|s| m.get(s).and_then(|f| f.get(field)).cloned())
            })
        })
    }

    /// PLAN-039 T-13（批次 E，E-D2）：Value 字段访问的形状型访问器——
    /// 按子字段表型选 as_i64/as_bool/as_str（klondike waste_card.rank
    /// int 株：启发式名单外字段此前恒 as_str → CardFace 参数错位）；
    /// 无形状回落 value_field_access 启发式（未知字段动态容差）。
    fn value_field_access_shaped(
        &self,
        obj_expr: &str,
        field: &str,
        shape: Option<&std::collections::HashMap<String, String>>,
    ) -> String {
        match shape.and_then(|s| s.get(field)).map(|s| s.as_str()) {
            Some("int") => {
                format!("({}[\"{}\"].as_i64().unwrap_or(0) as i32)", obj_expr, field)
            }
            Some("bool") => format!("({}[\"{}\"].as_bool().unwrap_or(false))", obj_expr, field),
            _ => self.value_field_access(obj_expr, field),
        }
    }

    /// PLAN-039 T-12（批次 E）：Dot 真链接收者的 store Value 基座判定——
    /// obj 剥层后为 `.store`/`self.store` 基座 + 尾字段 F，且 F 是 store
    /// Value 字段时返回 `self.store.<F>` 基座串（E-D1 使用点包裹的访问
    /// 接收者）。收 Dot(Dot(self, store), F) 三层与 Dot(store, F) 双层
    /// 两形态；更深/非 Value 返回 None 落回默认路径。
    /// PLAN-039 T-13（E-D2）：同程返回记录形状表（F 的子字段型）。
    fn dot_chain_store_value_base(
        &self,
        obj: &crate::ast::Expr,
    ) -> Option<(String, Option<std::collections::HashMap<String, String>>)> {
        use crate::ast::Expr;
        if let Expr::Dot(mid, f) = obj {
            let base_is_store = match mid.as_ref() {
                Expr::Dot(base, b) => {
                    matches!(base.as_ref(), Expr::Ident(n)
                        if n.as_str() == "self" || n.as_str() == ".")
                        && b.as_str() == "store"
                }
                Expr::Ident(n) => n.as_str() == "store" || n.as_str() == ".store",
                _ => false,
            };
            let field = f.as_str();
            if base_is_store && !field.contains('.') && self.store_field_is_value(field) {
                let shape = self.store_record_shape(field);
                return Some((format!("self.store.{}", field), shape));
            }
        }
        None
    }

    /// Plan 371 Task 22c: record a component's own scalar state fields so
    /// parents that use this component can hoist+sync them.
    pub fn register_component_state(
        &mut self,
        component_name: &str,
        fields: Vec<(String, String)>,
    ) {
        self.component_state_fields
            .insert(component_name.to_string(), fields);
    }

    /// Plan 371 L1: record a component's semantics (which props its handlers
    /// write), so the parent's child-forwarding arm can generate general
    /// prop-writeback instead of hardcoding "note".
    pub fn register_component_semantics(
        &mut self,
        component_name: &str,
        semantics: ComponentSemantics,
    ) {
        self.component_semantics
            .insert(component_name.to_string(), semantics);
    }

    /// Reset state for new widget
    fn reset(&mut self) {
        self.message_variants.clear();
        self.input_fields.clear();
        self.code_editor_sources.clear();
        self.state_types.clear();
        self.prop_names.clear();
        self.prop_types.clear();
        self.value_prop_names.clear();
        self.computed_names.clear();
        self.value_loop_vars.clear();
        self.value_locals.clear();
        self.array_locals.clear();
        self.local_record_shapes.clear();
        self.loop_var_collections.clear();
        self.declared_locals.clear();
        self.array_element_shapes.clear();
        self.child_components.clear();
        self.loop_child_components.clear();
        self.has_init = false;
        self.init_api_info = None;
        self.needs_imports = true;
        self.indent = 0;
        self.loop_vars.clear();
        self.outlet_route = OutletRoute::None;
    }

    /// Convert a string containing `${.field}` markers to a Rust `format!()` call
    fn interpolate_str(&self, s: &str) -> String {
        let mut format_str = s.to_string();
        let mut format_args = Vec::new();

        // Extract ${.field} and ${field} patterns
        let re = regex::Regex::new(r"\$\{\.?(\w+)\}").unwrap();
        for cap in re.captures_iter(s) {
            let binding = &cap[1];
            let arg = if self.is_loop_var(binding) {
                binding.to_string()
            } else {
                format!("self.{}", binding)
            };
            if !format_args.contains(&arg) {
                format_args.push(arg);
            }
        }

        // Replace ${.field} and ${field} with {}
        format_str = re.replace_all(&format_str, "{}").to_string();

        if format_args.is_empty() {
            format!("\"{}\"", s)
        } else {
            format!("format!(\"{}\", {})", format_str, format_args.join(", "))
        }
    }

    /// Get the widget-specific Msg enum name (e.g., "AppMsg", "EditorPanelMsg")
    fn current_msg_name(&self) -> String {
        match &self.current_widget {
            Some(name) => format!("{}Msg", name),
            None => "Msg".to_string(),
        }
    }

    /// Get the Rust type for a state var, using refined type from initial expression
    fn state_rust_type(&self, state: &crate::aura::AuraStateDef) -> String {
        self.state_types.get(&state.name)
            .cloned()
            .unwrap_or_else(|| self.auto_type_to_rust(&state.type_info))
    }

    /// Get the Rust type for a prop
    fn prop_rust_type(&self, prop: &crate::aura::AuraProp) -> String {
        self.auto_type_to_rust(&prop.type_info)
    }

    /// Check if any handler body accesses prop_name.field (dot access on a prop)
    fn prop_needs_value_type(&self, widget: &AuraWidget, prop_name: &str) -> bool {
        for (_pattern, payload) in &widget.handlers {
            let body_str = self.generate_handler_body(payload);
            // Look for self.{prop_name}.field patterns
            if body_str.contains(&format!("self.{}.", prop_name)) {
                return true;
            }
        }
        false
    }

    /// Check if the view tree contains field access on a prop (e.g., note.title)
    /// indicating the prop needs to be serde_json::Value, not String
    fn view_accesses_prop_field(&self, node: &AuraNode, prop_name: &str) -> bool {
        match node {
            AuraNode::Element { props, children, .. } => {
                // Check if any prop value is a FieldAccess on our prop
                for (_key, value) in props {
                    if let crate::aura::AuraPropValue::Expr(expr) = value {
                        if self.expr_accesses_field(expr, prop_name) {
                            return true;
                        }
                    }
                }
                for child in children {
                    if self.view_accesses_prop_field(child, prop_name) {
                        return true;
                    }
                }
            }
            AuraNode::ForLoop { body, .. } => {
                for child in body {
                    if self.view_accesses_prop_field(child, prop_name) {
                        return true;
                    }
                }
            }
            AuraNode::Conditional { then_body, else_body, .. } => {
                for child in then_body {
                    if self.view_accesses_prop_field(child, prop_name) {
                        return true;
                    }
                }
                if let Some(else_nodes) = else_body {
                    for child in else_nodes {
                        if self.view_accesses_prop_field(child, prop_name) {
                            return true;
                        }
                    }
                }
            }
            _ => {}
        }
        false
    }

    /// Check if an expression accesses a field on the given prop name
    fn expr_accesses_field(&self, expr: &crate::ast::Expr, prop_name: &str) -> bool {
        use crate::ast::Expr;
        match expr {
            Expr::Dot(object, _field) => {
                if let Expr::Ident(name) = object.as_ref() {
                    let resolved = if name.starts_with('.') { &name[1..] } else { name.as_str() };
                    if resolved == prop_name {
                        return true;
                    }
                }
                self.expr_accesses_field(object, prop_name)
            }
            Expr::Bina(left, _op, right) => {
                self.expr_accesses_field(left, prop_name) || self.expr_accesses_field(right, prop_name)
            }
            Expr::Call(call) => {
                self.expr_accesses_field(&call.name, prop_name)
                    || call.args.args.iter().any(|a| {
                        if let crate::ast::Arg::Pos(e) | crate::ast::Arg::Pair(_, e) = a {
                            self.expr_accesses_field(e, prop_name)
                        } else {
                            false
                        }
                    })
            }
            Expr::Index(target, index) => {
                self.expr_accesses_field(target, prop_name)
                    || self.expr_accesses_field(index, prop_name)
            }
            _ => false,
        }
    }

    /// Check if a name is a loop variable
    fn is_loop_var(&self, name: &str) -> bool {
        self.loop_vars.contains(&name.to_string())
    }

    /// Check if a dot access target needs index syntax (target["field"] instead of target.field)
    fn needs_index_access(&self, target_name: &str) -> bool {
        // Plan 547: the viewer `names()` endpoint is a native `Vec<String>`
        // result, not a serde_json::Value. Keep its local field/index access
        // in normal Rust form (`names_list.len()` / `names_list[i]`).
        if target_name == "names_list" {
            return false;
        }
        // Props that are actually serde_json::Value type
        if let Some(ty) = self.prop_types.get(target_name) {
            if ty == "serde_json::Value" {
                return true;
            }
        }
        // Plan 374: User-defined type props (like Note) need Value bracket access
        if self.value_prop_names.contains(target_name) {
            return true;
        }
        // State vars that are serde_json::Value (not Vec<Value>)
        if let Some(ty) = self.state_types.get(target_name) {
            return ty == "serde_json::Value";
        }
        // Loop variables iterating over Value-type collections
        if self.value_loop_vars.contains(target_name) {
            return true;
        }
        // Local variables from function call results (likely serde_json::Value)
        if self.value_locals.contains(target_name) {
            return true;
        }
        false
    }

    /// Push loop variables into scope
    fn push_loop_vars(&mut self, var: &str, index: Option<&str>) {
        self.loop_vars.push(var.to_string());
        if let Some(idx) = index {
            self.loop_vars.push(idx.to_string());
        }
    }

    /// Pop loop variables from scope
    fn pop_loop_vars(&mut self, var: &str, index: Option<&str>) {
        self.loop_vars.retain(|v| v != var);
        if let Some(idx) = index {
            self.loop_vars.retain(|v| v != idx);
        }
    }

    /// Generate complete Rust code from AuraWidget
    pub fn generate_rust(&mut self, widget: &AuraWidget) -> GenResult<String> {
        self.current_widget = Some(widget.name.clone());
        // PLAN-627: 限定名 api.X() 改写门数据（reset 不清——widget 级装载）。
        self.api_imports = widget.api_imports.clone();
        self.reset();

        // Plan 436 T1(决策 1-A):setup 前导槽是 a2vue 语义(script setup
        // 顶层,每组件实例同步执行、先于首渲染);Rust 目标的 Elm 架构
        // (struct + new + on/view)尚无每实例 setup 槽位——显式报错止血,
        // 不再静默丢弃(PLAN-037 T7 哲学,同 use.web 门控先例)。1-B 生成
        // (new() 后、首次 view() 前注入)需将 setup 绑定接入 struct 字段
        // 与视图/处理器的标识符解析,面较大,留待需要时立项。
        if widget.setup.is_some() {
            return Err(GenError::UnsupportedStmt(format!(
                "widget `{}` declares a `setup {{}}` block; setup is vue-render only for now — the Rust target has no per-instance setup slot yet (Plan 436 决策 1-A)",
                widget.name
            )));
        }

        // Populate state_types for handler generation
        for state in &widget.state_vars {
            let ty = if matches!(state.type_info, crate::ast::Type::Unknown) {
                // Infer type from initial expression for untyped state vars
                match &state.initial {
                    crate::ast::Expr::Array(_) => "Vec<serde_json::Value>".to_string(),
                    crate::ast::Expr::Object(_) => "serde_json::Value".to_string(),
                    crate::ast::Expr::Str(_) => "String".to_string(),
                    crate::ast::Expr::Int(_) => "i32".to_string(),
                    crate::ast::Expr::Float(_, _) | crate::ast::Expr::Double(_, _) => "f64".to_string(),
                    crate::ast::Expr::Bool(_) => "bool".to_string(),
                    _ => self.auto_type_to_rust(&state.type_info),
                }
            } else {
                self.auto_type_to_rust(&state.type_info)
            };
            self.state_types.insert(state.name.clone(), ty);
        }

        // Populate prop_names and prop_types for self. prefix resolution and type checking
        for prop in &widget.props {
            self.prop_names.insert(prop.name.clone());
            let mut prop_ty = self.prop_rust_type(prop);
            // Apply the same Value upgrade logic as generate_struct
            if self.prop_needs_value_type(widget, &prop.name) && prop_ty == "String" {
                prop_ty = "serde_json::Value".to_string();
            }
            // Also check if the view tree accesses fields on this prop (e.g., note.title)
            // which means it needs to be serde_json::Value, not String
            if prop_ty == "String" && self.view_accesses_prop_field(&widget.view_tree, &prop.name) {
                prop_ty = "serde_json::Value".to_string();
            }
            self.prop_types.insert(prop.name.clone(), prop_ty.clone());

            // Plan 374: Track user-defined type props (like `Note`) that need
            // serde_json::Value bracket access (self.prop["field"]).
            if matches!(&prop.type_info, crate::ast::Type::User(_)) {
                self.value_prop_names.insert(prop.name.clone());
            }
        }

        // Collect all message variants
        for msg in &widget.messages {
            for variant in &msg.variants {
                self.message_variants.push(variant.clone());
            }
        }

        // Plan 374: Collect computed property names for method-call syntax
        for computed in &widget.computed {
            self.computed_names.insert(computed.name.clone());
        }

        // PLAN-039 T-14（组件传型）：自注册 Msg 型（无 msg 块且无子件
        // → "()"；有子件 → {}Msg——与 msg_type 的 child gate 同口径）。
        {
            let msg_ty = if widget.messages.is_empty() && widget_has_children(widget) {
                format!("{}Msg", widget.name)
            } else if widget.messages.is_empty() {
                "()".to_string()
            } else {
                format!("{}Msg", widget.name)
            };
            CHILD_MSG_TYPES.with(|m| {
                m.borrow_mut().insert(widget.name.clone(), msg_ty);
            });
        }

        let mut code = String::new();

        // File header
        code.push_str("// Auto-generated from Auto language\n");
        code.push_str("// DO NOT EDIT - changes will be overwritten\n\n");

        // Imports
        if self.needs_imports {
            code.push_str("use auto_lang::ui::{Component, View};\n\n");
        }

        // Pre-scan view tree for child component references (needed for wrapper msg variants)
        self.scan_child_components(&widget.view_tree);

        // PLAN-039 D5（§5.1 定案记录）：单路由折平策略装配——必须在
        // generate_msg_enum（:605）前完成 child_components 注册。折平
        // module 走既有持久子件全链（msg 包装变体 :659-662 / struct 字段
        // :733-740 / 构造后重建 :970-988 / on() 转发+store 回写
        // :1461-1490）；多路由/带参标记 Reject（Outlet 位响亮拒）。
        self.outlet_route = match &widget.routes {
            None => OutletRoute::None,
            Some(rb) if rb.routes.len() == 1
                && rb.routes[0].path == "/"
                && rb.routes[0].params.is_empty() =>
            {
                let module = rb.routes[0].module.clone();
                if !self.child_components.contains(&module) {
                    self.child_components.push(module.clone());
                }
                OutletRoute::Fold(module)
            }
            Some(_) => OutletRoute::Reject,
        };

        // Pre-scan handlers to find local variables from function calls (likely Value type)
        self.scan_handler_locals(widget);

        // Scan lifecycle handlers (.Init, .Destroy) for local variables and has_init flag
        for lc in &widget.lifecycle {
            if lc.name == "Init" {
                self.has_init = true;
                // Detect async Init pattern: self.X = api_func()
                self.detect_init_api_call(&lc.payload, &widget.api_imports);
            }
            self.scan_payload_locals(&lc.payload);
        }

        // If there's an .Init lifecycle handler, add Init variant to message enum
        if self.has_init {
            if !self.message_variants.iter().any(|v| v.name == "Init") {
                self.message_variants.push(AuraMsgVariant {
                    name: "Init".to_string(),
                    quoted: false,
                    payload: vec![],
                    payload_names: vec![],
                });
            }
            // If Init calls an API function (async init), add __InitLoaded variant
            // We can't use AuraMsgVariant because Vec<serde_json::Value> doesn't map
            // to any AST Type variant. Instead, inject it directly in generate_msg_enum.
            // (See generate_msg_enum for the direct string injection.)
        }

        // If widget has a tick_interval, add Tick variant to message enum.
        // Timer-block entries already name their message variants, but keep a
        // defensive insertion here so generated Rust remains total even when a
        // hand-built AuraWidget omits the declaration from `msg { ... }`.
        if widget.tick_interval.is_some() {
            if !self.message_variants.iter().any(|v| v.name == "Tick") {
                self.message_variants.push(AuraMsgVariant {
                    name: "Tick".to_string(),
                    quoted: false,
                    payload: vec![],
                    payload_names: vec![],
                });
            }
        }
        for timer in &widget.timers {
            if !self.message_variants.iter().any(|v| v.name == timer.event) {
                self.message_variants.push(AuraMsgVariant {
                    name: timer.event.clone(),
                    quoted: false,
                    payload: vec![],
                    payload_names: vec![],
                });
            }
        }

        // PLAN-533 T4: on-only handlers（无 msg 块声明的 vue 风格源——gallery
        // 页/探针均此形态）此前静默跳过（generate_on_method 的 Plan 374 skip）
        // → 生成 type Msg = ()，而 view 派发闭包仍引用 <Widget>Msg::<variant>
        // 的悬垂路径，编译断。rust 轨在此把 handler 的零参变体补进枚举，
        // 枚举/match/派发三方一致。带参 on-only handler 仍走悬垂编译错
        // （响亮失败：payload 类型无法从 on 块推断）。
        for pattern in widget.handlers.keys() {
            let variant_name = self.extract_variant_name(pattern);
            if !self.message_variants.iter().any(|v| v.name == variant_name) {
                self.message_variants.push(AuraMsgVariant {
                    name: variant_name,
                    quoted: false,
                    payload: vec![],
                    payload_names: vec![],
                });
            }
        }

        // Message enum (includes wrapper variants for child components + Init lifecycle)
        if !self.message_variants.is_empty() || !self.child_components.is_empty() {
            code.push_str(&self.generate_msg_enum()?);
            code.push('\n');
        }

        // Struct definition
        code.push_str(&self.generate_struct(widget));
        code.push('\n');

        // Constructor
        code.push_str(&self.generate_constructor(widget));
        code.push('\n');

        // Pre-scan view tree for input event→field mappings
        self.scan_input_fields(&widget.view_tree);

        // Component impl
        code.push_str(&self.generate_component_impl(widget));

        // Computed properties impl (if any)
        if !widget.computed.is_empty() {
            code.push('\n');
            code.push_str(&self.generate_computed_impl(widget));
        }

        // NOTE: API function stubs are generated at the file level in rust_ui.rs,
        // not per-widget, to avoid duplicate definitions.

        Ok(code)
    }

    /// Generate Msg enum definition
    fn generate_msg_enum(&self) -> GenResult<String> {
        let mut code = String::new();
        let msg_name = self.current_msg_name();

        code.push_str("#[derive(Clone, Debug, PartialEq)]\n");
        code.push_str(&format!("pub enum {} {{\n", msg_name));

        for variant in &self.message_variants {
            if !variant.payload.is_empty() {
                // Plan 043 M5 #1: emit each payload type as a tuple field, so
                // `Complete(str, int)` → `Complete(String, i32)` and a single
                // `Set(int)` → `Set(i32)`.
                let ty_strs: Vec<String> = variant.payload.iter()
                    .map(|t| self.auto_type_to_rust(t))
                    .collect();
                code.push_str(&format!("    {}({}),\n", variant.name, ty_strs.join(", ")));
            } else {
                code.push_str(&format!("    {},\n", variant.name));
            }
        }

        // Add wrapper variants for child components (e.g., EditorPanel(EditorPanelMsg))
        // PLAN-039 T-14（组件传型）：无 msg 子组件（type Msg = ()）变体
        // 载荷发 ()——此前恒 `{Child}Msg` 而该 enum 不存在（E0425 株）。
        for child_name in &self.child_components {
            let child_msg = CHILD_MSG_TYPES.with(|m| {
                m.borrow()
                    .get(child_name)
                    .cloned()
                    .unwrap_or_else(|| format!("{}Msg", child_name))
            });
            code.push_str(&format!("    {}({}),\n", child_name, child_msg));
        }

        // If async Init detected, add __InitLoaded variant (injected as raw string
        // because Vec<serde_json::Value> doesn't map to any AST Type variant)
        if self.init_api_info.is_some() {
            code.push_str(&format!("    __InitLoaded(Vec<serde_json::Value>),\n"));
        }

        code.push_str("}\n");

        Ok(code)
    }

    fn generate_struct(&self, widget: &AuraWidget) -> String {
        let mut code = String::new();

        // Plan 371 Task 22c / L3: stores + persistent child components need Clone.
        let is_store_itself = STORE_NAMES.with(|sn| {
            sn.borrow().values().any(|s| s.as_str() == widget.name)
        });
        let has_store_field = STORE_NAMES.with(|sn| !sn.borrow().is_empty()) && !is_store_itself;
        let has_persistent_child = self.child_components.iter()
            .any(|c| self.is_persistent_child(c));
        if is_store_itself || has_store_field || has_persistent_child {
            code.push_str("#[derive(Clone, Debug)]\n");
        } else {
            code.push_str("#[derive(Debug)]\n");
        }
        code.push_str(&format!("pub struct {} {{\n", widget.name));

        // Track this widget's own fields (for dedup with root state fields).
        let mut own_fields: std::collections::HashSet<String> = std::collections::HashSet::new();

        // Props (from widget signature, e.g., EditorPanel's `note` parameter)
        // Plan 374: Skip `msg`-typed callback props — they use VM parent-to-child
        // message passing which is replaced by Rust's child-to-parent enum forwarding.
        for prop in &widget.props {
            let field_type = self.prop_types.get(&prop.name)
                .cloned()
                .unwrap_or_else(|| self.prop_rust_type(prop));
            if field_type == "msg" {
                continue; // Skip callback props in Rust output
            }
            code.push_str(&format!("    pub {}: {},\n", prop.name, field_type));
            own_fields.insert(prop.name.clone());
        }

        // Plan 374: Register widget prop declaration order for child component
        // constructor arg ordering.
        {
            let ordered: Vec<String> = widget.props.iter()
                .map(|p| p.name.clone())
                .collect();
            WIDGET_PROP_ORDERS.with(|po| {
                po.borrow_mut().insert(widget.name.clone(), ordered);
            });
        }

        // State variables (use refined types from state_types)
        for state in &widget.state_vars {
            let field_name = &state.name;
            let field_type = self.state_rust_type(state);
            code.push_str(&format!("    pub {}: {},\n", field_name, field_type));
            own_fields.insert(field_name.clone());
        }

        // Plan 371 Task 22c / L3: child component state.
        // - Persistent children (single-instance, not in for-loop): add a persistent
        //   instance field (e.g. `pub editor_panel: EditorPanel`). Their scalar
        //   state lives inside the instance, so we DON'T hoist those fields.
        // - Loop children: hoist their scalar state fields (old behavior).
        for child_name in &self.child_components {
            if self.is_persistent_child(child_name) {
                // L3: persistent instance field.
                let field = Self::child_field_name(child_name);
                if !own_fields.contains(&field) {
                    code.push_str(&format!("    pub {}: {},\n", field, child_name));
                    own_fields.insert(field);
                }
            } else {
                // Loop child: hoist scalar state fields (legacy behavior).
                if let Some(child_fields) = self.component_state_fields.get(child_name) {
                    for (f, ty) in child_fields {
                        if !own_fields.contains(f) {
                            code.push_str(&format!("    pub {}: {},\n", f, ty));
                            own_fields.insert(f.clone());
                        }
                    }
                }
            }
        }

        // Plan 346: If this is a child widget (not App/root), add root state
        // fields that it doesn't already have. This mirrors VM path's Plan 320
        // unified state — child handlers can reference parent state fields.
        let is_root = widget.name == "App";
        // Plan 374 Task 2: for root widget AND child widgets that reference store,
        // add `pub store: StoreName` field.
        // Plan 374: Skip store field injection for the store struct itself
        // to avoid recursive types (NotesStore { store: NotesStore }).
        let is_store_itself = STORE_NAMES.with(|sn| {
            sn.borrow().values().any(|s| s.as_str() == widget.name)
        });
        if !is_store_itself {
            if is_root {
            STORE_NAMES.with(|sn| {
                for (_alias, store_name) in sn.borrow().iter() {
                    code.push_str(&format!("    pub store: {},\n", store_name));
                }
            });
        } else {
            // Child widgets also need store field (they access self.store.X)
            STORE_NAMES.with(|sn| {
                for (_alias, store_name) in sn.borrow().iter() {
                    if !own_fields.contains("store") {
                        code.push_str(&format!("    pub store: {},\n", store_name));
                    }
                }
            });
        }
        } // !is_store_itself — skip store field injection for store structs
        if is_root {
            // Record root state fields for child widgets to pick up.
            ROOT_STATE_FIELDS.with(|rsf| {
                let mut fields = rsf.borrow_mut();
                fields.clear();
                for state in &widget.state_vars {
                    let ty = self.state_rust_type(state);
                    fields.push((state.name.clone(), ty));
                }
            });
        } else {
            // Add root state fields not already in own_fields.
            ROOT_STATE_FIELDS.with(|rsf| {
                let fields = rsf.borrow();
                for (name, ty) in fields.iter() {
                    if !own_fields.contains(name) {
                        code.push_str(&format!("    pub {}: {},\n", name, ty));
                    }
                }
            });
        }

        code.push_str("}\n");

        code
    }

    /// Generate constructor
    fn generate_constructor(&self, widget: &AuraWidget) -> String {
        let widget_name = &widget.name;
        let mut code = String::new();

        code.push_str(&format!("impl {} {{\n", widget_name));

        // new() constructor — accepts props as parameters
        // Plan 374: Skip `msg`-typed callback props in constructor.
        let non_msg_props: Vec<&crate::aura::AuraProp> = widget.props.iter()
            .filter(|p| {
                let ty = self.prop_types.get(&p.name)
                    .cloned()
                    .unwrap_or_else(|| self.prop_rust_type(p));
                ty != "msg"
            })
            .collect();
        let has_props = !non_msg_props.is_empty();
        if has_props {
            let params: Vec<String> = non_msg_props.iter()
                .map(|p| {
                    let ty = self.prop_types.get(&p.name)
                        .cloned()
                        .unwrap_or_else(|| self.prop_rust_type(*p));
                    format!("{}: {}", p.name, ty)
                })
                .collect();
            code.push_str(&format!("    pub fn new({}) -> Self {{\n", params.join(", ")));
        } else {
            code.push_str("    pub fn new() -> Self {\n");
        }

        // If the widget has an .Init lifecycle handler AND it's synchronous (not async API call),
        // dispatch Init message at construction.
        // Async Init (init_api_info is Some) is dispatched by the runtime boot task instead.
        let sync_init = self.has_init && self.init_api_info.is_none();
        // Plan 371 L3: force __self mode if we have persistent children (need
        // post-construct re-initialization with real props).
        let has_persistent_child = self.child_components.iter()
            .any(|c| self.is_persistent_child(c));
        let force_self = sync_init || has_persistent_child;
        if force_self {
            let _msg_name = self.current_msg_name();
            code.push_str("        let mut __self = Self {\n");
        } else {
            code.push_str("        Self {\n");
        }

        // Initialize props from parameters (skip msg-typed callback props)
        for prop in &non_msg_props {
            code.push_str(&format!("            {}: {},\n", prop.name, prop.name));
        }

        // Initialize state vars from their defaults
        for state in &widget.state_vars {
            // PLAN-039 T-12（批次 E）：Vec<Value> 态的数组字面量初始式
            // 逐元素 json! 包裹（launcher `var cats = ["all", …]` 株——
            // 此前元素按 String 发射，vec! 类型与 Vec<Value> 字段断）。
            let init = if self.state_rust_type(state) == "Vec<serde_json::Value>" {
                match &state.initial {
                    crate::ast::Expr::Array(elems) => format!(
                        "vec![{}]",
                        elems
                            .iter()
                            .map(|e| format!(
                                "serde_json::json!({})",
                                self.ast_expr_to_rust_no_to_string(e)
                            ))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    _ => self.ast_expr_to_rust(&state.initial),
                }
            } else {
                self.ast_expr_to_rust(&state.initial)
            };
            code.push_str(&format!("            {}: {},\n", state.name, init));
        }

        // Plan 371 Task 22c / L3: initialize child-component state.
        // - Persistent children: initialize with placeholder (Child::new(zero_values)).
        //   Real props are synced after __self construction (see post-construct block).
        // - Loop children: hoist scalar state fields with defaults (legacy).
        {
            let own_names: std::collections::HashSet<String> = widget
                .state_vars
                .iter()
                .map(|s| s.name.clone())
                .collect();
            for child_name in &self.child_components {
                if self.is_persistent_child(child_name) {
                    // L3: persistent instance — placeholder init via Default.
                    // Real props are applied post-construct below.
                    let field = Self::child_field_name(child_name);
                    if !own_names.contains(&field) {
                        code.push_str(&format!("            {}: {}::default(),\n", field, child_name));
                    }
                } else if let Some(child_fields) = self.component_state_fields.get(child_name) {
                    for (f, ty) in child_fields {
                        if own_names.contains(f) {
                            continue;
                        }
                        let default_val = match ty.as_str() {
                            "bool" => "false".to_string(),
                            "i32" | "u32" | "i64" | "u64" => "0".to_string(),
                            "f32" | "f64" => "0.0".to_string(),
                            _ => String::from("\"\".to_string()"),
                        };
                        code.push_str(&format!("            {}: {},\n", f, default_val));
                    }
                }
            }
        }

        // Plan 346: Initialize root state fields (for child widgets) with defaults.
        if widget.name != "App" {
            let own_names: std::collections::HashSet<String> = widget.state_vars.iter()
                .map(|s| s.name.clone())
                .collect();
            let own_props: std::collections::HashSet<String> = widget.props.iter()
                .map(|p| p.name.clone())
                .collect();
            ROOT_STATE_FIELDS.with(|rsf| {
                let fields = rsf.borrow();
                for (name, ty) in fields.iter() {
                    if !own_names.contains(name) && !own_props.contains(name) {
                        // Generate a type-appropriate default value.
                        let default_val = if ty.starts_with("Vec<") {
                            "vec![]".to_string()
                        } else if ty == "serde_json::Value" {
                            "serde_json::Value::Null".to_string()
                        } else if ty == "i32" {
                            "0".to_string()
                        } else if ty == "bool" {
                            "false".to_string()
                        } else {
                            "\"\".to_string()".to_string()
                        };
                        code.push_str(&format!("            {}: {},\n", name, default_val));
                    }
                }
            });
        }

        // Plan 374 Task 2: initialize store field for all widgets (except store itself).
        // Must come BEFORE sync_init check so both paths include it.
        STORE_NAMES.with(|sn| {
            let has_store = !sn.borrow().is_empty();
            let is_self = sn.borrow().values().any(|s| s.as_str() == widget.name);
            if has_store && !is_self {
                let store_name = sn.borrow().values().next().cloned().unwrap_or_default();
                code.push_str(&format!("            store: {}::new(),\n", store_name));
            }
        });

        // Plan 371 L3: close struct literal. If force_self (sync_init or
        // persistent children), use __self pattern and add post-construct code.
        if force_self {
            let msg_name = self.current_msg_name();
            code.push_str(&format!("        }};\n"));
            // Run Init FIRST so data (e.g. store.notes from list_notes()) is
            // loaded before persistent children are re-constructed with that
            // data. Doing the re-construction before Init reads an empty store
            // (e.g. EditorPanel::new(store.notes[0]) index-out-of-bounds),
            // which panics in split/async-load modes where list_notes() yields
            // instead of returning synchronously. Init handlers that only touch
            // the store (not child components) are safe to run first.
            if sync_init {
                code.push_str(&format!("        __self.on({}::Init);\n", msg_name));
            }
            // L3: re-construct persistent children with real props (now that
            // Init has populated any data they depend on).
            //
            // Guard: if a constructor arg indexes a collection (e.g.
            // `__self.store.notes[idx]`), wrap the re-construction in an
            // `if !<collection>.is_empty() { ... }` guard. In split/async-load
            // modes list_notes() yields instead of returning synchronously, so
            // the collection is still empty right after Init returns — indexing
            // it would panic. The guard leaves the child at its Default() until
            // the view layer re-syncs it with loaded data.
            for child_name in &self.child_components {
                if self.is_persistent_child(child_name) {
                    let field = Self::child_field_name(child_name);
                    let constructor_args = self.find_constructor_args_for_child(widget, child_name);
                    // Replace self. with __self. in constructor args (we're in __self context).
                    let args = constructor_args.replace("self.", "__self.");
                    let stmt = format!(
                        "__self.{} = {}::new({});",
                        field, child_name, args
                    );
                    if let Some(collection) = first_indexed_collection(&args) {
                        code.push_str(&format!(
                            "        if !{}.is_empty() {{\n            {}\n        }}\n",
                            collection, stmt
                        ));
                    } else {
                        code.push_str(&format!("        {}\n", stmt));
                    }
                }
            }
            code.push_str("        __self\n");
        } else {
            code.push_str("        }\n");
        }

        code.push_str("    }\n");
        code.push_str("}\n");

        // Default impl — always generated. For widgets with props, use placeholder
        // values (serde_json::Value::Null for custom types) so persistent-child
        // fields can be initialized with Child::default() in the parent's struct literal.
        if !has_props {
            code.push_str(&format!(
                "impl Default for {} {{\n    fn default() -> Self {{ Self::new() }}\n}}\n",
                widget_name
            ));
        } else {
            // Generate Default with placeholder props matching each prop's type.
            let placeholder_args: Vec<String> = non_msg_props.iter()
                .map(|p| {
                    let ty = self.prop_types.get(&p.name)
                        .cloned()
                        .unwrap_or_else(|| self.prop_rust_type(p));
                    match ty.as_str() {
                        "String" => "\"\".to_string()",
                        "i32" | "u32" | "i64" | "u64" => "0",
                        "f32" | "f64" => "0.0",
                        "bool" => "false",
                        _ => "serde_json::Value::Null",
                    }.to_string()
                })
                .collect();
            code.push_str(&format!(
                "impl Default for {} {{\n    fn default() -> Self {{ Self::new({}) }}\n}}\n",
                widget_name, placeholder_args.join(", ")
            ));
        }

        code
    }

    /// Generate Component trait implementation
    fn generate_component_impl(&mut self, widget: &AuraWidget) -> String {
        let widget_name = &widget.name;
        let mut code = String::new();

        code.push_str(&format!("impl Component for {} {{\n", widget_name));

        // Message type
        let msg_type = if !self.message_variants.is_empty()
            // PLAN-039 T-14（组件传型）：child_components 非空时 msg enum
            // 已发（包装变体在）——type Msg 对齐 enum，子 view 的
            // map_msg 包装链才闭合（CardFace 无 msg 块但有 CardSuit/
            // CourtBadge 子件：View<()> 收 View<CardFaceMsg> 断株）。
            || !self.child_components.is_empty()
        {
            self.current_msg_name()
        } else {
            "()".to_string()
        };
        code.push_str(&format!("    type Msg = {};\n\n", msg_type));

        // on() method
        code.push_str(&self.generate_on_method(widget));
        code.push('\n');

        // view() method
        code.push_str(&self.generate_view_method(widget));

        // Carry declarative AutoUI `bind { ... }` entries into standalone
        // Rust/Iced. The runner resolves the normalized key to a typed msg.
        if !widget.key_bindings.is_empty() {
            let mut bindings: Vec<(&String, &String)> = widget.key_bindings.iter().collect();
            bindings.sort_by(|a, b| a.0.cmp(b.0));
            code.push_str("\n    fn key_bindings(&self) -> std::collections::HashMap<String, String> {\n");
            code.push_str("        let mut bindings = std::collections::HashMap::new();\n");
            for (key, handler) in &bindings {
                code.push_str(&format!(
                    "        bindings.insert({:?}.to_string(), {:?}.to_string());\n",
                    key, handler,
                ));
            }
            code.push_str("        bindings\n    }\n");
            code.push_str("\n    fn key_message(&self, key: &str) -> Option<Self::Msg> {\n");
            code.push_str("        match key {\n");
            for (key, handler) in &bindings {
                let variant = self.extract_variant_name(handler);
                let has_unit_variant = self.message_variants.iter().any(|v| {
                    v.name == variant && v.payload.is_empty()
                });
                if has_unit_variant {
                    code.push_str(&format!(
                        "            {:?} => Some({}::{}),\n",
                        key, msg_type, variant,
                    ));
                }
            }
            code.push_str("            _ => None,\n        }\n    }\n");
        }

        // Plan 371 Task 21: state_snapshot() override — emit only scalar fields
        // (String/i32/i64/u32/u64/f32/f64/bool). Collections and nested components
        // are skipped. Feeds the rust-mode MCP `autoui_state` tool via SharedState.
        let snapshot = self.generate_state_snapshot(widget);
        if !snapshot.is_empty() {
            code.push('\n');
            code.push_str(&snapshot);
        }

        // Plan 407: tick_msg() + tick_interval_ms() — for run_app subscription.
        // Plan 051 C7: the standalone Rust/Iced runner has one generic
        // periodic subscription, so expose the first `timer { ... }` entry
        // through that hook. Desktop/VM mode supports all entries via its
        // dynamic timer registry; this keeps the Rust path useful for the
        // common single-settle-timer case without changing Component's API.
        let periodic_timer = widget.timers.first().map(|timer| {
            let interval = u32::try_from(timer.every_ms).unwrap_or(u32::MAX);
            (interval, timer.event.as_str())
        });
        if let Some((interval, event)) = periodic_timer.or_else(|| {
            widget.tick_interval.map(|interval| (interval, "Tick"))
        }) {
            let msg_name = self.current_msg_name();
            code.push_str(&format!(
                "    fn tick_interval_ms(&self) -> Option<u32> {{ Some({}) }}\n",
                interval
            ));
            code.push_str(&format!(
                "    fn tick_msg(&self) -> Option<{}> {{ Some({}::{}) }}\n",
                msg_name, msg_name, event
            ));
        }

        code.push_str("}\n");

        // Plan 407: ComponentIced is blanket-impl'd in renderer.rs. No explicit
        // impl needed — subscription is built by run_app via tick_msg().
        // The blanket impl provides view_iced() and update().

        code
    }

    /// Generate a `state_snapshot()` override covering the scalar fields of
    /// this component (both props and state vars). Returns empty if there are
    /// no scalar fields (then the trait default empty map is used).
    fn generate_state_snapshot(&self, _widget: &AuraWidget) -> String {
        let mut scalars: Vec<(String, String)> = Vec::new();
        for prop in &_widget.props {
            if let Some(ty) = self.prop_types.get(&prop.name) {
                if is_scalar_state_type(ty) {
                    scalars.push((prop.name.clone(), ty.clone()));
                }
            }
        }
        for state in &_widget.state_vars {
            if let Some(ty) = self.state_types.get(&state.name) {
                if is_scalar_state_type(ty) {
                    scalars.push((state.name.clone(), ty.clone()));
                }
            }
        }
        // Plan 371 Task 22c / L3: include child-component scalar state fields.
        // - Loop children: hoist scalar fields directly (they live on the parent).
        // - Persistent children: their state lives in the instance field, so we
        //   recurse into self.<field>.state_snapshot() instead.
        for child_name in &self.child_components {
            if self.is_persistent_child(child_name) {
                // L3: persistent child — recurse into the instance's snapshot.
                // (handled by the component-typed field recursion below)
            } else if let Some(child_fields) = self.component_state_fields.get(child_name) {
                for (f, ty) in child_fields {
                    if is_scalar_state_type(ty) && !scalars.iter().any(|(n, _)| n == f) {
                        scalars.push((f.clone(), ty.clone()));
                    }
                }
            }
        }

        // Plan 371 Task 22b: collect component-typed fields whose state_snapshot
        // we should recurse into (with a "<field>." prefix) so the rust-mode
        // autoui_state tool can see child/store state. The store field is
        // injected via STORE_NAMES (alias "store"); child components declared
        // as struct fields show up in state_types/prop_types with a type that
        // matches a registered component name.
        let mut recurse_fields: Vec<String> = Vec::new();
        // Store composable field (always named "store" per generate_struct).
        let is_store_itself = STORE_NAMES.with(|sn| {
            sn.borrow().values().any(|s| s.as_str() == _widget.name)
        });
        if !is_store_itself {
            STORE_NAMES.with(|sn| {
                if !sn.borrow().is_empty() && !scalars.iter().any(|(n, _)| n == "store") {
                    recurse_fields.push("store".to_string());
                }
            });
        }
        // Child components stored as struct fields (type matches a known
        // component — registered store or a child_components entry).
        let known_components: std::collections::HashSet<String> = {
            let mut s: std::collections::HashSet<String> = STORE_NAMES
                .with(|sn| sn.borrow().values().cloned().collect());
            for c in &self.child_components {
                s.insert(c.clone());
            }
            s
        };
        for (name, ty) in self.state_types.iter().chain(self.prop_types.iter()) {
            if known_components.contains(ty) && !recurse_fields.contains(name) && name != "store" {
                recurse_fields.push(name.clone());
            }
        }
        // Plan 371 L3: persistent child component fields (e.g. editor_panel)
        // also need recursion so their state (editing/edit_title) is visible.
        for child_name in &self.child_components {
            if self.is_persistent_child(child_name) {
                let field = Self::child_field_name(child_name);
                if !recurse_fields.contains(&field) {
                    recurse_fields.push(field);
                }
            }
        }

        if scalars.is_empty() && recurse_fields.is_empty() {
            return String::new();
        }

        let mut code = String::new();
        code.push_str("    fn state_snapshot(&self) -> std::collections::HashMap<String, auto_lang::ui::auto_val::Value> {\n");
        code.push_str("        let mut m = std::collections::HashMap::new();\n");
        for (name, ty) in &scalars {
            let expr = scalar_to_auto_value_expr("self", name, ty);
            code.push_str(&format!(
                "        m.insert({:?}.to_string(), {});\n",
                name, expr
            ));
        }
        // Recurse into component-typed fields, prefixing keys with "<field>.".
        for field in &recurse_fields {
            code.push_str(&format!(
                "        for (k, v) in self.{}.state_snapshot() {{ m.insert(format!(\"{{}}.{{}}\", {:?}, k), v); }}\n",
                field, field
            ));
        }
        code.push_str("        m\n");
        code.push_str("    }\n");
        code
    }

    /// Generate on() method implementation
    fn generate_on_method(&self, widget: &AuraWidget) -> String {
        let mut code = String::new();
        let msg_name = self.current_msg_name();

        code.push_str("    fn on(&mut self, msg: Self::Msg) {\n");

        if !self.message_variants.is_empty() {
            code.push_str("        match msg {\n");

            // Generate match arms from handlers
            for (pattern, payload) in &widget.handlers {
                let variant_name = self.extract_variant_name(pattern);
                let body = self.generate_handler_body(payload);
                // Check if variant has payload — if so, bind it to a variable
                let variant_info = self.message_variants.iter()
                    .find(|v| v.name == variant_name);
                // Plan 374: Skip handlers whose variant is not in the message enum.
                // This handles cases like NewNoteInFolder where the handler exists
                // but the Msg enum doesn't declare the variant.
                if variant_info.is_none() {
                    continue;
                }
                let has_payload = variant_info.map_or(false, |v| !v.payload.is_empty());
                // Tick handler: guard with running check if "running" field exists
                let is_tick_guarded = variant_name == "Tick" && self.state_types.contains_key("running");
                if has_payload {
                    // Plan 346: use the source parameter name (e.g., `i` from
                    // `.SelectNote(i)`) as the match binding, not a hardcoded `id`.
                    let payload_name = self.extract_payload_name(pattern);
                    code.push_str(&format!("            {}::{}({}) => {{\n", msg_name, variant_name, payload_name));
                } else {
                    code.push_str(&format!("            {}::{} => {{\n", msg_name, variant_name));
                }
                if is_tick_guarded {
                    code.push_str("                if self.running == \"true\" {\n");
                }

                // If this event is from an input, prepend input text parsing
                if let Some(field_names) = self.input_fields.get(&variant_name) {
                    // Plan 413: code_editor events read the text via the keyed
                    // accessor instead of the single-slot thread-local.
                    let text_source = if let Some(keys) = self.code_editor_sources.get(&variant_name) {
                        format!(
                            "auto_lang::ui::code_editor::code_editor_text(\"{}\").unwrap_or_default()",
                            keys.first().map(|s| s.as_str()).unwrap_or("editor")
                        )
                    } else {
                        "auto_lang::ui::iced::last_input_text()".to_string()
                    };
                    code.push_str(&format!(
                        "                let _text = {};
",
                        text_source
                    ));
                    // Set ALL bound fields to the input text (multiple inputs may share one event)
                    let last_idx = field_names.len() - 1;
                    for (i, field_name) in field_names.iter().enumerate() {
                        let rust_type = self.state_types.get(field_name).map(|s| s.as_str()).unwrap_or("f64");
                        if rust_type == "String" {
                            // Last field can consume _text directly; others must clone
                            let text_expr = if i == last_idx { "_text".to_string() } else { "_text.clone()".to_string() };
                            code.push_str(&format!(
                                "                self.{} = {};\n",
                                field_name, text_expr
                            ));
                        } else {
                            let parse_method = match rust_type {
                                "i32" => "parse::<i32>()",
                                "i64" => "parse::<i64>()",
                                "u32" => "parse::<u32>()",
                                "u64" => "parse::<u64>()",
                                "f32" => "parse::<f32>()",
                                "f64" => "parse::<f64>()",
                                "bool" => "parse::<bool>()",
                                _ => "parse::<f64>()",
                            };
                            code.push_str(&format!(
                                "                self.{} = _text.{}.unwrap_or(self.{});\n",
                                field_name, parse_method, field_name
                            ));
                        }
                    }

                    // Skip redundant self-assignment body (e.g. `.email = .email`)
                    // or body that assigns the bound field from the msg payload
                    // (e.g. `.edit_title = t` / `.edit_title = t.to_string()`) —
                    // we already set it from last_input_text() above, so the
                    // payload binding would clobber it with the static empty arg.
                    let payload_name = if has_payload {
                        self.extract_payload_name(pattern)
                    } else {
                        String::new()
                    };
                    let body_redundant = field_names.iter().all(|f| {
                        // T-03 后 handler 块尾恒补 `;`,先剥掉再比对。
                        let b = body.trim().trim_end_matches(';');
                        b == format!("self.{} = self.{}", f, f)
                            || b == format!("self.{} = {}", f, payload_name)
                            || b == format!("self.{} = {}.to_string()", f, payload_name)
                            || b == format!("self.{} = {}.clone()", f, payload_name)
                    });
                    if !body_redundant && !body.trim().is_empty() {
                        code.push_str(&format!("                {}\n", body));
                    }
                } else {
                    code.push_str(&format!("                {}\n", body));
                }

                // Post-process Tick handler: if model has elapsed + time_display + ms_display,
                // append display computation after the user's tick body.
                if variant_name == "Tick"
                    && self.state_types.contains_key("elapsed")
                    && self.state_types.contains_key("time_display")
                    && self.state_types.contains_key("ms_display")
                {
                    // Ensure prior statement ends with semicolon
                    let trimmed = code.trim_end();
                    if !trimmed.ends_with(';') && !trimmed.ends_with('}') {
                        code.push_str(";\n");
                    }
                    code.push_str(
                        "                    let total_cs = self.elapsed / 10;\n\
                         \x20                   let cs = total_cs % 100;\n\
                         \x20                   let total_secs = total_cs / 100;\n\
                         \x20                   let secs = total_secs % 60;\n\
                         \x20                   let mins = total_secs / 60;\n\
                         \x20                   self.time_display = format!(\"{:02}:{:02}\", mins, secs);\n\
                         \x20                   self.ms_display = format!(\".{:02}\", cs);\n"
                    );
                }

                // Close the running guard for Tick handler
                if is_tick_guarded {
                    code.push_str("                }\n");
                }

                // Plan 371 L1: If this handler changes the store data that feeds
                // a child component's props (e.g. NewNote changes active_id →
                // EditorPanel's `note` prop changes), the child needs its Init
                // lifecycle re-triggered so it resets its editing state for the
                // new empty note. VM mode does this implicitly via the unified
                // state heap; rust mode has no such lifecycle, so simulate it.
                // General criteria (replaces the old hardcoded "NewNote" +
                // name-contains-"Editor" check):
                //   1. The handler body mutates store data (calls a mutating
                //      store method or assigns to store.active_id / store.notes).
                //   2. There exists a child component whose props are written by
                //      its own handlers (component_semantics written_props), i.e.
                //      a child that owns editable state tied to those props.
                if self.handler_mutates_store_data(payload)
                    && !self.child_components.is_empty()
                {
                    // Find the first child whose handlers write props (i.e. it has
                    // editable state to reset on data change). This replaces the
                    // old `.contains("Editor")` name match.
                    let target = self.child_components.iter()
                        .find(|c| {
                            self.component_semantics.get(*c)
                                .map(|s| !s.written_props.is_empty())
                                .unwrap_or(false)
                        })
                        .cloned();
                    if let Some(child_name) = target {
                        let child_msg = format!("{}Msg", child_name);
                        let persistent = self.is_persistent_child(&child_name);
                        let field = Self::child_field_name(&child_name);
                        // Ensure the preceding body statement ends with ';'.
                        code.push_str("                ;\n");
                        if persistent {
                            // L3: use persistent instance — update props then Init.
                            let constructor_args = self.find_constructor_args_for_child(widget, &child_name);
                            // Sync props from constructor args, then call on(Init).
                            code.push_str(&format!(
                                "                self.{} = {}::new({});\n",
                                field, child_name, constructor_args
                            ));
                            code.push_str(&format!(
                                "                self.{}.on({}::Init);\n",
                                field, child_msg
                            ));
                        } else {
                            // Loop child: temp-construct, Init, sync back.
                            let sync: Vec<String> = self.component_state_fields.get(&child_name)
                                .map(|fs| fs.iter().map(|(f,_)| f.clone()).collect())
                                .unwrap_or_default();
                            let constructor_args = self.find_constructor_args_for_child(widget, &child_name);
                            code.push_str(&format!(
                                "                {{ let mut __ep = {}::new({});\n",
                                child_name, constructor_args
                            ));
                            code.push_str(&format!(
                                "                __ep.on({}::Init);\n",
                                child_msg
                            ));
                            for f in &sync {
                                code.push_str(&format!(
                                    "                self.{} = __ep.{}.clone();\n", f, f
                                ));
                            }
                            code.push_str("                }\n");
                        }
                    }
                }

                code.push_str("            }\n");
            }

            // Generate match arms from lifecycle handlers (.Init, .Destroy)
            for lc in &widget.lifecycle {
                let body = self.generate_handler_body(&lc.payload);
                code.push_str(&format!("            {}::{} => {{\n", msg_name, lc.name));
                if lc.name == "Init" && self.init_api_info.is_some() {
                    // Async Init: body is handled by __InitLoaded message from boot task
                    code.push_str("                // async init — data arrives via __InitLoaded\n");
                } else {
                    code.push_str(&format!("                {}\n", body));
                }
                code.push_str("            }\n");
            }

            // If async Init detected, generate __InitLoaded handler
            if let Some(ref info) = self.init_api_info {
                code.push_str(&format!(
                    "            {}::__InitLoaded(__data) => {{\n                self.{} = __data\n            }}\n",
                    msg_name, info.state_var
                ));
            }

            // Add handler forwarding for child component message wrappers.
            for child_name in &self.child_components {
                let persistent = self.is_persistent_child(child_name);
                let field_name = Self::child_field_name(child_name);
                let sync_fields = self.find_sync_fields_for_child(widget, child_name);
                let constructor_args = self.find_constructor_args_for_child(widget, child_name);

                code.push_str(&format!(
                    "            {}::{}(inner) => {{\n",
                    msg_name, child_name
                ));

                if persistent {
                    // Plan 371 L3: persistent child — operate directly on the field.
                    // The child's on() may mutate its cloned store (e.g. NavTree's
                    // SelectPinned sets store.active_folder). Sync store back so the
                    // parent sees the change. Private state (editing etc.) persists
                    // in the field and needs no sync.
                    code.push_str(&format!(
                        "                self.{}.on(inner);\n",
                        field_name
                    ));
                    // Sync store back if the child has one.
                    let has_store = STORE_NAMES.with(|sn| !sn.borrow().is_empty());
                    if has_store {
                        code.push_str(&format!(
                            "                self.store = self.{}.store.clone();\n",
                            field_name
                        ));
                    }
                } else {
                    // Loop child (or legacy): temp-construct, sync in/out, then drop.
                    // PLAN-039 T-14：循环子组件的转发构造用 Default——
                    // view 树提取的构造实参含循环变量（slot.suit），on()
                    // 转发闭包不在该作用域（E0425 slot 株）；view 层每
                    // 次渲染全新实例，转发到 Default 实例语义等价。
                    let _ = &constructor_args;
                    code.push_str(&format!(
                        "                let mut __child = {}::default();\n",
                        child_name
                    ));
                    for field in &sync_fields {
                        code.push_str(&format!(
                            "                __child.{} = self.{}.clone();\n",
                            field, field
                        ));
                    }
                    code.push_str("                __child.on(inner);\n");
                    for field in &sync_fields {
                        code.push_str(&format!(
                            "                self.{} = __child.{};\n",
                            field, field
                        ));
                    }
                }

                // Plan 371 L1: General prop-writeback. For each prop the child's
                // handlers WRITE, write the (possibly mutated) child prop back to the
                // parent's data source. Persistent children use self.<field>.<prop>;
                // temp children use __child.<prop>.
                let written_props: Vec<String> = self.component_semantics.get(child_name)
                    .map(|s| s.written_props.clone())
                    .unwrap_or_default();
                if !written_props.is_empty() {
                    let has_notes = self.state_types.contains_key("notes");
                    let has_store_notes = STORE_NAMES.with(|sn| !sn.borrow().is_empty())
                        && !STORE_NAMES.with(|sn| sn.borrow().values().any(|s| s.as_str() == widget.name));
                    let notes_prefix = if has_notes { "self.notes" } else if has_store_notes { "self.store.notes" } else { "" };
                    let active_prefix = if self.state_types.contains_key("active_id") { "self.active_id" } else if has_store_notes { "self.store.active_id" } else { "" };
                    let prop_owner = if persistent {
                        format!("self.{}", field_name)
                    } else {
                        "__child".to_string()
                    };
                    if !notes_prefix.is_empty() && !active_prefix.is_empty() {
                        for prop in &written_props {
                            code.push_str(&format!(
                                "                if let Some(__n) = {}.get_mut({} as usize) {{\n                    *__n = {}.{}.clone();\n                }}\n",
                                notes_prefix, active_prefix, prop_owner, prop
                            ));
                        }
                    }
                }

                code.push_str("            }\n");
            }

            // Wildcard arm must come AFTER all named arms (including child forwarding).
            // The enum has message_variants.len() + child_components.len() total variants.
            // We generate arms for: handlers + lifecycle + child_components + __InitLoaded (if async).
            // If there are more enum variants than named arms, we need a wildcard.
            let async_init_arm = if self.init_api_info.is_some() { 1 } else { 0 };
            let total_enum_variants = self.message_variants.len() + self.child_components.len() + async_init_arm;
            let named_arms = widget.handlers.len() + widget.lifecycle.len() + self.child_components.len() + async_init_arm;
            if total_enum_variants > named_arms {
                code.push_str("            _ => {}\n");
            }

            code.push_str("        }\n");
        }

        code.push_str("    }\n");

        code
    }

    /// Generate view() method implementation
    fn generate_view_method(&mut self, widget: &AuraWidget) -> String {
        let mut code = String::new();

        code.push_str("    fn view(&self) -> View<Self::Msg> {\n");

        // Generate view tree
        let mut view_code = self.generate_view_tree(&widget.view_tree);

        // Plan 374: Post-process view code for known patterns.
        // Fix 1: Replace bare `active` in conditions with `i == self.store.active_id`
        // (from view fragment NoteItem parameter substitution that didn't complete).
        view_code = view_code.replace("if active {", "if i == self.store.active_id {");
        // Fix 2: Value["field"].iter() → Value["field"].as_array().into_iter().flatten()
        view_code = view_code.replace("[\"tags\"].iter()", "[\"tags\"].as_array().unwrap_or(&Vec::new()).iter()");
        // Fix 3: String == &Value → String == X.as_str().unwrap_or_default()
        view_code = view_code.replace("self.store.active_tag == t {", "self.store.active_tag == t.as_str().unwrap_or_default() {");
        view_code = view_code.replace("*self.store.active_tag == *t {", "self.store.active_tag == t.as_str().unwrap_or_default() {");
        // Fix 4: RemoveTag(t) where t is &Value → RemoveTag(t.to_string())
        view_code = view_code.replace("EditorPanelMsg::RemoveTag(t)", "EditorPanelMsg::RemoveTag(t.to_string())");
        // Fix 5: SelectTag(t) where t is &Value → SelectTag(t.to_string())
        view_code = view_code.replace("NavTreeMsg::SelectTag(t)", "NavTreeMsg::SelectTag(t.to_string())");

        code.push_str(&format!("        {}\n", view_code));

        code.push_str("    }\n");

        code
    }

    /// Generate computed properties impl block
    fn generate_computed_impl(&self, widget: &AuraWidget) -> String {
        let widget_name = &widget.name;
        let mut code = String::new();

        code.push_str(&format!("impl {} {{\n", widget_name));

        for computed_prop in &widget.computed {
            let method_name = &computed_prop.name;
            // Register store computed property names globally for cross-widget access
            STORE_COMPUTED_NAMES.with(|sn| {
                sn.borrow_mut().insert(method_name.clone());
            });
            let mut expr_rust = self.ast_expr_to_rust(&computed_prop.expr);

            // Plan 374: Vec<Value> doesn't have .filter()/.map() directly —
            // insert .iter() before them so the iterator chain type-checks.
            // e.g., self.notes.filter(...) → self.notes.iter().filter(...)
            for method in &[".filter(", ".map("] {
                if expr_rust.contains(method) && !expr_rust.contains(".iter()") {
                    // Find state vars that are Vec<...> and insert .iter() after them
                    for state in &widget.state_vars {
                        let field_ref = format!("self.{}", state.name);
                        let field_ref_with_iter = format!("self.{}.iter()", state.name);
                        let target = format!("{}{}", field_ref, method);
                        let replacement = format!("{}{}", field_ref_with_iter, method);
                        if expr_rust.contains(&target) && !expr_rust.contains(&field_ref_with_iter) {
                            expr_rust = expr_rust.replace(&target, &replacement);
                        }
                    }
                }
            }

            // Generate getter method.
            // a2r fix: infer return type from the computed expression.
            // - If expr has .iter()/.filter()/.map() → Vec<serde_json::Value> + collect
            // - If expr is a string literal or str field access → String
            // - If expr is a self.field that's typed String → String
            // - Otherwise default to String (safer than Vec for scalar computed).
            let needs_collect = expr_rust.contains(".iter().") || expr_rust.contains(".filter(") || expr_rust.contains(".map(");
            let is_string_expr = !needs_collect && (
                expr_rust.contains("\"")  // string literal
                || expr_rust.contains(".to_string()")
                || expr_rust.contains("+ \"")  // string concatenation
                || self.state_types.iter().any(|(k, v)|
                    v == "String" && expr_rust.contains(&format!("self.{}", k)))
            );
            let (return_type, final_expr) = if needs_collect {
                let rt = "Vec<serde_json::Value>";
                let fe = if !expr_rust.contains(".collect(") {
                    if expr_rust.contains(".iter().") {
                        format!("{}.cloned().collect::<Vec<_>>()", expr_rust)
                    } else {
                        format!("{}.collect::<Vec<_>>()", expr_rust)
                    }
                } else {
                    expr_rust.clone()
                };
                (rt, fe)
            } else if is_string_expr {
                ("String", expr_rust.clone())
            } else {
                // Default: String for scalar computed (int/bool/str).
                ("String", expr_rust.clone())
            };
            code.push_str(&format!("    pub fn {}(&self) -> {} {{\n", method_name, return_type));
            code.push_str(&format!("        {}\n", final_expr));
            code.push_str("    }\n\n");
        }

        code.push_str("}\n");

        code
    }

    /// Check if a tag is a leaf element that has no children (text, button, etc.)
    fn is_leaf_tag(&self, tag: &str) -> bool {
        matches!(tag, "text" | "label" | "span" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p" | "button")
    }

    /// Pre-scan handler bodies to find local `let` bindings from function calls.
    /// These locals likely hold `serde_json::Value` results and need index access
    /// for field reads (e.g., `note.id` → `note["id"]`).
    fn scan_handler_locals(&mut self, widget: &AuraWidget) {
        for (_pattern, payload) in &widget.handlers {
            self.scan_payload_locals(payload);
        }
        for lc in &widget.lifecycle {
            self.scan_payload_locals(&lc.payload);
        }
    }

    /// Scan a single LogicPayload for local variables that hold serde_json::Value.
    /// Detects:
    /// - `let x = func_call()` — function call results
    /// - `let x = collection[idx]` — indexing into Vec<Value>
    /// - `let x = todos[idx]` — same pattern with named collection
    /// - `for x in collection` — loop variables iterating over Vec<Value>
    fn scan_payload_locals(&mut self, payload: &LogicPayload) {
        match payload {
            LogicPayload::AstStmts(stmts) => {
                self.scan_ast_stmts_for_value_locals(stmts);
            }
            _ => {}
        }
    }

    /// PLAN-039 T-14（E-D3 第二轨）：递归收集无类型局部的赋值形态集
    /// （Store 初始式 + Bina Asn 局部赋值；For/If/Block 递归体）。
    fn collect_assign_kinds(
        &self,
        stmts: &[crate::ast::Stmt],
        out: &mut std::collections::HashMap<String, std::collections::HashSet<&'static str>>,
        declared: &std::collections::HashSet<String>,
    ) {
        use crate::ast::{Expr, Stmt};
        for stmt in stmts {
            match stmt {
                Stmt::Store(store)
                    if matches!(
                        store.kind,
                        crate::ast::StoreKind::Let | crate::ast::StoreKind::Const | crate::ast::StoreKind::Var
                    ) && matches!(store.ty, crate::ast::Type::Unknown) =>
                {
                    let name = store.name.as_str().trim_start_matches('.');
                    out.entry(name.to_string())
                        .or_default()
                        .insert(self.assign_kind(&store.expr));
                }
                Stmt::Expr(Expr::Bina(l, op, r)) if matches!(op, auto_val::Op::Asn) => {
                    if let Expr::Ident(n) = l.as_ref() {
                        let name = n.as_str().trim_start_matches('.');
                        if !name.starts_with('.')
                            && !self.state_types.contains_key(name)
                            // PLAN-039 T-14（E-D3）：显式声明局部不联合
                            // （`var c_card int` 后续 Asn 不收 Value 格——
                            // 赋值点强转路径已接）。
                            && !declared.contains(name)
                        {
                            out.entry(name.to_string())
                                .or_default()
                                .insert(self.assign_kind(r));
                        }
                    }
                }
                Stmt::For(f) => self.collect_assign_kinds(&f.body.stmts, out, declared),
                Stmt::If(i) => {
                    for b in &i.branches {
                        self.collect_assign_kinds(&b.body.stmts, out, declared);
                    }
                    if let Some(e) = &i.else_ {
                        self.collect_assign_kinds(&e.stmts, out, declared);
                    }
                }
                Stmt::Block(b) => self.collect_assign_kinds(&b.stmts, out, declared),
                _ => {}
            }
        }
    }

    /// PLAN-039 T-14（E-D3）：递归收集显式声明局部名（联合格的排除面）。
    fn collect_declared_names(
        &self,
        stmts: &[crate::ast::Stmt],
        out: &mut std::collections::HashSet<String>,
    ) {
        use crate::ast::Stmt;
        for stmt in stmts {
            match stmt {
                Stmt::Store(store)
                    if matches!(
                        store.kind,
                        crate::ast::StoreKind::Let | crate::ast::StoreKind::Const | crate::ast::StoreKind::Var
                    ) && !matches!(store.ty, crate::ast::Type::Unknown) =>
                {
                    out.insert(store.name.as_str().trim_start_matches('.').to_string());
                }
                Stmt::For(f) => self.collect_declared_names(&f.body.stmts, out),
                Stmt::If(i) => {
                    for b in &i.branches {
                        self.collect_declared_names(&b.body.stmts, out);
                    }
                    if let Some(e) = &i.else_ {
                        self.collect_declared_names(&e.stmts, out);
                    }
                }
                Stmt::Block(b) => self.collect_declared_names(&b.stmts, out),
                _ => {}
            }
        }
    }

    /// PLAN-039 T-14（E-D3 第二轨）：赋值 RHS 的形态格——value/int/str/
    /// bool/arr/other 六态（Index on Value 集合、非标量 Call、记录字面量
    /// → value；数值/串/布字面量 → 标量；名字形态查 state/局部格）。
    fn assign_kind(&self, expr: &crate::ast::Expr) -> &'static str {
        use crate::ast::Expr;
        match expr {
            Expr::Int(_) | Expr::I64(_) | Expr::Float(_, _) | Expr::Double(_, _) => "int",
            Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_) => "str",
            Expr::Bool(_) => "bool",
            Expr::Object(_) => "value",
            Expr::Array(_) => "arr",
            Expr::Index(target, _) => {
                if let Some(coll) = self.resolve_expr_name(target) {
                    let vec_value = self
                        .state_types
                        .get(&coll)
                        .map_or(false, |t| t == "Vec<serde_json::Value>")
                        || self.array_locals.contains(&coll);
                    if vec_value {
                        return "value";
                    }
                }
                if let crate::ast::Expr::Ident(n) = target.as_ref() {
                    let s = n.as_str();
                    if s.starts_with(".store.") {
                        let f = &s[".store.".len()..];
                        if !f.contains('.')
                            && self
                                .store_field_rust_type(f)
                                .map_or(false, |t| t == "Vec<serde_json::Value>")
                        {
                            return "value";
                        }
                    }
                }
                "other"
            }
            Expr::Call(call) => {
                // api typed 返回 = other（typed）；内建标量族 = int/str。
                if let Some(n) = call.get_name_text_safe() {
                    let n = n.as_str();
                    if API_TYPED_FNS.with(|m| m.borrow().contains(n)) {
                        return "other";
                    }
                    if n == "names" {
                        return "other";
                    }
                }
                if let crate::ast::Expr::Dot(_, m) = call.name.as_ref() {
                    match m.as_str() {
                        "len" | "str" => return "int",
                        "slice" | "lower" | "upper" | "trim" | "replace"
                        | "to_string" | "to_lowercase" | "to_uppercase" => return "str",
                        _ => {}
                    }
                }
                "value"
            }
            Expr::Ident(name) => {
                let s = name.as_str();
                let resolved = s.strip_prefix('.').unwrap_or(s);
                if let Some(ty) = self.state_types.get(resolved) {
                    return match ty.as_str() {
                        "i32" | "i64" | "u32" | "u64" | "f32" | "f64" => "int",
                        "bool" => "bool",
                        "String" => "str",
                        "serde_json::Value" => "value",
                        _ => "other",
                    };
                }
                if self.array_locals.contains(resolved) {
                    return "arr";
                }
                "other"
            }
            _ => "other",
        }
    }

    /// Recursively scan AST statements for value-typed locals and loop vars
    fn scan_ast_stmts_for_value_locals(&mut self, stmts: &[crate::ast::Stmt]) {
        // PLAN-039 T-14（批次 E，E-D3 第二轨）：无类型局部两遍收格——
        // 第一遍收集赋值 RHS 形态集（初始式 + 后续 Asn），联合含 Value
        // 用例（`var card = 999` + `card = waste_cards[i]`）→ Value 格
        // （发射侧 let/Asn 标量 json! 化配套）。
        // **只在顶层跑一次**（递归体单独 scan 时外层声明不可见——
        // `var c int` 顶层 + `c = items[0]` 在 if 体：子集 declared 空
        // 致误收，probe 实证株）；递归走 scan_stmts_no_union。
        let mut assign_kinds: std::collections::HashMap<String, std::collections::HashSet<&'static str>> =
            std::collections::HashMap::new();
        let mut declared = std::collections::HashSet::new();
        self.collect_declared_names(stmts, &mut declared);
        self.collect_assign_kinds(stmts, &mut assign_kinds, &declared);
        for (name, kinds) in &assign_kinds {
            if kinds.contains(&"value") {
                self.value_locals.insert(name.clone());
            }
        }
        self.scan_stmts_no_union(stmts);
    }

    /// 单遍收格（scan 的原逻辑体）——联合段由顶层统一先行。
    fn scan_stmts_no_union(&mut self, stmts: &[crate::ast::Stmt]) {
        for stmt in stmts {
            match stmt {
                crate::ast::Stmt::Store(store) => {
                    if matches!(store.kind, crate::ast::StoreKind::Let | crate::ast::StoreKind::Const | crate::ast::StoreKind::Var) {
                        let name = store.name.as_str();
                        // PLAN-039 T-12（批次 E，E-D3 第一轨）：显式声明型
                        // （`var cx int = …`）保持声明型——不收 Value/集合
                        // 格（minesweeper `var cx int = stack.pop()` 回归株：
                        // 误收致数值使用点错包 __at_num(&i32)）。只有无类型
                        // （Type::Unknown）局部按初始式形态收格。
                        let untyped = matches!(store.ty, crate::ast::Type::Unknown);
                        // PLAN-039 T-14（E-D3 第一轨）：显式声明局部型记录。
                        if !untyped {
                            let kind = match &store.ty {
                                crate::ast::Type::Int | crate::ast::Type::I64
                                | crate::ast::Type::Uint | crate::ast::Type::U64 => "int",
                                crate::ast::Type::Bool => "bool",
                                crate::ast::Type::StrFixed(_)
                                | crate::ast::Type::StrOwned
                                | crate::ast::Type::StrSlice => "str",
                                _ => "",
                            };
                            if !kind.is_empty() {
                                self.declared_locals
                                    .insert(name.trim_start_matches('.').to_string(), kind.to_string());
                            }
                        }
                        // Check if the value is a function call (likely returns Value).
                        // Plan 547: `names()` is a native `Vec<String>` API result,
                        // so it must retain ordinary Rust field/index syntax.
                        if untyped {
                            if let crate::ast::Expr::Call(call) = &store.expr {
                                let call_name = call.get_name_text_safe()
                                    .map(|n| n.as_str().to_string());
                                // PLAN-039 T-13（E-D5-A 配套）：返回类型化
                                // user 型的 api 桩调用不收 Value 格——
                                // `let r = board_cards(..)` 的 r.cards 直达
                                // （typed 局部形态，Object/Array 分支天然
                                // 不命中 Call 形态）。
                                let api_typed = API_TYPED_FNS.with(|m| {
                                    call_name.as_deref().map_or(false, |n| m.borrow().contains(n))
                                });
                                if !api_typed && call_name.as_deref() != Some("names") {
                                    self.value_locals.insert(name.to_string());
                                }
                                // PLAN-039 T-12（批次 E）：内建表方法调用赋值
                                // 的局部不收 Value 格——len/str/slice/lower 族
                                // 返回标量/串（`var napps int = .xs.len()` 株：
                                // 误收致数值使用点错包 __at_num(&i32)）。内建
                                // 之外的调用（storage.get 等）维持 Value 收格。
                                if let crate::ast::Expr::Dot(_, m) = call.name.as_ref() {
                                    if matches!(
                                        m.as_str(),
                                        "len" | "str" | "slice" | "lower" | "upper" | "trim"
                                            | "replace" | "contains" | "starts_with" | "ends_with"
                                    ) {
                                        self.value_locals.remove(name);
                                    }
                                }
                            }
                        }
                        // PLAN-039 T-12（批次 E，E-D2）：记录字面量局部
                        // （`var row = { name: … }`）→ Value 格；无类型集合
                        // 局部（`var scored = []`）→ array_locals（Vec<Value>
                        // 格——原生 push/pop/len，元素字段直访降链）。
                        // launcher ranked 行构造/收集株；此前两者不入格，
                        // `row.category`、`scored[i].score` 编译断。
                        // （显式声明型不收——E-D3 第一轨门。）
                        if untyped {
                            if let crate::ast::Expr::Object(_) = &store.expr {
                                self.value_locals.insert(name.to_string());
                                // E-D2：局部记录形状（row.score 访问器选型）。
                                if let Some(shape) = self.object_literal_shape(&store.expr) {
                                    self.local_record_shapes
                                        .insert(name.to_string(), shape);
                                }
                            }
                            if let crate::ast::Expr::Array(elems) = &store.expr {
                                self.array_locals.insert(name.to_string());
                                // E-D2：字面量数组的元素形状（记录元素首位推）。
                                if let Some(first) = elems.first() {
                                    if let Some(shape) = self.object_literal_shape(first) {
                                        self.array_element_shapes
                                            .insert(name.to_string(), shape);
                                    }
                                }
                            }
                        }
                        // Check if the value is an index into a state Vec<Value>
                        if untyped {
                            if let crate::ast::Expr::Index(target, _idx) = &store.expr {
                            // Plan 407 R4a: resolve collection name from Ident or Dot patterns.
                            let coll_stripped: Option<&str> = match target.as_ref() {
                                crate::ast::Expr::Ident(collection) => {
                                    let s = collection.as_str();
                                    Some(s.strip_prefix('.').unwrap_or(s))
                                }
                                crate::ast::Expr::Dot(inner, field) => {
                                    if matches!(inner.as_ref(), crate::ast::Expr::Ident(_)) {
                                        // PLAN-039 T-13：任意 Ident 基座
                                        // （r.cards 局部集合 → "cards"，
                                        // state/local 判定随后收口）。
                                        Some(field.as_str())
                                    } else { None }
                                }
                                _ => None,
                            };
                            if let Some(coll) = coll_stripped {
                                if self.state_types.get(coll)
                                    // PLAN-039 T-13（E-D5-A）：收窄为
                                    // Vec<Value> 专属（typed Vec 元素是 typed）。
                                    .map(|ty| ty == "Vec<serde_json::Value>")
                                    .unwrap_or(false)
                                {
                                    self.value_locals.insert(name.to_string());
                                }
                            }
                            }
                        }
                    }
                }
                crate::ast::Stmt::For(for_stmt) => {
                    // Register loop variable as value var if iterating over a Value collection
                    // PLAN-039 T-13（E-D5-A）：收窄为 Vec<serde_json::Value>
                    // 专属——typed Vec（Vec<Card>，back 型发射后）的循环变量
                    // 是 typed，`c.column` 直达字段（此前 starts_with("Vec<")
                    // 误收致 typed 元素被索引访问 ×42）。
                    match &for_stmt.iter {
                        crate::ast::Iter::Named(name) => {
                            // `for todo in .todos` — check if .todos is Vec<Value>
                            if let crate::ast::Expr::Dot(obj, field) = &for_stmt.range {
                                if let crate::ast::Expr::Ident(_) = obj.as_ref() {
                                    if self.state_types.get(field.as_str())
                                        .map(|ty| ty == "Vec<serde_json::Value>")
                                        .unwrap_or(false)
                                    {
                                        self.value_loop_vars.insert(name.as_str().to_string());
                                        // PLAN-039 T-14（E-D2）：循环变量 →
                                        // 集合映射（元素形状查询面）。
                                        self.loop_var_collections.insert(
                                            name.as_str().to_string(),
                                            field.as_str().to_string(),
                                        );
                                    }
                                }
                            } else if let crate::ast::Expr::Ident(name_expr) = &for_stmt.range {
                                let coll = name_expr.as_str();
                                if self.state_types.get(coll)
                                    .map(|ty| ty == "Vec<serde_json::Value>")
                                    .unwrap_or(false)
                                {
                                    self.value_loop_vars.insert(name.as_str().to_string());
                                    self.loop_var_collections.insert(
                                        name.as_str().to_string(),
                                        coll.to_string(),
                                    );
                                }
                            }
                        }
                        crate::ast::Iter::Indexed(_idx, name) => {
                            if let crate::ast::Expr::Dot(obj, field) = &for_stmt.range {
                                if let crate::ast::Expr::Ident(_) = obj.as_ref() {
                                    if self.state_types.get(field.as_str())
                                        .map(|ty| ty == "Vec<serde_json::Value>")
                                        .unwrap_or(false)
                                    {
                                        self.value_loop_vars.insert(name.as_str().to_string());
                                    }
                                }
                            } else if let crate::ast::Expr::Ident(name_expr) = &for_stmt.range {
                                let coll = name_expr.as_str();
                                if self.state_types.get(coll)
                                    .map(|ty| ty == "Vec<serde_json::Value>")
                                    .unwrap_or(false)
                                {
                                    self.value_loop_vars.insert(name.as_str().to_string());
                                }
                            }
                        }
                        _ => {}
                    }
                    // Recurse into for loop body
                    self.scan_stmts_no_union(&for_stmt.body.stmts);
                }
                crate::ast::Stmt::If(if_stmt) => {
                    for branch in &if_stmt.branches {
                        self.scan_stmts_no_union(&branch.body.stmts);
                    }
                    if let Some(else_body) = &if_stmt.else_ {
                        self.scan_stmts_no_union(&else_body.stmts);
                    }
                }
                // PLAN-039 T-13（批次 E，E-D2）：`arr.push(x)` 数组局部元素
                // 形状登记——实参是记录字面量或已知形状局部时，数组的
                // 元素字段访问（scored[i].score）按形状选访问器
                // （launcher ranked 行收集株：score 启发式名单外恒 as_str）。
                crate::ast::Stmt::Expr(expr) => {
                    if let crate::ast::Expr::Call(call) = expr {
                        if let crate::ast::Expr::Dot(obj, m) = call.name.as_ref() {
                            if m.as_str() == "push" {
                                if let crate::ast::Expr::Ident(n) = obj.as_ref() {
                                    let arr = n.as_str().trim_start_matches('.').to_string();
                                    if let Some(arg0) = call.args.args.first() {
                                        let e = arg0.get_expr();
                                        let shape = self.object_literal_shape(&e).or_else(|| {
                                            if let crate::ast::Expr::Ident(an) = e {
                                                let an = an.as_str().trim_start_matches('.');
                                                self.local_record_shapes.get(an).cloned()
                                            } else {
                                                None
                                            }
                                        });
                                        if let Some(shape) = shape {
                                            self.array_element_shapes
                                                .entry(arr)
                                                .or_insert(shape);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Detect if the Init handler body is a single `self.X = api_func()` assignment
    /// where api_func matches one of the API imports. If so, store in init_api_info
    /// so we can generate an async init pattern (boot task + __InitLoaded message).
    ///
    /// In the AST, `.notes = list_notes()` is parsed as:
    ///   Stmt::Expr(Expr::Bina(Expr::Dot(Ident("self"), "notes"), Op::Asn, Expr::Call("list_notes")))
    fn detect_init_api_call(&mut self, payload: &LogicPayload, api_imports: &[String]) {
        if api_imports.is_empty() {
            return;
        }
        if let LogicPayload::AstStmts(stmts) = payload {
            if stmts.len() != 1 {
                return;
            }
            if let crate::ast::Stmt::Expr(expr) = &stmts[0] {
                // Pattern: Bina(Dot(self, field), Asn, Call(func))
                if let crate::ast::Expr::Bina(left, op, right) = expr {
                    use auto_val::Op;
                    if !matches!(op, Op::Asn) {
                        return;
                    }
                    // Left side: Expr::Dot(Ident("self"), Name("field"))
                    let state_var = extract_dot_self_field(left);
                    // Right side: Expr::Call(...). PLAN-627 (R627-F2): a
                    // qualified head `api.X(...)` unwraps to X when X is in
                    // the import list, so async-Init takes the same
                    // __InitLoaded shape (byte-for-byte) as the bare name.
                    let fn_name = extract_call_name(right).or_else(|| match right.as_ref() {
                        crate::ast::Expr::Call(call) => match call.name.as_ref() {
                            crate::ast::Expr::Dot(obj, method) => {
                                if matches!(obj.as_ref(), crate::ast::Expr::Ident(n) if n.as_str() == "api")
                                    && api_imports.iter().any(|f| f == method.as_str())
                                {
                                    Some(method.as_str().to_string())
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        },
                        _ => None,
                    });
                    if let (Some(var), Some(func)) = (state_var, fn_name) {
                        if api_imports.iter().any(|api| api == &func) {
                            self.init_api_info = Some(InitApiInfo {
                                state_var: var,
                            });
                        }
                    }
                }
            }
        }
    }

    /// Pre-scan view tree to find input/textarea elements and record event→field mappings
    fn scan_input_fields(&mut self, node: &AuraNode) {
        match node {
            AuraNode::Element { tag, props, events, children, .. } => {
                // Plan 413: code_editor registers field + storage key.
                if tag == "code_editor" {
                    let key = props.get("key")
                        .or_else(|| props.get("id"))
                        .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v { Some(s.to_string()) } else { None })
                        .unwrap_or_else(|| "editor".to_string());
                    let value_field: Option<String> = match props.get("content").or_else(|| props.get("value")) {
                        Some(AuraPropValue::Expr(crate::ast::Expr::Ident(name))) => Some(name.to_string()),
                        Some(AuraPropValue::Expr(crate::ast::Expr::Dot(obj, field))) => {
                            let is_direct_self = match obj.as_ref() {
                                crate::ast::Expr::Ident(name) => name.as_str() == "self" || name.as_str() == ".",
                                _ => false,
                            };
                            if is_direct_self { Some(field.to_string()) } else { None }
                        }
                        _ => None,
                    };
                    if let Some(name) = value_field {
                        for (event, handler) in events {
                            if matches!(event.as_str(), "oninput" | "onInput" | "onchange" | "onChange") {
                                let variant = self.extract_variant_name(&handler.handler);
                                self.input_fields.entry(variant.clone()).or_default().push(name.to_string());
                                self.code_editor_sources.entry(variant).or_default().push(key.clone());
                            }
                        }
                    }
                }
                if tag == "input" || tag == "textarea" {
                    // Resolve the `value` binding to a field name. Source uses
                    // `.field` which parses to Expr::Dot(self, "field"); older
                    // code only matched Expr::Ident, missing `.field` bindings
                    // (Plan 371 T5c: edit_title input had no last_input_text injection).
                    let value_field: Option<String> = match props.get("value") {
                        Some(AuraPropValue::Expr(crate::ast::Expr::Ident(name))) => {
                            // PLAN-039 T-03④：点链拍平形（`.store.edit_x`）
                            // 不注册——写回由 handler 自持；点链名注入会落
                            // f64 缺省写编译断。
                            let f = name.as_str().trim_start_matches('.');
                            (!f.is_empty() && !f.contains('.')).then(|| f.to_string())
                        }
                        // Only match direct self.field (one-level Dot). Multi-level
                        // dots like self.store.edit_text should NOT trigger field
                        // injection — the value comes from the store, not a local
                        // state var, so injecting self.edit_text = ... would fail.
                        Some(AuraPropValue::Expr(crate::ast::Expr::Dot(obj, field))) => {
                            // Check it's a direct self.field, not self.store.field
                            let is_direct_self = match obj.as_ref() {
                                crate::ast::Expr::Ident(name) => name.as_str() == "self" || name.as_str() == ".",
                                _ => false,
                            };
                            if is_direct_self { Some(field.to_string()) } else { None }
                        }
                        _ => None,
                    };
                    if let Some(name) = value_field {
                        for (event, handler) in events {
                            if matches!(event.as_str(), "oninput" | "onInput" | "onchange" | "onChange") {
                                let variant = self.extract_variant_name(&handler.handler);
                                self.input_fields.entry(variant).or_default().push(name.to_string());
                            }
                        }
                    }
                }
                for child in children {
                    self.scan_input_fields(child);
                }
            }
            AuraNode::ForLoop { body, .. } => {
                for child in body {
                    self.scan_input_fields(child);
                }
            }
            AuraNode::Conditional { then_body, else_body, .. } => {
                for child in then_body {
                    self.scan_input_fields(child);
                }
                if let Some(else_nodes) = else_body {
                    for child in else_nodes {
                        self.scan_input_fields(child);
                    }
                }
            }
            _ => {}
        }
    }

    /// Pre-scan the view tree to find custom widget references (e.g., EditorPanel, Sidebar).
    /// These need wrapper message variants in the parent's enum.
    fn scan_child_components(&mut self, node: &AuraNode) {
        self.scan_child_components_inner(node, false);
    }

    /// Recursive scan with in_loop tracking.
    /// Plan 371 L3: children found inside a for-loop are added to
    /// `loop_child_components` so they are NOT promoted to persistent fields.
    fn scan_child_components_inner(&mut self, node: &AuraNode, in_loop: bool) {
        match node {
            AuraNode::Element { tag, children, .. } => {
                if self.is_custom_widget(tag) {
                    if !self.child_components.contains(&tag.to_string()) {
                        self.child_components.push(tag.clone());
                    }
                    if in_loop {
                        self.loop_child_components.insert(tag.clone());
                    }
                }
                for child in children {
                    self.scan_child_components_inner(child, in_loop);
                }
            }
            AuraNode::ForLoop { body, .. } => {
                // Plan 371 L3: mark children inside for-loops — they can't be
                // persistent fields (multiple instances). Scan with in_loop=true.
                for child in body {
                    self.scan_child_components_inner(child, true);
                }
            }
            AuraNode::Conditional { then_body, else_body, .. } => {
                for child in then_body {
                    self.scan_child_components_inner(child, in_loop);
                }
                if let Some(else_nodes) = else_body {
                    for child in else_nodes {
                        self.scan_child_components_inner(child, in_loop);
                    }
                }
            }
            AuraNode::Component { name, .. } => {
                if !self.child_components.contains(name) {
                    self.child_components.push(name.clone());
                }
                if in_loop {
                    self.loop_child_components.insert(name.clone());
                }
            }
            _ => {}
        }
    }

    /// Wrap multiple view expressions into a builder chain.
    /// Single view: returns as-is. Multiple views: View::col().child(...).child(...).build()
    fn wrap_views(views: &[String]) -> String {
        if views.len() == 1 {
            views[0].clone()
        } else {
            // Use col() for multi-statement conditional bodies so siblings
            // stack VERTICALLY (the common case — e.g. a section header row
            // followed by a for-loop list, repeated per category). The parent
            // col/row already controls the outer layout direction; an if-body
            // with several children is almost always a vertical sequence.
            // (Previously row() was used, which laid mixed row+for siblings out
            // side-by-side, producing a diagonal/tilted arrangement.)
            let mut builder = "View::col()".to_string();
            for v in views {
                builder = format!("{}.child({})", builder, v);
            }
            format!("{}.build()", builder)
        }
    }

    /// Collect a composite label expression for a button that has children.
    ///
    /// `View::Button` has no children field, so a button with child views
    /// (e.g. `button "" { col { text .note.title; text .note.time } }`) would
    /// render empty (the `.child()` calls are dropped at build). This folds
    /// the button's TEXT-bearing children into a single `format!` label so the
    /// content is visible.
    ///
    /// Recurses into col/row containers to reach their text children. Non-text
    /// children (buttons, components, images) are skipped. Returns a Rust
    /// expression evaluating to `String`, or `"\"\""` if no text children found.
    fn collect_button_label(&self, children: &[AuraNode]) -> String {
        let mut parts: Vec<String> = Vec::new();
        for child in children {
            self.collect_text_parts(child, &mut parts);
        }
        if parts.is_empty() {
            return "\"\"".to_string();
        }
        if parts.len() == 1 {
            return format!("format!(\"{{}}\", {})", parts[0]);
        }
        let fmt = parts.iter().map(|_| "{}").collect::<Vec<_>>().join("\\n");
        format!("format!(\"{}\", {})", fmt, parts.join(", "))
    }

    /// Recursive helper for `collect_button_label`: push a Rust String
    /// expression for each text-bearing node into `parts`.
    fn collect_text_parts(&self, node: &AuraNode, parts: &mut Vec<String>) {
        match node {
            AuraNode::Text(content) => match content {
                AuraTextContent::Literal(s) => {
                    parts.push(format!("\"{}\".to_string()", s));
                }
                AuraTextContent::Interpolated { template, bindings } => {
                    let mut fmt = template.clone();
                    let mut args: Vec<String> = Vec::new();
                    for name in bindings.iter() {
                        if let Some(start) = fmt.find("${") {
                            if let Some(end) = fmt[start..].find('}') {
                                fmt.replace_range(start..=start + end, "{}");
                            }
                        }
                        let stripped = name.trim_start_matches('.');
                        args.push(format!("self.{}", stripped));
                    }
                    if args.is_empty() {
                        parts.push(format!("\"{}\".to_string()", fmt));
                    } else {
                        parts.push(format!("format!(\"{}\", {})", fmt, args.join(", ")));
                    }
                }
            },
            AuraNode::Element { tag, props, children, .. } => {
                if tag == "text" {
                    if let Some(AuraPropValue::Expr(expr)) = props.get("text") {
                        parts.push(self.ast_expr_to_rust(expr));
                        return;
                    }
                    for c in children {
                        self.collect_text_parts(c, parts);
                    }
                } else if tag == "col" || tag == "row" || tag == "column" {
                    for c in children {
                        self.collect_text_parts(c, parts);
                    }
                }
            }
            _ => {}
        }
    }

    /// Plan 407: generate a for-loop's map expression for grid cells (no col wrapper).
    /// Calls generate_view_tree (which produces View::col().children(MAP.collect()).build())
    /// then strips the col wrapper to get just MAP.
    fn generate_for_loop_cells(&mut self, node: &AuraNode) -> String {
        let col_expr = self.generate_view_tree(node);
        // Strip "View::col().children(" prefix and ").collect::<Vec<_>>()).build()" suffix
        if col_expr.starts_with("View::col().children(") && col_expr.ends_with(".collect::<Vec<_>>()).build()") {
            let inner = &col_expr["View::col().children(".len()..col_expr.len() - ".collect::<Vec<_>>()).build()".len()];
            inner.to_string()
        } else {
            col_expr
        }
    }

    fn generate_view_tree(&mut self, node: &AuraNode) -> String {
        match node {
            AuraNode::Element { tag, props, events, children, .. } => {
                // PLAN-571: button 注入 variant/size preset（单源 ui::style::variants，
                // 与 VM 臂同表）；preset 前置、user class 后置（后类胜，与 VM 臂同语义）。
                let props = self.with_button_preset(tag, props);
                let props: &std::collections::HashMap<String, AuraPropValue> = props.as_ref();
                // PLAN-036 T-02：prop 迭代序确定化——HashMap RandomState
                // 进程间随机序曾致再生成漂移（freshness 门字节对拍所需）。
                let props_sorted: Vec<(&String, &AuraPropValue)> = {
                    let mut v: Vec<(&String, &AuraPropValue)> = props.iter().collect();
                    v.sort_by(|a, b| a.0.cmp(b.0));
                    v
                };
                // PLAN-039 D1-A（§5.1 定案记录）：button `ondblclick` 从
                // 事件流剥离——View::Button 无双击槽（view.rs:1650-1657），
                // 出口统一 MouseArea 包裹降级（wrap_button_double_click：
                // 既有原语 on_double_click，Plan 496 M5，三消费端零涟漪）。
                // 非 button tag 的 ondblclick 维持事件流（拒绝门响亮拒——I1）。
                let dbl_event: Option<&AuraEvent> = if tag == "button" {
                    events.iter()
                        .find(|(e, _)| e.split('.').next() == Some("ondblclick"))
                        .map(|(_, h)| h)
                } else {
                    None
                };
                let events_sorted: Vec<(&String, &AuraEvent)> = {
                    let mut v: Vec<(&String, &AuraEvent)> = events.iter()
                        .filter(|(e, _)| {
                            !(tag == "button" && e.split('.').next() == Some("ondblclick"))
                        })
                        .collect();
                    v.sort_by(|a, b| a.0.cmp(b.0));
                    v
                };
                // Handle custom widget references (e.g., EditorPanel, Sidebar)
                if self.is_custom_widget(tag) {
                    return self.generate_child_component(tag, props);
                }

                // SVG support: compile into View::image_styled("svgdoc:<svg>...</svg>", style)
                if tag == "svg" {
                    return self.generate_svg_element(node);
                }

                // grid-item is transparent — emit its child(ren) directly. A
                // wrapping col would be Shrink-width and break the enclosing
                // grid's equal-column Fill distribution.
                if tag == "grid-item" {
                    if children.len() == 1 {
                        return self.generate_view_tree(&children[0]);
                    } else if !children.is_empty() {
                        let mut col = "View::col()".to_string();
                        for child in children {
                            col = format!("{}.child({})", col, self.generate_view_tree(child));
                        }
                        return format!("{}.build()", col);
                    }
                    return "View::Empty".to_string();
                }

                // Rich-content elements with no native iced mapping (e.g.
                // autodown_editor { content: .note.body }) would otherwise fall
                // through to an empty View::col(), hiding the content. Render
                // their `content` prop as styled text so the data is visible.
                if tag == "autodown_editor" || tag == "markdown" || tag == "editor" {
                    if let Some(AuraPropValue::Expr(expr)) = props.get("content") {
                        let content_expr = self.ast_expr_to_rust(expr);
                        // text_styled takes content by value; clone self-field
                        // references (e.g. self.edit_body) to avoid E0507 moves.
                        let content_expr = if content_expr.starts_with("self.") {
                            format!("{}.clone()", content_expr)
                        } else {
                            content_expr
                        };
                        let style_str = props.get("style").or_else(|| props.get("class"))
                            .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v { Some(s.to_string()) } else { None })
                            .unwrap_or_default();
                        return format!(
                            "View::col().style(\"{}\").child(View::text_styled({}, \"text-sm text-foreground whitespace-pre-wrap\")).build()",
                            style_str, content_expr
                        );
                    }
                }

                // Plan 450 / 019 批次三: AutoDown 面板词汇 a2r 发射 —— 与 VM 侧
                // aura_view_builder 的面板臂同款降级(分解为既有 View 变体,Plan 319
                // 单臂规则,renderer 零改动),样式串两处保持一致。heading 的 level /
                // callout 的 kind 为字面量时静态选样式;动态表达式则发射 match(全臂
                // 覆盖,未知值落 zinc 档)——生成代码保持纯表达式形态。
                if tag == "heading" {
                    let content_expr = props.get("text").or_else(|| props.get("content"))
                        .map(|v| self.autodown_panel_prop_expr(v))
                        .unwrap_or_else(|| "\"\".to_string()".to_string());
                    // level: Int 或可 parse 的 Str 字面量 → 静态选样式(grid 特例
                    // 同款 parse);其他表达式 → 发射全臂 match(纯表达式形态)。
                    let level_static = props.get("level").and_then(|v| match v {
                        AuraPropValue::Expr(crate::ast::Expr::Int(n)) => Some(*n),
                        AuraPropValue::Expr(crate::ast::Expr::Str(s)) => s.trim().parse::<i32>().ok(),
                        _ => None,
                    });
                    let style_expr = if let Some(n) = level_static {
                        format!("\"{}\"", Self::autodown_heading_style(n))
                    } else if let Some(AuraPropValue::Expr(expr)) = props.get("level") {
                        let lvl = self.ast_expr_to_rust_no_to_string(expr);
                        format!(
                            "match {lvl} {{ 1 => \"{}\", 2 => \"{}\", 3 => \"{}\", 4 => \"{}\", 5 => \"{}\", _ => \"{}\" }}",
                            Self::autodown_heading_style(1), Self::autodown_heading_style(2),
                            Self::autodown_heading_style(3), Self::autodown_heading_style(4),
                            Self::autodown_heading_style(5), Self::autodown_heading_style(6),
                        )
                    } else {
                        format!("\"{}\"", Self::autodown_heading_style(1))
                    };
                    return format!("View::text_styled({}, {})", content_expr, style_expr);
                }
                if tag == "quote" || tag == "blockquote" {
                    let inner = self.autodown_panel_children_col(children)
                        .unwrap_or_else(|| {
                            props.get("text").or_else(|| props.get("content"))
                                .map(|v| self.autodown_panel_prop_expr(v))
                                .map(|e| format!("View::text_styled({}, \"text-sm\")", e))
                                .unwrap_or_else(|| "View::text(\"\".to_string())".to_string())
                        });
                    return format!(
                        "View::container({}).style(\"border-l-4 pl-4 py-2 w-full text-muted-foreground\").build()",
                        inner
                    );
                }
                if tag == "callout" {
                    let kind_static = props.get("kind")
                        .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v { Some(s.to_string()) } else { None });
                    // (容器 tint, 标题样式) —— 与 VM 侧面板臂同表。
                    let tint_of = |k: &str| match k {
                        "tip" | "success" => ("border-emerald-500/40 bg-emerald-500/10", "text-sm font-medium text-emerald-400"),
                        "warning" | "warn" => ("border-amber-500/40 bg-amber-500/10", "text-sm font-medium text-amber-400"),
                        "danger" | "error" | "caution" => ("border-red-500/40 bg-red-500/10", "text-sm font-medium text-red-400"),
                        "note" | "info" => ("border-blue-500/40 bg-blue-500/10", "text-sm font-medium text-blue-400"),
                        _ => ("border-zinc-500/40 bg-zinc-500/10", "text-sm font-medium text-zinc-400"),
                    };
                    let (tint_expr, title_style) = match (&kind_static, props.get("kind")) {
                        (Some(k), _) => (format!("\"{}\"", tint_of(k).0), tint_of(k).1.to_string()),
                        (None, Some(AuraPropValue::Expr(expr))) => {
                            // 动态 kind:容器样式走 match,标题样式退中性色。
                            let k = self.ast_expr_to_rust(expr);
                            let arms = format!(
                                "match {k}.as_str() {{ \"tip\" | \"success\" => \"{}\", \"warning\" | \"warn\" => \"{}\", \"danger\" | \"error\" | \"caution\" => \"{}\", \"note\" | \"info\" => \"{}\", _ => \"{}\" }}",
                                tint_of("tip").0, tint_of("warning").0, tint_of("danger").0, tint_of("note").0, tint_of("").0,
                            );
                            (arms, "text-sm font-medium".to_string())
                        }
                        _ => (format!("\"{}\"", tint_of("").0), tint_of("").1.to_string()),
                    };
                    let mut col = "View::col()".to_string();
                    if let Some(AuraPropValue::Expr(expr)) = props.get("title") {
                        let title_expr = self.ast_expr_to_rust(expr);
                        let title_expr = if title_expr.starts_with("self.") { format!("{}.clone()", title_expr) } else { title_expr };
                        col = format!("{}.child(View::text_styled({}, \"{}\"))", col, title_expr, title_style);
                    }
                    for c in children {
                        col = format!("{}.child({})", col, self.generate_view_tree(c));
                    }
                    return format!(
                        "View::container({}.build()).style(&format!(\"rounded-lg border {{}} p-4 w-full\", {})).build()",
                        col, tint_expr
                    );
                }
                if tag == "details" {
                    let summary_expr = props.get("summary")
                        .map(|v| self.autodown_panel_prop_expr(v))
                        .unwrap_or_else(|| "\"Details\".to_string()".to_string());
                    let mut item = format!("auto_lang::ui::view::AccordionItem::new({})", summary_expr);
                    if !children.is_empty() {
                        let kids: Vec<String> = children.iter().map(|c| self.generate_view_tree(c)).collect();
                        item = format!("{}.with_children(vec![{}])", item, kids.join(", "));
                    }
                    item = format!("{}.with_expanded(true)", item);
                    return format!("View::accordion().items(vec![{}]).build()", item);
                }
                // 注册位面板(消费方注册渲染器,plan 017 待澄清 #2):iced 侧降级为
                // 可见的源码/引用文本,避免内容静默丢弃。tag 族与 VM 侧一致。
                if tag == "math_block" || tag == "mathblock" || tag == "math-block" {
                    let source = props.get("source").map(|v| self.autodown_panel_prop_expr(v))
                        .unwrap_or_else(|| "\"\".to_string()".to_string());
                    return format!(
                        "View::container(View::text_styled({}, \"font-mono text-sm\")).style(\"rounded-lg border bg-card p-4 w-full\").build()",
                        source
                    );
                }
                if tag == "query_block" || tag == "queryblock" || tag == "query-block" {
                    let query = props.get("query").map(|v| self.autodown_panel_prop_expr(v))
                        .unwrap_or_else(|| "\"\".to_string()".to_string());
                    return format!(
                        "View::container(View::text_styled({}, \"font-mono text-xs text-muted-foreground\")).style(\"rounded-lg border p-3 w-full\").build()",
                        query
                    );
                }
                if tag == "embed_block" || tag == "embedblock" || tag == "embed-block" {
                    let target = props.get("target").map(|v| self.autodown_panel_prop_expr(v))
                        .unwrap_or_else(|| "\"\".to_string()".to_string());
                    return format!(
                        "View::container(View::text_styled(format!(\"↪ {{}}\", {}), \"text-sm text-muted-foreground\")).style(\"rounded-lg border bg-muted p-3 w-full\").build()",
                        target
                    );
                }

                // PLAN-534: hovercard 家族 → MouseArea 包锚 + Bottom 非模态
                // Popover（根臂）;组外兜底 trigger/content 透传。
                if let Some(hrole) = Self::hover_card_role(tag) {
                    match hrole {
                        "root" => return self.generate_hover_card_popover(props, children),
                        "trigger" | "content" => {
                            let views: Vec<String> = children
                                .iter()
                                .map(|c| self.generate_view_tree(c))
                                .collect();
                            return match views.len() {
                                0 => "auto_lang::ui::view::View::Empty".to_string(),
                                1 => views.into_iter().next().unwrap(),
                                _ => {
                                    let mut b = "View::row()".to_string();
                                    for v in views {
                                        b = format!("{}.child({})", b, v);
                                    }
                                    format!("{}.build()", b)
                                }
                            };
                        }
                        _ => {}
                    }
                }

                // PLAN-533 T3: 模态对话框家族（alert-dialog/dialog）→
                // View::Popover 模态构造（与解释器侧 PLAN-530 W13 臂同形态,
                // 双轨视觉一致）。根臂拆解 trigger/content;组外兜底子件按
                // 角色降级渲染（透传/预设样式/按钮预设）。
                if let Some(role) = Self::modal_dialog_tag_role(tag) {
                    match role {
                        "root" => return self.generate_modal_popover(tag, props, children),
                        "trigger" | "content" => {
                            // 透传:组内已被根臂拆解消费;组外裸渲染子件。
                            let views: Vec<String> = children
                                .iter()
                                .map(|c| self.generate_view_tree(c))
                                .collect();
                            return match views.len() {
                                0 => "auto_lang::ui::view::View::Empty".to_string(),
                                1 => views.into_iter().next().unwrap(),
                                _ => {
                                    let mut b = "View::row()".to_string();
                                    for v in views {
                                        b = format!("{}.child({})", b, v);
                                    }
                                    format!("{}.build()", b)
                                }
                            };
                        }
                        "title" | "description" => {
                            let preset = if role == "title" {
                                "text-lg font-semibold"
                            } else {
                                "text-sm text-muted-foreground"
                            };
                            let label = Self::modal_child_label(props, children);
                            let user_class = Self::modal_user_class(props);
                            let class = if user_class.is_empty() {
                                preset.to_string()
                            } else {
                                format!("{} {}", preset, user_class)
                            };
                            let text_expr = if label.contains("${") {
                                self.interpolate_str(&label)
                            } else {
                                format!("\"{}\".to_string()", label)
                            };
                            return format!("View::text_styled({}, \"{}\")", text_expr, class);
                        }
                        "header" => {
                            let mut b = "View::col()".to_string();
                            for c in children {
                                b = format!("{}.child({})", b, self.generate_view_tree(c));
                            }
                            return format!(
                                "{}.style(\"flex flex-col gap-2\").build()",
                                b
                            );
                        }
                        "footer" => {
                            let mut b = "View::row()".to_string();
                            for c in children {
                                b = format!("{}.child({})", b, self.generate_view_tree(c));
                            }
                            return format!(
                                "{}.style(\"flex justify-end gap-2\").build()",
                                b
                            );
                        }
                        "item" => {
                            // PLAN-533 T7: dropdown-menu-item——有 onclick
                            // 走按钮（菜单项交互预设）;纯文本项 text_styled。
                            let preset = "w-full px-2 py-1.5 text-sm cursor-pointer hover:bg-secondary text-start";
                            let label = Self::modal_child_label(props, children);
                            let onclick = ["onclick", "onClick", "on_click"]
                                .iter()
                                .find_map(|k| events.get(*k))
                                .map(|h| self.handler_to_rust_closure_with_params(&h.handler, &h.params));
                            return match onclick {
                                Some(cl) => format!(
                                    "View::button(\"{}\").style(\"{}\").on_click({}).build()",
                                    label, preset, cl
                                ),
                                None => format!(
                                    "View::text_styled(\"{}\".to_string(), \"{}\")",
                                    label, preset
                                ),
                            };
                        }
                        "label" => {
                            let label = Self::modal_child_label(props, children);
                            return format!(
                                "View::text_styled(\"{}\".to_string(), \"px-2 py-1.5 text-sm font-semibold\")",
                                label
                            );
                        }
                        "separator" => {
                            return "View::col().style(\"w-full h-px bg-border my-1\").build()".to_string();
                        }
                        "cancel" | "close" => {
                            return self.generate_modal_button(
                                props,
                                events,
                                children,
                                "border border-input bg-background text-foreground rounded-md hover:bg-secondary hover:text-secondary-foreground h-10 px-4",
                            );
                        }
                        "action" => {
                            return self.generate_modal_button(
                                props,
                                events,
                                children,
                                "bg-primary text-primary-foreground font-medium rounded-md hover:bg-primary/90 h-10 px-4",
                            );
                        }
                        _ => {}
                    }
                }

                // PLAN-027 T-04 扩面（普查修正 C）：mouse-area codegen 臂——
                // 设计 §3a-a5 误记"shell 未直接用"，实勘五件 26 处（desktop
                // ×12/shell ×7/notification ×5/dashboard ×2）。映射与解释臂
                // convert_mouse_area_untracked（aura_view_builder.rs:11637）
                // 全同源：onmouseenter|onhover→on_enter、onmouseleave|
                // onhoverout→on_exit、ondblclick→on_double_click、
                // onmousedown|onclick→on_click、onmouseup→on_release、
                // oncontextmenu(.prevent)→on_context_menu。事件槽 = Option<M>
                // 直发消息（非闭包）。PLAN-668 R-01：onmousemove+coords
                // 坐标面回迁（PLAN-022 T-03 能力——027 T-04 本臂曾硬编码
                // None，把后位 022 臂挡成死代码致发射漂移，死臂已删）。
                if tag == "mouse-area" || tag == "mouse_area" {
                    let content = if children.is_empty() {
                        "auto_lang::ui::view::View::Empty".to_string()
                    } else if children.len() == 1 {
                        self.generate_view_tree(&children[0])
                    } else {
                        let mut col = "View::col()".to_string();
                        for c in children {
                            col = format!("{}.child({})", col, self.generate_view_tree(c));
                        }
                        format!("{col}.build()")
                    };
                    let ev = |names: &[&str]| -> Option<String> {
                        events.iter().find(|(k, _)| {
                            let base = k.split('.').next().unwrap_or(k);
                            names.contains(&base)
                        }).map(|(_, h)| {
                            format!(
                                "Some({})",
                                self.handler_to_rust_direct_msg(&h.handler, &h.params)
                            )
                        })
                    };
                    let user_style = props
                        .get("style")
                        .or_else(|| props.get("class"))
                        .and_then(|v| {
                            if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v {
                                Some(s.to_string())
                            } else {
                                None
                            }
                        });
                    let style_expr = match user_style {
                        Some(s) => {
                            format!("auto_lang::ui::style::Style::parse(\"{s}\").ok()")
                        }
                        None => "None".to_string(),
                    };
                    // PLAN-668 R-01（PLAN-022 T-03 回迁）：onmousemove →
                    // PointerMoveHandler——无参绑定转发归一坐标（变体须双
                    // float 载荷，020 分隔条契约形态）；显式参绑定弃坐标。
                    let move_h = events
                        .iter()
                        .find(|(k, _)| {
                            let base = k.split('.').next().unwrap_or(k);
                            base == "onmousemove" || base == "on_mouse_move"
                        })
                        .map(|(_, h)| {
                            let variant = self.extract_variant_name(&h.handler);
                            let msg_name = self.current_msg_name();
                            if h.params.is_empty() {
                                format!(
                                    "Some(auto_lang::ui::view::PointerMoveHandler::new(move |x: f32, y: f32| {msg_name}::{variant}(x, y)))"
                                )
                            } else {
                                let converted: Vec<String> = h
                                    .params
                                    .iter()
                                    .map(|p| self.convert_param_value_access(p, &variant))
                                    .collect();
                                format!(
                                    "Some(auto_lang::ui::view::PointerMoveHandler::new(move |x: f32, y: f32| {{ let _ = (x, y); {msg_name}::{variant}({}) }}))",
                                    converted.join(", ")
                                )
                            }
                        })
                        .unwrap_or_else(|| "None".to_string());
                    // coords "WxH" → logical_extent（{:?} f32 定点打印，
                    // 整数字面量在约束位会推成 i32）。
                    let extent = props
                        .get("coords")
                        .and_then(|v| match v {
                            AuraPropValue::Expr(crate::ast::Expr::Str(s)) => {
                                s.split_once(['x', 'X']).and_then(|(w, hh)| {
                                    match (w.trim().parse::<f32>(), hh.trim().parse::<f32>()) {
                                        (Ok(w), Ok(hh)) if w > 0.0 && hh > 0.0 => {
                                            Some(format!("Some(({w:?}, {hh:?}))"))
                                        }
                                        _ => None,
                                    }
                                })
                            }
                            _ => None,
                        })
                        .unwrap_or_else(|| "None".to_string());
                    return format!(
                        "View::MouseArea {{ content: Box::new({content}), on_enter: {}, on_exit: {}, on_double_click: {}, on_click: {}, on_context_menu: {}, on_release: {}, on_move: {move_h}, logical_extent: {extent}, style: {} }}",
                        ev(&["onmouseenter", "onhover"]).unwrap_or_else(|| "None".to_string()),
                        ev(&["onmouseleave", "onhoverout"]).unwrap_or_else(|| "None".to_string()),
                        ev(&["ondblclick"]).unwrap_or_else(|| "None".to_string()),
                        ev(&["onmousedown", "onclick", "onClick", "on_click"])
                            .unwrap_or_else(|| "None".to_string()),
                        ev(&["oncontextmenu"]).unwrap_or_else(|| "None".to_string()),
                        ev(&["onmouseup"]).unwrap_or_else(|| "None".to_string()),
                        style_expr
                    );
                }

                // PLAN-027 T-02: 裸 popover 臂 —— View::Popover 直发（此前
                // 落 tag_to_view_fn `_ => "col"` 降级，open/placement/
                // ondismiss/x/y 静默丢弃）。语义与解释侧 convert_popover
                // （aura_view_builder.rs:8449）对齐（PLAN-027 parity 锚）：
                // x/y 双全 = 坐标锚（placement 缺省 BottomStart，children
                // 全为面板）；否则首 plain 子 = 锚件（placement 缺省
                // Bottom，其余子 = 面板列）。详见 generate_bare_popover。
                if tag == "popover" {
                    return self.generate_bare_popover(props, events, children);
                }

                // grid → View::grid() builder. iced has no native grid; the
                // col-of-rows decomposition (final-row padding + w-full rows)
                // now lives in ONE place — the shared generic `build_grid`
                // (Plan 319) — so the rust `into_iced` path and the VM
                // `render_dynamic_view` path share it and can never drift.
                if tag == "grid" {
                    // PLAN-039 D4（§5.1 定案记录）：cols 双臂——字面量维持
                    // 编译期提取；Dot/点链 Ident 表达式走运行期求值臂。
                    // View::Grid.cols 为运行期 usize 字段（view.rs:1658-1661
                    // builder `.cols(usize)` + build 时 `.max(1)`），IR 无
                    // 编译期常量假设——静态近似不诚实（难度切换整局重开
                    // 时列数变化需正确）。`.store.cols` 经 ast_expr_to_rust
                    // （:7151 `.starts_with(".store.")` 臂）→ `self.store.cols`；
                    // `self..` 双点残留为 "." 基座 Dot 形的已知伪影，剥之。
                    let cols_prop = props.get("cols").or_else(|| props.get("columns"));
                    let cols_static: Option<usize> = cols_prop
                        .and_then(|v| match v {
                            AuraPropValue::Expr(crate::ast::Expr::Int(n)) => Some(*n as usize),
                            AuraPropValue::Expr(crate::ast::Expr::Str(s)) => s.trim().parse::<usize>().ok(),
                            _ => None,
                        })
                        .map(|c| c.max(1));
                    let cols = match cols_static {
                        Some(c) => c.to_string(),
                        None => cols_prop
                            .and_then(|v| match v {
                                AuraPropValue::Expr(e @ crate::ast::Expr::Dot(..)) => Some(
                                    format!("({}) as usize", self.ast_expr_to_rust(e).replace("self..", "self.")),
                                ),
                                AuraPropValue::Expr(crate::ast::Expr::Ident(name))
                                    if name.as_str().starts_with('.') => Some(
                                    format!("({}) as usize", self.ast_expr_to_rust(&crate::ast::Expr::Ident(name.clone())).replace("self..", "self.")),
                                ),
                                _ => None,
                            })
                            .unwrap_or_else(|| "1".to_string()),
                    };
                    let gap = props.get("gap")
                        .and_then(|v| match v {
                            AuraPropValue::Expr(crate::ast::Expr::Int(n)) => Some(*n as u16),
                            AuraPropValue::Expr(crate::ast::Expr::Str(s)) => s.trim().parse::<u16>().ok(),
                            _ => None,
                        })
                        .unwrap_or(0);
                    let style_str = props.get("style").or_else(|| props.get("class"))
                        .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v {
                            Some(s.to_string())
                        } else { None })
                        .unwrap_or_default();

                    let mut g = "View::grid()".to_string();
                    g = format!("{}.cols({})", g, cols);
                    if gap > 0 { g = format!("{}.spacing({})", g, gap); }
                    for c in children {
                        // Plan 407: for grid children that are ForLoops, use
                        // .children() (bulk add) instead of .child() (single).
                        // A ForLoop generates View::col().children(map.collect()).build(),
                        // which would be one child (a col) — the grid then treats it as
                        // a single cell. Instead, unwrap: pass the map directly to
                        // .children() so each iteration becomes a separate grid cell.
                        match c {
                            AuraNode::ForLoop { .. } => {
                                // Generate the map expression WITHOUT the col wrapper.
                                let map_expr = self.generate_for_loop_cells(c);
                                g = format!("{}.children({}.collect::<Vec<_>>())", g, map_expr);
                            }
                            _ => {
                                g = format!("{}.child({})", g, self.generate_view_tree(c));
                            }
                        }
                    }
                    if !style_str.is_empty() {
                        g = format!("{}.style(\"{}\")", g, style_str);
                    }
                    return format!("{}.build()", g);
                }


                // PLAN-013 T3: terminal 臂——View::Terminal 真身组件(props-feed
                // 形态甲)直达发射。key 为状态存储键;cols/rows 字面量或 .field
                // 绑定;lines 为 Vec<String> 表达式(识别 .field → self.field.clone())。
                // 014 直键入:oninput 信号位发射(Some(AppMsg::X));载荷走
                // TerminalCore 键入队列(宿主引擎泵排空),消息不带载荷。
                if tag == "terminal" {
                    // PLAN-019 T-00 勘定:key 支持 .field 动态绑定(多 Pane
                    // 槽位键 "pane-<id>" 随 Tab/分屏切换;T-B 布局消费面必需,
                    // VM 臂 extract_string_with(bindings) 既有同语义)。字面量
                    // 原样;Ident → self.field 克隆(lines 臂同款);其余回落
                    // "main"。
                    let key_expr = match props.get("key") {
                        Some(AuraPropValue::Expr(crate::ast::Expr::Str(s))) => format!("\"{s}\".to_string()"),
                        Some(AuraPropValue::Expr(crate::ast::Expr::Ident(id))) => {
                            format!("self.{}.clone()", id.as_str())
                        }
                        Some(AuraPropValue::Expr(expr)) => {
                            // `.field` 实际解析为 FieldAccess(lines 同款回退);
                            // 只有 self. 前缀的求值可信,其余落 "main"。
                            let e = self.ast_expr_to_rust(expr);
                            if e.starts_with("self.") {
                                format!("{e}.clone()")
                            } else {
                                "\"main\".to_string()".to_string()
                            }
                        }
                        _ => "\"main\".to_string()".to_string(),
                    };
                    let geom = |name: &str, dft: u16| -> String {
                        match props.get(name) {
                            Some(AuraPropValue::Expr(crate::ast::Expr::Int(n))) => format!("{n}u16"),
                            Some(AuraPropValue::Expr(crate::ast::Expr::Ident(id))) => format!("self.{} as u16", id.as_str()),
                            // `.field` 实际解析为 FieldAccess(lines 同款回退);
                            // 只有 self. 前缀的求值可信,其余落默认。
                            Some(AuraPropValue::Expr(expr)) => {
                                let e = self.ast_expr_to_rust(expr);
                                if e.starts_with("self.") {
                                    format!("({e}) as u16")
                                } else {
                                    format!("{dft}u16")
                                }
                            }
                            _ => format!("{dft}u16"),
                        }
                    };
                    let lines = match props.get("lines") {
                        Some(AuraPropValue::Expr(crate::ast::Expr::Ident(id))) => format!("self.{}.clone()", id.as_str()),
                        Some(AuraPropValue::Expr(expr)) => {
                            let e = self.ast_expr_to_rust(expr);
                            if e.starts_with("self.") {
                                format!("{}.clone()", e)
                            } else {
                                e
                            }
                        }
                        _ => "Vec::new()".to_string(),
                    };
                    // PLAN-019:scheme 同款——.field FieldAccess 绑定支持
                    // (仅认 Ident 时动态绑定静默回落 0)。
                    let scroll = match props.get("scroll_offset") {
                        Some(AuraPropValue::Expr(crate::ast::Expr::Ident(id))) => format!("self.{} as u16", id.as_str()),
                        Some(AuraPropValue::Expr(expr)) => {
                            let e = self.ast_expr_to_rust(expr);
                            if e.starts_with("self.") {
                                format!("({e}) as u16")
                            } else {
                                "0u16".to_string()
                            }
                        }
                        _ => "0u16".to_string(),
                    };
                    // 014 光标格:字面量或 .field 绑定(缺省 0,0 = 占位)。
                    let cursor = |name: &str| -> String {
                        match props.get(name) {
                            Some(AuraPropValue::Expr(crate::ast::Expr::Int(n))) => format!("{n}u16"),
                            Some(AuraPropValue::Expr(crate::ast::Expr::Ident(id))) => format!("self.{} as u16", id.as_str()),
                            Some(AuraPropValue::Expr(expr)) => {
                                let e = self.ast_expr_to_rust(expr);
                                if e.starts_with("self.") {
                                    format!("({e}) as u16")
                                } else {
                                    "0u16".to_string()
                                }
                            }
                            _ => "0u16".to_string(),
                        }
                    };
                    let on_input = ["oninput", "input", "onkey"]
                        .iter()
                        .find_map(|k| events.get(*k))
                        .map(|h| self.handler_to_rust_direct_msg(&h.handler, &h.params));
                    let on_input_expr = match on_input {
                        Some(msg) => format!("Some({msg})"),
                        None => "None".to_string(),
                    };
                    // PLAN-015 D3:onmenu 信号位——右键菜单项选择(载荷走
                    // TerminalCore 菜单通道,宿主经 terminal_take_menu_item(_any)
                    // 取走;消息只当触发器,对齐 oninput 模式)。键集与 VM 臂
                    // (aura_view_builder convert_terminal 的 oncontextmenu/
                    // contextmenu/onmenu)同源,另收 on_menu 拼写。
                    let on_menu_expr = ["onmenu", "on_menu", "oncontextmenu", "contextmenu"]
                        .iter()
                        .find_map(|k| events.get(*k))
                        .map(|h| {
                            format!("Some({})", self.handler_to_rust_direct_msg(&h.handler, &h.params))
                        })
                        .unwrap_or_else(|| "None".to_string());
                    // PLAN-018 D10:scheme prop(Int 字面量或 .field 绑定;
                    // 缺省 -1 = 跟随桌面主题)。PLAN-019:`.field` 实际解析为
                    // FieldAccess(geom/lines 同款回退)——仅认 Ident 时动态
                    // 绑定静默回落 -1,scheme 按钮失效(app.at 实测)。
                    let scheme = match props.get("scheme") {
                        Some(AuraPropValue::Expr(crate::ast::Expr::Int(n))) => format!("{n}i32"),
                        Some(AuraPropValue::Expr(crate::ast::Expr::Ident(id))) => {
                            format!("self.{} as i32", id.as_str())
                        }
                        Some(AuraPropValue::Expr(expr)) => {
                            let e = self.ast_expr_to_rust(expr);
                            if e.starts_with("self.") {
                                format!("({e}) as i32")
                            } else {
                                "-1i32".to_string()
                            }
                        }
                        _ => "-1i32".to_string(),
                    };
                    // PLAN-019 D4:应用级捷径表 —— onkeydown.<键名> 事件收集
                    // (VM 臂 convert_terminal 收集器同款语义;规范化键名沿
                    // terminal_key_binding_name:ctrl./alt./shift. 前缀小写)。
                    let mut shortcut_pairs: Vec<String> = Vec::new();
                    for (ek, h) in events.iter() {
                        if let Some(rest) = ek.strip_prefix("onkeydown.") {
                            let norm = rest
                                .split('.')
                                .filter(|seg| {
                                    !matches!(*seg, "prevent" | "stop" | "exact" | "capture" | "self" | "once")
                                })
                                .collect::<Vec<_>>()
                                .join(".")
                                .to_lowercase();
                            if !norm.is_empty() {
                                let msg = self.handler_to_rust_direct_msg(&h.handler, &h.params);
                                shortcut_pairs.push(format!("(\"{norm}\".to_string(), {msg})"));
                            }
                        }
                    }
                    return format!(
                        "View::Terminal {{ key: {key_expr}, cols: {}, rows: {}, lines: {lines}, scroll_offset: {scroll}, preedit: None, on_select: None, on_menu: {on_menu_expr}, on_input: {on_input_expr}, cursor_row: {}, cursor_col: {}, scheme: {scheme}, shortcuts: vec![{}], style: None }}",
                        geom("cols", 80),
                        geom("rows", 24),
                        cursor("cursor_row"),
                        cursor("cursor_col"),
                        shortcut_pairs.join(", "),
                    );
                }

                let view_fn = self.tag_to_view_fn(tag);

                // For text elements with a "text" prop and no extra styling/events,
                // emit View::text("content") or View::text(format!(...)) directly.
                if tag == "text" && children.is_empty() && events.is_empty() {
                    let style_count = props.keys()
                        .filter(|k| *k != "text")
                        .count();
                    if style_count == 0 {
                        if let Some(AuraPropValue::Expr(crate::ast::Expr::Str(s))) = props.get("text") {
                            if s.contains("${") {
                                return format!("View::text({})", self.interpolate_str(s));
                            }
                            return format!("View::text(\"{}\".to_string())", s);
                        }
                        if let Some(AuraPropValue::Expr(crate::ast::Expr::Ident(name))) = props.get("text") {
                            let name_str = name.as_str();
                            if self.is_loop_var(name_str) {
                                return format!("View::text(format!(\"{{}}\", {}))", name_str);
                            }
                            return format!("View::text(format!(\"{{}}\", self.{}))", name_str);
                        }
                    } else {
                        // Text with styling — collect classes and use View::text_styled
                        let class_str = props.get("style")
                            .or_else(|| props.get("class"))
                            .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v { Some(s.to_string()) } else { None })
                            .unwrap_or_default();
                        if let Some(AuraPropValue::Expr(crate::ast::Expr::Ident(name))) = props.get("text") {
                            let name_str = name.as_str();
                            if self.is_loop_var(name_str) {
                                return format!("View::text_styled(format!(\"{{}}\", {}), \"{}\")", name_str, class_str);
                            }
                            return format!("View::text_styled(format!(\"{{}}\", self.{}), \"{}\")", name_str, class_str);
                        }
                        if let Some(AuraPropValue::Expr(crate::ast::Expr::Str(s))) = props.get("text") {
                            if s.contains("${") {
                                return format!("View::text_styled({}, \"{}\")", self.interpolate_str(s), class_str);
                            }
                            return format!("View::text_styled(\"{}\".to_string(), \"{}\")", s, class_str);
                        }
                    }
                }

                // Special handling for input elements — View::input(placeholder).value(...).on_change(...)
                if tag == "input" {
                    let placeholder = props.get("placeholder")
                        .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v { Some(s.to_string()) } else { None })
                        .unwrap_or_default();

                    let mut builder = format!("View::input(\"{}\")", placeholder);

                    // Value binding: value: .field → .value(format!("{}", self.field))
                    // PLAN-025 T-07：绑定形状容差对齐 client_runtime::
                    // binding_field（Ident 带 '.' 前缀 / Dot("."|"self", f)
                    // ——build_rust_ui 提取路径产 Dot 形，缺臂曾致 003
                    // 真源 value 绑定整段丢失）。
                    let value_field = props.get("value").and_then(|v| match v {
                        AuraPropValue::Expr(crate::ast::Expr::Ident(name)) => {
                            let f = name.as_str().trim_start_matches('.');
                            // PLAN-039 T-03④：单级才可作 input_fields 写回
                            // 注册名（多级点链拍平形 `.store.edit_title` 走
                            // 下方通用求值臂——注册点链名会落 f64 缺省写）。
                            (!f.is_empty() && !f.contains('.')).then(|| f.to_string())
                        }
                        AuraPropValue::Expr(crate::ast::Expr::Dot(obj, field)) => {
                            match obj.as_ref() {
                                crate::ast::Expr::Ident(base)
                                    if base.as_str() == "." || base.as_str() == "self" =>
                                {
                                    Some(field.as_str().to_string())
                                }
                                _ => None,
                            }
                        }
                        _ => None,
                    });
                    if let Some(name) = &value_field {
                        builder = format!("{}.value(format!(\"{{}}\", self.{}))", builder, name);
                    } else if let Some(AuraPropValue::Expr(crate::ast::Expr::Str(s))) = props.get("value") {
                        builder = format!("{}.value(\"{}\".to_string())", builder, s);
                    } else if let Some(AuraPropValue::Expr(e)) = props.get("value") {
                        // PLAN-039 T-03④：多级点链/复杂表达式值面通用求值
                        // （`.store.edit_title` → `self.store.edit_title`；写回
                        // 由 handler 自持，与 scan_input_fields 同口径）。
                        builder = format!(
                            "{}.value(format!(\"{{}}\", {}))",
                            builder,
                            self.ast_expr_to_rust(e).replace("self..", "self.")
                        );
                    }

                    // Password mode: type: "password"
                    if let Some(AuraPropValue::Expr(crate::ast::Expr::Str(s))) = props.get("type") {
                        if s == "password" {
                            builder = format!("{}.password()", builder);
                        }
                    }

                    // Other props (class, style, width — skip placeholder, value, type)
                    for (key, value) in props_sorted.clone() {
                        if key == "placeholder" || key == "value" || key == "type" { continue; }
                        builder = self.add_prop_to_builder(&builder, key, value);
                    }

                    // Events: oninput/onchange → on_change (takes M, not a closure)
                    //         onenter → on_submit (fires on Enter key)
                    for (event, handler) in events_sorted.clone() {
                        match event.as_str() {
                            "oninput" | "onInput" | "onchange" | "onChange" => {
                                let variant = self.extract_variant_name(&handler.handler);
                                let msg_name = self.current_msg_name();
                                // Plan 374: For variants with String payload, pass a default
                                // value — the handler reads actual text via last_input_text().
                                // This keeps the builder's type parameter as M (not fn pointer).
                                let has_string_payload = self.message_variants.iter()
                                    .find(|v| v.name == variant)
                                    .map(|v| v.payload.first().map_or(false, |t| matches!(t, crate::ast::Type::StrOwned | crate::ast::Type::StrSlice | crate::ast::Type::StrFixed(_))))
                                    .unwrap_or(false);
                                if has_string_payload {
                                    builder = format!("{}.on_change({}::{}(\"\".to_string()))", builder, msg_name, variant);
                                } else {
                                    builder = format!("{}.on_change({}::{})", builder, msg_name, variant);
                                }
                                // Record event→field mapping for handler generation
                                if let Some(name) = &value_field {
                                    self.input_fields.entry(variant).or_default().push(name.clone());
                                }
                            }
                            "onenter" | "onEnter" | "onsubmit" | "onSubmit" => {
                                let variant = self.extract_variant_name(&handler.handler);
                                let msg_name = self.current_msg_name();
                                builder = format!("{}.on_submit({}::{})", builder, msg_name, variant);
                            }
                            _ => {}
                        }
                    }

                    return format!("{}.build()", builder);
                }

                // Special handling for textarea elements — View::textarea(placeholder).value(...).on_change(...)
                if tag == "textarea" {
                    let placeholder = props.get("placeholder")
                        .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v { Some(s.to_string()) } else { None })
                        .unwrap_or_default();

                    let mut builder = format!("View::textarea(\"{}\")", placeholder);

                    // Value binding: value: .field → .value(format!("{}", self.field))
                    // PLAN-039 T-03④（§5.1 定案记录④）：textarea 对齐 input
                    // 容差（PLAN-025 T-07）并补多级点链——字面量 Str /
                    // Ident（'.' 前缀，点链拍平形 `.store.edit_detail` →
                    // `self.store.edit_detail`）/ Dot 表达式通用求值
                    // （ast_expr_to_rust；`self..` 双点伪影剥之）。此前只收
                    // 裸 Ident，kanban `.store.edit_detail` 静默丢初值。
                    // input_fields 写回注册仍只收单级（scan_input_fields
                    // :1896-1907 同口径——store 源值的写回由 handler 自持，
                    // 注入点链名会落 f64 缺省写编译断）。
                    let ta_value_expr: Option<String> = match props.get("value") {
                        Some(AuraPropValue::Expr(crate::ast::Expr::Str(s))) => {
                            Some(format!("\"{}\".to_string()", s))
                        }
                        Some(AuraPropValue::Expr(crate::ast::Expr::Ident(name))) => {
                            let f = name.as_str().trim_start_matches('.');
                            (!f.is_empty())
                                .then(|| format!("format!(\"{{}}\", self.{})", f))
                        }
                        Some(AuraPropValue::Expr(e)) => Some(format!(
                            "format!(\"{{}}\", {})",
                            self.ast_expr_to_rust(e).replace("self..", "self.")
                        )),
                        _ => None,
                    };
                    if let Some(ve) = ta_value_expr {
                        builder = format!("{}.value({})", builder, ve);
                    }

                    // Other props (skip placeholder, value)
                    for (key, value) in props_sorted.clone() {
                        if key == "placeholder" || key == "value" { continue; }
                        builder = self.add_prop_to_builder(&builder, key, value);
                    }

                    // Events: oninput/onchange → on_change
                    for (event, handler) in events_sorted.clone() {
                        match event.as_str() {
                            "oninput" | "onInput" | "onchange" | "onChange" => {
                                let variant = self.extract_variant_name(&handler.handler);
                                let msg_name = self.current_msg_name();
                                // Plan 374: For variants with String payload, pass a default
                                // value — the handler reads actual text via last_input_text().
                                // This keeps the builder's type parameter as M (not fn pointer).
                                let has_string_payload = self.message_variants.iter()
                                    .find(|v| v.name == variant)
                                    .map(|v| v.payload.first().map_or(false, |t| matches!(t, crate::ast::Type::StrOwned | crate::ast::Type::StrSlice | crate::ast::Type::StrFixed(_))))
                                    .unwrap_or(false);
                                if has_string_payload {
                                    builder = format!("{}.on_change({}::{}(\"\".to_string()))", builder, msg_name, variant);
                                } else {
                                    builder = format!("{}.on_change({}::{})", builder, msg_name, variant);
                                }
                                if let Some(AuraPropValue::Expr(crate::ast::Expr::Ident(name))) = props.get("value") {
                                    self.input_fields.entry(variant).or_default().push(name.to_string());
                                }
                            }
                            _ => {}
                        }
                    }

                    return format!("{}.build()", builder);
                }

                // Special handling for code_editor elements (Plan 413) -
                // View::code_editor(key).value(...).lang(...).on_change(...)
                if tag == "code_editor" {
                    let key = props.get("key")
                        .or_else(|| props.get("id"))
                        .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v { Some(s.to_string()) } else { None })
                        .unwrap_or_else(|| "editor".to_string());
                    let mut builder = format!("View::code_editor(\"{}\")", key);

                    // Value binding: content: .field | value: .field (or literal).
                    let value_prop = props.get("content").or_else(|| props.get("value"));
                    if let Some(AuraPropValue::Expr(crate::ast::Expr::Ident(name))) = value_prop {
                        builder = format!("{}.value(self.{}.clone())", builder, name);
                    } else if let Some(AuraPropValue::Expr(crate::ast::Expr::Str(s))) = value_prop {
                        builder = format!("{}.value(\"{}\".to_string())", builder, s);
                    }

                    // Config props.
                    if let Some(AuraPropValue::Expr(crate::ast::Expr::Str(lang))) = props.get("lang") {
                        builder = format!("{}.lang(\"{}\")", builder, lang);
                    }
                    let bool_prop = |props: &std::collections::HashMap<String, AuraPropValue>, name: &str| -> Option<bool> {
                        match props.get(name) {
                            Some(AuraPropValue::Expr(crate::ast::Expr::Bool(b))) => Some(*b),
                            _ => None,
                        }
                    };
                    if let Some(b) = bool_prop(props, "line_numbers") {
                        builder = format!("{}.line_numbers({})", builder, b);
                    }
                    if let Some(b) = bool_prop(props, "wrap") {
                        builder = format!("{}.wrap({})", builder, b);
                    }
                    if let Some(b) = bool_prop(props, "vi") {
                        builder = format!("{}.vi({})", builder, b);
                    }
                    if let Some(b) = bool_prop(props, "highlight_current_line") {
                        builder = format!("{}.highlight_current_line({})", builder, b);
                    }
                    if let Some(AuraPropValue::Expr(crate::ast::Expr::Int(n))) = props.get("tab_width") {
                        builder = format!("{}.tab_width({})", builder, n);
                    }
                    if let Some(AuraPropValue::Expr(crate::ast::Expr::Float(f, _))) = props.get("font_size") {
                        builder = format!("{}.font_size({})", builder, f);
                    }

                    // Plan 413 follow-up: regex search (live highlight).
                    let search_prop = props.get("search");
                    if let Some(AuraPropValue::Expr(crate::ast::Expr::Str(pattern))) = search_prop {
                        builder = format!("{}.search(\"{}\")", builder, pattern);
                    } else if let Some(AuraPropValue::Expr(crate::ast::Expr::Ident(name))) = search_prop {
                        builder = format!("{}.search(self.{}.clone())", builder, name);
                    }

                    // Events: oninput/onchange -> on_change, oncursor -> on_cursor,
                    // oncontextmenu -> on_context_menu.
                    for (event, handler) in events_sorted.clone() {
                        match event.as_str() {
                            "oninput" | "onInput" | "onchange" | "onChange" => {
                                let variant = self.extract_variant_name(&handler.handler);
                                let msg_name = self.current_msg_name();
                                let has_string_payload = self.message_variants.iter()
                                    .find(|v| v.name == variant)
                                    .map(|v| v.payload.first().map_or(false, |t| matches!(t, crate::ast::Type::StrOwned | crate::ast::Type::StrSlice | crate::ast::Type::StrFixed(_))))
                                    .unwrap_or(false);
                                if has_string_payload {
                                    builder = format!("{}.on_change({}::{}(\"\".to_string()))", builder, msg_name, variant);
                                } else {
                                    builder = format!("{}.on_change({}::{})", builder, msg_name, variant);
                                }
                                // Handler reads the text via code_editor_text(key).
                                self.code_editor_sources.entry(variant.clone()).or_default().push(key.clone());
                                if let Some(AuraPropValue::Expr(crate::ast::Expr::Ident(name))) = value_prop {
                                    self.input_fields.entry(variant).or_default().push(name.to_string());
                                }
                            }
                            "oncursor" | "onCursor" => {
                                let variant = self.extract_variant_name(&handler.handler);
                                let msg_name = self.current_msg_name();
                                builder = format!("{}.on_cursor({}::{})", builder, msg_name, variant);
                            }
                            "oncontextmenu" | "onContextMenu" => {
                                let variant = self.extract_variant_name(&handler.handler);
                                let msg_name = self.current_msg_name();
                                builder = format!("{}.on_context_menu({}::{})", builder, msg_name, variant);
                            }
                            _ => {}
                        }
                    }

                    return format!("{}.build()", builder);
                }
                // For leaf tags (text, button) with a "text" prop, use it as the initial value.
                // For buttons: View::button("-") instead of View::button(())
                // For text with state ref: View::text(format!("{}", self.name))
                // Also extract text content from a single Text child (e.g. text f"..." { class: "..." })
                let child_text_content: Option<AuraTextContent> = if children.len() == 1 {
                    if let AuraNode::Text(content) = &children[0] {
                        Some(content.clone())
                    } else {
                        None
                    }
                } else {
                    None
                };

                let text_prop = props.get("text")
                    .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v { Some(s.to_string()) } else { None })
                    .or_else(|| {
                        // Fallback: extract literal text from child Text node
                        match &child_text_content {
                            Some(AuraTextContent::Literal(s)) => Some(s.clone()),
                            Some(AuraTextContent::Interpolated { template, .. }) => Some(template.clone()),
                            None => None,
                        }
                    });

                // Check if text prop is a state reference (text .name)
                let text_state_ref = props.get("text")
                    .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Ident(name)) = v { Some(name.clone()) } else { None });

                // Generate a Rust expression string for the text prop, handling ALL AuraExpr types.
                // This catches FieldAccess (note.title), Index, and other dynamic expressions
                // that fall through the Literal/StateRef checks above.
                let text_rust_expr: Option<String> = if text_prop.is_some() || text_state_ref.is_some() {
                    None // Already handled by text_prop or text_state_ref
                } else {
                    props.get("text").and_then(|v| {
                        if let AuraPropValue::Expr(expr) = v {
                            Some(self.ast_expr_to_rust(expr))
                        } else {
                            None
                        }
                    })
                };

                // PLAN-036 T-02：button icon 词汇臂（真编译门首漏——词汇表
                // 超记、实臂缺位；variant/size 已由 with_button_preset 消费，
                // 本块只管 icon）。对齐解释臂 convert_button：icon 经 PUA
                // 标记嵌 label（`\u{EE01}{icon}\u{EE02}{text}`——renderer 同
                // 协议双轨同形，Plan 409 §10 组 A 同款）。
                if tag == "button" && props.contains_key("icon") {
                    let icon_lit = props.get("icon").and_then(|v| match v {
                        AuraPropValue::Expr(crate::ast::Expr::Str(s)) => Some(s.to_string()),
                        _ => None,
                    });
                    // label：text 三通道（字面量/状态引用/动态式）→ 子件折叠
                    // → 空；icon 在场时 PUA 前缀包裹。
                    let plain_label: Option<String> = if let Some(t) = &text_prop {
                        if t.contains("${") {
                            Some(self.interpolate_str(t))
                        } else {
                            Some(format!("\"{}\"", t))
                        }
                    } else if let Some(ref name) = text_state_ref {
                        let name_ref = if self.is_loop_var(name) {
                            name.to_string()
                        } else {
                            format!("self.{}", name)
                        };
                        Some(format!("format!(\"{{}}\", {})", name_ref))
                    } else if let Some(ref e) = text_rust_expr {
                        Some(e.clone())
                    } else if !children.is_empty() {
                        Some(self.collect_button_label(children))
                    } else {
                        None
                    };
                    let label_expr = match (&icon_lit, &plain_label) {
                        (Some(icon), Some(tp)) => {
                            format!("format!(\"\\u{{EE01}}{}\\u{{EE02}}{{}}\", {})", icon, tp)
                        }
                        (Some(icon), None) => {
                            format!("\"\\u{{EE01}}{}\\u{{EE02}}\".to_string()", icon)
                        }
                        (None, Some(tp)) => tp.clone(),
                        (None, None) => "\"\".to_string()".to_string(),
                    };
                    let mut builder = format!("View::button({})", label_expr);
                    for (key, value) in props_sorted.clone() {
                        if matches!(key.as_str(), "icon" | "text") {
                            continue;
                        }
                        builder = self.add_prop_to_builder(&builder, key, value);
                    }
                    for (event, handler) in events_sorted.clone() {
                        builder = self.add_event_to_builder(&builder, event, handler);
                    }
                    if !events.iter().any(|(e, _)| e == "onclick" || e == "onClick") {
                        builder = format!("{}.on_click(|_| ())", builder);
                    }
                    return self.wrap_button_double_click(format!("{}.build()", builder), dbl_event);
                }

                // Handle image element — generate View::image() or View::image_styled()
                // PLAN-026 T-07：src 绑定形状容差（025 value 绑定同款——
                // Ident '.' 前缀 / Dot("."|"self", f)，build_rust_ui 提取
                // 路径产 Dot 形；缺臂曾致 004 真源 src 静默丢失——AC-05
                // 非静默丢关键 prop 纪律）。
                if tag == "image" {
                    let src_field = props.get("src").and_then(|v| match v {
                        AuraPropValue::Expr(crate::ast::Expr::Ident(name)) => {
                            let f = name.as_str().trim_start_matches('.');
                            (!f.is_empty()).then(|| f.to_string())
                        }
                        AuraPropValue::Expr(crate::ast::Expr::Dot(obj, field)) => {
                            match obj.as_ref() {
                                crate::ast::Expr::Ident(base)
                                    if base.as_str() == "." || base.as_str() == "self" =>
                                {
                                    Some(field.as_str().to_string())
                                }
                                _ => None,
                            }
                        }
                        _ => None,
                    });
                    let src = match &src_field {
                        Some(f) => format!("format!(\"{{}}\", self.{f})"),
                        None => props
                            .get("src")
                            .and_then(|v| {
                                if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v {
                                    Some(format!("\"{}\"", s))
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_else(|| "\"\"".to_string()),
                    };
                    let style_str = props.get("style")
                        .or_else(|| props.get("class"))
                        .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v { Some(s.to_string()) } else { None })
                        .unwrap_or_default();
                    if style_str.is_empty() {
                        return format!("View::image({})", src);
                    } else {
                        return format!("View::image_styled({}, \"{}\")", src, style_str);
                    }
                }

                // Plan 547 Task 22: ImageSurface is emitted through the
                // backend-neutral constructor so literal, state-ref and
                // conditional expressions all remain live in generated Rust.
                // Runtime event payloads are appended by the Iced surface;
                // the typed message variant and declared Aura arguments are
                // captured here as Option<M> values.
                if matches!(tag.as_str(), "imagesurface" | "image-surface" | "image_surface" | "ImageSurface") {
                    let expr_for = |key: &str, default: &str| -> String {
                        props
                            .get(key)
                            .and_then(|v| match v {
                                AuraPropValue::Expr(expr) => Some(self.ast_expr_to_rust(expr)),
                                AuraPropValue::StyleBinding(_) => None,
                            })
                            .unwrap_or_else(|| default.to_string())
                    };
                    let owned_expr = |expr: String| -> String {
                        if expr.starts_with("self.") {
                            format!("{}.clone()", expr)
                        } else {
                            expr
                        }
                    };
                    let src = owned_expr(expr_for("src", "\"\".to_string()"));
                    let alt = owned_expr(expr_for("alt", "\"\".to_string()"));
                    let width = format!("({}) as u32", expr_for("width", "0"));
                    let height = format!("({}) as u32", expr_for("height", "0"));
                    let quality = format!("({}).clamp(0, 100) as u8", expr_for("quality", "90"));
                    let fit = owned_expr(expr_for("fit", "\"contain\".to_string()"));
                    let zoom = format!("({}) as f32", expr_for("zoom", "1.0"));
                    let offset_x = format!("({}) as f32", expr_for("offset_x", "0.0"));
                    let offset_y = format!("({}) as f32", expr_for("offset_y", "0.0"));
                    let rotation = format!("({}) as i32", expr_for("rotation", "0"));
                    let filter = owned_expr(expr_for("filter", "\"high\".to_string()"));
                    let event_expr = |names: &[&str]| -> String {
                        events
                            .iter()
                            .find(|(name, _)| names.iter().any(|candidate| *candidate == name.as_str()))
                            .map(|(_, event)| {
                                format!(
                                    "Some({})",
                                    self.handler_to_rust_direct_msg(&event.handler, &event.params)
                                )
                            })
                            .unwrap_or_else(|| "None".to_string())
                    };
                    let mut surface = format!(
                        "View::image_surface({src}).image_surface_props({alt}, {width}, {height}, {quality}, {fit}, {zoom}, {offset_x}, {offset_y}, {rotation}, {filter})",
                    );
                    surface = format!(
                        "{}.image_surface_events({}, {}, {}, {}, {})",
                        surface,
                        event_expr(&["onerror", "on_error", "error"]),
                        event_expr(&["onload", "onloaded", "on_loaded", "loaded"]),
                        event_expr(&["onwheel", "wheel"]),
                        event_expr(&["onpan", "pan"]),
                        event_expr(&["ondblclick", "dblclick", "doubleclick"]),
                    );
                    if let Some(style) = props
                        .get("style")
                        .or_else(|| props.get("class"))
                        .and_then(|v| match v {
                            AuraPropValue::Expr(crate::ast::Expr::Str(s)) => Some(s.to_string()),
                            _ => None,
                        })
                        .filter(|s| !s.is_empty())
                    {
                        surface = format!("{}.image_surface_style(\"{}\")", surface, style);
                    }
                    return surface;
                }

                // PLAN-026 T-02（§5.1 D1/D1' 定案）：display 族 a2r 降级臂
                // ——对齐 VM 轨 AuraViewBuilder 既有降级形态（零 View 变体；
                // icon = convert_image_or_icon :6455 的 lucide 承载 + 尺寸
                // 契约；divider/spacer/avatar = convert_divider :6885 /
                // convert_spacer :6813 / convert_avatar :8559 同型）。
                // 降级纪律：label/src/progress 等关键 prop 不静默丢失
                // （字面量必达；动态求值面逐臂随注）。
                let user_style_str = |props: &std::collections::HashMap<String, AuraPropValue>| -> String {
                    props
                        .get("style")
                        .or_else(|| props.get("class"))
                        .and_then(|v| {
                            if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v {
                                Some(s.to_string())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default()
                };

                // PLAN-027 T-03: 宿主合成件槽位 —— window_thumbnail /
                // workspace_preview codegen 直发**既有** View 变体（定案
                // 记录 D4/修正 B：变体已在册 view.rs:878/:890，iced 消费
                // renderer.rs:5146/:5230 与解释轨同一 into_iced 面——双形态
                // 快照渲染臂/miss request_capture/fallback 语义同源）。
                // 解释侧构造同构 = aura_view_builder convert_window_
                // thumbnail :6709 / convert_workspace_preview :6725：
                // key prop（wid / ws）字面量或动态表达式；fallback 档
                // 缺省 app-window；style 直传（None = 缺省）。
                if tag == "window_thumbnail" || tag == "workspace_preview" {
                    let key_prop = if tag == "window_thumbnail" { "wid" } else { "ws" };
                    let key_expr = match props.get(key_prop) {
                        Some(AuraPropValue::Expr(crate::ast::Expr::Str(s))) => {
                            format!("\"{s}\".to_string()")
                        }
                        Some(AuraPropValue::Expr(e)) => {
                            let e = self.ast_expr_to_rust(e);
                            // PLAN-036 T-02：数值类 key（Value 访问 `as i32)`
                            // 收尾——View 键面为 String，降串（VM 宽松串化
                            // 等价，.at `ws: .ws.id` 数字源）。
                            let e = if e.ends_with("as i32)") {
                                format!("{}.to_string()", e)
                            } else if e.starts_with("self.") {
                                format!("{e}.clone()")
                            } else {
                                e
                            };
                            e
                        }
                        _ => "String::new()".to_string(),
                    };
                    let fb_prop = if tag == "window_thumbnail" { "fallback_icon" } else { "fallback" };
                    let fallback_expr = match props.get(fb_prop) {
                        Some(AuraPropValue::Expr(crate::ast::Expr::Str(s))) => {
                            format!("\"{s}\".to_string()")
                        }
                        Some(AuraPropValue::Expr(e)) => {
                            let e = self.ast_expr_to_rust(e);
                            if e.starts_with("self.") {
                                format!("{e}.clone()")
                            } else {
                                e
                            }
                        }
                        _ => "\"app-window\".to_string()".to_string(),
                    };
                    let user_style = user_style_str(props);
                    let style_expr = if user_style.is_empty() {
                        "None".to_string()
                    } else {
                        format!("auto_lang::ui::style::Style::parse(\"{}\").ok()", user_style)
                    };
                    if tag == "window_thumbnail" {
                        return format!(
                            "View::WindowThumbnail {{ wid: {key_expr}, fallback_icon: {fallback_expr}, style: {style_expr} }}"
                        );
                    }
                    return format!(
                        "View::WorkspacePreview {{ ws: {key_expr}, fallback_icon: {fallback_expr}, style: {style_expr} }}"
                    );
                }

                // icon → View::image("lucide:{name}")（VM 同型：PLAN-018
                // 前缀 iconfile:/hicon:/lucide: 透传，裸名补 lucide:）；
                // 尺寸契约 = 显式 w-/h- 类 > size prop（精确 px，任意值
                // w-[Npx] 通道）> 默认 20px（VM DEFAULT_ICON_PX）。
                if tag == "icon" {
                    let src = match props.get("name") {
                        Some(AuraPropValue::Expr(crate::ast::Expr::Str(s))) => {
                            if s.starts_with("iconfile:")
                                || s.starts_with("hicon:")
                                || s.starts_with("lucide:")
                            {
                                format!("\"{s}\".to_string()")
                            } else {
                                format!("\"lucide:{s}\".to_string()")
                            }
                        }
                        Some(AuraPropValue::Expr(expr)) => {
                            let e = self.ast_expr_to_rust(expr);
                            format!(
                                "{{ let n = format!(\"{{}}\", {e}); \
                                 if n.starts_with(\"iconfile:\") || n.starts_with(\"hicon:\") \
                                 || n.starts_with(\"lucide:\") {{ n }} \
                                 else {{ format!(\"lucide:{{}}\", n) }} }}"
                            )
                        }
                        _ => "\"\".to_string()".to_string(),
                    };
                    // PLAN-039 T-02②（§5.1 定案记录②）：动态 class 臂——
                    // class/style 为 Dot/点链 Ident 表达式（CardSuit
                    // `class: .style` 组件 prop 引用）此前静默丢样式
                    // （user_style_str 只收字面量），升运行期拼串求值；
                    // w-/h- 缺省档逻辑与字面量臂同款运行期复刻（动态
                    // size 求值维持 not-yet 缺省档——下方既有注记口径）。
                    let dyn_style: Option<String> = props
                        .get("style")
                        .or_else(|| props.get("class"))
                        .and_then(|v| match v {
                            AuraPropValue::Expr(crate::ast::Expr::Str(_)) => None,
                            AuraPropValue::Expr(e @ crate::ast::Expr::Dot(..)) => Some(
                                self.ast_expr_to_rust(e).replace("self..", "self."),
                            ),
                            AuraPropValue::Expr(crate::ast::Expr::Ident(name))
                                if name.as_str().starts_with('.') => Some(
                                self.ast_expr_to_rust(&crate::ast::Expr::Ident(name.clone()))
                                    .replace("self..", "self."),
                            ),
                            _ => None,
                        });
                    if let Some(dyn_expr) = dyn_style {
                        let px: f32 = props.get("size").and_then(|v| match v {
                            AuraPropValue::Expr(crate::ast::Expr::Int(n)) => Some(*n as f32),
                            AuraPropValue::Expr(crate::ast::Expr::Float(f, _)) => Some(*f as f32),
                            _ => None,
                        })
                        .filter(|v| *v > 0.0)
                        .unwrap_or(20.0);
                        return format!(
                            "{{ let __c = format!(\"{{}}\", {dyn_expr}); \
                             let mut __s = __c.clone(); \
                             if !__c.split_whitespace().any(|t| t.starts_with(\"w-\")) {{ __s.push_str(\" w-[{px}px]\"); }} \
                             if !__c.split_whitespace().any(|t| t.starts_with(\"h-\")) {{ __s.push_str(\" h-[{px}px]\"); }} \
                             View::image_styled({src}, &__s) }}"
                        );
                    }
                    let mut classes = user_style_str(props);
                    let has_w = classes.split_whitespace().any(|t| t.starts_with("w-"));
                    let has_h = classes.split_whitespace().any(|t| t.starts_with("h-"));
                    if !has_w || !has_h {
                        let px: Option<f32> = props.get("size").and_then(|v| match v {
                            AuraPropValue::Expr(crate::ast::Expr::Int(n)) => Some(*n as f32),
                            AuraPropValue::Expr(crate::ast::Expr::Float(f, _)) => Some(*f as f32),
                            _ => None,
                        })
                        .filter(|v| *v > 0.0);
                        // 动态 size 求值 not-yet（VM 走 bindings 求值；a2r
                        // 静态发射面暂只认字面量）——缺省档兜底。
                        let d = px.unwrap_or(20.0);
                        if !has_w {
                            classes.push_str(&format!(" w-[{d}px]"));
                        }
                        if !has_h {
                            classes.push_str(&format!(" h-[{d}px]"));
                        }
                    }
                    return format!("View::image_styled({src}, \"{classes}\")");
                }

                // divider/separator/hr → 1px 线容器（VM convert_divider
                // 同型底档 h-1 bg-gray-200；direction/orientation=vertical
                // 竖档——解释态 layout_divider client_runtime.rs:1474 同款
                // 双键）。separator label prop：解释态臂同样不载（同口径
                // 丢弃随注，非静默降级）。
                if matches!(tag.as_str(), "divider" | "separator" | "hr") {
                    let user_style = user_style_str(props);
                    let vertical = ["direction", "orientation"]
                        .iter()
                        .find_map(|k| props.get(*k))
                        .and_then(|v| match v {
                            AuraPropValue::Expr(crate::ast::Expr::Str(s)) => {
                                Some(s.eq_ignore_ascii_case("vertical"))
                            }
                            _ => None,
                        })
                        .unwrap_or(false);
                    let base = if tag == "separator" {
                        if vertical { "w-px h-6 bg-border" } else { "w-full h-px bg-border" }
                    } else if vertical {
                        "w-px h-6 bg-gray-200"
                    } else {
                        "w-full h-1 bg-gray-200"
                    };
                    if user_style.is_empty() {
                        return format!("View::container(View::Empty).style(\"{base}\").build()");
                    }
                    return format!(
                        "View::container(View::Empty).style(\"{base} {user_style}\").build()"
                    );
                }

                // spacer → 占位容器（VM convert_spacer 同型：无样式 =
                // flex-1 吃剩余主轴；显式 style 保真）。
                if tag == "spacer" {
                    let user_style = user_style_str(props);
                    if user_style.is_empty() {
                        return "View::container(View::Empty).style(\"flex-1\").build()".to_string();
                    }
                    return format!(
                        "View::container(View::Empty).style(\"{user_style}\").build()"
                    );
                }

                // avatar → 占位容器（VM convert_avatar 同型：缺省
                // w-10 h-10 bg-gray-300 rounded-full；子件组合；fallback
                // 字面量 → 首字母 text 子级——解释态 layout_avatar
                // client_runtime.rs:1415 占位口径；src 位图内容归图像
                // 通道独立线，占位随注）。
                if tag == "avatar" {
                    let user_style = user_style_str(props);
                    let base = if user_style.is_empty() {
                        "w-10 h-10 bg-gray-300 rounded-full".to_string()
                    } else {
                        user_style
                    };
                    let child_view = if children.is_empty() {
                        let initials: String = props
                            .get("fallback")
                            .and_then(|v| match v {
                                AuraPropValue::Expr(crate::ast::Expr::Str(s)) => Some(s.clone()),
                                _ => None,
                            })
                            .unwrap_or_default()
                            .split_whitespace()
                            .filter_map(|w| w.chars().next())
                            .take(2)
                            .collect::<String>()
                            .to_uppercase();
                        if initials.is_empty() {
                            "View::Empty".to_string()
                        } else {
                            format!("View::text(\"{initials}\")")
                        }
                    } else if children.len() == 1 {
                        self.generate_view_tree(&children[0])
                    } else {
                        let mut col = "View::col()".to_string();
                        for child in children {
                            col = format!("{}.child({})", col, self.generate_view_tree(child));
                        }
                        format!("{col}.build()")
                    };
                    return format!("View::container({child_view}).style(\"{base}\").build()");
                }

                // badge → 样式化 Row（VM convert_badge :9157 同型：shadcn
                // 基类 + variant 预设 + user 类；label = text prop/子件，
                // 动态求值经 interpolate/format 通道必达）。
                if tag == "badge" {
                    let variant = props
                        .get("variant")
                        .and_then(|v| match v {
                            AuraPropValue::Expr(crate::ast::Expr::Str(s)) => Some(s.clone()),
                            _ => None,
                        })
                        .unwrap_or_default();
                    let preset = match variant.as_str() {
                        "outline" => "border border-input text-foreground",
                        "secondary" => "bg-secondary text-secondary-foreground",
                        "destructive" => "bg-destructive text-destructive-foreground",
                        _ => "bg-primary text-primary-foreground",
                    };
                    let base = "items-center gap-1.5 rounded-full px-2.5 py-0.5 text-xs font-medium";
                    let user_style = user_style_str(props);
                    let merged = if user_style.is_empty() {
                        format!("{base} {preset}")
                    } else {
                        format!("{base} {preset} {user_style}")
                    };
                    let child_view = if children.is_empty() {
                        if let Some(ref name) = text_state_ref {
                            let name_ref = if self.is_loop_var(name) {
                                name.to_string()
                            } else {
                                format!("self.{}", name)
                            };
                            format!("View::text(format!(\"{{}}\", {name_ref}))")
                        } else if let Some(label) = &text_prop {
                            if label.contains("${") {
                                format!("View::text({})", self.interpolate_str(label))
                            } else {
                                format!("View::text(\"{label}\")")
                            }
                        } else if let Some(text) = &text_rust_expr {
                            let text = if text.starts_with("self.") {
                                format!("format!(\"{{}}\", {text})")
                            } else {
                                text.clone()
                            };
                            format!("View::text({text})")
                        } else {
                            String::new()
                        }
                    } else if children.len() == 1 {
                        self.generate_view_tree(&children[0])
                    } else {
                        let mut col = "View::col()".to_string();
                        for child in children {
                            col = format!("{}.child({})", col, self.generate_view_tree(child));
                        }
                        format!("{col}.build()")
                    };
                    if child_view.is_empty() {
                        return format!("View::row().style(\"{merged}\").build()");
                    }
                    return format!("View::row().style(\"{merged}\").child({child_view}).build()");
                }

                // card → styled 容器（§5.1 D1：container 降级；语义 card
                // 族表面档由作者 style 声明——VM 轨同无缺省注入）。
                if tag == "card" {
                    let user_style = user_style_str(props);
                    let child_view = if children.is_empty() {
                        "View::Empty".to_string()
                    } else if children.len() == 1 {
                        self.generate_view_tree(&children[0])
                    } else {
                        let mut col = "View::col()".to_string();
                        for child in children {
                            col = format!("{}.child({})", col, self.generate_view_tree(child));
                        }
                        format!("{col}.build()")
                    };
                    if user_style.is_empty() {
                        return format!("View::container({child_view}).build()");
                    }
                    return format!("View::container({child_view}).style(\"{user_style}\").build()");
                }

                // scroll → View::scrollable（既有构造器；style 透传）。
                if tag == "scroll" || tag == "scrollable" {
                    let user_style = user_style_str(props);
                    let child_view = if children.is_empty() {
                        "View::Empty".to_string()
                    } else if children.len() == 1 {
                        self.generate_view_tree(&children[0])
                    } else {
                        let mut col = "View::col()".to_string();
                        for child in children {
                            col = format!("{}.child({})", col, self.generate_view_tree(child));
                        }
                        format!("{col}.build()")
                    };
                    if user_style.is_empty() {
                        return format!("View::scrollable({child_view}).build()");
                    }
                    return format!(
                        "View::scrollable({child_view}).style(\"{user_style}\").build()"
                    );
                }

                // a（Element 形态）→ styled text（AuraNode::Link 臂同口径；
                // href 语义归 shell a2r S1，label 必达）。
                if tag == "a" {
                    let user_style = user_style_str(props);
                    let style_str = if user_style.is_empty() {
                        "text-blue-600 underline cursor-pointer".to_string()
                    } else {
                        user_style
                    };
                    if let Some(ref name) = text_state_ref {
                        let name_ref = if self.is_loop_var(name) {
                            name.to_string()
                        } else {
                            format!("self.{}", name)
                        };
                        return format!("View::text_styled(format!(\"{{}}\", {name_ref}), \"{style_str}\")");
                    }
                    if let Some(label) = &text_prop {
                        if label.contains("${") {
                            return format!("View::text_styled({}, \"{style_str}\")", self.interpolate_str(label));
                        }
                        return format!("View::text_styled(\"{label}\".to_string(), \"{style_str}\")");
                    }
                    if let Some(text) = &text_rust_expr {
                        let text = if text.starts_with("self.") {
                            format!("format!(\"{{}}\", {text})")
                        } else {
                            text.clone()
                        };
                        return format!("View::text_styled({text}, \"{style_str}\")");
                    }
                    return format!("View::text_styled(\"\", \"{style_str}\")");
                }

                // Handle progress — View::progress_bar(value / max)
                if tag == "progress" {
                    let value_expr = if let Some(AuraPropValue::Expr(crate::ast::Expr::Ident(name))) = props.get("value") {
                        format!("self.{}", name)
                    } else if let Some(AuraPropValue::Expr(crate::ast::Expr::Str(s))) = props.get("value") {
                        s.to_string()
                    } else {
                        "0".to_string()
                    };
                    let max_val = if let Some(AuraPropValue::Expr(crate::ast::Expr::Str(s))) = props.get("max") {
                        s.to_string()
                    } else {
                        "100".to_string()
                    };
                    let style_str = props.get("style")
                        .or_else(|| props.get("class"))
                        .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v { Some(s.to_string()) } else { None })
                        .unwrap_or_default();
                    if style_str.is_empty() {
                        return format!("View::progress_bar({} as f32 / {} as f32)", value_expr, max_val);
                    } else {
                        return format!("View::progress_bar_styled({} as f32 / {} as f32, \"{}\")", value_expr, max_val, style_str);
                    }
                }

                // Special handling for checkbox — View::Checkbox { ... } direct construction
                // View::checkbox() returns View<M> (enum), not a builder, so we can't chain
                // .style() / .on_click() / .build(). Use direct struct literal instead.
                if tag == "checkbox" {
                    let is_checked = props.get("checked")
                        .or_else(|| props.get("is_checked"))
                        .map(|v| match v {
                            AuraPropValue::Expr(crate::ast::Expr::Bool(b)) => b.to_string(),
                            AuraPropValue::Expr(crate::ast::Expr::Ident(name)) => format!("self.{}", name),
                            AuraPropValue::Expr(crate::ast::Expr::Dot(object, field)) => {
                                let field = field.clone();
                                let obj_str = match object.as_ref() {
                                    crate::ast::Expr::Ident(name) => {
                                        let resolved = if name.starts_with('.') { &name[1..] } else { name.as_str() };
                                        if self.is_loop_var(resolved) && self.value_loop_vars.contains(resolved) {
                                            resolved.to_string()
                                        } else {
                                            format!("self.{}", resolved)
                                        }
                                    }
                                    _ => format!("{:?}", object),
                                };
                                self.value_field_access(&obj_str, field.as_str())
                            }
                            // PLAN-035 T-11：一般表达式（如 `.__wm_dashboard
                            // == "1"` 比较）走既有 a2r 表达式渲染器——此前
                            // 未知形态静默 false（勾选态恒错的根因）。
                            AuraPropValue::Expr(other) => self
                                .ast_expr_to_rust_with_value_params(other, &[]),
                            _ => "false".to_string(),
                        })
                        .unwrap_or_else(|| "false".to_string());
                    let label = props.get("label")
                        .or_else(|| props.get("text"))
                        .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v { Some(s.to_string()) } else { None })
                        .unwrap_or_default();

                    // Parse class/style into Style
                    let class_str = props.get("class")
                        .or_else(|| props.get("style"))
                        .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v { Some(s.to_string()) } else { None })
                        .unwrap_or_default();
                    let style_expr = if class_str.is_empty() {
                        "None".to_string()
                    } else {
                        format!("Some(auto_lang::ui::style::Style::parse(\"{}\").unwrap())", class_str)
                    };

                    // Build on_toggle handler
                    // NOTE: Checkbox.on_toggle is Option<M>, NOT a closure.
                    // Must emit the message value directly, e.g. Some(AppMsg::ToggleTodo(42)),
                    // not Some(|_| AppMsg::ToggleTodo(42)).
                    let on_toggle = events.iter()
                        .find(|(e, _)| e.as_str() == "onclick" || e.as_str() == "onClick" || e.as_str() == "on_click")
                        .map(|(_, handler)| {
                            self.handler_to_rust_direct_msg(&handler.handler, &handler.params)
                        });

                    let result = match on_toggle {
                        Some(msg) => format!(
                            "View::Checkbox {{ is_checked: {}, label: \"{}\".to_string(), on_toggle: Some({}), style: {} }}",
                            is_checked, label, msg, style_expr
                        ),
                        None => format!(
                            "View::Checkbox {{ is_checked: {}, label: \"{}\".to_string(), on_toggle: None, style: {} }}",
                            is_checked, label, style_expr
                        ),
                    };
                    return result;
                }

                // PLAN-025 T-03（PLAN-661 T-02 形态迁移）: slider —
                // View::slider(min..=max, value) + .on_change(闭包)。载荷
                // 回写 = f32 载荷变体的构造器闭包（SliderChangeHandler newtype
                // 包装——可跨消息类型映射）；on() 侧载荷臂由 msg 声明 + on 块
                // 模式（.SetVol(v float) -> {...}）既有机制承担。
                if tag == "slider" {
                    let numeric = |v: Option<&AuraPropValue>, default: f64| -> String {
                        // f32 实参拒收整数字面量（Rust 整型字面量不向浮点
                        // 收敛）——恒带小数点输出。
                        match v {
                            Some(AuraPropValue::Expr(crate::ast::Expr::Float(f, _))) => format!("{f:.1}"),
                            Some(AuraPropValue::Expr(crate::ast::Expr::Double(f, _))) => format!("{f:.1}"),
                            Some(AuraPropValue::Expr(crate::ast::Expr::Int(i))) => format!("{i}.0"),
                            Some(AuraPropValue::Expr(crate::ast::Expr::Str(s))) => s.to_string(),
                            _ => format!("{default:.1}"),
                        }
                    };
                    let min = numeric(props.get("min"), 0.0);
                    let max = numeric(props.get("max"), 100.0);
                    // value 绑定：Ident → self.<field>（f64 字段补 as f32
                    // ——View::slider 载荷恒 f32）；字面量直用。
                    let value_expr = match props.get("value") {
                        Some(AuraPropValue::Expr(crate::ast::Expr::Ident(name))) => {
                            if self.state_types.get(name.as_str()).map(|s| s.as_str()) == Some("f64") {
                                format!("self.{name} as f32")
                            } else {
                                format!("self.{name}")
                            }
                        }
                        _ => numeric(props.get("value"), 0.0),
                    };
                    let mut builder = format!("View::slider({min}..={max}, {value_expr})");
                    // onchange → 载荷变体构造器闭包（SliderChangeHandler
                    // 由 builder .on_change 包装）。
                    if let Some((_, handler)) = events
                        .iter()
                        .find(|(e, _)| matches!(e.as_str(), "onchange" | "onChange"))
                    {
                        let variant = self.extract_variant_name(&handler.handler);
                        let msg_name = self.current_msg_name();
                        builder = format!(
                            "{builder}\n                .on_change(|v| {msg_name}::{variant}(v))"
                        );
                    }
                    // 无 onchange：不挂 .on_change（None = 无动作面，语义
                    // 等价且更净——旧占位零参闭合退役）。
                    if let Some(st) = props.get("step") {
                        builder = format!("{builder}.step({})", numeric(Some(st), 0.0));
                    }
                    for (key, value) in props_sorted.clone() {
                        if key == "min" || key == "max" || key == "value" || key == "step" { continue; }
                        builder = self.add_prop_to_builder(&builder, key, value);
                    }
                    return format!("{builder}.build()");
                }

                // PLAN-025 T-04: select — View::select(options) +
                // .selected(i) + .on_choose(|idx, val| Msg::Variant(idx,
                // val.to_string()))。on_select = SelectCallback（Arc<dyn
                // Fn(usize,&str)->M>），闭包内物化载荷消息——零
                // thread-local；on() 侧载荷臂由 msg 声明 + on 块模式既有
                // 机制承担。
                if tag == "select" {
                    let options_expr = match props.get("options") {
                        Some(AuraPropValue::Expr(crate::ast::Expr::Array(items))) => {
                            let elems: Vec<String> = items
                                .iter()
                                .filter_map(|e| match e {
                                    crate::ast::Expr::Str(s) => Some(format!("\"{s}\".to_string()")),
                                    _ => None,
                                })
                                .collect();
                            format!("vec![{}]", elems.join(", "))
                        }
                        _ => "Vec::new()".to_string(),
                    };
                    let mut builder = format!("View::select({options_expr})");
                    if let Some(AuraPropValue::Expr(crate::ast::Expr::Int(i))) = props.get("selected") {
                        builder = format!("{builder}.selected({i})");
                    }
                    if let Some((_, handler)) = events
                        .iter()
                        .find(|(e, _)| matches!(e.as_str(), "onchange" | "onChange"))
                    {
                        let variant = self.extract_variant_name(&handler.handler);
                        let msg_name = self.current_msg_name();
                        // 闭包形态随变体载荷数自适应：单 str 载荷（值绑定
                        // 常态）忽略 idx；双载荷取 idx as i32（int 载荷）。
                        let closure = match self
                            .message_variants
                            .iter()
                            .find(|v| v.name == variant)
                            .map(|v| v.payload.len())
                            .unwrap_or(0)
                        {
                            1 => format!("|_idx: usize, val: &str| {msg_name}::{variant}(val.to_string())"),
                            2 => format!("|idx: usize, val: &str| {msg_name}::{variant}(idx as i32, val.to_string())"),
                            _ => format!("|_idx: usize, _val: &str| {msg_name}::{variant}()"),
                        };
                        builder = format!("{builder}.on_choose({closure})");
                    }
                    for (key, value) in props_sorted.clone() {
                        if key == "options" || key == "selected" { continue; }
                        builder = self.add_prop_to_builder(&builder, key, value);
                    }
                    // View::select 直返 View（链式 self）——无 .build()。
                    return builder;
                }

                // PLAN-032 T-03（D2）：tabs 复合组件——a2r 断裂映射修复
                //（tag_to_view_fn "tabs"→View::tabs() 缺 Vec<String> 实参
                // / "tab"→View::tab() 不存在，046 a2r 轨不可编译——§1.8
                // 在册半句清偿）。折叠契约镜像 VM 轨 convert_tabs
                //（aura_view_builder.rs:1410）：tabslist 透明折叠 /
                // tabstrigger 标签+value / tabscontent(value+子件) / tab
                // 平铺；selected 解析 active（索引）→ value（按值匹配）
                // → defaultvalue/default → 0；onselect 首参 = value 串
                //（闭包物化载荷消息——select 臂先例）。M7-c②（jade-
                // garden/auto-musk tab 面）依赖解锁。
                if tag == "tabs" {
                    fn tag_lc(tag: &str) -> String {
                        tag.to_ascii_lowercase().replace('_', "-")
                    }
                    // 触发器/内容/平铺折叠（tabslist 透明）。
                    fn fold_tabs<'a>(
                        node: &'a crate::aura::AuraNode,
                        triggers: &mut Vec<&'a crate::aura::AuraNode>,
                        contents: &mut Vec<&'a crate::aura::AuraNode>,
                        flats: &mut Vec<&'a crate::aura::AuraNode>,
                    ) {
                        let crate::aura::AuraNode::Element { tag, children, .. } = node else {
                            return;
                        };
                        match tag_lc(tag).as_str() {
                            "tab" => flats.push(node),
                            "tabs-trigger" | "tabstrigger" => triggers.push(node),
                            "tabs-content" | "tabscontent" => contents.push(node),
                            "tabs-list" | "tabslist" | "tabrow" | "tab-row" => {
                                for c in children {
                                    fold_tabs(c, triggers, contents, flats);
                                }
                            }
                            _ => {}
                        }
                    }
                    let mut triggers: Vec<&crate::aura::AuraNode> = Vec::new();
                    let mut content_nodes: Vec<&crate::aura::AuraNode> = Vec::new();
                    let mut flats: Vec<&crate::aura::AuraNode> = Vec::new();
                    for c in children {
                        fold_tabs(c, &mut triggers, &mut content_nodes, &mut flats);
                    }
                    // 标签/值：prop text/label → 直接 Text 子件 → text-like
                    // 元素子件（convert_tabs 同序）；缺省 "Tab N"/索引串。
                    fn node_text(node: &crate::aura::AuraNode) -> Option<String> {
                        let crate::aura::AuraNode::Element { props, children, .. } = node else {
                            return None;
                        };
                        for key in ["text", "label"] {
                            if let Some(AuraPropValue::Expr(crate::ast::Expr::Str(s))) = props.get(key) {
                                return Some(s.to_string());
                            }
                        }
                        if let Some(crate::aura::AuraNode::Text(crate::aura::AuraTextContent::Literal(s))) =
                            children.iter().next()
                        {
                            return Some(s.clone());
                        }
                        children.iter().find_map(|c| {
                            let crate::aura::AuraNode::Element { tag, props, children, .. } = c else {
                                return None;
                            };
                            if !matches!(tag.as_str(), "text" | "label" | "span" | "p" | "h1" | "h2" | "h3") {
                                return None;
                            }
                            if let Some(AuraPropValue::Expr(crate::ast::Expr::Str(s))) = props.get("text") {
                                return Some(s.to_string());
                            }
                            children.iter().find_map(|d| match d {
                                crate::aura::AuraNode::Text(crate::aura::AuraTextContent::Literal(s)) => {
                                    Some(s.clone())
                                }
                                _ => None,
                            })
                        })
                    }
                    let use_flat = !flats.is_empty();
                    let mut labels: Vec<String> = Vec::new();
                    let mut values: Vec<String> = Vec::new();
                    if use_flat {
                        for (i, t) in flats.iter().enumerate() {
                            let label = node_text(t).unwrap_or_else(|| format!("Tab {}", i + 1));
                            let value = match t {
                                crate::aura::AuraNode::Element { props, .. } => match props.get("value") {
                                    Some(AuraPropValue::Expr(crate::ast::Expr::Str(s))) => s.to_string(),
                                    _ => i.to_string(),
                                },
                                _ => i.to_string(),
                            };
                            labels.push(label);
                            values.push(value);
                        }
                    } else {
                        for (i, t) in triggers.iter().enumerate() {
                            let label = node_text(t).unwrap_or_else(|| format!("Tab {}", i + 1));
                            let value = match t {
                                crate::aura::AuraNode::Element { props, .. } => match props.get("value") {
                                    Some(AuraPropValue::Expr(crate::ast::Expr::Str(s))) => s.to_string(),
                                    _ => i.to_string(),
                                },
                                _ => i.to_string(),
                            };
                            labels.push(label);
                            values.push(value);
                        }
                    }
                    // contents：文档序（VM convert_tabs 同款——trigger 与
                    // content 序一致时索引映射成立）；多子件包 col。
                    let content_sources: Vec<Vec<&crate::aura::AuraNode>> = if use_flat {
                        flats
                            .iter()
                            .map(|t| match t {
                                crate::aura::AuraNode::Element { children, .. } => {
                                    children.iter().collect()
                                }
                                _ => Vec::new(),
                            })
                            .collect()
                    } else {
                        content_nodes
                            .iter()
                            .map(|c| match c {
                                crate::aura::AuraNode::Element { children, .. } => {
                                    children.iter().collect()
                                }
                                _ => Vec::new(),
                            })
                            .collect()
                    };
                    let mut content_exprs: Vec<String> = Vec::new();
                    for parts in &content_sources {
                        let mut views: Vec<String> = Vec::new();
                        for n in parts {
                            views.push(self.generate_view_tree(n));
                        }
                        let expr = if views.len() == 1 {
                            views.into_iter().next().unwrap()
                        } else if views.is_empty() {
                            "View::Empty".to_string()
                        } else {
                            let mut col = "View::col()".to_string();
                            for v in views {
                                col = format!("{col}.child({v})");
                            }
                            format!("{col}.build()")
                        };
                        content_exprs.push(expr);
                    }
                    let labels_lit = format!(
                        "vec![{}]",
                        labels
                            .iter()
                            .map(|l| format!("\"{l}\".to_string()"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                    let contents_lit = format!(
                        "vec![{}]",
                        content_exprs.join(", ")
                    );
                    let mut builder = format!("View::tabs({labels_lit}).contents({contents_lit})");
                    // selected：active（索引）→ value（字面量=编译期匹配 /
                    // 绑定=运行时 position 表达式）→ defaultvalue/default
                    //（同前）→ 0；越界钳制 min（sel.min(n-1) 运行时兜底）。
                    let prop_str_lit = |key: &str| -> Option<String> {
                        match props.get(key) {
                            Some(AuraPropValue::Expr(crate::ast::Expr::Str(s))) => Some(s.to_string()),
                            _ => None,
                        }
                    };
                    let values_match_arm = |val: &str, values: &[String]| -> String {
                        match values.iter().position(|v| v == val) {
                            Some(idx) => idx.to_string(),
                            // 非字面量值集合内无匹配 → 0（数值串 VM 侧另
                            // 解析索引；a2r 简化为 0 兜底 + unwrap_or(0)
                            // 运行时兜底同语义）。
                            None => "0usize".to_string(),
                        }
                    };
                    let value_binding = |key: &str| -> Option<String> {
                        match props.get(key) {
                            Some(AuraPropValue::Expr(crate::ast::Expr::Ident(name))) => {
                                let f = name.as_str().trim_start_matches('.');
                                (!f.is_empty()).then(|| f.to_string())
                            }
                            Some(AuraPropValue::Expr(crate::ast::Expr::Dot(obj, field))) => match obj
                                .as_ref()
                            {
                                crate::ast::Expr::Ident(base)
                                    if base.as_str() == "." || base.as_str() == "self" =>
                                {
                                    Some(field.as_str().to_string())
                                }
                                _ => None,
                            },
                            _ => None,
                        }
                    };
                    let values_pos_expr = |field: &str, values: &[String]| -> String {
                        let arr = format!(
                            "[{}]",
                            values
                                .iter()
                                .map(|v| format!("\"{v}\""))
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                        format!("{arr}.iter().position(|v| *v == self.{field}.as_str()).unwrap_or(0)")
                    };
                    let has_active_or_value =
                        props.keys().any(|k| matches!(k.as_str(), "active" | "value"));
                    let selected_expr: Option<String> = if let Some(AuraPropValue::Expr(
                        crate::ast::Expr::Int(i),
                    )) = props.get("active")
                    {
                        Some(format!("({i} as usize)"))
                    } else if let Some(AuraPropValue::Expr(crate::ast::Expr::Str(s))) =
                        props.get("value")
                    {
                        Some(values_match_arm(s, &values))
                    } else if let Some(field) = value_binding("value") {
                        Some(values_pos_expr(&field, &values))
                    } else if !has_active_or_value {
                        prop_str_lit("defaultvalue")
                            .or_else(|| prop_str_lit("default"))
                            .map(|s| values_match_arm(&s, &values))
                            .or_else(|| value_binding("defaultvalue").map(|f| values_pos_expr(&f, &values)))
                    } else {
                        None
                    };
                    if let Some(expr) = selected_expr {
                        builder = format!("{builder}.selected({expr})");
                    }
                    // variant：TabsVariant::parse（full path——生成物预载
                    // use 不含该类型；PointerMoveHandler 同款先例）。
                    if let Some(v) = prop_str_lit("variant") {
                        builder =
                            format!("{builder}.variant(auto_lang::ui::view::TabsVariant::parse(\"{v}\"))");
                    }
                    // 样式（style/class prop → TabsBuilder.style）。
                    if let Some(st) = prop_str_lit("style").or_else(|| prop_str_lit("class")) {
                        builder = format!("{builder}.style(\"{st}\")");
                    }
                    // onselect：根事件优先，缺省取首个 trigger onclick；
                    // 闭包物化载荷消息（首参 = value 串——convert_tabs
                    // 回调契约；载荷数自适应 select 臂同款）。
                    let onselect_handler = events
                        .iter()
                        .find(|(e, _)| matches!(e.as_str(), "onselect" | "onSelect"))
                        .map(|(_, h)| h)
                        .cloned()
                        .or_else(|| {
                            triggers.iter().find_map(|t| match t {
                                crate::aura::AuraNode::Element { events: te, .. } => te
                                    .iter()
                                    .find(|(e, _)| matches!(e.as_str(), "onclick" | "onClick"))
                                    .map(|(_, h)| h.clone()),
                                _ => None,
                            })
                        });
                    if let Some(handler) = onselect_handler {
                        let variant = self.extract_variant_name(&handler.handler);
                        let msg_name = self.current_msg_name();
                        let closure = match self
                            .message_variants
                            .iter()
                            .find(|v| v.name == variant)
                            .map(|v| v.payload.len())
                            .unwrap_or(0)
                        {
                            1 => format!(
                                "{{ let vals = {labels_lit}; move |idx: usize| {msg_name}::{variant}(vals.get(idx).cloned().unwrap_or_else(|| idx.to_string())) }}"
                            ),
                            2 => format!(
                                "{{ let vals = {labels_lit}; move |idx: usize| {msg_name}::{variant}(idx as i32, vals.get(idx).cloned().unwrap_or_else(|| idx.to_string())) }}"
                            ),
                            _ => format!(
                                "move |idx: usize| {{ let _ = idx; {msg_name}::{variant}() }}"
                            ),
                        };
                        builder = format!("{builder}.on_select({closure})");
                    }
                    return format!("{builder}.build()");
                }

                let builder_start = if self.is_leaf_tag(tag.as_str()) {
                    if let Some(ref name) = text_state_ref {
                        if tag == "button" {
                            // Plan 374: Use loop var directly if it's in scope
                            let name_ref = if self.is_loop_var(name) { name.to_string() } else { format!("self.{}", name) };
                            format!("View::button(format!(\"{{}}\", {}))", name_ref)
                        } else {
                            let name_ref = if self.is_loop_var(name) { name.to_string() } else { format!("self.{}", name) };
                            format!("View::text(format!(\"{{}}\", {}))", name_ref)
                        }
                    } else if let Some(label) = &text_prop {
                        if tag == "button" {
                            format!("View::{}(\"{}\")", view_fn, label)
                        } else if label.contains("${") {
                            format!("View::{}({})", view_fn, self.interpolate_str(label))
                        } else {
                            format!("View::{}(\"{}\")", view_fn, label)
                        }
                    } else if let Some(ref text) = text_rust_expr {
                        // Dynamic expression (FieldAccess, Index, etc.) as text content
                        // PLAN-039 T-05 发现臂④（台账 M7-c「缺口即发现即修」）：
                        // 动态标签表达式统一 format! 借用包裹——直发 store
                        // String 字段（minesweeper `View::button(self.store.
                        // face_icon)`）是 move 断点；format! 走 Display 借用，
                        // 且数值型字段标签同型治愈。
                        if tag == "button" {
                            format!("View::button(format!(\"{{}}\", {}))", text)
                        } else {
                            format!("View::text(format!(\"{{}}\", {}))", text)
                        }
                    } else {
                        format!("View::{}(())", view_fn)
                    }
                } else {
                    format!("View::{}()", view_fn)
                };

                // Check if any styling props exist (class/style)
                let has_styling = props.keys().any(|k| k == "style" || k == "class");

                // For non-button leaf tags with text content and styling,
                // use View::text_styled() to avoid builder pattern issues
                // (View::text("str") returns View, not ViewBuilder, so chaining won't work)
                // Also handles text from a single Text child node (e.g. text f"..." { class: "..." })
                if self.is_leaf_tag(tag.as_str()) && tag != "button" && (children.is_empty() || child_text_content.is_some()) && has_styling {
                    let user_style = props.get("style")
                        .or_else(|| props.get("class"))
                        .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v { Some(s.to_string()) } else { None })
                        .unwrap_or_default();

                    // Prepend heading default styles (h1→text-4xl font-bold, etc.)
                    let style_str = match Self::heading_default_style(tag.as_str()) {
                        Some(default) if !user_style.is_empty() => format!("{} {}", default, user_style),
                        Some(default) => default.to_string(),
                        None => user_style,
                    };

                    if let Some(ref name) = text_state_ref {
                        if self.is_loop_var(name) {
                            return format!("View::text_styled(format!(\"{{}}\", {}), \"{}\")", name, style_str);
                        }
                        return format!("View::text_styled(format!(\"{{}}\", self.{}), \"{}\")", name, style_str);
                    }
                    if let Some(label) = &text_prop {
                        // Check if text contains interpolation like ${.field}
                        if label.contains("${") {
                            return format!("View::text_styled({}, \"{}\")", self.interpolate_str(label), style_str);
                        }
                        return format!("View::text_styled(\"{}\".to_string(), \"{}\")", label, style_str);
                    }
                    if let Some(ref text) = text_rust_expr {
                        // Plan 407 R2: wrap self. references in format! to avoid
                        // moving String fields out of self in the immutable view().
                        let text = if text.starts_with("self.") {
                            format!("format!(\"{{}}\", {})", text)
                        } else {
                            text.clone()
                        };
                        return format!("View::text_styled({}, \"{}\")", text, style_str);
                    }
                }

                // Whether the "text" prop was consumed as a constructor arg
                let text_prop_consumed = self.is_leaf_tag(tag.as_str())
                    && (text_prop.is_some() || text_state_ref.is_some() || text_rust_expr.is_some());

                // Non-button leaf tags with text and no styling:
                // View::text("str") returns View<M> directly, NOT a builder.
                // Skip .build() to avoid compile error.
                // Heading tags (h1-h3) always use text_styled with their default styles.
                let heading_default = Self::heading_default_style(tag.as_str());
                if self.is_leaf_tag(tag.as_str()) && tag != "button" && (children.is_empty() || child_text_content.is_some()) && !has_styling {
                    if let Some(ref name) = text_state_ref {
                        if let Some(default) = heading_default {
                            return format!("View::text_styled(format!(\"{{}}\", self.{}), \"{}\")", name, default);
                        }
                        return format!("View::text(format!(\"{{}}\", self.{}))", name);
                    }
                    if let Some(label) = &text_prop {
                        if let Some(default) = heading_default {
                            return format!("View::text_styled(\"{}\".to_string(), \"{}\")", label, default);
                        }
                        if label.contains("${") {
                            return format!("View::text({})", self.interpolate_str(label));
                        }
                        return format!("View::text(\"{}\".to_string())", label);
                    }
                    if let Some(ref text) = text_rust_expr {
                        // Plan 407 R2: wrap self. references to avoid String move.
                        let text = if text.starts_with("self.") {
                            format!("format!(\"{{}}\", {})", text)
                        } else {
                            text.clone()
                        };
                        if let Some(default) = heading_default {
                            return format!("View::text_styled({}, \"{}\")", text, default);
                        }
                        return format!("View::text({})", text);
                    }
                    // Leaf tag without text content but no styling — e.g. avatar
                    // These go through the builder path
                }

                // Special handling for "center" — View::center(child) takes a child directly,
                // not the builder pattern. Assemble children into a col, then wrap in center.
                if tag == "center" {
                    let style_str = props.get("style")
                        .or_else(|| props.get("class"))
                        .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v { Some(s.to_string()) } else { None })
                        .unwrap_or_default();

                    // Build children into a col
                    let child_view = if children.is_empty() {
                        "View::Empty".to_string()
                    } else if children.len() == 1 {
                        self.generate_view_tree(&children[0])
                    } else {
                        let mut col = "View::col()".to_string();
                        for child in children {
                            let child_code = self.generate_view_tree(child);
                            col = format!("{}.child({})", col, child_code);
                        }
                        format!("{}.build()", col)
                    };

                    let mut builder = format!("View::center({})", child_view);
                    if !style_str.is_empty() {
                        builder = format!("{}.style(\"{}\")", builder, style_str);
                    }
                    return format!("{}.build()", builder);
                }

                // PLAN-022 T-06 顺带修(2026-09-19):div/container 臂镜像
                // center——View::container 已是 container(child) 一参形态
                // (ViewContainerBuilder.child 为替换语义,多子链式静默丢
                // 子),旧发射 View::container()+.child() 链对 div 大户
                // (app.at 载具)编译不过且语义错。027 T-04 div→container
                // 映射的配套缺口,归 027 面备案。
                if tag == "container" {
                    let style_str = props.get("style")
                        .or_else(|| props.get("class"))
                        .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v { Some(s.to_string()) } else { None })
                        .unwrap_or_default();

                    let child_view = if children.is_empty() {
                        "View::Empty".to_string()
                    } else if children.len() == 1 {
                        self.generate_view_tree(&children[0])
                    } else {
                        let mut col = "View::col()".to_string();
                        for child in children {
                            let child_code = self.generate_view_tree(child);
                            col = format!("{}.child({})", col, child_code);
                        }
                        format!("{}.build()", col)
                    };

                    let mut builder = format!("View::container({})", child_view);
                    if !style_str.is_empty() {
                        builder = format!("{}.style(\"{}\")", builder, style_str);
                    }
                    return format!("{}.build()", builder);
                }

                if children.is_empty() {
                    // Single element without children
                    let mut builder = builder_start;

                    // Add props (skip "text" if already used as constructor arg)
                    for (key, value) in props_sorted.clone() {
                        if text_prop_consumed && key == "text" { continue; }
                        builder = self.add_prop_to_builder(&builder, key, value);
                    }

                    // Add events
                    for (event, handler) in events_sorted.clone() {
                        builder = self.add_event_to_builder(&builder, event, handler);
                    }

                    // Button without onclick — add no-op handler to prevent panic
                    if tag == "button" && !events.iter().any(|(e, _)| e == "onclick" || e == "onClick") {
                        builder = format!("{}.on_click(|_| ())", builder);
                    }

                    self.wrap_button_double_click(format!("{}.build()", builder), dbl_event)
                } else if tag == "button" {
                    // Button with children. The View::Button model only has a
                    // `label` (no children field), so `.child()` calls are
                    // silently dropped at build time and the button renders
                    // empty. Fold the button's text children into a single
                    // composite label so the content is visible.
                    let label_expr = self.collect_button_label(children);
                    let mut builder = format!("View::button({})", label_expr);
                    for (key, value) in props_sorted.clone() {
                        if key == "text" { continue; }
                        builder = self.add_prop_to_builder(&builder, key, value);
                    }
                    for (event, handler) in events_sorted.clone() {
                        builder = self.add_event_to_builder(&builder, event, handler);
                    }
                    if !events.iter().any(|(e, _)| e == "onclick" || e == "onClick") {
                        builder = format!("{}.on_click(|_| ())", builder);
                    }
                    self.wrap_button_double_click(format!("{}.build()", builder), dbl_event)
                } else {
                    // Element with children
                    let mut builder = builder_start;

                    // Add props (skip "text" if already used as constructor arg)
                    for (key, value) in props_sorted.clone() {
                        if text_prop_consumed && key == "text" { continue; }
                        builder = self.add_prop_to_builder(&builder, key, value);
                    }

                    // Add children — use .children() for for-loops (which produce Vec<View>),
                    // .child() for single views
                    for child in children {
                        // PLAN-019 T-00 勘定:row 直属 for 摊平(Plan 407 grid
                        // 先例推广)——ForLoop 的 col 包装会令 row 内按钮纵向
                        // 堆叠;横向 spread 才是 row 语义(VM 臂 convert_row
                        // Plan 047 既有同款)。col 臂不动防扰存量。
                        if tag == "row" {
                            if matches!(child, AuraNode::ForLoop { .. }) {
                                let map_expr = self.generate_for_loop_cells(child);
                                builder = format!("{}.children({}.collect::<Vec<_>>())", builder, map_expr);
                                continue;
                            }
                        }
                        // Plan 374: ForLoop now produces a single View (wrapped in col().children)
                        // so always use .child() regardless of node type.
                        let child_code = self.generate_view_tree(child);
                        builder = format!("{}.child({})", builder, child_code);
                    }

                    // Add events last
                    for (event, handler) in events_sorted.clone() {
                        builder = self.add_event_to_builder(&builder, event, handler);
                    }

                    // Button without onclick — add no-op handler to prevent panic
                    if tag == "button" && !events.iter().any(|(e, _)| e == "onclick" || e == "onClick") {
                        builder = format!("{}.on_click(|_| ())", builder);
                    }

                    self.wrap_button_double_click(format!("{}.build()", builder), dbl_event)
                }
            }

            AuraNode::Text(content) => {
                match content {
                    AuraTextContent::Literal(s) => {
                        format!("View::text(\"{}\")", s)
                    }
                    AuraTextContent::Interpolated { template, bindings } => {
                        // Convert template to format! string with {} placeholders
                        let mut format_str = template.clone();
                        let mut format_args = Vec::new();

                        for binding in bindings.iter() {
                            // Replace ${.binding} and ${binding} with {}
                            format_str = format_str.replace(
                                &format!("${{{}.{}}}", ".", binding),
                                "{}"
                            );
                            format_str = format_str.replace(
                                &format!("${{{}}}", binding),
                                "{}"
                            );

                            // Use binding directly if loop var, otherwise self.binding
                            let arg = if self.is_loop_var(binding) {
                                binding.clone()
                            } else {
                                format!("self.{}", binding)
                            };
                            format_args.push(arg);
                        }

                        format!("View::text(format!(\"{}\", {}))", format_str, format_args.join(", "))
                    }
                }
            }

            AuraNode::ForLoop { var, index, iterable, body, .. } => {
                // Plan 371 步骤3: Sanitize loop var to avoid Rust keyword/macro
                // conflicts (e.g. `for todo in ...` collides with `todo!()` macro).
                let var = sanitize_rust_ident(var);
                let index = index.as_ref().map(|i| sanitize_rust_ident(i));
                // Generate iterator-based view construction
                let iter_name = iterable.trim_start_matches('.');
                // Plan 374: Check if the last component is a computed property (needs ()).
                let last_component = iter_name.rsplit('.').next().unwrap_or(iter_name);
                let needs_method_call = self.computed_names.contains(last_component)
                    || STORE_COMPUTED_NAMES.with(|sn| sn.borrow().contains(last_component));
                let iter_expr = if iterable.starts_with('.') {
                    let base = if needs_method_call {
                        format!("self.{}()", iter_name)
                    } else {
                        self.resolve_dotted_path(iter_name)
                    };
                    base
                } else {
                    iterable.clone()
                };

                // Check if iterable is a Value-type collection.
                // Handle both simple names ("notes") and compound paths ("store.notes").
                let iter_last_name = iter_name.rsplit('.').next().unwrap_or(iter_name);
                // PLAN-039 T-13（E-D5-A）：store 字段 typed 短路——
                // `.store.cards`（Vec<Card>）迭代变量是 typed（元素字段
                // 直达），fallback 兜底臂不再误收 Value 格（×25 株）。
                let store_field_ty = if iter_name.starts_with("store.")
                    || iter_name.starts_with(".store.")
                {
                    self.store_field_rust_type(iter_last_name)
                } else {
                    None
                };
                let store_typed_iter = store_field_ty
                    .map_or(false, |t| t != "Vec<serde_json::Value>" && t != "serde_json::Value");
                let is_value_iter = self.state_types.get(iter_name)
                    .map(|ty| ty.contains("serde_json::Value"))
                    .or_else(|| self.state_types.get(iter_last_name)
                        .map(|ty| ty.contains("serde_json::Value")))
                    .unwrap_or(false)
                    // Plan 374: If we can't determine the type, assume Value (most
                    // data from API/store is serde_json::Value in this system).
                    || (iter_name.contains('.') 
                        && !self.state_types.contains_key(iter_name)
                        && !self.state_types.contains_key(iter_last_name)
                        && !store_typed_iter);

                // Push loop vars into scope
                self.push_loop_vars(&var, index.as_deref());
                if is_value_iter {
                    self.value_loop_vars.insert(var.clone());
                }

                // Generate body with loop vars in scope
                let body_code: Vec<String> = body.iter()
                    .map(|child| self.generate_view_tree(child))
                    .collect();

                // Pop loop vars from scope
                self.pop_loop_vars(&var, index.as_deref());
                self.value_loop_vars.remove(&var);

                // Auto-generate search filter: if the widget has a "search" state var
                // and we're iterating a Value collection, insert .filter() before .map()
                let search_filter = if is_value_iter && self.state_types.contains_key("search") {
                    let var_ref = var.clone();
                    // When enumerate() is used (index present), the filter closure
                    // receives &(usize, &Value) — destructure to access the element.
                    let filter_pattern = if index.is_some() {
                        format!("(_, {})", var_ref)
                    } else {
                        var_ref.clone()
                    };
                    Some(format!(
                        ".filter(|{}| {{ \
                            let __q = self.search.to_lowercase(); \
                            if __q.is_empty() {{ return true; }} \
                            let __t = {}[\"title\"].as_str().unwrap_or_default().to_lowercase(); \
                            __t.contains(&__q) \
                        }})",
                        filter_pattern, var_ref
                    ))
                } else {
                    None
                };

                let map_expr = if let Some(idx) = index {
                    // Cast the usize index to i32 so comparisons with state
                    // fields (typically i32) type-check.
                    format!("{}.iter().enumerate(){}{}.map(|({}, {})| {{ let {} = {} as i32; {} }})", iter_expr, search_filter.as_ref().map_or(String::new(), |f| f.clone()), "", idx, var, idx, idx, body_code.join("\n"))
                } else {
                    format!("{}.iter(){}{}.map(|{}| {{ {} }})", iter_expr, search_filter.as_ref().map_or(String::new(), |f| f.clone()), "", var, body_code.join("\n"))
                };
                // Plan 374: Always produce a single View by wrapping in col().children().
                // This works in both .child() and conditional/if contexts.
                format!("View::col().children({}.collect::<Vec<_>>()).build()", map_expr)
            }

            AuraNode::Conditional { condition, then_body, else_body, .. } => {
                let rust_condition = self.convert_condition(condition);
                let then_code: Vec<String> = then_body.iter()
                    .map(|child| self.generate_view_tree(child))
                    .collect();

                if let Some(else_nodes) = else_body {
                    let else_code: Vec<String> = else_nodes.iter()
                        .map(|child| self.generate_view_tree(child))
                        .collect();
                    format!("if {} {{ {} }} else {{ {} }}", rust_condition, Self::wrap_views(&then_code), Self::wrap_views(&else_code))
                } else {
                    format!("if {} {{ {} }} else {{ View::Empty }}", rust_condition, Self::wrap_views(&then_code))
                }
            }

            AuraNode::Component { name, props, .. } => {
                // Generate component instantiation with message wrapping
                let msg_name = self.current_msg_name();
                // Plan 374: Sort props for deterministic constructor
                // argument ordering. PLAN-039 T-14：优先 props 声明序
                // （组件 new 签名序——字母序调用=参数错位株）；未注册
                // 组件保持字母序（确定性）。
                let collected: Vec<_> = props.iter().collect();
                let sorted_props = self.order_component_props_by_decl(collected, name);
                let mut constructor_args: Vec<String> = Vec::new();
                for (key, value) in sorted_props {
                    // PLAN-039 T-14：实参按目标 prop 型强转。
                    let raw = self.arg_to_rust(value);
                    constructor_args.push(self.coerce_arg_to_prop_type(&raw, name, key));
                }
                let args_str = constructor_args.join(", ");
                format!(
                    "{}::new({}).view().map_msg(|m| {}::{}(m))",
                    name, args_str, msg_name, name
                )
            }

            // Plan 105/408: Router outlet and link.
            // PLAN-039 D5（§5.1 定案记录）：outlet 三态——
            //   Fold（routes{"/"->use X} 单路由无参）→ 持久子件 X 直用
            //     （同自定义 widget 臂 :5495-5504：store 同步 + map_msg
            //     包装；module 经 generate_rust 注册 child_components 走
            //     包装变体/持久字段/on() 转发全链）——kanban board 页整页
            //     空屏（View::empty 静默）的清偿臂；
            //   Reject（多路由/带参路由）→ compile_error 带 P039 债指针
            //     （原 View::empty 静默升响亮——I1）；
            //   None（无 routes 块）→ 维持 View::empty（防御形态）。
            AuraNode::Outlet => match self.outlet_route.clone() {
                OutletRoute::Fold(module) => {
                    let msg_name = self.current_msg_name();
                    let field = Self::child_field_name(&module);
                    let has_store = STORE_NAMES.with(|sn| !sn.borrow().is_empty());
                    if has_store {
                        format!(
                            "{{ let mut __c = self.{field}.clone(); __c.store = self.store.clone(); \
                             __c.view().map_msg(|m| {msg_name}::{module}(m)) }}"
                        )
                    } else {
                        format!("self.{field}.view().map_msg(|m| {msg_name}::{module}(m))")
                    }
                }
                OutletRoute::Reject => {
                    let msg = "a2r codegen: multi-route/parametric `routes` + `outlet` not yet supported (PLAN-039 D5: single-route folding only; router family tracked as P039 debt)";
                    format!("{{ std::compile_error!(\"{msg}\"); unreachable!() }}")
                }
                OutletRoute::None => {
                    // No routes block; keep the empty placeholder (defensive).
                    "View::empty()".to_string()
                }
            }

            AuraNode::Link { to, text, href, children, .. } => {
                // Render link as styled text (no routing in compiled Rust).
                // PLAN-026 T-02：有子件时组合子件（col）——原实现无条件落
                // to/href 占位标签，`link (to:) { text … }` 主形态子件内容
                // 静默丢失（AC-05 非 Silent 丢内容纪律）。无子件保留既有
                // text/href/to 标签兜底 + 链接缺省观感。
                if !children.is_empty() {
                    let child_view = if children.len() == 1 {
                        self.generate_view_tree(&children[0])
                    } else {
                        let mut col = "View::col()".to_string();
                        for child in children {
                            col = format!("{}.child({})", col, self.generate_view_tree(child));
                        }
                        format!("{}.build()", col)
                    };
                    return format!("View::container({child_view}).build()");
                }
                let label = if !text.is_empty() {
                    text.clone()
                } else if !href.is_empty() {
                    href.clone()
                } else {
                    to.clone()
                };
                format!("View::text_styled(\"{}\", \"text-blue-600 underline cursor-pointer\")", label)
            }
        }
    }

    /// Generate child component instantiation with message wrapping.
    /// E.g., EditorPanel(note: .notes[.active_id]) →
    ///   EditorPanel::new(self.notes[self.active_id as usize]).view().map_msg(|m| AppMsg::EditorPanel(m))
    /// PLAN-533 T3: 模态对话框家族 tag 角色。归一化（剥 `-`/`_` + 小写，
    /// 与 schema/aura.at 头部同规则）后匹配 alert-dialog 与 dialog 两族
    /// （alert-dialog-action ≡ alert_dialog_action ≡ AlertDialogAction）。
    fn modal_dialog_tag_role(tag: &str) -> Option<&'static str> {
        let norm: String = tag
            .chars()
            .filter(|c| *c != '-' && *c != '_')
            .collect::<String>()
            .to_lowercase();
        match norm.as_str() {
            "alertdialog" | "dialog" | "dropdownmenu"
            // PLAN-534: sheet/drawer 并入同表（可关闭族,同 dialog 语义）。
            | "sheet" | "drawer" => Some("root"),
            "alertdialogtrigger" | "dialogtrigger" | "dropdownmenutrigger"
            | "sheettrigger" | "drawertrigger" => Some("trigger"),
            "alertdialogcontent" | "dialogcontent" | "dropdownmenucontent"
            | "sheetcontent" | "drawercontent" => Some("content"),
            "alertdialogtitle" | "dialogtitle"
            | "sheettitle" | "drawertitle" => Some("title"),
            "alertdialogdescription" | "dialogdescription"
            | "sheetdescription" | "drawerdescription" => Some("description"),
            "alertdialogheader" | "dialogheader"
            | "sheetheader" | "drawerheader" => Some("header"),
            "alertdialogfooter" | "dialogfooter"
            | "sheetfooter" | "drawerfooter" => Some("footer"),
            "alertdialogcancel" => Some("cancel"),
            "alertdialogaction" => Some("action"),
            "alertdialogclose" | "dialogclose"
            | "sheetclose" | "drawerclose" => Some("close"),
            "dropdownmenuitem" => Some("item"),
            "dropdownmenulabel" => Some("label"),
            "dropdownmenuseparator" => Some("separator"),
            _ => None,
        }
    }

    /// PLAN-534 D4: hovercard 根/子件角色（归一化同上）。不并入
    /// modal_dialog_tag_role——hover 走 MouseArea 包锚 + Bottom 非模态,
    /// 发射面与模态族不同。
    fn hover_card_role(tag: &str) -> Option<&'static str> {
        let norm: String = tag
            .chars()
            .filter(|c| *c != '-' && *c != '_')
            .collect::<String>()
            .to_lowercase();
        match norm.as_str() {
            "hovercard" => Some("root"),
            "hovercardtrigger" => Some("trigger"),
            "hovercardcontent" => Some("content"),
            _ => None,
        }
    }

    /// PLAN-533 T7: dropdown-menu 根（锚定菜单族）。
    fn dropdown_menu_root(tag: &str) -> bool {
        let norm: String = tag
            .chars()
            .filter(|c| *c != '-' && *c != '_')
            .collect::<String>()
            .to_lowercase();
        norm == "dropdownmenu"
    }

    /// PLAN-534: sheet 根（贴边面板族）。
    fn sheet_root(tag: &str) -> bool {
        let norm: String = tag
            .chars()
            .filter(|c| *c != '-' && *c != '_')
            .collect::<String>()
            .to_lowercase();
        norm == "sheet"
    }

    /// PLAN-534: drawer 根（贴边面板族）。
    fn drawer_root(tag: &str) -> bool {
        let norm: String = tag
            .chars()
            .filter(|c| *c != '-' && *c != '_')
            .collect::<String>()
            .to_lowercase();
        norm == "drawer"
    }

    /// PLAN-534: sheet/drawer 发射表——side/direction 值 → (chrome,
    /// placement 构造串)。chrome 与解释器臂 side_panel_chrome 同串（双轨
    /// 一致断言的锚点）:横条 w-96 定宽 + h-full 拉满,纵条 w-full 拉满;
    /// drawer 竖向追加贴缘圆角（bottom rounded-t / top rounded-b）。
    fn side_panel_emission(side_val: &str, is_drawer: bool) -> (&'static str, &'static str) {
        const EDGE_LEFT: &str = "auto_lang::ui::view::PopoverPlacement::EdgeLeft";
        const EDGE_RIGHT: &str = "auto_lang::ui::view::PopoverPlacement::EdgeRight";
        const EDGE_TOP: &str = "auto_lang::ui::view::PopoverPlacement::EdgeTop";
        const EDGE_BOTTOM: &str = "auto_lang::ui::view::PopoverPlacement::EdgeBottom";
        const H_CHROME: &str = "w-96 bg-background border shadow-lg p-6 gap-4 h-full";
        const V_CHROME: &str = "bg-background border shadow-lg p-6 gap-4 w-full";
        match side_val {
            "left" => (H_CHROME, EDGE_LEFT),
            "top" => (
                if is_drawer {
                    "bg-background border shadow-lg p-6 gap-4 w-full rounded-b-lg"
                } else {
                    V_CHROME
                },
                EDGE_TOP,
            ),
            "bottom" => (
                if is_drawer {
                    "bg-background border shadow-lg p-6 gap-4 w-full rounded-t-lg"
                } else {
                    V_CHROME
                },
                EDGE_BOTTOM,
            ),
            _ => (H_CHROME, EDGE_RIGHT),
        }
    }

    /// PLAN-534: drawer 竖向装饰把手发射串——全宽容器内居中的 w-8 h-1
    /// 圆角条（与解释器臂 drawer_handle_view 同构:纯视觉无手势）。
    fn drawer_handle_emission() -> String {
        "View::container(View::container(auto_lang::ui::view::View::Empty).style(\"w-8 h-1 rounded-full bg-muted\").build()).center_x().style(\"w-full py-2\").build()".to_string()
    }

    /// PLAN-534 D4: hovercard 根臂——trigger 包 View::MouseArea（hover
    /// 进/出驱动铸造 `__dlg_enter_N/leave_N` 或用户显式绑定）,content 装
    /// 非模态面板（chrome 与解释器臂 convert_hovercard 同串）,
    /// placement=Bottom,on_dismiss=None（关闭只靠 leave）。
    fn generate_hover_card_popover(
        &mut self,
        props: &std::collections::HashMap<String, AuraPropValue>,
        children: &[AuraNode],
    ) -> String {
        let mut trigger: Option<&AuraNode> = None;
        let mut panel_nodes: Vec<&AuraNode> = Vec::new();
        for c in children {
            if let AuraNode::Element { tag, .. } = c {
                match Self::hover_card_role(tag) {
                    Some("trigger") if trigger.is_none() => {
                        trigger = Some(c);
                        continue;
                    }
                    Some("content") => {
                        if let AuraNode::Element { children: inner, .. } = c {
                            panel_nodes.extend(inner.iter());
                        }
                        continue;
                    }
                    _ => {}
                }
            }
        }
        let (anchor_code, on_enter, on_exit) = match trigger {
            Some(AuraNode::Element { children: t_children, props: t_props, events: t_events, .. }) => {
                let enter = ["onmouseenter", "onhover"]
                    .iter()
                    .find_map(|k| t_events.get(*k))
                    .map(|h| format!("Some({})", self.handler_to_rust_direct_msg(&h.handler, &h.params)))
                    .unwrap_or_else(|| "None".to_string());
                let exit = ["onmouseleave", "onhoverout"]
                    .iter()
                    .find_map(|k| t_events.get(*k))
                    .map(|h| format!("Some({})", self.handler_to_rust_direct_msg(&h.handler, &h.params)))
                    .unwrap_or_else(|| "None".to_string());
                let inner = if t_children.len() == 1 {
                    self.generate_view_tree(&t_children[0])
                } else if t_children.is_empty() {
                    let label = Self::modal_child_label(t_props, t_children);
                    if label.is_empty() {
                        "auto_lang::ui::view::View::Empty".to_string()
                    } else {
                        format!("View::text_styled(\"{}\".to_string(), \"\")", label)
                    }
                } else {
                    let mut b = "View::row()".to_string();
                    for c in t_children {
                        b = format!("{}.child({})", b, self.generate_view_tree(c));
                    }
                    format!("{}.build()", b)
                };
                (inner, enter, exit)
            }
            _ => (
                "auto_lang::ui::view::View::Empty".to_string(),
                "None".to_string(),
                "None".to_string(),
            ),
        };
        let mut panel = "View::col()".to_string();
        for c in panel_nodes {
            panel = format!("{}.child({})", panel, self.generate_view_tree(c));
        }
        let panel = format!(
            "{}.style(\"w-80 bg-popover border rounded-lg shadow-md p-4\").build()",
            panel
        );
        let open_expr = match props.get("open") {
            Some(AuraPropValue::Expr(crate::ast::Expr::Bool(b))) => b.to_string(),
            Some(AuraPropValue::Expr(e)) => self.ast_expr_to_rust(e),
            _ => "false".to_string(),
        };
        format!(
            "View::Popover {{ anchor: auto_lang::ui::view::PopoverAnchor::Widget(Box::new(View::MouseArea {{ content: Box::new({}), on_enter: {}, on_exit: {}, on_double_click: None, on_click: None, on_context_menu: None, on_release: None, on_move: None, logical_extent: None, style: None }})), content: Box::new({}), placement: auto_lang::ui::view::PopoverPlacement::Bottom, open: {}, on_dismiss: None }}",
            anchor_code, on_enter, on_exit, panel, open_expr
        )
    }

    /// PLAN-533 T6: 可关闭模态根（dialog 族）——非 alert 族。shadcn 语义：
    /// dialog 的 ESC/外点/锚点关闭经 on_dismiss 回流;alert-dialog 不关。
    /// PLAN-534: sheet/drawer 均可关闭族,同 dialog。
    fn dismissable_dialog_root(tag: &str) -> bool {
        let norm: String = tag
            .chars()
            .filter(|c| *c != '-' && *c != '_')
            .collect::<String>()
            .to_lowercase();
        norm == "dialog" || norm == "dropdownmenu" || norm == "sheet" || norm == "drawer"
    }

    /// 家族子件的文字内容：text prop → label prop → 首 Text 子节点。
    fn modal_child_label(
        props: &std::collections::HashMap<String, AuraPropValue>,
        children: &[AuraNode],
    ) -> String {
        for key in ["text", "label"] {
            if let Some(AuraPropValue::Expr(crate::ast::Expr::Str(s))) = props.get(key) {
                return s.to_string();
            }
        }
        if let Some(AuraNode::Text(AuraTextContent::Literal(s))) = children.first() {
            return s.to_string();
        }
        String::new()
    }

    /// 家族子件的 user class（class/style prop 的字面量串）。
    fn modal_user_class(props: &std::collections::HashMap<String, AuraPropValue>) -> String {
        props
            .get("class")
            .or_else(|| props.get("style"))
            .and_then(|v| {
                if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v {
                    Some(s.to_string())
                } else {
                    None
                }
            })
            .unwrap_or_default()
    }

    /// PLAN-533 T3: alert-dialog/dialog 根臂 —— 拆解 trigger（首枚锚）与
    /// content（面板），发射 View::Popover 模态构造。面板 chrome 与解释
    /// 器臂（PLAN-530 W13 convert_alert_dialog）同串（w-96 …），保双轨
    /// 视觉一致。open 取 `open` prop 绑定（缺省 false）；on_dismiss 暂
    /// None（T6 接 dialog 族 ESC/外点回流）。trigger 单 Text 子降级为无
    /// 操作按钮（vue trigger 语义；T5 换铸 __popover_toggle 自管开合）。
    fn generate_modal_popover(
        &mut self,
        tag: &str,
        props: &std::collections::HashMap<String, AuraPropValue>,
        children: &[AuraNode],
    ) -> String {
        let mut trigger: Option<&AuraNode> = None;
        let mut panel_nodes: Vec<&AuraNode> = Vec::new();
        for c in children {
            if let AuraNode::Element { tag, .. } = c {
                match Self::modal_dialog_tag_role(tag) {
                    Some("trigger") if trigger.is_none() => {
                        trigger = Some(c);
                        continue;
                    }
                    Some("content") => {
                        if let AuraNode::Element { children: inner, .. } = c {
                            panel_nodes.extend(inner.iter());
                        }
                        continue;
                    }
                    _ => {}
                }
            }
        }
        let anchor_code = match trigger {
            Some(AuraNode::Element { children: t_children, props: t_props, events: t_events, .. }) => {
                // 裸文本 trigger（`dialog-trigger "Open"` → text prop）或单
                // Text 子 → 真 button（vue trigger 语义）。onclick 优先取
                // trigger 自带事件（parser 铸造的 `.__dlg_toggle_<n>` 或用户
                // 显式绑定），无则 no-op 防恐慌。
                let trigger_onclick = ["onclick", "onClick", "on_click"]
                    .iter()
                    .find_map(|k| t_events.get(*k))
                    .map(|h| self.handler_to_rust_closure_with_params(&h.handler, &h.params))
                    .unwrap_or_else(|| "|_| ()".to_string());
                if t_children.len() == 1 {
                    if let AuraNode::Text(AuraTextContent::Literal(s)) = &t_children[0] {
                        format!(
                            "View::button(\"{}\").on_click({}).build()",
                            s, trigger_onclick
                        )
                    } else {
                        self.generate_view_tree(&t_children[0])
                    }
                } else if t_children.is_empty() {
                    let label = Self::modal_child_label(t_props, t_children);
                    if label.is_empty() {
                        "auto_lang::ui::view::View::Empty".to_string()
                    } else {
                        format!(
                            "View::button(\"{}\").on_click({}).build()",
                            label, trigger_onclick
                        )
                    }
                } else {
                    let mut b = "View::row()".to_string();
                    for c in t_children {
                        b = format!("{}.child({})", b, self.generate_view_tree(c));
                    }
                    format!("{}.build()", b)
                }
            }
            _ => "auto_lang::ui::view::View::Empty".to_string(),
        };
        let mut panel = "View::col()".to_string();
        // PLAN-534: sheet/drawer 贴边族——drawer 竖向装饰把手为首子
        // （与解释器臂 convert_side_panel 同序）。
        let is_drawer = Self::drawer_root(tag);
        let side_val = if is_drawer || Self::sheet_root(tag) {
            Some(match props.get(if is_drawer { "direction" } else { "side" }) {
                Some(AuraPropValue::Expr(crate::ast::Expr::Str(s))) => s.to_string(),
                _ => "right".to_string(),
            })
        } else {
            None
        };
        if let Some(side) = side_val.as_deref() {
            if is_drawer && matches!(side, "top" | "bottom") {
                panel = format!("{}.child({})", panel, Self::drawer_handle_emission());
            }
        }
        for c in panel_nodes {
            panel = format!("{}.child({})", panel, self.generate_view_tree(c));
        }
        // PLAN-533 T7: dropdown-menu 族锚定菜单 chrome（shadcn
        // DropdownMenuContent 同款 p-1 紧凑档）;对话框族保持 w-96 模态卡。
        // PLAN-534: sheet/drawer 贴边族 chrome 与解释器臂 side_panel_chrome
        // 同串（横条 w-96 h-full / 纵条 w-full;drawer 竖向贴缘圆角）。
        let panel_chrome: String = match side_val.as_deref() {
            Some(side) => Self::side_panel_emission(side, is_drawer).0.to_string(),
            None => {
                if Self::dropdown_menu_root(tag) {
                    "w-44 bg-popover border border-border rounded-md shadow-md p-1 gap-1".to_string()
                } else {
                    "w-96 bg-background border border-border rounded-lg shadow-lg p-6 gap-4".to_string()
                }
            }
        };
        panel = format!("{}.style(\"{}\").build()", panel, panel_chrome);
        let open_expr = match props.get("open") {
            Some(AuraPropValue::Expr(crate::ast::Expr::Bool(b))) => b.to_string(),
            Some(AuraPropValue::Expr(e)) => self.ast_expr_to_rust(e),
            _ => "false".to_string(),
        };
        // PLAN-533 T6: dialog（可关闭）族铸造形态的 on_dismiss 折算
        // __dlg_close_N —— popover Panel 的 ESC/外点/锚点 dismiss 经此回流
        // update:open(false)。alert-dialog 族（shadcn 语义）与显式 open
        // 绑定的自管形态不接管（None）。
        if std::env::var("P533_DBG").is_ok() {
            eprintln!("[P533] tag={tag} open_prop={:?} role={:?} dismissable={}", props.get("open"), Self::modal_dialog_tag_role(tag), Self::dismissable_dialog_root(tag));
        }
        let on_dismiss_expr = if Self::modal_dialog_tag_role(tag) == Some("root")
            && Self::dismissable_dialog_root(tag)
        {
            match props.get("open") {
                Some(AuraPropValue::Expr(crate::ast::Expr::Dot(obj, field)))
                    if matches!(
                        obj.as_ref(),
                        crate::ast::Expr::Ident(b) if b.as_str() == "self" || b.as_str() == "."
                    ) && field.to_string().starts_with("__dlg_open_") =>
                {
                    let n = field.to_string().trim_start_matches("__dlg_open_").to_string();
                    format!("Some({}::__dlg_close_{})", self.current_msg_name(), n)
                }
                _ => "None".to_string(),
            }
        } else {
            "None".to_string()
        };
        let placement_path: String = match side_val.as_deref() {
            Some(side) => Self::side_panel_emission(side, is_drawer).1.to_string(),
            None if Self::dropdown_menu_root(tag) => {
                "auto_lang::ui::view::PopoverPlacement::BottomStart".to_string()
            }
            None => "auto_lang::ui::view::PopoverPlacement::Modal".to_string(),
        };
        format!(
            "View::Popover {{ anchor: auto_lang::ui::view::PopoverAnchor::Widget(Box::new({})), content: Box::new({}), placement: {}, open: {}, on_dismiss: {} }}",
            anchor_code, panel, placement_path, open_expr, on_dismiss_expr
        )
    }

    /// PLAN-027 T-02: 裸 `popover` 元素 → `View::Popover` 发射（shell pack
    /// 右键菜单/壁纸选择器/拖拽幽灵 ×9 消费面；解释侧同构 =
    /// aura_view_builder convert_popover）。双形态：
    /// - **坐标锚**（x/y prop 双全）：`PopoverAnchor::Point{x,y}`，children
    ///   全为面板内容，placement 缺省 BottomStart（contextmenu 落点约定）。
    /// - **首子锚**（缺省）：plain[0] = 锚件，plain[1..] = 面板列，placement
    ///   缺省 Bottom。shadcn popover-trigger/content 嵌套形态不在此消化
    ///   （shell pack 无此形态；误入按 plain 逐子直译，不静默丢件）。
    ///
    /// 其余对齐点：open 缺省 false（解释臂 __popover_toggle 自管开合为 VM
    /// 交互特性，codegen 面不合成——shell pack 九处全显式带 open）；
    /// ondismiss 取 events 桶（parser on* 升格同源），缺省 None（解释臂
    /// 合成 __popover_close 同为自管专属）；class 缺省给 shadcn
    /// PopoverContent chrome；有 class 缺 Width 类注入 `w-auto`（解释臂
    /// StyleClass::Width(Auto) 注入同语义——面板列被宿主宽拉满的 T27 修）。
    fn generate_bare_popover(
        &mut self,
        props: &std::collections::HashMap<String, AuraPropValue>,
        events: &std::collections::HashMap<String, AuraEvent>,
        children: &[AuraNode],
    ) -> String {
        let expr_str = |name: &str| -> Option<String> {
            props.get(name).and_then(|v| match v {
                AuraPropValue::Expr(e) => Some(self.ast_expr_to_rust(e)),
                _ => None,
            })
        };
        let (x_expr, y_expr) = (expr_str("x"), expr_str("y"));
        let point_anchor = x_expr.is_some() && y_expr.is_some();

        let open_expr = match props.get("open") {
            Some(AuraPropValue::Expr(crate::ast::Expr::Bool(b))) => b.to_string(),
            Some(AuraPropValue::Expr(e)) => self.ast_expr_to_rust(e),
            _ => "false".to_string(),
        };

        // placement 串映射（解释臂表同源）；字面量静态选臂，动态表达式发
        // 全臂 match（纯表达式形态，autodown heading 同款纪律）。缺省 =
        // 坐标锚 BottomStart / 锚件 Bottom（PLAN-528 W9 对齐语义）。
        let default_placement = if point_anchor { "BottomStart" } else { "Bottom" };
        let placement_lit = |s: &str| -> Option<&'static str> {
            match s.to_ascii_lowercase().as_str() {
                "bottom" => Some("Bottom"),
                "bottom-start" | "bottomstart" => Some("BottomStart"),
                "bottom-end" | "bottomend" => Some("BottomEnd"),
                "top" => Some("Top"),
                "top-start" | "topstart" => Some("TopStart"),
                "top-end" | "topend" => Some("TopEnd"),
                "left" => Some("Left"),
                "right" => Some("Right"),
                "pointer" => Some("Pointer"),
                _ => None,
            }
        };
        let placement_expr = match props.get("placement") {
            Some(AuraPropValue::Expr(crate::ast::Expr::Str(s))) => {
                let lit = placement_lit(s).unwrap_or(default_placement);
                format!("auto_lang::ui::view::PopoverPlacement::{lit}")
            }
            Some(AuraPropValue::Expr(e)) => {
                let k = self.ast_expr_to_rust(e);
                let arms = [
                    "bottom", "bottom-start", "bottom-end", "top", "top-start", "top-end",
                    "left", "right", "pointer",
                ]
                .iter()
                .filter_map(|p| {
                    placement_lit(p).map(|lit| format!("\"{p}\" => auto_lang::ui::view::PopoverPlacement::{lit},"))
                })
                .collect::<Vec<_>>()
                .join(" ");
                format!(
                    "match {k}.to_ascii_lowercase().as_str() {{ {arms} _ => auto_lang::ui::view::PopoverPlacement::{default_placement} }}"
                )
            }
            _ => format!("auto_lang::ui::view::PopoverPlacement::{default_placement}"),
        };

        // ondismiss：events 桶基名（.prevent 等后缀容忍）。
        let on_dismiss_expr = events
            .iter()
            .find(|(k, _)| k.as_str() == "ondismiss" || k.starts_with("ondismiss."))
            .map(|(_, ev)| {
                format!(
                    "Some({})",
                    self.handler_to_rust_direct_msg(&ev.handler, &ev.params)
                )
            })
            .unwrap_or_else(|| "None".to_string());

        // 面板列：内容子件（坐标锚 = 全部 children；锚件形态 = plain[1..]）。
        let content_nodes: Vec<&AuraNode> = if point_anchor {
            children.iter().collect()
        } else {
            children.iter().skip(1).collect()
        };
        let mut content = "View::col()".to_string();
        for c in &content_nodes {
            content = format!("{}.child({})", content, self.generate_view_tree(c));
        }
        let class_str = props
            .get("class")
            .or_else(|| props.get("style"))
            .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v {
                Some(s.trim().to_string())
            } else {
                None
            })
            .unwrap_or_default();
        let panel_style = if class_str.is_empty() {
            "w-72 bg-popover border border-border rounded-md shadow-md p-4".to_string()
        } else if class_str.split_whitespace().any(|t| t.starts_with("w-")) {
            class_str
        } else {
            format!("{class_str} w-auto")
        };
        content = format!("{}.style(\"{}\").build()", content, panel_style);

        let anchor_expr = if point_anchor {
            format!(
                "auto_lang::ui::view::PopoverAnchor::Point {{ x: ({} ) as f32, y: ({} ) as f32 }}",
                x_expr.unwrap(),
                y_expr.unwrap()
            )
        } else if children.is_empty() {
            "auto_lang::ui::view::PopoverAnchor::Widget(Box::new(auto_lang::ui::view::View::Empty))".to_string()
        } else {
            let anchor_code = self.generate_view_tree(&children[0]);
            format!("auto_lang::ui::view::PopoverAnchor::Widget(Box::new({anchor_code}))")
        };

        format!(
            "View::Popover {{ anchor: {}, content: Box::new({}), placement: {}, open: {}, on_dismiss: {} }}",
            anchor_expr, content, placement_expr, open_expr, on_dismiss_expr
        )
    }

    /// PLAN-533 T3: cancel/action/close 子件 → 按钮。variant 预设与解释器
    /// convert_button 同表（outline/primary + h-10 px-4 尺寸档）;onclick 走
    /// 既有消息派发形态（Msg::Variant 闭包），缺省 no-op 防恐慌。
    /// PLAN-571: button 的 variant/size preset 注入。单源 = ui::style::variants
    /// （VM 解释器臂共用）；user class 为字面量时前置合并，动态表达式无法静态
    /// 合并则不注入（与本臂此前行为一致，无回归）。非 button 原样返回。
    /// `ui` feature 关闭时无 preset 表可依，由文件尾部恒等孪生接管（调用点无条件编译）。
    #[cfg(feature = "ui")]
    fn with_button_preset<'a>(
        &self,
        tag: &str,
        props: &'a std::collections::HashMap<String, AuraPropValue>,
    ) -> std::borrow::Cow<'a, std::collections::HashMap<String, AuraPropValue>> {
        if tag != "button" {
            return std::borrow::Cow::Borrowed(props);
        }
        fn prop_str(v: Option<&AuraPropValue>) -> Option<&str> {
            match v {
                Some(AuraPropValue::Expr(crate::ast::Expr::Str(s))) => Some(s),
                _ => None,
            }
        }
        let variant = prop_str(props.get("variant")).unwrap_or_default();
        let size = prop_str(props.get("size")).unwrap_or_default();
        let mut preset = crate::ui::style::variants::button_variant_preset(variant).to_string();
        // Plan 414 R13: variant=icon 自带方形尺寸；缺省 size preset 会覆盖它并把
        // svg 内容区挤没 —— 未显式给 size 时置空。
        let size_preset = if variant == "icon" && size.is_empty() {
            String::new()
        } else {
            crate::ui::style::variants::button_size_preset(size).to_string()
        };
        if !size_preset.is_empty() {
            preset.push(' ');
            preset.push_str(&size_preset);
        }
        if preset.is_empty() {
            // PLAN-027 T-04：variant/size 已消费词汇仍剥除（拒绝门一致面）。
            let mut merged = props.clone();
            merged.remove("variant");
            merged.remove("size");
            return std::borrow::Cow::Owned(merged);
        }
        // 动态 class/style（非字面量）：无法静态合并，保持现状不注入。
        let literal_class = |v: Option<&AuraPropValue>| -> Option<String> {
            match v {
                Some(AuraPropValue::Expr(crate::ast::Expr::Str(s))) => Some(s.to_string()),
                _ => None,
            }
        };
        if props.get("style").map(|v| literal_class(Some(v))).unwrap_or(Some(String::new())).is_none()
            || props.get("class").map(|v| literal_class(Some(v))).unwrap_or(Some(String::new())).is_none()
        {
            // PLAN-027 T-04：动态 class/style 不注入（PLAN-571 文档化先例），
            // 但 variant/size 已消费词汇仍剥除。
            let mut merged = props.clone();
            merged.remove("variant");
            merged.remove("size");
            return std::borrow::Cow::Owned(merged);
        }
        let mut merged = props.clone();
        let key = if merged.contains_key("style") { "style" } else { "class" };
        let user = literal_class(merged.get(key)).unwrap_or_default();
        let combined = if user.trim().is_empty() { preset } else { format!("{} {}", preset, user) };
        merged.insert(
            key.to_string(),
            AuraPropValue::Expr(crate::ast::Expr::Str(auto_val::AutoStr::from(combined))),
        );
        // PLAN-027 T-04：preset 注入后剥除 variant/size（拒绝门一致面；
        // shell pack 按钮 ×38 携 variant）。
        merged.remove("variant");
        merged.remove("size");
        std::borrow::Cow::Owned(merged)
    }

    /// `ui` feature 关闭时的恒等孪生（保持调用点无条件编译）。范围与
    /// `ui` 孪生对齐 = **button-only**：button 的 variant/size 是 preset
    /// 词汇（无 ui preset 表可依 → 剥除防误入通用 builder 路径）；其余
    /// tag 恒等返回。PLAN-032 复审 R1 F-2：旧实现对全 tag 剥除——
    /// PLAN-641 起 tabs 亦持 variant（enclosed 词表），tf/tt 档（无
    /// ui-iced）下 `tabs variant:"enclosed"` 发射丢失；icon size 同族
    /// 受害面一并恢复。
    #[cfg(not(feature = "ui"))]
    fn with_button_preset<'a>(
        &self,
        tag: &str,
        props: &'a std::collections::HashMap<String, AuraPropValue>,
    ) -> std::borrow::Cow<'a, std::collections::HashMap<String, AuraPropValue>> {
        if tag != "button" {
            return std::borrow::Cow::Borrowed(props);
        }
        let mut merged = props.clone();
        merged.remove("variant");
        merged.remove("size");
        std::borrow::Cow::Owned(merged)
    }

    fn generate_modal_button(
        &mut self,
        props: &std::collections::HashMap<String, AuraPropValue>,
        events: &std::collections::HashMap<String, AuraEvent>,
        children: &[AuraNode],
        preset: &str,
    ) -> String {
        let label = Self::modal_child_label(props, children);
        let user_class = Self::modal_user_class(props);
        let class = if user_class.is_empty() {
            preset.to_string()
        } else {
            format!("{} {}", preset, user_class)
        };
        let onclick = ["onclick", "onClick", "on_click"]
            .iter()
            .find_map(|k| events.get(*k))
            .map(|handler| {
                self.handler_to_rust_closure_with_params(&handler.handler, &handler.params)
            })
            .unwrap_or_else(|| "|_| ()".to_string());
        format!(
            "View::button(\"{}\").style(\"{}\").on_click({}).build()",
            label, class, onclick
        )
    }

    /// Plan 644 follow-up: compile SVG elements into View::image_styled("svgdoc:<svg>...</svg>", style).
    /// Serializes the SVG DOM tree, converting dynamic property expressions into format! arguments.
    fn generate_svg_element(&self, node: &AuraNode) -> String {
        let (props, _) = match node {
            AuraNode::Element { props, children, .. } => (props, children),
            _ => return "View::Empty".to_string(),
        };

        let style_str = props
            .get("style")
            .or_else(|| props.get("class"))
            .and_then(|v| match v {
                AuraPropValue::Expr(crate::ast::Expr::Str(s)) => Some(s.to_string()),
                AuraPropValue::Expr(crate::ast::Expr::CStr(s)) => Some(s.to_string()),
                _ => None,
            })
            .unwrap_or_default();

        let mut template = String::new();
        let mut args = Vec::new();
        self.serialize_svg_node(node, true, &mut template, &mut args);

        if args.is_empty() {
            if style_str.is_empty() {
                format!("View::image(\"svgdoc:{}\")", template)
            } else {
                format!("View::image_styled(\"svgdoc:{}\", \"{}\")", template, style_str)
            }
        } else {
            let args_joined = args.join(", ");
            if style_str.is_empty() {
                format!("View::image(format!(\"svgdoc:{}\", {}))", template, args_joined)
            } else {
                format!("View::image_styled(format!(\"svgdoc:{}\", {}), \"{}\")", template, args_joined, style_str)
            }
        }
    }

    fn serialize_svg_node(
        &self,
        node: &AuraNode,
        is_root: bool,
        template: &mut String,
        args: &mut Vec<String>,
    ) {
        match node {
            AuraNode::Element { tag, props, children, .. } => {
                template.push('<');
                template.push_str(tag);

                let mut sorted_props: Vec<_> = props.iter().collect();
                sorted_props.sort_by_key(|(k, _)| *k);

                for (key, val) in sorted_props {
                    if is_root && (key == "style" || key == "class") {
                        continue;
                    }
                    if tag == "text" && key == "text" {
                        continue;
                    }

                    template.push(' ');
                    template.push_str(key);
                    template.push_str("=\\\"");

                    match val {
                        AuraPropValue::Expr(crate::ast::Expr::Str(s)) => {
                            let escaped = s.as_str()
                                .replace('&', "&amp;")
                                .replace('"', "&quot;")
                                .replace('<', "&lt;")
                                .replace('>', "&gt;")
                                .replace('{', "{{")
                                .replace('}', "}}");
                            template.push_str(&escaped);
                        }
                        AuraPropValue::Expr(crate::ast::Expr::CStr(s)) => {
                            let escaped = s.as_str()
                                .replace('&', "&amp;")
                                .replace('"', "&quot;")
                                .replace('<', "&lt;")
                                .replace('>', "&gt;")
                                .replace('{', "{{")
                                .replace('}', "}}");
                            template.push_str(&escaped);
                        }
                        AuraPropValue::Expr(crate::ast::Expr::Int(n)) => {
                            template.push_str(&n.to_string());
                        }
                        AuraPropValue::Expr(crate::ast::Expr::I64(n)) => {
                            template.push_str(&n.to_string());
                        }
                        AuraPropValue::Expr(crate::ast::Expr::U64(n)) => {
                            template.push_str(&n.to_string());
                        }
                        AuraPropValue::Expr(crate::ast::Expr::Float(f, _)) => {
                            template.push_str(&f.to_string());
                        }
                        AuraPropValue::Expr(expr) => {
                            template.push_str("{}");
                            args.push(self.ast_expr_to_rust(expr));
                        }
                        AuraPropValue::StyleBinding(_) => {}
                    }
                    template.push_str("\\\"");
                }

                let text_prop = if tag == "text" { props.get("text") } else { None };

                if children.is_empty() && text_prop.is_none() {
                    template.push_str("/>");
                } else {
                    template.push('>');
                    if let Some(val) = text_prop {
                        match val {
                            AuraPropValue::Expr(crate::ast::Expr::Str(s)) => {
                                let escaped = s.as_str()
                                    .replace('&', "&amp;")
                                    .replace('<', "&lt;")
                                    .replace('>', "&gt;")
                                    .replace('{', "{{")
                                    .replace('}', "}}");
                                template.push_str(&escaped);
                            }
                            AuraPropValue::Expr(expr) => {
                                template.push_str("{}");
                                args.push(self.ast_expr_to_rust(expr));
                            }
                            _ => {}
                        }
                    }
                    for child in children {
                        self.serialize_svg_node(child, false, template, args);
                    }
                    template.push_str("</");
                    template.push_str(tag);
                    template.push('>');
                }
            }
            AuraNode::Text(content) => match content {
                crate::aura::AuraTextContent::Literal(s) => {
                    let escaped = s.as_str()
                        .replace('&', "&amp;")
                        .replace('<', "&lt;")
                        .replace('>', "&gt;")
                        .replace('{', "{{")
                        .replace('}', "}}");
                    template.push_str(&escaped);
                }
                crate::aura::AuraTextContent::Interpolated { template: tpl, bindings } => {
                    let mut fmt = tpl.clone();
                    for name in bindings {
                        if let Some(start) = fmt.find("${") {
                            if let Some(end) = fmt[start..].find('}') {
                                fmt.replace_range(start..=start + end, "{}");
                            }
                        }
                        let stripped = name.trim_start_matches('.');
                        args.push(format!("self.{}", stripped));
                    }
                    template.push_str(&fmt);
                }
            },
            _ => {}
        }
    }

    fn generate_child_component(&self, tag: &str, props: &std::collections::HashMap<String, crate::aura::AuraPropValue>) -> String {
        let msg_name = self.current_msg_name();

        // Build constructor arguments from props (for loop children or prop sync).
        // PLAN-039 T-14（组件传型）：按 props 声明序（未注册回落字母序）
        // ——组件 new 签名按声明序，字母序调用 = 参数错位株。
        let mut constructor_args: Vec<String> = Vec::new();
        let keep_style = self.component_keeps_style_prop(tag);
        let mut sorted_keys: Vec<&String> = props.keys()
            .filter(|k| (keep_style && *k == "style") || (*k != "style" && *k != "class"))
            .collect();
        self.sort_prop_keys_by_decl_order(&mut sorted_keys, tag);
        for key in sorted_keys {
            if let crate::aura::AuraPropValue::Expr(expr) = &props[key] {
                // PLAN-039 T-14：实参按目标 prop 型强转。
                let raw = self.arg_to_rust(expr);
                constructor_args.push(self.coerce_arg_to_prop_type(&raw, tag, key));
            }
        }
        let args_str = constructor_args.join(", ");

        // Plan 371 L3: persistent children use self.<field>.view() with prop sync.
        // Since view() is &self (immutable), we clone the instance, sync props on
        // the clone, then call .view() on it. The clone carries the persistent
        // private state (editing/edit_title etc.); props (note/store) are refreshed.
        if self.is_persistent_child(tag) {
            let field = Self::child_field_name(tag);
            // Collect prop sync assignments for the cloned instance.
            // PLAN-039 T-14：声明序（同构造参数口径；字母序 = 重复/错位
            // 同步株——court_badge specified more than once 家族）。
            let mut prop_keys: Vec<&String> = props.keys()
                .filter(|k| *k != "style" && *k != "class")
                .collect();
            self.sort_prop_keys_by_decl_order(&mut prop_keys, tag);
            let mut sync_code = String::new();
            for key in &prop_keys {
                if let crate::aura::AuraPropValue::Expr(expr) = &props[*key] {
                    let prop_val = self.arg_to_rust(expr);
                    sync_code.push_str(&format!("__c.{} = {}; ", key, prop_val));
                }
            }
            // Store sync: persistent children may need the parent's store.
            let has_store = STORE_NAMES.with(|sn| !sn.borrow().is_empty());
            if has_store {
                sync_code.push_str(&format!("__c.store = self.store.clone(); "));
            }
            if sync_code.is_empty() {
                format!("self.{}.view().map_msg(|m| {}::{}(m))", field, msg_name, tag)
            } else {
                format!("{{ let mut __c = self.{}.clone(); {}__c.view().map_msg(|m| {}::{}(m)) }}",
                    field, sync_code, msg_name, tag)
            }
        } else {
            // Loop child (or legacy): temp-construct, sync fields, view, drop.
            let mut sync_fields: Vec<String> = self.state_types.keys()
                .filter(|name| {
                    let ty = self.state_types.get(*name).map(|s| s.as_str()).unwrap_or("");
                    !ty.starts_with("Vec<")
                })
                .cloned()
                .collect();
            let has_store = STORE_NAMES.with(|sn| !sn.borrow().is_empty());
            if has_store && !sync_fields.iter().any(|f| f == "store") {
                sync_fields.push("store".to_string());
            }
            if let Some(child_fields) = self.component_state_fields.get(tag) {
                for (f, _ty) in child_fields {
                    if !sync_fields.iter().any(|e| e == f) {
                        sync_fields.push(f.clone());
                    }
                }
            }

            if sync_fields.is_empty() {
                format!(
                    "{}::new({}).view().map_msg(|m| {}::{}(m))",
                    tag, args_str, msg_name, tag
                )
            } else {
                let mut code = format!("{{ let mut __{} = {}::new({}); ", tag.to_lowercase(), tag, args_str);
                for field in &sync_fields {
                    code.push_str(&format!("__{}.{} = self.{}.clone(); ", tag.to_lowercase(), field, field));
                }
                code.push_str(&format!("__{}.view().map_msg(|m| {}::{}(m)) }}", tag.to_lowercase(), msg_name, tag));
                code
            }
        }
    }

    /// Find parent state vars that should be synced to/from child component fields.
    /// Matches by name: if parent has state var "editing" and child component likely
    /// has a field "editing", they should be synced.
    /// Generate Rust call arguments from a Call's args, applying `.clone()`
    /// to `self.<String_field>` references to avoid move errors (E0507) when
    /// passing state by value into a function or store message constructor.
    /// Shared by store-call rewriting (L4037) and ordinary function calls (L4052).
    fn rust_call_args_with_clone(&self, call: &crate::ast::Call) -> Vec<String> {
        call.args.args.iter()
            .map(|a| {
                let arg_expr = a.get_expr();
                let expr = self.ast_expr_to_rust(&arg_expr);
                if expr.starts_with("self.") {
                    let field_name = &expr[5..];
                    // Don't clone for index access patterns like self.note["id"]
                    if !field_name.contains('[') {
                        if let Some(ty) = self.state_types.get(field_name) {
                            if ty == "String" {
                                return format!("{}.clone()", expr);
                            }
                        }
                    }
                }
                // PLAN-039 T-13（批次 E）：消息构造/调用的标识符与字段访问
                // 参数补 clone——VM 参数拷贝语义的编译等价（kanban cid
                // 双用 moved 株：`MoveCard(cid, ..)` 两处发送）。字面量/
                // 调用/索引形态不动；数值参数 clone 冗余但合法（Copy 型）。
                if matches!(arg_expr, crate::ast::Expr::Ident(_) | crate::ast::Expr::Dot(..))
                    && !expr.starts_with('"')
                    && !expr.contains('[')
                {
                    return format!("{}.clone()", expr);
                }
                expr
            })
            .collect()
    }

    fn find_sync_fields_for_child(&self, widget: &AuraWidget, child_name: &str) -> Vec<String> {
        let mut fields = Vec::new();
        for state in &widget.state_vars {
            let name = &state.name;
            fields.push(name.clone());
        }
        // Plan 371 Task 22c: sync the injected store composable field.
        let has_store = STORE_NAMES.with(|sn| !sn.borrow().is_empty());
        let is_store_itself = STORE_NAMES.with(|sn| {
            sn.borrow().values().any(|s| s.as_str() == widget.name)
        });
        if has_store && !is_store_itself && !fields.iter().any(|f| f == "store") {
            fields.push("store".to_string());
        }
        // Plan 371 Task 22c: hoist+sync the child component's OWN scalar state fields.
        if let Some(child_fields) = self.component_state_fields.get(child_name) {
            for (f, _ty) in child_fields {
                if !fields.iter().any(|e| e == f) {
                    fields.push(f.clone());
                }
            }
        }
        fields
    }

    /// Plan 371 L1: Detect whether a handler body mutates the store data that
    /// feeds child component props (e.g. `store.NewNote()` changes active_id;
    /// `store.active_id = i` or `store.notes = list_notes()` change the data
    /// backing a child's `note` prop). Used to decide whether to re-trigger the
    /// child's Init lifecycle. Replaces the old hardcoded `variant_name ==
    /// "NewNote"` check.
    ///
    /// Detects two patterns in the handler's AST statements:
    ///   1. Assignment to `store.active_id` / `store.notes` (or `active_id` /
    ///      `notes` if the store alias is implicit) — i.e. the parent updates
    ///      the data a child prop reads from.
    ///   2. Call to a mutating store method (e.g. `store.NewNote()`,
    ///      `store.TogglePin(...)`) — the store's own `.on` handler changes data.
    fn handler_mutates_store_data(&self, payload: &LogicPayload) -> bool {
        use crate::ast::{Expr, Stmt};
        use auto_val::Op;

        let stmts = match payload {
            LogicPayload::AstStmts(s) => s,
            _ => return false,
        };
        for stmt in stmts {
            if stmt_mutates_store_data(stmt) {
                return true;
            }
        }
        false
    }

    /// Find constructor args expression for child component instantiation in handler.
    /// This mirrors generate_child_component but for the handler context.
    fn find_constructor_args_for_child(&self, widget: &AuraWidget, child_name: &str) -> String {
        // Scan the view tree for the SPECIFIC child component reference
        if let Some(args) = self.extract_child_constructor_args(&widget.view_tree, child_name) {
            return args;
        }
        String::new()
    }

    /// Recursively extract child component constructor args from view tree.
    /// Plan 374: Only match the specific child_name to avoid wrong- component args.
    fn extract_child_constructor_args(&self, node: &AuraNode, child_name: &str) -> Option<String> {
        match node {
            AuraNode::Element { tag, props, children, .. } => {
                if tag == child_name && self.is_custom_widget(tag) {
                    return Some(self.build_sorted_constructor_args_for_element(props, tag));
                }
                for child in children {
                    if let Some(args) = self.extract_child_constructor_args(child, child_name) {
                        return Some(args);
                    }
                }
                None
            }
            AuraNode::Component { name, props, .. } => {
                // Only match if the component name equals the child we're looking for
                if name == child_name {
                    return Some(self.build_sorted_constructor_args_for_component(props, name));
                }
                None
            }
            AuraNode::ForLoop { body, .. } => {
                for child in body {
                    if let Some(args) = self.extract_child_constructor_args(child, child_name) {
                        return Some(args);
                    }
                }
                None
            }
            AuraNode::Conditional { then_body, else_body, .. } => {
                for child in then_body {
                    if let Some(args) = self.extract_child_constructor_args(child, child_name) {
                        return Some(args);
                    }
                }
                if let Some(else_nodes) = else_body {
                    for child in else_nodes {
                        if let Some(args) = self.extract_child_constructor_args(child, child_name) {
                            return Some(args);
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// PLAN-039 T-14（组件传型）：构造实参按目标 prop 型强转——降链
    /// 形态（as_str/as_i64）与 prop 型不符时重写（`scard["rank"].
    /// as_str()…` 传 `rank: i32` → as_i64）；型相符/无型不动。
    fn coerce_arg_to_prop_type(&self, arg: &str, component: &str, prop: &str) -> String {
        let kind = CHILD_PROP_TYPES.with(|m| {
            m.borrow()
                .get(component)
                .and_then(|tys| tys.get(prop).cloned())
        });
        let Some(kind) = kind else { return arg.to_string() };
        let has_str = arg.contains(".as_str().unwrap_or_default().to_string()");
        let has_int = arg.contains(".as_i64().unwrap_or(0) as i32")
            || arg.contains(".as_bool().unwrap_or(false)");
        match kind.as_str() {
            "int" if has_str => arg
                .replace(
                    ".as_str().unwrap_or_default().to_string()",
                    ".as_i64().unwrap_or(0) as i32",
                ),
            "str" if has_int && arg.contains(".as_i64().unwrap_or(0) as i32") => arg
                .replace(
                    ".as_i64().unwrap_or(0) as i32",
                    ".as_str().unwrap_or_default().to_string()",
                ),
            "bool" if has_str => arg
                .replace(
                    ".as_str().unwrap_or_default().to_string()",
                    ".as_bool().unwrap_or(false)",
                ),
            _ => arg.to_string(),
        }
    }

    /// PLAN-039 T-14（组件传型）：`style`/`class` 滤除门——组件把
    /// `style` 声明为真 prop 时（CardSuit (suit, style)）不滤（构造
    /// 参数缺位 = E0061 株）；未注册/非 prop 形态维持滤除（CSS 语义）。
    fn component_keeps_style_prop(&self, component: &str) -> bool {
        WIDGET_PROP_ORDERS.with(|po| {
            po.borrow()
                .get(component)
                .map_or(false, |order| order.iter().any(|p| p == "style"))
        })
    }

    /// PLAN-039 T-14（组件传型）：Element 形态（HashMap props）的键按
    /// 声明序排——未注册组件回落字母序（确定性保持）。
    fn sort_prop_keys_by_decl_order(&self, keys: &mut Vec<&String>, component: &str) {
        let order = WIDGET_PROP_ORDERS.with(|po| po.borrow().get(component).cloned());
        match order {
            Some(order) => {
                keys.sort_by_key(|k| order.iter().position(|p| p.as_str() == k.as_str()).unwrap_or(usize::MAX));
            }
            None => {
                keys.sort();
            }
        }
    }

    /// PLAN-039 T-14（批次 E，组件传型）：构造参数按 props 声明序排
    /// ——组件 new 签名按声明序发射，字母序调用 = 参数错位株（klondike
    /// CardFace argument #1 i32 missing/bool↔String 错序 ×N 根因；
    /// WIDGET_PROP_ORDERS 此前只写不读）。未注册组件保持字母序
    /// （freshness 字节对拍的确定性不变）；声明序外的 prop 稳定垫尾。
    fn order_component_props_by_decl<'a>(
        &self,
        mut entries: Vec<&'a (String, crate::ast::Expr)>,
        component: &str,
    ) -> Vec<&'a (String, crate::ast::Expr)> {
        let order = WIDGET_PROP_ORDERS.with(|po| po.borrow().get(component).cloned());
        match order {
            Some(order) => {
                entries.sort_by_key(|(k, _)| {
                    order.iter().position(|p| *p == **k).unwrap_or(usize::MAX)
                });
                entries
            }
            None => {
                entries.sort_by_key(|(k, _)| k.as_str());
                entries
            }
        }
    }

    /// Build sorted constructor args for an Element node (HashMap props).
    /// Sorts by declaration order when the widget's prop order is registered
    /// (PLAN-039 T-14：声明序——字母序调用 = 参数错位株); otherwise
    /// alphabetically by key for deterministic order.
    fn build_sorted_constructor_args_for_element(
        &self,
        props: &std::collections::HashMap<String, crate::aura::AuraPropValue>,
        widget_name: &str,
    ) -> String {
        let keep_style = self.component_keeps_style_prop(widget_name);
        let mut keys: Vec<&String> = props.keys()
            .filter(|k| (keep_style && *k == "style") || (*k != "style" && *k != "class"))
            .collect();
        self.sort_prop_keys_by_decl_order(&mut keys, widget_name);
        keys.iter()
            .filter_map(|k| {
                if let crate::aura::AuraPropValue::Expr(expr) = &props[*k] {
                    Some(self.arg_to_rust(expr))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Build sorted constructor args for a Component node (Vec props).
    /// Sorts alphabetically by key for deterministic order.
    fn build_sorted_constructor_args_for_component(
        &self,
        props: &[(String, crate::ast::Expr)],
        widget_name: &str,
    ) -> String {
        let entries: Vec<&(String, crate::ast::Expr)> = props.iter().collect();
        // PLAN-039 T-14：声明序（未注册回落字母序）。
        let ordered = self.order_component_props_by_decl(entries, widget_name);
        ordered.iter()
            .map(|(_, v)| self.arg_to_rust(v))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Convert an expression to a Rust constructor argument, adding .clone()
    /// for self-references to avoid E0507 (move out of shared reference).
    fn arg_to_rust(&self, expr: &crate::ast::Expr) -> String {
        let rust_expr = self.ast_expr_to_rust(expr);
        if rust_expr.starts_with("self.") || rust_expr.starts_with("self[") {
            format!("{}.clone()", rust_expr)
        } else {
            rust_expr
        }
    }

    /// Convert AURA condition to Rust expression
    fn convert_condition(&self, condition: &str) -> String {
        let result = condition.trim().to_string();

        // Replace state-ref dots like ".notes" → "self.notes", but NOT method call dots
        // like ".len()" or ".to_string()". A state-ref dot is one where the previous
        // character is NOT alphanumeric/underscore (i.e. it's at a word boundary).
        // PLAN-036 T-02：后继字符兼收 `_`——`__` 前缀状态名（__wm_*/
        // __desktop_* 家族）原只认字母后继漏网，裸 `.` 落产物成语法错
        //（真编译门首漏）。
        let bytes = result.as_bytes();
        let mut output = String::new();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'.'
                && i + 1 < bytes.len()
                && (bytes[i + 1].is_ascii_alphabetic() || bytes[i + 1] == b'_')
            {
                // Check if this dot is a method call (preceded by ident char)
                let is_method_call = i > 0
                    && (bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_' || bytes[i - 1] == b')');
                if is_method_call {
                    // This is var.field — check if var is a Value-type loop variable
                    // Look backwards to find the identifier before the dot
                    let ident_end = i;
                    let mut ident_start = i;
                    for j in (0..i).rev() {
                        if bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_' {
                            ident_start = j;
                        } else {
                            break;
                        }
                    }
                    if ident_start < ident_end {
                        let var_name = &result[ident_start..ident_end];
                        if self.value_loop_vars.contains(var_name) {
                            // Find the field name after the dot
                            let mut field_end = i + 1;
                            for j in (i + 1)..bytes.len() {
                                if bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_' {
                                    field_end = j + 1;
                                } else {
                                    break;
                                }
                            }
                            let field_name = &result[i + 1..field_end];
                            // Replace var.field with bracket access, converting the result
                            // Remove the var.field from output and replace with bracket access
                            let output_var_name = var_name.to_string();
                            let bracket_access = self.value_field_access(&output_var_name, field_name);
                            // Remove the already-pushed var name and replace with bracket access
                            output.truncate(output.len() - var_name.len());
                            output.push_str(&bracket_access);
                            i = field_end;
                            continue;
                        }
                        // Plan 374: prop/svar.field → self.prop.field (e.g., note.pinned → self.note.pinned)
                        // But for Value-typed props, use bracket access: self.note["pinned"]
                        if self.value_prop_names.contains(var_name) {
                            // Find the field name after the dot
                            let mut field_end = i + 1;
                            for j in (i + 1)..bytes.len() {
                                if bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_' {
                                    field_end = j + 1;
                                } else {
                                    break;
                                }
                            }
                            let field_name = &result[i + 1..field_end];
                            output.truncate(output.len() - var_name.len());
                            output.push_str(&self.value_field_access(&format!("self.{}", var_name), field_name));
                            i = field_end;
                            continue;
                        }
                        if self.prop_names.contains(var_name) || self.state_types.contains_key(var_name) {
                            output.truncate(output.len() - var_name.len());
                            output.push_str(&format!("self.{}", var_name));
                            // Push the dot and advance past it (don't reset i, or we loop forever)
                            output.push('.');
                            i += 1;
                            continue;
                        }
                    }
                    output.push('.');
                } else {
                    output.push_str("self.");
                }
            } else {
                output.push(bytes[i] as char);
            }
            i += 1;
        }

        // Fix double self references
        output = output.replace("self.self.", "self.");

        // Plan 374: Fix .contains(self.field) → .contains(self.field.as_str())
        // because str::contains expects impl Pattern, not String.
        loop {
            let pos = match output.find(".contains(self.") {
                Some(p) => p,
                None => break,
            };
            let close = match output[pos..].find(')') {
                Some(c) => pos + c,
                None => break,
            };
            // Check if .as_str() is already there (don't double-apply)
            let segment = &output[pos..close];
            if segment.contains(".as_str") {
                break;
            }
            output.insert_str(close, ".as_str()");
        }

        // PLAN-036 T-02：`.contains(format!(...))` Pattern 修正（handler 体
        // 同款——条件串消费面）。
        fix_contains_string_pattern_for_ui(&mut output);

        // PLAN-036 T-02：条件串 Eq/Neq 跨型降串（ast_expr_to_rust 主臂
        // normalize_eq_neq_compare 的文本族——convert_condition 为字符串
        // 管线无 AST，按生成形判别 + state_types 类型感知）。
        output = self.normalize_condition_compares(&output);

        output
    }

    /// PLAN-036 T-02：条件串（convert_condition 产物）的跨型比较降串——
    /// 一侧 String 类（.as_str() 形/串字面量/String 态字段）另一侧数值类
    /// （`as i32)` 收尾）→ 数值侧补 `.to_string()`；对侧 Value 布尔访问 →
    /// 改 as_str 形。VM 宽松比较语义的编译等价（同 normalize_eq_neq_
    /// compare 注）。
    fn normalize_condition_compares(&self, cond: &str) -> String {
        let is_word = |c: u8| {
            c.is_ascii_alphanumeric()
                || c == b'_'
                || c == b'.'
                || c == b'"'
                || c == b'['
                || c == b']'
                || c == b'('
                || c == b')'
                || c == b':'
        };
        let stringy = |s: &str| -> bool {
            if s.contains(".as_str()") || s.starts_with('"') || s.contains("format!(") {
                return true;
            }
            // self.<field> / 裸 <field>：String 态字段。
            let name = s.strip_prefix("self.").unwrap_or(s);
            self.state_types.get(name).map_or(false, |ty| ty == "String")
        };
        let mut out = cond.to_string();
        for op in ["==", "!="] {
            let mut cursor = 0usize;
            loop {
                let Some(rel) = out[cursor..].find(op) else { break };
                let pos = cursor + rel;
                let bytes = out.as_bytes();
                // 左段：先回跳空格（生成文本 ` == ` 带空格）再词段回扫。
                let mut ls = pos;
                while ls > 0 && bytes[ls - 1] == b' ' {
                    ls -= 1;
                }
                while ls > 0 && is_word(bytes[ls - 1]) {
                    ls -= 1;
                }
                let left = out[ls..pos].trim_end().to_string();
                let mut re = pos + 2;
                while re < bytes.len() && is_word(bytes[re]) {
                    re += 1;
                }
                // 右段跨空格续读（`(0) as i32` 形含空格）——至表达式边界
                //（&&/||/{/}/, 或非词字符）。
                loop {
                    let mut j = re;
                    while j < bytes.len() && bytes[j] == b' ' {
                        j += 1;
                    }
                    if j >= bytes.len() {
                        break;
                    }
                    let two = &out[j..(j + 2).min(out.len())];
                    if two == "&&" || two == "||" || bytes[j] == b'{' || bytes[j] == b'}' || bytes[j] == b',' {
                        break;
                    }
                    if !is_word(bytes[j]) {
                        break;
                    }
                    re = j;
                    while re < bytes.len() && is_word(bytes[re]) {
                        re += 1;
                    }
                }
                let right = out[pos + 2..re].trim().to_string();
                let mut new_left = None;
                let mut new_right = None;
                let num_state = |s: &str| -> bool {
                    let name = s.strip_prefix("self.").unwrap_or(s);
                    matches!(
                        self.state_types.get(name).map(|t| t.as_str()),
                        Some("i32" | "i64" | "u32" | "u64" | "f32" | "f64")
                    )
                };
                if stringy(&left) && !stringy(&right) {
                    if right.contains(".as_bool().unwrap_or(false)") {
                        new_right = Some(right.replace(
                            ".as_bool().unwrap_or(false)",
                            ".as_str().unwrap_or_default().to_string()",
                        ));
                    } else if right.ends_with("as i32)") || num_state(&right) {
                        new_right = Some(format!("{}.to_string()", right));
                    }
                } else if stringy(&right) && !stringy(&left) {
                    if left.contains(".as_bool().unwrap_or(false)") {
                        new_left = Some(left.replace(
                            ".as_bool().unwrap_or(false)",
                            ".as_str().unwrap_or_default().to_string()",
                        ));
                    } else if left.ends_with("as i32)") || num_state(&left) {
                        new_left = Some(format!("{}.to_string()", left));
                    }
                }
                match (new_left, new_right) {
                    (None, None) => {
                        cursor = pos + 2;
                    }
                    (nl, nr) => {
                        let l = nl.unwrap_or_else(|| left.clone());
                        let r = nr.unwrap_or_else(|| right.clone());
                        let done = ls + l.len() + 3 + r.len();
                        out = format!("{}{} {} {}{}", &out[..ls], l, op, r, &out[re..]);
                        cursor = done;
                    }
                }
            }
        }
        out
    }

    /// Resolve a dotted path like "note.tags" or "store.notes" into proper Rust
    /// field access, using bracket syntax for Value-type props.
    /// e.g., "note.tags" → `self.note["tags"]` if note is a Value prop
    ///       "store.notes" → `self.store.notes` if store is not Value
    fn resolve_dotted_path(&self, path: &str) -> String {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return format!("self.{}", path);
        }
        // First component determines the base
        let first = parts[0];
        let mut result = if self.value_prop_names.contains(first) || self.needs_index_access(first) {
            format!("self.{}", first)
        } else {
            format!("self.{}", first)
        };
        // Remaining components: use bracket access if base is Value, else dot access
        for &part in &parts[1..] {
            // Check if the current result refers to a Value type
            let is_value = self.value_prop_names.contains(first)
                || self.needs_index_access(first);
            if is_value {
                result = format!("{}[\"{}\"]", result, part);
            } else {
                // Check if this is a store field that's Vec<Value>
                let store_check = format!("{}.{}", first, part);
                if self.state_types.contains_key(part)
                    && self.state_types.get(part).map(|t| t.starts_with("Vec<")).unwrap_or(false)
                {
                    result = format!("{}.{}", result, part);
                } else {
                    result = format!("{}.{}", result, part);
                }
            }
        }
        result
    }

    /// Resolve a collection name from an Index target expression.
    /// Handles patterns: Ident("notes"), Dot(self, "notes"), Dot(Dot(self, store), "notes")
    /// Returns (collection_path, is_self_prefixed).
    fn resolve_collection_name(&self, target: &crate::ast::Expr) -> (Option<String>, bool) {
        use crate::ast::Expr;
        match target {
            // Simple ident: notes
            Expr::Ident(name) => {
                let s = name.as_str();
                let resolved = s.trim_start_matches('.');
                (Some(resolved.to_string()), s.starts_with('.') || resolved != s)
            }
            // self.notes → Dot(Ident("self"), "notes")
            Expr::Dot(obj, field) => {
                let field_str = field.as_str();
                if let Expr::Ident(inner) = obj.as_ref() {
                    if inner.as_str() == "self" || inner.as_str() == ".self" {
                        return (Some(field_str.to_string()), true);
                    }
                    // self.store.notes → Dot(Dot(Ident("self"), "store"), "notes")
                    // The collection is "store.notes"
                    if inner.as_str() == "self" {
                        // Already handled above
                    }
                }
                // Compound: self.store.notes → extract full path
                let full = self.expr_to_simple_string(target);
                if !full.is_empty() {
                    let trimmed = full.trim_start_matches("self.");
                    return (Some(trimmed.to_string()), full.starts_with("self."));
                }
                (None, false)
            }
            _ => {
                let full = self.expr_to_simple_string(target);
                if !full.is_empty() {
                    let trimmed = full.trim_start_matches("self.");
                    return (Some(trimmed.to_string()), full.starts_with("self."));
                }
                (None, false)
            }
        }
    }

    /// Convert a simple expression to a dotted string path (best effort).
    fn expr_to_simple_string(&self, expr: &crate::ast::Expr) -> String {
        use crate::ast::Expr;
        match expr {
            Expr::Ident(name) => name.as_str().to_string(),
            Expr::Dot(obj, field) => {
                let base = self.expr_to_simple_string(obj);
                if base.is_empty() {
                    String::new()
                } else {
                    format!("{}.{}", base, field.as_str())
                }
            }
            _ => String::new(),
        }
    }

    /// Check if a tag is a custom widget reference (uppercase first letter, not a known tag)
    fn is_custom_widget(&self, tag: &str) -> bool {
        // Known tags that should not be treated as custom widgets
        const KNOWN_TAGS: &[&str] = &[
            "col", "column", "row", "grid", "scroll", "container", "center",
            "button", "input", "textarea", "code_editor", "checkbox", "toggle", "select", "option", "link",
            "text", "label", "span", "h1", "h2", "h3", "h4", "h5", "h6", "p",
            "table", "thead", "tbody", "tr", "th", "td", "tree", "tree_item",
            "tabs", "tab",
            "modal", "tooltip",
            "slider", "radio", "radiogroup",
            "progress", "badge", "spinner",
            "card", "avatar",
            "image", "icon", "imagesurface", "image-surface", "image_surface", "ImageSurface",
            "divider", "spacer",
            "for", "if",
        ];
        // Custom widgets start with uppercase letter
        tag.chars().next().map_or(false, |c| c.is_uppercase()) && !KNOWN_TAGS.contains(&tag)
    }

    /// Plan 371 L3: Check if a child component is a single-instance (not in a
    /// for-loop) and thus eligible to be a persistent struct field.
    fn is_persistent_child(&self, child_name: &str) -> bool {
        // PLAN-039 T-14（组件传型）：持久字段名撞 props/state 字段时不升
        // 持久——CardFace 的 prop `court_badge: str` 与子组件 CourtBadge
        // 的持久字段名同形，撞名后 `self.court_badge` 是 String 而组件
        // 访问（.badge/.store/.view()）全断（8 错连锁株）；降临时实例
        // （view 每次构造，语义等价——状态本就不持有）。
        let field = Self::child_field_name(child_name);
        if self.prop_names.contains(&field) || self.state_types.contains_key(&field) {
            return false;
        }
        !self.loop_child_components.contains(child_name)
    }

    /// Plan 371 L3: Convert a PascalCase component name to snake_case for the
    /// persistent field name (e.g. EditorPanel → editor_panel, NavTree → nav_tree).
    fn child_field_name(child_name: &str) -> String {
        let mut result = String::new();
        for (i, c) in child_name.chars().enumerate() {
            if c.is_uppercase() && i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        }
        result
    }

    /// Default heading styles for h1-h6 tags (consistent with aura_view_builder & vue.rs)
    fn heading_default_style(tag: &str) -> Option<&'static str> {
        match tag {
            "h1" => Some("text-4xl font-bold"),
            "h2" => Some("text-3xl font-bold"),
            "h3" => Some("text-xl font-semibold"),
            "h4" => Some("text-lg font-semibold"),
            "h5" => Some("text-base font-semibold"),
            "h6" => Some("text-sm font-semibold"),
            _ => None,
        }
    }

    /// Plan 450 / 019 批次三: AutoDown Heading 面板(level prop)的默认样式。
    /// 与 aura_view_builder 面板臂的 h1..h6 同源样式(text-primary + 页边距,
    /// plan 409 §8 一致);level 钳位 1..6(palette_map.at panelHeading 同款)。
    fn autodown_heading_style(level: i32) -> &'static str {
        match level.clamp(1, 6) {
            1 => "text-4xl font-bold tracking-tight text-primary mb-4",
            2 => "text-3xl font-bold tracking-tight text-primary mt-8 mb-4",
            3 => "text-xl font-semibold text-primary mb-3",
            4 => "text-lg font-semibold mb-2",
            5 => "text-base font-semibold mb-1",
            _ => "text-sm font-semibold mb-1",
        }
    }

    /// 面板 prop 值 → Rust 表达式。text_styled 按值收 content,self.* 字段
    /// 引用加 .clone() 防 E0507(markdown 特例同款处理);非 Expr 值落空串。
    fn autodown_panel_prop_expr(&self, v: &AuraPropValue) -> String {
        if let AuraPropValue::Expr(expr) = v {
            let e = self.ast_expr_to_rust(expr);
            if e.starts_with("self.") {
                return format!("{}.clone()", e);
            }
            return e;
        }
        "\"\".to_string()".to_string()
    }

    /// 面板子节点 → 发射表达式:单子直接用,多子包 col;无子返回 None
    /// (由调用方决定 content prop 降级路径)。
    fn autodown_panel_children_col(&mut self, children: &[AuraNode]) -> Option<String> {
        if children.is_empty() {
            return None;
        }
        if children.len() == 1 {
            return Some(self.generate_view_tree(&children[0]));
        }
        let mut col = "View::col()".to_string();
        for c in children {
            col = format!("{}.child({})", col, self.generate_view_tree(c));
        }
        Some(format!("{}.build()", col))
    }

    /// Map tag to View builder function
    fn tag_to_view_fn(&self, tag: &str) -> &'static str {
        match tag {
            // Layout
            "col" | "column" => "col",
            "row" => "row",
            // PLAN-027 T-04 扩面：taskbar = row（解释臂 aura_view_builder.rs
            // :1711 convert_row 同源——任务栏横向条，缺省 col 会纵向堆叠）。
            "taskbar" => "row",
            "grid" => "grid",
            // PLAN-026 T-02：scroll 走 display 降级臂（View::scrollable），
            // 断裂映射移除。
            // PLAN-022 T-06 复盘(2026-09-19):div 回退 col——027 T-04 的
            // div→container 对多子流式 div(如 app 载具内容区 relative
            // w-full flex-1)语义错:ViewContainerBuilder 无弹性类映射
            // (flex-1/h-full 失效,布局塌缩)且 .child 为替换语义(多子
            // 静默丢子)。container 标签保留一参包装形态(N 子装 col);
            // 与 VM 解释臂"container|div 同律"的分歧归 027 面备案。
            "container" => "container",
            "div" => "col",
            "center" => "center",

            // Content
            "button" => "button",
            "input" => "input",
            "textarea" => "textarea",
            "code_editor" | "codeEditor" | "codeeditor" => "code_editor",
            "checkbox" => "checkbox",
            "toggle" => "toggle",
            "select" => "select",
            "option" => "option",
            // PLAN-026 T-02：link/a 不再映射（Element 形态走 display 降级
            // 臂 text_styled；AuraNode::Link 有专属臂）——断裂映射移除，
            // 未达臂形态落 `_ => "col"` 兜底。

            // Typography
            "text" | "label" | "span" => "text",
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => "text",
            "p" => "text",

            // Data
            "table" => "table",
            "thead" => "thead",
            "tbody" => "tbody",
            "tr" => "tr",
            "th" => "th",
            "td" => "td",
            "tree" => "col",
            "tree_item" => "col",

            // Navigation
            // PLAN-032 T-03（D2）：tabs/tab 断裂映射移除——tabs 走上方
            // generate_view_tree 专属臂（View::tabs 折叠构造）；组外裸
            // tab/残余形态落 `_ => "col"` 兜底（026 T-02 link/a 先例）。

            // Overlay
            "modal" => "modal",
            "tooltip" => "tooltip",

            // Form
            "slider" => "slider",
            "radio" => "radio",
            "radiogroup" => "radiogroup",

            // Feedback
            "progress" => "progress",
            "spinner" => "spinner",

            // Display
            "avatar" => "avatar",

            // Media
            "image" => "image",

            // Utility
            "divider" => "divider",
            "spacer" => "spacer",

            _ => "col",
        }
    }

    /// Add property to builder
    fn add_prop_to_builder(&self, builder: &str, key: &str, value: &AuraPropValue) -> String {
        match value {
            AuraPropValue::Expr(expr) => {
                let value_str = self.ast_expr_to_rust(expr);
                match key {
                    "class" | "className" => {
                        // Plan 346: for Literal strings, use .style("literal");
                        // for dynamic expressions (If/Binary/StateRef), use
                        // .style(expr) without quotes (expr already produces String).
                        if matches!(expr, crate::ast::Expr::Str(_)) {
                            let class_str = value_str.trim_matches('"')
                                .trim_end_matches(".to_string()")
                                .trim_matches('"');
                            if class_str.is_empty() {
                                builder.to_string()
                            } else {
                                format!("{}.style(\"{}\")", builder, class_str)
                            }
                        } else {
                            format!("{}.style({}.as_str())", builder, value_str)
                        }
                    }
                    "style" => {
                        // Plan 346: same as class — Literal uses quoted string,
                        // dynamic expressions use unquoted Rust expression.
                        if matches!(expr, crate::ast::Expr::Str(_)) {
                            let style_str = value_str.trim_matches('"')
                                .trim_end_matches(".to_string()")
                                .trim_matches('"');
                            if style_str.is_empty() {
                                builder.to_string()
                            } else {
                                format!("{}.style(\"{}\")", builder, style_str)
                            }
                        } else {
                            format!("{}.style({}.as_str())", builder, value_str)
                        }
                    }
                    "padding" => format!("{}.padding({})", builder, value_str),
                    "spacing" => format!("{}.spacing({})", builder, value_str),
                    // PLAN-039 T-04（台账 M7-c「缺口即发现即修」）：`key:` =
                    // Vue 轨 reconciliation 提示词（kanban/klondike for 循环
                    // 惯用）——a2r 编译轨无 diff/reconcile 面，认知且双轨
                    // 同弃（onmouseenter 先例同款 parity 锚），非拒绝面。
                    "key" => builder.to_string(),
                    // PLAN-039 T-06（发现臂同款）：`aria-label`/ARIA 族 =
                    // a11y 提示词（tetris app.at 惯用）——a2r 编译轨无
                    // ARIA 面，认知且双轨同弃（key/onmouseenter parity）。
                    "aria-label" | "aria-labelledby" | "aria-describedby"
                    | "aria-hidden" | "aria-expanded" | "aria-controls"
                    | "role" | "tabindex" | "alt" => builder.to_string(),
                    _ => {
                        // PLAN-027 T-04: 显式拒绝门（设计 §3a-a4）——未知
                        // prop 由静默丢弃改编译期错（防"看似编译过实缺件"
                        // 的 shell 生成物）。编译期错形态 = 表达式位
                        // compile_error! 块（generate_view_tree 为 String
                        // 管线，生成期 hard error 需全链 Result 化——
                        // 成本不成比例，且编译期错同样拦截产物入库）。
                        let msg = format!(
                            "a2r codegen: prop `{key}` not in the recognized vocabulary (PLAN-027 explicit rejection gate)"
                        );
                        format!("{{ std::compile_error!(\"{msg}\"); unreachable!() }}")
                    }
                }
            }
            AuraPropValue::StyleBinding(bindings) => {
                // For Rust, generate conditional style application.
                // Each binding produces a conditional: if cond { "style" } else { "" }
                // Uses .with_style() with Style::parse() for safe string construction.
                let class_conditions: Vec<String> = bindings.iter()
                    .map(|b| {
                        let cond = self.ast_expr_to_rust(&b.condition);
                        format!("if {} {{ \"{}\" }} else {{ \"\" }}", cond, b.style_name)
                    })
                    .collect();
                if class_conditions.is_empty() {
                    builder.to_string()
                } else if class_conditions.len() == 1 {
                    // Single condition: if cond { "completed" } else { "" } is &str
                    format!("{}.style({})", builder, class_conditions[0])
                } else {
                    // Multiple conditions: build concatenated string.
                    // Rust if-expr returns &str, we need to combine them.
                    // Use nested format!: format!("{} {}", c1, c2) then .as_str()
                    // Actually, just construct Style directly from parts
                    let fmt_str = class_conditions.iter().map(|_| "{}").collect::<Vec<_>>().join(" ");
                    // Each condition is an `if ... { &str } else { &str }` expression
                    // format!() needs owned values for interpolation, but &str works fine
                    let args = class_conditions.join(", ");
                    let combined = format!("auto_lang::ui::style::Style::parse(&format!(\"{}\", {})).unwrap_or_default()", fmt_str, args);
                    format!("{}.with_style({})", builder, combined)
                }
            }
        }
    }

    /// PLAN-039 D1-A（§5.1 定案记录）：button `ondblclick` = MouseArea
    /// 包裹降级。View::Button 无双击事件槽（view.rs:1650-1657 仅
    /// onclick/on_right_click），MouseArea.on_double_click 为既有原语
    /// （Plan 496 M5「桌面图标双击启动」，view.rs:1034-1045）——iced/
    /// RqProjector/queue 三消费端零涟漪；布局代价 = 一层嵌套盒。
    /// klondike 自动收牌双击（app.at ondblclick ×8）为首发载体；非
    /// button tag 的 ondblclick 不经此臂（拒绝门响亮拒——I1）。
    fn wrap_button_double_click(&self, built: String, dbl: Option<&AuraEvent>) -> String {
        match dbl {
            None => built,
            Some(ev) => {
                let direct = self.handler_to_rust_direct_msg(&ev.handler, &ev.params);
                format!(
                    "View::MouseArea {{ content: Box::new({built}), on_enter: None, on_exit: None, \
                     on_double_click: Some({direct}), on_click: None, on_context_menu: None, \
                     on_release: None, on_move: None, logical_extent: None, style: None }}"
                )
            }
        }
    }

    /// Add event to builder
    fn add_event_to_builder(&self, builder: &str, event: &str, aura_event: &AuraEvent) -> String {
        let handler_fn = self.handler_to_rust_closure_with_params(&aura_event.handler, &aura_event.params);
        match event {
            "onclick" | "onClick" | "on_click" => {
                format!("{}.on_click({})", builder, handler_fn)
            }
            "oncontextmenu" | "oncontextmenu.prevent" => {
                format!("{}.on_right_click({})", builder, handler_fn)
            }
            "onchange" | "onChange" | "oninput" | "onInput" => {
                format!("{}.on_change({})", builder, handler_fn)
            }
            // PLAN-039 D3-A（§5.1 定案记录，用户确认 2026-09-21）：HTML5
            // in-app drag 家族显式 not-yet——a2r 编译轨无 drag 事件面板
            // （完整 DnD 语义另立，P039 债在册）。compile_error 带 P039
            // 债指针，区别于通用未知事件臂（kanban board.at
            // ondragover.prevent ×3 = 生成门预期唯一诚实红）；禁静默
            // no-op（I1 红线）。
            "ondragover" | "ondragover.prevent" | "ondragstart" | "ondragstart.prevent"
            | "ondragend" | "ondragend.prevent" | "ondrop" | "ondrop.prevent"
            | "ondragenter" | "ondragenter.prevent" | "ondragleave" | "ondragleave.prevent" => {
                let msg = format!(
                    "a2r codegen: event `{event}` (HTML5 drag family) not yet supported in a2r compiled mode (PLAN-039 D3-A explicit not-yet; in-app DnD tracked as P039 debt)"
                );
                format!("{{ std::compile_error!(\"{msg}\"); unreachable!() }}")
            }
            // PLAN-027 T-04: 认知且双轨同弃层——View IR 布局件无 hover
            // 事件槽，解释臂 set_layout_events（aura_view_builder.rs:1328
            // 只收 onclick/oncontextmenu）同弃。switcher.at row 的
            // onmouseenter 两轨一致落空（parity 锚）——非拒绝面。
            "onmouseenter" | "onmouseleave" | "onhover" | "onhoverout" => builder.to_string(),
            _ => {
                // PLAN-027 T-04: 显式拒绝门（同 add_prop_to_builder 臂注）
                // ——未知事件不再静默丢弃（shell 依赖的 ondismiss 等此前
                // 无译无警；视图事件槽丢失 = 交互缺件编译不可见）。
                let msg = format!(
                    "a2r codegen: event `{event}` not in the recognized vocabulary (PLAN-027 explicit rejection gate)"
                );
                format!("{{ std::compile_error!(\"{msg}\"); unreachable!() }}")
            }
        }
    }

    /// Convert handler pattern to Rust closure
    #[allow(dead_code)]
    fn handler_to_rust_closure(&self, handler: &str) -> String {
        let variant = self.extract_variant_name(handler);
        let msg_name = self.current_msg_name();
        format!("|_| {}::{}", msg_name, variant)
    }

    /// Convert handler pattern to Rust closure with parameters
    fn handler_to_rust_closure_with_params(&self, handler: &str, params: &[String]) -> String {
        let variant = self.extract_variant_name(handler);
        let msg_name = self.current_msg_name();
        if params.is_empty() {
            format!("|_| {}::{}", msg_name, variant)
        } else {
            // Convert dot access on Value-type vars to index access
            let converted_params: Vec<String> = params.iter()
                .map(|p| self.convert_param_value_access(p, &variant))
                .collect();
            format!("|_| {}::{}({})", msg_name, variant, converted_params.join(", "))
        }
    }

    /// Convert handler pattern to a direct Rust message expression (no closure wrapper).
    /// Used for fields like Checkbox.on_toggle which is Option<M>, not Option<impl Fn() -> M>.
    fn handler_to_rust_direct_msg(&self, handler: &str, params: &[String]) -> String {
        let variant = self.extract_variant_name(handler);
        let msg_name = self.current_msg_name();
        if params.is_empty() {
            format!("{}::{}", msg_name, variant)
        } else {
            let converted_params: Vec<String> = params.iter()
                .map(|p| self.convert_param_value_access(p, &variant))
                .collect();
            format!("{}::{}({})", msg_name, variant, converted_params.join(", "))
        }
    }

    /// Convert dot access in param expressions for Value-type variables
    /// e.g., "note.id" → "note[\"id\"].as_i64().unwrap_or(0) as i32" for i32 payloads
    fn convert_param_value_access(&self, param: &str, variant_name: &str) -> String {
        // Check for patterns like "varname.field" or "varname.field.subfield"
        let parts: Vec<&str> = param.split('.').collect();
        if parts.len() >= 2 {
            let var_name = parts[0];
            if self.value_loop_vars.contains(var_name) || self.needs_index_access(var_name) {
                let field = parts[1..].join(".");
                // Check payload type to determine conversion
                let payload_ty = self.message_variants.iter()
                    .find(|v| v.name == variant_name)
                    .and_then(|v| v.payload.first())
                    .map(|t| self.auto_type_to_rust(t));
                return match payload_ty.as_deref() {
                    Some("i32") => format!("{}[\"{}\"].as_i64().unwrap_or(0) as i32", var_name, field),
                    Some("i64") => format!("{}[\"{}\"].as_i64().unwrap_or(0)", var_name, field),
                    Some("String") => format!("{}[\"{}\"].as_str().unwrap_or_default().to_string()", var_name, field),
                    Some("bool") => format!("{}[\"{}\"].as_bool().unwrap_or(false)", var_name, field),
                    _ => format!("{}[\"{}\"]", var_name, field),
                };
            }
        }
        // Plan 374: String literal args need .to_string() when variant expects String
        let payload_ty = self.message_variants.iter()
            .find(|v| v.name == variant_name)
            .and_then(|v| v.payload.first())
            .map(|t| self.auto_type_to_rust(t));
        if payload_ty.as_deref() == Some("String") && param.starts_with('"') && !param.contains(".to_string()") {
            return format!("{}.to_string()", param);
        }
        // PLAN-039 T-13（批次 E）：typed 循环变量的字段访问参数补 clone
        // ——闭包内 move 出借用（kanban `SelectBoard(b.id)`：boards 迭代
        // b:&BoardDef，b.id String move 出 Fn 闭包 ×1 株）。数值参数
        // clone 冗余但合法（Copy 型）。
        // PLAN-039 T-14：Value 循环变量的裸名参数按目标 payload 降链
        // ——`PickCat(c)`：c ∈ Vec<Value> 迭代（Value），variant 收
        // String → __at_str 收口（int payload → __at_num）。
        if !param.contains('.')
            && !param.starts_with('"')
            && !param.contains('[')
            && (self.value_loop_vars.contains(param) || self.needs_index_access(param))
        {
            return match payload_ty.as_deref() {
                Some("String") => format!("__at_str(&({}))", param),
                Some("i32") | Some("i64") => format!("__at_num(&({}))", param),
                Some("bool") => format!("({}).as_bool().unwrap_or(false)", param),
                _ => format!("{}.clone()", param),
            };
        }
        if !param.starts_with('"')
            && !param.contains('[')
            && param
                .chars()
                .next()
                .map_or(false, |c| c.is_ascii_alphabetic() || c == '_')
            && payload_ty
                .as_deref()
                .map_or(false, |t| !matches!(t, "i32" | "i64" | "u32" | "u64" | "f32" | "f64" | "bool"))
        {
            // 纯标识符（`PickCat(c)` 循环变量 String）与点链同 clone
            // ——Fn 闭包 move 株（数值参数 clone 冗余但合法）。
            return format!("{}.clone()", param);
        }
        param.to_string()
    }

    /// Extract variant name from pattern (e.g., "Msg::Inc" or ".Inc" -> "Inc")
    fn extract_variant_name(&self, pattern: &str) -> String {
        if pattern.starts_with('.') {
            // .SelectNote(i) → SelectNote
            let after_dot = &pattern[1..];
            if let Some(paren) = after_dot.find('(') {
                after_dot[..paren].to_string()
            } else {
                after_dot.to_string()
            }
        } else if let Some(variant) = pattern.split("::").last() {
            variant.to_string()
        } else {
            pattern.to_string()
        }
    }

    /// Plan 346: Extract the payload parameter name from a handler pattern.
    /// `.SelectNote(i)` → `i`. The pattern stored by the aura extractor may
    /// be just "SelectNote" (without the arg). In that case, scan the handler
    /// body for the first identifier that's likely the payload parameter.
    /// Falls back to `_payload` if nothing found.
    fn extract_payload_name(&self, pattern: &str) -> String {
        if let Some(start) = pattern.find('(') {
            if let Some(end) = pattern.rfind(')') {
                let inner = &pattern[start + 1..end].trim();
                if !inner.is_empty() {
                    return inner.to_string();
                }
            }
        }
        // Fallback: common parameter names for single-int payloads.
        "i".to_string()
    }

    /// Generate handler body from LogicPayload
    /// PLAN-039 T-12：pub 化——auto-man 侧 store 文件级自由 fn（kanban
    /// `fn esc` 株）的函数体翻译复用此链（join/postprocess 全含）。
    pub fn generate_handler_body(&self, payload: &LogicPayload) -> String {
        let raw = match payload {
            LogicPayload::AstStmts(stmts) => {
                // handler 臂块 = 语句位置:块尾恒补 `;`(PLAN-634 T-03)。
                self.join_stmt_block(stmts, ";\n                ")
            }
            LogicPayload::Bytecode(_) => {
                "// bytecode handler".to_string()
            }
        };
        // Plan 374: Post-process handler body to fix Value array/bool operations.
        let mut body = self.postprocess_handler_body(&raw);
        // PLAN-020 T-02: UI handler 臂补数字转换下降——`.to_int()` 在 UI 语料
        // 有语义(499 M3 先例),但 a2r 生成的 f32/f64 无此方法;trans/rust.rs
        // 的 fix_numeric_conversion_methods 只覆盖非 UI 管线,此处对 handler
        // 体补同一 `(expr as i32)` 改写(x.to_int() → (x as i32))。
        fix_numeric_conversion_methods_for_ui(&mut body);
        // PLAN-025 T-07：VM math.* 内建降级（003-converter handler 真源
        // math.round 先例——解释态 VM 直算，a2r 臂需落到 Rust f64 方法）。
        lower_math_builtins_for_ui(&mut body);
        // PLAN-036 T-02：`.contains(format!(...))` Pattern 修正（str 收
        // &str——launching ack 求差 `,__wm_running.contains(","+x+",")` 先例）。
        fix_contains_string_pattern_for_ui(&mut body);
        // PLAN-036 T-02：String 局部收 Value 元素赋值降串（face_apps src）。
        fix_string_local_value_element_assign(&mut body);
        body
    }

    /// Fix known codegen patterns for Value type operations in handler bodies.
    fn postprocess_handler_body(&self, body: &str) -> String {
        let mut result = body.to_string();

        // Fix 1: .as_str().unwrap_or_default().to_string().push(X)
        // → Replace the push call on a Value-typed field with JSON array operation.
        // Pattern: self.note["field"].as_str().unwrap_or_default().to_string().push(ARG)
        // The ARG can contain nested parens, so we find it manually.
        while let Some(pos) = result.find(".as_str().unwrap_or_default().to_string().push(") {
            // Find the start of .push( — we need the base expression before .as_str()
            let push_paren = pos + ".as_str().unwrap_or_default().to_string().push(".len() - 1; // position of '('
            // Find matching close paren for .push(
            let bytes = result.as_bytes();
            let mut depth = 0;
            let mut pe = push_paren;
            let mut found = false;
            for j in push_paren..bytes.len() {
                match bytes[j] {
                    b'(' => depth += 1,
                    b')' => {
                        depth -= 1;
                        if depth == 0 { pe = j; found = true; break; }
                    }
                    _ => {}
                }
            }
            if !found { break; }
            let arg = &result[push_paren + 1..pe];
            // Find the base expression: walk backwards from pos to find self.note["tags"]
            // Look for pattern: word.word[...]
            let mut base_start = pos;
            for k in (0..pos).rev() {
                let c = bytes[k];
                if c.is_ascii_alphanumeric() || c == b'_' || c == b'.' || c == b'[' || c == b']' || c == b'"' {
                    base_start = k;
                } else {
                    break;
                }
            }
            let base = &result[base_start..pos];
            let replacement = format!(
                "{{ let mut __a = {}.as_array().cloned().unwrap_or_default(); __a.push(serde_json::json!({})); {} = serde_json::Value::Array(__a); }}",
                base.trim(), arg.trim(), base.trim()
            );
            result = format!("{}{}{}", &result[..base_start], replacement, &result[pe + 1..]);
        }

        // Fix 2: .as_str().unwrap_or_default().to_string().iter()
        //   → .as_array().into_iter().flatten()
        // Pattern: X.as_str().unwrap_or_default().to_string().iter()
        result = result.replace(
            ".as_str().unwrap_or_default().to_string().iter()",
            ".as_array().into_iter().flatten()"
        );

        // Fix 2b: #[api] Vec<String> argument marshalling.
        // update_tags(id, tags) expects `tags: Vec<String>` (both merged and
        // split clients), but value_field_access emits the tags field as a single
        // String (`.as_str().unwrap_or_default().to_string()`). Rewrite that
        // specific argument into a Vec<String> built from the JSON array.
        // We scan each update_tags(...) call and replace the String-marshalled
        // tags arg, leaving other tags usages (iter/contains/push) untouched.
        let needle = r#"["tags"].as_str().unwrap_or_default().to_string())"#;
        let marshalled = r#"["tags"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<String>>()).unwrap_or_default())"#;
        let mut search = 0;
        while let Some(rel) = result[search..].find("update_tags(") {
            let pos = search + rel;
            // Find the matching close paren of this update_tags(...) call.
            let bytes = result.as_bytes();
            let mut depth = 0i32;
            let mut end = pos;
            for (j, &b) in bytes[pos..].iter().enumerate() {
                match b {
                    b'(' => depth += 1,
                    b')' => { depth -= 1; if depth == 0 { end = pos + j; break; } }
                    _ => {}
                }
            }
            let call_end = end;
            // Only the substring within this call is a candidate.
            if result[pos..=call_end].contains(needle) {
                let before = &result[..pos];
                let this_call = &result[pos..=call_end];
                let after = &result[call_end + 1..];
                let new_call = this_call.replace(needle, marshalled);
                let gained = new_call.len();
                result = format!("{}{}{}", before, new_call, after);
                search = pos + gained;
            } else {
                search = call_end + 1;
            }
        }

        // Fix 3: Value bool toggle assignment
        // Pattern: self.notes[idx]["field"].as_bool().unwrap_or(false) = !(self.notes[idx]["field"].as_bool().unwrap_or(false))
        // This can't be assigned to — replace with a json! assignment.
        // Detect: ["field"].as_bool().unwrap_or(false) = !
        // Pattern: BASE.as_bool().unwrap_or(false) = !(BASE.as_bool().unwrap_or(false))
        // Replace: BASE = serde_json::json!(!(BASE.as_bool().unwrap_or(false)))
        if result.contains(".as_bool().unwrap_or(false) = !(") {
            // Find and replace using manual parsing instead of regex
            while let Some(pos) = result.find(".as_bool().unwrap_or(false) = !(") {
                // Find the base expression before .as_bool()
                let base_end = pos;
                let bytes = result.as_bytes();
                let mut base_start = base_end;
                for k in (0..base_end).rev() {
                    let c = bytes[k];
                    if c.is_ascii_alphanumeric() || c == b'_' || c == b'.' || c == b'[' || c == b']' || c == b'"' || c == b' ' {
                        base_start = k;
                    } else {
                        break;
                    }
                }
                let base = result[base_start..base_end].trim();
                // Find the matching close of !(BASE.as_bool().unwrap_or(false))
                let excl_start = pos + ".as_bool().unwrap_or(false) = ".len();
                // Find the matching ) for !(
                let mut depth = 0;
                let bytes = result.as_bytes();
                let mut close = excl_start;
                for j in excl_start..bytes.len() {
                    match bytes[j] {
                        b'(' => depth += 1,
                        b')' => { depth -= 1; if depth == 0 { close = j; break; } }
                        _ => {}
                    }
                }
                let replacement = format!("{} = serde_json::json!(!({}.as_bool().unwrap_or(false)))", base, base);
                result = format!("{}{}{}", &result[..base_start], replacement, &result[close+1..]);
            }
        }

        // Fix 4: if X != None { ... }; — use .is_some() to avoid type issues
        result = result.replace(" != None {", ".is_some() {");
        result = result.replace(" == None {", ".is_none() {");
        // E0317 fix: add `else {}` for `if note.is_some() { ... };`
        // The if-body is an #[api] call (e.g. update_note) which returns () in
        // both merged and split modes (PUT is fire-and-forget). An empty else
        // branch returns (), matching the if-body. (Previously `else { None }`
        // was injected, producing a `()` vs `Option<_>` mismatch.)
        if result.contains("if note.is_some() {") {
            // Find the pattern and add else {}
            let pattern = "if note.is_some() {";
            let mut search_start = 0;
            while let Some(pos) = result[search_start..].find(pattern) {
                let abs_pos = search_start + pos;
                // Find matching close brace
                let bytes = result.as_bytes();
                let brace_start = abs_pos + pattern.len() - 1; // position of {
                let mut depth = 0;
                let mut brace_end = brace_start;
                for j in brace_start..bytes.len() {
                    match bytes[j] {
                        b'{' => depth += 1,
                        b'}' => { depth -= 1; if depth == 0 { brace_end = j; break; } }
                        _ => {}
                    }
                }
                // Check if there's already an else after brace_end
                let after = &result[brace_end+1..];
                if !after.trim_start().starts_with("else") {
                    // Insert `else {}` (unit-typed else branch matches the
                    // statement-context if-body).
                    result.insert_str(brace_end + 1, " else {}");
                }
                search_start = brace_end + 1;
            }
        }

        // Fix 5: note.title where note is Option<Value>
        // Pattern: note.field → note.as_ref().and_then(|n| n.get("field")).and_then(|v| v.as_str()).unwrap_or_default()
        // Use as_str().unwrap_or_default() to return &str which converts to String for API calls.
        if result.contains("note.is_some()") {
            result = result.replace(
                "note.title",
                "note.as_ref().and_then(|n| n.get(\"title\")).and_then(|v| v.as_str()).unwrap_or_default().to_string()"
            );
            result = result.replace(
                "note.body",
                "note.as_ref().and_then(|n| n.get(\"body\")).and_then(|v| v.as_str()).unwrap_or_default().to_string()"
            );
        }

        // Fix 6: tg != t where tg is &Value and t is String → compare via as_str
        result = result.replace("tg != t", "tg.as_str().unwrap_or_default() != t.as_str()");

        result
    }

    /// 语句位置块发射(PLAN-634 T-03/#18 根修):join 后把**块尾语句恒补
    /// `;`**。Aura 源无分号惯例,块尾语句原本成为 Rust 块的尾表达式——
    /// 值返回调用落进 if 块尾即 `if c { api_fn() }` → rustc E0308
    /// (auto-term #18 实证);`let` 块尾同理(旧特判只补了 let,此处一并
    /// 取代)。值消费位置(表达式块,`ast_expr_to_rust` 的 If/Block 臂)
    /// 不走本函数,尾表达式语义不受影响。
    fn join_stmt_block(&self, stmts: &[crate::ast::Stmt], sep: &str) -> String {
        let bodies: Vec<String> = stmts
            .iter()
            .map(|s| self.ast_stmt_to_rust(s))
            .filter(|b| !b.trim().is_empty())
            .collect();
        let mut joined = bodies.join(sep);
        if let Some(last) = stmts.last() {
            let emitted = !matches!(
                last,
                crate::ast::Stmt::Comment(_) | crate::ast::Stmt::EmptyLine(_)
            );
            if emitted && !joined.trim().is_empty() {
                joined.push(';');
            }
        }
        joined
    }

    /// Convert a crate::ast::Stmt to Rust code (for on-handler bodies)
    ///
    /// 语句转换不带尾分号——块尾是否补 `;` 由 [`Self::join_stmt_block`]
    /// 按"语句位置/表达式位置"统一裁定(PLAN-634 T-03)。
    fn ast_stmt_to_rust(&self, stmt: &crate::ast::Stmt) -> String {
        match stmt {
            crate::ast::Stmt::Store(store) => {
                let name = store.name.as_str();
                let resolved = if name.starts_with('.') { &name[1..] } else { name };
                let mut value = self.ast_expr_to_rust(&store.expr);
                // Let/Const are local variables — use let binding
                // Var/Field are state variables — but only if they exist in state_types
                match store.kind {
                    crate::ast::StoreKind::Let | crate::ast::StoreKind::Const => {
                        // Check if value is an index into a Vec<Value> (e.g., todos[idx])
                        // If so, use &mut borrow so that mutations to todo.field affect the array
                        if let crate::ast::Expr::Index(target, _idx) = &store.expr {
                            // Plan 407 R4a: resolve collection name from Ident or Dot patterns.
                            let coll_stripped: Option<&str> = match target.as_ref() {
                                crate::ast::Expr::Ident(collection) => {
                                    let s = collection.as_str();
                                    Some(s.strip_prefix('.').unwrap_or(s))
                                }
                                crate::ast::Expr::Dot(inner, field) => {
                                    if matches!(inner.as_ref(), crate::ast::Expr::Ident(_)) {
                                        // PLAN-039 T-13：任意 Ident 基座
                                        // （r.cards 局部集合 → "cards"，
                                        // state/local 判定随后收口）。
                                        Some(field.as_str())
                                    } else { None }
                                }
                                _ => None,
                            };
                            // PLAN-039 T-13（批次 E）：局部集合（typed Vec<Card>
                            // 如 r.cards——api 返回局部不在 state_types）元素
                            // 索引同 clone（Var 分支同款；let c = r.cards[i]
                            // move 株）。
                            let state_vec = coll_stripped
                                .and_then(|coll| self.state_types.get(coll))
                                .map(|ty| ty.starts_with("Vec<"))
                                .unwrap_or(false);
                            let local_index_rhs = coll_stripped
                                .map(|coll| {
                                    self.array_locals.contains(coll) || self.value_locals.contains(coll)
                                })
                                .unwrap_or(false);
                            if state_vec || local_index_rhs {
                                return format!("let mut {} = {}.clone()", name, value);
                            }
                        }
                        format!("let {} = {}", name, value)
                    }
                    crate::ast::StoreKind::Var => {
                        // `var x = expr` → mutable local binding
                        // Plan 407: if indexing into a Vec<Value>, clone the element.
                        // PLAN-039 T-12（批次 E，E-D3 第一轨后半）：显式声明
                        // 型（`var ic1 str = .apps_icons[ai]`）赋值点强转——
                        // 声明型局部收到 Value 型 RHS 时包访问器（str →
                        // as_str 降串、int → __at_num），VM 宽松降链的编译
                        // 等价；无类型局部的类型格是 T-14 域。
                        let coerced = self.coerce_typed_local_init(&store.ty, &store.expr, &value);
                        if let crate::ast::Expr::Index(target, _idx) = &store.expr {
                            let coll_stripped: Option<&str> = match target.as_ref() {
                                crate::ast::Expr::Ident(c) => Some(c.as_str().strip_prefix('.').unwrap_or(c.as_str())),
                                crate::ast::Expr::Dot(inner, field) => {
                                    if matches!(inner.as_ref(), crate::ast::Expr::Ident(_)) {
                                        // PLAN-039 T-13：任意 Ident 基座
                                        // （r.cards 局部集合 → "cards"，
                                        // state/local 判定随后收口）。
                                        Some(field.as_str())
                                    } else { None }
                                }
                                _ => None,
                            };
                            let state_vec = coll_stripped
                                .and_then(|coll| self.state_types.get(coll))
                                .map(|t| t.starts_with("Vec<"))
                                .unwrap_or(false);
                            // PLAN-039 T-13（批次 E）：局部集合（typed Vec<Card>
                            // 如 r.cards——api 返回局部不在 state_types）的元素
                            // 索引同样 clone——`let c = r.cards[i]` move 出
                            // Vec<Card> 索引位 ×株（VM 拷贝语义；Value 元素
                            // clone 语义同）。
                            let local_index_rhs = coll_stripped
                                .map(|coll| self.array_locals.contains(coll) || self.value_locals.contains(coll))
                                .unwrap_or(false);
                            if state_vec || local_index_rhs {
                                // 强转形态自带所有权（as_str→to_string 拷贝）；
                                // 未强转维持 clone（&mut 借用语义，Plan 407）。
                                let rhs = if coerced == value {
                                    format!("{}.clone()", value)
                                } else {
                                    coerced
                                };
                                return format!("let mut {} = {}", name, rhs);
                            }
                        }
                        if self.state_types.contains_key(resolved) {
                            // Auto-coerce int → String when assigning to a String field
                            if self.state_types.get(resolved).map_or(false, |ty| ty == "String")
                                && !self.ast_expr_is_string(&store.expr)
                            {
                                value = format!("{}.to_string()", value);
                            }
                            // PLAN-039 T-13（批次 E）：非 Copy 态字段的局部
                            // 字段访问 RHS 补 clone——`self.cards = r.cards`
                            // move 后 r.meta/r.cards 复用株（VM 拷贝语义的
                            // 编译等价）。数值/布尔态（Copy）不动。
                            let state_non_copy = self
                                .state_types
                                .get(resolved)
                                .map_or(false, |ty| {
                                    !matches!(ty.as_str(), "i32" | "i64" | "u32" | "u64" | "f32" | "f64" | "bool")
                                });
                            if state_non_copy
                                && matches!(
                                    &store.expr,
                                    crate::ast::Expr::Ident(_) | crate::ast::Expr::Dot(..)
                                )
                                && !value.contains('[')
                                && !value.starts_with('"')
                            {
                                value = format!("{}.clone()", value);
                            }
                            format!("self.{} = {}", resolved, value)
                        } else {
                            // PLAN-026 T-03 配套: 登记 handler 局部
                            // List<int> 变量(索引消费窄化依据;同名再绑定
                            // 非 int-list 清出防跨声明误伤)。
                            if matches!(&store.ty, crate::ast::Type::List(inner)
                                if matches!(**inner, crate::ast::Type::Int))
                            {
                                self.handler_int_list_vars.borrow_mut().insert(name.to_string());
                            } else {
                                self.handler_int_list_vars.borrow_mut().remove(&name.to_string());
                            }
                            // Local mutable var in handler context.
                            // PLAN-039 T-14（E-D3 第二轨）：Value 联合局部
                            // 的标量字面量初始式 json! 化——`var card = 999`
                            // + 后续 Value 赋值（card = items[i]）的联合株。
                            let rhs = if self.value_locals.contains(name)
                                && !self.declared_locals.contains_key(name)
                                && matches!(
                                    &store.expr,
                                    crate::ast::Expr::Int(_)
                                        | crate::ast::Expr::I64(_)
                                        | crate::ast::Expr::Str(_)
                                        | crate::ast::Expr::Bool(_)
                                        | crate::ast::Expr::Float(_, _)
                                        | crate::ast::Expr::Double(_, _)
                                ) {
                                format!(
                                    "serde_json::json!({})",
                                    self.ast_expr_to_rust_no_to_string(&store.expr)
                                )
                            } else {
                                coerced
                            };
                            format!("let mut {} = {}", name, rhs)
                        }
                    }
                    _ => {
                        // If name is a known state var, use self. prefix
                        if self.state_types.contains_key(resolved) {
                            // Auto-coerce int → String when assigning to a String field
                            if self.state_types.get(resolved).map_or(false, |ty| ty == "String")
                                && !self.ast_expr_is_string(&store.expr)
                            {
                                value = format!("{}.to_string()", value);
                            }
                            format!("self.{} = {}", resolved, value)
                        } else {
                            // Otherwise it's a local var in handler context
                            format!("let {} = {}", name, value)
                        }
                    }
                }
            }
            crate::ast::Stmt::Expr(expr) => {
                self.ast_expr_to_rust(expr)
            }
            crate::ast::Stmt::If(if_stmt) => {
                let mut parts = Vec::new();
                for (i, branch) in if_stmt.branches.iter().enumerate() {
                    let cond = self.ast_expr_to_rust(&branch.cond);
                    // 分支体 = 语句位置:块尾恒补 `;`(#18:值调用块尾缺分号
                    // → E0308;旧 let 特判由 join_stmt_block 取代)。
                    let body_str = self.join_stmt_block(&branch.body.stmts, "; ");
                    if i == 0 {
                        parts.push(format!("if {} {{ {} }}", cond, body_str));
                    } else {
                        parts.push(format!("else if {} {{ {} }}", cond, body_str));
                    }
                }
                if let Some(else_body) = &if_stmt.else_ {
                    let body_str = self.join_stmt_block(&else_body.stmts, "; ");
                    parts.push(format!("else {{ {} }}", body_str));
                }
                parts.join(" ")
            }
            crate::ast::Stmt::For(for_stmt) => {
                // 循环体 = 语句位置:块尾恒补 `;`(join_stmt_block 统一规则)。
                let body_str = self.join_stmt_block(&for_stmt.body.stmts, "; ");
                match &for_stmt.iter {
                    crate::ast::Iter::Named(name) => {
                        // for todo in .todos { ... } → for todo in self.todos.iter() { ... }
                        // If body mutates loop var (value_loop_var), use iter_mut()
                        let iter_name = name.as_str();
                        let collection = self.ast_expr_to_rust(&for_stmt.range);
                        let needs_mut = self.value_loop_vars.contains(iter_name);
                        let iter_method = if needs_mut { "iter_mut" } else { "iter" };
                        let mut_prefix = if needs_mut { "mut " } else { "" };
                        format!("for {}{} in {}.{}() {{ {} }}", mut_prefix, iter_name, collection, iter_method, body_str)
                    }
                    crate::ast::Iter::Cond => {
                        // for i >= 0 { ... } → while i >= 0 { ... }
                        let cond = self.ast_expr_to_rust(&for_stmt.range);
                        format!("while {} {{ {} }}", cond, body_str)
                    }
                    crate::ast::Iter::Ever => {
                        // loop { ... }
                        format!("loop {{ {} }}", body_str)
                    }
                    crate::ast::Iter::Indexed(idx, name) => {
                        // for i, todo in .todos { ... } → for (i, todo) in self.todos.iter().enumerate() { ... }
                        let collection = self.ast_expr_to_rust(&for_stmt.range);
                        format!("for ({}, {}) in {}.iter().enumerate() {{ {} }}", idx.as_str(), name.as_str(), collection, body_str)
                    }
                    crate::ast::Iter::Destructured(key, val) => {
                        let collection = self.ast_expr_to_rust(&for_stmt.range);
                        format!("for ({}, {}) in {}.iter() {{ {} }}", key.as_str(), val.as_str(), collection, body_str)
                    }
                    crate::ast::Iter::Call(_) => {
                        // Fallback for Call-based iterators
                        let collection = self.ast_expr_to_rust(&for_stmt.range);
                        format!("for __item in {}.iter() {{ {} }}", collection, body_str)
                    }
                }
            }
            // Plan 407: support Break, Continue, Return in handler bodies.
            crate::ast::Stmt::Break => "break".to_string(),
            crate::ast::Stmt::Continue => "continue".to_string(),
            crate::ast::Stmt::Return(expr) => {
                // Plan 407: bare return (Expr::Nil) → `return;` not `return Value::Null`
                if matches!(expr.as_ref(), crate::ast::Expr::Nil) {
                    "return".to_string()
                } else {
                    format!("return {}", self.ast_expr_to_rust(expr))
                }
            }
            crate::ast::Stmt::Block(body) => {
                // 裸块语句 = 语句位置:块尾恒补 `;`(join_stmt_block)。
                format!("{{ {} }}", self.join_stmt_block(&body.stmts, "; "))
            }
            crate::ast::Stmt::Comment(_) => String::new(),
            crate::ast::Stmt::EmptyLine(_) => String::new(),
            _ => format!("/* unhandled stmt */"),
        }
    }

    /// PLAN-039 T-12（批次 E，E-D3 第一轨后半）：显式声明型局部的赋值点
    /// 强转——str/int 声明收到 Value 型 RHS 时包访问器（str → as_str 降串
    /// 拷贝、int → __at_num），VM 宽松降链的编译等价。返回原串 = 无需
    /// 强转（RHS 非 Value 或声明无类型）。
    fn coerce_typed_local_init(
        &self,
        ty: &crate::ast::Type,
        expr: &crate::ast::Expr,
        value: &str,
    ) -> String {
        use crate::ast::Type;
        if !self.expr_is_value_typed(expr) {
            return value.to_string();
        }
        match ty {
            Type::StrFixed(_) | Type::StrOwned | Type::StrSlice | Type::CStrLit => {
                format!("({}).as_str().unwrap_or_default().to_string()", value)
            }
            Type::Int | Type::I64 | Type::Uint | Type::U64 => {
                format!("__at_num(&({}))", value)
            }
            _ => value.to_string(),
        }
    }

    /// PLAN-039 T-12（批次 E）：表达式是否 Value 型——数值使用点包裹
    /// （__at_num）与串接降串（__at_str）的判据。覆盖：Value 局部/循环
    /// 变量、Vec<Value> 元素索引、Value 态字段/prop、`.store.<value 字段>`。
    fn expr_is_value_typed(&self, expr: &crate::ast::Expr) -> bool {
        use crate::ast::Expr;
        // Vec<Value> 元素：Index(target, _)
        if let Expr::Index(target, _) = expr {
            if let Some(coll) = self.resolve_expr_name(target) {
                if self
                    .state_types
                    .get(&coll)
                    .map_or(false, |t| t == "Vec<serde_json::Value>")
                    // PLAN-039 T-12：无类型集合/调用结果局部——元素 Value。
                    || self.array_locals.contains(&coll)
                    || self.value_locals.contains(&coll)
                {
                    return true;
                }
            }
            // .store.<vec 字段>[i] —— store 表中 Vec<Value> 的元素
            if let Expr::Dot(inner, field) = target.as_ref() {
                let hit_store = match inner.as_ref() {
                    Expr::Ident(n) => n.as_str() == "store",
                    _ => false,
                };
                if hit_store
                    && self
                        .store_field_rust_type(field.as_str())
                        .map_or(false, |t| t == "Vec<serde_json::Value>")
                {
                    return true;
                }
            }
            if let Expr::Ident(name) = target.as_ref() {
                let s = name.as_str();
                if s.starts_with(".store.") {
                    let field = &s[".store.".len()..];
                    if !field.contains('.')
                        && self
                            .store_field_rust_type(field)
                            .map_or(false, |t| t == "Vec<serde_json::Value>")
                    {
                        return true;
                    }
                }
            }
        }
        // `.store.<value 字段>`（Dot 真链/拍平 Ident）
        if let Expr::Dot(inner, field) = expr {
            if let Expr::Ident(n) = inner.as_ref() {
                if n.as_str() == "store" && self.store_field_is_value(field.as_str()) {
                    return true;
                }
            }
        }
        // PLAN-039 T-14：Value 集合的 pop() 调用——赋值点强转判据
        // （`var cx int = stack.pop()` 株）。
        if let Expr::Call(call) = expr {
            if let Expr::Dot(obj, m) = call.name.as_ref() {
                if m.as_str() == "pop" && self.receiver_is_value_collection(obj) {
                    return true;
                }
            }
        }
        if let Expr::Ident(name) = expr {
            let s = name.as_str();
            if s.starts_with(".store.") {
                let field = &s[".store.".len()..];
                if !field.contains('.') && self.store_field_is_value(field) {
                    return true;
                }
            }
        }
        // 名字形态：局部/循环/字段/prop
        if let Some(name) = self.resolve_expr_name(expr) {
            if self.value_locals.contains(&name) || self.value_loop_vars.contains(&name) {
                return true;
            }
            if self
                .state_types
                .get(&name)
                .map_or(false, |t| t == "serde_json::Value")
            {
                return true;
            }
            if self
                .prop_types
                .get(&name)
                .map_or(false, |t| t == "serde_json::Value")
            {
                return true;
            }
            if self.needs_index_access(&name) {
                return true;
            }
        }
        false
    }

    /// PLAN-039 T-12（批次 E）：内建方法接收者的类型格——value/string/
    /// vec/int/unknown 五态。判据序：state/prop 类型表 → value_locals/
    /// value_loop_vars（Value 局部与循环变量）→ store 字段表（`store.x`
    /// 接收者）→ 字面量。unknown = 局部变量声明面（T-14 类型格补全）。
    fn builtin_receiver_kind(&self, obj: &crate::ast::Expr) -> &'static str {
        use crate::ast::Expr;
        // PLAN-039 T-12：集合元素接收者（`xs[i].slice(…)` 形态）——
        // Vec<Value> 索引产出 Value 元素（expr_is_value_typed 同判据）。
        if let Expr::Index(_, _) = obj {
            if self.expr_is_value_typed(obj) {
                return "value";
            }
        }
        // `.store.<field>` 接收者（Dot 真链或拍平 Ident 两形态）。
        if let Expr::Dot(inner, field) = obj {
            if let Expr::Ident(n) = inner.as_ref() {
                if n.as_str() == "store" {
                    return match self
                        .store_field_rust_type(field.as_str())
                        .unwrap_or_default()
                        .as_str()
                    {
                        "serde_json::Value" => "value",
                        t if t.starts_with("Vec<") => "vec",
                        "String" => "string",
                        "i32" | "i64" | "u32" | "u64" | "f32" | "f64" => "int",
                        _ => "unknown",
                    };
                }
            }
        }
        if let Expr::Ident(name) = obj {
            let s = name.as_str();
            if s.starts_with(".store.") {
                let field = &s[".store.".len()..];
                if !field.contains('.') {
                    return match self.store_field_rust_type(field).unwrap_or_default().as_str() {
                        "serde_json::Value" => "value",
                        t if t.starts_with("Vec<") => "vec",
                        "String" => "string",
                        "i32" | "i64" | "u32" | "u64" | "f32" | "f64" => "int",
                        _ => "unknown",
                    };
                }
            }
        }
        // 局部/循环变量 Value 态。
        if let Some(name) = self.resolve_expr_name(obj) {
            // PLAN-039 T-12：无类型集合局部 → vec 格（原生 len/push/pop）。
            if self.array_locals.contains(&name) {
                return "vec";
            }
            if self.value_locals.contains(&name) || self.value_loop_vars.contains(&name) {
                return "value";
            }
            let table = self
                .state_types
                .get(&name)
                .or_else(|| self.prop_types.get(&name));
            if let Some(ty) = table {
                return match ty.as_str() {
                    "serde_json::Value" => "value",
                    t if t.starts_with("Vec<") => "vec",
                    "String" => "string",
                    "i32" | "i64" | "u32" | "u64" | "f32" | "f64" => "int",
                    _ => "unknown",
                };
            }
            // needs_index_access 兜底（value_prop_names 等通道）。
            if self.needs_index_access(&name) {
                return "value";
            }
        }
        match obj {
            Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_) => "string",
            Expr::Int(_) | Expr::I64(_) | Expr::Float(_, _) | Expr::Double(_, _) => "int",
            _ => "unknown",
        }
    }

    /// PLAN-039 T-12（批次 E，E-D4）：.at 内建方法翻译表——`x.len()/
    /// slice/str/lower/upper/trim/replace` 按接收者类型格发射。Value
    /// 接收者走 `__at_*` shim 家族（rust_ui.rs 文件级注入，VM 宽松语义：
    /// len 串按字符/数组按元素、str 数值降串不带引号）。返回 None =
    /// 非内建（或接收者 unknown 且方法不安全），落回默认 Call 路径。
    fn try_builtin_method_call(
        &self,
        obj: &crate::ast::Expr,
        method: &str,
        call: &crate::ast::Call,
    ) -> Option<String> {
        let kind = self.builtin_receiver_kind(obj);
        if kind == "unknown" {
            // unknown 接收者只收 str 安全族（串局部常见面）——len 的
            // 串/数歧义留给 T-14 类型格落格后再收；str() 三态全安全
            // （int → to_string、String → to_string 皆合法）。
            return match method {
                "str" => Some(format!("({}).to_string()", self.ast_expr_to_rust(obj))),
                "slice" | "lower" | "upper" | "trim" | "replace" | "contains"
                | "starts_with" | "ends_with" => self
                    .try_string_method_call(obj, method, call, /*force_string=*/ true),
                _ => None,
            };
        }
        match method {
            "len" => match kind {
                "value" => Some(format!("__at_len(&({}))", self.ast_expr_to_rust(obj))),
                "vec" => Some(format!("(({}).len() as i32)", self.ast_expr_to_rust(obj))),
                "string" => Some(format!(
                    "(({}).chars().count() as i32)",
                    self.ast_expr_to_rust(obj)
                )),
                _ => None,
            },
            "slice" => {
                let (a, b) = self.two_i32_args(call)?;
                match kind {
                    "value" => Some(format!(
                        "__at_slice_v(&({}), {}, {})",
                        self.ast_expr_to_rust(obj),
                        a,
                        b
                    )),
                    "string" => Some(format!(
                        "__at_slice(&({}), {}, {})",
                        self.ast_expr_to_rust(obj),
                        a,
                        b
                    )),
                    _ => None,
                }
            }
            "str" => match kind {
                "value" => Some(format!("__at_str(&({}))", self.ast_expr_to_rust(obj))),
                "int" | "string" => Some(format!("({}).to_string()", self.ast_expr_to_rust(obj))),
                _ => None,
            },
            "lower" => match kind {
                "value" => Some(format!("__at_lower(&({}))", self.ast_expr_to_rust(obj))),
                _ => self.try_string_method_call(obj, method, call, true),
            },
            "upper" => match kind {
                "value" => Some(format!("__at_upper(&({}))", self.ast_expr_to_rust(obj))),
                _ => self.try_string_method_call(obj, method, call, true),
            },
            "trim" => match kind {
                "value" => Some(format!("__at_trim(&({}))", self.ast_expr_to_rust(obj))),
                _ => self.try_string_method_call(obj, method, call, true),
            },
            "replace" => match kind {
                "value" => None, // Value 上 replace 不在认知面（拒绝门纪律交默认路径）
                _ => self.try_string_method_call(obj, method, call, true),
            },
            "contains" | "starts_with" | "ends_with" => {
                self.try_string_method_call(obj, method, call, false)
            }
            // PLAN-039 T-14（E-D3 第二轨）：Value 集合的 pop——Option<
            // Value>::unwrap_or(Null)（此前裸 pop 走 Vec 泛型，调用侧
            // .unwrap_or(0) 在 Vec<Value> 上 i32 形参断——minesweeper
            // stack.pop() 回归株）；显式 int 局部由赋值点强转接住。
            "pop" => {
                if self.receiver_is_value_collection(obj) {
                    Some(format!(
                        "(({}).pop().unwrap_or(serde_json::Value::Null))",
                        self.ast_expr_to_rust(obj)
                    ))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// PLAN-039 T-14：接收者是否 Value 集合（无类型集合局部或
    /// Vec<Value> 态字段）——push/pop 的 json!/Null 化判据。
    fn receiver_is_value_collection(&self, obj: &crate::ast::Expr) -> bool {
        if let Some(name) = self.resolve_expr_name(obj) {
            return self.array_locals.contains(&name)
                || self
                    .state_types
                    .get(&name)
                    .map_or(false, |t| t == "Vec<serde_json::Value>");
        }
        if let crate::ast::Expr::Ident(n) = obj {
            let s = n.as_str();
            if s.starts_with(".store.") {
                let f = &s[".store.".len()..];
                return !f.contains('.')
                    && self
                        .store_field_rust_type(f)
                        .map_or(false, |t| t == "Vec<serde_json::Value>");
            }
        }
        if let crate::ast::Expr::Dot(mid, f) = obj {
            let base_is_store = match mid.as_ref() {
                crate::ast::Expr::Ident(n) => n.as_str() == "store" || n.as_str() == ".store",
                _ => false,
            };
            return base_is_store
                && self
                    .store_field_rust_type(f.as_str())
                    .map_or(false, |t| t == "Vec<serde_json::Value>");
        }
        false
    }

    /// str 形内建方法发射（string/unknown 接收者共用；force_string =
    /// unknown 接收者按 String 假定发射——错型由编译错误显式拦截）。
    fn try_string_method_call(
        &self,
        obj: &crate::ast::Expr,
        method: &str,
        call: &crate::ast::Call,
        force_string: bool,
    ) -> Option<String> {
        let kind = self.builtin_receiver_kind(obj);
        if kind != "string" && !force_string {
            return None;
        }
        let obj_str = self.ast_expr_to_rust(obj);
        let args: Vec<String> = call
            .args
            .args
            .iter()
            .map(|a| self.ast_expr_to_rust(&a.get_expr()))
            .collect();
        Some(match method {
            "lower" => format!("({}).to_lowercase()", obj_str),
            "upper" => format!("({}).to_uppercase()", obj_str),
            "trim" => format!("({}).trim().to_string()", obj_str),
            "replace" if args.len() >= 2 => {
                format!("({}).replace(&{}, &{})", obj_str, args[0], args[1])
            }
            "contains" if !args.is_empty() => {
                format!("({}).contains(&{})", obj_str, args[0])
            }
            "starts_with" if !args.is_empty() => {
                format!("({}).starts_with(&{})", obj_str, args[0])
            }
            "ends_with" if !args.is_empty() => format!("({}).ends_with(&{})", obj_str, args[0]),
            "slice" if args.len() >= 2 => format!(
                "__at_slice(&({}), {} as i32, {} as i32)",
                obj_str, args[0], args[1]
            ),
            _ => return None,
        })
    }

    /// slice(a, b) 两参数的 i32 形翻译（数值域统一 i32——生成器 int 倾向
    /// 一致；as 收口，字面量/表达式通用）。
    fn two_i32_args(&self, call: &crate::ast::Call) -> Option<(String, String)> {
        let a = call.args.args.first()?;
        let b = call.args.args.get(1)?;
        Some((
            format!("({}) as i32", self.ast_expr_to_rust(&a.get_expr())),
            format!("({}) as i32", self.ast_expr_to_rust(&b.get_expr())),
        ))
    }

    fn value_field_access(&self, obj_expr: &str, field: &str) -> String {
        // Plan 374: Type-aware Value field access based on field name conventions.
        // Bool fields: use .as_bool().unwrap_or(false)
        // Int fields: use .as_i64().unwrap_or(0) as i32
        // Default (string/array/object): use .as_str().unwrap_or_default().to_string()
        // NOTE: array fields (e.g. tags) are intentionally handled as String here.
        // Iteration/push/contains on them is rewritten by postprocess_handler_body
        // (Fix 2 / __a push), and #[api] Vec<String> arguments are rewritten by
        // the update_tags special-case in postprocess. Keeping this branch as
        // String avoids breaking those rewrites.
        if field == "id" || field.ends_with("_id") || field == "idx" || field == "count"
            || field == "x" || field == "y" || field == "adjacent" {
            // Plan 407: parenthesize so callers can safely append .to_string() etc.
            format!("({}[\"{}\"].as_i64().unwrap_or(0) as i32)", obj_expr, field)
        } else if field == "pinned" || field == "done" || field == "deleted" || field == "active"
            || field == "editing" || field == "loading" || field == "dark_mode"
            || field == "show_tag_input" || field.starts_with("is_")
            || field == "mine" || field == "revealed" || field == "flagged" {
            format!("{}[\"{}\"].as_bool().unwrap_or(false)", obj_expr, field)
        } else {
            format!("{}[\"{}\"].as_str().unwrap_or_default().to_string()", obj_expr, field)
        }
    }

    /// PLAN-036 T-02：Eq/Neq 跨型比较统一降串（主臂与值参闭包臂共用）——
    /// VM 数值/布尔的宽松比较语义在编译轨的等价（shell pack Obj 线上形态：
    /// bool 降 "1"/"" 串、id 串/数两态）。一侧 String 类（字面量/String 态
    /// 字段/串接/文本化 Value 访问）另一侧数值类（`as i32)` 收尾）→ 数值侧
    /// 补 `.to_string()`；另一侧为 Value 布尔访问（`.as_bool().unwrap_
    /// or(false)`）→ 改 as_str 形（宿主降串 "1"/"" 判真——与解释臂逐语义
    /// 一致）。仅真改写时返回 Some（两侧同类返回 None 走既有路径——非
    /// shell 语料零扰动）。
    fn normalize_eq_neq_compare(
        &self,
        op: &auto_val::Op,
        left: &crate::ast::Expr,
        right: &crate::ast::Expr,
        left_str: &str,
        right_str: &str,
    ) -> Option<String> {
        use crate::ast::Expr;
        use auto_val::Op;
        if !matches!(op, Op::Eq | Op::Neq) {
            return None;
        }
        let side_is_stringy = |e: &Expr, s: &str| {
            matches!(e, Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_))
                || self.ast_expr_is_string(e)
                || s.contains(".as_str()")
                || s.starts_with('"')
                || s.contains("format!(")
        };
        let l_stringy = side_is_stringy(left, left_str);
        let r_stringy = side_is_stringy(right, right_str);
        let mut l = left_str.to_string();
        let mut r = right_str.to_string();
        let mut changed = false;
        let num_state = |e: &Expr| -> bool {
            self.resolve_expr_name(e)
                .and_then(|n| self.state_types.get(&n).map(|t| t.clone()))
                .map(|ty| matches!(ty.as_str(), "i32" | "i64" | "u32" | "u64" | "f32" | "f64"))
                .unwrap_or(false)
        };
        if l_stringy && !r_stringy {
            if r.contains(".as_bool().unwrap_or(false)") {
                r = r.replace(
                    ".as_bool().unwrap_or(false)",
                    ".as_str().unwrap_or_default().to_string()",
                );
                changed = true;
            } else if r.ends_with("as i32)") || num_state(right) {
                r = format!("{}.to_string()", r);
                changed = true;
            }
        } else if r_stringy && !l_stringy {
            if l.contains(".as_bool().unwrap_or(false)") {
                l = l.replace(
                    ".as_bool().unwrap_or(false)",
                    ".as_str().unwrap_or_default().to_string()",
                );
                changed = true;
            } else if l.ends_with("as i32)") || num_state(left) {
                l = format!("{}.to_string()", l);
                changed = true;
            }
        }
        if !changed {
            return None;
        }
        let op_str = match op {
            Op::Eq => "==",
            _ => "!=",
        };
        Some(format!("({}) {} ({})", l, op_str, r))
    }

    /// Check if an AST expression produces a String type (for detecting string concatenation)
    fn ast_expr_is_string(&self, expr: &crate::ast::Expr) -> bool {        use crate::ast::Expr;
        match expr {
            Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_) => true,
            Expr::Ident(name) => {
                let s = name.as_str();
                let resolved = if s.starts_with('.') { &s[1..] } else { s };
                self.state_types.get(resolved).map_or(false, |ty| ty == "String")
            }
            Expr::Dot(obj, field) => {
                // Dot(Ident("self"), "display") → check field "display" in state_types
                if let Expr::Ident(obj_name) = obj.as_ref() {
                    let obj_s = obj_name.as_str();
                    if obj_s == "self" || obj_s.starts_with('.') {
                        return self.state_types.get(field.as_str())
                            .map_or(false, |ty| ty == "String");
                    }
                }
                // Generic dot access: check the object chain
                self.ast_expr_is_string(obj)
            }
            Expr::Bina(_left, op, _right) => {
                // If this is an Add chain, check if either operand is string
                use auto_val::Op;
                if matches!(op, Op::Add) {
                    self.ast_expr_is_string(_left) || self.ast_expr_is_string(_right)
                } else {
                    false
                }
            }
            Expr::Call(_) => false,
            _ => false,
        }
    }

    /// Resolve an AST expression to a simple field name (for state_types lookup).
    /// Returns None if the expression is not a simple field reference.
    fn resolve_expr_name(&self, expr: &crate::ast::Expr) -> Option<String> {
        use crate::ast::Expr;
        match expr {
            Expr::Ident(name) => {
                let s = name.as_str();
                if s.starts_with('.') {
                    Some(s[1..].to_string())
                } else {
                    Some(s.to_string())
                }
            }
            Expr::Dot(obj, field) => {
                // Dot(Ident("self"), "field") → "field"
                if let Expr::Ident(obj_name) = obj.as_ref() {
                    let obj_s = obj_name.as_str();
                    if obj_s == "self" || obj_s.starts_with('.') {
                        return Some(field.as_str().to_string());
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Like ast_expr_to_rust, but treats specified param names as serde_json::Value variables.
    /// Used for closures passed to findIndex/.position() where params iterate over &Value.
    fn ast_expr_to_rust_with_value_params(&self, expr: &crate::ast::Expr, value_params: &[String]) -> String {
        use crate::ast::Expr;
        // Intercept Dot access on value params
        if let Expr::Dot(obj, field) = expr {
            if let Expr::Ident(name) = obj.as_ref() {
                if value_params.contains(&name.to_string()) {
                    return self.value_field_access(name.as_str(), field.as_str());
                }
            }
        }
        // For all other cases, delegate to ast_expr_to_rust.
        // We can't intercept nested closures or deeper expressions that reference value_params,
        // but the common case is `t.field == value` which is handled above.
        // For compound expressions, we recursively apply the same logic.
        match expr {
            Expr::Bina(left, op, right) => {
                let left_str = self.ast_expr_to_rust_with_value_params(left, value_params);
                let right_str = self.ast_expr_to_rust_with_value_params(right, value_params);
                // Use the same op handling as ast_expr_to_rust
                use auto_val::Op;
                // PLAN-036 T-02：串接检测（主臂同款——值参闭包体原直发 `+`，
                // String+数值型错配漏网）；Eq/Neq 跨型降串共用
                // normalize_eq_neq_compare。
                if matches!(op, Op::Add)
                    && (matches!(left.as_ref(), Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_))
                        || matches!(right.as_ref(), Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_))
                        || self.ast_expr_is_string(left)
                        || self.ast_expr_is_string(right))
                {
                    return format!("format!(\"{{}}{{}}\", {}, {})", left_str, right_str);
                }
                if let Some(cmp) =
                    self.normalize_eq_neq_compare(op, left, right, &left_str, &right_str)
                {
                    return cmp;
                }
                let op_str = match op {
                    Op::Eq => "==",
                    Op::Neq => "!=",
                    Op::Lt => "<",
                    Op::Le => "<=",
                    Op::Gt => ">",
                    Op::Ge => ">=",
                    Op::And => "&&",
                    Op::Or => "||",
                    Op::Add => "+",
                    Op::Sub => "-",
                    Op::Not => "!",
                    _ => "?",
                };
                format!("{} {} {}", left_str, op_str, right_str)
            }
            Expr::Unary(op, operand) => {
                let val = self.ast_expr_to_rust_with_value_params(operand, value_params);
                use auto_val::Op;
                match op {
                    Op::Not => format!("!({})", val),
                    Op::Sub => format!("-{}", val),
                    _ => format!("/* unimplemented unary {:?} */", op),
                }
            }
            // For everything else (idents, literals, etc.), use normal conversion
            _ => self.ast_expr_to_rust(expr),
        }
    }

    /// PLAN-039 T-04（台账 M7-c「缺口即发现即修」）：.at 字面量内容再
    /// 转义为 Rust 字面量体——.at 解析已剥源码层 `\"`，发射若不回转义，
    /// 含引号/反斜杠/换行的串会打断生成物语法（kanban boards_store.at
    /// `"{\"title\":\"" + esc(t) + …"` JSON 体拼接 = 4 个 mismatched
    /// delimiter 的根因）。CJK 等非 ASCII 原样保留（escape_default 会
    /// 全量转义破坏双语文本）。
    fn rust_str_lit_body(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }

    /// Same as ast_expr_to_rust but without appending .to_string() to Str literals
    fn ast_expr_to_rust_no_to_string(&self, expr: &crate::ast::Expr) -> String {
        use crate::ast::Expr;
        match expr {
            Expr::Str(s) => format!("\"{}\"", Self::rust_str_lit_body(s)),
            Expr::CStr(s) => format!("\"{}\"", Self::rust_str_lit_body(s)),
            // For everything else, delegate to ast_expr_to_rust
            _ => self.ast_expr_to_rust(expr),
        }
    }

    fn ast_expr_to_rust(&self, expr: &crate::ast::Expr) -> String {
        use crate::ast::Expr;
        use auto_val::Op;
        match expr {
            Expr::Str(s) => format!("\"{}\".to_string()", Self::rust_str_lit_body(s)),
            Expr::I64(n) => n.to_string(),
            Expr::Int(n) => n.to_string(),
            Expr::U64(n) => n.to_string(),
            Expr::Uint(n) => n.to_string(),
            Expr::Float(n, _) => {
                let s = format!("{}", n);
                if s.contains('.') { s } else { format!("{}.0", n) }
            }
            Expr::Double(n, _) => {
                let s = format!("{}", n);
                if s.contains('.') { s } else { format!("{}.0", n) }
            }
            Expr::Bool(b) => b.to_string(),
            // PLAN-039 T-13（批次 E，E-D5-A）：用户型构造字面量
            // （`Meta { source_root: "", count_active: 0 }` 解析为
            // Expr::Node——kanban store model 初始式株，此前落 `/* expr */`
            // 占位）。字段按 Pair 直发 + `..Default::default()` 兜底
            // （back 型 struct 已 derive Default）；无字段 = ::default()。
            crate::ast::Expr::Node(node) => {
                if node.args.args.is_empty() {
                    format!("{}::default()", node.name.as_str())
                } else {
                    let fields: Vec<String> = node.args.args.iter()
                        .filter_map(|a| match a {
                            crate::ast::Arg::Pair(n, e) => Some(format!(
                                "{}: {}",
                                n.as_str(),
                                self.ast_expr_to_rust(e)
                            )),
                            _ => None,
                        })
                        .collect();
                    format!(
                        "{} {{ {}, ..Default::default() }}",
                        node.name.as_str(),
                        fields.join(", ")
                    )
                }
            }
            Expr::Ident(name) => {
                let s = name.as_str();
                // Plan 374 Task 2: store composable rewriting
                if s == "store" || s == ".store" {
                    return "self.store".to_string();
                }
                if s.starts_with(".store.") {
                    // PLAN-039 T-12（批次 E）：`.store.<field>.<sub>` 多级链
                    // ——中间级为 Value 型 store 字段（记录字面量株，如
                    // klondike waste_card）时按 E-D1 使用点包裹降链发射
                    // `self.store.<field>["<sub>"]…`；首级直访（`.store.x`）
                    // 与非 Value 中间级维持直发。更深链（sub 含点）不在
                    // 认知面，直发交由下游编译错误显式拦截。
                    // T-13（E-D2）：形状表切换访问器（rank:int → as_i64）。
                    let path = &s[".store.".len()..];
                    if let Some(dot_pos) = path.find('.') {
                        let field = &path[..dot_pos];
                        let sub = &path[dot_pos + 1..];
                        if !sub.contains('.') && self.store_field_is_value(field) {
                            let shape = self.store_record_shape(field);
                            return self.value_field_access_shaped(
                                &format!("self.store.{}", field),
                                sub,
                                shape.as_ref(),
                            );
                        }
                    }
                    return format!("self.{}", &s[1..]);
                }
                if s.starts_with('.') {
                    let path = &s[1..];
                    // Check for dotted path on Value-type var (e.g., ".note.title")
                    if let Some(dot_pos) = path.find('.') {
                        let first = &path[..dot_pos];
                        if self.needs_index_access(first) {
                            let field = &path[dot_pos + 1..];
                            // Reading from serde_json::Value: use index + string conversion
                            return format!("self.{}[\"{}\"].as_str().unwrap_or_default().to_string()", first, field);
                        }
                    }
                    format!("self.{}", path)
                } else if self.state_types.contains_key(s) || self.prop_names.contains(s) {
                    format!("self.{}", s)
                } else {
                    s.to_string()
                }
            }
            Expr::Dot(obj, field) => {
                let field_str = field.as_str();
                // Plan 374 Task 2: store.field → self.store.field
                // EDGE-01/a2r fix: if field is a computed property, use method-call
                // syntax () — computed generates as fn, not a struct field.
                let is_computed = self.computed_names.contains(field_str)
                    || STORE_COMPUTED_NAMES.with(|sn| sn.borrow().contains(field_str));
                if let Expr::Ident(name) = obj.as_ref() {
                    if name.as_str() == "store" {
                        if is_computed {
                            return format!("self.store.{}()", field_str);
                        }
                        return format!("self.store.{}", field_str);
                    }
                }
                // Detect pattern: Dot(Dot(Ident("self"), prop_name), field_name)
                // This is self.prop_name.field_name — check if prop_name is Value-type
                if let Expr::Dot(inner_obj, inner_field) = obj.as_ref() {
                    if let Expr::Ident(inner_name) = inner_obj.as_ref() {
                        let inner_s = inner_name.as_str();
                        let prop_name = inner_field.as_str();
                        // Pattern: self.prop_name.field_str
                        if (inner_s == "self" || inner_s.starts_with('.')) && self.needs_index_access(prop_name) {
                            // Reading from Value: self.note["field"] with type-aware accessor
                            let obj_expr = format!("self.{}", prop_name);
                            return self.value_field_access(&obj_expr, field_str);
                        }
                    }
                }
                // PLAN-039 T-12（批次 E）：`.store.<value 字段>.<sub>` 真链
                // 形态通用判定——obj 剥层后基座是 `.store`/`self.store` 且其
                // 尾字段为 store Value 字段（parser 三层 Dot 形态的主消费
                // 面，probe 实证 Dot(Dot(Dot(self, store), F), sub)）。
                // T-13（E-D2）：形状表切换访问器（rank:int → as_i64）。
                if let Some((base, shape)) = self.dot_chain_store_value_base(obj) {
                    return self.value_field_access_shaped(&base, field_str, shape.as_ref());
                }
                // If accessing a field on a Value-type prop directly: obj.field where obj is a prop
                if let Expr::Ident(name) = obj.as_ref() {
                    let s = name.as_str();
                    let resolved = if s.starts_with('.') { &s[1..] } else { s };
                    // PLAN-039 T-14（E-D2）：循环变量的元素形状——
                    // `scard.rank` 按集合元素形状选访问器（showcase_cards
                    // 记录集合株：rank 启发式名单外恒 as_str）。
                    if let Some(coll) = self.loop_var_collections.get(resolved) {
                        if let Some(shape) = self.array_element_shapes.get(coll) {
                            return self.value_field_access_shaped(resolved, field_str, Some(shape));
                        }
                    }
                    if self.needs_index_access(resolved) {
                        let obj_str = if s == "self" || s.starts_with('.') {
                            format!("self.{}", resolved)
                        } else if self.state_types.contains_key(resolved) || self.prop_names.contains(resolved) {
                            format!("self.{}", resolved)
                        } else {
                            resolved.to_string()
                        };
                        // PLAN-039 T-13（E-D2）：局部记录形状优先——
                        // row.score 按形状型选访问器，未知回落启发式。
                        let shape = self.local_record_shapes.get(resolved).cloned();
                        return self.value_field_access_shaped(&obj_str, field_str, shape.as_ref());
                    }
                }
                // Check if object is an index into a Vec<Value>: todos[idx].field
                // Pattern: Dot(Index(Ident("todos"), idx), "field")
                if let Expr::Index(target, _idx) = obj.as_ref() {
                    // Resolve the collection name from various patterns:
                    // Ident("notes"), Dot(Ident("self"), "notes"), Dot(Dot(Ident("self"), "store"), "notes")
                    let (coll_name, is_self_prefixed) = self.resolve_collection_name(target);
                    if let Some(coll_name) = coll_name {
                        let resolved_coll = &coll_name;
                        // Check if this is a Vec<Value> collection
                        // PLAN-039 T-13（E-D5-A）：收窄为 Vec<Value> 专属
                        // ——typed Vec（Vec<Card>）元素字段直达，不降链。
                        let is_vec_value = self.state_types.get(resolved_coll)
                            .map(|ty| ty == "Vec<serde_json::Value>")
                            .unwrap_or(false)
                            // Also check store fields and compound paths
                            || resolved_coll.contains("notes")
                            // PLAN-039 T-12（批次 E）：无类型集合局部
                            // （`var scored = []` → array_locals 收格；Call
                            // 局部可能在 value_locals）——`scored[i].field`
                            // 元素字段直访降链。
                            || self.array_locals.contains(resolved_coll)
                            || self.value_locals.contains(resolved_coll);
                        if is_vec_value {
                            // Indexing into Vec<Value> produces Value — use bracket access
                            let idx_str = self.ast_expr_to_rust(_idx);
                            let target_str = if is_self_prefixed {
                                format!("self.{}", coll_name)
                            } else {
                                coll_name.clone()
                            };
                            let idx_cast = if idx_str.starts_with("self.")
                                || (!idx_str.parse::<usize>().is_ok() && idx_str != "0")
                            {
                                // Plan 407: parenthesize so `as usize` binds to whole expr.
                                format!("({}) as usize", idx_str)
                            } else {
                                idx_str
                            };
                            // PLAN-039 T-13（E-D2）：集合局部元素形状——
                            // scored[i].score 按形状型选访问器。
                            let elem_shape = self
                                .array_element_shapes
                                .get(resolved_coll)
                                .cloned();
                            return self.value_field_access_shaped(
                                &format!("{}[{}]", target_str, idx_cast),
                                field_str,
                                elem_shape.as_ref(),
                            );
                        }
                    }
                }
                let obj_str = self.ast_expr_to_rust(obj);
                // Plan 407 R1: wrap Bina objects in parens so `.field` binds to the
                // whole expression, not just the right operand. E.g.
                // `(.mine_count - .flags_placed).to_string()` must emit
                // `(self.mine_count - self.flags_placed).to_string`, not
                // `self.mine_count - self.flags_placed.to_string`.
                let obj_str = if matches!(obj.as_ref(), Expr::Bina(..)) {
                    format!("({})", obj_str)
                } else {
                    obj_str
                };
                format!("{}.{}", obj_str, field_str)
            }
            Expr::Bina(left, op, right) => {
                // Assignment: .count = expr → self.count = expr
                if matches!(op, Op::Asn) {
                    // Check if target is a Value field write like self.note.title = value
                    // Pattern: Dot(Dot(Ident("self"), "note"), "title")
                    if let Expr::Dot(outer_obj, outer_field) = left.as_ref() {
                        if let Expr::Dot(inner_obj, inner_field) = outer_obj.as_ref() {
                            if let Expr::Ident(inner_name) = inner_obj.as_ref() {
                                let inner_s = inner_name.as_str();
                                let prop_name = inner_field.as_str();
                                if (inner_s == "self" || inner_s.starts_with('.')) && self.needs_index_access(prop_name) {
                                    let field = outer_field.as_str();
                                    let value = self.ast_expr_to_rust(right);
                                    // Write to Value field: self.note["title"] = json!(value)
                                    return format!("self.{}[\"{}\"] = serde_json::json!({})", prop_name, field, value);
                                }
                            }
                        }
                    }
                    // Also check for single-dot Ident pattern like ".note.title"
                    if let Expr::Ident(name) = left.as_ref() {
                        let s = name.as_str();
                        if s.starts_with('.') {
                            let path = &s[1..];
                            if let Some(dot_pos) = path.find('.') {
                                let first = &path[..dot_pos];
                                if self.needs_index_access(first) {
                                    let field = &path[dot_pos + 1..];
                                    let value = self.ast_expr_to_rust(right);
                                    return format!("self.{}[\"{}\"] = serde_json::json!({})", first, field, value);
                                }
                            }
                        }
                    }
                    // Check for value_local.field = value (e.g., todo.done = !todo.done)
                    if let Expr::Dot(obj, field) = left.as_ref() {
                        if let Expr::Ident(name) = obj.as_ref() {
                            let s = name.as_str();
                            if self.value_locals.contains(s) || self.needs_index_access(s) {
                                let value = self.ast_expr_to_rust(right);
                                return format!("{}[\"{}\"] = serde_json::json!({})", s, field.as_str(), value);
                            }
                        }
                        // Check for indexed.field = value (e.g., todos[idx].text = .edit_text)
                        // Pattern: Dot(Index(Ident("collection"), idx), "field")
                        if let Expr::Index(target, _idx) = obj.as_ref() {
                            if let Expr::Ident(collection) = target.as_ref() {
                                let coll_name = collection.as_str();
                                let resolved_coll = if coll_name.starts_with('.') { &coll_name[1..] } else { coll_name };
                                if self.state_types.get(resolved_coll)
                                    .map(|ty| ty.starts_with("Vec<"))
                                    .unwrap_or(false)
                                {
                                    let idx_str = self.ast_expr_to_rust(_idx);
                                    let target_str = if resolved_coll != coll_name {
                                        format!("self.{}", resolved_coll)
                                    } else if self.state_types.contains_key(coll_name) {
                                        format!("self.{}", coll_name)
                                    } else {
                                        coll_name.to_string()
                                    };
                                    let idx_cast = if idx_str.starts_with("self.")
                                        || (!idx_str.parse::<usize>().is_ok() && idx_str != "0")
                                    {
                                        format!("{} as usize", idx_str)
                                    } else {
                                        idx_str
                                    };
                                    let value = self.ast_expr_to_rust(right);
                                    return format!("{}[{}][\"{}\"] = serde_json::json!({})", target_str, idx_cast, field.as_str(), value);
                                }
                            }
                        }
                    }
                    let target = self.ast_expr_to_rust(left);
                    let mut value = self.ast_expr_to_rust(right);
                    // PLAN-039 T-14（E-D3 第二轨）：局部收 state
                    // Vec<Value>/Value 字段的 clone——`raw_list =
                    // self.col0_cards` move 出 &mut self 字段（klondike
                    // raw_list 分发株 ×7）。
                    if let Expr::Ident(n) = left.as_ref() {
                        let ln = n.as_str().trim_start_matches('.');
                        if !self.state_types.contains_key(ln)
                            && !self.value_locals.contains(ln)
                            && matches!(right.as_ref(), Expr::Ident(_) | Expr::Dot(..))
                            && !value.contains('[')
                            && !value.starts_with('"')
                            && !value.contains(".clone()")
                        {
                            let rhs_name = self.resolve_expr_name(right);
                            if rhs_name
                                .as_ref()
                                .and_then(|rn| self.state_types.get(rn))
                                .map_or(false, |t| {
                                    t == "Vec<serde_json::Value>"
                                        || t == "serde_json::Value"
                                })
                            {
                                value = format!("{}.clone()", value);
                            }
                        }
                    }
                    // PLAN-039 T-14（E-D3 第二轨）：Value 集合的索引
                    // 写标量 RHS json! 化——`deck[j] = tmp`（tmp: int 存
                    // Vec<Value>；洗牌交换株）。
                    if target.contains('[') {
                        let base = target.split('[').next().unwrap_or("");
                        let base_name = base.rsplit('.').next().unwrap_or(base);
                        let coll_is_value = self.array_locals.contains(base_name)
                            || self
                                .state_types
                                .get(base_name)
                                .map_or(false, |t| t == "Vec<serde_json::Value>");
                        let rhs_scalar_or_local = matches!(
                            right.as_ref(),
                            Expr::Int(_)
                                | Expr::I64(_)
                                | Expr::Str(_)
                                | Expr::Bool(_)
                                | Expr::Ident(_)
                                | Expr::Bina(_, _, _)
                                | Expr::Index(_, _)
                        );
                        if coll_is_value
                            && rhs_scalar_or_local
                            && !value.starts_with('"')
                            && !value.contains("serde_json::json!")
                        {
                            // 读侧（deck[i] = deck[j] 的 RHS Index）move 出
                            // Vec<Value> 索引位——json! 同包（Value 直通）。
                            value = format!(
                                "serde_json::json!({})",
                                self.ast_expr_to_rust_no_to_string(right)
                            );
                        }
                    }
                    // PLAN-039 T-14（E-D3 第一轨）：显式声明局部的裸
                    // Value RHS 强转（`card_id = .col0_cards[i]` →
                    // __at_num / as_str）。
                    if let Expr::Ident(n) = left.as_ref() {
                        let ln = n.as_str().trim_start_matches('.');
                        if let Some(kind) = self.declared_locals.get(ln) {
                            let rhs_is_value = self.expr_is_value_typed(right)
                                || value.contains("[\"");
                            if rhs_is_value && !value.contains("__at_num") {
                                match kind.as_str() {
                                    "int" => {
                                        value = format!("__at_num(&({}))", value);
                                    }
                                    "str" => {
                                        if value.contains(
                                            ".as_i64().unwrap_or(0) as i32",
                                        ) {
                                            value = value.replace(
                                                ".as_i64().unwrap_or(0) as i32",
                                                ".as_str().unwrap_or_default().to_string()",
                                            );
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    // PLAN-039 T-14（E-D3 第二轨）：Value 联合局部的标量
                    // 字面量赋值 json! 化——`card = 999`（初始式 + 后续
                    // Asn 同口径）。
                    if let Expr::Ident(n) = left.as_ref() {
                        let ln = n.as_str().trim_start_matches('.');
                        if self.value_locals.contains(ln)
                            && !self.declared_locals.contains_key(ln)
                            && matches!(
                                right.as_ref(),
                                Expr::Int(_)
                                    | Expr::I64(_)
                                    | Expr::Str(_)
                                    | Expr::Bool(_)
                                    | Expr::Float(_, _)
                                    | Expr::Double(_, _)
                            )
                        {
                            value = format!(
                                "serde_json::json!({})",
                                self.ast_expr_to_rust_no_to_string(right)
                            );
                        }
                    }
                    // PLAN-039 T-14（E-D3 第二轨）：Value 访问器按赋值
                    // 目标型重写——value_field_access 启发式按字段名选串
                    // （snap["d0"]/snap["stock"]），而目标字段是 int/
                    // Vec<Value>（undo 快照回填株）：降串产物在 int 目标
                    // 侧改 as_i64 收口、在 Value/Vec<Value> 目标侧改 clone
                    // 直取。
                    {
                        let l_ty = self
                            .resolve_expr_name(left)
                            .and_then(|n| self.state_types.get(&n).cloned());
                        if let Some(ty) = l_ty {
                            let has_str_accessor =
                                value.contains(".as_str().unwrap_or_default().to_string()");
                            if has_str_accessor {
                                match ty.as_str() {
                                    "i32" | "i64" | "u32" | "u64" | "f32" | "f64" => {
                                        value = value.replace(
                                            ".as_str().unwrap_or_default().to_string()",
                                            ".as_i64().unwrap_or(0) as i32",
                                        );
                                    }
                                    "serde_json::Value" => {
                                        value = value.replace(
                                            ".as_str().unwrap_or_default().to_string()",
                                            ".clone()",
                                        );
                                    }
                                    "Vec<serde_json::Value>" => {
                                        // 数组目标：as_array 转换（unwrap_or
                                        // 空集容差——undo 快照缺键株）。
                                        value = value.replace(
                                            ".as_str().unwrap_or_default().to_string()",
                                            ".as_array().cloned().unwrap_or_default()",
                                        );
                                    }
                                    _ => {}
                                }
                            } else if matches!(
                                ty.as_str(),
                                "i32" | "i64" | "u32" | "u64" | "f32" | "f64"
                            ) && self.expr_is_value_typed(right)
                                && !value.contains("__at_num")
                            {
                                // 裸 Value 局部/元素（card_id / waste_cards[i]）
                                // → __at_num 收口（selected_card_id 株——
                                // 此前困在 has_str 分支内不触发）。
                                value = format!("__at_num(&({}))", value);
                            } else if ty == "Vec<serde_json::Value>"
                                && matches!(right.as_ref(), Expr::Array(_))
                            {
                                // 数组字面量整体赋值 → 逐元素 json!
                                // （`.col0_cards = [12, 13]` vec![i32] 株）。
                                if let Expr::Array(elems) = right.as_ref() {
                                    let items: Vec<String> = elems
                                        .iter()
                                        .map(|e| {
                                            format!(
                                                "serde_json::json!({})",
                                                self.ast_expr_to_rust_no_to_string(e)
                                            )
                                        })
                                        .collect();
                                    value = format!("vec![{}]", items.join(", "));
                                }
                            }
                        }
                    }
                    // Auto-coerce int → String when assigning to a String field
                    // e.g. .display = .val → self.display = self.val.to_string()
                    if self.ast_expr_is_string(left) && !self.ast_expr_is_string(right) {
                        value = format!("{}.to_string()", value);
                    } else if self.ast_expr_is_string(left) && self.ast_expr_is_string(right) {
                        // String-to-String assignment from a field ref needs .clone()
                        // (Rust's String doesn't impl Copy). Skip for literals/expressions
                        // that already produce owned String values.
                        let needs_clone = self.resolve_expr_name(right).is_some();
                        if needs_clone {
                            value = format!("{}.clone()", value);
                        }
                    } else {
                        // PLAN-039 T-13（批次 E）：非 Copy 态字段（Vec<Card>/
                        // Meta 等 back 型）的局部字段访问 RHS 补 clone——
                        // `.cards = r.cards` move 后 r.meta 复用株（VM 拷贝
                        // 语义的编译等价）。数值/布尔态（Copy）不动。
                        let l_state_non_copy = self
                            .resolve_expr_name(left)
                            .and_then(|n| self.state_types.get(&n))
                            .map_or(false, |ty| {
                                !matches!(ty.as_str(), "i32" | "i64" | "u32" | "u64" | "f32" | "f64" | "bool")
                            });
                        if l_state_non_copy
                            && matches!(
                                right.as_ref(),
                                Expr::Ident(_) | Expr::Dot(..)
                            )
                            && !value.contains('[')
                            && !value.starts_with('"')
                            && !value.contains(".clone()")
                        {
                            value = format!("{}.clone()", value);
                        }
                        // PLAN-039 T-13（批次 E）：typed 集合元素字段访问
                        // RHS 补 clone——`col = self.cards[i].column` move
                        // 出 Vec<Card> 索引位（String 字段株；数值字段
                        // clone 冗余但合法）。
                        if matches!(
                            right.as_ref(),
                            Expr::Dot(obj, _) if matches!(obj.as_ref(), Expr::Index(..))
                        ) && !value.contains(".clone()")
                        {
                            value = format!("{}.clone()", value);
                        }
                    }
                    return format!("{} = {}", target, value);
                }
                // Compound assignment: .count += expr → self.count += expr
                if matches!(op, Op::AddEq | Op::SubEq | Op::MulEq | Op::DivEq) {
                    let target = self.ast_expr_to_rust(left);
                    let value = self.ast_expr_to_rust(right);
                    // Check if target is a String field — need parse/add/to_string pattern
                    let target_name = self.resolve_expr_name(left);
                    if target_name.as_ref().map_or(false, |n| self.state_types.get(n).map_or(false, |ty| ty == "String")) {
                        // PLAN-036 T-02：String 字段的 += 分型——串侧（字面量/
                        // String 态源）= 追加语义 `format!`（VM 串接等价，
                        // __desktop_cmd 总线写点先例）；数值侧保留既有
                        // parse/add 数值串模式（计数器语料）。
                        if matches!(op, Op::AddEq)
                            && (self.ast_expr_is_string(right)
                                || matches!(
                                    right.as_ref(),
                                    Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_)
                                ))
                        {
                            let value = self.ast_expr_to_rust_no_to_string(right);
                            return format!("{} = format!(\"{{}}{{}}\", {}, {})", target, target, value);
                        }
                        let inner_op = match op {
                            Op::AddEq => "+",
                            Op::SubEq => "-",
                            Op::MulEq => "*",
                            Op::DivEq => "/",
                            _ => unreachable!(),
                        };
                        return format!("{} = ({}.parse::<i32>().unwrap_or(0) {} {}).to_string()", target, target, inner_op, value);
                    }
                    let op_str = match op {
                        Op::AddEq => "+=",
                        Op::SubEq => "-=",
                        Op::MulEq => "*=",
                        Op::DivEq => "/=",
                        _ => unreachable!(),
                    };
                    return format!("{} {} {}", target, op_str, value);
                }
                // String concatenation detection: use format! instead of +
                // because Rust's + only works with String + &str, not String + String
                // Check if EITHER side is a string literal (Expr::Str/CStr/FStr) — that
                // unambiguously means string concatenation, not numeric addition.
                let is_string_concat = matches!(op, Op::Add) && (
                    matches!(left.as_ref(), Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_))
                    || matches!(right.as_ref(), Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_))
                    || self.ast_expr_is_string(left)
                    || self.ast_expr_is_string(right)
                );
                if is_string_concat {
                    let left_str = self.ast_expr_to_rust_no_to_string(left);
                    let right_str = self.ast_expr_to_rust_no_to_string(right);
                    // PLAN-039 T-12（批次 E）：串接的 Value 侧降串——
                    // format! 的 `{}` 对 Value 走 Display 会带 JSON 引号
                    // （VM 串接是裸串拼接语义），包 __at_str 对齐。
                    let left_str = if self.expr_is_value_typed(left) {
                        format!("__at_str(&({}))", left_str)
                    } else {
                        left_str
                    };
                    let right_str = if self.expr_is_value_typed(right) {
                        format!("__at_str(&({}))", right_str)
                    } else {
                        right_str
                    };
                    return format!("format!(\"{{}}{{}}\", {}, {})", left_str, right_str);
                }
                let left_str = self.ast_expr_to_rust(left);
                let right_str = self.ast_expr_to_rust(right);
                // PLAN-036 T-02：Eq/Neq 跨型比较统一降串（normalize_eq_
                // neq_compare——值参闭包臂共用；VM 宽松比较语义的编译
                // 等价：bool 降 "1"/"" 串、数值补 .to_string()）。
                if let Some(cmp) =
                    self.normalize_eq_neq_compare(op, left, right, &left_str, &right_str)
                {
                    return cmp;
                }
                // PLAN-039 T-12（批次 E，E-D1 使用点包裹）：Value 操作数的
                // 数值使用点降链——比较/算术的 Value 侧包 `__at_num(&(..))`
                // （as_i64 容差，VM 宽松数值语义的编译等价；klondike
                // w_card >= 39 / % 13 株）。Eq/Neq 不入此臂：json! 元素与
                // int 元素的区分需 T-14 局部类型格（`used[i] == 0` 语料
                // 实证 int 元素误包负收益），串侧场景由
                // normalize_eq_neq_compare 降串臂先行处理。
                if matches!(
                    op,
                    Op::Lt | Op::Le | Op::Gt | Op::Ge | Op::Mod | Op::Sub | Op::Mul | Op::Div
                ) {
                    let l_val = self.expr_is_value_typed(left);
                    let r_val = self.expr_is_value_typed(right);
                    if l_val || r_val {
                        let l = if l_val {
                            format!("__at_num(&({}))", left_str)
                        } else {
                            left_str
                        };
                        let r = if r_val {
                            format!("__at_num(&({}))", right_str)
                        } else {
                            right_str
                        };
                        let num_op_str = match op {
                            Op::Lt => "<",
                            Op::Le => "<=",
                            Op::Gt => ">",
                            Op::Ge => ">=",
                            Op::Mod => "%",
                            Op::Sub => "-",
                            Op::Mul => "*",
                            _ => "/",
                        };
                        return format!("({}) {} ({})", l, num_op_str, r);
                    }
                }
                let op_str = match op {
                    Op::Add => "+",
                    Op::Sub => "-",
                    Op::Mul => "*",
                    Op::Div => "/",
                    Op::Mod => "%",
                    Op::Eq => "==",
                    Op::Neq => "!=",
                    Op::Lt => "<",
                    Op::Le => "<=",
                    Op::Gt => ">",
                    Op::Ge => ">=",
                    Op::And => "&&",
                    Op::Or => "||",
                    Op::Not => "!",
                    _ => "?",
                };
                let my_prec = bin_op_precedence(op);
                // Plan 407: wrap children in parens when needed.
                // Left child: needs parens if lower precedence.
                // Right child: needs parens if lower OR EQUAL precedence
                //   (left-associative: a % (b*c) ≠ (a%b)*c at same precedence).
                let left_wrapped = if bin_child_needs_parens(left, my_prec) {
                    format!("({})", left_str)
                } else {
                    left_str
                };
                let right_wrapped = if bin_child_needs_parens_side(right, my_prec, true) {
                    format!("({})", right_str)
                } else {
                    right_str
                };
                format!("{} {} {}", left_wrapped, op_str, right_wrapped)
            }
            Expr::Call(call) => {
                let fn_name: String = call.get_name_text_safe()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| self.ast_expr_to_rust(&call.name));
                // Plan 374 Task 2: store.Method(args) → self.store.on(StoreMsg::Method(args))
                // Only match PascalCase methods (store handlers like NewNote, TogglePin).
                // Don't match `store.notes.len()` or `store.field.lowercase()`.
                if (fn_name.starts_with("store.") || fn_name.starts_with("self.store.")) {
                    let method = if fn_name.starts_with("self.store.") {
                        &fn_name["self.store.".len()..]
                    } else {
                        &fn_name["store.".len()..]
                    };
                    // Only rewrite if it's a direct store method (no nested dots like "notes.len")
                    // and starts with uppercase (PascalCase handler name).
                    if !method.contains('.') && method.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    let args_str = self.rust_call_args_with_clone(call).join(", ");
                    let store_msg = STORE_NAMES.with(|sn| {
                        sn.borrow().values().next().cloned()
                            .map(|name| format!("{}Msg", name))
                            .unwrap_or_else(|| "StoreMsg".to_string())
                    });
                    // Plan 407 R7: when generating a handler INSIDE the store itself,
                    // use self.on(...) instead of self.store.on(...).
                    let is_in_store = STORE_NAMES.with(|sn| {
                        let cur = self.current_widget.as_deref();
                        sn.borrow().values().any(|name| Some(name.as_str()) == cur)
                    });
                    let receiver = if is_in_store { "self" } else { "self.store" };
                    if args_str.is_empty() {
                        return format!("{}.on({}::{})", receiver, store_msg, method);
                    } else {
                        return format!("{}.on({}::{}({}))", receiver, store_msg, method, args_str);
                    }
                    } // close if !method.contains('.')
                } // close if fn_name.starts_with("store.")
                // PLAN-627: 限定名 `api.X(...)`（模块形态 `use back.api`）——
                // head 为 Ident("api") 且方法名 ∈ 当前 widget 的 api 清单时按
                // 裸名同发射（命中文件级生成的 api 桩/merged 吸收 fn；清单门
                // 防用户自建同名 `api` 对象误伤）。
                if let crate::ast::Expr::Dot(obj, method) = call.name.as_ref() {
                    if let crate::ast::Expr::Ident(obj_name) = obj.as_ref() {
                        if obj_name.as_str() == "api"
                            && self.api_imports.iter().any(|f| f == method.as_str())
                        {
                            let args_str = self.rust_call_args_with_clone(call).join(", ");
                            return format!("{}({})", method.as_str(), args_str);
                        }
                    }
                }
                // PLAN-036 T-02：`.Variant(args)` 自消息发送（handler 体
                // 语句位）→ `self.on({Msg}::Variant(args))`——store.X 同型
                // 自指派臂（解释臂 call_handler 的编译等价；SendCmd 总线
                // 写点 = 消息经自身 handler 落 __desktop_cmd，
                // drain_desktop_commands 读走同径——027 D3 双轨单源）。
                if let crate::ast::Expr::Dot(obj, method) = call.name.as_ref() {
                    if matches!(obj.as_ref(), crate::ast::Expr::Ident(n) if n.as_str() == "." || n.as_str() == "self")
                        && self
                            .message_variants
                            .iter()
                            .any(|v| v.name == method.as_str())
                    {
                        let args_str = self.rust_call_args_with_clone(call).join(", ");
                        let msg = self.current_msg_name();
                        return if args_str.is_empty() {
                            format!("self.on({}::{})", msg, method.as_str())
                        } else {
                            format!("self.on({}::{}({}))", msg, method.as_str(), args_str)
                        };
                    }
                }
                // PLAN-039 T-12（批次 E，E-D4）：`.at` 内建方法翻译表——
                // `x.len()/slice/str/lower/…` 按接收者类型格发射（Value 走
                // __at_* shim、str/int 直落 Rust 同义方法）。此前方法名落
                // Dot 字段降链再挂尾巴 `()`（launcher ic1.len() 25 错株：
                // `ic1["len"].as_str()…()()`）；None 落默认路径不变。
                if let crate::ast::Expr::Dot(obj, method) = call.name.as_ref() {
                    if let Some(translated) =
                        self.try_builtin_method_call(obj, method.as_str(), call)
                    {
                        return translated;
                    }
                    // PLAN-039 T-14（E-D3 第二轨）：Value 集合的 push 实参
                    // json! 化——`moving_stack.push(waste_cards.pop()…)` /
                    // `ranked.push(srow)`（VM 无类型集合收动态值的编译
                    // 等价；typed Vec 不动）。
                    if method.as_str() == "push" {
                        // 接收者名解析收 Dot 链形态（`.f0_cards.push` 解析为
                        // Dot(Dot(self, f0_cards), push)——obj 非裸 Ident）。
                        let coll = self.resolve_expr_name(obj);
                        let is_value_coll = coll.as_ref().map_or(false, |c| {
                            self.array_locals.contains(c)
                                || self
                                    .state_types
                                    .get(c)
                                    .map_or(false, |t| t == "Vec<serde_json::Value>")
                        }) || self.receiver_is_value_collection(obj);
                        if is_value_coll && !call.args.args.is_empty() {
                            let args: Vec<String> = call
                                .args
                                .args
                                .iter()
                                .map(|a| {
                                    format!(
                                        "serde_json::json!({})",
                                        self.ast_expr_to_rust_no_to_string(&a.get_expr())
                                    )
                                })
                                .collect();
                            return format!(
                                "({}).push({})",
                                self.ast_expr_to_rust(obj),
                                args.join(", ")
                            );
                        }
                    }
                }
                let args: Vec<String> = self.rust_call_args_with_clone(call);
                match fn_name.as_str() {
                    "print" => {
                        let print_args: Vec<String> = args.iter()
                            .map(|a| a.trim_end_matches(".to_string()").to_string())
                            .collect();
                        format!("println!({})", print_args.join(", "))
                    }
                    // Plan 413 follow-up: code editor payload accessors (§3.2) —
                    // the generated handlers read live editor state by key.
                    "code_editor_text" => format!(
                        "auto_lang::ui::code_editor::code_editor_text({}).unwrap_or_default()",
                        args.first().cloned().unwrap_or_else(|| "\"editor\".to_string()".to_owned())
                    ),
                    "code_editor_cursor_line" => format!(
                        "auto_lang::ui::code_editor::code_editor_cursor({}).map(|c| c.0 as i32).unwrap_or(0)",
                        args.first().cloned().unwrap_or_else(|| "\"editor\".to_string()".to_owned())
                    ),
                    "code_editor_cursor_col" => format!(
                        "auto_lang::ui::code_editor::code_editor_cursor({}).map(|c| c.1 as i32).unwrap_or(0)",
                        args.first().cloned().unwrap_or_else(|| "\"editor\".to_string()".to_owned())
                    ),
                    "code_editor_selection_len" => format!(
                        "auto_lang::ui::code_editor::code_editor_cursor({}).map(|c| c.2 as i32).unwrap_or(0)",
                        args.first().cloned().unwrap_or_else(|| "\"editor\".to_string()".to_owned())
                    ),
                    "code_editor_find" => format!(
                        "auto_lang::ui::code_editor::code_editor_find({})",
                        args.first().cloned().unwrap_or_else(|| "\"editor\".to_string()".to_owned())
                    ),
                    "code_editor_set_text" => format!(
                        "auto_lang::ui::code_editor::code_editor_set_text({}, {})",
                        args.first().cloned().unwrap_or_else(|| "\"editor\".to_string()".to_owned()),
                        args.get(1).cloned().unwrap_or_else(|| "String::new()".to_owned())
                    ),
                    "Time.now_sec" | "time.now_sec" | "time_now_sec" => {
                        "auto_lang::vm::ffi::stdlib::shim_time_now_sec() as i32".to_string()
                    }
                    "Time.now_ms" | "time.now_ms" | "time_now_ms" => {
                        "auto_lang::vm::ffi::stdlib::shim_time_now_ms()".to_string()
                    }
                    "Time.now" | "time.now" => {
                        "auto_lang::vm::ffi::stdlib::shim_time_now()".to_string()
                    }
                    "storage.get" | "Storage.get" => {
                        let key = args.first().cloned().unwrap_or_else(|| "\"\"".to_string());
                        format!("auto_lang::vm::ffi::stdlib::shim_storage_get(({}).to_string())", key)
                    }
                    "storage.set" | "Storage.set" => {
                        let key = args.get(0).cloned().unwrap_or_else(|| "\"\"".to_string());
                        let val = args.get(1).cloned().unwrap_or_else(|| "\"\"".to_string());
                        format!("auto_lang::vm::ffi::stdlib::shim_storage_set(({}).to_string(), ({}).to_string())", key, val)
                    }
                    "storage.remove" | "Storage.remove" => {
                        let key = args.first().cloned().unwrap_or_else(|| "\"\"".to_string());
                        format!("auto_lang::vm::ffi::stdlib::shim_storage_remove(({}).to_string())", key)
                    }
                    _ => {
                        // Plan 374: Callback prop calls (on_delete, on_toggle_pin, etc.)
                        // are no-ops in Rust — child-to-parent communication uses enum wrapping.
                        if self.prop_types.get(fn_name.as_str()).map(|t| t == "msg").unwrap_or(false) {
                            return "()".to_string();
                        }
                        // Plan 374: .contains(x) where x is a String needs .as_str()
                        // because str::contains expects impl Pattern, not String.
                        if fn_name.ends_with(".contains") && args.len() == 1 {
                            let arg = &args[0];
                            let fixed_arg = if arg.ends_with(".clone()") {
                                format!("{}.as_str()", &arg[..arg.len() - ".clone()".len()])
                            } else if arg.contains("\"") {
                                arg.clone()
                            } else {
                                format!("({}).as_str()", arg)
                            };
                            let obj = &fn_name[..fn_name.len() - ".contains".len()];
                            return format!("{}.contains({})", obj, fixed_arg);
                        }
                        // findIndex(closure) → iter().position(closure).map(|i| i as i32).unwrap_or(-1)
                        if fn_name.ends_with(".findIndex") {
                            let obj = &fn_name[..fn_name.len() - ".findIndex".len()];
                            let closure_arg = args.first().map(|s| s.as_str()).unwrap_or("|_| false");
                            return format!("{}.iter().position({}).map(|i| i as i32).unwrap_or(-1)", obj, closure_arg);
                        }
                        let result = if fn_name.ends_with(".remove") {
                            // .remove() takes usize, cast args. Discard return value.
                            // Use drop() instead of `let _ =` because `let` can't be the last
                            // expression in an `if` block in Rust.
                            let casted_args: Vec<String> = args.iter()
                                .map(|a| format!("{} as usize", a))
                                .collect();
                            format!("drop({}({}))", fn_name, casted_args.join(", "))
                        } else if fn_name.ends_with(".push") {
                            // .push() for Value vectors — clone args that are value_locals
                            // to avoid borrow-after-move when the local is used later
                            let cloned_args: Vec<String> = args.iter()
                                .map(|a| {
                                    let bare = a.trim_start_matches("self.");
                                    if self.value_locals.contains(bare) {
                                        format!("{}.clone()", a)
                                    } else {
                                        a.clone()
                                    }
                                })
                                .collect();
                            format!("{}({})", fn_name, cloned_args.join(", "))
                        } else {
                            format!("{}({})", fn_name, args.join(", "))
                        };
                        // .len() returns usize — cast to i32 for AURA compatibility
                        if fn_name.ends_with(".len") {
                            format!("{} as i32", result)
                        } else if fn_name.ends_with(".pop") {
                            // Plan 407: Vec::pop returns Option<T>; unwrap_or(0) for i32
                            format!("{}.unwrap_or(0)", result)
                        } else {
                            result
                        }
                    }
                }
            }
            Expr::Object(pairs) => {
                let fields: Vec<String> = pairs.iter()
                    .map(|p| {
                        let key_str = match &p.key {
                            crate::ast::Key::NamedKey(name) => format!("\"{}\"", name.as_str()),
                            crate::ast::Key::IntKey(i) => i.to_string(),
                            crate::ast::Key::BoolKey(b) => b.to_string(),
                            crate::ast::Key::StrKey(s) => format!("\"{}\"", s),
                        };
                        let value = self.ast_expr_to_json_value(&p.value);
                        format!("{}: {}", key_str, value)
                    })
                    .collect();
                format!("serde_json::json!({{{}}})", fields.join(", "))
            }
            Expr::Array(elems) => {
                let elems_str: Vec<String> = elems.iter()
                    .map(|e| self.ast_expr_to_rust(e))
                    .collect();
                format!("vec![{}]", elems_str.join(", "))
            }
            Expr::Index(target, index) => {
                let target_str = self.ast_expr_to_rust(target);
                let index_str = self.ast_expr_to_rust(index);
                // Vec<Value> requires usize index — cast non-literal indexes to usize
                // since handler vars are typically i32 from findIndex or loop counters
                let index_cast = if index_str.parse::<usize>().is_ok() {
                    index_str // literal usize, no cast needed
                } else {
                    // Plan 407: parenthesize the expression so `as usize` binds
                    // to the whole index, not just the last operand.
                    format!("({}) as usize", index_str)
                };
                // PLAN-026 T-03 配套: List<int> 局部变量元素窄化 as i32
                // (db 层 Vec<i64> 惯例;与 i32 模型/字面语境对齐,值域
                // 终端尺度,截断无损)。
                if let crate::ast::Expr::Ident(n) = target.as_ref() {
                    if self.handler_int_list_vars.borrow().contains(n.as_str()) {
                        return format!("({}[{}] as i32)", target_str, index_cast);
                    }
                }
                format!("{}[{}]", target_str, index_cast)
            }
            Expr::Unary(op, operand) => {
                let val = self.ast_expr_to_rust(operand);
                match op {
                    Op::Not => format!("!({})", val),
                    Op::Sub => format!("-{}", val),
                    _ => format!("/* unimplemented unary {:?} */", op),
                }
            }
            Expr::Closure(closure) => {
                // (t => t.id == id) → |t| t["id"].as_i64().unwrap_or(0) as i32 == id
                // Closure params from findIndex/.position() iterate over &Value,
                // so any dot access on a closure param needs bracket access.
                let param_names: Vec<String> = closure.params.iter()
                    .map(|p| p.name.as_str().to_string())
                    .collect();
                // Temporarily register closure params as value loop vars so that
                // dot access on them gets converted to bracket access.
                // We can't mutate self, so we handle it inline by checking the
                // param names during Dot processing.
                // Instead, we convert the closure body manually with param awareness.
                let body = self.ast_expr_to_rust_with_value_params(&closure.body, &param_names);
                format!("|{}| {}", param_names.join(", "), body)
            }
            Expr::FStr(fstr) => {
                // f"${.active_count} items left" → format!("{} items left", self.active_count)
                let mut fmt_str = String::new();
                let mut args = Vec::new();
                for part in &fstr.parts {
                    match part {
                        Expr::Str(s) | Expr::CStr(s) => {
                            fmt_str.push_str(&s.as_str().replace('{', "{{").replace('}', "}}"));
                        }
                        _ => {
                            fmt_str.push_str("{}");
                            args.push(self.ast_expr_to_rust(part));
                        }
                    }
                }
                if args.is_empty() {
                    format!("\"{}\".to_string()", fmt_str)
                } else {
                    format!("format!(\"{}\", {})", fmt_str, args.join(", "))
                }
            }
            Expr::Range(range) => {
                let start = self.ast_expr_to_rust(&range.start);
                let end = self.ast_expr_to_rust(&range.end);
                if range.eq {
                    format!("{}..={}", start, end)
                } else {
                    format!("{}..{}", start, end)
                }
            }
            Expr::Nil | Expr::Null => "serde_json::Value::Null".to_string(),
            Expr::None => "None".to_string(),
            Expr::Some(e) => {
                let inner = self.ast_expr_to_rust(e);
                format!("Some({})", inner)
            }
            Expr::If(if_expr) => {
                // Convert if-expression to Rust if/else expression.
                // Used for conditional style values like: style: if active { "x" } else { "y" }
                let cond = if let Some(branch) = if_expr.branches.first() {
                    self.ast_expr_to_rust(&branch.cond)
                } else {
                    "true".to_string()
                };
                let then_body = if let Some(branch) = if_expr.branches.first() {
                    branch.body.stmts.iter()
                        .filter_map(|s| {
                            if let crate::ast::Stmt::Expr(e) = s {
                                Some(self.ast_expr_to_rust(e))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("")
                } else {
                    String::new()
                };
                let else_body = if let Some(else_b) = &if_expr.else_ {
                    else_b.stmts.iter()
                        .filter_map(|s| {
                            if let crate::ast::Stmt::Expr(e) = s {
                                Some(self.ast_expr_to_rust(e))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("")
                } else {
                    String::new()
                };
                if else_body.is_empty() {
                    // PLAN-036 T-02：值位 if 表达式缺 else = 空值（VM 语义）
                    // ——补 `else { "" }` 防 E0317（值位必须有 else；条件样式
                    // `style: if .x { "A" }` 无 else 先例）。
                    format!("if {} {{ {} }} else {{ \"\".to_string() }}", cond, then_body)
                } else {
                    format!("if {} {{ {} }} else {{ {} }}", cond, then_body, else_body)
                }
            }
            Expr::Block(body) => {
                // Plan 043 M5 #2: render a multi-statement computed body as a
                // Rust block `{ stmt; ...; tail }`. The final `return e;` becomes
                // the trailing expression `e`; any other trailing expression
                // statement is used as-is. Statements in between are joined by
                // `; ` (ast_stmt_to_rust already omits the trailing semicolon).
                let n = body.stmts.len();
                let mut parts: Vec<String> = Vec::with_capacity(n);
                for (i, stmt) in body.stmts.iter().enumerate() {
                    let is_last = i + 1 == n;
                    match stmt {
                        crate::ast::Stmt::Return(expr) if is_last => {
                            parts.push(self.ast_expr_to_rust(expr));
                        }
                        _ => parts.push(self.ast_stmt_to_rust(stmt)),
                    }
                }
                format!("{{ {} }}", parts.join("; "))
            }
            _ => format!("/* expr */"),
        }
    }

    /// Generate a json!()-compatible value expression (strings without .to_string())
    fn ast_expr_to_json_value(&self, expr: &crate::ast::Expr) -> String {
        use crate::ast::Expr;
        match expr {
            Expr::Str(s) => format!("\"{}\"", Self::rust_str_lit_body(s)),
            Expr::I64(n) => n.to_string(),
            Expr::Int(n) => n.to_string(),
            Expr::U64(n) => n.to_string(),
            Expr::Uint(n) => n.to_string(),
            Expr::Bool(b) => b.to_string(),
            Expr::Ident(name) => {
                let s = name.as_str();
                // Plan 374 Task 2: store composable rewriting
                if s == "store" || s == ".store" {
                    return "self.store".to_string();
                }
                if s.starts_with(".store.") {
                    return format!("self.{}", &s[1..]);
                }
                if s.starts_with('.') {
                    format!("self.{}", &s[1..])
                } else if self.state_types.contains_key(s) || self.prop_names.contains(s) {
                    format!("self.{}", s)
                } else {
                    s.to_string()
                }
            }
            Expr::Object(pairs) => {
                let fields: Vec<String> = pairs.iter()
                    .map(|p| {
                        let key_str = match &p.key {
                            crate::ast::Key::NamedKey(name) => format!("\"{}\"", name.as_str()),
                            crate::ast::Key::IntKey(i) => i.to_string(),
                            _ => String::new(),
                        };
                        let value = self.ast_expr_to_json_value(&p.value);
                        format!("{}: {}", key_str, value)
                    })
                    .collect();
                format!("serde_json::json!({{{}}})", fields.join(", "))
            }
            _ => self.ast_expr_to_rust(expr),
        }
    }

    /// Convert Auto type to Rust type
    fn auto_type_to_rust(&self, ty: &crate::ast::Type) -> String {
        use crate::ast::Type;
        match ty {
            Type::Int => "i32".to_string(),
            Type::Uint => "u32".to_string(),
            Type::I64 => "i64".to_string(),
            Type::U64 => "u64".to_string(),
            Type::Float => "f32".to_string(),
            Type::Double => "f64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::StrFixed(_) | Type::StrOwned | Type::StrSlice => "String".to_string(),
            Type::Void => "()".to_string(),
            Type::Array(arr) => format!("Vec<{}>", self.auto_type_to_rust(&arr.elem)),
            Type::RuntimeArray(arr) => format!("Vec<{}>", self.auto_type_to_rust(&arr.elem)),
            Type::List(inner) => format!("Vec<{}>", self.auto_type_to_rust(inner)),
            Type::Slice(sl) => format!("Vec<{}>", self.auto_type_to_rust(&sl.elem)),
            Type::Map(k, v) => format!("std::collections::HashMap<{}, {}>", self.auto_type_to_rust(k), self.auto_type_to_rust(v)),
            Type::User(td) => td.name.to_string(),
            Type::Unknown => "serde_json::Value".to_string(),
            _ => "serde_json::Value".to_string(), // Fallback for unrecognized types
        }
    }
}

/// Plan 371 Task 21: whether a rust field type is a scalar we can safely emit
/// into `state_snapshot()`. Collections (`Vec<...>`, `serde_json::Value`) and
/// nested components are excluded — their shape is not a clean scalar.
fn is_scalar_state_type(ty: &str) -> bool {
    matches!(
        ty.trim(),
        "String"
            | "i8" | "i16" | "i32" | "i64" | "isize"
            | "u8" | "u16" | "u32" | "u64" | "usize"
            | "f32" | "f64"
            | "bool"
    )
}

/// Plan 371 Task 21: render the rust expression that converts `<receiver>.<field>`
/// (of the given scalar rust type) into an `auto_val::Value`.
fn scalar_to_auto_value_expr(receiver: &str, field: &str, ty: &str) -> String {
    let val_expr = format!("{}.{}", receiver, field);
    match ty.trim() {
        "String" => format!("auto_lang::ui::auto_val::Value::str(&{})", val_expr),
        "bool" => format!("auto_lang::ui::auto_val::Value::Bool({})", val_expr),
        "i32" | "u32" => format!("auto_lang::ui::auto_val::Value::Int({})", val_expr),
        "i8" | "i16" | "u8" | "u16" | "isize" | "usize" | "i64" | "u64" => {
            format!("auto_lang::ui::auto_val::Value::Int({} as i32)", val_expr)
        }
        "f32" | "f64" => format!("auto_lang::ui::auto_val::Value::Float({} as f64)", val_expr),
        _ => "auto_lang::ui::auto_val::Value::Nil".to_string(),
    }
}

/// Plan 371 步骤3: Sanitize an Auto identifier for use as a Rust identifier.
/// Handles two classes of conflict:
///   1. Rust reserved keywords (type, match, fn, crate, move, self, ...) →
///      prefix with `r#` (Rust raw identifier syntax).
///   2. Rust macro/type names that would shadow loop variables (todo, vec,
///      format, println, String, Vec, ...) → append `_` suffix.
/// Applied to `for`-loop variables so `for todo in ...` doesn't collide with
/// the `todo!()` macro.
fn sanitize_rust_ident(name: &str) -> String {
    // Rust reserved keywords (2021 edition) that can't be bare identifiers.
    const RUST_KEYWORDS: &[&str] = &[
        "as", "break", "const", "continue", "crate", "dyn", "else", "enum",
        "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop",
        "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self",
        "static", "struct", "super", "trait", "true", "type", "unsafe", "use",
        "where", "while", "async", "await", "box",
    ];
    // Common Rust macros/types that conflict with plausible loop var names.
    const RUST_MACROS_TYPES: &[&str] = &[
        "todo", "unimplemented", "panic", "vec", "format", "println", "print",
        "eprintln", "eprint", "dbg", "assert", "String", "Vec", "Option",
        "Result", "Box",
    ];
    if RUST_KEYWORDS.contains(&name) {
        format!("r#{}", name)
    } else if RUST_MACROS_TYPES.contains(&name) {
        format!("{}_", name)
    } else {
        name.to_string()
    }
}

/// Extract field name from `Expr::Dot(Expr::Ident("self"), Name("field"))`.
/// Returns `None` if the pattern doesn't match.
fn extract_dot_self_field(expr: &crate::ast::Expr) -> Option<String> {
    if let crate::ast::Expr::Dot(obj, field) = expr {
        if let crate::ast::Expr::Ident(name) = obj.as_ref() {
            if name.as_str() == "self" {
                return Some(field.as_str().to_string());
            }
        }
    }
    None
}

/// Extract function name from `Expr::Call(...)`.
fn extract_call_name(expr: &crate::ast::Expr) -> Option<String> {
    if let crate::ast::Expr::Call(call) = expr {
        call.get_name_text_safe().map(|s| s.to_string())
    } else {
        None
    }
}

/// Plan 371 L1: Check if a statement mutates the store data that feeds child
/// component props. Detects:
///   1. Assignment to `store.active_id` or `store.notes` (the canonical props
///      that back a child's `note` prop via `notes[active_id]`).
///   2. A call to a mutating store method via `store.Xxx(...)` — the store's
///      `.on` handler changes `notes`/`active_id` internally.
/// Recurses into control-flow bodies (if/for/block) so nested mutations count.
fn stmt_mutates_store_data(stmt: &crate::ast::Stmt) -> bool {
    use crate::ast::{Expr, Stmt};
    use auto_val::Op;

    match stmt {
        // Assignment: lhs = rhs. Check if lhs targets store data.
        Stmt::Expr(Expr::Bina(left, op, _)) if matches!(op, Op::Asn) => {
            // Pattern: store.active_id = ...  → Dot(Ident("store"), "active_id")
            //          store.notes = ...       → Dot(Ident("store"), "notes")
            if let Expr::Dot(obj, field) = left.as_ref() {
                if let Expr::Ident(obj_name) = obj.as_ref() {
                    let f = field.as_str();
                    if (obj_name.as_str() == "store")
                        && (f == "active_id" || f == "notes")
                    {
                        return true;
                    }
                }
            }
            false
        }
        // Call: store.NewNote(...), store.TogglePin(...), etc.
        // These route to the store's `.on` handler which mutates notes/active_id.
        // The call name is an Expr::Dot(Ident("store"), method) — check the AST
        // structure directly (get_name_text_safe format is unreliable here).
        Stmt::Expr(Expr::Call(call)) => {
            if let Expr::Dot(obj, _method) = &*call.name {
                if let Expr::Ident(obj_name) = obj.as_ref() {
                    if obj_name.as_str() == "store" {
                        return true;
                    }
                }
            }
            false
        }
        // Recurse into control flow.
        Stmt::Expr(Expr::If(if_stmt)) => {
            if_stmt.branches.iter().any(|b| b.body.stmts.iter().any(stmt_mutates_store_data))
                || if_stmt.else_.as_ref().map_or(false, |e| e.stmts.iter().any(stmt_mutates_store_data))
        }
        Stmt::For(for_stmt) => for_stmt.body.stmts.iter().any(stmt_mutates_store_data),
        Stmt::Block(body) => body.stmts.iter().any(stmt_mutates_store_data),
        _ => false,
    }
}

/// Return precedence level for binary operators (higher = tighter binding)
fn bin_op_precedence(op: &auto_val::Op) -> u8 {
    use auto_val::Op;
    match op {
        Op::Mul | Op::Div | Op::Mod => 5,
        Op::Add | Op::Sub => 4,
        Op::Eq | Op::Neq | Op::Lt | Op::Le | Op::Gt | Op::Ge => 3,
        Op::And => 2,
        Op::Or => 1,
        _ => 0,
    }
}

fn bin_child_needs_parens(expr: &crate::ast::Expr, parent_prec: u8) -> bool {
    bin_child_needs_parens_side(expr, parent_prec, false)
}

fn bin_child_needs_parens_side(expr: &crate::ast::Expr, parent_prec: u8, is_right: bool) -> bool {
    use crate::ast::Expr;
    use auto_val::Op;
    if let Expr::Bina(_, child_op, _) = expr {
        let child_prec = bin_op_precedence(child_op);
        if matches!(child_op, Op::Asn | Op::AddEq | Op::SubEq | Op::MulEq | Op::DivEq) {
            return false;
        }
        // Plan 407: right child needs parens at same precedence too (left-assoc).
        if is_right {
            child_prec <= parent_prec
        } else {
            child_prec < parent_prec
        }
    } else {
        false
    }
}

impl BackendGenerator for RustGenerator {
    fn generate(&mut self, widget: &AuraWidget) -> GenResult<String> {
        self.generate_rust(widget)
    }

    fn extension(&self) -> &'static str {
        "rs"
    }
}

impl Default for RustGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a Tailwind class string (e.g. "gap-4 p-4 bg-white items-center")
/// into chained method calls on a builder expression (e.g. ".gap(4).p(4).bg(\"white\").items_center()").
///
/// Classes that are not recognized are silently skipped so the generated code
/// always compiles.
#[allow(dead_code)]
fn tailwind_to_methods(builder: &str, class_str: &str) -> String {
    let mut result = builder.to_string();
    let mut residual_classes: Vec<&str> = Vec::new();

    for class in class_str.split_whitespace() {
        let method = tailwind_single_to_method(class);
        if method.is_empty() {
            residual_classes.push(class);
        } else {
            result.push_str(&method);
        }
    }

    // Pass through unrecognized classes as a .style() call
    if !residual_classes.is_empty() {
        result.push_str(&format!(".style(\"{}\")", residual_classes.join(" ")));
    }

    result
}

/// Convert a single Tailwind class token to a builder method call string.
#[allow(dead_code)]
fn tailwind_single_to_method(class: &str) -> String {
    // --- Spacing ---
    if let Some(rest) = class.strip_prefix("p-") {
        if rest == "0" { return ".p(0)".to_string(); }
        if let Ok(n) = rest.parse::<u16>() { return format!(".p({})", n); }
    }
    if let Some(rest) = class.strip_prefix("px-") {
        if let Ok(n) = rest.parse::<u16>() { return format!(".px({})", n); }
    }
    if let Some(rest) = class.strip_prefix("py-") {
        if let Ok(n) = rest.parse::<u16>() { return format!(".py({})", n); }
    }
    if let Some(rest) = class.strip_prefix("m-") {
        if let Ok(n) = rest.parse::<u16>() { return format!(".m({})", n); }
    }
    if let Some(rest) = class.strip_prefix("mx-") {
        if let Ok(n) = rest.parse::<u16>() { return format!(".mx({})", n); }
    }
    if let Some(rest) = class.strip_prefix("my-") {
        if let Ok(n) = rest.parse::<u16>() { return format!(".my({})", n); }
    }
    if let Some(rest) = class.strip_prefix("gap-") {
        if let Ok(n) = rest.parse::<u16>() { return format!(".gap({})", n); }
    }

    // --- Colors ---
    if let Some(color) = class.strip_prefix("bg-") {
        return format!(".bg(\"{}\")", color);
    }
    // text-{color} must come after text size/alignment checks below,
    // but we handle it here and let the ordering in match below
    // override for known text- keywords.

    // --- Sizing ---
    if class == "w-full" { return ".w_full()".to_string(); }
    if let Some(rest) = class.strip_prefix("w-") {
        if let Ok(n) = rest.parse::<u16>() { return format!(".w({})", n); }
    }
    if class == "h-full" { return ".h_full()".to_string(); }
    if let Some(rest) = class.strip_prefix("h-") {
        if let Ok(n) = rest.parse::<u16>() { return format!(".h({})", n); }
    }

    // --- Layout ---
    match class {
        "flex" => return ".flex()".to_string(),
        "flex-1" => return ".flex1()".to_string(),
        "flex-row" => return ".flex_row()".to_string(),
        "flex-col" => return ".flex_col()".to_string(),
        "items-center" => return ".items_center()".to_string(),
        "items-start" => return ".items_start()".to_string(),
        "items-end" => return ".items_end()".to_string(),
        "justify-center" => return ".justify_center()".to_string(),
        "justify-between" => return ".justify_between()".to_string(),
        "justify-start" => return String::new(), // no direct method, skip
        "justify-end" => return String::new(),    // no direct method, skip
        _ => {}
    }

    // --- Border radius ---
    match class {
        "rounded" => return ".rounded()".to_string(),
        "rounded-sm" => return ".rounded_sm()".to_string(),
        "rounded-md" => return ".rounded_md()".to_string(),
        "rounded-lg" => return ".rounded_lg()".to_string(),
        _ => {}
    }

    // --- Border ---
    if class == "border" { return ".border()".to_string(); }

    // --- Typography (text size) ---
    match class {
        "text-xs" | "text-sm" | "text-base" | "text-lg" | "text-xl" | "text-2xl" | "text-3xl" => {
            // These are font-size utilities; for now emit as a comment-style pass-through.
            // They have no direct builder method on layout builders.
            return String::new();
        }
        _ => {}
    }

    // --- Font weight ---
    match class {
        "font-bold" => return ".font_bold()".to_string(),
        "font-medium" => return ".font_medium()".to_string(),
        "font-normal" => return String::new(),
        _ => {}
    }

    // --- Text alignment ---
    match class {
        "text-center" | "text-left" | "text-right" => return String::new(),
        _ => {}
    }

    // --- Text color (must come after text-size/align) ---
    if let Some(color) = class.strip_prefix("text-") {
        return format!(".text_color(\"{}\")", color);
    }

    // --- Effects ---
    match class {
        "shadow" | "shadow-sm" | "shadow-md" | "shadow-lg" | "shadow-xl" | "shadow-2xl" | "shadow-none" => {
            return String::new(); // no direct builder method yet
        }
        _ => {}
    }

    // --- Opacity ---
    if class.starts_with("opacity-") { return String::new(); }

    // --- Position ---
    if class == "relative" || class == "absolute" { return String::new(); }

    // --- Z-index ---
    if class.starts_with("z-") { return String::new(); }

    // --- Overflow ---
    if class.starts_with("overflow") { return String::new(); }

    // --- Grid ---
    if class == "grid" || class.starts_with("grid-") { return String::new(); }
    if class.starts_with("col-") || class.starts_with("row-") { return String::new(); }

    // Unknown class -- skip silently
    String::new()
}

/// Extract the collection expression from the first `[...]` index access in
/// `args`, if any. E.g. `__self.store.notes[__self.store.active_id as usize].clone()`
/// → `Some("__self.store.notes")`. Used to guard persistent-child re-construction
/// against empty collections in async-load modes.
fn first_indexed_collection(args: &str) -> Option<String> {
    let brack = args.find('[')?;
    // Walk back from the `[` to find the collection expression: a run of
    // identifier chars, `.`, and `_` (field/index path like __self.store.notes).
    let bytes = args.as_bytes();
    let mut start = brack;
    while start > 0 {
        let c = bytes[start - 1];
        if c.is_ascii_alphanumeric() || c == b'_' || c == b'.' {
            start -= 1;
        } else {
            break;
        }
    }
    let collection = &args[start..brack];
    if collection.is_empty() {
        None
    } else {
        Some(collection.to_string())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Type;
    use crate::aura::{AuraMessage, AuraStateDef};
    use std::collections::HashMap;

    #[test]
    fn test_rust_generator_creation() {
        let gen = RustGenerator::new();
        assert!(gen.current_widget.is_none());
    }

    /// Plan 547 Task 22: Rust generator preserves ImageSurface props and all
    /// five typed event hooks, including state-backed values.
    #[cfg(feature = "ui-iced")]
    #[test]
    fn image_surface_rust_codegen() {
        let msg = |name: &str| AuraMsgVariant {
            payload_names: vec![],
            name: name.to_string(),
            quoted: false,
            payload: vec![],
        };
        let widget = AuraWidget {
            named_views: Vec::new(),
            actions: None,
            timers: Vec::new(),
            name: "ImageViewer".to_string(),
            state_vars: vec![
                AuraStateDef { name: "asset_src".to_string(), type_info: Type::StrOwned, initial: crate::ast::Expr::Str("/media/a".into()), decorators: vec![] },
                AuraStateDef { name: "viewport_width".to_string(), type_info: Type::Int, initial: crate::ast::Expr::Int(640), decorators: vec![] },
                AuraStateDef { name: "fit_mode".to_string(), type_info: Type::StrOwned, initial: crate::ast::Expr::Str("contain".into()), decorators: vec![] },
                AuraStateDef { name: "zoom".to_string(), type_info: Type::Float, initial: crate::ast::Expr::Float(1.0, "1.0".into()), decorators: vec![] },
            ],
            messages: vec![AuraMessage { variants: vec![
                msg("ImageLoaded"), msg("ImageFailed"), msg("ZoomAt"), msg("PanBy"), msg("ToggleOneToOne"),
            ] }],
            view_tree: AuraNode::element("imagesurface")
                .with_prop("src", crate::ast::Expr::Ident(".asset_src".into()))
                .with_prop("width", crate::ast::Expr::Ident(".viewport_width".into()))
                .with_prop("fit", crate::ast::Expr::Ident(".fit_mode".into()))
                .with_prop("zoom", crate::ast::Expr::Ident(".zoom".into()))
                .with_prop("alt", crate::ast::Expr::Str("hero".into()))
                .with_event("onload", ".ImageLoaded")
                .with_event("onerror", ".ImageFailed")
                .with_event("onwheel", ".ZoomAt")
                .with_event("onpan", ".PanBy")
                .with_event("ondblclick", ".ToggleOneToOne"),
            handlers: std::collections::BTreeMap::new(),
            props: vec![],
            computed: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
            setup: None,
        };
        let mut gen = RustGenerator::new();
        let code = gen.generate(&widget).expect("rust generation");
        assert!(code.contains("View::image_surface("), "missing constructor:\n{code}");
        assert!(code.contains("self.asset_src.clone()"), "src state binding lost:\n{code}");
        assert!(code.contains("self.viewport_width"), "width state binding lost:\n{code}");
        assert!(code.contains("self.fit_mode.clone()"), "fit state binding lost:\n{code}");
        assert!(code.contains("image_surface_props"), "scalar props missing:\n{code}");
        assert!(code.contains("image_surface_events(Some(ImageViewerMsg::ImageFailed)"), "error event missing:\n{code}");
        assert!(code.contains("Some(ImageViewerMsg::ImageLoaded)"), "load event missing:\n{code}");
        assert!(code.contains("Some(ImageViewerMsg::ZoomAt)"), "wheel event missing:\n{code}");
        assert!(code.contains("Some(ImageViewerMsg::PanBy)"), "pan event missing:\n{code}");
        assert!(code.contains("Some(ImageViewerMsg::ToggleOneToOne)"), "double-click event missing:\n{code}");
    }

    /// Plan 547 desktop validation: a declarative `timer { ... }` entry must
    /// become the named periodic message used by the standalone Rust/Iced
    /// runner. Without this bridge the image viewer remains stuck in loading
    /// because its async media session is never polled for readiness.
    #[test]
    fn timer_block_rust_codegen_emits_named_tick_subscription() {
        let src = r#"
widget App {
    msg { SettleTick }
    model { var status str = "loading" }
    timer { SettleTick (every_ms: 80) }
    view { text .status }
    on { .SettleTick -> { .status = "ready" } }
}
"#;
        let session = crate::session::CompilerSession::ui().with_backend("rust");
        let mut parser = crate::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast.stmts.iter().find_map(|s| match s {
            crate::ast::Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        }).expect("widget decl");
        let widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");
        let code = RustGenerator::new().generate(&widget).expect("generate");

        assert!(
            code.contains("fn tick_interval_ms(&self) -> Option<u32> { Some(80) }")
                && code.contains("fn tick_msg(&self) -> Option<AppMsg> { Some(AppMsg::SettleTick) }")
        );
        assert!(!code.contains("AppMsg::Tick"), "timer entry must retain its message name");
    }

    /// Plan 436 T1(决策 1-A):带 setup 前导槽的 widget 在 Rust 目标显式
    /// 报错(PLAN-037 T7 哲学),不再静默丢弃 setup 语义。
    #[test]
    fn test_setup_block_rejected_on_rust_target() {
        let widget = AuraWidget {
            named_views: Vec::new(),
            actions: None,
            name: "SetupWidget".to_string(),
            state_vars: vec![],
            messages: vec![],
            view_tree: AuraNode::element("col"),
            handlers: std::collections::BTreeMap::new(),
            props: vec![],
            computed: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            timers: Vec::new(),
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
            setup: Some(crate::ast::ui::SetupBlock {
                body: crate::ast::Body {
                    stmts: vec![],
                    has_new_line: false,
                    source_lines: vec![],
                },
                ref_annotations: vec![],
            }),
        };
        let mut gen = RustGenerator::new();
        let err = gen.generate(&widget).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("setup"), "{msg}");
        assert!(msg.contains("SetupWidget"), "{msg}");
        assert!(msg.contains("vue-render"), "{msg}");
    }

    #[test]
    fn test_simple_counter() {
        let widget = AuraWidget {
            named_views: Vec::new(),
            actions: None,
            timers: Vec::new(),
            name: "Counter".to_string(),
            state_vars: vec![AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: crate::ast::Expr::Int(0),
                decorators: vec![],
            }],
            messages: vec![AuraMessage {
                variants: vec![
                    AuraMsgVariant { payload_names: vec![], name: "Inc".to_string(), quoted: false, payload: vec![] },
                    AuraMsgVariant { payload_names: vec![], name: "Dec".to_string(), quoted: false, payload: vec![] },
                ],
            }],
            view_tree: AuraNode::element("col")
                .with_child(AuraNode::text("Count: 0")),
            handlers: std::collections::BTreeMap::new(),
            props: vec![],
            computed: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
            setup: None,
        }
;

        let mut gen = RustGenerator::new();
        let code = gen.generate(&widget).unwrap();

        assert!(code.contains("pub enum CounterMsg"), "got:\n{}", code);
        assert!(code.contains("Inc"), "got:\n{}", code);
        assert!(code.contains("Dec"), "got:\n{}", code);
        assert!(code.contains("pub struct Counter"), "got:\n{}", code);
        assert!(code.contains("pub count: i32"), "got:\n{}", code);
        assert!(code.contains("impl Component for Counter"), "got:\n{}", code);
    }

    /// Plan 450 / 019 批次三: AutoDown 面板词汇 a2r 发射断言。面板 tag 此前
    /// 落 tag_to_view_fn 的 `_ => "col"` fallback —— 内容静默丢弃;现在发射
    /// 组合 View(与 VM 侧 aura_view_builder 面板臂同款降级)。
    fn autodown_panel_widget(view_tree: AuraNode) -> AuraWidget {
        AuraWidget {
            named_views: Vec::new(),
            name: "PanelDoc".to_string(),
            state_vars: vec![],
            messages: vec![],
            timers: Vec::new(),
            view_tree,
            handlers: std::collections::BTreeMap::new(),
            props: vec![],
            computed: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
            setup: None,
            actions: None,
        }
    }

    #[test]
    fn test_autodown_panel_heading_codegen() {
        // 字面量 level → 静态样式。
        let widget = autodown_panel_widget(
            AuraNode::element("heading")
                .with_prop("level", crate::ast::Expr::Int(2))
                .with_prop("text", crate::ast::Expr::Str("Title".into())),
        );
        let code = RustGenerator::new().generate(&widget).unwrap();
        assert!(
            code.contains("View::text_styled(\"Title\".to_string(), \"text-3xl font-bold tracking-tight text-primary mt-8 mb-4\")"),
            "got:\n{}", code
        );

        // Str 数字 level 同样走静态(parse);动态表达式 → 全臂 match。
        let widget = autodown_panel_widget(
            AuraNode::element("heading")
                .with_prop("level", crate::ast::Expr::Ident("lvl".into()))
                .with_prop("text", crate::ast::Expr::Str("T".into())),
        );
        let code = RustGenerator::new().generate(&widget).unwrap();
        assert!(code.contains("match lvl {"), "got:\n{}", code);
        assert!(code.contains("_ => \"text-sm font-semibold mb-1\""), "got:\n{}", code);
    }

    #[test]
    fn test_autodown_panel_quote_callout_codegen() {
        let widget = autodown_panel_widget(
            AuraNode::element("quote").with_child(AuraNode::text("cited")),
        );
        let code = RustGenerator::new().generate(&widget).unwrap();
        assert!(code.contains("View::container("), "got:\n{}", code);
        assert!(
            code.contains(".style(\"border-l-4 pl-4 py-2 w-full text-muted-foreground\")"),
            "got:\n{}", code
        );

        let widget = autodown_panel_widget(
            AuraNode::element("callout")
                .with_prop("kind", crate::ast::Expr::Str("warning".into()))
                .with_prop("title", crate::ast::Expr::Str("小心".into()))
                .with_child(AuraNode::text("body")),
        );
        let code = RustGenerator::new().generate(&widget).unwrap();
        assert!(code.contains("View::container(View::col()"), "got:\n{}", code);
        assert!(code.contains("border-amber-500/40 bg-amber-500/10"), "got:\n{}", code);
        assert!(code.contains("text-amber-400"), "got:\n{}", code);
    }

    #[test]
    fn test_autodown_panel_details_and_registry_slots_codegen() {
        let widget = autodown_panel_widget(
            AuraNode::element("details")
                .with_prop("summary", crate::ast::Expr::Str("展开".into()))
                .with_child(AuraNode::text("hidden")),
        );
        let code = RustGenerator::new().generate(&widget).unwrap();
        assert!(
            code.contains("auto_lang::ui::view::AccordionItem::new(\"展开\".to_string())"),
            "got:\n{}", code
        );
        assert!(code.contains(".with_expanded(true)"), "got:\n{}", code);
        assert!(code.contains("View::accordion().items(vec!["), "got:\n{}", code);

        let widget = autodown_panel_widget(
            AuraNode::element("embed_block")
                .with_prop("target", crate::ast::Expr::Str("blk-1".into())),
        );
        let code = RustGenerator::new().generate(&widget).unwrap();
        assert!(
            code.contains("format!(\"↪ {}\", \"blk-1\".to_string())"),
            "got:\n{}", code
        );
        assert!(code.contains(".style(\"rounded-lg border bg-muted p-3 w-full\")"), "got:\n{}", code);
    }

    /// Plan 448 B1: inline-lambda events through the real pipeline
    /// (parse → extract → generate). The minted `__evt_*` variant must reach
    /// the generated enum — generate_on_method silently skips handlers whose
    /// variant is missing from the enum, so that is the load-bearing
    /// assertion — the match arm carries the lambda body, and the view
    /// builder dispatches the synthetic variant.
    #[test]
    fn test_inline_lambda_event_rust_codegen() {
        let src = r#"
widget Counter {
    model { var count int = 0 }
    view {
        row {
            button "+" { onclick: () => {.count += 1} }
        }
    }
}
"#;
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast.stmts.iter().find_map(|s| match s {
            crate::ast::Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        }).expect("widget decl");
        let widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");

        let mut gen = RustGenerator::new();
        let code = gen.generate(&widget).unwrap();

        assert!(
            code.contains("__evt_onclick_1,"),
            "minted variant in the enum:\n{}",
            code
        );
        assert!(
            code.contains("__evt_onclick_1 => {"),
            "match arm for the minted variant:\n{}",
            code
        );
        assert!(
            code.contains("CounterMsg::__evt_onclick_1"),
            "dispatch closure references the variant:\n{}",
            code
        );
        assert!(code.contains("self.count += 1"), "lambda body:\n{}", code);
    }

    /// PLAN-025 T-03: slider codegen golden（fixture 真源：
    /// tests/fixtures/025-native-input/slider.at——View::slider 构造 +
    /// f32 载荷变体闭包（PLAN-661 T-02: SliderChangeHandler newtype，
    /// .on_change 包装）+ on() 载荷臂，零 thread-local 回写）。
    #[test]
    fn test_slider_codegen_arm_fixture() {
        let src = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/025-native-input/slider.at"
        ))
        .expect("read slider fixture");
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(src.as_str()).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast.stmts.iter().find_map(|s| match s {
            crate::ast::Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        })
        .expect("widget decl");
        let widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");

        let mut gen = RustGenerator::new();
        let code = gen.generate(&widget).unwrap();

        assert!(
            code.contains("View::slider("),
            "slider 构造在册:
{}",
            code
        );
        assert!(
            // PLAN-661 T-02 新发射形态：构造器双参 + .on_change 闭包
            // （SliderChangeHandler 由 builder 包装），旧 fn 指针三参
            // 形态退役。
            code.contains(".on_change(|v| SliderBoxMsg::SetVol(v))"),
            "on_change 闭包 = 载荷变体构造器包装:
{}",
            code
        );
        assert!(
            code.contains("SliderBoxMsg::SetVol"),
            "载荷变体构造器在册:
{}",
            code
        );
        assert!(
            code.contains("SetVol(f32)"),
            "载荷变体 f32:
{}",
            code
        );
        assert!(
            code.contains(".step(1.0)"),
            "step prop 消费:
{}",
            code
        );
        assert!(
            code.contains("SetVol(v") && code.contains("self.vol = v"),
            "on() 载荷臂绑定 v 写 vol:
{}",
            code
        );
        assert!(
            !code.contains("last_input_text"),
            "slider 回写零 thread-local:
{}",
            code
        );
    }

    /// PLAN-026 T-02: display 族 codegen golden（fixture 真源：
    /// tests/fixtures/026-native-display/display.at——§5.1 D1/D1' 定案
    /// 的降级形态逐件钉：icon→Image lucide+精确 px 尺寸、badge→样式
    /// Row、card/divider/separator/spacer/avatar→styled container、
    /// scroll→scrollable、link 子件组合不丢内容）。
    #[test]
    fn test_display_family_codegen_arm_fixture() {
        let src = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/026-native-display/display.at"
        ))
        .expect("read display fixture");
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(src.as_str()).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast.stmts.iter().find_map(|s| match s {
            crate::ast::Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        })
        .expect("widget decl");
        let widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");

        let mut gen = RustGenerator::new();
        let code = gen.generate(&widget).unwrap();

        // image：既有臂（View::image/image_styled）。
        assert!(
            code.contains("View::image(") || code.contains("View::image_styled("),
            "image 构造在册:\n{}",
            code
        );
        // icon：lucide 承载 + size prop 精确 px（w-[14px]）+ user 类透传。
        assert!(
            code.contains("\"lucide:search\"") && code.contains("View::image_styled("),
            "icon → lucide Image:\n{}",
            code
        );
        assert!(
            code.contains("w-[14px]") && code.contains("h-[14px]"),
            "icon size prop 精确 px:\n{}",
            code
        );
        assert!(
            code.contains("text-muted-foreground"),
            "icon user 类透传:\n{}",
            code
        );
        // badge：样式 Row + variant 预设 + label 子级（text prop 必达）。
        assert!(
            code.contains("View::row().style(\"items-center gap-1.5 rounded-full px-2.5 py-0.5 text-xs font-medium bg-secondary text-secondary-foreground\")"),
            "badge shadcn 基类+variant:\n{}",
            code
        );
        assert!(
            code.contains("View::text(\"Active\")"),
            "badge label 必达:\n{}",
            code
        );
        // card：styled container（user 类必达）。
        assert!(
            code.contains("View::container(") && code.contains("bg-card border rounded-xl p-4"),
            "card container 降级 + style 必达:\n{}",
            code
        );
        // divider/separator：1px 线容器。
        assert!(
            code.contains("w-full h-1 bg-gray-200"),
            "divider 底档:\n{}",
            code
        );
        assert!(
            code.contains("w-full h-px bg-border"),
            "separator 底档:\n{}",
            code
        );
        // spacer：显式 style 保真。
        assert!(
            code.contains("View::container(View::Empty).style(\"w-8\").build()"),
            "spacer 显式 style:\n{}",
            code
        );
        // avatar：缺省档 + 子件组合。
        assert!(
            code.contains("w-10 h-10 bg-gray-300 rounded-full"),
            "avatar 缺省档:\n{}",
            code
        );
        assert!(
            code.contains("View::text(\"JC\".to_string())"),
            "avatar 子件:\n{}",
            code
        );
        // scroll：scrollable 构造 + style 透传。
        assert!(
            code.contains("View::scrollable(") && code.contains(".style(\"h-40 w-full\")"),
            "scroll → scrollable:\n{}",
            code
        );
        // link：子件组合（to 仅作兜底，内容不丢）。
        assert!(
            code.contains("View::text(\"Library\".to_string())"),
            "link 子件内容必达:\n{}",
            code
        );
        // 断裂构造器零发射（badge/card/icon/scroll/link 映射已移除）。
        for broken in ["View::badge(", "View::card(", "View::icon(", "View::scroll(", "View::link("] {
            assert!(
                !code.contains(broken),
                "断裂构造器 {broken} 不得发射:\n{}",
                code
            );
        }
    }


    /// PLAN-025 T-04: select codegen golden（fixture 真源：
    /// tests/fixtures/025-native-input/select.at——View::select 构造 +
    /// on_choose SelectCallback 物化闭包 + on() 载荷臂，零 thread-local）。
    #[test]
    fn test_select_codegen_arm_fixture() {
        let src = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/025-native-input/select.at"
        ))
        .expect("read select fixture");
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(src.as_str()).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast.stmts.iter().find_map(|s| match s {
            crate::ast::Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        })
        .expect("widget decl");
        let widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");

        let mut gen = RustGenerator::new();
        let code = gen.generate(&widget).unwrap();

        assert!(
            code.contains(r#"View::select(vec!["Small".to_string(), "Medium".to_string(), "Large".to_string()])"#),
            "options 数组构造:\n{}",
            code
        );
        assert!(
            code.contains(".selected(0)"),
            "selected prop 消费:\n{}",
            code
        );
        assert!(
            code.contains(".on_choose(|_idx: usize, val: &str| SelectBoxMsg::Pick(val.to_string()))"),
            "SelectCallback 物化闭包:\n{}",
            code
        );
        assert!(
            code.contains("Pick(String)"),
            "载荷变体 (str):\n{}",
            code
        );
        assert!(
            !code.contains("last_input_text"),
            "select 回写零 thread-local:\n{}",
            code
        );
    }

    /// PLAN-533 T4: on-only handler（无 msg 块声明,vue 风格源——gallery 页
    /// 与探针均此形态）此前在 rust 轨静默丢失：type Msg = ()、view 派发
    /// 闭包引用不存在的变体（T3 字符串断言 dangling 的根因）。修复后枚举
    /// 补零参变体 + match 臂 + type Msg 回升为 <Widget>Msg。
    #[test]
    fn test_on_only_handlers_get_msg_variants_rust_track() {
        let src = r#"
widget App {
    model { show bool = false }
    on { .openDialog -> { .show = true } }
    view {
        button (text: "Show", onclick: .openDialog) {}
    }
}
"#;
        let session = crate::session::CompilerSession::ui().with_backend("rust");
        let mut parser = crate::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast.stmts.iter().find_map(|s| match s {
            crate::ast::Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        }).expect("widget decl");
        let widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");
        let mut gen = RustGenerator::new();
        let code = gen.generate(&widget).unwrap();
        assert!(
            code.contains("pub enum AppMsg {") && code.contains("    openDialog,"),
            "on-only handler variant must reach the enum:
{}", code
        );
        assert!(
            code.contains("type Msg = AppMsg;"),
            "Msg type must promote from ():
{}", code
        );
        assert!(
            code.contains("AppMsg::openDialog => {"),
            "match arm for on-only handler:
{}", code
        );
        assert!(
            code.contains("self.show = true"),
            "handler body survives:
{}", code
        );
    }

    /// PLAN-533 T3 产物级断言：真实 gallery 页（widgets-gallery
    /// alertdialog.at）经完整管线后，AlertdialogPage 的视图产物包含
    /// Modal Popover 构造与 w-96 面板 chrome（gallery 整仓 rust 生成因壳层
    /// 词汇存量红，页级断言为 codegen 臂的产物门禁）。
    /// PLAN-590：画廊迁 auto-os 顶层——解析序定位（solo 检出 SKIP）。
    #[test]
    fn test_gallery_alertdialog_page_codegen_contains_modal() {
        let Some(path) = crate::os_paths::resolve_os_top_dir(
            &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."),
            "widgets-gallery",
        )
        .map(|g| g.join("src/front/pages/alertdialog.at")) else {
            eprintln!(
                "test_gallery_alertdialog_page: SKIPPED — auto-os/widgets-gallery 未解析(solo 检出)"
            );
            return;
        };
        let src = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read gallery page: {e}"));
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(src.as_str()).with_session(session);
        let ast = parser.parse().expect("parse");
        let mut codes = Vec::new();
        for stmt in &ast.stmts {
            if let crate::ast::Stmt::WidgetDecl(decl) = stmt {
                let widget = crate::aura::extract::extract_widget_from_decl(decl)
                    .unwrap_or_else(|e| panic!("extract {}: {e:?}", decl.name));
                let mut gen = RustGenerator::new();
                codes.push(gen.generate(&widget).expect("generate"));
            }
        }
        let all = codes.join("\n");
        assert!(
            all.contains("View::Popover {")
                && all.contains("auto_lang::ui::view::PopoverPlacement::Modal"),
            "gallery alertdialog page must emit Modal Popover construction"
        );
        assert!(
            all.contains("w-96 bg-background border border-border rounded-lg shadow-lg p-6 gap-4"),
            "panel chrome must match interpreter arm"
        );
        assert!(
            all.contains("cancelAction"),
            "cancel onclick dispatch must survive the real page"
        );
    }

    /// PLAN-533 T3: alert-dialog 家族经真实管线（parse → extract → generate）
    /// 发射模态 Popover 构造——根臂拆解 trigger/content，面板 w-96 chrome 与
    /// 解释器臂（PLAN-530 W13）同串，action·cancel onclick 走既有消息派发。
    #[test]
    fn test_alert_dialog_codegen_emits_modal_popover() {
        let src = r#"
widget Demo {
    model { show bool = false }
    on {
        .openDialog -> { .show = true }
        .cancelAction -> { .show = false }
    }
    view {
        alert-dialog (open: .show) {
            alert-dialog-trigger {
                button (text: "Show Dialog", onclick: .openDialog) {}
            }
            alert-dialog-content {
                alert-dialog-header {
                    alert-dialog-title "Are you sure?"
                    alert-dialog-description "This cannot be undone."
                }
                alert-dialog-footer {
                    alert-dialog-cancel "Cancel" { onclick: .cancelAction }
                    alert-dialog-action "Continue" { onclick: .openDialog }
                }
            }
        }
    }
}
"#;
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast.stmts.iter().find_map(|s| match s {
            crate::ast::Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        }).expect("widget decl");
        let widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");

        let mut gen = RustGenerator::new();
        let code = gen.generate(&widget).unwrap();

        assert!(code.contains("View::Popover {"), "Popover variant literal:\n{}", code);
        assert!(code.contains("auto_lang::ui::view::PopoverPlacement::Modal"), "Modal placement:\n{}", code);
        assert!(code.contains("auto_lang::ui::view::PopoverAnchor::Widget"), "widget anchor:\n{}", code);
        assert!(code.contains("open: self.show"), "open bound to state:\n{}", code);
        assert!(
            code.contains("w-96 bg-background border border-border rounded-lg shadow-lg p-6 gap-4"),
            "panel chrome matches interpreter arm:\n{}", code
        );
        assert!(
            code.contains("View::text_styled(\"Are you sure?\".to_string(), \"text-lg font-semibold\")"),
            "title default class:\n{}", code
        );
        assert!(
            code.contains("text-sm text-muted-foreground"),
            "description default class:\n{}", code
        );
        assert!(
            code.contains("border border-input bg-background text-foreground rounded-md"),
            "cancel outline preset:\n{}", code
        );
        assert!(
            code.contains("bg-primary text-primary-foreground font-medium rounded-md"),
            "action primary preset:\n{}", code
        );
        assert!(code.contains("DemoMsg::openDialog"), "trigger onclick dispatch:\n{}", code);
        assert!(code.contains("DemoMsg::cancelAction"), "cancel onclick dispatch:\n{}", code);
    }

    /// PLAN-027 T-02: 裸 popover codegen 臂 golden —— 坐标锚形态（desktop.at
    /// 空白菜单/拖拽幽灵同构）：x/y → Point 锚、open 动态表达式、ondismiss
    /// 消息、placement 缺省 BottomStart、class 落面板 + w-auto 注入。此前
    /// 落 `_ => "col"` 降级 + props 静默丢弃（shell pack a2r 编译阻断面）。
    #[test]
    fn test_bare_popover_point_anchor_codegen() {
        let src = r#"
widget Probe {
    msg { BlankClose }
    model {
        var blank_menu str = ""
        var cx float = 0.0
        var cy float = 0.0
    }
    view {
        popover (open: .blank_menu != "", x: .cx, y: .cy, ondismiss: .BlankClose, class: "p-1 border rounded bg-card") {
            text "menu"
        }
    }
}
"#;
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast.stmts.iter().find_map(|s| match s {
            crate::ast::Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        }).expect("widget decl");
        let widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");

        let mut gen = RustGenerator::new();
        let code = gen.generate(&widget).unwrap();

        assert!(code.contains("View::Popover {"), "Popover variant literal:\n{}", code);
        assert!(
            code.contains("auto_lang::ui::view::PopoverAnchor::Point { x: (self.cx"),
            "Point anchor bound to cursor state:\n{}", code
        );
        assert!(
            code.contains("open: self.blank_menu !="),
            "open bound to state expression:\n{}", code
        );
        assert!(
            code.contains("on_dismiss: Some(ProbeMsg::BlankClose)"),
            "ondismiss → on_dismiss message:\n{}", code
        );
        assert!(
            code.contains("placement: auto_lang::ui::view::PopoverPlacement::BottomStart,"),
            "point anchor default placement (PLAN-528 W9):\n{}", code
        );
        assert!(
            code.contains("p-1 border rounded bg-card w-auto"),
            "user class on panel + w-auto width injection:\n{}", code
        );
    }

    /// PLAN-027 T-02: 裸 popover 首子锚形态（shell.at 任务栏右键菜单同构）：
    /// plain[0] = 锚件、plain[1..] = 面板列、placement "top" 直译、缺省
    /// Bottom 不误落 Modal。
    #[test]
    fn test_bare_popover_widget_anchor_codegen() {
        let src = r#"
widget Taskbar {
    msg { WinMenuClose, Ping }
    model { var win_menu str = "" }
    view {
        popover (open: .win_menu == "1", placement: "top", ondismiss: .WinMenuClose, class: "p-1 border rounded bg-card") {
            text "anchor"
            col {
                text "menu item"
            }
        }
    }
}
"#;
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast.stmts.iter().find_map(|s| match s {
            crate::ast::Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        }).expect("widget decl");
        let widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");

        let mut gen = RustGenerator::new();
        let code = gen.generate(&widget).unwrap();

        assert!(
            code.contains("anchor: auto_lang::ui::view::PopoverAnchor::Widget(Box::new("),
            "first plain child becomes widget anchor:\n{}", code
        );
        assert!(
            code.contains("placement: auto_lang::ui::view::PopoverPlacement::Top,"),
            "literal top placement (not TopStart/TopEnd):\n{}", code
        );
        assert!(
            code.contains("on_dismiss: Some(TaskbarMsg::WinMenuClose)"),
            "ondismiss handler:\n{}", code
        );
        // 内容列 = plain[1..]（锚件不进面板）；面板内子件存活。
        let panel_zone = code.split("content: Box::new(").nth(1).unwrap_or("");
        assert!(
            panel_zone.contains("menu item"),
            "second plain child lands in panel:\n{}", code
        );
        assert!(
            !panel_zone.contains(">anchor<"),
            "anchor text must not leak into panel:\n{}", code
        );
    }

    /// PLAN-027 T-03: 宿主合成件槽位 codegen —— window_thumbnail（动态
    /// wid 绑定，switcher 行循环同构）/ workspace_preview（静态 ws，
    /// shell pager 面板同构）直发既有 View 变体（D4 修正 B）；fallback
    /// 档缺省 app-window；style 直传。
    #[test]
    fn test_host_synth_slot_codegen() {
        let src = r#"
widget Probe {
    model {
        var rows = []
        var __wp_current str = ""
    }
    view {
        col {
            for r in .rows {
                window_thumbnail (wid: r.wid, fallback_icon: r.icon) { style: "w-24 h-14 border rounded" }
            }
            workspace_preview (ws: "2") { style: "w-44 h-16 rounded-lg" }
            window_thumbnail (wid: "7") { }
        }
    }
}
"#;
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast.stmts.iter().find_map(|s| match s {
            crate::ast::Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        }).expect("widget decl");
        let widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");

        let mut gen = RustGenerator::new();
        let code = gen.generate(&widget).unwrap();

        assert!(
            code.contains("View::WindowThumbnail { wid: r[\"wid\"]"),
            "dynamic wid binding (loop var Value index-read convention):\n{}", code
        );
        assert!(
            code.contains("fallback_icon: r[\"icon\"]"),
            "dynamic fallback_icon survives (no silent default):\n{}", code
        );
        assert!(
            code.contains("style: auto_lang::ui::style::Style::parse(\"w-24 h-14 border rounded\").ok()"),
            "style passthrough:\n{}", code
        );
        assert!(
            code.contains("View::WorkspacePreview { ws: \"2\".to_string(), fallback_icon: \"app-window\".to_string()"),
            "static ws + default fallback:\n{}", code
        );
        assert!(
            code.contains("View::WindowThumbnail { wid: \"7\".to_string(), fallback_icon: \"app-window\".to_string(), style: None }"),
            "literal wid + bare tag defaults:\n{}", code
        );
    }

    /// PLAN-027 T-04: 显式拒绝门 —— 未知 prop/事件不再静默丢弃，发射
    /// compile_error!（编译期错拦截"看似编译过实缺件"的生成物）。
    #[test]
    fn test_codegen_rejects_unknown_prop_and_event() {
        // prop 与事件分源断言——同源时事件块的拒绝发射会覆盖 prop 块
        //（builder 链后写胜），rustc 仍红但诊断只剩一条。
        let prop_src = r#"
widget Probe {
    msg { Ping }
    view {
        col {
            frobnicate: "yes"
        }
    }
}
"#;
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(prop_src).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast.stmts.iter().find_map(|s| match s {
            crate::ast::Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        }).expect("widget decl");
        let widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");
        let mut gen = RustGenerator::new();
        let code = gen.generate(&widget).unwrap();
        assert!(
            code.contains("compile_error!(\"a2r codegen: prop `frobnicate`"),
            "unknown prop must compile_error (was silently dropped):
{}", code
        );

        let event_src = r#"
widget Probe2 {
    msg { Ping }
    view {
        col {
            onwiggle: .Ping
        }
    }
}
"#;
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(event_src).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast.stmts.iter().find_map(|s| match s {
            crate::ast::Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        }).expect("widget decl");
        let widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");
        let mut gen = RustGenerator::new();
        let code = gen.generate(&widget).unwrap();
        assert!(
            code.contains("compile_error!(\"a2r codegen: event `onwiggle`"),
            "unknown event must compile_error (was silently dropped):
{}", code
        );
    }

    /// PLAN-027 T-04: shell pack 全量 tag/prop/事件清单编译门 —— 真源
    /// 五件（shell/desktop/switcher/notification_center/dashboard.at）视图
    /// 每一对 (tag, prop) / (tag, event-base) 都必须落在 a2r 认知表内
    /// （防 pack 演进引入"解释态能跑、a2r 缺译"的面）。表即合同：pack
    /// 新词汇须同步扩 rust.rs 臂 + 本表（复审门）。solo（pack 不可解析）
    /// 跳过 pass。handler 侧词汇（while/push/len/contains/storage.*/）不
    /// 在本表——真实编译门 = T-07 shell-lib crate cargo build。
    #[test]
    fn test_shell_pack_codegen_vocabulary_gate() {
        let Some(dir) = crate::os_paths::resolve_os_top_dir(
            &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."),
            "shell",
        ) else {
            eprintln!("test_shell_pack_vocabulary_gate: SKIPPED — auto-os/shell 未解析(solo 检出)");
            return;
        };
        const NAMES: [&str; 5] = [
            "shell.at",
            "desktop.at",
            "switcher.at",
            "notification_center.at",
            "dashboard.at",
        ];

        // (tag, prop) 认知表 —— 每项 = rust.rs 对应臂的已译 prop。
        let known_props: &[(&str, &str)] = &[
            ("col", "style"),
            ("col", "class"),
            ("row", "style"),
            ("row", "class"),
            ("row", "onclick"),
            ("row", "onmouseenter"),
            ("taskbar", "style"),
            ("div", "style"),
            ("spacer", "style"),
            ("grid", "cols"),
            ("grid", "gap"),
            ("grid", "style"),
            ("button", "text"),
            ("button", "icon"),
            ("button", "variant"),
            ("button", "style"),
            ("button", "onclick"),
            ("text", "style"),
            ("icon", "name"),
            ("icon", "style"),
            ("icon", "size"),
            ("image", "src"),
            ("image", "alt"),
            ("image", "fit"),
            ("image", "style"),
            ("mouse-area", "style"),
            ("popover", "open"),
            ("popover", "placement"),
            ("popover", "ondismiss"),
            ("popover", "x"),
            ("popover", "y"),
            ("popover", "class"),
            ("popover", "style"),
            // PLAN-035 T-11：desktop.at 右键菜单「桌面小组件」checkbox。
            ("checkbox", "checked"),
            ("checkbox", "style"),
            ("window_thumbnail", "wid"),
            ("window_thumbnail", "fallback_icon"),
            ("window_thumbnail", "style"),
            ("workspace_preview", "ws"),
            ("workspace_preview", "fallback"),
            ("workspace_preview", "style"),
            // text 主 prop 简写（`text .state` → props["text"]，primary
            // prop 铸造；parser.rs get_primary_prop）。
            ("text", "text"),
        ];
        // tag → 事件认知（base 名；.prevent 等后缀剥离后比对）。
        let known_events: &[(&str, &str)] = &[
            ("button", "onclick"),
            ("button", "oncontextmenu"),
            ("icon", "onclick"),
            ("mouse-area", "onclick"),
            ("mouse-area", "onmousedown"),
            ("mouse-area", "onmouseenter"),
            ("mouse-area", "onmouseleave"),
            ("mouse-area", "ondblclick"),
            ("mouse-area", "onmouseup"),
            ("mouse-area", "oncontextmenu"),
            ("row", "onclick"),
            ("col", "oncontextmenu"),
            // PLAN-035 T-11：desktop.at 右键菜单「桌面小组件」checkbox。
            ("checkbox", "onclick"),
            ("popover", "ondismiss"),
            // 认知且双轨同弃（View IR 布局件无 hover 槽，解释臂同弃）。
            ("row", "onmouseenter"),
        ];
        let known_tags: &[&str] = &[
            "col", "row", "taskbar", "div", "spacer", "grid", "button", "text", "icon",
            "image", "mouse-area", "popover", "window_thumbnail", "workspace_preview",
        ];

        fn walk(node: &crate::aura::AuraNode, out: &mut Vec<(String, String, bool)>) {
            use crate::aura::AuraNode;
            match node {
                AuraNode::Element { tag, props, events, children, .. } => {
                    for k in props.keys() {
                        out.push((tag.clone(), k.clone(), true));
                    }
                    for k in events.keys() {
                        out.push((tag.clone(), k.clone(), false));
                    }
                    for c in children {
                        walk(c, out);
                    }
                }
                AuraNode::ForLoop { body, .. } => {
                    for c in body {
                        walk(c, out);
                    }
                }
                AuraNode::Conditional { then_body, else_body, .. } => {
                    for c in then_body {
                        walk(c, out);
                    }
                    if let Some(els) = else_body {
                        for c in els {
                            walk(c, out);
                        }
                    }
                }
                _ => {}
            }
        }

        let mut violations: Vec<String> = Vec::new();
        for name in NAMES {
            let path = dir.join(name);
            let src = match std::fs::read_to_string(&path) {
                Ok(s) => s,
                Err(e) => {
                    violations.push(format!("{name}: read failed {e}"));
                    continue;
                }
            };
            let session = crate::session::CompilerSession::ui();
            let mut parser = crate::Parser::from(src.as_str()).with_session(session);
            let ast = match parser.parse() {
                Ok(a) => a,
                Err(e) => {
                    violations.push(format!("{name}: parse failed {e}"));
                    continue;
                }
            };
            for stmt in &ast.stmts {
                let crate::ast::Stmt::WidgetDecl(d) = stmt else { continue };
                let Ok(widget) = crate::aura::extract::extract_widget_from_decl(d) else {
                    continue;
                };
                let mut trees = vec![&widget.view_tree];
                for (_, nv) in &widget.named_views {
                    trees.push(nv);
                }
                for tree in trees {
                    let mut found = Vec::new();
                    walk(tree, &mut found);
                    for (tag, key, is_prop) in found {
                        let base = key.split('.').next().unwrap_or(&key).to_string();
                        let table = if is_prop { known_props } else { known_events };
                        let ok = table.iter().any(|(t, k)| *t == tag && *k == base)
                            // 布局/容器 style 的 class 别名（add_prop_to_builder
                            // 认知集）+ 动态 style 表达式（.style(expr.as_str())）。
                            || (is_prop && (base == "style" || base == "class"));
                        if !ok {
                            violations.push(format!(
                                "{name}: {tag} {} `{base}` 不在 a2r 认知表",
                                if is_prop { "prop" } else { "event" }
                            ));
                        }
                    }
                }
            }
        }
        assert!(
            violations.is_empty(),
            "shell pack a2r 词汇门违例（在 rust.rs 补臂 + 扩本表，或修正 pack）：\n{}",
            violations.join("\n")
        );
    }

    /// PLAN-534 T9: sheet/drawer/hovercard 经真实管线发射——placement/
    /// chrome 与解释器臂同串（双轨一致断言）:sheet 缺省 EdgeRight + 横条
    /// chrome + 铸造 dismiss 折算;drawer bottom → EdgeBottom + 贴缘圆角 +
    /// 装饰把手;hovercard → MouseArea 包锚 + Bottom 非模态 + enter 接线。
    #[test]
    fn test_side_panels_codegen_matches_interpreter_chrome() {
        let src = r#"
widget Panels {
    view {
        col {
            sheet (side: "right") {
                sheet-trigger {
                    button (text: "Open", variant: "outline") {}
                }
                sheet-content {
                    sheet-title "Edit Profile"
                }
            }
            drawer (direction: "bottom") {
                drawer-trigger "Open Drawer"
                drawer-content {
                    drawer-title "Settings"
                }
            }
            hovercard {
                hover-card-trigger {
                    text "@mentor"
                }
                hover-card-content {
                    text "bio"
                }
            }
        }
    }
}
"#;
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast.stmts.iter().find_map(|s| match s {
            crate::ast::Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        }).expect("widget decl");
        let widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");

        let mut gen = RustGenerator::new();
        let code = gen.generate(&widget).unwrap();

        // sheet: 缺省 right → EdgeRight;横条 chrome 同串;铸造 dismiss 折算。
        assert!(
            code.contains("auto_lang::ui::view::PopoverPlacement::EdgeRight"),
            "sheet default EdgeRight:\n{}", code
        );
        assert!(
            code.contains("w-96 bg-background border shadow-lg p-6 gap-4 h-full"),
            "sheet chrome matches interpreter side_panel_chrome:\n{}", code
        );
        assert!(
            code.contains("on_dismiss: Some(PanelsMsg::__dlg_close_1)"),
            "sheet minted dismiss folding:\n{}", code
        );
        // drawer: bottom → EdgeBottom;贴缘圆角;装饰把手。
        assert!(
            code.contains("auto_lang::ui::view::PopoverPlacement::EdgeBottom"),
            "drawer bottom EdgeBottom:\n{}", code
        );
        assert!(
            code.contains("bg-background border shadow-lg p-6 gap-4 w-full rounded-t-lg"),
            "drawer bottom chrome with rounded-t:\n{}", code
        );
        assert!(
            code.contains("w-8 h-1 rounded-full bg-muted"),
            "drawer handle emission:\n{}", code
        );
        // hovercard: MouseArea 包锚 + Bottom 非模态 + chrome 同串 + enter 接线。
        assert!(
            code.contains("View::MouseArea { content: Box::new("),
            "hovercard anchor wrapped in MouseArea:\n{}", code
        );
        assert!(
            code.contains("auto_lang::ui::view::PopoverPlacement::Bottom"),
            "hovercard Bottom placement:\n{}", code
        );
        assert!(
            code.contains("w-80 bg-popover border rounded-lg shadow-md p-4"),
            "hovercard chrome matches interpreter arm:\n{}", code
        );
        assert!(
            code.contains("on_enter: Some(PanelsMsg::__dlg_enter_3)"),
            "hovercard enter wiring from minted handler:\n{}", code
        );
        assert!(
            code.contains("on_dismiss: None"),
            "hovercard non-modal (no dismiss):\n{}", code
        );
    }

    /// PLAN-533 T3/T6: dialog 家族（可关闭模态，schema sub_widgets 无连字符
    /// 形态 + dashed 形态均识别）发射 Modal Popover；无 open 绑定形态走
    /// parser 铸造（__dlg_open_1 + toggle/close），on_dismiss 折算 close 回流。
    #[test]
    fn test_dialog_family_codegen_modal_and_bare_trigger() {
        let src = r#"
widget DialogDemo {
    view {
        dialog {
            dialog-trigger "Open Dialog"
            dialog-content {
                dialog-header {
                    dialog-title "Edit Profile"
                    dialog-description "Make changes here."
                }
                dialog-footer {
                    dialog-close "Save changes"
                }
            }
        }
    }
}
"#;
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast.stmts.iter().find_map(|s| match s {
            crate::ast::Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        }).expect("widget decl");
        let widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");

        let mut gen = RustGenerator::new();
        let code = gen.generate(&widget).unwrap();

        assert!(code.contains("View::Popover {"), "dialog family emits Popover:\n{}", code);
        assert!(code.contains("auto_lang::ui::view::PopoverPlacement::Modal"), "Modal placement:\n{}", code);
        assert!(code.contains("open: self.__dlg_open_1"), "minted open bound:\n{}", code);
        assert!(code.contains("DialogDemoMsg::__dlg_toggle_1"), "minted toggle dispatch:\n{}", code);
        assert!(code.contains("DialogDemoMsg::__dlg_close_1"), "minted close dispatch:\n{}", code);
        assert!(
            code.contains("View::button(\"Open Dialog\")"),
            "bare-text trigger renders as button:\n{}", code
        );
        assert!(
            code.contains("View::button(\"Save changes\")"),
            "dialog-close renders as button:\n{}", code
        );
        // PLAN-533 T6: dialog（可关闭）族铸造形态 on_dismiss 折算
        // __dlg_close_N（ESC/外点/锚点 → update:open(false)）。
        assert!(
            code.contains("on_dismiss: Some(DialogDemoMsg::__dlg_close_1)"),
            "unbound dialog must wire dismiss reflow:\n{}", code
        );
    }

    /// PLAN-533 T7: dropdown-menu 家族 → 锚定 Popover（BottomStart + 菜单
    /// chrome p-1 gap-1）;无 open 绑定走铸造（trigger 包裹形态 onclick 落
    /// 内层按钮）;外点/ESC 经 on_dismiss=__dlg_close_N 回流;item 预设样式。
    #[test]
    fn test_dropdown_menu_codegen_anchored_popover() {
        let src = r#"
widget MenuDemo {
    view {
        dropdown-menu {
            dropdown-menu-trigger {
                button (text: "Open", variant: "outline") {}
            }
            dropdown-menu-content {
                dropdown-menu-item "Profile"
                dropdown-menu-item "Billing"
                dropdown-menu-separator {}
                dropdown-menu-item "Log out"
            }
        }
    }
}
"#;
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast.stmts.iter().find_map(|s| match s {
            crate::ast::Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        }).expect("widget decl");
        let widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");
        let mut gen = RustGenerator::new();
        let code = gen.generate(&widget).unwrap();

        assert!(
            code.contains("auto_lang::ui::view::PopoverPlacement::BottomStart"),
            "dropdown-menu anchors BottomStart:\n{}", code
        );
        assert!(
            code.contains("open: self.__dlg_open_1"),
            "minted open bound:\n{}", code
        );
        assert!(
            code.contains("MenuDemoMsg::__dlg_toggle_1"),
            "minted toggle reaches the wrapped trigger button:\n{}", code
        );
        assert!(
            code.contains("on_dismiss: Some(MenuDemoMsg::__dlg_close_1)"),
            "dismiss reflow:\n{}", code
        );
        assert!(
            code.contains("bg-popover border border-border rounded-md shadow-md p-1"),
            "menu chrome:\n{}", code
        );
        assert!(
            code.contains("View::text_styled(\"Profile\".to_string(), \"w-full px-2 py-1.5 text-sm cursor-pointer hover:bg-secondary text-start\")"),
            "item preset style:\n{}", code
        );
    }

    /// Plan 448 C: bare `value:` input through the real pipeline. The minted
    /// `__bind_*` variant must reach the enum AND the on_fields injection
    /// (`self.email = last_input_text`) — without the variant the match arm
    /// is silently skipped and typing never persists.
    #[test]
    fn test_bare_value_input_rust_codegen() {
        let src = r#"
widget LoginForm {
    model { var email str = "" }
    view {
        input { value: .email, placeholder: "you@example.com" }
    }
}
"#;
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast.stmts.iter().find_map(|s| match s {
            crate::ast::Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        }).expect("widget decl");
        let widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");

        let mut gen = RustGenerator::new();
        let code = gen.generate(&widget).unwrap();

        assert!(
            code.contains("__bind_LoginForm_oninput_1,"),
            "auto-sync variant in the enum:\n{}",
            code
        );
        assert!(
            code.contains("__bind_LoginForm_oninput_1 => {"),
            "match arm for the auto-sync variant:\n{}",
            code
        );
        assert!(
            code.contains("last_input_text()"),
            "input_fields injection reads the typed text:\n{}",
            code
        );
        assert!(
            code.contains("self.email"),
            "input_fields injection writes the bound field:\n{}",
            code
        );
    }

    /// Plan 413 Phase 3: code_editor codegen — builder chain shape, payload
    /// default for String variants, input_fields + code_editor_sources
    /// registration (handler must read code_editor_text("key")).
    #[test]
    fn test_code_editor_codegen() {
        let mut element = AuraNode::element("code_editor");
        {
            if let AuraNode::Element { props, events, .. } = &mut element {
                props.insert("key".to_owned(), AuraPropValue::Expr(crate::ast::Expr::Str("src".into())));
                props.insert("lang".to_owned(), AuraPropValue::Expr(crate::ast::Expr::Str("rust".into())));
                props.insert("content".to_owned(), AuraPropValue::Expr(crate::ast::Expr::Ident("source".into())));
                props.insert("wrap".to_owned(), AuraPropValue::Expr(crate::ast::Expr::Bool(false)));
                props.insert("search".to_owned(), AuraPropValue::Expr(crate::ast::Expr::Ident("query".into())));
                events.insert(
                    "oninput".to_owned(),
                    AuraEvent { handler: ".SourceChanged".into(), params: vec![] },
                );
            }
        }

        let widget = AuraWidget {
            named_views: Vec::new(),
            actions: None,
            timers: Vec::new(),
            name: "Playground".to_string(),
            state_vars: vec![
                AuraStateDef {
                    name: "source".to_string(),
                    type_info: Type::StrFixed(0),
                    initial: crate::ast::Expr::Str("fn main() {}".into()),
                    decorators: vec![],
                },
                AuraStateDef {
                    name: "query".to_string(),
                    type_info: Type::StrFixed(0),
                    initial: crate::ast::Expr::Str("".into()),
                    decorators: vec![],
                },
            ],
            messages: vec![AuraMessage {
                variants: vec![AuraMsgVariant {
                    name: "SourceChanged".to_string(),
                    payload: vec![Type::StrFixed(0)],
                    payload_names: vec![],
                    quoted: false,
                }],
            }],
            view_tree: element,
            handlers: {
                let mut h = std::collections::BTreeMap::new();
                h.insert(
                    "SourceChanged".to_owned(),
                    LogicPayload::AstStmts(vec![crate::ast::Stmt::Expr(
                        crate::ast::Expr::Bina(
                            Box::new(crate::ast::Expr::Dot(
                                Box::new(crate::ast::Expr::Ident("self".into())),
                                "source".into(),
                            )),
                            auto_val::Op::Asn,
                            Box::new(crate::ast::Expr::Call(crate::ast::Call {
                                name: Box::new(crate::ast::Expr::Ident("code_editor_text".into())),
                                args: {
                                    let mut a = crate::ast::Args::new();
                                    a.args.push(crate::ast::Arg::Pos(
                                        crate::ast::Expr::Str("src".into()),
                                    ));
                                    a
                                },
                                ret: crate::ast::Type::StrOwned,
                                type_args: vec![],
                                generic_args: vec![],
                                pos: None,
                            })),
                        ),
                    )]),
                );
                h
            },
            props: vec![],
            computed: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
            setup: None,
        };

        let mut gen = RustGenerator::new();
        let code = gen.generate(&widget).unwrap();

        assert!(
            code.contains("View::code_editor(\"src\")"),
            "builder must key on the storage key:\n{}",
            code
        );
        assert!(
            code.contains(".value(self.source.clone())"),
            "value binding:\n{}",
            code
        );
        assert!(
            code.contains(".lang(\"rust\")"),
            "lang prop:\n{}",
            code
        );
        assert!(
            code.contains(".on_change(PlaygroundMsg::SourceChanged(\"\".to_string()))"),
            "String-payload variant gets a default arg:\n{}",
            code
        );
        assert!(
            code.contains("auto_lang::ui::code_editor::code_editor_text(\"src\")"),
            "handler reads text via code_editor_text:\n{}",
            code
        );
        assert!(
            code.contains(".search(self.query.clone())"),
            "search binding from state:\n{}",
            code
        );
    }

    /// Plan 413 follow-up: end-to-end compile of the generated code_editor
    /// widget code — the generator output is wrapped with the same preamble
    /// auto-man emits and compiled against auto-lang (ui-iced) in a
    /// throwaway crate. Ignored by default (compiles the dependency tree).
    #[test]
    #[ignore = "e2e compile: minutes on a cold target dir"]
    fn test_code_editor_codegen_compiles() {
        // Reuse the codegen test's widget construction.
        let mut element = AuraNode::element("code_editor");
        {
            if let AuraNode::Element { props, events, .. } = &mut element {
                props.insert("key".to_owned(), AuraPropValue::Expr(crate::ast::Expr::Str("src".into())));
                props.insert("lang".to_owned(), AuraPropValue::Expr(crate::ast::Expr::Str("rust".into())));
                props.insert("content".to_owned(), AuraPropValue::Expr(crate::ast::Expr::Ident("source".into())));
                props.insert("search".to_owned(), AuraPropValue::Expr(crate::ast::Expr::Ident("query".into())));
                events.insert(
                    "oninput".to_owned(),
                    AuraEvent { handler: ".SourceChanged".into(), params: vec![] },
                );
                events.insert(
                    "oncursor".to_owned(),
                    AuraEvent { handler: ".CursorMoved".into(), params: vec![] },
                );
            }
        }

        let widget = AuraWidget {
            named_views: Vec::new(),
            actions: None,
            timers: Vec::new(),
            name: "Playground".to_string(),
            state_vars: vec![
                AuraStateDef {
                    name: "source".to_string(),
                    type_info: Type::StrFixed(0),
                    initial: crate::ast::Expr::Str("fn main() {}".into()),
                    decorators: vec![],
                },
                AuraStateDef {
                    name: "query".to_string(),
                    type_info: Type::StrFixed(0),
                    initial: crate::ast::Expr::Str("".into()),
                    decorators: vec![],
                },
            ],
            messages: vec![AuraMessage {
                variants: vec![
                    AuraMsgVariant { payload_names: vec![], name: "SourceChanged".to_string(), payload: vec![Type::StrFixed(0)], quoted: false },
                    AuraMsgVariant { payload_names: vec![], name: "CursorMoved".to_string(), payload: vec![], quoted: false },
                ],
            }],
            view_tree: element,
            handlers: std::collections::BTreeMap::new(),
            props: vec![],
            computed: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
            setup: None,
        };

        let mut gen = RustGenerator::new();
        let code = gen.generate(&widget).unwrap();

        let main_rs = format!(
            "#![allow(dead_code, unused)]
{}
fn main() {{}}
",
            code
        );

        // Throwaway crate under the workspace target dir (path deps resolve,
        // cargo cache reused).
        let root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let root = std::path::Path::new(&root)
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf();
        let tmp = root.join("target").join("code-editor-e2e").join("playground");
        let src_dir = tmp.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(src_dir.join("main.rs"), &main_rs).unwrap();
        let auto_lang_path = root.join("crates").join("auto-lang");
        std::fs::write(
            tmp.join("Cargo.toml"),
            format!(
                "[package]\nname = \"code_editor_e2e_playground\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nauto-lang = {{ path = {:?}, features = [\"ui-iced\"] }}\n\n[workspace]\n",
                auto_lang_path.to_string_lossy().replace("\\\\", "/")
            ),
        )
        .unwrap();

        let output = std::process::Command::new("cargo")
            .arg("build")
            .current_dir(&tmp)
            .output()
            .unwrap_or_else(|e| panic!("cargo build in {}: {}", tmp.display(), e));
        assert!(
            output.status.success(),
            "generated code_editor code failed to compile.\n--- main.rs ---\n{}\n--- stderr ---\n{}",
            main_rs,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn test_auto_type_to_rust() {
        let gen = RustGenerator::new();

        assert_eq!(gen.auto_type_to_rust(&Type::Int), "i32");
        assert_eq!(gen.auto_type_to_rust(&Type::Bool), "bool");
        assert_eq!(gen.auto_type_to_rust(&Type::StrFixed(0)), "String");
        assert_eq!(gen.auto_type_to_rust(&Type::Float), "f32");
    }

    /// Plan 371 Task 21: scalar state vars must emit a `state_snapshot()`
    /// override mapping each scalar field to `auto_lang::ui::auto_val::Value`.
    /// Non-scalar (Vec/serde_json::Value) fields must be skipped.
    #[test]
    fn test_state_snapshot_scalar_override() {
        let widget = AuraWidget {
            named_views: Vec::new(),
            actions: None,
            timers: Vec::new(),
            name: "App".to_string(),
            state_vars: vec![
                AuraStateDef {
                    name: "count".to_string(),
                    type_info: Type::Int,
                    initial: crate::ast::Expr::Int(0),
                    decorators: vec![],
                },
                AuraStateDef {
                    name: "title".to_string(),
                    type_info: Type::StrFixed(0),
                    initial: crate::ast::Expr::Str("x".into()),
                    decorators: vec![],
                },
                AuraStateDef {
                    name: "editing".to_string(),
                    type_info: Type::Bool,
                    initial: crate::ast::Expr::Bool(false),
                    decorators: vec![],
                },
                // Non-scalar: array literal -> Vec<serde_json::Value>, skipped.
                AuraStateDef {
                    name: "items".to_string(),
                    type_info: Type::Unknown,
                    initial: crate::ast::Expr::Array(vec![]),
                    decorators: vec![],
                },
            ],
            messages: vec![AuraMessage {
                variants: vec![AuraMsgVariant { payload_names: vec![], name: "Inc".to_string(), quoted: false, payload: vec![] }],
            }],
            view_tree: AuraNode::element("col"),
            handlers: std::collections::BTreeMap::new(),
            props: vec![],
            computed: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
            setup: None,
        };

        let mut gen = RustGenerator::new();
        let code = gen.generate(&widget).unwrap();

        assert!(
            code.contains("fn state_snapshot(&self) -> std::collections::HashMap<String, auto_lang::ui::auto_val::Value>"),
            "missing state_snapshot signature, got:\n{}",
            code
        );
        assert!(code.contains(r#""count""#), "missing count: {}", code);
        assert!(code.contains("Value::Int(self.count)"), "count not Int: {}", code);
        assert!(code.contains(r#""title""#), "missing title: {}", code);
        assert!(code.contains("Value::str(&self.title)"), "title not str: {}", code);
        assert!(code.contains(r#""editing""#), "missing editing: {}", code);
        assert!(code.contains("Value::Bool(self.editing)"), "editing not Bool: {}", code);
        assert!(!code.contains(r#""items""#), "non-scalar items leaked: {}", code);
    }

    /// Plan 371 Task 21: no scalar fields -> no override (trait default).
    #[test]
    fn test_state_snapshot_no_scalars_no_override() {
        let widget = AuraWidget {
            named_views: Vec::new(),
            actions: None,
            timers: Vec::new(),
            name: "OnlyCollections".to_string(),
            state_vars: vec![AuraStateDef {
                name: "items".to_string(),
                type_info: Type::Unknown,
                initial: crate::ast::Expr::Array(vec![]),
                decorators: vec![],
            }],
            messages: vec![AuraMessage {
                variants: vec![AuraMsgVariant { payload_names: vec![], name: "Tick".to_string(), quoted: false, payload: vec![] }],
            }],
            view_tree: AuraNode::element("col"),
            handlers: std::collections::BTreeMap::new(),
            props: vec![],
            computed: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
            setup: None,
        };

        let mut gen = RustGenerator::new();
        let code = gen.generate(&widget).unwrap();
        assert!(!code.contains("fn state_snapshot"), "should not emit override: {}", code);
    }

    /// Plan 371 Task 22b: a component that has a registered store composable
    /// must recurse into `self.store.state_snapshot()` with a `store.` prefix,
    /// so child/store state is visible to the rust-mode autoui_state tool.
    #[test]
    fn test_state_snapshot_recurses_into_store() {
        let widget = AuraWidget {
            named_views: Vec::new(),
            actions: None,
            timers: Vec::new(),
            name: "App".to_string(),
            state_vars: vec![AuraStateDef {
                name: "search".to_string(),
                type_info: Type::StrFixed(0),
                initial: crate::ast::Expr::Str("x".into()),
                decorators: vec![],
            }],
            messages: vec![AuraMessage {
                variants: vec![AuraMsgVariant { payload_names: vec![], name: "Tick".to_string(), quoted: false, payload: vec![] }],
            }],
            view_tree: AuraNode::element("col"),
            handlers: std::collections::BTreeMap::new(),
            props: vec![],
            computed: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
            setup: None,
        };

        let mut gen = RustGenerator::new();
        // Register a store composable (as rust_ui.rs does before generating).
        gen.register_store("store", "NotesStore");
        let code = gen.generate(&widget).unwrap();

        // The override recurses into the store field with a "store." prefix.
        assert!(
            code.contains("self.store.state_snapshot()"),
            "missing store recursion: {}",
            code
        );
        assert!(
            code.contains(r#""store""#) && code.contains("format!("),
            "missing store. prefix formatting: {}",
            code
        );
        // The store struct itself should NOT recurse into a `store` field
        // (avoid NotesStore { store: NotesStore } infinite recursion).
        let store_widget = AuraWidget {
            named_views: Vec::new(),
            actions: None,
            timers: Vec::new(),
            name: "NotesStore".to_string(),
            state_vars: vec![AuraStateDef {
                name: "dark_mode".to_string(),
                type_info: Type::Bool,
                initial: crate::ast::Expr::Bool(false),
                decorators: vec![],
            }],
            messages: vec![],
            view_tree: AuraNode::element("col"),
            handlers: std::collections::BTreeMap::new(),
            props: vec![],
            computed: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
            setup: None,
        };
        let store_code = gen.generate(&store_widget).unwrap();
        assert!(
            !store_code.contains("self.store.state_snapshot()"),
            "store struct must not recurse into itself: {}",
            store_code
        );
    }

    #[test]
    fn test_ast_expr_to_rust() {
        let gen = RustGenerator::new();

        assert_eq!(gen.ast_expr_to_rust(&crate::ast::Expr::Int(42)), "42");
        assert_eq!(gen.ast_expr_to_rust(&crate::ast::Expr::Bool(true)), "true");
    }

    #[test]
    fn test_extract_variant_name() {
        let gen = RustGenerator::new();

        assert_eq!(gen.extract_variant_name("Msg::Inc"), "Inc");
        assert_eq!(gen.extract_variant_name(".Inc"), "Inc");
        assert_eq!(gen.extract_variant_name("Dec"), "Dec");
    }

    #[test]
    fn test_tag_to_view_fn() {
        let gen = RustGenerator::new();

        assert_eq!(gen.tag_to_view_fn("col"), "col");
        assert_eq!(gen.tag_to_view_fn("button"), "button");
        assert_eq!(gen.tag_to_view_fn("text"), "text");
        // PLAN-032 T-03（D2）：tabs/tab 断裂映射已移除——tabs 走专属臂，
        // 残余形态落 `_ => "col"` 兜底。
        assert_eq!(gen.tag_to_view_fn("tabs"), "col");
        assert_eq!(gen.tag_to_view_fn("tab"), "col");
    }

    /// PLAN-032 T-03（D2）：tabs a2r 断裂映射修复——View::tabs 折叠构造
    ///（labels/contents 文档序/value 绑定运行时 position/variant/onselect
    /// 闭包物化 value 串载荷——convert_tabs 契约镜像）。
    #[test]
    fn test_tabs_codegen_view_tabs_folding() {
        let src = r#"
widget T {
    msg { Select(str) }
    model { var s str = "a" }
    view {
        tabs (value: .s, variant: "enclosed", onselect: .Select("a")) {
            tabslist {
                tabstrigger (value: "a") { text "Alpha" }
                tabstrigger (value: "b") { text "Beta" }
            }
            tabscontent (value: "a") { text "Content A" }
            tabscontent (value: "b") { text "Content B" }
        }
    }
    on { .Select(t) -> { .s = t } }
}
"#;
        let session = crate::session::CompilerSession::ui().with_backend("rust");
        let mut parser = crate::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast.stmts.iter().find_map(|s| match s {
            crate::ast::Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        }).expect("widget decl");
        let widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");
        let code = RustGenerator::new().generate(&widget).expect("generate");

        // labels/contents 折叠 + 文档序。
        assert!(
            code.contains("View::tabs(vec![\"Alpha\".to_string(), \"Beta\".to_string()])"),
            "labels 折叠: {}",
            &code[code.len().saturating_sub(4000)..]
        );
        assert!(code.contains(".contents(vec!["), "contents 折叠");
        assert!(code.contains("Content A"), "内容子树生成");
        // value 绑定 → 运行时 position 表达式（字面量串编译期匹配）。
        assert!(
            code.contains(".position(|v| *v == self.s.as_str()).unwrap_or(0)"),
            "value 绑定运行时匹配: {}",
            &code[code.len().saturating_sub(4000)..]
        );
        // variant full-path（生成物 use 预载不含 TabsVariant）。
        assert!(
            code.contains(".variant(auto_lang::ui::view::TabsVariant::parse(\"enclosed\"))"),
            "variant 发射"
        );
        // onselect 闭包物化：首参 = value 串（vals 表捕获）。
        assert!(
            code.contains("::Select(vals.get(idx).cloned().unwrap_or_else(|| idx.to_string()))"),
            "onselect 闭包载荷: {}",
            &code[code.len().saturating_sub(4000)..]
        );
        // 断裂映射残留清零。
        assert!(!code.contains("View::tabs()"), "无断裂 View::tabs() 空参");
        assert!(!code.contains("View::tab()"), "无不存在方法 View::tab()");
    }

    /// PLAN-032 T-03（D2）：046-tabs-variants 真源 a2r 全量生成 + 编译
    ///（code_editor e2e 先例：throwaway crate + cargo build——断言断裂
    /// 修复后 046 a2r 轨可编译）。Ignored by default（编译依赖树分钟级）。
    #[test]
    #[ignore = "e2e compile: minutes on a cold target dir"]
    fn test_tabs_codegen_046_compiles() {
        let src = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/capability-tests/tabs-variants/src/front/app.at"
        ))
        .expect("read 046 app.at");
        let session = crate::session::CompilerSession::ui().with_backend("rust");
        let mut parser = crate::Parser::from(src.as_str()).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast.stmts.iter().find_map(|s| match s {
            crate::ast::Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        }).expect("widget decl");
        let widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");
        let code = RustGenerator::new().generate(&widget).expect("generate");

        let main_rs = format!(
            "#![allow(dead_code, unused)]\n{}\nfn main() {{}}\n",
            code
        );
        let root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let root = std::path::Path::new(&root)
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf();
        let tmp = root.join("target").join("tabs-046-e2e");
        let src_dir = tmp.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(src_dir.join("main.rs"), &main_rs).unwrap();
        let auto_lang_path = root.join("crates").join("auto-lang");
        std::fs::write(
            tmp.join("Cargo.toml"),
            format!(
                "[package]\nname = \"tabs_046_e2e\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nauto-lang = {{ path = {:?}, features = [\"ui-iced\"] }}\n\n[workspace]\n",
                auto_lang_path.to_string_lossy().replace("\\\\", "/")
            ),
        )
        .unwrap();
        let output = std::process::Command::new("cargo")
            .arg("build")
            .current_dir(&tmp)
            .output()
            .unwrap_or_else(|e| panic!("cargo build in {}: {}", tmp.display(), e));
        assert!(
            output.status.success(),
            "046 a2r 生成物编译失败。\n--- main.rs 尾段 ---\n{}\n--- stderr ---\n{}",
            &main_rs[main_rs.len().saturating_sub(6000)..],
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // ========== Plan 180 Phase 7: tailwind_to_methods tests ==========

    #[test]
    fn test_tailwind_single_padding() {
        assert_eq!(tailwind_single_to_method("p-4"), ".p(4)");
    }

    #[test]
    fn test_tailwind_single_padding_xy() {
        assert_eq!(tailwind_single_to_method("px-4"), ".px(4)");
        assert_eq!(tailwind_single_to_method("py-2"), ".py(2)");
    }

    #[test]
    fn test_tailwind_single_margin() {
        assert_eq!(tailwind_single_to_method("m-4"), ".m(4)");
        assert_eq!(tailwind_single_to_method("mx-2"), ".mx(2)");
        assert_eq!(tailwind_single_to_method("my-2"), ".my(2)");
    }

    #[test]
    fn test_tailwind_single_gap() {
        assert_eq!(tailwind_single_to_method("gap-4"), ".gap(4)");
    }

    #[test]
    fn test_tailwind_single_bg() {
        assert_eq!(tailwind_single_to_method("bg-white"), ".bg(\"white\")");
        assert_eq!(tailwind_single_to_method("bg-blue-500"), ".bg(\"blue-500\")");
    }

    #[test]
    fn test_tailwind_single_width() {
        assert_eq!(tailwind_single_to_method("w-full"), ".w_full()");
        assert_eq!(tailwind_single_to_method("w-10"), ".w(10)");
    }

    #[test]
    fn test_tailwind_single_height() {
        assert_eq!(tailwind_single_to_method("h-full"), ".h_full()");
        assert_eq!(tailwind_single_to_method("h-12"), ".h(12)");
    }

    #[test]
    fn test_tailwind_single_layout() {
        assert_eq!(tailwind_single_to_method("flex"), ".flex()");
        assert_eq!(tailwind_single_to_method("flex-1"), ".flex1()");
        assert_eq!(tailwind_single_to_method("flex-row"), ".flex_row()");
        assert_eq!(tailwind_single_to_method("flex-col"), ".flex_col()");
        assert_eq!(tailwind_single_to_method("items-center"), ".items_center()");
        assert_eq!(tailwind_single_to_method("justify-center"), ".justify_center()");
        assert_eq!(tailwind_single_to_method("justify-between"), ".justify_between()");
    }

    #[test]
    fn test_tailwind_single_border_radius() {
        assert_eq!(tailwind_single_to_method("rounded"), ".rounded()");
        assert_eq!(tailwind_single_to_method("rounded-sm"), ".rounded_sm()");
        assert_eq!(tailwind_single_to_method("rounded-md"), ".rounded_md()");
        assert_eq!(tailwind_single_to_method("rounded-lg"), ".rounded_lg()");
    }

    #[test]
    fn test_tailwind_single_border() {
        assert_eq!(tailwind_single_to_method("border"), ".border()");
    }

    #[test]
    fn test_tailwind_single_font_weight() {
        assert_eq!(tailwind_single_to_method("font-bold"), ".font_bold()");
        assert_eq!(tailwind_single_to_method("font-medium"), ".font_medium()");
    }

    #[test]
    fn test_tailwind_single_text_color() {
        assert_eq!(tailwind_single_to_method("text-slate-500"), ".text_color(\"slate-500\")");
    }

    #[test]
    fn test_tailwind_to_methods_chain() {
        let result = tailwind_to_methods("View::col()", "gap-4 p-4 bg-white items-center");
        assert_eq!(result, "View::col().gap(4).p(4).bg(\"white\").items_center()");
    }

    #[test]
    fn test_tailwind_to_methods_empty() {
        let result = tailwind_to_methods("View::col()", "");
        assert_eq!(result, "View::col()");
    }

    #[test]
    fn test_tailwind_to_methods_unknown_classes_passthrough() {
        let result = tailwind_to_methods("View::col()", "p-4 unknown-class gap-2");
        assert_eq!(result, "View::col().p(4).gap(2).style(\"unknown-class\")");
    }

    #[test]
    fn test_tailwind_to_methods_complex() {
        let result = tailwind_to_methods(
            "View::row()",
            "w-full h-full justify-center items-center bg-white"
        );
        assert_eq!(
            result,
            "View::row().w_full().h_full().justify_center().items_center().bg(\"white\")"
        );
    }

    #[test]
    fn test_text_element_with_text_prop() {
        // text "Hello, World!" parsed as Element { tag: "text", props: { text: "Hello, World!" } }
        let node = AuraNode::element("text")
            .with_prop("text", crate::ast::Expr::Str("Hello, World!".into()));

        let mut gen = RustGenerator::new();
        let code = gen.generate_view_tree(&node);
        assert!(code.contains("View::text(\"Hello, World!\".to_string())"), "got: {}", code);
        assert!(!code.contains(".build()"), "View::text(str) returns View directly, got: {}", code);
    }

    #[test]
    fn test_svg_lowering_in_rust_view() {
        let circle = AuraNode::element("circle")
            .with_prop("cx", crate::ast::Expr::Str("120".into()))
            .with_prop("cy", crate::ast::Expr::Str("120".into()))
            .with_prop("r", crate::ast::Expr::Str("114".into()));
        let svg_node = AuraNode::element("svg")
            .with_prop("viewBox", crate::ast::Expr::Str("0 0 240 240".into()))
            .with_prop("style", crate::ast::Expr::Str("w-56 h-56".into()))
            .with_child(circle);

        let mut gen = RustGenerator::new();
        let code = gen.generate_view_tree(&svg_node);
        assert!(code.contains("View::image_styled("), "got: {}", code);
        assert!(code.contains("svgdoc:<svg"), "got: {}", code);
        assert!(code.contains("viewBox=\\\"0 0 240 240\\\""), "got: {}", code);
        assert!(code.contains("<circle cx=\\\"120\\\" cy=\\\"120\\\" r=\\\"114\\\"/>"), "got: {}", code);
        assert!(code.contains("\"w-56 h-56\""), "got: {}", code);
    }

    /// Plan 043 M5 #1: multi-param msg variants emit a multi-field Rust enum.
    fn widget_with_msg(variants: Vec<AuraMsgVariant>) -> AuraWidget {
        AuraWidget {
            named_views: Vec::new(),
            actions: None,
            timers: Vec::new(),
            name: "Shell".to_string(),
            state_vars: vec![],
            computed: vec![],
            messages: vec![AuraMessage { variants }],
            view_tree: AuraNode::element("col"),
            handlers: std::collections::BTreeMap::new(),
            props: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
            setup: None,
        }
    }

    #[test]
    fn test_msg_multi_param_enum_output() {
        use crate::ast::Type;
        let widget = widget_with_msg(vec![
            AuraMsgVariant { payload_names: vec![], name: "Init".to_string(), quoted: false, payload: vec![] },
            AuraMsgVariant { payload_names: vec![], name: "Complete".to_string(), quoted: false, payload: vec![Type::StrSlice, Type::Int] },
            AuraMsgVariant {
                name: "RunSmart".to_string(),
                quoted: false,
                payload: vec![Type::Int, Type::StrSlice, Type::Unknown],
                payload_names: vec![],
            },
            AuraMsgVariant { payload_names: vec![], name: "SetTag".to_string(), quoted: false, payload: vec![Type::StrSlice] },
        ]);

        let mut gen = RustGenerator::new();
        let code = gen.generate(&widget).unwrap();

        // Unit variant.
        assert!(code.contains("    Init,\n"), "unit variant Init, got:\n{}", code);
        // Two-field variant (StrSlice renders as String in the Rust backend).
        assert!(
            code.contains("    Complete(String, i32),\n"),
            "multi-param Complete should emit two fields, got:\n{}",
            code
        );
        // Single-field variant still one field (regression guard).
        assert!(
            code.contains("    SetTag(String),\n"),
            "single-param SetTag stays one field, got:\n{}",
            code
        );
    }

    /// PLAN-015 D3 金样:terminal `onmenu` 事件 → `on_menu: Some(...)` 直发
    /// (载荷走 TerminalCore 菜单通道,消息只当触发器,对齐 oninput 模式);
    /// 无 onmenu 时保持 `on_menu: None`。VM 臂(aura_view_builder
    /// convert_terminal)同键集已支持,本样钉死 rust 发射臂双轨同源。
    #[test]
    fn terminal_onmenu_emits_on_menu_direct_msg() {
        let gen_one = |onmenu_attr: &str| {
            let src = format!(
                r#"
widget TermApp {{
    msg {{ Init, Tick, KeyIn, Menu }}

    model {{
        var lines List<str> = []
    }}

    on {{
        .Init -> {{
            .lines = []
        }}
    }}

    view {{
        terminal {{
            key: "auto-term"
            cols: 80
            rows: 24
            lines: .lines
            oninput: .KeyIn
            {onmenu_attr}
        }}
    }}
}}
"#
            );
            let session = crate::session::CompilerSession::ui().with_backend("rust");
            let mut parser = crate::Parser::from(src.as_str()).with_session(session);
            let ast = parser.parse().expect("parse");
            let decl = ast
                .stmts
                .iter()
                .find_map(|s| match s {
                    crate::ast::Stmt::WidgetDecl(d) => Some(d),
                    _ => None,
                })
                .expect("widget decl");
            let widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");
            let mut gen = RustGenerator::new();
            gen.generate_rust(&widget).expect("generate rust")
        };

        let with_menu = gen_one("onmenu: .Menu");
        assert!(
            with_menu.contains("on_menu: Some(TermAppMsg::Menu)"),
            "onmenu 必须直发 on_menu 信号位:\n{with_menu}"
        );
        let without_menu = gen_one("");
        assert!(
            without_menu.contains("on_menu: None"),
            "无 onmenu 时保持 None(缺省零扰):\n{without_menu}"
        );
    }

    /// PLAN-022 T-03 金样(分隔条形态):`mouse-area` 不再落 "col" 通配
    /// ——onmousedown → on_click 槽、onmouseup → on_release、onmousemove
    /// → PointerMoveHandler 转发归一坐标(coords "WxH" → logical_extent);
    /// 拖拽捕获层的 `absolute inset-0` 样式经 Style::parse 保留。
    #[test]
    fn mouse_area_emits_events_and_logical_extent() {
        let src = r#"
widget SplitApp {
    msg { Press(int), Drag(float, float), Drop }

    model {
        var dragging int = 0
    }

    view {
        mouse-area (coords: "1000x1000", style: "absolute inset-0 z-30", onmousemove: .Drag, onmouseup: .Drop) {}
    }
}
"#;
        let session = crate::session::CompilerSession::ui().with_backend("rust");
        let mut parser = crate::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                crate::ast::Stmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");
        let mut gen = RustGenerator::new();
        let code = gen.generate_rust(&widget).expect("generate rust");

        assert!(
            code.contains("View::MouseArea {"),
            "mouse-area 必须发射 View::MouseArea(不得落 col 通配):\n{code}"
        );
        assert!(
            code.contains("PointerMoveHandler::new(move |x: f32, y: f32| SplitAppMsg::Drag(x, y))"),
            "无参 onmousemove 必须转发归一坐标:\n{code}"
        );
        assert!(
            code.contains("on_release: Some(SplitAppMsg::Drop)"),
            "onmouseup 必须接 on_release 槽:\n{code}"
        );
        assert!(
            code.contains("logical_extent: Some((1000.0, 1000.0))"),
            "coords WxH 必须落 logical_extent(f32 字面量):\n{code}"
        );
        assert!(
            code.contains("Style::parse(\"absolute inset-0 z-30\").ok()"),
            "样式串必须经 Style::parse 保留(空层判定/定位依赖):\n{code}"
        );
    }

    /// PLAN-022 T-03:带 onmousedown 的分隔条薄条形态(定尺寸 div +
    /// mouse-area Fill 子件)——press 闭包发射 + Fill 样式。
    #[test]
    fn mouse_area_press_slot_emits_on_click() {
        let src = r#"
widget SplitApp {
    msg { Press(int), Tick }

    model {
        var dragging int = 0
    }

    view {
        mouse-area (style: "w-full h-full", onmousedown: .Press(7)) {}
    }
}
"#;
        let session = crate::session::CompilerSession::ui().with_backend("rust");
        let mut parser = crate::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                crate::ast::Stmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");
        let mut gen = RustGenerator::new();
        let code = gen.generate_rust(&widget).expect("generate rust");

        assert!(
            code.contains("on_click: Some(SplitAppMsg::Press(7))"),
            "onmousedown 必须接 on_click 槽(显式参消息值直发):\n{code}"
        );
        assert!(
            code.contains("View::MouseArea {"),
            "mouse-area 必须发射 View::MouseArea:\n{code}"
        );
    }

    /// PLAN-019 D4 金样:terminal `onkeydown.<键名>` 事件 →
    /// `shortcuts: vec![("<键名>", AppMsg)]`(应用级捷径表;命中拦截/
    /// 未命中透传的表来源);同样钉死 key 动态绑定(`key: .field` →
    /// self.field 克隆,T-B 槽位键形态)。
    #[test]
    fn terminal_onkeydown_emits_shortcuts_and_dynamic_key() {
        let gen_one = || {
            let src = r#"
widget TermApp {
    msg { Init, Tick, KeyIn, Shortcut(int) }

    model {
        var lines List<str> = []
        var slotkey str = "pane-1"
    }

    on {
        .Init -> {
            .lines = []
        }
    }

    view {
        terminal {
            key: .slotkey
            cols: 80
            rows: 24
            lines: .lines
            oninput: .KeyIn
            onkeydown.ctrl.shift.t: .Shortcut(1)
            onkeydown.ctrl.shift.e: .Shortcut(2)
        }
    }
}
"#;
            let session = crate::session::CompilerSession::ui().with_backend("rust");
            let mut parser = crate::Parser::from(src).with_session(session);
            let ast = parser.parse().expect("parse");
            let decl = ast
                .stmts
                .iter()
                .find_map(|s| match s {
                    crate::ast::Stmt::WidgetDecl(d) => Some(d),
                    _ => None,
                })
                .expect("widget decl");
            let widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");
            let mut gen = RustGenerator::new();
            gen.generate_rust(&widget).expect("generate rust")
        };

        let code = gen_one();
        assert!(
            code.contains("shortcuts: vec!["),
            "onkeydown 声明必须发射 shortcuts 表:\n{code}"
        );
        assert!(
            code.contains("\"ctrl.shift.t\".to_string()"),
            "规范化键名必须原样入表:\n{code}"
        );
        assert!(
            code.contains("self.slotkey.clone()"),
            "key 动态绑定必须发射 self.field 克隆(T-B 槽位键):\n{code}"
        );
    }

    /// PLAN-019 用户反馈回归:terminal `scheme: .field` / `scroll_offset:
    /// .field` 的 FieldAccess 绑定必须发射 self.field 索引(此前仅认
    /// Int 字面量/Ident,scheme 静默回落 -1 → ◐ 按钮失效)。
    #[test]
    fn terminal_scheme_and_scroll_fieldaccess_bindings_emit() {
        let src = r#"
widget TermApp {
    msg { Init }

    model {
        var lines List<str> = []
        var schemesel int = 0
        var scrollofs int = 0
    }

    on {
        .Init -> {
            .lines = []
        }
    }

    view {
        terminal {
            key: "pane-1"
            cols: 80
            rows: 24
            lines: .lines
            scheme: .schemesel
            scroll_offset: .scrollofs
            oninput: .KeyIn2
        }
    }
}
"#;
        // 上面的 oninput 引用需要存在,msg 里没有 KeyIn2 —— 用 KeyIn。
        let src = src.replace(".KeyIn2", ".Init");
        let session = crate::session::CompilerSession::ui().with_backend("rust");
        let mut parser = crate::Parser::from(src.as_str()).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                crate::ast::Stmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");
        let mut gen = RustGenerator::new();
        let code = gen.generate_rust(&widget).expect("generate rust");

        assert!(
            code.contains("scheme: (self.schemesel) as i32"),
            "scheme FieldAccess 绑定必须发射 (self.field) as i32:
{code}"
        );
        assert!(
            code.contains("scroll_offset: (self.scrollofs) as u16"),
            "scroll_offset FieldAccess 绑定必须发射 (self.field) as u16:
{code}"
        );
    }

    /// PLAN-019 T-00 使能:row 直属 for 摊平 —— `.children(<map>.collect())`
    /// 批量加,而非 ForLoop 的 col 包装单子(纵向堆叠缺陷);VM 臂
    /// convert_row Plan 047 既有同款语义,本样钉死 rust 发射臂对齐。
    #[test]
    fn row_direct_for_flattens_to_children_bulk_add() {
        let src = r#"
widget TabBar {
    msg { TabActivate(int) }

    model {
        var labels List<str> = ["a", "b"]
    }

    view {
        row {
            for i, label in .labels {
                button (text: label) {
                    onclick: .TabActivate(i)
                }
            }
        }
    }
}
"#;
        let session = crate::session::CompilerSession::ui().with_backend("rust");
        let mut parser = crate::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                crate::ast::Stmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");
        let mut gen = RustGenerator::new();
        let code = gen.generate(&widget).expect("generate");

        assert!(
            code.contains("View::row().children("),
            "row 直属 for 必须摊平为 children 批量加:\n{code}"
        );
        assert!(
            !code.contains("View::row().child(View::col().children("),
            "row 内不得再出现 for 的 col 包装单子:\n{code}"
        );
    }

    #[test]
    fn test_time_and_storage_builtins_lowering() {
        let src = r#"
widget StorageDemo {
    msg { Save, Load, Tick }
    model {
        var x str = ""
        var t int = 0
    }
    view { col { button "ok" { onclick: .Save } } }
    on {
        .Save -> {
            storage.set("k", "v")
            storage.remove("k")
        }
        .Load -> {
            .x = storage.get("k")
        }
        .Tick -> {
            .t = Time.now_sec()
        }
    }
}
"#;
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let mut code = String::new();
        for stmt in &ast.stmts {
            if let crate::ast::Stmt::WidgetDecl(decl) = stmt {
                let widget = crate::aura::extract::extract_widget_from_decl(decl).unwrap();
                let mut gen = RustGenerator::new();
                code = gen.generate(&widget).unwrap();
            }
        }
        assert!(code.contains("auto_lang::vm::ffi::stdlib::shim_storage_set"), "storage.set lowered:\n{code}");
        assert!(code.contains("auto_lang::vm::ffi::stdlib::shim_storage_get"), "storage.get lowered:\n{code}");
        assert!(code.contains("auto_lang::vm::ffi::stdlib::shim_storage_remove"), "storage.remove lowered:\n{code}");
        assert!(code.contains("auto_lang::vm::ffi::stdlib::shim_time_now_sec() as i32"), "Time.now_sec lowered:\n{code}");
    }
}

// ── PLAN-571: codegen 臂 button preset 注入（产物级断言）────────────
// preset 注入本体 cfg(feature="ui")（见 with_button_preset）；tf 档不带 ui-iced
// （Plan 507），断言随门关闭——日常档 cargo t（带 ui-iced）承接。
#[cfg(all(test, feature = "ui"))]
mod plan571_button_preset_codegen_tests {
    use super::*;

    fn gen_button_view(widget_src: &str) -> String {
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(widget_src).with_session(session);
        let ast = parser.parse().expect("parse");
        let mut codes = Vec::new();
        for stmt in &ast.stmts {
            if let crate::ast::Stmt::WidgetDecl(decl) = stmt {
                let widget = crate::aura::extract::extract_widget_from_decl(decl)
                    .unwrap_or_else(|e| panic!("extract: {e:?}"));
                let mut gen = RustGenerator::new();
                codes.push(gen.generate(&widget).expect("generate"));
            }
        }
        codes.join("\n")
    }

    #[test]
    fn plain_button_gets_default_neutral_preset() {
        let src = r#"
widget Demo {
    msg { Tap }
    view {
        col {
            button "Save" { onclick: .Tap }
        }
    }
}
"#;
        let code = gen_button_view(src);
        // PLAN-080：缺省 button = preflight 等价 chromeless——无 variant/size
        // 预设注入（原 PLAN-571 UA 中性 preset 改为显式 variant="default" 档）。
        assert!(
            !code.contains("bg-muted"),
            "缺省 button 不再注入 UA 中性 preset:\n{}",
            code
        );
        assert!(
            !code.contains("h-10"),
            "缺省 size preset 不再前置:\n{}",
            code
        );
        assert!(
            !code.contains("bg-primary"),
            "缺省 button 不得落主题色填充:\n{}",
            code
        );
    }

    #[test]
    fn user_class_stands_alone_without_preset() {
        let src = r#"
widget Demo {
    msg { Tap }
    view {
        col {
            button "Save" { onclick: .Tap, style: "px-2.5 py-1.5 text-xs" }
        }
    }
}
"#;
        let code = gen_button_view(src);
        assert!(
            code.contains("px-2.5 py-1.5 text-xs"),
            "user class 保留:\n{}",
            code
        );
        // PLAN-080：无 variant 时不再前置任何 preset——user class 独挑外观。
        assert!(
            !code.contains("bg-muted border border-border"),
            "无 variant 不注入 UA preset:\n{}",
            code
        );
    }

    #[test]
    fn explicit_primary_variant_keeps_accent_fill() {
        let src = r#"
widget Demo {
    msg { Tap }
    view {
        col {
            button "Save" { onclick: .Tap, variant: "primary" }
        }
    }
}
"#;
        let code = gen_button_view(src);
        assert!(
            code.contains("bg-primary text-primary-foreground"),
            "variant=primary 保持主题色填充:\n{}",
            code
        );
        assert!(
            !code.contains("bg-muted border"),
            "primary 不带中性基线:\n{}",
            code
        );
    }
}

// ── PLAN-039: a2r codegen 缺口五臂（产物级断言，§5.1 定案记录）──────
// 五臂均核心 codegen 路径（无 ui-iced 依赖），tf/t 双档共担：
//   D1-A button ondblclick → MouseArea 包裹降级（klondike ×8 载体）
//   D4   grid cols 动态运行期求值（minesweeper `.store.cols` 载体）
//   ②   icon 动态 class 运行期拼串（CardSuit `class: .style` 载体）
//   ④   textarea value Dot/点链容差（kanban `.store.edit_detail` 载体）
//   D5   outlet 单路由折平/多路由响亮拒（kanban routes 载体）
//   D3-A ondragover 家族显式 not-yet 拒绝（kanban board ×3 载体）
#[cfg(test)]
mod plan039_a2r_gap_codegen_tests {
    use super::*;

    fn gen_first_widget(src: &str) -> String {
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let mut code = String::new();
        for stmt in &ast.stmts {
            if let crate::ast::Stmt::WidgetDecl(decl) = stmt {
                let widget = crate::aura::extract::extract_widget_from_decl(decl)
                    .unwrap_or_else(|e| panic!("extract: {e:?}"));
                let mut gen = RustGenerator::new();
                code = gen.generate(&widget).expect("generate");
            }
        }
        assert!(!code.is_empty(), "no widget generated");
        code
    }

    /// D1-A：button ondblclick 剥离事件流（不落拒绝门）+ 出口 MouseArea
    /// 包裹（on_double_click 直发消息；onclick 保留在按钮本体）。
    #[test]
    fn button_ondblclick_mousearea_wrap() {
        let code = gen_first_widget(r#"
widget Demo {
    msg { AutoSendWaste, AutoSendCol(int) }
    view {
        col {
            button "发牌" { onclick: .AutoSendWaste, ondblclick: .AutoSendCol(0) }
        }
    }
}
"#);
        assert!(
            code.contains("View::MouseArea { content: Box::new(View::button"),
            "ondblclick 必须 MouseArea 包裹降级:\n{code}"
        );
        assert!(
            code.contains("on_double_click: Some(DemoMsg::AutoSendCol(0))"),
            "双击直发消息带参:\n{code}"
        );
        assert!(
            code.contains("on_click(|_| DemoMsg::AutoSendWaste)"),
            "onclick 保留按钮本体:\n{code}"
        );
        assert!(
            !code.contains("not in the recognized vocabulary"),
            "ondblclick 不得落拒绝门:\n{code}"
        );
    }

    /// D1-A 边界：非 button tag 的 ondblclick 维持拒绝门响亮拒（I1）。
    /// （text 叶子短路路径不经过事件门——既有行为，非本臂面。）
    #[test]
    fn non_button_ondblclick_stays_rejected() {
        let code = gen_first_widget(r#"
widget Demo {
    msg { Noop }
    view {
        col {
            ondblclick: .Noop
            text "x"
        }
    }
}
"#);
        assert!(
            code.contains("not in the recognized vocabulary"),
            "非 button ondblclick 维持响亮拒:\n{code}"
        );
    }

    /// D4：grid cols 动态表达式 → 运行期求值臂（字面量维持编译期路径）。
    #[test]
    fn grid_dynamic_cols_runtime_eval() {
        let code = gen_first_widget(r#"
widget Demo {
    msg { NewGame }
    model { var cols int = 9 }
    view {
        grid (cols: .cols) {
            text "cell"
        }
    }
}
"#);
        assert!(
            code.contains(".cols((self.cols) as usize)"),
            "动态 cols 运行期求值:\n{code}"
        );
        let lit = gen_first_widget(r#"
widget Demo {
    msg { NewGame }
    view {
        grid (cols: 3) { text "c" }
    }
}
"#);
        assert!(
            lit.contains(".cols(3)"),
            "字面量 cols 编译期路径不变:\n{lit}"
        );
    }

    /// ②：icon class 动态表达式 → 运行期拼串 + w-/h- 缺省档运行期复刻。
    #[test]
    fn icon_dynamic_class_runtime_eval() {
        let code = gen_first_widget(r#"
widget CardSuit (style: str = "w-8 h-8") {
    view {
        icon (name: "spade", class: .style) {}
    }
}
"#);
        assert!(
            code.contains("let __c = format!(\"{}\", self.style);"),
            "动态 class 运行期求值:\n{code}"
        );
        assert!(
            code.contains("View::image_styled(") && code.contains("&__s"),
            "image_styled 消费运行期串:\n{code}"
        );
        assert!(
            code.contains("t.starts_with(\"w-\"))"),
            "w- 缺省档运行期复刻:\n{code}"
        );
    }

    /// ④：textarea value Dot/点链容差（单级 + `.store.x` 多级拍平形）。
    #[test]
    fn textarea_dot_value_bindings() {
        let code = gen_first_widget(r#"
widget board {
    msg { EditDetail(str) }
    model { var detail str = "hello" }
    view {
        textarea { value: .detail, oninput: .EditDetail }
    }
}
"#);
        assert!(
            code.contains(".value(format!(\"{}\", self.detail))"),
            "单级 Dot value:\n{code}"
        );
        let multi = gen_first_widget(r#"
widget board {
    msg { EditDetail(str) }
    view {
        textarea { value: .store.edit_detail, oninput: .EditDetail }
    }
}
"#);
        assert!(
            multi.contains(".value(format!(\"{}\", self.store.edit_detail))"),
            "多级点链 value（kanban 形态）:\n{multi}"
        );
        assert!(
            !multi.contains("self.."),
            "双点伪影必须剥除:\n{multi}"
        );
    }

    /// D5：单路由 routes{"/"->use X} + outlet → 持久子件折平直用。
    #[test]
    fn outlet_single_route_folds_to_persistent_child() {
        let code = gen_first_widget(r#"
widget App {
    routes {
        "/" -> use board
    }
    msg { Init }
    view {
        col {
            outlet
        }
    }
}
"#);
        assert!(
            code.contains("board(boardMsg)"),
            "包装变体入册:\n{code}"
        );
        assert!(
            code.contains("self.board.view().map_msg(|m| AppMsg::board(m))"),
            "outlet 折平持久子件直用:\n{code}"
        );
        assert!(
            code.contains("pub board: board"),
            "持久子件字段:\n{code}"
        );
        assert!(
            !code.contains("View::empty()"),
            "单路由不再静默空屏:\n{code}"
        );
    }

    /// D5 边界：多路由 + outlet → 响亮拒（原 View::empty 静默升级）。
    #[test]
    fn outlet_multi_route_loud_reject() {
        let code = gen_first_widget(r#"
widget App {
    routes {
        "/" -> use home
        "/other" -> use other
    }
    view {
        col { outlet }
    }
}
"#);
        assert!(
            code.contains("std::compile_error!(\"a2r codegen: multi-route/parametric `routes` + `outlet` not yet supported (PLAN-039 D5"),
            "多路由响亮拒带 P039 指针:\n{code}"
        );
        assert!(
            !code.contains("home(homeMsg)"),
            "多路由不注册折平子件:\n{code}"
        );
    }

    /// D3-A：ondragover 家族显式 not-yet 拒绝（带 P039 债指针，区别
    /// 通用未知事件臂）。
    #[test]
    fn drag_family_events_rejected_with_debt_pointer() {
        let code = gen_first_widget(r#"
widget board {
    msg { AllowDrop }
    view {
        col {
            ondragover.prevent: .AllowDrop
            text "x"
        }
    }
}
"#);
        assert!(
            code.contains("std::compile_error!(\"a2r codegen: event `ondragover.prevent` (HTML5 drag family) not yet supported in a2r compiled mode (PLAN-039 D3-A"),
            "drag 家族专属拒绝臂:\n{code}"
        );
        assert!(
            code.contains("P039 debt"),
            "债指针在案:\n{code}"
        );
    }

    /// T-04 发现臂①：`key:` = Vue reconciliation 提示词——认知且双轨
    /// 同弃（非拒绝面），for 循环键提示不得打断生成。
    #[test]
    fn key_prop_silently_skipped() {
        let code = gen_first_widget(r#"
widget Demo {
    msg { Tap }
    model { var items = ["a", "b"] }
    view {
        col {
            for it in .items {
                button it {
                    key: it,
                    onclick: .Tap
                }
            }
        }
    }
}
"#);
        assert!(
            !code.contains("prop `key` not in the recognized vocabulary"),
            "key 不得落拒绝门:\n{code}"
        );
    }

    /// T-04 发现臂②：.at 字面量含引号/反斜杠时发射必须再转义
    /// （kanban boards_store.at JSON 体拼接先例：源码层 \" 解析后裸发射
    /// 曾打断生成物语法）。
    #[test]
    fn string_literals_reescaped_on_emission() {
        let code = gen_first_widget(r#"
widget Demo {
    model { var payload str = "{\"title\":\"" + "x" + "\"}" }
    view { text .payload }
}
"#);
        assert!(
            code.contains("\\\"title\\\":"),
            "引号必须转义发射（生成物含 \\\"title\\\": 形态）:\n{code}"
        );
    }

    /// PLAN-039 T-12（批次 E，E-D4）：内建方法翻译表——接收者类型格
    /// 分流。Vec<Value> 态 `.len()` 落原生 len（i64 收口）；String 态
    /// `.slice/.lower` 落 __at_slice/to_lowercase；Value 局部（集合元素
    /// 赋值）`.len()` 落 __at_len shim。此前方法名一律落 Dot 字段降链
    /// 再挂尾巴 `()`（launcher ic1.len() 25 错株）。
    #[test]
    fn builtin_method_table_receiver_kinds() {
        let code = gen_first_widget(r#"
widget Demo {
    msg { Tap }
    model {
        var items = []
        var label str = "Hello"
    }
    view { col { text "t" } }
    on {
        .Tap -> {
            var n int = items.len()
            var head str = label.slice(0, 2)
            var lo str = label.lower()
            var ic1 = items[0]
            var m int = ic1.len()
        }
    }
}
"#);
        assert!(
            code.contains(".len() as i32"),
            "Vec 集合 len 落原生（i32 收口）:\n{code}"
        );
        assert!(
            code.contains("__at_slice(&(self.label)"),
            "String slice 走 __at_slice:\n{code}"
        );
        assert!(
            code.contains(".to_lowercase()"),
            "String lower 落 to_lowercase:\n{code}"
        );
        assert!(
            code.contains("__at_len(&(ic1))"),
            "Value 局部 len 走 __at_len shim:\n{code}"
        );
    }

    /// PLAN-039 T-12（E-D1 使用点包裹）：Value 操作数的数值使用点降链
    /// ——比较/取模的 Value 侧包 `__at_num(&(..))`（klondike w_card >= 39
    /// 与 % 13 株）；同型数值比较不受扰动。
    #[test]
    fn value_numeric_use_site_wrap() {
        let code = gen_first_widget(r#"
widget Demo {
    msg { Tap }
    model {
        var items = []
        var picked int = 0
    }
    view { col { text "t" } }
    on {
        .Tap -> {
            var w = items[0]
            if w >= 39 {
                .picked = w % 13
            }
        }
    }
}
"#);
        assert!(
            code.contains("__at_num(&(w))) >= (39)"),
            "Value 局部比较包裹 __at_num:\n{code}"
        );
        assert!(
            code.contains("__at_num(&(w))) % (13)"),
            "Value 局部取模包裹 __at_num:\n{code}"
        );
    }

    /// PLAN-039 T-12（批次 E）：`.store.<value 字段>.<sub>` 多级链降链
    /// ——store 字段类型表判定中间级 Value（记录字面量株），view 位组件
    /// 参数与 handler 位表达式双消费面；`.store.<标量字段>` 直发不变。
    #[test]
    fn store_value_field_chain_demotion() {
        let src = r#"
store DemoStore {
    model {
        var waste_card = { id: 0 }
        var score int = 0
    }
    msg { Tap }
    on { .Tap -> { .score = 1 } }
}
widget Demo {
    msg { Tap }
    view {
        col { text .store.waste_card.suit }
    }
    on {
        .Tap -> {
            .score = 2
        }
    }
}
"#;
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let mut code = String::new();
        let mut gen = RustGenerator::new();
        for stmt in &ast.stmts {
            match stmt {
                crate::ast::Stmt::StoreDecl(store) => {
                    gen.register_store("store", store.name.as_str());
                    gen.prime_store_field_types(store);
                }
                crate::ast::Stmt::WidgetDecl(decl) => {
                    let widget = crate::aura::extract::extract_widget_from_decl(decl)
                        .unwrap_or_else(|e| panic!("extract: {e:?}"));
                    code = gen.generate(&widget).expect("generate");
                }
                _ => {}
            }
        }
        assert!(
            code.contains(".waste_card[\"suit\"]"),
            "store Value 字段多级链降链发射:\n{code}"
        );
    }

    /// PLAN-039 T-13（批次 E，E-D5-A）：用户型构造字面量（`Meta { … }`
    /// 解析为 Expr::Node）发射 struct 初始化 + Default 兜底——此前落
    /// `/* expr */` 占位（kanban store model meta 初始式株）。
    #[test]
    fn user_type_constructor_literal_emits_struct_init() {
        let code = gen_first_widget(r#"
widget Demo {
    msg { Tap }
    model {
        var meta Meta = Meta { source_root: "", count_active: 0 }
    }
    view { col { text "t" } }
    on { .Tap -> { .meta = .meta } }
}
"#);
        assert!(
            code.contains("Meta { source_root:"),
            "Node 构造字面量必须发射 struct 初始化:
{code}"
        );
        assert!(
            code.contains("..Default::default()"),
            "字段缺漏由 Default 兜底:
{code}"
        );
    }

    /// PLAN-039 T-14（E-D3）：显式 int 局部 + 集合元素 Asn——联合格
    /// 不得收（klondike c_card 株：若收则数值用点错包 __at_num(&i32)）。
    /// 配套：赋值点强转（c = items[0] → __at_num）+ 用点免包裹。
    #[test]
    fn declared_int_local_stays_out_of_value_union() {
        let code = gen_first_widget(r#"
widget Demo {
    msg { Tap }
    model { var items = [] }
    view { col { text "t" } }
    on {
        .Tap -> {
            var c int = 999
            if items.len() > 0 { c = items[0] }
            if c >= 13 { c = 0 }
        }
    }
}
"#);
        assert!(
            !code.contains("let mut c = serde_json::json!"),
            "显式 int 初始不得 json!:
{code}"
        );
        assert!(
            !code.contains("__at_num(&(c)"),
            "显式 int 用点不得包裹:
{code}"
        );
    }

    /// PLAN-039 T-13（批次 E，E-D2）：记录形状注册表——store 记录字面量
    /// 的 int 子字段访问按形状发射 as_i64（启发式名单外字段此前恒
    /// as_str——klondike waste_card.rank 株）。
    #[test]
    fn record_shape_int_field_access() {
        let src = r#"
store DemoStore {
    model {
        var waste_card = { id: 0, rank: 7, svg_src: "" }
    }
    msg { Tap }
    on { .Tap -> { .id = 1 } }
}
widget Demo {
    msg { Tap }
    view {
        col { text .store.waste_card.rank  text .store.waste_card.svg_src }
    }
}
"#;
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let mut code = String::new();
        let mut gen = RustGenerator::new();
        for stmt in &ast.stmts {
            match stmt {
                crate::ast::Stmt::StoreDecl(store) => {
                    gen.register_store("store", store.name.as_str());
                    gen.prime_store_field_types(store);
                }
                crate::ast::Stmt::WidgetDecl(decl) => {
                    let widget = crate::aura::extract::extract_widget_from_decl(decl)
                        .unwrap_or_else(|e| panic!("extract: {e:?}"));
                    code = gen.generate(&widget).expect("generate");
                }
                _ => {}
            }
        }
        assert!(
            code.contains(".waste_card[\"rank\"].as_i64()"),
            "int 子字段按形状走 as_i64:
{code}"
        );
        assert!(
            code.contains(".waste_card[\"svg_src\"].as_str()"),
            "str 子字段保持 as_str:
{code}"
        );
    }
}

/// PLAN-020 T-02: UI handler 臂 `.to_int()` 下降——`.to_int()` 在 UI 语料
/// 有语义(499 M3 charts 先例;VM 轨在库),但 a2r 生成的 f32/f64 无此方法。
/// trans/rust.rs 的 fix_numeric_conversion_methods 只覆盖非 UI 管线,此处
/// 对 handler 体补同一保守改写(IDENT/链式接收者 → `(expr as i32)`)。
/// PLAN-025 T-07：VM `math.*` 内建 → Rust f64 方法（handler 臂）。
/// `math.round(EXPR)` → `(EXPR).round()`——round/floor/ceil/abs/sqrt
/// 五族；括号配对扫描（正则不配嵌套）。
/// PLAN-036 T-02：`.contains(format!(...))` Pattern 修正——str::contains
/// 收 &str 模式，String 实参（format! 产物）补 `.as_str()`（Plan 374 的
/// `.contains(self.field)` 修正同族；条件串与 handler 体双消费——括号
/// 配对扫描，字符串字面量内的括号不计（`","` 实参形安全）。
/// PLAN-039 T-14（组件传型）：widget 是否持有子件引用（msg enum 与
/// type Msg 的口径面——有子件则 enum 必发，Msg 对齐 enum）。
fn widget_has_children(widget: &AuraWidget) -> bool {
    fn node_has_component(n: &crate::aura::AuraNode) -> bool {
        match n {
            crate::aura::AuraNode::Element { tag, children, .. } => {
                let tag_pascal = tag
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false);
                if tag_pascal {
                    return true;
                }
                children.iter().any(node_has_component)
            }
            crate::aura::AuraNode::Component { .. } => true,
            crate::aura::AuraNode::ForLoop { body, .. } => body.iter().any(node_has_component),
            crate::aura::AuraNode::Conditional { then_body, else_body, .. } => {
                then_body.iter().any(node_has_component)
                    || else_body
                        .as_ref()
                        .map(|e| e.iter().any(node_has_component))
                        .unwrap_or(false)
            }
            _ => false,
        }
    }
    node_has_component(&widget.view_tree)
        || widget
            .named_views
            .iter()
            .any(|(_, n)| node_has_component(n))
}

pub(crate) fn fix_contains_string_pattern_for_ui(content: &mut String) {
    let mut out = String::new();
    let mut rest = content.as_str();
    const PAT: &str = ".contains(format!(";
    while let Some(pos) = rest.find(PAT) {
        // 从 contains 的开括号起做括号配对（跳过字符串字面量）。
        let open = pos + ".contains(".len() - 1; // '(' of contains
        let bytes = rest.as_bytes();
        let mut depth = 0usize;
        let mut end = None;
        let mut i = open;
        let mut in_str = false;
        while i < bytes.len() {
            match bytes[i] {
                b'"' => in_str = !in_str,
                b'\\' if in_str => {
                    i += 1; // 转义符跳一
                }
                b'(' if !in_str => depth += 1,
                b')' if !in_str => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(i);
                        break;
                    }
                }
                _ => {}
            }
            i += 1;
        }
        let Some(e) = end else {
            out.push_str(&rest[..pos + PAT.len()]);
            rest = &rest[pos + PAT.len()..];
            continue;
        };
        out.push_str(&rest[..e]);
        out.push_str(".as_str()");
        rest = &rest[e..];
    }
    out.push_str(rest);
    if *content != out {
        *content = out;
    }
}

/// PLAN-036 T-02：String 局部变量收 Value 元素赋值降串——`var s str = ""`
/// 局部随后 `.s = .some_vec[idx]`（Vec<serde_json::Value> 元素 = Value），
/// VM 宽松串化在编译轨补 `.as_str().unwrap_or_default().to_string()`
///（识别面 = 体首 `let mut NAME = "".to_string()` 声明的局部名集；
/// dashboard face_apps src 先例）。
pub(crate) fn fix_string_local_value_element_assign(content: &mut String) {
    let mut names: Vec<String> = Vec::new();
    // 收集 `let mut NAME = "".to_string()` 声明（体内任意位置——while/if
    // 块单行拼接形态）。
    {
        let mut rest = content.as_str();
        while let Some(pos) = rest.find("let mut ") {
            let after = &rest[pos + "let mut ".len()..];
            let name_end = after
                .find(|c: char| c == ' ' || c == '=')
                .unwrap_or(after.len());
            let name = &after[..name_end];
            let tail = after[name_end..].trim_start();
            if !name.is_empty()
                && name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_')
                && tail.starts_with("= \"\".to_string()")
            {
                names.push(name.to_string());
            }
            rest = after;
        }
    }
    if names.is_empty() {
        return;
    }
    for name in names {
        let pat = format!("{name} = self.");
        let mut out = String::new();
        let mut rest = content.as_str();
        while let Some(pos) = rest.find(&pat) {
            let ok_before = pos == 0
                || {
                    let b = rest.as_bytes()[pos - 1];
                    !(b.is_ascii_alphanumeric() || b == b'_')
                };
            let seg = &rest[pos..];
            let end = seg.find(';').unwrap_or(seg.len());
            let stmt = &seg[..end];
            if ok_before
                && stmt.contains('[')
                && stmt.ends_with(']')
                && !stmt.contains(".as_str()")
            {
                out.push_str(&rest[..pos + end]);
                out.push_str(".as_str().unwrap_or_default().to_string()");
                rest = &rest[pos + end..];
            } else {
                out.push_str(&rest[..pos + pat.len()]);
                rest = &rest[pos + pat.len()..];
            }
        }
        out.push_str(rest);
        *content = out;
    }
}

pub(crate) fn lower_math_builtins_for_ui(content: &mut String) {
    const MAP: [(&str, &str); 5] = [
        ("math.round", "round"),
        ("math.floor", "floor"),
        ("math.ceil", "ceil"),
        ("math.abs", "abs"),
        ("math.sqrt", "sqrt"),
    ];
    for (pat, method) in MAP {
        let mut out = String::new();
        let mut rest = content.as_str();
        while let Some(pos) = rest.find(pat) {
            let after = &rest[pos + pat.len()..];
            if !after.starts_with('(') {
                out.push_str(&rest[..pos + pat.len()]);
                rest = after;
                continue;
            }
            out.push_str(&rest[..pos]);
            let bytes = after.as_bytes();
            let mut depth = 0usize;
            let mut end = None;
            for (i, b) in bytes.iter().enumerate() {
                match b {
                    b'(' => depth += 1,
                    b')' => {
                        depth -= 1;
                        if depth == 0 {
                            end = Some(i);
                            break;
                        }
                    }
                    _ => {}
                }
            }
            let Some(e) = end else {
                out.push_str(&rest[..pos + pat.len()]);
                rest = after;
                continue;
            };
            let expr = &after[1..e];
            out.push_str(&format!("({expr}).{method}()"));
            rest = &after[e + 1..];
        }
        out.push_str(rest);
        if *content != out {
            *content = out;
        }
    }
}

pub(crate) fn fix_numeric_conversion_methods_for_ui(content: &mut String) {
    for (method, cast) in [("to_float", "f64"), ("to_uint", "u32"), ("to_int", "i32")] {
        let pat = format!(r"([\w.()]+)\.{}\(\)", method);
        if let Ok(re) = regex::Regex::new(&pat) {
            let new = re
                .replace_all(content.as_str(), |caps: &regex::Captures| {
                    let recv = caps.get(1).unwrap().as_str();
                    if recv.starts_with('(') && recv.ends_with(')') {
                        format!("{} as {})", &recv[..recv.len() - 1], cast)
                    } else {
                        format!("({} as {})", recv, cast)
                    }
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }
    }
}
