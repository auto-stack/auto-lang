use crate::ast::*;
use crate::database::Database;
use crate::error::{pos_to_span, AutoError, AutoResult, NameError, SyntaxError, Warning};
use crate::infer::{check_field_type, InferenceContext};
use crate::lexer::Lexer;
use crate::parser_helpers::{LambdaIdGenerator, ModuleTracker};
use crate::scope::Meta;
use crate::scope_manager::ScopeManager;
use crate::symbols::SymbolLocation;
use crate::token::{Pos, Token, TokenKind};
use crate::types;
use auto_val::AutoStr;
use auto_val::Op;
use auto_val::{shared, Shared};
use miette::SourceSpan;
use std::collections::HashMap;
use std::i32;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

/// TODO: T should be a generic AST node type
pub trait ParserExt {
    fn parse(input: impl Into<AutoStr>) -> AutoResult<Self>
    where
        Self: Sized;
}

pub struct PostfixPrec {
    l: u8,
    _r: (),
}

pub struct InfixPrec {
    l: u8,
    r: u8,
}

pub struct PrefixPrec {
    _l: (),
    r: u8,
}

const fn prefix_prec(n: u8) -> PrefixPrec {
    PrefixPrec { _l: (), r: 2 * n }
}

const fn postfix_prec(n: u8) -> PostfixPrec {
    PostfixPrec { l: 2 * n, _r: () }
}

const fn infix_prec(n: u8) -> InfixPrec {
    if n == 0 {
        InfixPrec { l: 0, r: 0 }
    } else {
        InfixPrec {
            l: 2 * n - 1,
            r: 2 * n,
        }
    }
}

const PREC_ASN: InfixPrec = infix_prec(1);
const PREC_ADDEQ: InfixPrec = infix_prec(2);
const PREC_MULEQ: InfixPrec = infix_prec(3);
const PREC_PAIR: PostfixPrec = postfix_prec(4);
const PREC_OR: InfixPrec = infix_prec(5); // Logical or (Plan 072)
const PREC_AND: InfixPrec = infix_prec(6); // Logical and (Plan 072)
const PREC_EQ: InfixPrec = infix_prec(7);
const PREC_CMP: InfixPrec = infix_prec(8);
const PREC_RANGE: InfixPrec = infix_prec(9);
const PREC_ADD: InfixPrec = infix_prec(10);
const PREC_MUL: InfixPrec = infix_prec(11);
const PREC_NULLCOALESCE: InfixPrec = infix_prec(5); // ?? operator (same as OR)

const _PREC_REF: PrefixPrec = prefix_prec(12);
const PREC_SIGN: PrefixPrec = prefix_prec(13);
const PREC_NOT: PrefixPrec = prefix_prec(14);
const PREC_CALL: PostfixPrec = postfix_prec(15);
const PREC_INDEX: PostfixPrec = postfix_prec(16);
const PREC_POSTFIX: PostfixPrec = postfix_prec(17); // Bang operator (!)
const PREC_DOT: InfixPrec = infix_prec(18);
const _PREC_ATOM: InfixPrec = infix_prec(18);

fn prefix_power(op: Op, span: SourceSpan) -> AutoResult<PrefixPrec> {
    match op {
        Op::Add | Op::Sub => Ok(PREC_SIGN),
        Op::Not => Ok(PREC_NOT),
        // Plan 379: unary `*` (dereference) as a prefix operator. The transpiler
        // already maps `Op::Mul` → `*` in Expr::Unary (Plan 052); only the parser
        // rejected `*x` in prefix position before.
        Op::Mul => Ok(PREC_SIGN),
        _ => Err(SyntaxError::Generic {
            message: format!("Invalid prefix operator: {}", op),
            span,
        }
        .into()),
    }
}

fn postfix_power(op: Op) -> AutoResult<Option<PostfixPrec>> {
    match op {
        Op::LSquare => Ok(Some(PREC_INDEX)),
        Op::LParen => Ok(Some(PREC_CALL)),
        Op::Colon => Ok(Some(PREC_PAIR)),
        Op::Not => Ok(Some(PREC_POSTFIX)), // Bang operator (!) for eager collection
        _ => Ok(None),
    }
}

/// Helper function to capitalize first letter for backwards compatibility
/// Converts "int" -> "Int", "string" -> "String", etc.
#[allow(dead_code)]
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
fn infix_power(op: Op, span: SourceSpan) -> AutoResult<InfixPrec> {
    match op {
        Op::Add | Op::Sub => Ok(PREC_ADD),
        Op::Mul | Op::Div | Op::Mod => Ok(PREC_MUL),
        Op::AddEq | Op::SubEq => Ok(PREC_ADDEQ),
        Op::MulEq | Op::DivEq | Op::ModEq => Ok(PREC_MULEQ),
        Op::Asn => Ok(PREC_ASN),
        Op::Eq | Op::Neq => Ok(PREC_EQ),
        Op::Lt | Op::Gt | Op::Le | Op::Ge => Ok(PREC_CMP),
        Op::Range | Op::RangeEq => Ok(PREC_RANGE),
        Op::Dot => Ok(PREC_DOT),
        // Property keywords (Phase 3): same precedence as dot
        // Error propagation (Phase 1b..3): ?. same precedence as dot
        // Plan 120: .? same precedence as dot
        Op::DotView | Op::DotMut | Op::DotMove | Op::DotTake | Op::DotQuestion | Op::DotQuest => {
            Ok(PREC_DOT)
        }
        // May type operators (Phase 1b.3)
        Op::QuestionQuestion => Ok(PREC_NULLCOALESCE),
        // Logical operators (Plan 072)
        Op::Or => Ok(PREC_OR),
        Op::And => Ok(PREC_AND),
        _ => Err(SyntaxError::Generic {
            message: format!("Invalid infix operator: {}", op),
            span,
        }
        .into()),
    }
}

// pub fn parse(code: &str, scope: Rc<RefCell<Universe>>, interpreter: &'a Interpreter) -> AutoResult<Code> {
// let mut parser = Parser::new(code, scope, interpreter);
// parser.parse()
// }

#[derive(Debug, Clone, PartialEq)]
pub enum CompileDest {
    Interp,    // for interperter
    TransC,    // for tranpiler to C
    TransRust, // for tranpiler to Rust
    Config,    // for pac.at manifest config evaluation (Plan 364)
}

/// Plan 260: Parsed function annotations returned by parse_fn_annotations()
#[derive(Default)]
struct FnAnnotations {
    has_c: bool,
    has_vm: bool,
    has_rs: bool,
    has_pub: bool,
    has_test: bool,
    /// Plan 306 Phase 3: GDScript variable annotations (`#[export]`, `#[onready]`,
    /// `#[export_range(0,100)]`, …) captured as full text to emit verbatim after `@`.
    store_attrs: Vec<AutoStr>,
    with_params: Vec<crate::ast::TypeParam>,
    /// Plan 312: #[api(method = "GET", path = "/api/notes")] — HTTP endpoint.
    /// Parsed into ApiAttrs; propagated to Fn.api_attrs by the caller.
    api_endpoint: Option<crate::ast::ApiAttrs>,
    /// Plan 610 ⑤: `#[export]` / `#[export(system)]` — cdylib export face
    /// (a2r sibling-wrapper emission). Some("C") for the bare form. Shared
    /// with GDScript var annotations (`#[export] var x` → store_attrs); fn
    /// paths read this field, var paths read store_attrs — no interference.
    export_abi: Option<AutoStr>,
}

/// Plan 419 §8.3:块嵌套深度护栏(防真实病态递归把分段栈无限吃穿;
/// 正常程序 < 50。debug 构建每层嵌套 ~670KB 巨帧,256 层上限把
/// 最坏分段栈提交约束在 ~170MB 量级)。
const MAX_BODY_NESTING: u32 = 256;

/// PLAN-801：Parser 实例身份计数器（端口快照凭据的跨上下文拒绝判据）。
fn next_port_instance_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

pub struct Parser<'a> {
    pub scope: ScopeManager,
    /// Plan 419 §8.3:parse_body 嵌套深度(护栏计数)。
    body_depth: u32,
    /// Plan 091: Optional Database for incremental compilation
    /// When set, Database methods are used instead of Universe methods
    pub db: Option<Arc<RwLock<Database>>>,
    lexer: Lexer<'a>,
    pub cur: Token,
    prev: Token, // Track previous token for validation
    /// Plan 351: true when parsing inside an on-block handler body.
    /// Prevents `store` (contextual keyword) from being misinterpreted as a
    /// store declaration when it's actually a variable reference (store.action()).
    pub(crate) in_on_body: bool,
    pub(crate) in_fn_body: bool,
    /// Plan 567 T12（P560-D1）: with 头部窗口旗标——置位期间 `as` Cast 臂
    /// 截断（`with e as x` 的绑定位由 with_stmt 消费，不被 parse_expr
    /// 整吞为 Cast）。仅语句头部窗口生效，正常模式 Cast 语义零变化。
    with_header: bool,
    /// Plan 043 M5: true while parsing the RHS of a Dot expression
    /// (`obj.field`). The struct-literal widening in atom() (any PascalCase
    /// ident + `{` → construction) must NOT fire here — `text cell.Text { }`
    /// would otherwise parse `Text{` as a Node and swallow the element's
    /// props brace ("Expected term, got RBrace"). Struct literals are always
    /// standalone `Type{...}`, never `obj.Field{...}`.
    in_dot_rhs: bool,
    /// 方言表三段（PLAN-778）：pre = `DialectPriority::Override` 前插的外部
    /// 方言；builtin = `with_session` 按 session 装配的内建方言（UiDialect 等，
    /// 随 with_session 重建）；post = `Append` 后接的外部方言。派发序恒为
    /// pre → builtin → post，外部注册跨 `with_session` 保留。
    /// R1（F778-R1-01）：元素为 Arc 共享句柄——派发时克隆 Arc 调用而非移出，
    /// 活跃方言在自身委托核心解析（如闭包体内嵌语句再派发）期间仍留在表内，
    /// 合法递归派发可见完整方言表。
    dialects_pre: Vec<std::sync::Arc<dyn crate::dialect::Dialect>>,
    dialects_builtin: Vec<std::sync::Arc<dyn crate::dialect::Dialect>>,
    dialects_post: Vec<std::sync::Arc<dyn crate::dialect::Dialect>>,
    /// PLAN-801 端口快照句柄表：`ParseContext::checkpoint` 的有作用域后端
    ///（T-01 D3；F-801-R1-01 修订）。槽位 tombstone 复用，签发 gen 在本
    /// Parser 内全局单调不重复；**每个槽位记录签发它的方言调用作用域**
    ///（`port_call_scope`），派发器在每次方言调用退出时按归属失效该调用
    /// 签发的快照（`port_release_scope`），嵌套调用作用域保存/恢复——
    /// 外层存活快照不受内层退出影响，内层退出凭据必失效，与槽位复用
    /// 与否无关。
    port_snapshots: Vec<crate::parser_bridge::PortSnapshotSlot<'a>>,
    /// 本 Parser 实例身份（端口凭据跨上下文拒绝的判据；全局计数器签发）。
    port_instance_id: u64,
    /// 端口凭据下一次签发代次（单调递增，永不重复→槽位复用无误接受）。
    port_gen_next: u64,
    /// 当前方言调用作用域（派发器每次调用前递增、退出后恢复；checkpoint
    /// 以此标记槽位归属）。
    port_call_scope: u64,
    pub skip_check: bool,
    pub compile_dest: CompileDest,
    /// Error recovery: collected errors during parsing
    pub errors: Vec<AutoError>,
    /// Plan 122: Collected warnings during parsing
    pub warnings: Vec<Warning>,
    /// Plan 312: Pending #[api] attributes from the most recent parse_fn_annotations
    /// call, consumed by fn_decl_stmt_with_annotations when building the Fn AST.
    pending_api_attrs: Option<crate::ast::ApiAttrs>,
    /// Plan 610 ⑤: pending `#[export]` ABI from the most recent
    /// parse_fn_annotations call, consumed by fn_decl_stmt_with_annotations
    /// (same pending flow as pending_api_attrs).
    pending_export_abi: Option<AutoStr>,
    /// Plan 395: Explicit generic type args (`<T1, T2>`) parsed right before a
    /// call's `(` in expr_pratt_with_left. The `<`-intercept stores them here and
    /// the very next loop iteration's LParen arm mounts them onto the Call.
    /// Sequential-only: the intercept REQUIRES `(` to immediately follow
    /// `<types>`, so no nesting can interleave a stale pending value.
    pending_generic_args: Vec<Type>,
    /// Maximum number of errors to collect before aborting
    pub error_limit: usize,
    /// Current type parameters being parsed (for generic type definitions)
    current_type_params: Vec<Name>,
    /// Current const generic parameters being parsed (Plan 052)
    /// Maps const parameter name to its type (e.g., N -> u32)
    current_const_params: HashMap<Name, Type>,
    /// Type inference context (Plan 010 Phase 5: Integration)
    pub infer_ctx: InferenceContext,
    /// Type registry for REPL (Plan 087)
    /// Allows checking if an identifier is a type across REPL inputs
    pub type_registry: Option<crate::type_registry::SharedTypeRegistry>,
    /// Plan 084: 统一类型存储
    pub type_store: Arc<RwLock<types::TypeStore>>,
    /// Plan 090: 模块路径追踪器
    pub module_tracker: ModuleTracker,
    /// Plan 090: Lambda ID 生成器
    pub lambda_id_gen: LambdaIdGenerator,
    /// Plan 096: Compiler session for scenario-based parsing
    pub session: crate::session::CompilerSession,
    /// Plan 159 Phase 6B-2: Collected raw attribute strings for derive/serde passthrough
    raw_attrs: Vec<AutoStr>,
    /// Plan 364 W1: Dotted attribute-macro paths (`#[zbus.interface]` →
    /// `#[zbus::interface]`) collected for the following decl's **impl block**
    /// (fn-level `#[tokio.main]` included). Separate from `raw_attrs` so
    /// struct-level derives and impl-level macros don't mix.
    impl_attrs: Vec<AutoStr>,
    /// Plan 306 Phase 3: Script-level GDScript annotations (`tool`, `icon(...)`)
    /// collected from top-level `#[tool]`/`#[icon]`; drained into Code.file_attrs.
    file_attrs: Vec<AutoStr>,
    /// Collected doc comment lines (///) to attach to the next declaration
    pending_docs: Vec<AutoStr>,
    /// Track imported module names from `use` statements for check_symbol
    use_imports: Vec<AutoStr>,
    /// Plan 369 P4: Track item names imported via `use.py module: a, b` so the
    /// symbol checker treats bare references to py-imported constants/functions
    /// (e.g. `NAMESPACE_DNS`, `pi`, `uuid5`) as defined. Without this, py
    /// constants only resolve in method-call position (`pi.to(str)`) and fail
    /// with "undefined variable" in argument or assignment position
    /// (`uuid5(NAMESPACE_DNS, ...)`). See Task 13.
    py_item_imports: Vec<AutoStr>,
    /// Plan 550 T10: 文件级 `#[script]` pragma——脚本模式生产者门控标注。
    /// 仅登记（→ CompileSession.script_marked），不改变任何编译/运行
    /// 行为；.as 扩展名与模式管线归 W1。
    pub script_pragma: bool,
    /// Plan 555 T02: 文件级 `#[rust]` pragma——脚本模式的显式压回通道
    /// （.as 文件标注 #[rust] 即回正常模式；设计 §2 pragma 覆盖）。
    /// 优先序：#[rust] > #[script] > 扩展名（见 mode.rs resolve_script_mode）。
    pub rust_pragma: bool,
    /// Plan 550 T10: 文件含裸 null/nil 字面量（生产者门控三信号之一；
    /// None/Some 是 Option 构造器，不计入）。
    pub saw_bare_null: bool,
}

impl<'a> Parser<'a> {
    /// Create parser (no Universe needed)
    /// Prefer this for new code.
    pub fn from(code: &'a str) -> Self {
        Self::new(code)
    }

    pub fn new(code: &'a str) -> Self {
        let mut lexer = Lexer::new(code);
        let cur = lexer.next().expect("lexer should produce first token");
        let mut parser = Parser {
            scope: ScopeManager::new(),
            body_depth: 0,
            db: None, // Plan 091: Optional Database
            lexer,
            cur,
            prev: Token {
                kind: TokenKind::EOF,
                pos: Pos {
                    line: 0,
                    at: 0,
                    pos: 0,
                    len: 0,
                },
                text: "".into(),
            }, // Initialize with EOF token
            in_on_body: false,
            in_fn_body: false,
            in_dot_rhs: false,
            with_header: false,
            compile_dest: CompileDest::Interp,
            dialects_pre: Vec::new(),
            dialects_builtin: Vec::new(),
            dialects_post: Vec::new(),
            port_snapshots: Vec::new(),
            port_instance_id: next_port_instance_id(),
            port_gen_next: 0,
            port_call_scope: 0,
            skip_check: false,
            errors: Vec::new(),
            warnings: Vec::new(),             // Plan 122: Warnings collection
            pending_api_attrs: None,          // Plan 312
            pending_export_abi: None,         // Plan 610
            pending_generic_args: Vec::new(), // Plan 395
            error_limit: crate::get_error_limit(), // Use global error limit
            current_type_params: Vec::new(),
            current_const_params: HashMap::new(),
            infer_ctx: InferenceContext::new(), // Plan 010 Phase 5: Initialize inference context
            type_registry: None,                // Plan 087: Type registry for REPL
            type_store: Arc::new(RwLock::new(types::TypeStore::new())), // Plan 084
            module_tracker: ModuleTracker::new(), // Plan 090
            lambda_id_gen: LambdaIdGenerator::new(), // Plan 090
            session: crate::session::CompilerSession::default(), // Plan 096: Default session
            raw_attrs: Vec::new(),              // Plan 159 Phase 6B-2
            impl_attrs: Vec::new(),             // Plan 364 W1
            file_attrs: Vec::new(),             // Plan 306 Phase 3
            pending_docs: Vec::new(),
            use_imports: Vec::new(),
            py_item_imports: Vec::new(),
            script_pragma: false, // Plan 550 T10
            rust_pragma: false,   // Plan 555 T02
            saw_bare_null: false, // Plan 550 T10
        };
        parser.skip_comments();
        parser
    }

    pub fn set_dest(&mut self, dest: CompileDest) {
        self.compile_dest = dest;
    }

    /// Set type registry for REPL (Plan 087)
    ///
    /// This allows the parser to check if an identifier is a type
    /// across REPL inputs, enabling node instance syntax like `Point{x:1, y:2}`.
    pub fn set_type_registry(&mut self, registry: crate::type_registry::SharedTypeRegistry) {
        self.type_registry = Some(registry);
    }

    /// Set Database for incremental compilation (Plan 091)
    ///
    /// When set, Parser will use Database methods instead of Universe methods
    /// for symbol and type storage.
    pub fn set_database(&mut self, db: Arc<RwLock<Database>>) {
        self.db = Some(db);
    }

    pub fn new_with_note(code: &'a str, note: char) -> Self {
        let mut lexer = Lexer::new(code);
        lexer.set_fstr_note(note);
        let cur = lexer.next().expect("lexer should produce first token");
        let mut parser = Parser {
            scope: ScopeManager::new(),
            body_depth: 0,
            db: None, // Plan 091: Optional Database
            lexer,
            cur,
            prev: Token {
                kind: TokenKind::EOF,
                pos: Pos {
                    line: 0,
                    at: 0,
                    pos: 0,
                    len: 0,
                },
                text: "".into(),
            }, // Initialize with EOF token
            in_on_body: false,
            in_fn_body: false,
            in_dot_rhs: false,
            with_header: false,
            compile_dest: CompileDest::Interp,
            dialects_pre: Vec::new(),
            dialects_builtin: Vec::new(),
            dialects_post: Vec::new(),
            port_snapshots: Vec::new(),
            port_instance_id: next_port_instance_id(),
            port_gen_next: 0,
            port_call_scope: 0,
            skip_check: false,
            errors: Vec::new(),
            warnings: Vec::new(),             // Plan 122: Warnings collection
            pending_api_attrs: None,          // Plan 312
            pending_export_abi: None,         // Plan 610
            pending_generic_args: Vec::new(), // Plan 395
            error_limit: crate::get_error_limit(), // Use global error limit
            current_type_params: Vec::new(),
            current_const_params: HashMap::new(),
            infer_ctx: InferenceContext::new(), // Plan 010 Phase 5: Initialize inference context
            type_registry: None,                // Plan 087: Type registry for REPL
            type_store: Arc::new(RwLock::new(types::TypeStore::new())), // Plan 084
            module_tracker: ModuleTracker::new(), // Plan 090
            lambda_id_gen: LambdaIdGenerator::new(), // Plan 090
            session: crate::session::CompilerSession::default(), // Plan 096: Default session
            raw_attrs: Vec::new(),              // Plan 159 Phase 6B-2
            impl_attrs: Vec::new(),             // Plan 364 W1
            file_attrs: Vec::new(),             // Plan 306 Phase 3
            pending_docs: Vec::new(),
            use_imports: Vec::new(),
            py_item_imports: Vec::new(),
            script_pragma: false, // Plan 550 T10
            rust_pragma: false,   // Plan 555 T02
            saw_bare_null: false, // Plan 550 T10
        };
        parser.skip_comments();
        parser
    }

    /// Create a new parser with a pre-lexed first token
    pub fn new_with_note_and_first_token(
        _code: &'a str,
        _note: char,
        first_token: Token,
        lexer: Lexer<'a>,
    ) -> Self {
        let mut parser = Parser {
            scope: ScopeManager::new(),
            body_depth: 0,
            db: None, // Plan 091: Optional Database
            lexer,
            cur: first_token,
            prev: Token {
                kind: TokenKind::EOF,
                pos: Pos {
                    line: 0,
                    at: 0,
                    pos: 0,
                    len: 0,
                },
                text: "".into(),
            }, // Initialize with EOF token
            in_on_body: false,
            in_fn_body: false,
            in_dot_rhs: false,
            with_header: false,
            compile_dest: CompileDest::Interp,
            dialects_pre: Vec::new(),
            dialects_builtin: Vec::new(),
            dialects_post: Vec::new(),
            port_snapshots: Vec::new(),
            port_instance_id: next_port_instance_id(),
            port_gen_next: 0,
            port_call_scope: 0,
            skip_check: false,
            errors: Vec::new(),
            warnings: Vec::new(),             // Plan 122: Warnings collection
            pending_api_attrs: None,          // Plan 312
            pending_export_abi: None,         // Plan 610
            pending_generic_args: Vec::new(), // Plan 395
            error_limit: crate::get_error_limit(), // Use global error limit
            current_type_params: Vec::new(),
            current_const_params: HashMap::new(),
            infer_ctx: InferenceContext::new(), // Plan 010 Phase 5: Initialize inference context
            type_registry: None,                // Plan 087: Type registry for REPL
            type_store: Arc::new(RwLock::new(types::TypeStore::new())), // Plan 084
            module_tracker: ModuleTracker::new(), // Plan 090
            lambda_id_gen: LambdaIdGenerator::new(), // Plan 090
            session: crate::session::CompilerSession::default(), // Plan 096: Default session
            raw_attrs: Vec::new(),              // Plan 159 Phase 6B-2
            impl_attrs: Vec::new(),             // Plan 364 W1
            file_attrs: Vec::new(),             // Plan 306 Phase 3
            pending_docs: Vec::new(),
            use_imports: Vec::new(),
            py_item_imports: Vec::new(),
            script_pragma: false, // Plan 550 T10
            rust_pragma: false,   // Plan 555 T02
            saw_bare_null: false, // Plan 550 T10
        };
        parser.skip_comments();
        parser
    }

    pub fn skip_check(mut self) -> Self {
        self.skip_check = true;
        self
    }

    /// Set the compiler session (Plan 096)
    ///
    /// This enables scenario-based parsing where keywords like
    /// `widget`, `view`, `model`, `msg`, `on` are only keywords
    /// in UI scenario.
    ///
    /// PLAN-778: 只重建内建段；`register_dialect`/`with_dialect` 注册的
    /// 外部方言（pre/post 段）跨 with_session 保留（V-03 传播不变量）。
    pub fn with_session(mut self, session: crate::session::CompilerSession) -> Self {
        self.session = session;
        self.dialects_builtin = Self::build_dialects(&self.session);
        self
    }

    /// 显式注册外部方言（PLAN-778）。
    ///
    /// - `DialectPriority::Override`：前插 pre 段，先于内建方言查询——外部
    ///   显式覆盖内建同名关键字处理，内建退为 Ok(None) 后备。
    /// - `DialectPriority::Append`：后接 post 段，仅在内建方言全部 Ok(None)
    ///   后可见。
    pub fn register_dialect(
        &mut self,
        dialect: Box<dyn crate::dialect::Dialect>,
        priority: crate::dialect::DialectPriority,
    ) {
        // R1：内部存储 Arc 共享句柄（派发期间活跃方言保持在表）。
        let dialect: std::sync::Arc<dyn crate::dialect::Dialect> = dialect.into();
        match priority {
            crate::dialect::DialectPriority::Override => self.dialects_pre.push(dialect),
            crate::dialect::DialectPriority::Append => self.dialects_post.push(dialect),
        }
    }

    /// `register_dialect` 的 builder 形态（PLAN-778）。
    pub fn with_dialect(
        mut self,
        dialect: Box<dyn crate::dialect::Dialect>,
        priority: crate::dialect::DialectPriority,
    ) -> Self {
        self.register_dialect(dialect, priority);
        self
    }

    /// PLAN-801：注册新接口外部方言（`auto_parser_api::ExternalDialect`，
    /// 只依赖轻端口 crate 的实现方）。经 `ExternalDialectAdapter` 协议转换
    /// 进入与旧 trait **同一张**三段派发表：Override 前插 pre 段 /
    /// Append 后接 post 段，派发序与恢复不变量（Arc 注册永不出表 /
    /// None 强制回滚 / Err 边界恢复 context）与 `register_dialect` 完全
    /// 一致；注册跨 `with_session` 保留。
    pub fn register_external_dialect(
        &mut self,
        dialect: Box<dyn crate::dialect::ExternalDialect>,
        priority: crate::dialect::DialectPriority,
    ) {
        let adapter: std::sync::Arc<dyn crate::dialect::Dialect> =
            std::sync::Arc::new(crate::dialect::external::ExternalDialectAdapter {
                inner: dialect.into(),
            });
        match priority {
            crate::dialect::DialectPriority::Override => self.dialects_pre.push(adapter),
            crate::dialect::DialectPriority::Append => self.dialects_post.push(adapter),
        }
    }

    /// `register_external_dialect` 的 builder 形态（PLAN-801）。
    pub fn with_external_dialect(
        mut self,
        dialect: Box<dyn crate::dialect::ExternalDialect>,
        priority: crate::dialect::DialectPriority,
    ) -> Self {
        self.register_external_dialect(dialect, priority);
        self
    }

    // ---- PLAN-801 端口快照句柄表后端（ParseContext::checkpoint/restore） ----

    /// 签发一张端口快照凭据：快照数据（游标+五旗标）入表，凭据=（实例
    /// id, 槽位, 单调 gen），槽位标记**当前方言调用作用域**。tombstone
    /// 槽位复用（复用槽位换新 gen，旧凭据自然失效）；无 unsafe、无
    /// 'static、无全局池（T-01 D3）。调用退出按作用域失效由派发器
    /// `port_release_scope` 负责。
    /// PLAN-806 端口原语（F806-R2-01 引入/R3-01 定形）：原始单 token 探测
    /// ——读下一个原始 token，成功则立即回推（零副作用/幂等）；词法失败
    /// 遵循旧回推前瞻的失败消费协议（是否推进输入由 Lexer 失败模式决定，
    /// 错误不记录返回 None——与旧 `lexer.next()`+`push_token` 逐字同构，
    /// 失败路径不承诺幂等）。见 auto-parser-api
    /// `ParseContext::probe_next_kind`。
    pub(crate) fn port_probe_next_kind(&mut self) -> Option<TokenKind> {
        match self.lexer.next() {
            Ok(tok) => {
                let kind = tok.kind;
                self.lexer.push_token(tok);
                Some(kind)
            }
            Err(_) => None,
        }
    }

    pub(crate) fn port_checkpoint(&mut self) -> auto_parser_api::ParseCheckpoint {
        let gen = self.port_gen_next;
        self.port_gen_next += 1;
        let slot = match self.port_snapshots.iter().position(|s| s.data.is_none()) {
            Some(i) => i,
            None => {
                self.port_snapshots
                    .push(crate::parser_bridge::PortSnapshotSlot {
                        scope: self.port_call_scope,
                        gen,
                        data: None,
                    });
                self.port_snapshots.len() - 1
            }
        };
        let data = self.bridge_checkpoint();
        self.port_snapshots[slot] = crate::parser_bridge::PortSnapshotSlot {
            scope: self.port_call_scope,
            gen,
            data: Some(data),
        };
        auto_parser_api::ParseCheckpoint::new(self.port_instance_id, slot as u32, gen)
    }

    /// 失效指定方言调用作用域签发的全部快照（调用退出清理，
    /// F-801-R1-01：按签发归属而非表长——tombstone 复用可使本轮新快照
    /// 落在调用入口表长之下，长度截断清不到它们）。仅置 tombstone（gen
    /// 保留），未兑现凭据在 `port_restore` 处被存活位拒绝且游标不动；
    /// 其他作用域（含外层存活快照）不受影响。尾部 tombstone 保守收缩
    /// （只剥尾，不动中部，槽位下标不漂移）。
    pub(crate) fn port_release_scope(&mut self, scope: u64) {
        for slot in &mut self.port_snapshots {
            if slot.scope == scope {
                slot.data = None;
            }
        }
        while matches!(self.port_snapshots.last(), Some(s) if s.data.is_none()) {
            self.port_snapshots.pop();
        }
    }

    /// 兑现端口快照凭据：合法（本实例 + 槽位在表 + gen 匹配 + 未消费）则
    /// 恢复游标与五旗标（诊断单调追加），凭据一次性；否则**受控拒绝**——
    /// 返回当前 token 位置的语法错误（跨上下文/过期/重复使用）。
    pub(crate) fn port_restore(&mut self, cp: auto_parser_api::ParseCheckpoint) -> AutoResult<()> {
        let (parser_id, slot, gen) = cp.parts();
        let rejected = |cur: &Token| -> AutoError {
            crate::error::SyntaxError::Generic {
                message: "dialect checkpoint rejected: credential from another parse context \
                          or already consumed/expired"
                    .to_string(),
                span: pos_to_span(cur.pos),
            }
            .into()
        };
        if parser_id != self.port_instance_id {
            return Err(rejected(&self.cur));
        }
        let idx = slot as usize;
        let valid = idx < self.port_snapshots.len()
            && self.port_snapshots[idx].gen == gen
            && self.port_snapshots[idx].data.is_some();
        if !valid {
            return Err(rejected(&self.cur));
        }
        let data = self.port_snapshots[idx]
            .data
            .take()
            .expect("gen-matched live slot carries snapshot data");
        self.bridge_restore(data);
        Ok(())
    }

    // ---- PLAN-841：DomainParseContext 中立端口后端 ----

    /// 原始游标投机的回滚：与 `port_restore` 同凭据校验/一次性消费，但
    /// **只**恢复 lexer + cur/prev——不恢复五旗标、不回滚诊断（与旧臂
    /// widget 近失/computed/style recipe 投机位点的手工 save/restore 逐字
    /// 等价：中间 `next()` 的 docs 收集与词法错误记录保留）。
    pub(crate) fn port_restore_raw(
        &mut self,
        cp: auto_parser_api::ParseCheckpoint,
    ) -> AutoResult<()> {
        let (parser_id, slot, gen) = cp.parts();
        let rejected = |cur: &Token| -> AutoError {
            crate::error::SyntaxError::Generic {
                message: "dialect checkpoint rejected: credential from another parse context \
                          or already consumed/expired"
                    .to_string(),
                span: pos_to_span(cur.pos),
            }
            .into()
        };
        if parser_id != self.port_instance_id {
            return Err(rejected(&self.cur));
        }
        let idx = slot as usize;
        let valid = idx < self.port_snapshots.len()
            && self.port_snapshots[idx].gen == gen
            && self.port_snapshots[idx].data.is_some();
        if !valid {
            return Err(rejected(&self.cur));
        }
        let data = self.port_snapshots[idx]
            .data
            .take()
            .expect("gen-matched live slot carries snapshot data");
        self.lexer.restore_state(data.lexer);
        self.cur = data.cur;
        self.prev = data.prev;
        Ok(())
    }

    /// 原始单 token 前看（probe 全 token 形态）：词法成功即回推（零副作用、
    /// 幂等）；失败原样上抛（失败推进由 Lexer 失败模式决定——806 冻结的
    /// 失败消费协议，错误是否丢弃由调用方决定 `?`/`.ok()`）。
    pub(crate) fn port_probe_next_token(&mut self) -> AutoResult<Token> {
        let tok = self.lexer.next()?;
        self.lexer.push_token(tok.clone());
        Ok(tok)
    }

    /// 回推一个 owned token（双 token 回推再 next() 重同步的旧臂模式）。
    pub(crate) fn port_push_back_token(&mut self, tok: Token) {
        self.lexer.push_token(tok);
    }

    /// 注释跳过前看（PLAN-810；**on stream 位点形态**——F810-R1-04 区分）：
    /// 完整 advance（全部副作用），观察 kind，回推观察到的 token 并恢复
    /// cur；prev 保持 advance 后的值（旧臂手动恢复形态的泄漏）。
    pub(crate) fn port_advance_peek_kind(&mut self) -> TokenKind {
        let saved = self.cur.clone();
        self.next();
        let kind = self.cur.kind;
        let observed = self.cur.clone();
        self.lexer.push_token(observed);
        self.cur = saved;
        kind
    }

    /// 注释跳过前看·双 push 重同步（PLAN-810；**scene 门/typed
    /// subresource 位点形态**）：next → 观察 → push observed → push saved
    /// → next 重同步。与 `port_advance_peek_kind` 仅 **prev** 终值不同：
    /// 第二次 next 使 prev=被观察 token（旧臂两 push 形态逐字复刻，
    /// F810-R1-04）。
    pub(crate) fn port_advance_peek_kind_resync(&mut self) -> TokenKind {
        let saved = self.cur.clone();
        self.next();
        let kind = self.cur.kind;
        let observed = self.cur.clone();
        self.lexer.push_token(observed);
        self.lexer.push_token(saved);
        self.next();
        kind
    }

    /// 释放 raw 投机快照凭据（F810-R1-01）：仅按（实例 id + 槽位 + 代次）
    /// 校验后 tombstone 该槽，不动游标/旗标/诊断；失效凭据为无操作
    ///（Drop 语义）。尾部 tombstone 保守收缩（同 `port_release_scope`，
    /// 只剥尾不动中部，槽位下标不漂移；tombstone 本身可被复用）。
    pub(crate) fn port_release_raw(&mut self, cp: auto_parser_api::ParseCheckpoint) {
        let (parser_id, slot, gen) = cp.parts();
        if parser_id != self.port_instance_id {
            return;
        }
        let idx = slot as usize;
        if idx < self.port_snapshots.len()
            && self.port_snapshots[idx].gen == gen
            && self.port_snapshots[idx].data.is_some()
        {
            self.port_snapshots[idx].data = None;
        }
        while matches!(self.port_snapshots.last(), Some(s) if s.data.is_none()) {
            self.port_snapshots.pop();
        }
    }

    /// raw CSS 块捕获（前置条件：cur 为 LBrace 且 lexer 无缓冲 token）。
    pub(crate) fn port_capture_raw_block(&mut self) -> AutoResult<String> {
        Ok(self.lexer.capture_raw_block()?)
    }

    pub(crate) fn port_prev_pos(&self) -> Pos {
        self.prev.pos
    }

    pub(crate) fn port_set_skip_check(&mut self, value: bool) -> bool {
        std::mem::replace(&mut self.skip_check, value)
    }

    pub(crate) fn port_set_in_on_body(&mut self, value: bool) -> bool {
        std::mem::replace(&mut self.in_on_body, value)
    }

    pub(crate) fn port_diagnostics_mark(&self) -> (usize, usize) {
        (self.errors.len(), self.warnings.len())
    }

    /// 局部诊断事务回退（**仅** raw 投机回退位点：computed object→body）。
    pub(crate) fn port_diagnostics_truncate(&mut self, mark: (usize, usize)) {
        self.errors.truncate(mark.0);
        self.warnings.truncate(mark.1);
    }

    /// on 处理器参数符号定义（与旧臂同构的 Meta::Store(Var) 构造）。
    pub(crate) fn port_define_handler_param(&mut self, name: &str) {
        let meta = crate::scope::Meta::Store(crate::ast::Store {
            kind: crate::ast::StoreKind::Var,
            attrs: vec![],
            is_pub: false,
            name: name.into(),
            expr: crate::ast::Expr::Nil,
            ty: crate::ast::Type::Unknown,
        });
        self.define(name, meta);
    }

    pub(crate) fn port_warn_suspicious_block_keyword(&mut self, found: String, suggestion: String) {
        let span = pos_to_span(self.cur.pos);
        self.warn(crate::error::Warning::SuspiciousBlockKeyword {
            found,
            suggestion,
            span,
        });
    }

    /// 按 session 装配内建方言表。集中所有内建方言注册，便于一眼看清
    /// "当前有哪些内建方言"（外部方言经 register_dialect 显式注册，不在此）。
    fn build_dialects(
        _session: &crate::session::CompilerSession,
    ) -> Vec<std::sync::Arc<dyn crate::dialect::Dialect>> {
        Vec::new()
    }

    // ---- PLAN-778 方言派发（三段，恢复不变量核心保证） ----

    /// PLAN-814 R3-03b：方言表只读观测口（段长；0=pre/1=builtin/2=post）。
    /// 方言注册是公开特性（PLAN-778），外部消费者与随迁回归测试经此观测
    /// 注册结果，不暴露内部表结构。
    pub fn dialect_section_len(&self, section: u8) -> usize {
        self.dialect_section(section).len()
    }

    fn dialect_section(&self, section: u8) -> &Vec<std::sync::Arc<dyn crate::dialect::Dialect>> {
        match section {
            0 => &self.dialects_pre,
            1 => &self.dialects_builtin,
            _ => &self.dialects_post,
        }
    }

    /// 游标+上下文 checkpoint（PLAN-778；R1 F778-R1-02 扩展）：lexer 全状态
    /// （Plan 395 LexerState）+ cur/prev token 快照 + 语句级 context 旗标
    /// （in_on_body/in_fn_body/in_dot_rhs/with_header/body_depth）。
    /// 供 ParserBridge 与方言派发器共用。
    pub(crate) fn bridge_checkpoint(&self) -> crate::parser_bridge::BridgeCheckpoint<'a> {
        crate::parser_bridge::BridgeCheckpoint {
            lexer: self.lexer.save_state(),
            cur: self.cur.clone(),
            prev: self.prev.clone(),
            in_on_body: self.in_on_body,
            in_fn_body: self.in_fn_body,
            in_dot_rhs: self.in_dot_rhs,
            with_header: self.with_header,
            body_depth: self.body_depth,
        }
    }

    pub(crate) fn bridge_restore(&mut self, cp: crate::parser_bridge::BridgeCheckpoint<'a>) {
        self.restore_context_flags(&cp);
        self.lexer.restore_state(cp.lexer);
        self.cur = cp.cur;
        self.prev = cp.prev;
    }

    /// 仅恢复语句级 context 旗标（不动游标/lexer）——派发错误边界用：
    /// 委托核心解析的 Err 可能泄漏 fn/on 体旗标，恢复点继续解析时必须回到
    /// 方言调用前的语句上下文；游标保持在失败位置（错误位置不变量）。
    pub(crate) fn restore_context_flags(
        &mut self,
        cp: &crate::parser_bridge::BridgeCheckpoint<'a>,
    ) {
        self.in_on_body = cp.in_on_body;
        self.in_fn_body = cp.in_fn_body;
        self.in_dot_rhs = cp.in_dot_rhs;
        self.with_header = cp.with_header;
        self.body_depth = cp.body_depth;
    }

    /// 尝试用已注册的方言解析当前语句起始标识符。
    /// 在 parse_stmt_inner 的 TokenKind::Ident 分支首位置调用。
    /// 返回 Ok(Some(stmt)) 表示某方言已处理；Ok(None) 表示无方言接管。
    ///
    /// PLAN-778 恢复不变量（核心保证，非方言纪律；R1 修订）：
    /// - 注册永不移出（F778-R1-01）：方言表存 Arc 共享句柄，派发时克隆调用。
    ///   活跃方言在自身委托核心解析（闭包体内嵌语句再派发）期间仍在表内，
    ///   合法递归派发可见完整方言表；Err/None 天然不丢注册（从未移出，
    ///   旧 `mem::take` 整表 + `?` 丢表缺陷不复存在）。
    /// - Ok(None) 不吞 token：调用方言前取 checkpoint，None 后强制 restore
    ///   （含游标与 context 旗标）。
    /// - Err 边界恢复 context（F778-R1-02）：委托核心解析泄漏的语句级
    ///   context 旗标在错误边界恢复；游标保持在失败位置（错误位置不变量）。
    fn try_dialect_stmt(&mut self, ident: &str) -> AutoResult<Option<Stmt>> {
        let mut section = 0u8;
        while section < 3 {
            let mut i = 0;
            while i < self.dialect_section_len(section) {
                // PLAN-778：matches() 是"该方言在当前 session 下生效"的契约——
                // 外部方言跨 with_session 持久注册后，场景门控必须在派发时生效
                // （内建方言此前仅靠 build_dialects 的注册时机巧合满足）。
                let dialect = &self.dialect_section(section)[i];
                if !dialect.matches(&self.session) || !dialect.keywords().contains(&ident) {
                    i += 1;
                    continue;
                }
                let cp = self.bridge_checkpoint();
                let dialect = std::sync::Arc::clone(dialect);
                // PLAN-801（F-801-R1-01 修订）：端口快照按**签发作用域**
                // 清理——调用前递增作用域（嵌套保存/恢复），退出时失效本
                // 作用域签发的快照。外层存活快照（他作用域）不受影响；
                // tombstone 复用导致的新快照落在入口表长之下也能正确失效。
                let prev_port_scope = self.port_call_scope;
                self.port_call_scope += 1;
                let call_scope = self.port_call_scope;
                let outcome = {
                    let mut bridge = crate::parser_bridge::ParserBridge::new(self);
                    dialect.try_parse_stmt(&mut bridge, ident)
                };
                self.port_release_scope(call_scope);
                self.port_call_scope = prev_port_scope;
                match outcome {
                    Ok(Some(stmt)) => return Ok(Some(stmt)),
                    Ok(None) => self.bridge_restore(cp),
                    Err(e) => {
                        self.restore_context_flags(&cp);
                        return Err(e);
                    }
                }
                i += 1;
            }
            section += 1;
        }
        Ok(None)
    }

    /// 尝试用已注册的方言解析当前真实 TokenKind（view/on 等非 Ident 的 UI token）。
    /// 在 parse_stmt_inner 的 TokenKind::View / TokenKind::On arm 里调用。
    /// 恢复不变量同 `try_dialect_stmt`（Arc 注册永不出表 / None 强制回滚
    /// 游标+context / Err 边界恢复 context，游标留失败位置）。
    fn try_token_dialect_stmt(&mut self, kind: TokenKind) -> AutoResult<Option<Stmt>> {
        let mut section = 0u8;
        while section < 3 {
            let mut i = 0;
            while i < self.dialect_section_len(section) {
                let dialect = &self.dialect_section(section)[i];
                if !dialect.matches(&self.session) {
                    i += 1;
                    continue;
                }
                let cp = self.bridge_checkpoint();
                let dialect = std::sync::Arc::clone(dialect);
                // PLAN-801（F-801-R1-01 修订）：端口快照按签发作用域清理
                //（同 try_dialect_stmt——嵌套保存/恢复，退出失效本作用域）。
                let prev_port_scope = self.port_call_scope;
                self.port_call_scope += 1;
                let call_scope = self.port_call_scope;
                let outcome = {
                    let mut bridge = crate::parser_bridge::ParserBridge::new(self);
                    dialect.try_parse_token_stmt(&mut bridge, kind)
                };
                self.port_release_scope(call_scope);
                self.port_call_scope = prev_port_scope;
                match outcome {
                    Ok(Some(stmt)) => return Ok(Some(stmt)),
                    Ok(None) => self.bridge_restore(cp),
                    Err(e) => {
                        self.restore_context_flags(&cp);
                        return Err(e);
                    }
                }
                i += 1;
            }
            section += 1;
        }
        Ok(None)
    }

    /// Check if current scenario is UI (Plan 096)
    pub fn is_ui_scenario(&self) -> bool {
        self.session.scenario == crate::session::Scenario::UI
    }

    /// Create a new parser with an external type_store (Plan 085)
    ///
    /// Used after `CompileSession.resolve_uses()` to pass the pre-loaded
    /// type_store containing module symbols.
    pub fn new_with_type_store(code: &'a str, type_store: Arc<RwLock<types::TypeStore>>) -> Self {
        let mut parser = Self::from(code);
        parser.type_store = type_store;
        parser
    }

    pub fn peek(&mut self) -> &Token {
        &self.cur
    }

    pub fn kind(&mut self) -> TokenKind {
        self.peek().kind
    }

    pub fn pos(&mut self) -> Pos {
        self.peek().pos
    }

    pub fn is_kind(&mut self, kind: TokenKind) -> bool {
        self.peek().kind == kind
    }

    /// Plan 356: is the current token a *soft keyword* that should be accepted
    /// as an ordinary identifier in expression/argument position?
    ///
    /// Some words are lexed as reserved keyword tokens (e.g. `tag` →
    /// `TokenKind::Tag`, `type` → `TokenKind::Type`) but users legitimately use
    /// them as variable names (a `for tag in ...` loop variable, a struct field
    /// named `type`). When such a name appears as a function-call argument —
    /// e.g. `onclick: .Select(tag)` — the event-arg parser used to match only
    /// `TokenKind::Ident`, so it consumed nothing and spun forever (OOM).
    ///
    /// This returns true for keyword tokens whose spelling is itself a valid
    /// identifier and which carry no structural meaning in argument position.
    pub(crate) fn cur_is_soft_ident(&self) -> bool {
        matches!(
            self.cur.kind,
            TokenKind::Tag
                | TokenKind::Type
                | TokenKind::Union
                | TokenKind::Spec
                | TokenKind::Super
                | TokenKind::Has
                | TokenKind::Copy
                | TokenKind::Move
                | TokenKind::Take
                | TokenKind::Hold
                | TokenKind::Alias
                | TokenKind::Ext
                | TokenKind::Impl
                | TokenKind::Mod
                | TokenKind::Enum
        )
    }

    /// Check if the current position is a function annotation followed by fn
    /// This is used by the statement parser to distinguish between:
    /// - [c] fn foo() ...  (function declaration)
    /// - [1, 2, 3]         (array literal)
    #[allow(dead_code)]
    fn is_fn_annotation(&mut self) -> bool {
        // We're at [ now, need to check if this is [c] or [vm] followed by fn
        // Manually peek ahead by consuming and checking tokens
        // Collect tokens to restore if this isn't an annotation
        let mut tokens = Vec::new();

        // Save the current token ([)
        tokens.push(self.cur.clone());

        // Skip the [ and check the next token
        self.next(); // Now self.cur is the token after [

        // Check for c or vm identifier
        let is_annot =
            self.cur.kind == TokenKind::Ident && (self.cur.text == "c" || self.cur.text == "vm");

        if !is_annot {
            // Not an annotation, restore and return false
            // We need to restore self.cur to the saved [
            for token in tokens.into_iter().rev() {
                self.lexer.push_token(token);
            }
            // Restore self.cur
            self.next();
            return false;
        }

        // Save the c/vm token
        tokens.push(self.cur.clone());

        // Skip the annotation, check for ]
        self.next(); // Now self.cur is the token after c/vm

        if self.cur.kind != TokenKind::RSquare {
            // Not a properly formed annotation, restore
            for token in tokens.into_iter().rev() {
                self.lexer.push_token(token);
            }
            self.next();
            return false;
        }

        // Save the ] token
        tokens.push(self.cur.clone());

        // Skip ], check for fn
        self.next(); // Now self.cur is the token after ]

        let has_fn = self.cur.kind == TokenKind::Fn;

        // Save the fn token
        tokens.push(self.cur.clone());

        // Restore all tokens in reverse order
        for token in tokens.into_iter().rev() {
            self.lexer.push_token(token);
        }
        // Restore self.cur to the first token
        self.next();

        has_fn
    }

    pub fn skip_comments(&mut self) {
        loop {
            match self.kind() {
                TokenKind::DocComment => {
                    let text = self.cur.text.clone();
                    self.pending_docs.push(text);
                    self.cur = self.lexer.next().expect("lexer should produce token");
                }
                TokenKind::CommentLine
                | TokenKind::CommentStart
                | TokenKind::CommentContent
                | TokenKind::CommentEnd => {
                    self.cur = self.lexer.next().expect("lexer should produce token");
                }
                _ => {
                    break;
                }
            }
        }
    }

    /// Collect pending doc comments and return them as a single string (joined by \n).
    /// Returns None if no doc comments were collected.
    pub fn take_docs(&mut self) -> Option<AutoStr> {
        if self.pending_docs.is_empty() {
            return None;
        }
        let doc: String = self
            .pending_docs
            .drain(..)
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        Some(doc.into())
    }

    pub fn next(&mut self) -> &Token {
        self.prev = self.cur.clone();
        // Try to get the next token, if lexer returns an error, record it and use EOF
        let new_token = match self.lexer.next() {
            Ok(token) => token,
            Err(err) => {
                // Record the lexer error
                self.errors.push(err.into());
                // Check if we've hit the error limit
                if self.errors.len() >= self.error_limit {
                    // Set cur to EOF and return to stop parsing
                    self.cur = Token {
                        kind: TokenKind::EOF,
                        pos: Pos {
                            line: 0,
                            at: 0,
                            pos: 0,
                            len: 0,
                        },
                        text: "".into(),
                    };
                    return &self.cur;
                }
                // Create an EOF token to continue parsing
                Token {
                    kind: TokenKind::EOF,
                    pos: Pos {
                        line: 0,
                        at: 0,
                        pos: 0,
                        len: 0,
                    },
                    text: "".into(),
                }
            }
        };
        self.cur = new_token;
        self.skip_comments();
        &self.cur
    }

    pub fn expect(&mut self, kind: TokenKind) -> AutoResult<()> {
        if self.is_kind(kind) {
            self.next();
            Ok(())
        } else {
            let expected = format!("{:?}", kind);
            let found = self.cur.text.to_string();
            let span = pos_to_span(self.cur.pos);
            Err(SyntaxError::UnexpectedToken {
                expected,
                found,
                span,
            }
            .into())
        }
    }

    fn define(&mut self, name: &str, meta: Meta) {
        // Plan 089: Register to InferenceContext for fn, spec, type, and store
        match &meta {
            // 变量绑定 - 绑定类型到 type_env
            Meta::Store(ref store) => {
                use crate::ast::Name;
                let name_obj = Name::from(name);
                self.infer_ctx.bind_var(name_obj, store.ty.clone());
            }
            // 函数声明 - 注册到 fn_registry 和 TypeStore
            Meta::Fn(ref fn_decl) => {
                self.infer_ctx.register_fn(fn_decl.clone());
                // Plan 084 Phase 2: Also register to TypeStore
                if let Ok(mut store) = self.type_store.write() {
                    store.register_fn_decl(fn_decl);
                }
            }
            // Spec 声明 - 注册到 spec_registry 和 TypeStore
            Meta::Spec(ref spec_decl) => {
                self.infer_ctx.register_spec(spec_decl.clone());
                // Plan 084 Phase 2: Also register to TypeStore
                if let Ok(mut store) = self.type_store.write() {
                    store.register_spec_decl(spec_decl);
                }
            }
            // 类型声明和 Enum 声明 - 确保类型已经注册到 type_registry
            // 注意: TypeDecl 和 EnumDecl 已经在 parse_type_decl/parse_enum_decl 中
            // 通过 register_type_decl() 注册到 type_registry 了（在 Universe 之前）
            // 这里不需要额外注册，只需要确保类型在 registry 中即可
            Meta::Type(_) | Meta::Enum(_) => {
                // 类型声明已经在 parse_type_decl/parse_enum_decl 中通过 register_type_decl() 注册过了
                // Plan 084 Phase 2: Also register to TypeStore
                if let Meta::Type(ref ty) = meta {
                    // Type::User(TypeDecl) contains the TypeDecl
                    if let Type::User(ref type_decl) = ty {
                        if let Ok(mut store) = self.type_store.write() {
                            store.register_type_decl(type_decl);
                        }
                    } else if let Type::Tag(ref tag) = ty {
                        // Register tag types to InferenceContext's global type_env so lookup_type() can find them
                        // Use type_env directly to ensure global visibility (not affected by scopes)
                        use crate::ast::Name;
                        let name_obj = Name::from(name);
                        self.infer_ctx
                            .type_env
                            .insert(name_obj, Type::Tag(tag.clone()));
                    }
                } else if let Meta::Enum(ref enum_decl) = meta {
                    // EnumDecl is similar to TypeDecl, register it with minimal fields
                    let type_decl = crate::ast::TypeDecl {
                        consts: Vec::new(),
                        name: enum_decl.name.clone(),
                        kind: crate::ast::TypeDeclKind::UserType,
                        parent: None,
                        has: vec![],
                        specs: vec![],
                        spec_impls: vec![],
                        generic_params: vec![],
                        members: vec![],
                        methods: vec![],
                        delegations: vec![],
                        attrs: vec![],
                        impl_attrs: vec![],
                        doc: None,
                        is_pub: false,
                    };
                    if let Ok(mut store) = self.type_store.write() {
                        store.register_type_decl(&type_decl);
                    }
                }
            }
            // 其他类型 - 不需要处理
            _ => {}
        }

        // Plan 091: Removed Universe.define() - TypeStore + InferenceContext are sufficient
    }

    fn define_alias(&mut self, alias: AutoStr, target: AutoStr) {
        // Plan 091: Register to TypeStore only (removed Universe fallback)
        if let Ok(mut store) = self.type_store.write() {
            store.register_type_alias(alias.clone(), target);
        }
    }

    #[allow(dead_code)]
    fn define_rc(&mut self, name: &str, meta: Rc<Meta>) {
        // Plan 090: Register to TypeStore + infer_ctx based on meta type
        match meta.as_ref() {
            Meta::Store(ref store) => {
                use crate::ast::Name;
                let name_obj = Name::from(name);
                self.infer_ctx.bind_var(name_obj, store.ty.clone());
            }
            Meta::Fn(ref fn_decl) => {
                self.infer_ctx.register_fn(fn_decl.clone());
                if let Ok(mut store) = self.type_store.write() {
                    store.register_fn_decl(fn_decl);
                }
            }
            Meta::Spec(ref spec_decl) => {
                self.infer_ctx.register_spec(spec_decl.clone());
                if let Ok(mut store) = self.type_store.write() {
                    store.register_spec_decl(spec_decl);
                }
            }
            Meta::Type(ref ty) => {
                if let Type::User(ref type_decl) = ty {
                    if let Ok(mut store) = self.type_store.write() {
                        store.register_type_decl(type_decl);
                    }
                }
            }
            Meta::Enum(ref enum_decl) => {
                let type_decl = crate::ast::TypeDecl {
                    consts: Vec::new(),
                    name: enum_decl.name.clone(),
                    kind: crate::ast::TypeDeclKind::UserType,
                    parent: None,
                    has: vec![],
                    specs: vec![],
                    spec_impls: vec![],
                    generic_params: vec![],
                    members: vec![],
                    methods: vec![],
                    delegations: vec![],
                    attrs: vec![],
                    impl_attrs: vec![],
                    doc: None,
                    is_pub: false,
                };
                if let Ok(mut store) = self.type_store.write() {
                    store.register_type_decl(&type_decl);
                }
            }
            _ => {}
        }
        // Plan 091: Removed Universe.define() - TypeStore + InferenceContext are sufficient
    }

    /// 检查符号是否存在（使用 TypeStore 和 InferenceContext）
    ///
    /// Plan 090: 优先使用 TypeStore，然后是 InferenceContext，最后是 Universe
    fn exists(&mut self, name: &str) -> bool {
        // Plan 090: 首先从 TypeStore 查找
        if let Ok(store) = self.type_store.read() {
            if store.lookup_fn_decl_str(name).is_some() {
                return true;
            }
            if store.lookup_spec_decl_str(name).is_some() {
                return true;
            }
            if store.lookup_type_decl_str(name).is_some() {
                return true;
            }
            if store.lookup_type_alias_str(name).is_some() {
                return true;
            }
        }

        // 从 InferenceContext 查找变量绑定
        // Plan 514 W1（447-① 债①）：嵌套具名 fn 体内不跨外层 fn 边界解析
        // （Rust 嵌套 fn 语义；此前跨界解析 → 运行期静默 0）。闭包不走
        // fn 声明路径，不受影响。
        let found = if self.infer_ctx.fn_scope_idxs.len() >= 2 {
            self.infer_ctx.lookup_type_no_capture(&Name::from(name))
        } else {
            self.infer_ctx.lookup_type(&Name::from(name))
        };
        if found.is_some() {
            return true;
        }

        // Plan 369 P4 (Task 13): py-imported items (`use.py m: a, b`) are
        // resolved at runtime as native shims (py.<name>), so the symbol
        // checker must treat them as defined — otherwise bare references to
        // py constants/functions fail with "undefined variable" outside
        // method-call position (e.g. `uuid5(NAMESPACE_DNS, ...)`).
        if self.py_item_imports.iter().any(|n| n.as_str() == name) {
            return true;
        }

        // PLAN-841 T-05: neutral external-identifier port. UI-owned symbol
        // families (e.g. imported style recipes) live outside Core; their
        // owner registers a typed existence check here so bare references
        // parse (the old core-side design_tokens consult retired with the
        // grammar-table port). No UI names or data flow into Core.
        if let Some(check) = crate::external_ident_checker() {
            if check(name) {
                return true;
            }
        }

        // Plan 091: Removed Universe fallback
        false
    }

    /// 定义符号位置（用于 LSP 和调试）
    ///
    /// Plan 091: 优先使用 Database，然后回退到 Universe
    fn define_symbol_location(&mut self, name: AutoStr, location: SymbolLocation) {
        // Plan 091: 首先尝试使用 Database
        if let Some(ref db) = self.db {
            if let Ok(mut db) = db.write() {
                use crate::scope::Sid;
                let sid = Sid::from(name.as_str());
                db.define_symbol_location(sid, location);
                return;
            }
        }
        // Plan 091: If no Database, symbol locations are not tracked
    }

    pub(crate) fn exit_scope(&mut self) {
        // Plan 091: Use InferenceContext for scope management
        self.infer_ctx.pop_scope();
    }

    /// Plan 514 W1（447-① 债①）：fn 声明体的对称出口（同步弹 fn 边界栈）。
    fn exit_fn_scope(&mut self) {
        self.infer_ctx.pop_fn_scope();
    }

    pub(crate) fn enter_scope(&mut self) {
        // Plan 091: Use InferenceContext for scope management
        self.infer_ctx.push_scope();
    }

    /// 查找元数据（使用 TypeStore 和 InferenceContext）
    ///
    /// Plan 084: 优先使用 TypeStore，然后是 InferenceContext，最后是 Universe
    fn lookup_meta(&mut self, name: &str) -> Option<Rc<Meta>> {
        // Plan 084: 首先从 TypeStore 查找（如果已注册）
        // 注意：TypeStore 存储 Fn, Spec, Type 声明
        if let Ok(store) = self.type_store.read() {
            if let Some(fn_decl) = store.lookup_fn_decl_str(name) {
                return Some(Rc::new(Meta::Fn(fn_decl.clone())));
            }
            if let Some(spec_decl) = store.lookup_spec_decl_str(name) {
                return Some(Rc::new(Meta::Spec(spec_decl.clone())));
            }
            if let Some(type_decl) = store.lookup_type_decl_str(name) {
                return Some(Rc::new(Meta::Type(Type::User(type_decl.as_ref().clone()))));
            }
        }

        // 从 InferenceContext 查找
        self.infer_ctx.lookup_meta(name)
    }

    /// 查找类型（使用 TypeStore 和 InferenceContext）
    ///
    /// Plan 091: Use TypeStore + InferenceContext (removed Universe fallback)
    fn lookup_type(&mut self, name: &str) -> Shared<Type> {
        // Plan 125: Check for built-in type aliases first
        match name {
            "str" => return shared(Type::StrSlice),
            "Str" => return shared(Type::StrOwned),
            "int" => return shared(Type::Int),
            "uint" => return shared(Type::Uint),
            "i64" => return shared(Type::I64),
            "u64" => return shared(Type::U64),
            "float" => return shared(Type::Float),
            "double" => return shared(Type::Double),
            "bool" => return shared(Type::Bool),
            "byte" => return shared(Type::Byte),
            "char" => return shared(Type::Char),
            "void" => return shared(Type::Void),
            "usize" => return shared(Type::USize),
            _ => {}
        }

        // Plan 091/190: 首先从 TypeStore 查找类型声明
        if let Ok(store) = self.type_store.read() {
            if let Some(type_decl) = store.lookup_type_decl_str(name) {
                // Plan 190: Return Type::Rust for use.rust imports
                if let Some(full_path) = store.get_rust_type_path(name) {
                    return shared(Type::Rust(RustSource::new(full_path)));
                }
                return shared(Type::User(type_decl.as_ref().clone()));
            }
            // Also check enum declarations (for heterogeneous enum pattern matching)
            if let Some(enum_decl) = store.lookup_enum_decl_str(name) {
                return shared(Type::Enum(shared(enum_decl.as_ref().clone())));
            }
            // Plan 159 Phase 6B-2.2: Check spec declarations
            // When a spec name is used as a type annotation, return Type::Spec
            // so the transpiler can generate Box<dyn Trait>
            if let Some(spec_decl) = store.lookup_spec_decl_str(name) {
                return shared(Type::Spec(shared(spec_decl.clone())));
            }
        }

        // PLAN-037 后续修复:类型位置不再解析到 InferenceContext。
        // 该表只承载变量绑定(define→bind_var),在类型位置查询它会使
        // 同名变量遮蔽类型名——`let list = ...` 之后的 `let out list = []`
        // 曾因此把标注解析为变量的推断类型(Value→Unknown)而静默丢失。
        // 类型声明走上方 TypeStore;泛型参数走 current_const_params/
        // 类型别名分支。全量测试套件验证此改动不破坏既有解析。

        // Plan 05-Nav: Preserve unknown type names for code generation
        // Only preserve names that look like external types (not common collection names)
        // Common collection names without parameters should return Unknown
        let common_collection_types = [
            "List", "Map", "Set", "Array", "Vec", "HashMap", "HashSet", "Option", "Result",
        ];
        if common_collection_types.contains(&name) {
            // Return typed version for bare collection names (e.g., List -> Type::List(Unknown))
            match name {
                "List" => return shared(Type::List(Box::new(Type::Unknown))),
                "Map" => {
                    return shared(Type::Map(Box::new(Type::Unknown), Box::new(Type::Unknown)))
                }
                _ => return shared(Type::Unknown),
            }
        }

        // Return Type::User with just the name so generators can use it
        // Plan 599 (004 S5-4): F6 formalized -- when an unresolved type name
        // is passed through as a User placeholder, emit a one-time
        // (process-deduped) warning so typos no longer silently produce
        // invalid target code; the passthrough semantics itself is unchanged
        // (foreign names rely on it).
        {
            // Opt-in via env: full-corpus runs showed lookup_type is fed many
            // non-type/lowercase idents and intentional foreign types alike —
            // default-on would be pure noise. AUTO_WARN_UNRESOLVED_TYPES=1
            // enables the one-shot deduped warning (dedupe per process).
            if std::env::var_os("AUTO_WARN_UNRESOLVED_TYPES").is_some() {
                static WARNED: std::sync::OnceLock<
                    std::sync::Mutex<std::collections::HashSet<String>>,
                > = std::sync::OnceLock::new();
                let warned = WARNED.get_or_init(Default::default);
                if let Ok(mut set) = warned.lock() {
                    if set.insert(name.to_string()) {
                        eprintln!(
                            "[type warning] unresolved type name '{}' passed through as opaque",
                            name
                        );
                    }
                }
            }
        }
        shared(Type::User(TypeDecl {
            consts: Vec::new(),
            name: Name::from(name),
            kind: TypeDeclKind::UserType,
            parent: None,
            has: Vec::new(),
            specs: Vec::new(),
            spec_impls: Vec::new(),
            generic_params: Vec::new(),
            members: Vec::new(),
            delegations: Vec::new(),
            methods: Vec::new(),
            attrs: vec![],
            impl_attrs: vec![],
            doc: None,
            is_pub: false,
        }))
    }

    /// 获取所有已定义的名称（用于错误提示）
    ///
    /// Plan 091: Use TypeStore only (removed Universe fallback)
    fn get_defined_names(&self) -> Vec<String> {
        if let Ok(store) = self.type_store.read() {
            return store.get_defined_names();
        }
        Vec::new()
    }

    /// 根据名称查找类型（用于继承和方法查找）
    ///
    /// Plan 091: Use TypeStore only (removed Universe fallback)
    fn find_type_for_name(&self, name: &str) -> Option<Type> {
        if let Ok(store) = self.type_store.read() {
            return store.find_type_for_name(name);
        }
        None
    }

    /// 检查标识符是否是类型名（用于泛型类型解析）
    ///
    /// Plan 091: Use TypeStore only (removed Universe fallback)
    fn lookup_ident_type(&self, name: &str) -> Option<Type> {
        if let Ok(store) = self.type_store.read() {
            if let Some(decl) = store.lookup_type_decl_str(name) {
                return Some(Type::User(decl.as_ref().clone()));
            }
            // 也检查类型别名
            if store.lookup_type_alias_str(name).is_some() {
                return Some(Type::Unknown);
            }
        }
        None
    }

    /// Synchronize parser state after encountering an error
    ///
    /// This method skips tokens until we reach a statement boundary,
    /// allowing the parser to continue collecting errors instead of
    /// aborting on the first error.
    ///
    /// Statement boundaries are:
    /// - Semicolon
    /// - Keywords that start statements (fn, let, var, mut, for, while, if, return, etc.)
    /// - End of file
    fn synchronize(&mut self) {
        // Skip tokens until we reach a statement boundary.
        // Tracks brace depth so we skip past entire block bodies (nodes, if, for, etc.)
        // instead of getting stuck inside them.
        let mut brace_depth: u32 = 0;

        while !self.is_at_end() {
            // Track brace nesting: skip entire blocks
            if self.is_kind(TokenKind::LBrace) {
                brace_depth += 1;
                self.next();
                continue;
            }
            if self.is_kind(TokenKind::RBrace) {
                if brace_depth > 0 {
                    brace_depth -= 1;
                    self.next();
                    continue;
                }
                // Closing brace at depth 0 — this is a block boundary, stop here
                return;
            }

            // Inside a block, keep skipping
            if brace_depth > 0 {
                self.next();
                continue;
            }

            // Semicolon is a statement boundary
            if self.is_kind(TokenKind::Semi) {
                self.next();
                return;
            }

            // Check if we're at a keyword that starts a statement
            match self.kind() {
                TokenKind::Fn
                | TokenKind::Let
                | TokenKind::Var
                | TokenKind::Mut
                | TokenKind::Hold
                | TokenKind::For
                | TokenKind::Loop
                | TokenKind::If
                | TokenKind::Break
                | TokenKind::Continue
                | TokenKind::Use
                | TokenKind::Type
                | TokenKind::Union
                | TokenKind::Tag
                | TokenKind::Enum
                | TokenKind::On
                | TokenKind::Alias
                | TokenKind::Is => {
                    // We've reached a statement boundary, don't consume the token
                    return;
                }
                _ => {
                    // Not at a boundary, skip this token
                    self.next();
                }
            }
        }
    }

    /// Check if we're at the end of the input
    fn is_at_end(&mut self) -> bool {
        self.kind() == TokenKind::EOF
    }

    /// Add an error to the error collection
    ///
    /// Returns true if we should continue parsing (haven't hit error limit),
    /// false if we should abort.
    fn add_error(&mut self, error: AutoError) -> bool {
        // Plan 419 §8.3:超限即拒收(先查后推)。原实现先 push 再查,超限后
        // 每层恢复路径仍把(嵌套着全部下层错误的)传播错误推入,错误树
        // O(n²) 嵌套增长 —— 深嵌套输入实测内存 ~25GB、解析挂死。
        if self.errors.len() >= self.error_limit {
            return false;
        }
        self.errors.push(error);
        self.errors.len() < self.error_limit
    }

    /// Build the error to return when the error limit is exceeded.
    ///
    /// Plan error-spans (jade gap 37a): previously this popped and returned
    /// only the LAST collected error — typically a derivative of error
    /// recovery ("Expected term, got RBrace" at some closing brace), while
    /// the FIRST collected error is the root cause carrying the correct
    /// span. Return all collected errors as MultipleErrors (same shape as
    /// the under-limit path) so the root cause and its span survive.
    fn error_limit_exceeded(&self) -> AutoError {
        let error_count = self.errors.len();
        AutoError::MultipleErrors {
            count: error_count,
            plural: if error_count > 1 {
                "s".into()
            } else {
                "".into()
            },
            errors: self.errors.clone(),
        }
    }

    /// Plan 122: Add a warning to the warnings collection
    fn warn(&mut self, warning: Warning) {
        self.warnings.push(warning);
    }
}

pub enum CodeSection {
    None,
    C,
    Rust,
    Auto,
}

impl<'a> Parser<'a> {
    pub fn parse(&mut self) -> AutoResult<Code> {
        // Plan 419 §8.3(巨帧治理):解析器在 debug 构建下单帧可达 50~270KB
        // (cdb 实测:expr_pratt_with_left ~116KB / parse_stmt_dispatch
        // ~267KB / atom ~74KB,每层嵌套 ~670KB),2MB 线程栈在 3 层嵌套
        // 即溢出 ——「循环内结构体字面量赋值」「循环内嵌套块」等正常形态
        // 都会触发(曾误判为无限递归;实测 64MB 栈下 9 行程序正常解析,
        // 嵌套深度仅 ~13 层 → 有限深度 + 病态巨帧)。
        //
        // 修复:递归咽喉(parse_body / expr_pratt_with_left / atom)挂
        // stacker::maybe_grow —— 余量低于 512KB 时在同线程上切换新的
        // 1MB 堆上栈段继续,数据零跨线程;余量充足时仅一次线程本地读,
        // 近零开销。护栏:body_depth 上限把真病态递归转为干净语法错误。
        self.body_depth = 0;
        self.parse_impl()
    }

    fn parse_impl(&mut self) -> AutoResult<Code> {
        let mut stmts = Vec::new();
        let mut source_lines = Vec::new();
        self.skip_empty_lines();
        let mut current_section = CodeSection::None;
        let mut stmt_index = 0; // Track statement index for is_first_stmt check

        while !self.is_kind(TokenKind::EOF) {
            // deal with sections (but not #[...] annotations)
            if self.is_kind(TokenKind::Hash) {
                // Check if this is a #[...] annotation by looking ahead
                // Save current state for lookahead
                let saved_cur = self.cur.clone();
                // Consume # and check next token
                self.next();
                let is_annotation = self.is_kind(TokenKind::LSquare);
                // Restore state
                self.lexer.push_token(self.cur.clone());
                self.cur = saved_cur;

                if is_annotation {
                    // This is #[...], let parse_stmt() handle it
                    // Don't process as section, continue to parse_stmt()
                } else {
                    // This is a #section declaration — or a corpus-style
                    // `# prose note ...` line (musk relay_store's hand-written
                    // PLAN-032 notes): `#` + name + NON-newline continuation
                    // means prose, skip the whole line (Plan 442 C2).
                    self.next();
                    let section = self.parse_name()?;
                    if !self.is_kind(TokenKind::Newline)
                        && !self.is_kind(TokenKind::EOF)
                        && !self.is_kind(TokenKind::RBrace)
                    {
                        let _ = self.skip_line();
                        continue;
                    }
                    match section.as_str() {
                        "C" => {
                            current_section = CodeSection::C;
                        }
                        "RUST" => {
                            current_section = CodeSection::Rust;
                        }
                        "AUTO" => {
                            current_section = CodeSection::Auto;
                        }
                        _ => {
                            let error = SyntaxError::Generic {
                                message: format!("Unknown section {}", section),
                                span: pos_to_span(self.cur.pos),
                            }
                            .into();
                            if !self.add_error(error) {
                                return Err(self.error_limit_exceeded()); // Error limit exceeded
                            }
                            self.synchronize();
                            continue;
                        }
                    }
                    // skip until newline
                    let _ = self.skip_line();
                    continue;
                }
            }

            if !self.is_kind(TokenKind::Hash) {
                match self.compile_dest {
                    CompileDest::Interp => match current_section {
                        CodeSection::None | CodeSection::Auto => {}
                        _ => {
                            self.skip_line()?;
                            continue;
                        }
                    },
                    CompileDest::TransC => match current_section {
                        CodeSection::None | CodeSection::C => {}
                        _ => {
                            self.skip_line()?;
                            continue;
                        }
                    },
                    CompileDest::TransRust => match current_section {
                        CodeSection::None | CodeSection::Rust => {}
                        _ => {
                            self.skip_line()?;
                            continue;
                        }
                    },
                    CompileDest::Config => match current_section {
                        CodeSection::None | CodeSection::Auto => {}
                        _ => {
                            self.skip_line()?;
                            continue;
                        }
                    },
                }
            }

            let stmt_line = self.cur.pos.line;
            match self.parse_stmt() {
                Ok(stmt) => {
                    // First level pairs are viewed as variable declarations
                    // TODO: this should only happen in a Config scenario
                    if let Stmt::Expr(Expr::Pair(Pair { key, value })) = &stmt {
                        if let Some(name) = key.name() {
                            self.define(
                                name,
                                Meta::Store(Store {
                                    name: name.into(),
                                    kind: StoreKind::Var,
                                    attrs: vec![],
                                    is_pub: false,
                                    ty: Type::Unknown,
                                    expr: *value.clone(),
                                }),
                            );
                        }
                    }
                    stmts.push(stmt);
                    source_lines.push(stmt_line);
                    let is_first = stmt_index == 0;
                    // Check for ambiguous syntax errors in expect_eos
                    let newline_count = match self.expect_eos(is_first) {
                        Ok(count) => count,
                        Err(e) => {
                            // Plan 419 §8.3:护栏错误不可恢复 —— 立即中止。
                            if e.to_string().contains("[non-recoverable]") {
                                return Err(e);
                            }
                            // Ambiguous syntax errors should not be recovered from
                            if e.to_string().contains("Ambiguous syntax") {
                                return Err(e);
                            }
                            // Other EOS errors are added to collection and synchronized
                            if !self.add_error(e) {
                                return Err(self.error_limit_exceeded());
                            }
                            self.synchronize();
                            0
                        }
                    };
                    // Insert EmptyLine statement for 2+ consecutive newlines
                    if newline_count > 1 {
                        stmts.push(Stmt::EmptyLine(newline_count - 1));
                        source_lines.push(stmt_line);
                    }
                    stmt_index += 1;
                }
                Err(e) => {
                    // Check if this is an ambiguous syntax error - these should not be recovered from
                    if e.to_string().contains("Ambiguous syntax") {
                        return Err(e);
                    }
                    // Add error to collection and synchronize
                    if !self.add_error(e) {
                        return Err(self.error_limit_exceeded()); // Error limit exceeded
                    }
                    self.synchronize();
                }
            }
        }

        // Check if we collected any errors
        if !self.errors.is_empty() {
            // Return MultipleErrors with all collected errors
            let error_count = self.errors.len();
            let plural = if error_count > 1 { "s" } else { "" };

            return Err(AutoError::MultipleErrors {
                count: error_count,
                plural: plural.to_string(),
                errors: self.errors.clone(),
            });
        }

        stmts = self.convert_last_block(stmts)?;

        // Post-processing: Merge ext blocks into their target TypeDecl
        stmts = self.merge_ext_blocks(stmts)?;

        Ok(Code {
            stmts,
            source_lines,
            file_attrs: std::mem::take(&mut self.file_attrs),
        })
    }

    pub fn parse_domain<D: auto_parser_api::DomainDialect>(
        &mut self,
        dialect: &D,
    ) -> AutoResult<auto_parser_api::DomainCode<D::Output>> {
        self.body_depth = 0;
        self.parse_domain_impl(dialect)
    }

    fn try_domain_item<D: auto_parser_api::DomainDialect>(
        &mut self,
        dialect: &D,
    ) -> AutoResult<Option<auto_parser_api::DomainStmt<D::Output>>> {
        if !dialect.matches(&self.session) {
            return Ok(None);
        }
        let kind = self.kind();
        let keyword = if kind == TokenKind::Ident {
            self.cur.text.to_string()
        } else {
            String::new()
        };
        let is_pub = kind == TokenKind::Ident && keyword == "pub";
        if !is_pub && kind == TokenKind::Ident && !dialect.keywords().contains(&keyword.as_str()) {
            return Ok(None);
        }

        let cp = self.bridge_checkpoint();
        let prev_port_scope = self.port_call_scope;
        self.port_call_scope += 1;
        let call_scope = self.port_call_scope;
        let outcome = {
            let mut bridge = crate::parser_bridge::ParserBridge::new(self);
            if is_pub {
                dialect.try_parse_pub_stmt(&mut bridge)
            } else if kind == TokenKind::Ident {
                dialect.try_parse_stmt(&mut bridge, &keyword)
            } else {
                dialect.try_parse_token_stmt(&mut bridge, kind)
            }
        };
        self.port_release_scope(call_scope);
        self.port_call_scope = prev_port_scope;
        match outcome {
            Ok(Some(item)) => Ok(Some(item)),
            Ok(None) => {
                self.bridge_restore(cp);
                Ok(None)
            }
            Err(e) => {
                self.restore_context_flags(&cp);
                Err(e)
            }
        }
    }

    fn parse_domain_impl<D: auto_parser_api::DomainDialect>(
        &mut self,
        dialect: &D,
    ) -> AutoResult<auto_parser_api::DomainCode<D::Output>> {
        use auto_parser_api::{DomainCode, DomainStmt};

        let mut items = Vec::new();
        let mut source_lines = Vec::new();
        self.skip_empty_lines();
        let mut current_section = CodeSection::None;
        let mut stmt_index = 0;

        while !self.is_kind(TokenKind::EOF) {
            if self.is_kind(TokenKind::Hash) {
                let saved_cur = self.cur.clone();
                self.next();
                let is_annotation = self.is_kind(TokenKind::LSquare);
                self.lexer.push_token(self.cur.clone());
                self.cur = saved_cur;

                if !is_annotation {
                    self.next();
                    let section = self.parse_name()?;
                    if !self.is_kind(TokenKind::Newline)
                        && !self.is_kind(TokenKind::EOF)
                        && !self.is_kind(TokenKind::RBrace)
                    {
                        let _ = self.skip_line();
                        continue;
                    }
                    match section.as_str() {
                        "C" => current_section = CodeSection::C,
                        "RUST" => current_section = CodeSection::Rust,
                        "AUTO" => current_section = CodeSection::Auto,
                        _ => {
                            let error = SyntaxError::Generic {
                                message: format!("Unknown section {}", section),
                                span: pos_to_span(self.cur.pos),
                            }
                            .into();
                            if !self.add_error(error) {
                                return Err(self.error_limit_exceeded());
                            }
                            self.synchronize();
                            continue;
                        }
                    }
                    let _ = self.skip_line();
                    continue;
                }
            }

            if !self.is_kind(TokenKind::Hash) {
                let include = match self.compile_dest {
                    CompileDest::Interp | CompileDest::Config => {
                        matches!(current_section, CodeSection::None | CodeSection::Auto)
                    }
                    CompileDest::TransC => {
                        matches!(current_section, CodeSection::None | CodeSection::C)
                    }
                    CompileDest::TransRust => {
                        matches!(current_section, CodeSection::None | CodeSection::Rust)
                    }
                };
                if !include {
                    self.skip_line()?;
                    continue;
                }
            }

            let stmt_line = self.cur.pos.line;
            let parsed = match self.try_domain_item(dialect)? {
                Some(item) => Ok(item),
                None => self.parse_stmt().map(DomainStmt::Core),
            };
            match parsed {
                Ok(item) => {
                    if let DomainStmt::Core(Stmt::Expr(Expr::Pair(Pair { key, value }))) = &item {
                        if let Some(name) = key.name() {
                            self.define(
                                name,
                                Meta::Store(Store {
                                    name: name.into(),
                                    kind: StoreKind::Var,
                                    attrs: vec![],
                                    is_pub: false,
                                    ty: Type::Unknown,
                                    expr: *value.clone(),
                                }),
                            );
                        }
                    }
                    items.push(item);
                    source_lines.push(stmt_line);
                    let is_first = stmt_index == 0;
                    let newline_count = match self.expect_eos(is_first) {
                        Ok(count) => count,
                        Err(e) => {
                            if e.to_string().contains("[non-recoverable]")
                                || e.to_string().contains("Ambiguous syntax")
                            {
                                return Err(e);
                            }
                            if !self.add_error(e) {
                                return Err(self.error_limit_exceeded());
                            }
                            self.synchronize();
                            0
                        }
                    };
                    if newline_count > 1 {
                        items.push(DomainStmt::Core(Stmt::EmptyLine(newline_count - 1)));
                        source_lines.push(stmt_line);
                    }
                    stmt_index += 1;
                }
                Err(e) => {
                    if e.to_string().contains("Ambiguous syntax") {
                        return Err(e);
                    }
                    if !self.add_error(e) {
                        return Err(self.error_limit_exceeded());
                    }
                    self.synchronize();
                }
            }
        }

        if !self.errors.is_empty() {
            return Err(AutoError::MultipleErrors {
                count: self.errors.len(),
                plural: if self.errors.len() > 1 { "s" } else { "" }.to_string(),
                errors: self.errors.clone(),
            });
        }

        if matches!(items.last(), Some(DomainStmt::Core(_))) {
            let Some(DomainStmt::Core(last)) = items.pop() else {
                unreachable!()
            };
            let converted = self.convert_last_block(vec![last])?;
            items.push(DomainStmt::Core(converted.into_iter().next().unwrap()));
        }
        let mut code = DomainCode {
            items,
            source_lines,
            file_attrs: std::mem::take(&mut self.file_attrs),
        };
        self.merge_domain_ext_blocks(&mut code)?;
        Ok(code)
    }

    fn merge_domain_ext_blocks<D>(
        &self,
        code: &mut auto_parser_api::DomainCode<D>,
    ) -> AutoResult<()> {
        use auto_parser_api::DomainStmt;

        let mut type_decl_indices: std::collections::HashMap<Name, usize> =
            std::collections::HashMap::new();
        let mut ext_statements: Vec<(usize, crate::ast::Ext)> = Vec::new();
        for (i, item) in code.items.iter().enumerate() {
            let DomainStmt::Core(stmt) = item else {
                continue;
            };
            match stmt {
                Stmt::TypeDecl(decl) => {
                    type_decl_indices.insert(decl.name.clone(), i);
                }
                Stmt::Ext(ext) if ext.trait_name.is_none() => {
                    ext_statements.push((i, ext.clone()));
                }
                _ => {}
            }
        }

        let mut merged_ext_indices = Vec::new();
        for (ext_idx, ext) in ext_statements {
            let Some(&decl_idx) = type_decl_indices.get(&ext.target) else {
                continue;
            };
            let DomainStmt::Core(Stmt::TypeDecl(decl)) = &code.items[decl_idx] else {
                continue;
            };
            let mut merged_decl = decl.clone();
            for ext_field in ext.fields.clone() {
                merged_decl.members.retain(|m| m.name != ext_field.name);
                merged_decl.members.push(ext_field);
            }
            for ext_method in ext.methods.clone() {
                merged_decl.methods.retain(|m| m.name != ext_method.name);
                merged_decl.methods.push(ext_method);
            }
            for ext_const in ext.consts.clone() {
                merged_decl.consts.retain(|c| c.name != ext_const.name);
                merged_decl.consts.push(ext_const);
            }
            merged_decl.impl_attrs.extend(ext.attrs.iter().cloned());
            code.items[decl_idx] = DomainStmt::Core(Stmt::TypeDecl(merged_decl));
            merged_ext_indices.push(ext_idx);
        }

        let mut index = 0;
        code.items.retain(|item| {
            let keep = !matches!(item, DomainStmt::Core(Stmt::Ext(_)))
                || !merged_ext_indices.contains(&index);
            index += 1;
            keep
        });
        let mut index = 0;
        code.source_lines.retain(|_| {
            let keep = !merged_ext_indices.contains(&index);
            index += 1;
            keep
        });
        Ok(())
    }

    /// Merge ext blocks into their target TypeDecl
    /// This processes all Stmt::Ext statements and merges their fields/methods
    /// into the corresponding Stmt::TypeDecl
    fn merge_ext_blocks(&self, mut stmts: Vec<Stmt>) -> AutoResult<Vec<Stmt>> {
        use crate::ast::Stmt;

        // Collect all TypeDecls and Exts in separate passes
        let mut type_decl_indices: std::collections::HashMap<Name, usize> =
            std::collections::HashMap::new();
        let mut ext_statements: Vec<(usize, crate::ast::Ext)> = Vec::new();

        for (i, stmt) in stmts.iter().enumerate() {
            if let Stmt::TypeDecl(ref decl) = stmt {
                type_decl_indices.insert(decl.name.clone(), i);
            } else if let Stmt::Ext(ref ext) = stmt {
                // Plan 164: ext blocks with "for Trait" should NOT be merged into TypeDecl
                // They must remain as standalone Stmt::Ext so the transpiler can emit
                // "impl Trait for Type" instead of "impl Type"
                if ext.trait_name.is_none() {
                    ext_statements.push((i, ext.clone()));
                }
            }
        }

        // Track which Ext statements were successfully merged
        let mut merged_ext_indices: Vec<usize> = Vec::new();

        // Process each Ext statement and merge into target TypeDecl
        for (ext_idx, ext) in ext_statements {
            // Find the target TypeDecl
            if let Some(&decl_idx) = type_decl_indices.get(&ext.target) {
                // Clone the TypeDecl, merge ext into it, then replace
                if let Stmt::TypeDecl(ref decl) = &stmts[decl_idx] {
                    let mut merged_decl = decl.clone();

                    // Merge fields: ext fields override type fields with same name
                    for ext_field in ext.fields.clone() {
                        // Remove existing field with same name if it exists
                        merged_decl.members.retain(|m| m.name != ext_field.name);
                        // Add the ext field
                        merged_decl.members.push(ext_field);
                    }

                    // Merge methods: ext methods override type methods with same name
                    for ext_method in ext.methods.clone() {
                        // Remove existing method with same name if it exists
                        merged_decl.methods.retain(|m| m.name != ext_method.name);
                        // Add the ext method
                        merged_decl.methods.push(ext_method);
                    }

                    // C8: merge associated consts (same-name ext const wins).
                    for ext_const in ext.consts.clone() {
                        merged_decl.consts.retain(|c| c.name != ext_const.name);
                        merged_decl.consts.push(ext_const);
                    }

                    // Plan 364 W1: ext's impl-level macro attrs move onto the merged
                    // type's impl block (e.g. `#[zbus.interface] impl CosmicSession`).
                    merged_decl.impl_attrs.extend(ext.attrs.iter().cloned());

                    stmts[decl_idx] = Stmt::TypeDecl(merged_decl);
                    // Mark this Ext as merged
                    merged_ext_indices.push(ext_idx);
                }
            }
            // If no TypeDecl exists (extending built-in type like int, str, etc.),
            // the Ext statement is kept in the AST for later processing
        }

        // Remove only Ext statements that were successfully merged
        let mut final_stmts: Vec<Stmt> = Vec::new();
        for (i, stmt) in stmts.into_iter().enumerate() {
            if let Stmt::Ext(_) = stmt {
                // Only keep this Ext if it wasn't merged
                if !merged_ext_indices.contains(&i) {
                    final_stmts.push(stmt);
                }
                // If it was merged, skip it (don't add to final_stmts)
            } else {
                final_stmts.push(stmt);
            }
        }

        Ok(final_stmts)
    }

    fn skip_line(&mut self) -> AutoResult<()> {
        // println!("Skiipping line"); // Debug output disabled for LSP
        while !self.is_kind(TokenKind::Newline) {
            self.next();
        }
        self.skip_empty_lines();
        Ok(())
    }

    fn skip_block(&mut self) -> AutoResult<()> {
        // Skip a { ... } block, handling nested braces
        self.expect(TokenKind::LBrace)?;
        let mut depth = 1;
        while depth > 0 && !self.is_kind(TokenKind::EOF) {
            if self.is_kind(TokenKind::LBrace) {
                depth += 1;
            } else if self.is_kind(TokenKind::RBrace) {
                depth -= 1;
            }
            self.next();
        }
        if depth > 0 {
            return Err(SyntaxError::Generic {
                message: "Unclosed block, missing '}'".to_string(),
                span: pos_to_span(self.prev.pos),
            }
            .into());
        }
        Ok(())
    }

    fn convert_last_block(&mut self, mut stmts: Vec<Stmt>) -> AutoResult<Vec<Stmt>> {
        let last = stmts.last();
        if let Some(st) = last {
            match st {
                // Plan 567 T12: 尾块仅在**纯 pair 形**时转对象（UI 单元构造
                // 语义）；含语句（Store/Try 等）的块是作用域块（with-as 块
                // 形态产物等），保留原样——原实现一律 body_to_obj，非 pair
                // 尾块报 "Last block must be an object!" 挡死块语句收尾。
                Stmt::Block(body) => {
                    let all_pairs = body
                        .stmts
                        .iter()
                        .all(|s| matches!(s, Stmt::Expr(Expr::Pair(_))));
                    if all_pairs && !body.stmts.is_empty() {
                        let obj = self.body_to_obj(body)?;
                        stmts.pop();
                        stmts.push(Stmt::Expr(Expr::Object(obj)))
                    }
                }
                _ => {}
            }
        }
        Ok(stmts)
    }

    fn body_to_obj(&mut self, body: &Body) -> AutoResult<Vec<Pair>> {
        let mut pairs = Vec::new();
        for stmt in body.stmts.iter() {
            match stmt {
                Stmt::Expr(expr) => match expr {
                    Expr::Pair(p) => {
                        pairs.push(p.clone());
                    }
                    _ => {
                        return Err(SyntaxError::Generic {
                            message: "Last block must be an object!".to_string(),
                            span: pos_to_span(self.cur.pos),
                        }
                        .into());
                    }
                },
                _ => {
                    return Err(SyntaxError::Generic {
                        message: "Last block must be an object!".to_string(),
                        span: pos_to_span(self.cur.pos),
                    }
                    .into());
                }
            }
        }
        Ok(pairs)
    }
}

// Expressions
impl<'a> Parser<'a> {
    pub fn key(&mut self) -> AutoResult<Key> {
        match self.kind() {
            TokenKind::Ident => {
                let name = self.cur.text.clone();
                self.next();
                Ok(Key::NamedKey(name))
            }
            TokenKind::Int => {
                let value = self.cur.text.as_str().parse().unwrap();
                self.next();
                Ok(Key::IntKey(value))
            }
            TokenKind::True => {
                self.next();
                Ok(Key::BoolKey(true))
            }
            TokenKind::False => {
                self.next();
                Ok(Key::BoolKey(false))
            }
            TokenKind::Str => {
                let value = self.cur.text.clone();
                self.next();
                Ok(Key::StrKey(value))
            }
            // type关键字用作key，应该不冲突
            TokenKind::Type => {
                let value = self.cur.text.clone();
                self.next();
                Ok(Key::NamedKey(value))
            }
            // Plan 502 M2:`to` 是 Plan 162 的 .to(Type) 方法关键字,但
            // diagram 边 record 的合法键(设计文档 diagram-components.md
            // §5 edge schema { from, to })。键位上下文化,与 Type/Tag/
            // Task 同款——.to() 转换走点后缀位,与此不冲突。
            TokenKind::To => {
                let value = self.cur.text.clone();
                self.next();
                Ok(Key::NamedKey(value))
            }
            // 2026-08-22(tag 软关键字):tag 作字段名(Cell{ tag: "..." })
            TokenKind::Tag => {
                let value = self.cur.text.clone();
                self.next();
                Ok(Key::NamedKey(value))
            }
            // Plan 028 T14：`task` 是 Plan 121 关键字 token，但合法的对象键
            // （errand 状态 { task: ... }）。与 Type 同样的上下文化处理。
            TokenKind::Task => {
                let value = self.cur.text.clone();
                self.next();
                Ok(Key::NamedKey(value))
            }
            _ => {
                let message = format!("Expected key, got {:?}", self.kind());
                let span = pos_to_span(self.cur.pos);
                return Err(SyntaxError::Generic { message, span }.into());
            }
        }
    }
}

/// Plan 312: Parse `#[api(method = "GET", path = "/api/notes/:id")]` argument string.
/// Input is the raw args collected by collect_annotation_args, e.g.:
///   `(method = "GET", path = "/api/notes")`
/// Returns (method, path) with sensible defaults if fields are missing.
fn parse_api_args(args: &str) -> (String, String) {
    let mut method = String::from("GET");
    let mut path = String::from("/");
    // collect_annotation_args returns "(method = \"POST\", path = \"/x\")"
    // with surrounding parens; strip them so split(',') yields clean key=value parts.
    let args = args.trim_start_matches('(').trim_end_matches(')');
    for part in args.split(',') {
        let part = part.trim();
        if let Some(val) = extract_quoted_value(part, "method") {
            method = val;
        } else if let Some(val) = extract_quoted_value(part, "path") {
            path = val;
        }
    }
    (method, path)
}

/// Plan 610 ⑤: normalize `#[export(...)]` args to a Rust ABI string.
/// collect_annotation_args yields "(system)" / "(\"system\")" / None;
/// bare `#[export]` and "c" map to "C", "system"/"stdcall" to "system"
/// (win32), anything else passes through verbatim (rustc validates).
fn parse_export_abi_arg(args: Option<String>) -> AutoStr {
    let raw = match args {
        Some(a) => a
            .trim_start_matches('(')
            .trim_end_matches(')')
            .trim()
            .trim_matches('"')
            .to_string(),
        None => String::new(),
    };
    match raw.as_str() {
        "" | "c" | "C" => "C".to_string().into(),
        "system" | "stdcall" => "system".to_string().into(),
        other => other.to_string().into(),
    }
}

/// Helper: extract the quoted string value from `key = "value"` form.
fn extract_quoted_value(s: &str, key: &str) -> Option<String> {
    let s = s.trim();
    let prefix = format!("{} = \"", key);
    if let Some(rest) = s.strip_prefix(&prefix) {
        if let Some(end) = rest.find('"') {
            return Some(rest[..end].to_string());
        }
    }
    None
}

// Statements
impl<'a> Parser<'a> {
    // End of statement
    pub fn expect_eos(&mut self, _is_first_stmt: bool) -> AutoResult<usize> {
        // Save the previous token before consuming separators
        let token_before_sep = self.prev.clone();

        let mut has_sep = false;
        let mut newline_count = 0;
        while self.is_kind(TokenKind::Semi)
            || self.is_kind(TokenKind::Newline)
            || self.is_kind(TokenKind::Comma)
        {
            has_sep = true;
            if self.is_kind(TokenKind::Newline) {
                newline_count += 1;
            }
            self.next();
        }
        if self.is_kind(TokenKind::EOF) || self.is_kind(TokenKind::RBrace) {
            return Ok(newline_count);
        }
        if has_sep {
            // Check for ambiguous sequence: ')', '\n', '{'
            // This is ambiguous - use 2+ newlines to disambiguate
            if self.is_kind(TokenKind::LBrace)
                && token_before_sep.kind == TokenKind::RParen
                && newline_count == 1
            {
                return Err(SyntaxError::Generic {
                    message:
                        "Ambiguous syntax: statement ending with ')' followed by newline and '{'. \
                    Use 2+ newlines to separate statements, or put the '{' on the same line."
                            .to_string(),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
            Ok(newline_count)
        } else {
            Err(SyntaxError::Generic {
                message: format!(
                    "Expected end of statement, got {:?}<{}>",
                    self.kind(),
                    self.cur.text
                ),
                span: pos_to_span(self.cur.pos),
            }
            .into())
        }
    }

    fn expect_ident_str(&mut self) -> AutoResult<AutoStr> {
        if self.is_kind(TokenKind::Ident) {
            let name = self.cur.text.clone();
            self.next(); // skip name
            Ok(name)
        } else {
            return Err(SyntaxError::Generic {
                message: format!("Expected identifier, got {:?}", self.kind()),
                span: pos_to_span(self.cur.pos),
            }
            .into());
        }
    }

    /// Skip balanced parentheses: (...)
    /// Plan 159 Phase 6B-2: Used for skipping attribute arguments like #[derive(Debug, Clone)]
    pub fn skip_balanced_parens(&mut self) -> AutoResult<()> {
        if !self.is_kind(TokenKind::LParen) {
            return Ok(());
        }
        self.next(); // skip (
        let mut depth = 1;
        while depth > 0 && !self.is_kind(TokenKind::EOF) {
            if self.is_kind(TokenKind::LParen) {
                depth += 1;
            } else if self.is_kind(TokenKind::RParen) {
                depth -= 1;
            }
            self.next();
        }
        Ok(())
    }

    pub fn skip_empty_lines(&mut self) -> usize {
        let mut count = 0;
        while self.is_kind(TokenKind::Newline) {
            count += 1;
            self.next();
        }
        count
    }

    pub fn expr_stmt(&mut self) -> AutoResult<Stmt> {
        Ok(Stmt::Expr(self.parse_expr()?))
    }

    /// Check if the next token is likely the start of a type
    fn next_token_is_type(&mut self) -> bool {
        // Peek at the next token
        let next = self.peek();
        matches!(
            next.kind,
            TokenKind::Ident |      // Type name (int, List, etc.)
            TokenKind::Question |   // May type (?T)
            TokenKind::LSquare |    // Array type ([N]T)
            TokenKind::Star |       // Pointer type (*T)
            TokenKind::Lt // Nested generic (List<List<int>>)
        )
    }

    /// Plan 408 P12 §10.3: browser/宿主全局标识符白名单——parser 豁免，
    /// codegen（ts_adapter）原样直出。Vue script-setup 运行时这些是全局可用的。
    const HOST_GLOBALS: &'static [&'static str] = &[
        "document",
        "window",
        "navigator",
        "localStorage",
        "sessionStorage",
        "matchMedia",
        "confirm",
        "alert",
        "prompt",
        "location",
        "history",
        "console",
        "fetch",
        "requestAnimationFrame",
        "cancelAnimationFrame",
        "setTimeout",
        "setInterval",
        "clearTimeout",
        "clearInterval",
        "HTMLElement",
        "Event",
        "EventTarget",
        "IntersectionObserver",
        "ResizeObserver",
        "MutationObserver",
        "URL",
        "URLSearchParams",
        "FormData",
        "Blob",
        "File",
        "FileReader",
        "Headers",
        "Request",
        "Response",
        "ReadableStream",
        "WritableStream",
        "TextEncoder",
        "TextDecoder",
        "crypto",
        "performance",
        "screen",
    ];

    /// Plan 410: `err_span` is the expression-start span recorded by the caller
    /// BEFORE the expression was parsed. UndefinedVariable errors point there
    /// (the offending identifier for the checked forms) instead of at the
    /// cursor, which has already moved past the whole expression.
    pub fn check_symbol(&mut self, expr: Expr, err_span: SourceSpan) -> AutoResult<Expr> {
        if self.skip_check {
            return Ok(expr);
        }
        // Plan 364: in Config mode (pac.at manifest), `port` and other context
        // variables are injected at evaluation time; skip strict undefined checks.
        if self.compile_dest == CompileDest::Config {
            return Ok(expr);
        }
        match &expr {
            Expr::Bina(l, op, _) => match op {
                Op::Dot => {
                    if let Expr::Ident(name) = l.as_ref() {
                        // Check if it's a type name (for static method calls like HashMap.new())
                        let is_type = self.lookup_type(name);
                        let is_type_valid = match *is_type.borrow() {
                            Type::User(_) | Type::Tag(_) | Type::Enum(_) => true,
                            _ => false,
                        };

                        if !self.exists(&name) && !is_type_valid {
                            // Allow module-qualified names (e.g., flow.FlowSpec)
                            // where the left side is an imported module, not a local variable
                            if !self.is_imported_module(name) {
                                let candidates = self.get_defined_names();
                                return Err(NameError::undefined_variable(
                                    name.to_string(),
                                    err_span,
                                    &candidates,
                                )
                                .into());
                            }
                        }
                    }
                    Ok(expr)
                }
                _ => {
                    // Plan 514 W1（447-① 债①）：非 Dot 二元的裸 Ident 操作数
                    // 此前漏检——嵌套具名 fn 体内 `base + n` 的捕获经默认臂
                    // 静默放行（运行期得 0）。在嵌套 fn 内对两侧 Ident 做
                    // no-capture 检查（外层 fn 局部不可捕获）。
                    if self.infer_ctx.fn_scope_idxs.len() >= 2 {
                        if let Expr::Ident(name) = l.as_ref() {
                            if !self.exists(name.as_str())
                                && self
                                    .infer_ctx
                                    .lookup_type(&Name::from(name.as_str()))
                                    .is_some()
                            {
                                let candidates = self.get_defined_names();
                                return Err(NameError::undefined_variable(
                                    name.to_string(),
                                    err_span,
                                    &candidates,
                                )
                                .into());
                            }
                        }
                    }
                    Ok(expr)
                }
            },
            Expr::Ident(name) => {
                // Plan 408 P12 §10.3:豁免宿主全局标识符（browser API）——
                // document/window/navigator/localStorage 等在 .at 运行时
                // （Vue script setup）是全局可用的，parser 不应把它们当
                // 未定义变量。codegen（ts_adapter）原样直出，无需特殊处理。
                if Self::HOST_GLOBALS.contains(&name.as_str()) {
                    return Ok(expr);
                }
                // Plan 514 W1（447-① 债②）：`struct` 不是 Auto 关键字——C 风格
                // `struct Name {...}` 误用此前经本臂报 E0201 "Variable ... is
                // not defined"（名字解析错，误导排查方向）。在表达式位撞见裸
                // `struct`（且未作变量定义）直接报语法错并指出正确写法 `type`。
                if name.as_str() == "struct" && !self.exists(&name) {
                    return Err(SyntaxError::Generic {
                        message: "unexpected `struct`: Auto 的具名结构体声明是 `type Name { ... }`"
                            .to_string(),
                        span: err_span,
                    }
                    .into());
                }
                if !self.exists(&name) {
                    let candidates = self.get_defined_names();
                    return Err(NameError::undefined_variable(
                        name.to_string(),
                        err_span,
                        &candidates,
                    )
                    .into());
                }
                Ok(expr)
            }
            Expr::Call(call) => {
                match call.name.as_ref() {
                    Expr::Ident(_name) => {
                        // TODO: Re-enable this check for production, but skip for AST tests
                        // if !self.exists(&name) {
                        //     return error_pos!("Function {} not define!", name);
                        // }
                    }
                    Expr::Bina(lhs, op, _rhs) => {
                        // check tag creation or static method call (TypeName.method())
                        if let Op::Dot = op {
                            if let Expr::Ident(lname) = lhs.as_ref() {
                                let ltype = self.lookup_type(lname);
                                match *ltype.borrow() {
                                    Type::Tag(ref _t) => {}
                                    Type::User(ref _name) => {
                                        // Allow type names for static method calls (e.g., HashMap.new())
                                        // The evaluator will route these to the VM registry
                                    }
                                    _ => {}
                                };
                            }
                        }
                    }
                    _ => {}
                }
                Ok(expr)
            }
            _ => Ok(expr),
        }
    }

    /// Plan 364: Inject context globals into the parser symbol table for Config
    /// mode (pac.at manifest evaluation). Mirrors old auto-man 0.1.3's
    /// `env.set_global("port", port_name)` — the `port` variable holds the
    /// currently selected port name so `if port == "win32" { ... }` can resolve.
    pub fn define_config_globals(&mut self, port: &str) {
        self.define(
            "port",
            Meta::Store(Store {
                name: AutoStr::from("port"),
                kind: StoreKind::Var,
                attrs: vec![],
                is_pub: false,
                ty: Type::Unknown,
                expr: Expr::Str(port.into()),
            }),
        );
    }

    /// Helper: expect an identifier with specific text
    pub(crate) fn expect_ident(&mut self, expected: &str) -> AutoResult<()> {
        if self.cur.text.as_str() == expected {
            self.next();
            Ok(())
        } else {
            Err(SyntaxError::Generic {
                message: format!("Expected '{}', got '{}'", expected, self.cur.text),
                span: pos_to_span(self.cur.pos),
            }
            .into())
        }
    }
}

/// Plan 560 T07：中缀糖直产桥调用（`a @ b`→py_matmul、`a ** b`→py_pow）。
fn mk_infix_sugar_call(name: &str, l: crate::ast::Expr, r: crate::ast::Expr) -> crate::ast::Expr {
    use crate::ast::{Arg, Args, Call, Expr, Type};
    Expr::Call(Call {
        name: Box::new(Expr::Ident(name.into())),
        args: Args {
            args: vec![Arg::Pos(l), Arg::Pos(r)],
        },
        ret: Type::Unknown,
        type_args: Vec::new(),
        generic_args: Vec::new(),
        pos: None,
    })
}

// ---- PLAN-843 结构拆分：语法族主题模块（原单文件热点，纯搬移） ----
mod decls;
mod expressions;
mod scenario;
mod statements;
#[cfg(test)]
mod tests;
mod type_syntax;
