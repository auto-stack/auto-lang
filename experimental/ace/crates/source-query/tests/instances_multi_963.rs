//! PLAN-963 T-02：通用多类型参数实例面。
//!
//! 覆盖：声明序 substitution、同键去重（不因参数个数退化成调用点计数）、
//! 排列不同即不同实例（T/U 不混淆）、身份键可复算（单参字节兼容由 935
//! `choose_chain_935` 继续钉住）、实例页 codec v3（多实参往返 + v1/v2 字节
//! 按版本 MISS 不重释）。

use ace_core::source::cst::INSTANCES_CAPABILITY;
use ace_source_query::{
    compile_scalar_product, decode_instance_page, encode_instance_page, sq_hash, CompileLane,
    InstancePage, InstancePageError, InstanceRecord, InstanceTy, SourceQuerySession,
    INSTANCES_ALGO, INSTANCES_KEY_DOMAIN, INSTANCES_LOGICAL_DOMAIN, INSTANCES_SCHEMA,
};

const PKG: &str = "plan963";
const MOD: &str = "pick.at";

/// T 与 U 各自被两个调用点解出（同模板两种排列）。
const PICK: &str = "\
fn pick<T, U>(a T, b U, flag bool) T {
    if flag { return a }
    return a
}
fn main() int {
    if pick(true, 7, false) { return pick(42, true, false) }
    return 0
}
";

/// 同排列的两个调用点（去重面）。
const TWICE: &str = "\
fn twice<T, U>(a T, b U) T { return a }
fn main() int { return twice(1, true) + twice(2, false) }
";

fn compile(src: &str) -> ace_source_query::ProductCompile {
    let mut session = SourceQuerySession::new(PKG, MOD);
    compile_scalar_product(
        src,
        MOD,
        &[INSTANCES_CAPABILITY.to_string()],
        PKG,
        MOD,
        &mut session,
    )
    .expect("multi-param instances compile")
}

fn by_name<'a>(instances: &'a [InstanceRecord], name: &str) -> &'a InstanceRecord {
    instances
        .iter()
        .find(|i| i.concrete_name == name)
        .unwrap_or_else(|| panic!("no instance `{name}`: {instances:?}"))
}

/// O-963-A：两个排列各产一个实例；substitution 按声明序，键=逻辑+版本+
/// 逐位 `(参数名, 类型)`+算法/schema（可复算，不含 span/调用点）。
#[test]
fn multi_param_instances_keep_declaration_order() {
    let p = compile(PICK);
    assert_eq!(p.lane, CompileLane::Instances);
    assert_eq!(p.instances.len(), 2, "T/U 两种排列 = 两个实例");
    let a = by_name(&p.instances, "pick_i32_bool");
    assert_eq!(
        a.args,
        vec![
            ace_source_query::InstanceArg {
                param: "T".into(),
                ty: InstanceTy::I32
            },
            ace_source_query::InstanceArg {
                param: "U".into(),
                ty: InstanceTy::Bool
            },
        ],
        "substitution 按声明序落位"
    );
    let b = by_name(&p.instances, "pick_bool_i32");
    assert_eq!(b.args[0].ty, InstanceTy::Bool);
    assert_eq!(b.args[1].ty, InstanceTy::I32);
    assert_ne!(a.key, b.key, "排列不同即不同实现身份");
    assert_eq!(a.logical, b.logical, "逻辑实例只按声明 owner 取 id");
    // 键可复算（顺序契约：logical,sig,body,(param,ty)*,algo,schema）。
    let logical = sq_hash(
        INSTANCES_LOGICAL_DOMAIN,
        &[PKG.as_bytes(), MOD.as_bytes(), b"tpl", b"pick"],
    );
    let expect = sq_hash(
        INSTANCES_KEY_DOMAIN,
        &[
            logical.as_bytes(),
            b"sig",
            a.sig_version.as_bytes(),
            b"body",
            a.body_version.as_bytes(),
            b"T",
            b"i32",
            b"U",
            b"bool",
            INSTANCES_ALGO.as_bytes(),
            INSTANCES_SCHEMA.as_bytes(),
        ],
    );
    assert_eq!(a.key, expect, "多参身份键 = 声明序成分序");
}

/// O-963-B：同一排列的两个调用点只产一个实例，demands 计 2（去重不是
/// 每调用点一份）。
#[test]
fn multi_param_dedup_counts_call_sites() {
    let p = compile(TWICE);
    assert_eq!(p.instances.len(), 1);
    let only = &p.instances[0];
    assert_eq!(only.concrete_name, "twice_i32_bool");
    assert_eq!(only.demands, 2);
}

/// O-963-C：实例页 codec v3——多实参往返逐位保序；v1/v2 页字节按版本拒
/// （不重释旧页），magic/信封不变。
#[test]
fn multi_param_page_codec_v3() {
    let p = compile(PICK);
    for inst in &p.instances {
        let page = InstancePage::from(inst);
        assert_eq!(page.args.len(), 2);
        let bytes = encode_instance_page(&page).expect("encode v3");
        let back = decode_instance_page(&bytes).expect("decode v3");
        assert_eq!(back, page);
    }
    let page = InstancePage::from(&p.instances[0]);
    let bytes = encode_instance_page(&page).expect("encode");
    for stale in [1u8, 2u8] {
        let mut old = bytes.clone();
        old[6] = stale;
        assert_eq!(
            decode_instance_page(&old),
            Err(InstancePageError::BadVersion { found: stale }),
            "v{stale} 页不得按 v3 解释"
        );
    }
    // 尾部截断（demands 不足 8 字节）仍须按截断拒。
    let mut truncated = bytes.clone();
    truncated.truncate(bytes.len() - 4);
    assert!(decode_instance_page(&truncated).is_err());
    // 实参中部截断（记录名读取越界）同样不得重释。
    let mut mid = bytes.clone();
    mid.truncate(bytes.len() - 12);
    assert!(decode_instance_page(&mid).is_err());
}
