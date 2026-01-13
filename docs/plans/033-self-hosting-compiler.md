# Self-Hosting Auto Compiler Implementation Plan

## Implementation Status: ⏳ PLANNED

**Dependencies:**
- Plan 024 (Ownership-Based Memory System) - Must complete first
- Plan 025 (String Type Redesign) - Must complete first
- Plan 027 (Stdlib C Foundation) - Must complete first
**Estimated Start:** After Plan 027 completion (~35-44 weeks from now)

## Executive Summary

Implement an AutoLang compiler IN AutoLang that can be transpiled to C using the existing a2c transpiler and built with auto-man. This achieves self-hosting: the compiler can compile itself, reducing dependency on the Rust implementation.

**Timeline**: 24-30 weeks (6-7.5 months) after Plan 027
**Complexity**: Very High (parser = 4,399 lines, transpiler = 2,505 lines)
**Priority**: HIGH - Core strategic goal for AutoLang

**Key Decision:**
- ✅ Target: **C** (not Rust)
- ✅ Build system: **auto-man** (existing, mature)
- ✅ Stdlib: C-based foundation (Plans 024 & 025)

**Goal:** Build a minimal viable compiler → iterate to feature parity → achieve self-hosting

---

## 1. Strategic Context

### 1.1 Why Self-Hosting?

**Benefits:**
1. **Independence** - Reduce dependency on Rust implementation
2. **Credibility** - Prove AutoLang is production-ready
3. **Bootstrapping** - Compiler can compile itself
4. **Community** - Easier to contribute (C more widely known)
5. **Embedded Focus** - Aligns with Auto-Man's embedded ecosystem

**Risks:**
- Performance may be worse than Rust implementation initially
- Limited feature set in first version
- Memory safety concerns (C code generation)

### 1.2 Why C + Auto-Man?

**Advantages over Rust:**
- ✅ **Auto-Man exists** - Complete build system ready
- ✅ **Simpler build** - gcc/clang everywhere
- ✅ **Faster development** - No need to build tooling first
- ✅ **Wider adoption** - More C developers than Rust
- ✅ **Proven path** - Go, Swift started with C backends

**Disadvantages:**
- ❌ Manual memory management (mitigated with arena allocation)
- ❌ No compile-time safety (mitigated with testing)
- ❌ Verbose code (acceptable trade-off)

### 1.3 Bootstrap Strategy

**Iterative Bootstrapping:**
```
Round 1: Rust compiler (existing) → Minimal Auto/C compiler
Round 2: Minimal Auto/C → Enhanced Auto/C compiler
Round 3: Enhanced Auto/C → Full-featured Auto/C compiler
Round 4: Full Auto/C → Self-hosting milestone!
```

**Why this works:**
- Start small: lexer + parser + simple code gen
- Add features incrementally through self-compilation
- Each version compiled by previous version
- Final result independent of Rust implementation

---

## 2. Current State Analysis

### 2.1 Existing Rust Implementation

**Mature components to reference:**
- **Lexer** ([lexer.rs](../../crates/auto-lang/src/lexer.rs:1)) - 962 lines
- **Parser** ([parser.rs](../../crates/auto-lang/src/parser.rs:1)) - 4,399 lines
- **AST** ([ast.rs](../../crates/auto-lang/src/ast.rs:1)) - Complete type definitions
- **C Transpiler** ([trans/c.rs](../../crates/auto-lang/src/trans/c.rs:1)) - 2,505 lines
- **Scope Management** ([scope.rs](../../crates/auto-lang/src/scope.rs:1)) - 150+ lines
- **Type Inference** ([infer/](../../crates/auto-lang/src/infer/)) - 1,794 lines (285+ tests)

**What this means:**
- Proven architecture to replicate
- Reference implementations for every component
- Comprehensive test suite to match
- Known edge cases and solutions

### 2.2 Existing Self-Hosting Effort

**Current auto/ directory status:**
- ⏸️ **Very early stage** - Only 5 basic .at files
- `auto.at` - Hello world
- `lexer.at` - Source abstraction with character iteration
- `token.at` - Token type definitions (partial, 25+ kinds missing)
- `pos.at` - Position tracking
- `pac.at` - Parser combinator (experimental)

**Missing components:**
- ❌ Complete token system (50+ kinds)
- ❌ Full lexer implementation
- ❌ Parser implementation
- ❌ AST representation in AutoLang
- ❌ Symbol table
- ❌ Type checker
- ❌ Code generator

### 2.3 Stdlib Foundation (Plan 027)

**Prerequisites from Plan 027:**
- ✅ HashMap/HashSet - For symbol tables
- ✅ StringBuilder - For code generation
- ✅ Result/Option - For error handling
- ✅ String interning - For fast identifier comparison
- ✅ Args parsing - For CLI interface

**Status:** Must be completed first (7-8 months)

---

## 3. Architecture Overview

### 3.1 Compiler Pipeline

```
AutoLang Source (.at)
    ↓
[Lexer] → Tokens
    ↓
[Parser] → AST (Abstract Syntax Tree)
    ↓
[Symbol Table] → Scopes & Symbols
    ↓
[Type Checker] → Typed AST
    ↓
[C Transpiler] → C Code
    ↓
[GCC/Clang] → Binary
```

### 3.2 Module Structure

```
auto/                          # Self-hosted compiler
├── lib/                       # Core libraries
│   ├── token.at              # Token types
│   ├── pos.at                # Position tracking
│   ├── error.at              # Error reporting
│   ├── ast.at                # AST node definitions
│   ├── symbol.at             # Symbol table
│   └── type_check.at         # Type checking
├── compiler/                  # Compiler components
│   ├── lexer.at              # Lexical analysis
│   ├── parser.at             # Parsing
│   ├── transpiler.at         # C code generation
│   └── compiler.at           # Main driver
├── tests/                     # Test suites
│   ├── lexer/
│   ├── parser/
│   ├── transpiler/
│   └── integration/
├── examples/                  # Example programs
└── auto.at                    # Main entry point
```

### 3.3 Data Flow

**Compilation flow:**
```auto
// Pseudocode
fn compile(source_path str) Result<(), Error> {
    // 1. Read source
    let source = read_file(source_path)?

    // 2. Lexical analysis
    let mut lexer = Lexer::new(source)
    let tokens = lexer.tokenize_all()?

    // 3. Parsing
    let mut parser = Parser::new(tokens)
    let ast = parser.parse()?

    // 4. Symbol table building
    let mut symbols = SymbolTable::new()
    symbols.build_from_ast(&ast)?

    // 5. Type checking
    let mut checker = TypeChecker::new(&symbols)
    checker.check(&ast)?

    // 6. Code generation
    let mut transpiler = CTranspiler::new(&symbols)
    let c_code = transpiler.transpile(&ast)

    // 7. Write C file
    let c_path = source_path + ".c"
    write_file(c_path, c_code)

    Ok(())
}
```

---

## 4. Implementation Phases

### Phase 1: Foundation - Token System & Position Tracking (3 weeks)

**Objective:** Build token and position infrastructure.

**Dependencies:** Stdlib foundation (Plan 027)

#### 1.1 Complete Token System

**File:** `auto/lib/token.at`

```auto
use stdlib/result: Result
use stdlib/option: Option

// All token kinds (50+ total)
extern enum TokenKind {
    // Literals
    I8Lit, U8Lit, I16Lit, U16Lit, I32Lit, U32Lit, I64Lit, U64Lit
    DecLit, FloatLit, DoubleLit, StrLit, CStrLit, CharLit, RuneLit

    // Operators
    Add, Sub, Mul, Div, Mod
    AddEq, SubEq, MulEq, DivEq, ModEq
    Eq, Neq, Lt, Gt, Le, Ge
    And, Or, Not
    Assign, Asn

    // Delimiters
    LParen, RParen, LSquare, RSquare, LBrace, RBrace
    Comma, Colon, Semicolon, Dot, DotDot, DotDotEq, Arrow, FatArrow

    // Keywords
    Let, Var, Const, Mut, Type, Alias
    Fn, Return, If, Else, For, While, Loop, Break, Continue
    Is, In, Has, Use, Spec, Enum, Struct, Union
    True, False, Nil

    Ident,
    Eof
}

type Token {
    kind TokenKind
    pos Pos
    text str
    len uint

    fn new(kind TokenKind, pos Pos, text str, len uint) Token {
        Token {
            kind: kind,
            pos: pos,
            text: text,
            len: len
        }
    }

    fn is_eof(token Token) bool {
        token.kind == TokenKind::Eof
    }

    fn is_keyword(token Token) bool {
        match token.kind {
            TokenKind::Let => true,
            TokenKind::Fn => true,
            TokenKind::Return => true,
            TokenKind::If => true,
            TokenKind::Else => true,
            TokenKind::For => true,
            TokenKind::While => true,
            TokenKind::Loop => true,
            TokenKind::Break => true,
            TokenKind::Continue => true,
            _ => false
        }
    }
}

// Convert identifier text to keyword kind
fn keyword_kind(text str) TokenKind {
    match text {
        "let" => TokenKind::Let,
        "mut" => TokenKind::Mut,
        "var" => TokenKind::Var,
        "const" => TokenKind::Const,
        "fn" => TokenKind::Fn,
        "return" => TokenKind::Return,
        "if" => TokenKind::If,
        "else" => TokenKind::Else,
        "for" => TokenKind::For,
        "while" => TokenKind::While,
        "loop" => TokenKind::Loop,
        "break" => TokenKind::Break,
        "continue" => TokenKind::Continue,
        "is" => TokenKind::Is,
        "type" => TokenKind::Type,
        "enum" => TokenKind::Enum,
        "struct" => TokenKind::Struct,
        "true" => TokenKind::True,
        "false" => TokenKind::False,
        "nil" => TokenKind::Nil,
        _ => TokenKind::Ident
    }
}
```

#### 1.2 Position Tracking

**File:** `auto/lib/pos.at`

```auto
// Enhanced position tracking
type Pos {
    line uint      // Line number (1-based)
    at uint        // Column from line start (1-based)
    total uint     // Absolute byte offset (0-based)

    fn new() Pos {
        Pos { line: 1, at: 1, total: 0 }
    }

    fn advance(mut pos Pos, c char) {
        pos.total = pos.total + 1
        if c == '\n' {
            pos.line = pos.line + 1
            pos.at = 1
        } else {
            pos.at = pos.at + 1
        }
    }

    fn to_string(pos Pos) str {
        f"Pos(line: \(pos.line), at: \(pos.at), total: \(pos.total))"
    }
}
```

**Testing:**
```auto
// tests/token/test_token.at
fn test_token_creation() {
    let pos = Pos::new()
    let token = Token::new(TokenKind::Let, pos, "let", 3)
    assert(token.kind == TokenKind::Let)
    assert(token.text == "let")
    assert(!token.is_eof())
    assert(token.is_keyword())
}

fn test_keyword_detection() {
    assert(keyword_kind("let") == TokenKind::Let)
    assert(keyword_kind("fn") == TokenKind::Fn)
    assert(keyword_kind("identifier") == TokenKind::Ident)
}
```

**Success Criteria:**
- All 50+ token kinds defined
- Position tracking accurate
- 20+ unit tests passing

---

### Phase 2: Lexer Implementation (5-6 weeks)

**Objective:** Complete lexer with all token types.

**Dependencies:** Phase 1 (token system)

**Reference:** [lexer.rs](../../crates/auto-lang/src/lexer.rs:1) (962 lines)

#### 2.1 Lexer Core

**File:** `auto/compiler/lexer.at`

```auto
use stdlib/result: Result
use stdlib/string: builder
use lib/token: {Token, TokenKind, keyword_kind}
use lib/pos: Pos
use lib/error: ErrorCollector

type Lexer {
    source str
    len uint
    pos Pos
    cur_char char
    errors ErrorCollector

    fn new(source str) Lexer {
        let mut lex = Lexer {
            source: source,
            len: source.len(),
            pos: Pos::new(),
            cur_char: -1,
            errors: ErrorCollector::new()
        }
        lex.advance()
        lex
    }

    fn advance(mut lexer Lexer) {
        if lexer.pos.total >= lexer.len {
            lexer.cur_char = -1  // EOF
            return
        }
        lexer.cur_char = lexer.source[lexer.pos.total]
        Pos::advance(mut lexer.pos, lexer.cur_char)
    }

    fn peek(lexer Lexer, offset int) char {
        let pos = lexer.pos.total + offset
        if pos >= lexer.len {
            return -1
        }
        lexer.source[pos]
    }

    fn skip_whitespace(mut lexer Lexer) {
        while lexer.cur_char != -1 {
            if lexer.cur_char == ' ' || lexer.cur_char == '\t' || lexer.cur_char == '\r' {
                lexer.advance()
            } else if lexer.cur_char == '\n' {
                lexer.pos.line = lexer.pos.line + 1
                lexer.pos.at = 1
                lexer.advance()
            } else if lexer.cur_char == '/' {
                // Check for // comments
                if lexer.peek(0) == '/' {
                    while lexer.cur_char != '\n' && lexer.cur_char != -1 {
                        lexer.advance()
                    }
                } else {
                    break
                }
            } else {
                break
            }
        }
    }

    fn next_token(mut lexer Lexer) Token {
        lexer.skip_whitespace()

        let start_pos = Pos {
            line: lexer.pos.line,
            at: lexer.pos.at,
            total: lexer.pos.total
        }

        if lexer.cur_char == -1 {
            return Token::new(TokenKind::Eof, start_pos, "", 0)
        }

        // Numbers
        if lexer.cur_char >= '0' && lexer.cur_char <= '9' {
            return lexer.number(start_pos)
        }

        // Identifiers and keywords
        if lexer.cur_char == '_' ||
           (lexer.cur_char >= 'a' && lexer.cur_char <= 'z') ||
           (lexer.cur_char >= 'A' && lexer.cur_char <= 'Z') {
            return lexer.ident_or_keyword(start_pos)
        }

        // Strings
        if lexer.cur_char == '"' {
            return lexer.string(start_pos)
        }

        // F-strings
        if lexer.cur_char == 'f' && lexer.peek(0) == '"' {
            lexer.advance()
            return lexer.fstring(start_pos)
        }

        // Operators and delimiters (extensive matching)
        lexer.operator_or_delimiter(start_pos)
    }

    fn number(mut lexer Lexer, start_pos Pos) Token {
        let mut sb = StringBuilder::new(32)
        let mut kind = TokenKind::I32Lit

        while lexer.cur_char >= '0' && lexer.cur_char <= '9' {
            StringBuilder_append_char(mut sb, lexer.cur_char)
            lexer.advance()
        }

        // TODO: Handle decimals, hex, type suffixes

        let text = StringBuilder_build(sb)
        Token::new(kind, start_pos, text, text.len())
    }

    fn ident_or_keyword(mut lexer Lexer, start_pos Pos) Token {
        let mut sb = StringBuilder::new(32)

        while lexer.cur_char == '_' ||
              (lexer.cur_char >= 'a' && lexer.cur_char <= 'z') ||
              (lexer.cur_char >= 'A' && lexer.cur_char <= 'Z') ||
              (lexer.cur_char >= '0' && lexer.cur_char <= '9') {
            StringBuilder_append_char(mut sb, lexer.cur_char)
            lexer.advance()
        }

        let text = StringBuilder_build(sb)
        let kind = keyword_kind(text)
        Token::new(kind, start_pos, text, text.len())
    }

    fn string(mut lexer Lexer, start_pos Pos) Token {
        lexer.advance()  // Skip opening "
        let mut sb = StringBuilder::new(64)

        while lexer.cur_char != '"' && lexer.cur_char != -1 {
            if lexer.cur_char == '\\' {
                lexer.advance()
                // Handle escape sequences
            }
            StringBuilder_append_char(mut sb, lexer.cur_char)
            lexer.advance()
        }

        if lexer.cur_char == '"' {
            lexer.advance()  // Skip closing "
        }

        let text = StringBuilder_build(sb)
        Token::new(TokenKind::StrLit, start_pos, text, text.len())
    }

    fn fstring(mut lexer Lexer, start_pos Pos) Token {
        lexer.advance()  // Skip opening "
        let mut sb = StringBuilder::new(64)
        StringBuilder_append_char(mut sb, 'f')
        StringBuilder_append_char(mut sb, '"')

        while lexer.cur_char != '"' && lexer.cur_char != -1 {
            // TODO: Handle $var and ${expr} interpolation
            StringBuilder_append_char(mut sb, lexer.cur_char)
            lexer.advance()
        }

        if lexer.cur_char == '"' {
            lexer.advance()  // Skip closing "
        }

        StringBuilder_append_char(mut sb, '"')
        let text = StringBuilder_build(sb)
        Token::new(TokenKind::StrLit, start_pos, text, text.len())
    }

    fn operator_or_delimiter(mut lexer Lexer, start_pos Pos) Token {
        // Extensive matching for all operators
        match lexer.cur_char {
            '+' => {
                lexer.advance()
                if lexer.cur_char == '=' {
                    lexer.advance()
                    Token::new(TokenKind::AddEq, start_pos, "+=", 2)
                } else {
                    Token::new(TokenKind::Add, start_pos, "+", 1)
                }
            }
            '-' => {
                lexer.advance()
                if lexer.cur_char == '=' {
                    lexer.advance()
                    Token::new(TokenKind::SubEq, start_pos, "-=", 2)
                } else {
                    Token::new(TokenKind::Sub, start_pos, "-", 1)
                }
            }
            // ... all other operators
            _ => {
                lexer.errors.add(ErrorLevel::Error,
                    f"unexpected character: \(lexer.cur_char)", start_pos)
                lexer.advance()
                Token::new(TokenKind::Eof, start_pos, "", 0)
            }
        }
    }
}
```

**Testing:**
```auto
// tests/lexer/test_lexer.at
fn test_lexer_numbers() {
    let source = "123 456 789"
    let mut lexer = Lexer::new(source)

    let t1 = lexer.next_token()
    assert(t1.kind == TokenKind::I32Lit)
    assert(t1.text == "123")

    let t2 = lexer.next_token()
    assert(t2.kind == TokenKind::I32Lit)
    assert(t2.text == "456")
}

fn test_lexer_keywords() {
    let source = "let mut fn return"
    let mut lexer = Lexer::new(source)

    assert(lexer.next_token().kind == TokenKind::Let)
    assert(lexer.next_token().kind == TokenKind::Mut)
    assert(lexer.next_token().kind == TokenKind::Fn)
    assert(lexer.next_token().kind == TokenKind::Return)
}

fn test_lexer_strings() {
    let source = "\"hello\" \"world\""
    let mut lexer = Lexer::new(source)

    let t1 = lexer.next_token()
    assert(t1.kind == TokenKind::StrLit)
    assert(t1.text == "hello")
}

fn test_lexer_identifiers() {
    let source = "foo bar_baz Baz123"
    let mut lexer = Lexer::new(source)

    let t1 = lexer.next_token()
    assert(t1.kind == TokenKind::Ident)
    assert(t1.text == "foo")

    let t2 = lexer.next_token()
    assert(t2.kind == TokenKind::Ident)
    assert(t2.text == "bar_baz")
}
```

**Success Criteria:**
- All 50+ token types tokenize correctly
- F-string interpolation parsed
- Position tracking accurate
- 50+ unit tests passing
- Matches Rust lexer behavior on all a2c test cases

---

### Phase 3: Symbol Table & Scope Management (4-5 weeks)

**Objective:** Implement hierarchical symbol table.

**Dependencies:** Stdlib HashMap (Plan 027)

**Reference:** [scope.rs](../../crates/auto-lang/src/scope.rs:1) (150+ lines)

#### 3.1 Symbol Table Implementation

**File:** `auto/lib/symbol.at`

```auto
use stdlib/collections: HashMap
use lib/token: TokenKind

// Symbol kinds
extern enum SymbolKind {
    Var,
    Const,
    Fn,
    Type,
    Param,
    Local
}

// Type representation (simplified)
extern enum Type {
    Void,
    Nil,
    Bool,
    I8, I16, I32, I64,
    U8, U16, U32, U64,
    Float, Double,
    Str, CStr,
    Array { elem Type, len int? },
    Ptr { inner Type },
    Named { name str },
    Unknown,
    Error { msg str }
}

// Symbol information
type Symbol {
    name str
    kind SymbolKind
    type Type
    defined_at Pos
}

// Scope level
type Scope {
    parent int?        // Parent scope index
    level uint         // Nesting depth
    symbols HashMap<str, Symbol>  // Symbol map

    fn new(parent int?, level uint) Scope {
        Scope {
            parent: parent,
            level: level,
            symbols: HashMap::new()
        }
    }

    fn define(mut scope Scope, sym Symbol) {
        scope.symbols.insert(sym.name, sym)
    }

    fn lookup(scope Scope, name str) Option<Symbol> {
        scope.symbols.get(name)
    }
}

// Symbol table with hierarchical scopes
type SymbolTable {
    scopes []Scope
    cur int            // Current scope index

    fn new() SymbolTable {
        let mut st = SymbolTable {
            scopes: [],
            cur: -1
        }
        st.push_scope()
        st
    }

    fn push_scope(mut st SymbolTable) {
        let parent = if st.cur >= 0 { Some(st.cur) } else { None }
        let level = st.scopes.len() as uint
        st.cur = st.scopes.len()
        st.scopes.append(Scope::new(parent, level))
    }

    fn pop_scope(mut st SymbolTable) {
        if st.cur > 0 {
            let parent = st.scopes[st.cur].parent
            if parent != nil {
                st.cur = parent.unwrap()
            } else {
                st.cur = -1
            }
        }
    }

    fn define(mut st SymbolTable, sym Symbol) {
        st.scopes[st.cur].define(sym)
    }

    fn lookup(st SymbolTable, name str) Option<Symbol> {
        mut idx = st.cur
        while idx >= 0 {
            let sym = st.scopes[idx].lookup(name)
            if sym != nil {
                return sym
            }
            let parent = st.scopes[idx].parent
            if parent == nil {
                break
            }
            idx = parent.unwrap()
        }
        nil
    }

    fn lookup_local(st SymbolTable, name str) Option<Symbol> {
        st.scopes[st.cur].lookup(name)
    }
}
```

**Testing:**
```auto
// tests/symbol/test_symbol_table.at
fn test_symbol_table_basic() {
    let mut st = SymbolTable::new()

    // Define in global scope
    let sym = Symbol {
        name: "x",
        kind: SymbolKind::Var,
        type: Type::I32,
        defined_at: Pos::new()
    }
    st.define(sym)

    // Lookup should find it
    let found = st.lookup("x")
    assert(found != nil)
    assert(found.unwrap().name == "x")
}

fn test_nested_scopes() {
    let mut st = SymbolTable::new()

    // Global scope
    st.define(Symbol{name: "global", kind: SymbolKind::Var,
                     type: Type::I32, defined_at: Pos::new()})

    // Inner scope
    st.push_scope()
    st.define(Symbol{name: "local", kind: SymbolKind::Var,
                     type: Type::I32, defined_at: Pos::new()})

    // Should find local
    assert(st.lookup("local") != nil)

    // Should find global (parent scope)
    assert(st.lookup("global") != nil)

    st.pop_scope()

    // Should still find global
    assert(st.lookup("global") != nil)

    // Should NOT find local (popped scope)
    assert(st.lookup("local") == nil)
}

fn test_shadowing() {
    let mut st = SymbolTable::new()

    // Outer x
    st.define(Symbol{name: "x", kind: SymbolKind::Var,
                     type: Type::I32, defined_at: Pos::new()})

    // Inner scope shadows x
    st.push_scope()
    st.define(Symbol{name: "x", kind: SymbolKind::Var,
                     type: Type::Float, defined_at: Pos::new()})

    let found = st.lookup("x")
    assert(found != nil)
    assert(found.unwrap().type == Type::Float)  // Shadowed type
}
```

**Success Criteria:**
- Hierarchical scopes working
- Shadowing handled correctly
- Parent scope lookup functional
- 50+ unit tests passing

---

### Phase 4: AST Representation (5-6 weeks)

**Objective:** Define AST node structure.

**Dependencies:** Phase 3 (symbol table)

**Reference:** [ast.rs](../../crates/auto-lang/src/ast.rs:1)

#### 4.1 AST Types

**File:** `auto/lib/ast.at`

```auto
use lib/symbol: {Symbol, SymbolKind, Type}

// Operators
extern enum Op {
    Add, Sub, Mul, Div, Mod,
    Eq, Neq, Lt, Gt, Le, Ge,
    And, Or, Not,
    Assign
}

// Function parameter
type Param {
    name str
    type Type
    default Option<Type>  // Default value expression
}

// Enum variant
type Variant {
    name str
    fields []Param
}

// Expressions
extern enum Expr {
    Int { value int, type Type },
    Float { value float, type Type },
    Str { value str },
    Bool { value bool },
    Nil,

    Ident { name str, symbol Option<Symbol> },

    Binary { op Op, left Box<Expr>, right Box<Expr>, type Type },
    Unary { op Op, operand Box<Expr>, type Type },

    Call { func Box<Expr>, args []Expr, type Type },
    Index { target Box<Expr>, index Box<Expr>, type Type },

    If { cond Box<Expr>, then Box<Expr>, else_ Box<Expr>?, type Type },

    Array { elems []Expr, type Type },
    Block { stmts []Stmt, expr Box<Expr>?, type Type }
}

// Statements
extern enum Stmt {
    Expr { expr Expr },
    Let { name str, type Option<Type>, init Option<Expr> },
    Mut { name str, type Option<Type>, init Option<Expr> },
    Assign { target Expr, value Expr },

    Fn { name str, params []Param, ret_type Type, body Box<Stmt> },
    Return { value Option<Expr> },

    If { cond Expr, then_block Stmt, else_block Option<Stmt> },
    For { var str, iter Expr, body Stmt },
    While { cond Expr, body Stmt },
    Loop { body Stmt },

    Break,
    Continue,

    TypeDecl { name str, type Type },
    Enum { name str, variants []Variant },

    Use { path str, imports []str }
}

// AST builder helpers
type AstBuilder {
    errors ErrorCollector

    fn new() AstBuilder {
        AstBuilder { errors: ErrorCollector::new() }
    }

    fn int_lit(builder AstBuilder, value int) Expr {
        Expr::Int {
            value: value,
            type: Type::I32
        }
    }

    fn float_lit(builder AstBuilder, value float) Expr {
        Expr::Float {
            value: value,
            type: Type::Float
        }
    }

    fn bool_lit(builder AstBuilder, value bool) Expr {
        Expr::Bool { value: value }
    }

    fn nil(builder AstBuilder) Expr {
        Expr::Nil
    }

    fn ident(builder AstBuilder, name str) Expr {
        Expr::Ident {
            name: name,
            symbol: nil
        }
    }

    fn binary(builder AstBuilder, op Op, left Expr, right Expr) Expr {
        let ty = infer_binary_type(left, right)
        Expr::Binary {
            op: op,
            left: Box::new(left),
            right: Box::new(right),
            type: ty
        }
    }

    fn call(builder AstBuilder, func Expr, args []Expr) Expr {
        Expr::Call {
            func: Box::new(func),
            args: args,
            type: Type::Unknown
        }
    }

    fn block(builder AstBuilder, stmts []Stmt) Expr {
        Expr::Block {
            stmts: stmts,
            expr: nil,
            type: Type::Unknown
        }
    }

    fn let_stmt(builder AstBuilder, name str, type Option<Type>, init Option<Expr>) Stmt {
        Stmt::Let {
            name: name,
            type: type,
            init: init
        }
    }

    fn fn_decl(builder AstBuilder, name str, params []Param,
               ret_type Type, body Stmt) Stmt {
        Stmt::Fn {
            name: name,
            params: params,
            ret_type: ret_type,
            body: Box::new(body)
        }
    }

    fn if_stmt(builder AstBuilder, cond Expr, then_block Stmt, else_block Option<Stmt>) Stmt {
        Stmt::If {
            cond: cond,
            then_block: then_block,
            else_block: else_block
        }
    }

    fn return_stmt(builder AstBuilder, value Option<Expr>) Stmt {
        Stmt::Return {
            value: value
        }
    }
}

// Simple type inference for binary ops
fn infer_binary_type(left Expr, right Expr) Type {
    match left {
        Expr::Int { type: Type::I32 } => Type::I32,
        Expr::Float { type: Type::Float } => Type::Float,
        _ => Type::Unknown
    }
}
```

**Testing:**
```auto
// tests/ast/test_ast.at
fn test_ast_builder() {
    let builder = AstBuilder::new()

    // Build: 1 + 2
    let left = builder.int_lit(1)
    let right = builder.int_lit(2)
    let expr = builder.binary(Op::Add, left, right)

    match expr {
        Expr::Binary { op: Op::Add } => {
            assert(true)
        }
        _ => assert(false)
    }
}

fn test_ast_function() {
    let builder = AstBuilder::new()

    let params = []
    let body = builder.block([])

    let fn_decl = builder.fn_decl("main", params, Type::Void, body)

    match fn_decl {
        Stmt::Fn { name: "main" } => assert(true),
        _ => assert(false)
    }
}
```

**Success Criteria:**
- All AST node types defined
- Builder methods create valid nodes
- Pretty-print matches input
- 50+ unit tests passing

---

### Phase 5: Parser Implementation (8-10 weeks)

**Objective:** Recursive descent parser with precedence climbing.

**Dependencies:** Phases 1-4

**Reference:** [parser.rs](../../crates/auto-lang/src/parser.rs:1) (4,399 lines)

#### 5.1 Parser Core

**File:** `auto/compiler/parser.at`

```auto
use stdlib/result: Result
use stdlib/option: Option
use compiler/lexer: Lexer
use lib/token: {Token, TokenKind}
use lib/ast: *
use lib/ast_builder: AstBuilder
use lib/symbol: SymbolTable
use lib/error: ErrorCollector

type Parser {
    lexer Lexer
    cur Token
    prev Token
    symbols SymbolTable
    errors ErrorCollector
    builder AstBuilder

    fn new(source str) Parser {
        let mut lexer = Lexer::new(source)
        let mut parser = Parser {
            lexer: lexer,
            cur: Token::new(TokenKind::Eof, Pos::new(), "", 0),
            prev: Token::new(TokenKind::Eof, Pos::new(), "", 0),
            symbols: SymbolTable::new(),
            errors: ErrorCollector::new(),
            builder: AstBuilder::new()
        }
        parser.advance()
        parser
    }

    fn advance(mut parser Parser) {
        parser.prev = parser.cur
        parser.cur = parser.lexer.next_token()
    }

    fn expect(mut parser Parser, kind TokenKind) Result<(), str> {
        if parser.cur.kind == kind {
            parser.advance()
            Result::Ok(())
        } else {
            let msg = f"expected \(kind), found \(parser.cur.kind)"
            parser.errors.add(ErrorLevel::Error, msg, parser.cur.pos)
            Result::Err(msg)
        }
    }

    // Entry point
    fn parse(mut parser Parser) Result<Stmt, str> {
        mut stmts = []

        while parser.cur.kind != TokenKind::Eof {
            let stmt = parser.parse_stmt()?
            stmts.append(stmt)
        }

        Ok(parser.builder.block(stmts))
    }

    // Statement parser
    fn parse_stmt(mut parser Parser) Result<Stmt, str> {
        match parser.cur.kind {
            TokenKind::Let => parser.parse_let(),
            TokenKind::Mut => parser.parse_mut(),
            TokenKind::Fn => parser.parse_fn(),
            TokenKind::If => parser.parse_if(),
            TokenKind::For => parser.parse_for(),
            TokenKind::While => parser.parse_while(),
            TokenKind::Return => parser.parse_return(),
            _ => {
                let expr = parser.parse_expr()?
                Ok(Stmt::Expr { expr: expr })
            }
        }
    }

    fn parse_let(mut parser Parser) Result<Stmt, str> {
        parser.advance()  // consume 'let'
        let name = parser.cur.text
        parser.expect(TokenKind::Ident)

        let mut type = nil
        if parser.cur.kind == TokenKind::Colon {
            parser.advance()
            type = Some(parser.parse_type()?)
        }

        let mut init = nil
        if parser.cur.kind == TokenKind::Assign {
            parser.advance()
            init = Some(parser.parse_expr()?)
        }

        Ok(parser.builder.let_stmt(name, type, init))
    }

    fn parse_fn(mut parser Parser) Result<Stmt, str> {
        parser.advance()  // consume 'fn'
        let name = parser.cur.text
        parser.expect(TokenKind::Ident)
        parser.expect(TokenKind::LParen)

        // Parse parameters
        mut params = []
        while parser.cur.kind != TokenKind::RParen {
            let param_name = parser.cur.text
            parser.advance()
            parser.expect(TokenKind::Colon)
            let param_type = parser.parse_type()?
            params.append(Param {
                name: param_name,
                type: param_type,
                default: nil
            })

            if parser.cur.kind == TokenKind::Comma {
                parser.advance()
            }
        }
        parser.expect(TokenKind::RParen)

        // Parse return type
        let mut ret_type = Type::Void
        if parser.cur.kind == TokenKind::Arrow {
            parser.advance()
            ret_type = parser.parse_type()?
        }

        // Parse body
        parser.symbols.push_scope()
        let body = parser.parse_block()?
        parser.symbols.pop_scope()

        Ok(parser.builder.fn_decl(name, params, ret_type, body))
    }

    fn parse_if(mut parser Parser) Result<Stmt, str> {
        parser.advance()  // consume 'if'
        let cond = parser.parse_expr()?
        let then_block = parser.parse_block()?

        let mut else_block = nil
        if parser.cur.kind == TokenKind::Else {
            parser.advance()
            if parser.cur.kind == TokenKind::If {
                else_block = Some(parser.parse_if()?)
            } else {
                else_block = Some(parser.parse_block()?)
            }
        }

        Ok(parser.builder.if_stmt(cond, then_block, else_block))
    }

    fn parse_block(mut parser Parser) Result<Stmt, str> {
        parser.expect(TokenKind::LBrace)
        parser.symbols.push_scope()

        mut stmts = []
        while parser.cur.kind != TokenKind::RBrace {
            let stmt = parser.parse_stmt()?
            stmts.append(stmt)
        }

        parser.symbols.pop_scope()
        parser.expect(TokenKind::RBrace)

        Ok(Stmt::Expr { expr: parser.builder.block(stmts) })
    }

    fn parse_return(mut parser Parser) Result<Stmt, str> {
        parser.advance()  // consume 'return'
        let mut value = nil

        if parser.cur.kind != TokenKind::Semicolon {
            value = Some(parser.parse_expr()?)
        }

        Ok(parser.builder.return_stmt(value))
    }

    // Expression parser with precedence climbing
    fn parse_expr(mut parser Parser) Result<Expr, str> {
        parser.parse_expr_with_prec(0)
    }

    fn parse_expr_with_prec(mut parser Parser, prec int) Result<Expr, str> {
        mut left = parser.parse_atom()?

        while parser.is_binary_op() && parser.precedence() >= prec {
            let op_token = parser.cur
            parser.advance()
            let right = parser.parse_expr_with_prec(prec + 1)?
            left = parser.builder.binary(op_token.kind.to_op(), left, right)
        }

        Ok(left)
    }

    fn parse_atom(mut parser Parser) Result<Expr, str> {
        match parser.cur.kind {
            TokenKind::I32Lit => {
                let value = parser.cur.text.to_int()
                let pos = parser.cur.pos
                parser.advance()
                Ok(parser.builder.int_lit(value))
            }
            TokenKind::FloatLit => {
                let value = parser.cur.text.to_float()
                parser.advance()
                Ok(parser.builder.float_lit(value))
            }
            TokenKind::Ident => {
                let name = parser.cur.text
                parser.advance()
                Ok(parser.builder.ident(name))
            }
            TokenKind::LParen => {
                parser.advance()
                let expr = parser.parse_expr()?
                parser.expect(TokenKind::RParen)
                Ok(expr)
            }
            _ => {
                let msg = f"unexpected token: \(parser.cur.kind)"
                parser.errors.add(ErrorLevel::Error, msg, parser.cur.pos)
                Result::Err(msg)
            }
        }
    }

    fn parse_type(mut parser Parser) Result<Type, str> {
        match parser.cur.kind {
            TokenKind::Ident => {
                let name = parser.cur.text
                parser.advance()
                Ok(Type::Named { name: name })
            }
            TokenKind::LBrace => {
                // Array type: [T]
                parser.advance()
                let elem = parser.parse_type()?
                parser.expect(TokenKind::RSquare)
                Ok(Type::Array { elem: elem, len: nil })
            }
            _ => {
                let msg = f"expected type, found: \(parser.cur.kind)"
                Result::Err(msg)
            }
        }
    }

    fn is_binary_op(parser Parser) bool {
        match parser.cur.kind {
            TokenKind::Add => true,
            TokenKind::Sub => true,
            TokenKind::Mul => true,
            TokenKind::Div => true,
            TokenKind::Eq => true,
            TokenKind::Lt => true,
            TokenKind::Gt => true,
            _ => false
        }
    }

    fn precedence(parser Parser) int {
        match parser.cur.kind {
            TokenKind::Eq => 1,
            TokenKind::Lt | TokenKind::Gt => 2,
            TokenKind::Add | TokenKind::Sub => 3,
            TokenKind::Mul | TokenKind::Div => 4,
            _ => 0
        }
    }
}

// Convert TokenKind to Op
fn to_op(kind TokenKind) Op {
    match kind {
        TokenKind::Add => Op::Add,
        TokenKind::Sub => Op::Sub,
        TokenKind::Mul => Op::Mul,
        TokenKind::Div => Op::Div,
        TokenKind::Eq => Op::Eq,
        TokenKind::Lt => Op::Lt,
        TokenKind::Gt => Op::Gt,
        _ => Op::Add  // fallback
    }
}
```

**Testing:**
```auto
// tests/parser/test_parser.at
fn test_parser_let() {
    let source = "let x: int = 42"
    let mut parser = Parser::new(source)
    let stmt = parser.parse().unwrap()

    match stmt {
        Stmt::Let { name: "x" } => assert(true),
        _ => assert(false)
    }
}

fn test_parser_function() {
    let source = "fn add(a: int, b: int) int { a + b }"
    let mut parser = Parser::new(source)
    let stmt = parser.parse().unwrap()

    match stmt {
        Stmt::Fn { name: "add" } => assert(true),
        _ => assert(false)
    }
}

fn test_parser_binary_expr() {
    let source = "1 + 2 * 3"
    let mut parser = Parser::new(source)
    let expr = parser.parse_expr().unwrap()

    // Should be: 1 + (2 * 3) due to precedence
    match expr {
        Expr::Binary { op: Op::Add } => assert(true),
        _ => assert(false)
    }
}
```

**Success Criteria:**
- Parse all a2c test cases successfully
- 100+ test cases passing
- Error recovery continues compilation
- Symbol table populated correctly

---

### Phase 6: Type Checker (6-8 weeks)

**Objective:** Type inference and checking.

**Dependencies:** Phase 5 (parser + symbol table)

**Reference:** [infer/](../../crates/auto-lang/src/infer/) (1,794 lines)

#### 6.1 Type Checker

**File:** `auto/lib/type_check.at`

```auto
use lib/ast: *
use lib/symbol: SymbolTable
use lib/error: ErrorCollector

type TypeChecker {
    symbols SymbolTable
    errors ErrorCollector

    fn new() TypeChecker {
        TypeChecker {
            symbols: SymbolTable::new(),
            errors: ErrorCollector::new()
        }
    }

    // Main entry
    fn check(mut checker TypeChecker, stmt Stmt) Result<Type, str> {
        match stmt {
            Stmt::Expr { expr } => checker.check_expr(expr),
            Stmt::Let { name, type, init } => {
                let init_type = checker.check_expr(init.unwrap_or(Expr::Nil))?
                if type != nil {
                    checker.unify(type.unwrap(), init_type)?
                }
                checker.symbols.define(Symbol {
                    name: name,
                    kind: SymbolKind::Var,
                    type: init_type,
                    defined_at: Pos::new()
                })
                Ok(init_type)
            }
            Stmt::Fn { name, params, ret_type, body } => {
                checker.symbols.push_scope()
                for param in params {
                    checker.symbols.define(Symbol {
                        name: param.name,
                        kind: SymbolKind::Param,
                        type: param.type,
                        defined_at: Pos::new()
                    })
                }
                let body_type = checker.check_stmt(*body)?
                checker.symbols.pop_scope()
                checker.unify(ret_type, body_type)?;
                Ok(ret_type)
            }
            _ => Ok(Type::Void)
        }
    }

    fn check_stmt(mut checker TypeChecker, stmt Stmt) Result<Type, str> {
        match stmt {
            Stmt::Expr { expr } => checker.check_expr(expr),
            Stmt::Let { name, type, init } => checker.check(Stmt::Let {
                name: name,
                type: type,
                init: init
            }),
            Stmt::If { cond, then_block, else_block } => {
                let cond_type = checker.check_expr(cond)?;
                checker.unify(cond_type, Type::Bool)?;
                checker.check_stmt(*then_block)?;
                if else_block != nil {
                    checker.check_stmt(*else_block.unwrap())?;
                }
                Ok(Type::Void)
            }
            _ => Ok(Type::Void)
        }
    }

    fn check_expr(mut checker TypeChecker, expr Expr) Result<Type, str> {
        match expr {
            Expr::Int { type } => Ok(type),
            Expr::Float { type } => Ok(type),
            Expr::Bool { } => Ok(Type::Bool),
            Expr::Nil => Ok(Type::Nil),
            Expr::Ident { name, symbol } => {
                let sym = checker.symbols.lookup(name)
                if sym == nil {
                    return Result::Err(f"undefined variable: \(name)")
                }
                Ok(sym.unwrap().type)
            }
            Expr::Binary { op, left, right, type } => {
                let left_ty = checker.check_expr(*left)?;
                let right_ty = checker.check_expr(*right)?;
                checker.unify(left_ty, right_ty)?;
                Ok(type)
            }
            Expr::Call { func, args } => {
                let func_type = checker.check_expr(*func)?;
                // TODO: Check function signature
                Ok(func_type)
            }
            _ => Ok(Type::Unknown)
        }
    }

    fn unify(mut checker TypeChecker, ty1 Type, ty2 Type) Result<Type, str> {
        if ty1 == ty2 {
            return Ok(ty1)
        }

        // Type coercion rules
        if checker.can_coerce(ty1, ty2) {
            return Ok(ty2)
        }

        checker.errors.add(ErrorLevel::Error,
            f"type mismatch: cannot unify \(ty1) with \(ty2)", Pos::new())
        Result::Err("type mismatch")
    }

    fn can_coerce(checker TypeChecker, from Type, to Type) bool {
        match (from, to) {
            (Type::I8, Type::I16) => true,
            (Type::I8, Type::I32) => true,
            (Type::I8, Type::I64) => true,
            (Type::I16, Type::I32) => true,
            (Type::I16, Type::I64) => true,
            (Type::I32, Type::I64) => true,
            (Type::Float, Type::Double) => true,
            _ => false
        }
    }
}
```

**Testing:**
```auto
// tests/type_check/test_type_check.at
fn test_type_check_int() {
    let mut checker = TypeChecker::new()
    let expr = Expr::Int { value: 42, type: Type::I32 }
    let ty = checker.check_expr(expr).unwrap()
    assert(ty == Type::I32)
}

fn test_type_check_binary() {
    let mut checker = TypeChecker::new()
    let left = Expr::Int { value: 1, type: Type::I32 }
    let right = Expr::Int { value: 2, type: Type::I32 }
    let expr = Expr::Binary {
        op: Op::Add,
        left: Box::new(left),
        right: Box::new(right),
        type: Type::I32
    }
    let ty = checker.check_expr(expr).unwrap()
    assert(ty == Type::I32)
}

fn test_type_check_mismatch() {
    let mut checker = TypeChecker::new()
    let left = Expr::Int { value: 1, type: Type::I32 }
    let right = Expr::Bool { value: true }
    let expr = Expr::Binary {
        op: Op::Add,
        left: Box::new(left),
        right: Box::new(right),
        type: Type::I32
    }
    let result = checker.check_expr(expr)
    assert(result.is_err())
}
```

**Success Criteria:**
- Type checker catches all type errors from Rust implementation
- 100+ test cases passing
- Unification algorithm correct
- Error messages clear

---

### Phase 7: C Transpiler (8-10 weeks)

**Objective:** Generate C code from validated AST.

**Dependencies:** Phase 6 (type-checked AST)

**Reference:** [trans/c.rs](../../crates/auto-lang/src/trans/c.rs:1) (2,505 lines)

#### 7.1 C Transpiler

**File:** `auto/compiler/transpiler.at`

```auto
use lib/ast: *
use lib/symbol: SymbolTable
use stdlib/string: builder

type CTranspiler {
    symbols SymbolTable
    indent uint
    headers HashSet<str>

    fn new() CTranspiler {
        CTranspiler {
            symbols: SymbolTable::new(),
            indent: 0,
            headers: HashSet::new()
        }
    }

    fn transpile(mut trans CTranspiler, stmt Stmt) str {
        let mut sb = StringBuilder::new(4096)
        trans.transpile_stmt(mut sb, stmt)
        StringBuilder_build(sb)
    }

    fn transpile_stmt(mut trans CTranspiler, mut sb StringBuilder, stmt Stmt) {
        match stmt {
            Stmt::Let { name, type, init } => {
                trans.print_indent(mut sb)
                let c_type = trans.type_to_c(type.unwrap_or(Type::I32))
                StringBuilder_append(mut sb, c_type)
                StringBuilder_append(mut sb, " ")
                StringBuilder_append(mut sb, name)

                if init != nil {
                    StringBuilder_append(mut sb, " = ")
                    trans.transpile_expr(mut sb, init.unwrap())
                }
                StringBuilder_append(mut sb, ";\n")
            }
            Stmt::Fn { name, params, ret_type, body } => {
                trans.transpile_fn(mut sb, name, params, ret_type, *body)
            }
            Stmt::If { cond, then_block, else_block } => {
                trans.print_indent(mut sb)
                StringBuilder_append(mut sb, "if (")
                trans.transpile_expr(mut sb, cond)
                StringBuilder_append(mut sb, ") ")
                trans.transpile_stmt(mut sb, *then_block)

                if else_block != nil {
                    trans.print_indent(mut sb)
                    StringBuilder_append(mut sb, "else ")
                    trans.transpile_stmt(mut sb, *else_block.unwrap())
                }
            }
            Stmt::Expr { expr } => {
                trans.print_indent(mut sb)
                trans.transpile_expr(mut sb, expr)
                StringBuilder_append(mut sb, ";\n")
            }
            _ => {}
        }
    }

    fn transpile_fn(mut trans CTranspiler, mut sb StringBuilder,
                     name str, params []Param, ret_type Type, body Stmt) {
        // Return type
        let c_ret_type = trans.type_to_c(ret_type)
        StringBuilder_append(mut sb, c_ret_type)
        StringBuilder_append(mut sb, " ")
        StringBuilder_append(mut sb, name)
        StringBuilder_append(mut sb, "(")

        // Parameters
        for i in 0..params.len() {
            if i > 0 {
                StringBuilder_append(mut sb, ", ")
            }
            let param = params[i]
            let c_type = trans.type_to_c(param.type)
            StringBuilder_append(mut sb, c_type)
            StringBuilder_append(mut sb, " ")
            StringBuilder_append(mut sb, param.name)
        }

        StringBuilder_append(mut sb, ") ")

        // Body
        trans.indent = trans.indent + 1
        trans.transpile_stmt(mut sb, body)
        trans.indent = trans.indent - 1
    }

    fn transpile_expr(mut trans CTranspiler, mut sb StringBuilder, expr Expr) {
        match expr {
            Expr::Int { value } => {
                StringBuilder_append_int(mut sb, value)
            }
            Expr::Float { value } => {
                StringBuilder_append(mut sb, f"\(value)")
            }
            Expr::Str { value } => {
                StringBuilder_append(mut sb, "\"")
                StringBuilder_append(mut sb, value)
                StringBuilder_append(mut sb, "\"")
            }
            Expr::Bool { value } => {
                StringBuilder_append(mut sb, if value { "true" } else { "false" })
            }
            Expr::Nil => {
                StringBuilder_append(mut sb, "NULL")
            }
            Expr::Ident { name } => {
                StringBuilder_append(mut sb, name)
            }
            Expr::Binary { op, left, right } => {
                StringBuilder_append(mut sb, "(")
                trans.transpile_expr(mut sb, *left)
                StringBuilder_append(mut sb, " ")
                StringBuilder_append(mut sb, trans.op_to_c(op))
                StringBuilder_append(mut sb, " ")
                trans.transpile_expr(mut sb, *right)
                StringBuilder_append(mut sb, ")")
            }
            Expr::Call { func, args } => {
                trans.transpile_expr(mut sb, *func)
                StringBuilder_append(mut sb, "(")
                for i in 0..args.len() {
                    if i > 0 {
                        StringBuilder_append(mut sb, ", ")
                    }
                    trans.transpile_expr(mut sb, args[i])
                }
                StringBuilder_append(mut sb, ")")
            }
            _ => {}
        }
    }

    fn type_to_c(trans CTranspiler, ty Type) str {
        match ty {
            Type::Void => "void",
            Type::I8 => "int8_t",
            Type::I16 => "int16_t",
            Type::I32 => "int32_t",
            Type::I64 => "int64_t",
            Type::U8 => "uint8_t",
            Type::U16 => "uint16_t",
            Type::U32 => "uint32_t",
            Type::U64 => "uint64_t",
            Type::Float => "float",
            Type::Double => "double",
            Type::Bool => "bool",
            Type::Str => "const char*",
            Type::Named { name } => name,
            _ => "/* UNKNOWN */"
        }
    }

    fn op_to_c(trans CTranspiler, op Op) str {
        match op {
            Op::Add => "+",
            Op::Sub => "-",
            Op::Mul => "*",
            Op::Div => "/",
            Op::Eq => "==",
            Op::Neq => "!=",
            Op::Lt => "<",
            Op::Gt => ">",
            Op::Le => "<=",
            Op::Ge => ">=",
            Op::And => "&&",
            Op::Or => "||",
            Op::Not => "!",
            Op::Assign => "=",
        }
    }

    fn print_indent(trans CTranspiler, mut sb StringBuilder) {
        for _ in 0..trans.indent {
            StringBuilder_append(mut sb, "    ")
        }
    }
}
```

**Testing:**
```auto
// tests/transpiler/test_transpiler.at
fn test_transpile_let() {
    let mut trans = CTranspiler::new()
    let stmt = Stmt::Let {
        name: "x",
        type: Some(Type::I32),
        init: Some(Expr::Int { value: 42, type: Type::I32 })
    }
    let c_code = trans.transpile(stmt)
    assert(c_code.contains("int32_t x = 42"))
}

fn test_transpile_function() {
    let mut trans = CTranspiler::new()
    let stmt = Stmt::Fn {
        name: "main",
        params: [],
        ret_type: Type::Void,
        body: Box::new(Stmt::Expr { expr: Expr::Int { value: 0, type: Type::I32 }})
    }
    let c_code = trans.transpile(stmt)
    assert(c_code.contains("void main()"))
}
```

**Success Criteria:**
- Generate C for all a2c test cases
- Generated C compiles with gcc/clang
- 100+ test cases passing
- Generated executables run correctly

---

### Phase 8: Compiler Driver & Bootstrap (4-6 weeks)

**Objective:** Complete self-hosting pipeline.

**Dependencies:** All previous phases

#### 8.1 Compiler Driver

**File:** `auto/compiler.at`

```auto
use compiler/lexer: Lexer
use compiler/parser: Parser
use lib/type_check: TypeChecker
use compiler/transpiler: CTranspiler
use stdlib/result: Result
use stdlib/io: {read_file, write_file}
use stdlib/sys: args

type Compiler {
    files []str
    errors ErrorCollector

    fn new() Compiler {
        Compiler {
            files: [],
            errors: ErrorCollector::new()
        }
    }

    fn compile(mut compiler Compiler, source_path str) Result<str, str> {
        // 1. Read source
        let source = read_file(source_path)?
        print("[lex] \(source_path)")

        // 2. Lex
        let lexer = Lexer::new(source)
        print("[parse] parsing...")

        // 3. Parse
        let mut parser = Parser::new(source)
        let ast = parser.parse()?
        if parser.errors.has_errors() {
            return Result::Err("parse errors")
        }
        print("[parse] OK")

        // 4. Type check
        print("[type] checking...")
        let mut checker = TypeChecker::new()
        checker.check(ast)?;
        if checker.errors.has_errors() {
            return Result::Err("type errors")
        }
        print("[type] OK")

        // 5. Generate C
        print("[trans] generating C...")
        let mut trans = CTranspiler::new()
        let c_code = trans.transpile(ast)
        print("[trans] OK")

        // 6. Write C file
        let c_path = source_path + ".c"
        write_file(c_path, c_code)
        print(f"[write] \(c_path)")

        Ok(c_path)
    }

    fn compile_to_executable(compiler Compiler, source_path str) Result<str, str> {
        let c_path = compiler.compile(source_path)?

        // 7. Call C compiler
        let exe_path = source_path + ".exe"
        print(f"[gcc] \(c_path) -> \(exe_path)")

        // TODO: Call gcc via system call
        // sys::run(f"gcc {c_path} -o {exe_path}")

        Ok(exe_path)
    }
}

fn main() {
    let args_count = args_count()
    if args_count < 2 {
        print("Usage: auto <input.at>")
        return
    }

    let input_path = args_get(1)
    let mut compiler = Compiler::new()

    let result = compiler.compile_to_executable(input_path)

    match result {
        Ok(exe_path) => {
            print(f"[success] {exe_path}")
        }
        Err(msg) => {
            print(f"[error] {msg}")
            sys::exit(1)
        }
    }
}
```

**Testing:**
```auto
// tests/integration/test_compiler.at
fn test_hello_world() {
    let source = `
        fn main() {
            print("hello, world!")
        }
    `

    let mut compiler = Compiler::new()
    let result = compiler.compile_to_executable("hello.at")

    match result {
        Ok(exe_path) => {
            // Run executable and check output
            assert(true)
        }
        Err(msg) => {
            assert(false, f"compilation failed: \(msg)")
        }
    }
}

fn test_self_compile() {
    // Bootstrap test: compile compiler with itself
    let mut compiler = Compiler::new()
    let result = compiler.compile("auto/compiler.at")

    match result {
        Ok(c_path) => {
            print(f"[bootstrap] {c_path}")
            assert(true)
        }
        Err(msg) => {
            assert(false, f"self-compilation failed: {msg}")
        }
    }
}
```

**Success Criteria:**
- Compiler compiles itself successfully
- Generated C code compiles and runs
- Performance within 10x of Rust compiler
- Auto-man integration working

---

## 5. Integration with Auto-Man

### 5.1 Build Configuration

**File:** `auto-man.yaml`

```yaml
name: "auto-compiler"
version: "0.1.0"
description: "Self-hosted AutoLang compiler"

dependencies:
  - url: "https://gitee.com/auto-stack/stdlib-c"
    version: "latest"

build:
  type: "auto-to-c"
  sources:
    - "auto/lib/*.at"
    - "auto/compiler/*.at"
    - "auto/*.at"

  output:
    binary: "auto-compiler"
    c_dir: "build/c"

  toolchain:
    default: "gcc"
    options:
      - "-Wall"
      - "-Wextra"
      - "-O2"

test:
  - name: "lexer-tests"
    sources: ["tests/lexer/*.at"]
  - name: "parser-tests"
    sources: ["tests/parser/*.at"]
  - name: "integration-tests"
    sources: ["tests/integration/*.at"]
```

### 5.2 Build Workflow

```bash
# Build self-hosted compiler
auto-man build

# Run tests
auto-man test

# Install system-wide
auto-man install

# Use self-hosted compiler to compile programs
auto-compiler hello.at -o hello
```

---

## 6. Success Criteria

### Phase 1-2: Token System & Lexer (8-9 weeks)
- [ ] All 50+ token kinds defined
- [ ] Lexer tokenizes all a2c test cases
- [ ] Position tracking accurate
- [ ] 50+ unit tests passing

### Phase 3-4: Symbol Table & AST (9-11 weeks)
- [ ] Hierarchical scopes working
- [ ] All AST node types defined
- [ ] Builder methods functional
- [ ] 100+ unit tests passing

### Phase 5: Parser (8-10 weeks)
- [ ] Parse all a2c/a2r test cases
- [ ] 100+ test cases passing
- [ ] Error recovery working
- [ ] Symbol table integration

### Phase 6: Type Checker (6-8 weeks)
- [ ] Type inference working
- [ ] Unification algorithm correct
- [ ] 100+ test cases passing
- [ ] Error messages clear

### Phase 7: C Transpiler (8-10 weeks)
- [ ] Generate C for all tests
- [ ] Generated C compiles without warnings
- [ ] Generated executables run correctly
- [ ] 100+ test cases passing

### Phase 8: Bootstrap (4-6 weeks)
- [ ] Compiler compiles itself successfully
- [ ] Auto-man integration working
- [ ] Performance within 10x of Rust compiler
- [ ] Zero memory leaks (valgrind clean)

### Final Milestone
- [ ] Self-hosted compiler can compile full AutoLang stdlib
- [ ] Can maintain and enhance itself without Rust dependency
- [ ] Comprehensive documentation complete
- [ ] Community onboarding materials ready

---

## 7. Timeline Summary

| Phase | Duration | Complexity | Deliverable |
|-------|----------|------------|-------------|
| 1. Token System | 3 weeks | Medium | Complete token types |
| 2. Lexer | 5-6 weeks | Medium | Full tokenizer |
| 3. Symbol Table | 4-5 weeks | High | Hierarchical scopes |
| 4. AST | 5-6 weeks | High | AST representation |
| 5. Parser | 8-10 weeks | Very High | Recursive descent parser |
| 6. Type Checker | 6-8 weeks | Very High | Type inference |
| 7. C Transpiler | 8-10 weeks | Very High | Code generation |
| 8. Bootstrap | 4-6 weeks | High | Self-hosting |

**Total: 43-62 weeks (10-15 months)**

**Critical Path:** Phase 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 (must be sequential)

**Parallel Opportunities:**
- Phase 3 (Symbol Table) can overlap with Phase 2 (Lexer)
- Phase 4 (AST) can start during Phase 3

---

## 8. Risks and Mitigations

### Risk 1: Timeline Slippage
**Risk:** Complex phases take longer than estimated

**Mitigation:**
- Buffer time in estimates
- Can ship minimal viable compiler first
- Incremental feature additions

### Risk 2: Performance Issues
**Risk:** Self-hosted compiler much slower than Rust

**Mitigation:**
- Profile hot paths early
- Use C stdlib for performance-critical operations
- Accept 10x slower initially (still usable)
- Optimize after correctness

### Risk 3: Memory Safety
**Risk:** C code generation has memory bugs

**Mitigation:**
- Extensive valgrind testing
- Use arena allocation (reduces fragmentation)
- Reference counting via auto-val
- Sanitizers in CI

### Risk 4: Feature Parity
**Risk:** Self-hosted compiler lacks features of Rust version

**Mitigation:**
- Start with minimal feature set
- Add features incrementally
- Use Rust compiler for complex programs initially
- Gradual migration path

---

## 9. Next Steps

### Immediate Actions (Week 1-4)
1. **Complete Plan 027** - Build stdlib foundation first
2. **Set up auto/ directory structure**
3. **Implement Phase 1** - Token system
4. **Create test infrastructure**

### First Quarter Goals
- Complete Phases 1-2 (Token system + Lexer)
- Set up CI/CD pipeline
- Have working lexer for all test cases

### First Year Goals
- Complete all phases
- Achieve self-hosting milestone
- Release self-hosted compiler v0.1.0

---

## 10. Related Documentation

- **Plan 024**: Ownership-Based Memory System
- **Plan 025**: String Type Redesign
- **Plan 027**: Standard Library C Foundation
- [C Transpiler Documentation](../c-transpiler.md)
- [Auto-Man Documentation](https://gitee.com/auto-stack/auto-man)
- [Rust Implementation](../../crates/auto-lang/src/) - Reference for all phases

---

## 11. Conclusion

This plan provides a practical roadmap to self-hosting using C + Auto-Man. Key advantages:

1. **Faster development** - Auto-Man exists, no tooling to build
2. **Broader appeal** - More C developers than Rust
3. **Proven path** - Go, Swift started with C backends
4. **Embedded focus** - Aligns with Auto-Man's ecosystem

The 10-15 month investment yields a self-sustaining AutoLang ecosystem where the compiler can compile itself, reducing dependency on the Rust implementation while positioning AutoLang for broader adoption in embedded systems and beyond.
