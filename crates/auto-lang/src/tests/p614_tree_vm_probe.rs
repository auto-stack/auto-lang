//! PLAN-614 临时探针:VM 轨模块 fn 对 Obj 嵌套 children/List/bool 参数的
//! 行为隔离。fixture = 最小树 + flatten_tree 同款实现,经生产 DynamicComponent
//! 路径构建后直接 call_vm_fn / call_computed_fn 检查中间产物。

use auto_val::Value;

const TREE_UTIL_AT: &str = r#"// components/tree_util.at — tree 组件族纯函数工具(PLAN-614)。
//
// flatten_tree: 嵌套树 × 展开态 → 可见行序列。显式栈迭代 DFS(无递归——
// chart/diagram 族同款纪律),row 记录携带渲染所需一切(缩进/图标/chevron/
// 状态 class 字面量),组件 view 零逻辑。受控契约:expanded/selected 由
// 页面持有,本模块只做纯派生,双端(vue 转译进 SFC / VM import_aliases)
// 同源(Plan 522 路径,calendar_util 先例)。
//
// pads 阶梯数组字面量随 use-fn 转译进 SFC 源 → Tailwind JIT 可扫描;
// VM 侧 class.rs pl-{N} 任意数值。depth>8 钳制 pl-32(任意形状兜底)。
//
// ⚠ VM 轨遍历纪律(P614 实证):for-in 对函数参数列表迭代零次(对状态
// 路径迭代正常)——本模块所有列表遍历一律 while + 索引(索引访问参数
// 列表已实证正常)。嵌套 children 同式。
//
// node schema(数据轨;字段必填全键书写——VM Obj 字面量为形状锁定类型
// 实例,缺键字段访问=硬错,P614 实证。kind: "dir"|"file"|"" 仅 directory
// 模式消费;icon 显式指定优先于自动映射):
// { id: str, label: str, children: List, kind: str, icon: str,
//   is_leaf: bool, badge: str }

/// 线性成员测试(while+索引;for-in 参数列表 VM 零次迭代,见头注)。
pub fn has_id(list List, id str) bool {
    var found bool = false
    var i int = 0
    while i < list.len() {
        if list[i] == id {
            found = true
        }
        i = i + 1
    }
    return found
}

/// 切换 id 在列表中的成员资格(返回新 List,不动输入——受控更新函数式)。
pub fn toggle_id(list List, id str) List {
    var out List = []
    var found bool = false
    var i int = 0
    while i < list.len() {
        if list[i] == id {
            found = true
        } else {
            out.push(list[i])
        }
        i = i + 1
    }
    if !found {
        out.push(id)
    }
    return out
}

/// 全部(或仅目录)id 收集——Expand All / Collapse All 用。
pub fn collect_ids(nodes List, dirs_only bool) List {
    var out List = []
    var stack List = []
    var i int = nodes.len()
    while i > 0 {
        i = i - 1
        stack.push(nodes[i])
    }
    while stack.len() > 0 {
        var top int = stack.len() - 1
        var nd = stack[top]
        stack.pop()
        var kids List = nd.children
        var is_dir bool = nd.kind == "dir" || kids.len() > 0
        if !dirs_only {
            out.push(nd.id)
        } else {
            if is_dir {
                out.push(nd.id)
            }
        }
        var j int = kids.len()
        while j > 0 {
            j = j - 1
            stack.push(kids[j])
        }
    }
    return out
}

/// 文件名 → 文件图标(lucide 名)。图片族/文本族/其余 file。
pub fn ext_icon(name str) str {
    var ic str = "file"
    if name.ends_with(".png") || name.ends_with(".jpg") || name.ends_with(".jpeg") || name.ends_with(".gif") || name.ends_with(".webp") || name.ends_with(".svg") {
        ic = "image"
    } else {
        if name.ends_with(".at") || name.ends_with(".rs") || name.ends_with(".ts") || name.ends_with(".vue") || name.ends_with(".md") || name.ends_with(".txt") || name.ends_with(".json") || name.ends_with(".toml") || name.ends_with(".css") || name.ends_with(".js") {
            ic = "file-text"
        }
    }
    return ic
}

/// 嵌套树 → 可见行序列(跳过未展开子树)。
/// row = { id, label, depth, has_kids, open, leaf, icon, badge, pad, chev, state }。
pub fn flatten_tree(nodes List, expanded List, directory bool, selected str) List {
    var pads List = ["", "pl-4", "pl-8", "pl-12", "pl-16", "pl-20", "pl-24", "pl-28", "pl-32"]
    var outRows List = []
    var stack List = []
    var depths List = []
    var i int = nodes.len()
    while i > 0 {
        i = i - 1
        stack.push(nodes[i])
        depths.push(0)
    }
    while stack.len() > 0 {
        var top int = stack.len() - 1
        var nd = stack[top]
        var d int = depths[top]
        stack.pop()
        depths.pop()
        var kids List = nd.children
        var is_leaf bool = nd.is_leaf == true
        var has_kids bool = false
        if kids.len() > 0 && !is_leaf {
            has_kids = true
        }
        var open bool = false
        if has_kids && has_id(expanded, nd.id) {
            open = true
        }
        var ic str = nd.icon
        if ic == "" && directory {
            var is_dir bool = nd.kind == "dir" || has_kids
            if is_dir {
                if open {
                    ic = "folder-open"
                } else {
                    ic = "folder"
                }
            } else {
                ic = ext_icon(nd.label)
            }
        }
        var pi int = d
        if pi > 8 {
            pi = 8
        }
        var chev str = "chevron-right"
        if open {
            chev = "chevron-down"
        }
        var state str = "hover:bg-accent/50"
        if selected != "" {
            if selected == nd.id {
                state = "bg-accent text-accent-foreground"
            }
        }
        outRows.push({ id: nd.id, label: nd.label, depth: d, has_kids: has_kids, open: open, leaf: !has_kids, icon: ic, badge: nd.badge, pad: pads[pi], chev: chev, state: state })
        if open {
            var j int = kids.len()
            while j > 0 {
                j = j - 1
                stack.push(kids[j])
                depths.push(d + 1)
            }
        }
    }
    return outRows
}


pub fn len_of(list List) int {
    return list.len()
}

pub fn first_of(list List) str {
    return list[0]
}

pub fn iter_last(list List) str {
    var out str = "<none>"
    var i int = 0
    while i < list.len() {
        out = list[i]
        i = i + 1
    }
    return out
}

pub fn contains_manual(list List, id str) str {
    var out str = "no"
    var i int = 0
    while i < list.len() {
        if list[i] == id {
            out = "yes"
        }
        i = i + 1
    }
    return out
}

pub fn compare_eq(a str, b str) bool {
    return a == b
}
"#;

const APP_AT: &str = r#"
use tree_util: flatten_tree, has_id, len_of, first_of, contains_manual, iter_last, compare_eq

widget TreeProbe {
    msg {}

    model {
        nodes = [
            { id: "a", label: "A", kind: "dir", icon: "", is_leaf: false, badge: "", children: [
                { id: "b", label: "B", kind: "file", icon: "", is_leaf: false, badge: "", children: [] }
            ] },
            { id: "c", label: "C", kind: "file", icon: "", is_leaf: false, badge: "", children: [] }
        ]
        expanded = ["a"]
    }

    computed {
        rows => flatten_tree(.nodes, .expanded, false, "")
    }

    view {
        col {
            for r in .rows {
                text r.label
            }
        }
    }
}
"#;

fn setup_fixture() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("p614_probe_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("tree_util.at"), TREE_UTIL_AT).unwrap();
    std::fs::write(dir.join("app.at"), APP_AT).unwrap();
    dir.join("app.at")
}

#[cfg(all(test, feature = "ui-iced"))]
#[test]
fn p614_vm_flatten_tree_probe() {
    let manifest = setup_fixture();
    let Some(mut dc) =
        crate::plan370_test_support::build_component_from_app(&manifest)
    else {
        eprintln!("p614 probe: fixture build returned None");
        return;
    };

    // 0) 页面状态里的嵌套模型已就位
    let nodes = dc.bridge().read_state("nodes").expect("nodes state");
    println!("[probe] nodes = {:?}", nodes);

    // 1) 隔离调用 has_id:List 参数 + for-in
    let expanded = dc.bridge().read_state("expanded").expect("expanded state");
    let hit = dc
        .bridge()
        .call_vm_fn("has_id", &[expanded.clone(), Value::str("a")])
        .expect("has_id call via bare alias");
    println!("[probe] has_id(expanded, 'a') = {:?}", hit);

    // 微探针:参数到达性逐层隔离
    let exp_dbg = dc.bridge().index_list_all(4000006);
    let len_v = dc.bridge().call_vm_fn("len_of", &[expanded.clone()]).expect("len_of");
    println!("[probe] rust-side expanded content = {:?} | vm len_of = {:?}", exp_dbg, len_v);
    let first = dc.bridge().call_vm_fn("first_of", &[expanded.clone()]);
    println!("[probe] first_of = {:?}", first);
    let manual = dc.bridge().call_vm_fn("contains_manual", &[expanded.clone(), Value::str("a")]).expect("contains_manual");
    println!("[probe] contains_manual(expanded,'a') = {:?}", manual);
    let nodes_len = dc.bridge().call_vm_fn("len_of", &[nodes.clone()]).expect("len_of nodes");
    println!("[probe] len_of(nodes) = {:?}", nodes_len);
    let it = dc.bridge().call_vm_fn("iter_last", &[expanded.clone()]).expect("iter_last");
    println!("[probe] iter_last(expanded) = {:?}", it);
    let eq = dc.bridge().call_vm_fn("compare_eq", &[Value::str("a"), Value::str("a")]).expect("compare_eq");
    println!("[probe] compare_eq('a','a') = {:?}", eq);

    // 2) 隔离调用 flatten_tree:直接传状态值
    let rows = dc
        .bridge()
        .call_vm_fn(
            "flatten_tree",
            &[nodes.clone(), expanded.clone(), Value::Bool(false), Value::str("")],
        )
        .expect("flatten_tree call");
    let row_list = match &rows {
        Value::Array(a) => a.values.clone(),
        Value::VmRef(r) => dc.bridge().index_list_all(r.id),
        other => panic!("rows should be a list, got {other:?}"),
    };
    println!("[probe] flatten_tree rows = {}", row_list.len());
    for r in &row_list {
        println!("[probe]   row = {:?}", r);
    }
    assert_eq!(row_list.len(), 3, "a expanded → a, b, c all visible");

    // 3) computed 形态(生产路径:隐藏 fn 从根状态自读)
    let comp = dc
        .bridge()
        .call_computed_fn("TreeProbe", "rows")
        .expect("computed rows");
    let comp_list = match &comp {
        Value::Array(a) => a.values.clone(),
        Value::VmRef(r) => dc.bridge().index_list_all(r.id),
        other => panic!("computed rows should be a list, got {other:?}"),
    };
    println!("[probe] computed rows = {}", comp_list.len());
    assert_eq!(comp_list.len(), 3, "computed path sees the same 3 rows");
}
