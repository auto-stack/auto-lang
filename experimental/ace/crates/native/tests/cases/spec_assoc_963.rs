//! PLAN-963 T-05 / V-G05（native 端，关联类型 + super-spec）：投影期擦除的
//! spec 义务经 mn 管线真实构建+运行，oracle 闭式手算。
//!
//! oracle：
//! - 关联类型 `Container<Item=int>`：`get(2) = data(40) + 2 = 42`。
//! - super-spec `Tool: Named`：`name() = 2`（自身实现）+ `size() = 40`
//!   （super-spec 默认体注入）= 42。

use ace_native::driver::MnDriver;

const ASSOC_PROG: &str = "\
spec Container {
    type Item
    fn get(i int) Item
}
type IntBox as Container<Item=int> {
    data int
    fn get(i int) int { return self.data + i }
}
fn main() int {
    let b IntBox = IntBox { data: 40 }
    return b.get(2)
}
";

const SUPER_PROG: &str = "\
spec Named {
    fn name() int
}
spec Tool: Named {
    fn size() int { return 40 }
}
type Wrench as Tool {
    v int
    fn name() int { return 2 }
}
fn main() int {
    let w Wrench = Wrench { v: 0 }
    return w.name() + w.size()
}
";

fn run(src: &str, tag: &str) -> Result<i32, String> {
    let checked = ace_core::source::compile_record_source(src, &format!("{tag}.at"), &[])
        .map_err(|ds| format!("compile failed: {ds:#?}"))?
        .checked;
    let dir = std::env::temp_dir().join(format!("mn963-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let snapshot = ace_av_core::contract::fnv1a64(src.as_bytes());
    let mut driver = MnDriver::ephemeral();
    let mn = driver.request_checked(&checked, "d_main", &dir, None, snapshot);
    let _ = std::fs::remove_dir_all(&dir);
    assert!(mn.ok, "{tag} mn leg failed: {:?}", mn.error);
    Ok(mn.exit_code.unwrap_or(-1))
}

/// V-G05：关联类型绑定后的方法真实执行 → 42。
#[test]
fn native_associated_type_method_exit_42() {
    assert_eq!(run(ASSOC_PROG, "assoc963").unwrap(), 42);
}

/// V-G05：super-spec 的默认体注入 + 自身实现协同执行 → 42。
#[test]
fn native_super_spec_default_body_exit_42() {
    assert_eq!(run(SUPER_PROG, "super963").unwrap(), 42);
}
