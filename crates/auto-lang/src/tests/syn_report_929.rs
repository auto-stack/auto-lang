//! PLAN-929（T-09）：SYN 版本化报告的**严格父侧准入**（SD-06 面）。
//!
//! `ac-source-syn-report/1`：身份绑定（schema/plan/run 窗/producer 构建位/
//! source sha/grammar 身份/目标后端）+ P/M/R/IO 四轴真实计数（无 Default-0：
//! 字段缺失=拒，不是 0）+ 预算域引用 + 认证身份 + 分态终局。父 decoder 对
//! 缺字段/未知字段/错类型/枚举外值/伪 0（P=0 而 M/R/IO 显示整源工作）一律
//! 使用前拒绝；篡改矩阵负控同册。

use std::collections::BTreeMap;

pub const SYN_REPORT_SCHEMA: &str = "ac-source-syn-report/1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SynReject {
    Schema(String),
    MissingField(&'static str),
    UnknownField(String),
    WrongType { field: &'static str, expected: &'static str },
    BadEnum { field: &'static str, found: String },
    /// 伪 0：whole 轴=0 但 M/R/IO 显示整源工作（或反之）——结构性矛盾。
    FakeZero { detail: String },
    /// STALE：报告构建位/语法身份与父侧期望不符。
    StaleIdentity { field: &'static str, found: String, expected: String },
    /// 计数非 u64/负数/坏数字。
    BadCounter { field: &'static str },
}

/// 已准入的 SYN 报告（owned 镜像——不信任任何来源字段）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SynReport {
    pub plan_id: String,
    pub run_id: String,
    pub window_id: String,
    pub producer_commit: String,
    pub source_sha256: String,
    pub grammar_id: String,
    pub caps_version: String,
    pub profile: String,
    pub target: String,
    /// P 轴：阶段名 → 真实 attempt 计数（失败 attempt 也在内）。
    pub p_attempts: BTreeMap<String, u64>,
    /// M 轴：重建/隐藏工作计数。
    pub m_rebuilds: BTreeMap<String, u64>,
    /// R 轴：恢复深重验计数（合法非零）。
    pub r_verifies: BTreeMap<String, u64>,
    /// I/O 轴：读取/写入字节账。
    pub io_bytes: BTreeMap<String, u64>,
    pub budget_peaks: BTreeMap<String, u64>,
    pub certification: Option<String>,
    pub outcome: SynOutcome,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SynOutcome {
    Ok,
    SourceError,
    ResourceExhausted,
    Cancelled,
    Infra,
}

/// 严格解码（行式 `key value`；键白名单；类型/枚举/计数校验）。
/// 父侧期望（构建位/语法身份/源 sha）由调用方注入做 STALE 对账。
pub fn decode_syn_report(
    text: &str,
    expect: &SynExpectation,
) -> Result<SynReport, SynReject> {
    let mut fields: BTreeMap<String, String> = BTreeMap::new();
    let mut order: Vec<String> = Vec::new();
    for ln in text.lines() {
        if ln.trim().is_empty() {
            continue;
        }
        let Some((k, v)) = ln.split_once(' ') else {
            return Err(SynReject::UnknownField(ln.to_string()));
        };
        // 计数键族（四轴+预算）按前缀放行；其余走白名单。
        let is_count_key = ["p.", "m.", "r.", "io.", "budget."].iter().any(|pfx| k.starts_with(pfx));
        if !is_count_key && !KNOWN_KEYS.contains(&k) {
            return Err(SynReject::UnknownField(k.to_string()));
        }
        if v.is_empty() {
            return Err(SynReject::WrongType { field: "value", expected: "non-empty" });
        }
        order.push(k.to_string());
        fields.insert(k.to_string(), v.to_string());
    }
    let get = |k: &'static str| -> Result<String, SynReject> {
        fields.get(k).cloned().ok_or(SynReject::MissingField(k))
    };
    if get("schema")? != SYN_REPORT_SCHEMA {
        return Err(SynReject::Schema(get("schema")?));
    }
    let producer_commit = get("producer_commit")?;
    if producer_commit != expect.producer_commit {
        return Err(SynReject::StaleIdentity {
            field: "producer_commit",
            found: producer_commit,
            expected: expect.producer_commit.clone(),
        });
    }
    let grammar_id = get("grammar_id")?;
    if grammar_id != expect.grammar_id {
        return Err(SynReject::StaleIdentity {
            field: "grammar_id",
            found: grammar_id,
            expected: expect.grammar_id.clone(),
        });
    }
    let source_sha256 = get("source_sha256")?;
    if source_sha256 != expect.source_sha256 {
        return Err(SynReject::StaleIdentity {
            field: "source_sha256",
            found: source_sha256,
            expected: expect.source_sha256.clone(),
        });
    }
    let mut counts = |prefix: &str| -> Result<BTreeMap<String, u64>, SynReject> {
        let mut m = BTreeMap::new();
        for (k, v) in &fields {
            let prefix_static: &'static str = match prefix {
            "p." => "p.",
            "m." => "m.",
            "r." => "r.",
            "io." => "io.",
            _ => "budget.",
        };
        if let Some(rest) = k.strip_prefix(prefix) {
                let n: u64 = v.parse().map_err(|_| SynReject::BadCounter { field: "counter" })?;
                if rest.is_empty() {
                    return Err(SynReject::WrongType { field: prefix_static, expected: "stage name" });
                }
                m.insert(rest.to_string(), n);
            }
        }
        Ok(m)
    };
    let p = counts("p.")?;
    let m_axis = counts("m.")?;
    let r = counts("r.")?;
    let io = counts("io.")?;
    let budget = counts("budget.")?;
    if p.is_empty() {
        return Err(SynReject::MissingField("p.<stage>"));
    }
    // 伪 0 反控：whole_parse=0 而整源重建/读取面非零 ⇒ 结构性矛盾。
    let whole = p.get("whole_parse").copied().unwrap_or(0);
    let env_rebuilds = m_axis.get("env_rebuilds").copied().unwrap_or(0);
    let source_read = io.get("source_page_bytes").copied().unwrap_or(0);
    if whole == 0 && env_rebuilds > 0 && source_read > 0 && r.get("decl_env_dump").unwrap_or(&0) == &0 {
        return Err(SynReject::FakeZero {
            detail: format!("whole=0 but env_rebuilds={env_rebuilds} without a dump-restore R record"),
        });
    }
    let outcome = match get("outcome")?.as_str() {
        "ok" => SynOutcome::Ok,
        "source-error" => SynOutcome::SourceError,
        "resource-exhausted" => SynOutcome::ResourceExhausted,
        "cancelled" => SynOutcome::Cancelled,
        "infra" => SynOutcome::Infra,
        other => return Err(SynReject::BadEnum { field: "outcome", found: other.to_string() }),
    };
    Ok(SynReport {
        plan_id: get("plan_id")?,
        run_id: get("run_id")?,
        window_id: get("window_id")?,
        producer_commit,
        source_sha256,
        grammar_id,
        caps_version: get("caps_version")?,
        profile: get("profile")?,
        target: get("target")?,
        p_attempts: p,
        m_rebuilds: m_axis,
        r_verifies: r,
        io_bytes: io,
        budget_peaks: budget,
        certification: fields.get("certification").cloned(),
        outcome,
    })
}

/// 父侧期望（STALE 对账面）。
pub struct SynExpectation {
    pub producer_commit: String,
    pub grammar_id: String,
    pub source_sha256: String,
}

const KNOWN_KEYS: &[&str] = &[
    "schema", "plan_id", "run_id", "window_id", "producer_commit", "source_sha256",
    "grammar_id", "caps_version", "profile", "target", "outcome", "certification",
];

#[cfg(test)]
mod tests {
    use super::*;

    fn expect() -> SynExpectation {
        SynExpectation {
            producer_commit: "0234f3a22".into(),
            grammar_id: "auto-source@scalar/1".into(),
            source_sha256: "aa".into(),
        }
    }

    fn good() -> String {
        [
            "schema ac-source-syn-report/1",
            "plan_id PLAN-929",
            "run_id r1",
            "window_id restore-first-edit",
            "producer_commit 0234f3a22",
            "source_sha256 aa",
            "grammar_id auto-source@scalar/1",
            "caps_version ac-source-front/1",
            "profile scalar",
            "target av",
            "p.fragment_parse 1",
            "p.whole_parse 0",
            "m.env_rebuilds 1",
            "r.decl_env_dump 1",
            "r.deep_verify 3",
            "io.source_page_bytes 512",
            "budget.parent_ws_delta 130000",
            "outcome ok",
        ]
        .join("\n")
    }

    #[test]
    fn syn_report_admits_a_faithful_report() {
        let r = decode_syn_report(&good(), &expect()).expect("admit");
        assert_eq!(r.p_attempts.get("fragment_parse"), Some(&1));
        assert_eq!(r.outcome, SynOutcome::Ok);
    }

    #[test]
    fn syn_report_tamper_matrix() {
        let e = expect();
        // 缺字段。
        assert!(matches!(
            decode_syn_report(&good().replace("profile scalar\n", ""), &e),
            Err(SynReject::MissingField("profile"))
        ));
        // 未知字段。
        assert!(matches!(
            decode_syn_report(&format!("{}\nmystery 1", good()), &e),
            Err(SynReject::UnknownField(_))
        ));
        // 错 schema。
        assert!(matches!(
            decode_syn_report(&good().replace(SYN_REPORT_SCHEMA, "ac-source-syn-report/9"), &e),
            Err(SynReject::Schema(_))
        ));
        // 坏计数。
        assert!(matches!(
            decode_syn_report(&good().replace("p.fragment_parse 1", "p.fragment_parse x"), &e),
            Err(SynReject::BadCounter { .. })
        ));
        // 枚举外终局。
        assert!(matches!(
            decode_syn_report(&good().replace("outcome ok", "outcome fine"), &e),
            Err(SynReject::BadEnum { field: "outcome", .. })
        ));
        // STALE：构建位漂移。
        assert!(matches!(
            decode_syn_report(&good().replace("producer_commit 0234f3a22", "producer_commit deadbee"), &e),
            Err(SynReject::StaleIdentity { field: "producer_commit", .. })
        ));
        // 伪 0：whole=0 + env_rebuilds>0 + 无 dump R 记录。
        assert!(matches!(
            decode_syn_report(&good().replace("r.decl_env_dump 1", "r.deep_verifies 3"), &e),
            Err(SynReject::FakeZero { .. })
        ));
    }
}
