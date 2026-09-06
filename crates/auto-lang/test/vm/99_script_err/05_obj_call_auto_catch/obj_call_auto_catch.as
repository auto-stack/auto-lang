// Plan 569 D2 (T07): obj_call 组合子 Auto 臂 TypeError 可 catch。
//
// 分支不敏感保守正报的实测形态：pick 尾表达式是 py 调用 → fn_may_py_returns
// 记名 → pick(1) 实际返回 Auto str（"auto"）仍标记 py 可能 → .upper() 改发
// obj_call → Auto 臂（非外对象不可调用，对标 550 callable 守卫）报
// TypeError——响亮正报优于静默垃圾读，且可 catch 续行。
use.py builtins: list

fn pick(flag int) -> str {
    if flag > 0 {
        return "auto"
    }
    return list("xy")
}

fn main() {
    try {
        var up = pick(1).upper()
        print("no-error")
    } catch e {
        print("caught: " + e.to(str))
    }
    print("end")
}
