//! PLAN-618 探针:VM 轨模块 fn 递归(filter_tree 树过滤)能力验证。
//! 026 对象树过滤依赖递归纯函数;若 VM 不支持则 026 需降级实现。

use auto_val::Value;

const TREE_UTIL_AT: &str = r#"
pub fn fact(n int) int {
    if n <= 1 {
        return 1
    }
    return n * fact(n - 1)
}

pub fn label_of(nodes List, i int) str {
    var nd = nodes[i]
    return nd.label
}

pub fn filter_simple(nodes List, kw str) List {
    var out List = []
    var i int = 0
    while i < nodes.len() {
        var nd = nodes[i]
        if nd.label.contains(kw) {
            out.push({ id: nd.id, label: nd.label, children: [] })
        }
        var fk List = filter_simple(nd.children, kw)
        var k int = 0
        while k < fk.len() {
            out.push(fk[k])
            k = k + 1
        }
        i = i + 1
    }
    return out
}

pub fn label_lower(nodes List, i int) str {
    var nd = nodes[i]
    return nd.label.lower()
}

pub fn hit_lower(nodes List, i int, kw str) bool {
    var nd = nodes[i]
    return nd.label.lower().contains(kw.lower())
}

pub fn hit_chain2(nodes List, i int) bool {
    var nd = nodes[i]
    return nd.label.contains("data")
}

pub fn lower_twostep(nodes List, i int) str {
    var nd = nodes[i]
    var lbl str = nd.label
    var ll str = lbl.lower()
    return ll
}

pub fn lower_direct(nodes List, i int) str {
    var nd = nodes[i]
    return nd.label.lower()
}

pub fn filter_tree(nodes List, kw str) List {
    var out List = []
    var i int = 0
    while i < nodes.len() {
        var nd = nodes[i]
        var fk List = filter_tree(nd.children, kw)
        var selfHit bool = kw == "" || nd.label.contains(kw)
        if selfHit || fk.len() > 0 {
            out.push({ id: nd.id, label: nd.label, kind: nd.kind, icon: nd.icon, is_leaf: nd.is_leaf, badge: nd.badge, children: fk })
        }
        i = i + 1
    }
    return out
}
"#;

const APP_AT: &str = r#"
use tree_util: filter_tree, fact, label_of, filter_simple, label_lower, hit_lower, hit_chain2, lower_twostep, lower_direct

widget FilterProbe {
    msg {}

    model {
        nodes = [
            { id: "db", label: "database", kind: "dir", icon: "", is_leaf: false, badge: "", children: [
                { id: "t/customers", label: "customers", kind: "file", icon: "", is_leaf: false, badge: "91", children: [] },
                { id: "t/products", label: "products", kind: "file", icon: "", is_leaf: false, badge: "77", children: [] }
            ] }
        ]
    }

    computed {
        filtered => filter_tree(.nodes, "cust")
    }

    view {
        col {
            for r in .filtered {
                text r.label
            }
        }
    }
}
"#;

fn setup_fixture() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("p618_filter_probe_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("tree_util.at"), TREE_UTIL_AT).unwrap();
    std::fs::write(dir.join("app.at"), APP_AT).unwrap();
    dir.join("app.at")
}

#[cfg(all(test, feature = "ui-iced"))]
#[test]
fn p618_vm_filter_tree_recursion_probe() {
    let manifest = setup_fixture();
    let Some(mut dc) = crate::plan370_test_support::build_component_from_app(&manifest) else {
        eprintln!("p618 probe: fixture build returned None");
        return;
    };

    let nodes = dc.bridge().read_state("nodes").expect("nodes state");

    let f = dc.bridge().call_vm_fn("fact", &[Value::Int(5)]).expect("fact call");
    println!("[probe] fact(5) = {:?}", f);
    let ll = dc.bridge().call_vm_fn("label_lower", &[nodes.clone(), Value::Int(0)]);
    println!("[probe] label_lower = {:?}", ll);
    let h2 = dc.bridge().call_vm_fn("hit_chain2", &[nodes.clone(), Value::Int(0)]);
    println!("[probe] hit_chain2 = {:?}", h2);
    let hl = dc.bridge().call_vm_fn("hit_lower", &[nodes.clone(), Value::Int(0), Value::str("DATA")]);
    println!("[probe] hit_lower = {:?}", hl);
    let ts = dc.bridge().call_vm_fn("lower_twostep", &[nodes.clone(), Value::Int(0)]);
    println!("[probe] lower_twostep = {:?}", ts);
    let dr = dc.bridge().call_vm_fn("lower_direct", &[nodes.clone(), Value::Int(0)]);
    println!("[probe] lower_direct = {:?}", dr);
    let lo = dc.bridge().call_vm_fn("label_of", &[nodes.clone(), Value::Int(0)]).expect("label_of");
    println!("[probe] label_of(nodes,0) = {:?}", lo);
    let fs = dc.bridge().call_vm_fn("filter_simple", &[nodes.clone(), Value::str("cust")]).expect("filter_simple");
    let fs_rows = match &fs { Value::Array(a) => a.values.len(), Value::VmRef(r) => dc.bridge().index_list_all(r.id).len(), other => panic!("{other:?}") };
    println!("[probe] filter_simple(cust) rows = {}", fs_rows);

    // 命中过滤:cust → 保留 db(祖先) + t/customers
    let hit = dc
        .bridge()
        .call_vm_fn("filter_tree", &[nodes.clone(), Value::str("cust")])
        .expect("filter_tree call");
    let rows = match &hit {
        Value::Array(a) => a.values.clone(),
        Value::VmRef(r) => dc.bridge().index_list_all(r.id),
        other => panic!("filter result should be a list, got {other:?}"),
    };
    println!("[probe] filter(cust) top rows = {}", rows.len());
    assert_eq!(rows.len(), 1, "only db kept at top (customers nested, no self match)");
    // 嵌套断言:db.children 应保留 t/customers(祖先保留语义)
    let db_obj = dc.bridge().materialize_obj_ref(&rows[0]);
    let db_kids = match &db_obj {
        Value::Obj(o) => match o.get("children") {
            Some(v) => match v {
                Value::Array(a) => a.values.len(),
                Value::VmRef(r) => dc.bridge().index_list_all(r.id).len(),
                _ => 999,
            },
            None => 999,
        },
        other => panic!("db should be obj, got {other:?}"),
    };
    println!("[probe] db.children len = {}", db_kids);
    assert_eq!(db_kids, 1, "ancestor db keeps matching child");

    // 无命中:全剪
    let miss = dc
        .bridge()
        .call_vm_fn("filter_tree", &[nodes.clone(), Value::str("zzz")])
        .expect("filter_tree miss call");
    let miss_rows = match &miss {
        Value::Array(a) => a.values.clone(),
        Value::VmRef(r) => dc.bridge().index_list_all(r.id),
        other => panic!("filter result should be a list, got {other:?}"),
    };
    assert_eq!(miss_rows.len(), 0, "no match → empty tree");

    // computed 形态(生产路径)
    let comp = dc
        .bridge()
        .call_computed_fn("FilterProbe", "filtered")
        .expect("computed filtered");
    let comp_rows = match &comp {
        Value::Array(a) => a.values.clone(),
        Value::VmRef(r) => dc.bridge().index_list_all(r.id),
        other => panic!("computed should be a list, got {other:?}"),
    };
    assert_eq!(comp_rows.len(), 1, "computed path sees filtered tree (db only at top)");
}
