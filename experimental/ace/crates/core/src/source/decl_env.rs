//! PLAN-929（T-08）：声明面 CheckedEnv 持久化/恢复。
//!
//! §5.3/§5.6：恢复后下一次 body 编辑禁止整源 parse/resolve/重建环境；若需
//! 持久化**声明环境**，则版本化 required 边+恢复顺序+预算由本包给出。
//! 本模块把 `CheckedEnv` 的声明面（签名/参数 mode/receiver/记录形状/
//! VT 池）编为**确定性无指针清单**（`core-decl-env/1`）；恢复侧从清单装配
//! **stub-body** 环境——`recheck_swap` 对被编辑体自带解析事务（§828 单体
//! 事务合同），未改函数的 body 从不被读取，因此 stub 是诚实表示而非简化。

use super::ast;
use super::resolve::{
    checked_env_from_parts_profile, CheckedEnv, ClosureFnType, ResolvedField, ResolvedFn,
    ResolvedMethod, ResolvedRecord, ResolvedVariant,
};
use crate::atom_text::Span;
use std::collections::BTreeMap;

pub const DECL_ENV_SCHEMA: &str = "core-decl-env/1";

/// 恢复装配的受控拒绝（使用前；V-05 面）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeclEnvError {
    Schema(String),
    Profile {
        found: String,
        expected: &'static str,
    },
    Caps {
        found: String,
        expected: String,
    },
    SourceHash {
        found: String,
        expected: String,
    },
    Malformed(String),
    IndexOutOfRange {
        what: &'static str,
        index: usize,
        len: usize,
    },
    FnNameMismatch {
        expected: String,
        found: String,
    },
}

/// 声明面清单（owned；编解码的镜像结构）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeclEnvDump {
    pub profile: String,
    pub caps_version: String,
    pub source_sha256: String,
    pub fns: Vec<DeclFnStub>,
    pub records: Vec<DeclRecordStub>,
    pub lists: Vec<String>,
    pub variants: Vec<DeclVariantStub>,
    pub function_types: Vec<DeclFnTypeStub>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeclFnStub {
    pub name: String,
    pub name_span: (usize, usize),
    pub item_span: (usize, usize),
    pub param_names: Vec<String>,
    pub params: Vec<String>,
    pub param_modes: Vec<bool>,
    pub result: String,
    pub receiver: String,
    pub owner_rec: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeclRecordStub {
    pub name: String,
    pub name_span: (usize, usize),
    pub fields: Vec<(String, String, (usize, usize))>,
    pub methods: Vec<(String, bool, usize, (usize, usize))>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeclVariantStub {
    pub name: String,
    pub name_span: (usize, usize),
    pub arms: Vec<(String, Option<String>)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeclFnTypeStub {
    pub params: Vec<String>,
    pub result: String,
}

fn vt_token(v: &super::resolve::VT) -> String {
    use super::resolve::VT;
    match v {
        VT::I32 => "i32".into(),
        VT::Bool => "bool".into(),
        VT::Str => "str".into(),
        VT::StrOwned => "Str".into(),
        VT::Char => "char".into(),
        VT::Unit => "unit".into(),
        VT::Rec(i) => format!("rec:{i}"),
        VT::List(i) => format!("list:{i}"),
        VT::Variant(i) => format!("var:{i}"),
        VT::Function(i) => format!("fn:{i}"),
    }
}

fn vt_parse(tok: &str) -> Result<super::resolve::VT, DeclEnvError> {
    use super::resolve::VT;
    let idx = |s: &str, what: &'static str| -> Result<usize, DeclEnvError> {
        s.parse::<usize>()
            .map_err(|_| DeclEnvError::Malformed(format!("bad {what} index {s:?}")))
    };
    Ok(match tok {
        "i32" => VT::I32,
        "bool" => VT::Bool,
        "str" => VT::Str,
        "Str" => VT::StrOwned,
        "char" => VT::Char,
        "unit" => VT::Unit,
        _ => {
            if let Some(s) = tok.strip_prefix("rec:") {
                VT::Rec(idx(s, "rec")?)
            } else if let Some(s) = tok.strip_prefix("list:") {
                VT::List(idx(s, "list")?)
            } else if let Some(s) = tok.strip_prefix("var:") {
                VT::Variant(idx(s, "var")?)
            } else if let Some(s) = tok.strip_prefix("fn:") {
                VT::Function(idx(s, "fn")?)
            } else {
                return Err(DeclEnvError::Malformed(format!("unknown VT token {tok:?}")));
            }
        }
    })
}

fn receiver_token(r: ast::Receiver) -> &'static str {
    match r {
        ast::Receiver::Free => "free",
        ast::Receiver::Static => "static",
        ast::Receiver::View => "view",
    }
}

fn receiver_parse(t: &str) -> Result<ast::Receiver, DeclEnvError> {
    Ok(match t {
        "free" => ast::Receiver::Free,
        "static" => ast::Receiver::Static,
        "view" => ast::Receiver::View,
        _ => return Err(DeclEnvError::Malformed(format!("unknown receiver {t:?}"))),
    })
}

/// 从现有环境提取声明面清单（cold init 时一次；body/locals 不入清单）。
pub fn decl_env_dump(env: &CheckedEnv, caps_version: &str, source_sha256_hex: &str) -> DeclEnvDump {
    let p = &env.program;
    DeclEnvDump {
        profile: super::cst::profile_slug(env.profile).to_string(),
        caps_version: caps_version.to_string(),
        source_sha256: source_sha256_hex.to_string(),
        fns: p
            .fns
            .iter()
            .map(|f| DeclFnStub {
                name: f.name.clone(),
                name_span: (f.name_span.start, f.name_span.end),
                item_span: (f.item_span.start, f.item_span.end),
                param_names: f.param_names.clone(),
                params: f.params.iter().map(vt_token).collect(),
                param_modes: f.param_modes.clone(),
                result: vt_token(&f.result),
                receiver: receiver_token(f.receiver).to_string(),
                owner_rec: f.owner_rec,
            })
            .collect(),
        records: p
            .records
            .iter()
            .map(|r| DeclRecordStub {
                name: r.name.clone(),
                name_span: (r.name_span.start, r.name_span.end),
                fields: r
                    .fields
                    .iter()
                    .map(|fl| {
                        (
                            fl.name.clone(),
                            vt_token(&fl.vt),
                            (fl.span.start, fl.span.end),
                        )
                    })
                    .collect(),
                methods: r
                    .methods
                    .iter()
                    .map(|m| {
                        (
                            m.name.clone(),
                            m.is_static,
                            m.fn_index,
                            (m.span.start, m.span.end),
                        )
                    })
                    .collect(),
            })
            .collect(),
        lists: p.lists.iter().map(vt_token).collect(),
        variants: p
            .variants
            .iter()
            .map(|v| DeclVariantStub {
                name: v.name.clone(),
                name_span: (v.name_span.start, v.name_span.end),
                arms: v
                    .arms
                    .iter()
                    .map(|(n, t)| (n.clone(), t.as_ref().map(vt_token)))
                    .collect(),
            })
            .collect(),
        function_types: p
            .function_types
            .iter()
            .map(|t| DeclFnTypeStub {
                params: t.params.iter().map(vt_token).collect(),
                result: vt_token(&t.result),
            })
            .collect(),
    }
}

/// 确定性行式编码（无空格 token；schema 首行——与清单 codec 同纪律）。
pub fn encode_decl_env(d: &DeclEnvDump) -> String {
    let mut out = String::new();
    out.push_str(DECL_ENV_SCHEMA);
    out.push('\n');
    out.push_str(&format!("profile {}\n", d.profile));
    out.push_str(&format!("caps {}\n", d.caps_version));
    out.push_str(&format!("source {}\n", d.source_sha256));
    for f in &d.fns {
        out.push_str(&format!(
            "fn {} {} {} {} {} {} {} {} {}\n",
            f.name,
            f.name_span.0,
            f.name_span.1,
            f.item_span.0,
            f.item_span.1,
            f.param_names.join(","),
            f.params.join(","),
            f.param_modes
                .iter()
                .map(|b| if *b { "1" } else { "0" })
                .collect::<String>(),
            f.result
        ));
        out.push_str(&format!(
            "fnx {} {}\n",
            f.receiver,
            f.owner_rec.map(|i| i.to_string()).unwrap_or("-".into())
        ));
    }
    for r in &d.records {
        out.push_str(&format!(
            "rec {} {} {}\n",
            r.name, r.name_span.0, r.name_span.1
        ));
        for (n, vt, (s, e)) in &r.fields {
            out.push_str(&format!("fld {} {} {} {}\n", n, vt, s, e));
        }
        for (n, st, fi, (s, e)) in &r.methods {
            out.push_str(&format!(
                "mth {} {} {} {} {}\n",
                n,
                if *st { 1 } else { 0 },
                fi,
                s,
                e
            ));
        }
    }
    for (i, l) in d.lists.iter().enumerate() {
        out.push_str(&format!("list {i} {l}\n"));
    }
    for v in &d.variants {
        out.push_str(&format!(
            "var {} {} {}\n",
            v.name, v.name_span.0, v.name_span.1
        ));
        for (n, t) in &v.arms {
            out.push_str(&format!("arm {} {}\n", n, t.clone().unwrap_or("-".into())));
        }
    }
    for (i, t) in d.function_types.iter().enumerate() {
        out.push_str(&format!("fty {i} {} {}\n", t.params.join(","), t.result));
    }
    out
}

/// 消费前校验解码（未知行/坏 token/坏数字一律 Err）。
pub fn decode_decl_env(text: &str) -> Result<DeclEnvDump, DeclEnvError> {
    let bad = |m: String| Err(DeclEnvError::Malformed(m));
    let body = text.strip_suffix('\n').unwrap_or(text);
    if body.contains("\n\n") || body.ends_with('\n') {
        return bad("blank line or trailing garbage".into());
    }
    let mut lines = body.split('\n');
    let schema = lines.next().unwrap_or("");
    if schema != DECL_ENV_SCHEMA {
        return Err(DeclEnvError::Schema(schema.to_string()));
    }
    fn header(lines: &mut dyn Iterator<Item = &str>, key: &str) -> Result<String, DeclEnvError> {
        let ln = lines
            .next()
            .ok_or_else(|| DeclEnvError::Malformed(format!("truncated at {key}")))?;
        let (k, v) = ln
            .split_once(' ')
            .ok_or_else(|| DeclEnvError::Malformed(format!("malformed {key} header")))?;
        if k != key || v.is_empty() {
            return Err(DeclEnvError::Malformed(format!("malformed {key} header")));
        }
        Ok(v.to_string())
    }
    let profile = header(&mut lines, "profile")?;
    let caps_version = header(&mut lines, "caps")?;
    let source_sha256 = header(&mut lines, "source")?;
    let mut d = DeclEnvDump {
        profile,
        caps_version,
        source_sha256,
        ..Default::default()
    };
    // 两次走：fn 行后随 fnx（receiver/owner）。
    let mut pending_fn: Option<DeclFnStub> = None;
    let flush =
        |pending: &mut Option<DeclFnStub>, d: &mut DeclEnvDump| -> Result<(), DeclEnvError> {
            if let Some(f) = pending.take() {
                d.fns.push(f);
            }
            Ok(())
        };
    let mut cur_rec: Option<DeclRecordStub> = None;
    let mut cur_var: Option<DeclVariantStub> = None;
    for ln in lines {
        let parts: Vec<&str> = ln.split(' ').collect();
        let kind = parts[0];
        match kind {
            "fn" => {
                flush(&mut pending_fn, &mut d)?;
                if parts.len() != 10 {
                    return bad(format!("malformed fn record: {ln:?}"));
                }
                let name_span = (
                    parts[2]
                        .parse::<usize>()
                        .map_err(|_| DeclEnvError::Malformed("bad name span".into()))?,
                    parts[3]
                        .parse::<usize>()
                        .map_err(|_| DeclEnvError::Malformed("bad name span".into()))?,
                );
                let item_span = (
                    parts[4]
                        .parse::<usize>()
                        .map_err(|_| DeclEnvError::Malformed("bad item span".into()))?,
                    parts[5]
                        .parse::<usize>()
                        .map_err(|_| DeclEnvError::Malformed("bad item span".into()))?,
                );
                let param_names: Vec<String> = if parts[6].is_empty() {
                    vec![]
                } else {
                    parts[6].split(',').map(|s| s.to_string()).collect()
                };
                let params: Vec<String> = if parts[7].is_empty() {
                    vec![]
                } else {
                    parts[7].split(',').map(|s| s.to_string()).collect()
                };
                let modes: Vec<bool> = parts[8].chars().map(|c| c == '1').collect();
                if param_names.len() != params.len() || param_names.len() != modes.len() {
                    return bad("fn param arity mismatch".into());
                }
                pending_fn = Some(DeclFnStub {
                    name: parts[1].to_string(),
                    name_span,
                    item_span,
                    param_names,
                    params,
                    param_modes: modes,
                    result: parts[9].to_string(),
                    receiver: "free".to_string(),
                    owner_rec: None,
                });
            }
            "fnx" => {
                if parts.len() != 3 {
                    return bad(format!("malformed fnx record: {ln:?}"));
                }
                let f = pending_fn
                    .as_mut()
                    .ok_or_else(|| DeclEnvError::Malformed("fnx without fn".into()))?;
                f.receiver = parts[1].to_string();
                receiver_parse(parts[1])?;
                f.owner_rec = if parts[2] == "-" {
                    None
                } else {
                    Some(
                        parts[2]
                            .parse()
                            .map_err(|_| DeclEnvError::Malformed("bad owner rec".into()))?,
                    )
                };
                flush(&mut pending_fn, &mut d)?;
            }
            "rec" => {
                if let Some(r) = cur_rec.take() {
                    d.records.push(r.clone());
                }
                if let Some(v) = cur_var.take() {
                    d.variants.push(v);
                }
                if parts.len() != 4 {
                    return bad(format!("malformed rec record: {ln:?}"));
                }
                cur_rec = Some(DeclRecordStub {
                    name: parts[1].to_string(),
                    name_span: (
                        parts[2]
                            .parse()
                            .map_err(|_| DeclEnvError::Malformed("bad rec span".into()))?,
                        parts[3]
                            .parse()
                            .map_err(|_| DeclEnvError::Malformed("bad rec span".into()))?,
                    ),
                    fields: vec![],
                    methods: vec![],
                });
            }
            "fld" => {
                if parts.len() != 5 {
                    return bad(format!("malformed fld record: {ln:?}"));
                }
                let r = cur_rec
                    .as_mut()
                    .ok_or_else(|| DeclEnvError::Malformed("fld without rec".into()))?;
                r.fields.push((
                    parts[1].to_string(),
                    parts[2].to_string(),
                    (
                        parts[3]
                            .parse()
                            .map_err(|_| DeclEnvError::Malformed("bad fld span".into()))?,
                        parts[4]
                            .parse()
                            .map_err(|_| DeclEnvError::Malformed("bad fld span".into()))?,
                    ),
                ));
            }
            "mth" => {
                if parts.len() != 6 {
                    return bad(format!("malformed mth record: {ln:?}"));
                }
                let r = cur_rec
                    .as_mut()
                    .ok_or_else(|| DeclEnvError::Malformed("mth without rec".into()))?;
                r.methods.push((
                    parts[1].to_string(),
                    parts[2] == "1",
                    parts[3]
                        .parse()
                        .map_err(|_| DeclEnvError::Malformed("bad mth fn_index".into()))?,
                    (
                        parts[4]
                            .parse()
                            .map_err(|_| DeclEnvError::Malformed("bad mth span".into()))?,
                        parts[5]
                            .parse()
                            .map_err(|_| DeclEnvError::Malformed("bad mth span".into()))?,
                    ),
                ));
            }
            "list" => {
                if parts.len() != 3 {
                    return bad(format!("malformed list record: {ln:?}"));
                }
                d.lists.push(parts[2].to_string());
            }
            "var" => {
                if let Some(r) = cur_rec.take() {
                    d.records.push(r);
                }
                if let Some(v) = cur_var.take() {
                    d.variants.push(v);
                }
                if parts.len() != 4 {
                    return bad(format!("malformed var record: {ln:?}"));
                }
                cur_var = Some(DeclVariantStub {
                    name: parts[1].to_string(),
                    name_span: (
                        parts[2]
                            .parse()
                            .map_err(|_| DeclEnvError::Malformed("bad var span".into()))?,
                        parts[3]
                            .parse()
                            .map_err(|_| DeclEnvError::Malformed("bad var span".into()))?,
                    ),
                    arms: vec![],
                });
            }
            "arm" => {
                if parts.len() != 3 {
                    return bad(format!("malformed arm record: {ln:?}"));
                }
                let v = cur_var
                    .as_mut()
                    .ok_or_else(|| DeclEnvError::Malformed("arm without var".into()))?;
                v.arms.push((
                    parts[1].to_string(),
                    if parts[2] == "-" {
                        None
                    } else {
                        Some(parts[2].to_string())
                    },
                ));
            }
            "fty" => {
                if let Some(v) = cur_var.take() {
                    d.variants.push(v);
                }
                if parts.len() != 4 {
                    return bad(format!("malformed fty record: {ln:?}"));
                }
                let params: Vec<String> = if parts[2].is_empty() {
                    vec![]
                } else {
                    parts[2].split(',').map(|s| s.to_string()).collect()
                };
                d.function_types.push(DeclFnTypeStub {
                    params,
                    result: parts[3].to_string(),
                });
            }
            _ => return bad(format!("unknown record kind {kind:?}")),
        }
    }
    flush(&mut pending_fn, &mut d)?;
    if let Some(r) = cur_rec.take() {
        d.records.push(r);
    }
    if let Some(v) = cur_var.take() {
        d.variants.push(v);
    }
    Ok(d)
}

/// 从清单装配 stub-body `CheckedEnv`（不 parse/不 resolve——恢复 R 轴）。
/// 身份对账：profile/caps/source sha 与调用方期望一致；VT 池索引越界受控拒。
pub fn decl_env_restore(
    d: &DeclEnvDump,
    profile: super::SourceProfile,
    caps_version: &str,
    source_sha256_hex: &str,
) -> Result<CheckedEnv, DeclEnvError> {
    let expected_slug = super::cst::profile_slug(profile);
    if d.profile != expected_slug {
        return Err(DeclEnvError::Profile {
            found: d.profile.clone(),
            expected: expected_slug,
        });
    }
    if d.caps_version != caps_version {
        return Err(DeclEnvError::Caps {
            found: d.caps_version.clone(),
            expected: caps_version.to_string(),
        });
    }
    if d.source_sha256 != source_sha256_hex {
        return Err(DeclEnvError::SourceHash {
            found: d.source_sha256.clone(),
            expected: source_sha256_hex.to_string(),
        });
    }
    // VT 池先装配（索引边界在此判定）。
    let lists: Vec<super::resolve::VT> = d
        .lists
        .iter()
        .map(|t| vt_parse(t))
        .collect::<Result<_, _>>()?;
    let variants: Vec<ResolvedVariant> = d
        .variants
        .iter()
        .map(|v| -> Result<ResolvedVariant, DeclEnvError> {
            Ok(ResolvedVariant {
                name: v.name.clone(),
                name_span: Span {
                    start: v.name_span.0,
                    end: v.name_span.1,
                },
                arms: v
                    .arms
                    .iter()
                    .map(|(n, t)| {
                        Ok((
                            n.clone(),
                            match t {
                                Some(t) => Some(vt_parse(t)?),
                                None => None,
                            },
                        ))
                    })
                    .collect::<Result<_, DeclEnvError>>()?,
            })
        })
        .collect::<Result<_, _>>()?;
    let function_types: Vec<ClosureFnType> = d
        .function_types
        .iter()
        .map(|t| {
            Ok(ClosureFnType {
                params: t
                    .params
                    .iter()
                    .map(|s| vt_parse(s))
                    .collect::<Result<_, _>>()?,
                result: vt_parse(&t.result)?,
            })
        })
        .collect::<Result<_, _>>()?;
    let records: Vec<ResolvedRecord> = d
        .records
        .iter()
        .map(|r| -> Result<ResolvedRecord, DeclEnvError> {
            Ok(ResolvedRecord {
                name: r.name.clone(),
                name_span: Span {
                    start: r.name_span.0,
                    end: r.name_span.1,
                },
                fields: r
                    .fields
                    .iter()
                    .map(|(n, t, (s, e))| {
                        Ok(ResolvedField {
                            name: n.clone(),
                            vt: vt_parse(t)?,
                            span: Span { start: *s, end: *e },
                        })
                    })
                    .collect::<Result<_, DeclEnvError>>()?,
                methods: r
                    .methods
                    .iter()
                    .map(|(n, st, fi, (s, e))| ResolvedMethod {
                        name: n.clone(),
                        is_static: *st,
                        fn_index: *fi,
                        span: Span { start: *s, end: *e },
                    })
                    .collect(),
            })
        })
        .collect::<Result<_, _>>()?;
    // fn stub（body 空——未改函数 body 在暖路径永不被读；recheck 自带事务）。
    let fns: Vec<ResolvedFn> = d
        .fns
        .iter()
        .map(|f| {
            let params: Vec<super::resolve::VT> = f
                .params
                .iter()
                .map(|t| vt_parse(t))
                .collect::<Result<_, _>>()?;
            for vt in params.iter().chain(std::iter::once(&vt_parse(&f.result)?)) {
                check_pool_idx(
                    vt,
                    records.len(),
                    lists.len(),
                    variants.len(),
                    function_types.len(),
                )?;
            }
            if let Some(ri) = f.owner_rec {
                if ri >= records.len() {
                    return Err(DeclEnvError::IndexOutOfRange {
                        what: "rec",
                        index: ri,
                        len: records.len(),
                    });
                }
            }
            Ok(ResolvedFn {
                name: f.name.clone(),
                name_span: Span {
                    start: f.name_span.0,
                    end: f.name_span.1,
                },
                item_span: Span {
                    start: f.item_span.0,
                    end: f.item_span.1,
                },
                param_names: f.param_names.clone(),
                params,
                param_modes: f.param_modes.clone(),
                result: vt_parse(&f.result)?,
                locals: Vec::new(),
                body: super::resolve::RBlock {
                    stmts: Vec::new(),
                    span: Span {
                        start: f.item_span.0,
                        end: f.item_span.1,
                    },
                },
                loop_count: 0,
                used_intrinsics: Default::default(),
                receiver: receiver_parse(&f.receiver)?,
                owner_rec: f.owner_rec,
            })
        })
        .collect::<Result<_, _>>()?;
    for r in &records {
        for m in &r.methods {
            if m.fn_index >= fns.len() {
                return Err(DeclEnvError::IndexOutOfRange {
                    what: "fn",
                    index: m.fn_index,
                    len: fns.len(),
                });
            }
        }
    }
    let mut rec_index: BTreeMap<String, usize> = BTreeMap::new();
    for (i, r) in records.iter().enumerate() {
        rec_index.insert(r.name.clone(), i);
    }
    // sigs 键与 resolve_env 同约定：方法按 owner 限定（work_key——裸名
    // 方法不得与同名自由函数混键）。
    let mut sigs = BTreeMap::new();
    for (i, f) in fns.iter().enumerate() {
        sigs.insert(
            f.work_key(&records),
            super::resolve::fn_sig_from_parts(
                f.param_names.clone(),
                f.params.clone(),
                f.result.clone(),
                i,
            ),
        );
    }
    let program = super::resolve::ResolvedProgram {
        fns,
        records,
        lists,
        variants,
        function_types,
    };
    Ok(checked_env_from_parts_profile(
        program, rec_index, sigs, false, profile,
    ))
}

fn check_pool_idx(
    vt: &super::resolve::VT,
    nrec: usize,
    nlist: usize,
    nvar: usize,
    nfty: usize,
) -> Result<(), DeclEnvError> {
    use super::resolve::VT;
    match vt {
        VT::Rec(i) if *i >= nrec => Err(DeclEnvError::IndexOutOfRange {
            what: "rec",
            index: *i,
            len: nrec,
        }),
        VT::List(i) if *i >= nlist => Err(DeclEnvError::IndexOutOfRange {
            what: "list",
            index: *i,
            len: nlist,
        }),
        VT::Variant(i) if *i >= nvar => Err(DeclEnvError::IndexOutOfRange {
            what: "var",
            index: *i,
            len: nvar,
        }),
        VT::Function(i) if *i >= nfty => Err(DeclEnvError::IndexOutOfRange {
            what: "fn-type",
            index: *i,
            len: nfty,
        }),
        _ => Ok(()),
    }
}
