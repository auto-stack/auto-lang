//! Plan: VM 真异步调度统一 — 调研探针(research probes)
//!
//! 每个 probe 验证一个异步机制断点的真实状态。
//! 用 run_with_capture 跑,捕获 result(返回值 repr)和 stdout,
//! 判断该机制在 AutoVM 下是否真的工作。
//!
//! 调研完成后本文件会被删除或转为正式 plan 的验收用例。

use crate::{run, run_with_capture};

fn probe(label: &str, code: &str) -> (String, String) {
    eprintln!("\n========== PROBE: {} ==========", label);
    let (result, stdout) = match run_with_capture(code) {
        Ok((r, s)) => (r, s),
        Err(e) => (format!("ERROR: {}", e), String::new()),
    };
    eprintln!("[{}] result = {:?}", label, result);
    eprintln!("[{}] stdout = {:?}", label, stdout);
    (result, stdout)
}

// ===========================================================================
// 断点 1: ~{} async block — body 是否执行
// ===========================================================================
#[test]
fn probe_01_async_block_basic() {
    let (result, stdout) = probe("async_block_basic", r#"
fn main() {
    let f = ~{ print("inside async block") }
    print("after create")
}
"#);
    // 若 ~{} body 真执行,"inside async block" 应出现在 stdout。
    // 断点预测:body_offset 占位,body 可能不执行或乱跑。
    let _ = (result, stdout);
}

// ===========================================================================
// 断点 1b: ~{} + .await — future body 在 await 时执行
// ===========================================================================
#[test]
fn probe_01b_async_block_await() {
    let (result, stdout) = probe("async_block_await", r#"
fn main() {
    let f = ~{
        print("body running")
        42
    }
    let v = f.await
    print(f"got ${v}")
}
"#);
    let _ = (result, stdout);
}

// ===========================================================================
// 断点 2: ~T 异步函数 + .await
// ===========================================================================
#[test]
fn probe_02_async_fn_await() {
    let (result, stdout) = probe("async_fn_await", r#"
fn compute() ~int {
    return 89
}
fn main() {
    let v = compute().await
    print(f"got ${v}")
}
"#);
    let _ = (result, stdout);
}

// ===========================================================================
// 断点 2b: ~T 函数带 body 逻辑(非直接 return)
// ===========================================================================
#[test]
fn probe_02b_async_fn_body() {
    let (result, stdout) = probe("async_fn_body", r#"
fn compute(x int) ~int {
    let y = x * 2
    return y
}
fn main() {
    let v = compute(21).await
    print(f"got ${v}")
}
"#);
    let _ = (result, stdout);
}

// ===========================================================================
// 断点 3: TaskSystem.run(~{...})
// ===========================================================================
#[test]
fn probe_03_task_system_run() {
    let (result, stdout) = probe("task_system_run", r#"
fn main() {
    TaskSystem.run(~{
        print("inside TaskSystem.run")
    })
    print("after run")
}
"#);
    let _ = (result, stdout);
}

// ===========================================================================
// 断点 4: 独立 task 定义 + handler 执行
// 语法(task parser 验证):task Name { fn start() { } on { Pat -> body } }
// 消息是 enum variant(转 i32),Task.spawn(type_str, cap) -> handle_id
// ===========================================================================
#[test]
fn probe_04_task_definition() {
    let (result, stdout) = probe("task_definition", r#"
enum Msg {
    Ping = 1
    Pong = 2
}

task Counter {
    count = 0
    fn start() {
        print("Counter started")
    }
    on {
        Msg.Ping -> {
            print("got Ping")
        }
        else -> {
            print("got other")
        }
    }
}

fn main() {
    print("main before")
    let h = Task.spawn("Counter", 16)
    h.send(Msg.Ping)
    print("main after")
}
"#);
    let _ = (result, stdout);
}

// ===========================================================================
// 断点 4b: TaskSystem.start() 驱动 actor 循环(阻塞)
// ===========================================================================
#[test]
fn probe_04b_task_system_start() {
    let (result, stdout) = probe("task_system_start", r#"
task Greeter {
    fn start() {
        print("Greeter started")
    }
    on {
        _ -> {
            print("Greeter got msg")
        }
    }
}

fn main() {
    print("main: spawning")
    Task.spawn("Greeter", 16)
    print("main: calling TaskSystem.start")
    TaskSystem.start()
    print("main: after start")
}
"#);
    let _ = (result, stdout);
}

// ===========================================================================
// 断点 5: producer/consumer — 两个 task 并发
// ===========================================================================
#[test]
fn probe_05_producer_consumer() {
    let (result, stdout) = probe("producer_consumer", r#"
enum Item {
    Num = 1
    End = 2
}

task Consumer {
    sum = 0
    on {
        Item.Num -> {
            print("consumer got Num")
        }
        Item.End -> {
            print("consumer done")
        }
    }
}

fn main() {
    let c = Task.spawn("Consumer", 16)
    c.send(Item.Num)
    c.send(Item.End)
    print("sent messages")
}
"#);
    let _ = (result, stdout);
}

// ===========================================================================
// 断点 6: actor 通信靠 TaskHandle.send(无原生 channel)
// 验证 send + ask(同步请求-回复)
// ===========================================================================
#[test]
fn probe_06_task_handle_send() {
    let (result, stdout) = probe("task_handle_send", r#"
task Echo {
    on {
        _ -> {
            print("echo got msg")
        }
    }
}

fn main() {
    let h = Task.spawn("Echo", 16)
    let ok = h.send(1)
    print(f"send returned ${ok}")
}
"#);
    let _ = (result, stdout);
}

// ===========================================================================
// 断点 7: yield/Iter 与 future/await 互通(修正 var 语法)
// 对照基线:generator for-loop(已知工作)
// ===========================================================================
#[test]
fn probe_07_yield_await_interop() {
    let (result, stdout) = probe("yield_await_interop", r#"
fn gen() ~Iter<int> {
    yield 1
    yield 2
    yield 3
}
fn main() {
    var s = 0
    for n in gen() {
        s = s + n
    }
    print(f"sum=${s}")
}
"#);
    // 这个我已知工作(generator 修复后)。作为对照基线。
    let _ = (result, stdout);
}

// ===========================================================================
// 汇总:跑所有 probe,打印汇总表
// ===========================================================================
#[test]
fn probe_summary() {
    eprintln!("\n\n############ ASYNC PROBE SUMMARY ############");
    eprintln!("See individual probe_* test outputs above for details.");
    eprintln!("Each probe prints [label] result=... and stdout=...");
    eprintln!("Interpret each: does the async mechanism actually work?");
}
