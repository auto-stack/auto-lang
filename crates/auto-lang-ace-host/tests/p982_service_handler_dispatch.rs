//! PLAN-982（983 服务链的地基）：**ACE 能按名字运行指定入口**，而 **AV 车道的
//! args/input 面仍是具名拒绝**。
//!
//! 983 的 AV 服务路径要把每个 HTTP 请求交给一个**具名 handler**执行：
//! 1. 底座已具备：`AceRequest.entry_def_id` 能选中 `d_main` 以外的 def，返回值走
//!    `AceOutcome::Success { value }`——本文件第一例钉住它；
//! 2. 缺口已确认：`AceRequest.args`（非空）与 `AceRequest.input`（`Some`）在 AV 车道
//!    是具名拒绝（`ace.args/unsupported` / `ace.input/unsupported`），结构体注释
//!    写明 "never a silent drop and never a legacy-VM route"——第二例钉住它，
//!    因为**请求内容送不进程序**是 983 必须先补的那块面。

use auto_lang_core::{run_ace, AceEngineChoice, AceOutcome, AceRequest};

fn install() {
    auto_lang_ace_host::install().expect("the host executor installs");
}

fn run(entry: &str, source: &str) -> auto_lang_core::AceRunReport {
    let request = AceRequest {
        entry_def_id: entry,
        ..AceRequest::simple(source)
    };
    run_ace(AceEngineChoice::AceAV, &request)
}

#[test]
fn p982_ace_runs_a_named_non_main_entry() {
    install();
    let source = "fn main() int { return 1 }\nfn handler() int { return 41 }\n";
    match &run("d_handler", source).outcome {
        AceOutcome::Success { value, .. } => {
            let rendered = format!("{value:?}");
            eprintln!("p982: d_handler returned {rendered}");
            assert!(
                rendered.contains("41"),
                "the named entry's value comes back: {rendered}"
            );
        }
        other => panic!("a named entry must run: {other:?}"),
    }
}

#[test]
fn p982_ace_av_lane_refuses_args_and_input_by_name() {
    install();
    let source = "fn main() int { 0 }\n";

    let args = ["7"];
    let with_args = AceRequest {
        args: &args,
        ..AceRequest::simple(source)
    };
    let args_outcome = format!("{:?}", run_ace(AceEngineChoice::AceAV, &with_args).outcome);
    eprintln!("p982: non-empty args outcome = {args_outcome}");
    assert!(
        args_outcome.contains("ace.args/unsupported"),
        "non-empty args is the named `ace.args/unsupported` refusal: {args_outcome}"
    );

    let with_input = AceRequest {
        input: Some("body"),
        ..AceRequest::simple(source)
    };
    let input_outcome = format!("{:?}", run_ace(AceEngineChoice::AceAV, &with_input).outcome);
    eprintln!("p982: input outcome = {input_outcome}");
    assert!(
        input_outcome.contains("ace.input/unsupported"),
        "input is the named `ace.input/unsupported` refusal: {input_outcome}"
    );
}
