//! 最小语法程序（单元/闭环冒烟用；真实双 profile 由 ac-syntax-grammar 从 .at 编译）。
//!
//! mini：`api name(a: i32) -> i32;` / `note "text";` 声明序列，
//! 带引号字符串（转义）与可嵌套块注释——覆盖跨块 scanner 状态。

#![allow(dead_code)]

use crate::identity::package_identity;
use crate::program::*;

/// token 下标。
pub mod toks {
    pub const WS: u32 = 0;
    pub const LINE_COMMENT: u32 = 1;
    pub const BLOCK_COMMENT: u32 = 2;
    pub const API: u32 = 3;
    pub const NOTE: u32 = 4;
    pub const IDENT: u32 = 5;
    pub const INT: u32 = 6;
    pub const STRING: u32 = 7;
    pub const LPAREN: u32 = 8;
    pub const RPAREN: u32 = 9;
    pub const COLON: u32 = 10;
    pub const COMMA: u32 = 11;
    pub const ARROW: u32 = 12;
    pub const SEMI: u32 = 13;
    pub const I32: u32 = 14;
    pub const STAR: u32 = 15;
    pub const PLUS: u32 = 16;
    pub const LT: u32 = 17;
}
pub mod rules {
    pub const PROGRAM: u32 = 0;
    pub const DECL: u32 = 1;
    pub const API_DECL: u32 = 2;
    pub const NOTE_DECL: u32 = 3;
    pub const PARAMS: u32 = 4;
    pub const PARAM_TAIL: u32 = 5;
    pub const PARAM: u32 = 6;
    pub const TYPE: u32 = 7;
    pub const EXPR: u32 = 8;
    pub const ATOM_EXPR: u32 = 9;
}

fn cs(spec: &[(u8, u8)]) -> CharSet {
    CharSet::from_ranges(spec)
}

/// 手工构造的最小可执行程序（单 Pratt 表：`<`<`+`<`*` 左结合，复刻 core 序）。
pub fn mini_program() -> GrammarProgram {
    let mut rules_scan: Vec<ScanRule> = vec![
        ScanRule {
            mode: 0,
            matcher: Matcher::Charset {
                first: cs(&[(b' ', b' '), (b'\t', b'\t'), (b'\r', b'\r'), (b'\n', b'\n')]),
                cont: cs(&[(b' ', b' '), (b'\t', b'\t'), (b'\r', b'\r'), (b'\n', b'\n')]),
                min_len: 1,
            },
            token: toks::WS,
            trivia: true,
            next_mode: 0,
        },
        ScanRule {
            mode: 0,
            matcher: Matcher::LineComment("//".into()),
            token: toks::LINE_COMMENT,
            trivia: true,
            next_mode: 0,
        },
        ScanRule {
            mode: 0,
            matcher: Matcher::BlockComment {
                start: "/*".into(),
                end: "*/".into(),
                nested: true,
            },
            token: toks::BLOCK_COMMENT,
            trivia: true,
            next_mode: 0,
        },
    ];
    let literals: &[(u32, &str)] = &[
        (toks::API, "api"),
        (toks::NOTE, "note"),
        (toks::I32, "i32"),
        (toks::LPAREN, "("),
        (toks::RPAREN, ")"),
        (toks::COLON, ":"),
        (toks::COMMA, ","),
        (toks::ARROW, "->"),
        (toks::SEMI, ";"),
        (toks::STAR, "*"),
        (toks::PLUS, "+"),
        (toks::LT, "<"),
    ];
    for (tok, lit) in literals {
        rules_scan.push(ScanRule {
            mode: 0,
            matcher: Matcher::Literal((*lit).into()),
            token: *tok,
            trivia: false,
            next_mode: 0,
        });
    }
    rules_scan.push(ScanRule {
        mode: 0,
        matcher: Matcher::StringLit {
            quote: b'"',
            escape: true,
        },
        token: toks::STRING,
        trivia: false,
        next_mode: 0,
    });
    rules_scan.push(ScanRule {
        mode: 0,
        matcher: Matcher::Charset {
            first: cs(&[(b'a', b'z'), (b'A', b'Z'), (b'_', b'_')]),
            cont: cs(&[(b'a', b'z'), (b'A', b'Z'), (b'0', b'9'), (b'_', b'_')]),
            min_len: 1,
        },
        token: toks::IDENT,
        trivia: false,
        next_mode: 0,
    });
    rules_scan.push(ScanRule {
        mode: 0,
        matcher: Matcher::Charset {
            first: cs(&[(b'0', b'9')]),
            cont: cs(&[(b'0', b'9')]),
            min_len: 1,
        },
        token: toks::INT,
        trivia: false,
        next_mode: 0,
    });

    let step = |atom: Atom| Step {
        atom,
        field: None,
        repeat: Repeat::Once,
        same_line: false,
    };
    let stepf = |atom: Atom, f: u32| Step {
        atom,
        field: Some(f),
        repeat: Repeat::Once,
        same_line: false,
    };
    let rep = |atom: Atom| Step {
        atom,
        field: None,
        repeat: Repeat::ZeroOrMore,
        same_line: false,
    };
    #[allow(unused_variables)]
    let opt = |atom: Atom| Step {
        atom,
        field: None,
        repeat: Repeat::Optional,
        same_line: false,
    };
    let listp = |atom: Atom| Step {
        atom,
        field: None,
        repeat: Repeat::ListPlain,
        same_line: false,
    };

    let rs = |name: &str, alts: Vec<Alt>, recover: bool| RuleSpec {
        name: name.into(),
        alts,
        recover,
    };
    let grammar_rules = vec![
        // program := decl*（ListPlain：项间跳过 `;` 分隔）
        rs(
            "program",
            vec![Alt {
                steps: vec![listp(Atom::Rule(rules::DECL))],
            }],
            false,
        ),
        // decl := api-decl | note-decl
        rs(
            "decl",
            vec![
                Alt {
                    steps: vec![step(Atom::Rule(rules::API_DECL))],
                },
                Alt {
                    steps: vec![step(Atom::Rule(rules::NOTE_DECL))],
                },
            ],
            false,
        ),
        // api-decl := api ident ( params? ) -> type ;
        rs(
            "api-decl",
            vec![Alt {
                steps: vec![
                    stepf(Atom::Tok(toks::API), 0),
                    stepf(Atom::Tok(toks::IDENT), 1),
                    step(Atom::Tok(toks::LPAREN)),
                    Step {
                        atom: Atom::Rule(rules::PARAMS),
                        field: Some(2),
                        repeat: Repeat::Optional,
                        same_line: false,
                    },
                    step(Atom::Tok(toks::RPAREN)),
                    step(Atom::Tok(toks::ARROW)),
                    stepf(Atom::Rule(rules::TYPE), 3),
                    step(Atom::Tok(toks::SEMI)),
                ],
            }],
            true,
        ),
        // note-decl := note string ;
        rs(
            "note-decl",
            vec![Alt {
                steps: vec![
                    stepf(Atom::Tok(toks::NOTE), 0),
                    stepf(Atom::Tok(toks::STRING), 1),
                    step(Atom::Tok(toks::SEMI)),
                ],
            }],
            true,
        ),
        // params := param (, param)*
        rs(
            "params",
            vec![Alt {
                steps: vec![
                    step(Atom::Rule(rules::PARAM)),
                    rep(Atom::Rule(rules::PARAM_TAIL)),
                ],
            }],
            false,
        ),
        rs(
            "param-tail",
            vec![Alt {
                steps: vec![step(Atom::Tok(toks::COMMA)), step(Atom::Rule(rules::PARAM))],
            }],
            false,
        ),
        // param := ident : type
        rs(
            "param",
            vec![Alt {
                steps: vec![
                    stepf(Atom::Tok(toks::IDENT), 0),
                    step(Atom::Tok(toks::COLON)),
                    stepf(Atom::Rule(rules::TYPE), 1),
                ],
            }],
            false,
        ),
        // type := i32
        rs(
            "type",
            vec![Alt {
                steps: vec![step(Atom::Tok(toks::I32))],
            }],
            false,
        ),
        // expr := Pratt
        rs(
            "expr",
            vec![Alt {
                steps: vec![step(Atom::Pratt(0))],
            }],
            false,
        ),
        // atom-expr := int | ident | ( expr )
        rs(
            "atom-expr",
            vec![
                Alt {
                    steps: vec![step(Atom::Tok(toks::INT))],
                },
                Alt {
                    steps: vec![step(Atom::Tok(toks::IDENT))],
                },
                Alt {
                    steps: vec![
                        step(Atom::Tok(toks::LPAREN)),
                        step(Atom::Rule(rules::EXPR)),
                        step(Atom::Tok(toks::RPAREN)),
                    ],
                },
            ],
            false,
        ),
    ];

    let mut p = GrammarProgram {
        name: "mini-test".into(),
        revision: 1,
        schema_version: GRAMMAR_SCHEMA_VERSION,
        scanner: ScannerProgram {
            mode_names: vec!["code".into()],
            rules: rules_scan,
        },
        tokens: vec![
            TokenSpec { name: "ws".into() },
            TokenSpec {
                name: "line-comment".into(),
            },
            TokenSpec {
                name: "block-comment".into(),
            },
            TokenSpec { name: "api".into() },
            TokenSpec {
                name: "note".into(),
            },
            TokenSpec {
                name: "ident".into(),
            },
            TokenSpec { name: "int".into() },
            TokenSpec {
                name: "string".into(),
            },
            TokenSpec { name: "(".into() },
            TokenSpec { name: ")".into() },
            TokenSpec { name: ":".into() },
            TokenSpec { name: ",".into() },
            TokenSpec { name: "->".into() },
            TokenSpec { name: ";".into() },
            TokenSpec { name: "i32".into() },
            TokenSpec { name: "*".into() },
            TokenSpec { name: "+".into() },
            TokenSpec { name: "<".into() },
        ],
        rules: grammar_rules,
        field_names: vec!["kw".into(), "name".into(), "params".into(), "ty".into()],
        pratt: vec![PrattSpec {
            primary: rules::ATOM_EXPR,
            ops: vec![
                PrattOp {
                    op: toks::LT,
                    binding: 1,
                    assoc: Assoc::Left,
                },
                PrattOp {
                    op: toks::PLUS,
                    binding: 2,
                    assoc: Assoc::Left,
                },
                PrattOp {
                    op: toks::STAR,
                    binding: 3,
                    assoc: Assoc::Left,
                },
            ],
            group_open: Some(toks::LPAREN),
            group_close: Some(toks::RPAREN),
            call_paren: None,
            call_args: None,
        }],
        start: rules::PROGRAM,
        newline: NewlinePolicy {
            significant: false,
            stmt_end: vec![toks::SEMI],
            sep_consume: vec![toks::SEMI],
        },
        recovery: RecoverySpec {
            sync: vec![toks::SEMI],
        },
        outline: Some(OutlineSpec {
            decl_rule: "decl".into(),
            name_field: "name".into(),
            params_field: Some("params".into()),
        }),
        identity: crate::identity::PackageIdentity {
            name: String::new(),
            grammar_id: crate::identity::GrammarId(String::new()),
        },
    };
    // PARAM_TAIL 常量：rules vec 索引 = PARAMS(4)+1
    debug_assert_eq!(p.rules[5].name, "param-tail");
    debug_assert_eq!(p.rules[6].name, "param");
    let canonical = p.canonical_bytes_pub();
    let versions = [
        crate::GRAMMAR_SCHEMA_VERSION_STR,
        crate::RUNTIME_VERSION,
        crate::SCANNER_ABI_VERSION,
        crate::PARSER_ABI_VERSION,
    ];
    p.identity = package_identity(&p.name, &canonical, &versions);
    p
}

impl GrammarProgram {
    /// testkit 专用：identity 未填时也可编码（canonical 不含 identity）。
    fn canonical_bytes_pub(&self) -> Vec<u8> {
        crate::program_codec::encode_canonical(self)
    }
}
