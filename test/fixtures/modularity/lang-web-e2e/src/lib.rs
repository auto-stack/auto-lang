//! PLAN-906: language/Web HTTP e2e family（原 `crates/auto-lang/src/tests/`
//! plan705/707/729/730/734/736/776/781 八文件整族迁移，测试 fn 名/ID/条件
//! 原样保留）。facade 退役 auto-web 依赖后，真 TCP e2e 需要 Web 宿主的
//! 家族在本独立探针工程执行——显式依赖 `auto_lang`（语言 facade 公开面）
//! 与 `auto_web`（宿主安装），不再经 facade 的 Web 再导出边。
//!
//! 非语言 workspace 成员（组内兄弟布局解析 `../../../../../auto-lang` 与
//! `../../../../../auto-web`）；由四仓 fixture 门驱动（web-repository-
//! fixture / web-four-repository），与 web-port-probe 等既有探针同形态。
pub fn install_web_http_host() {
    auto_web::install_http_service_host_if_absent();
}

#[cfg(test)]
mod plan705_spike_tests;
#[cfg(test)]
mod plan707_stream_e2e_tests;
#[cfg(test)]
mod plan729_http_server_file_tests;
#[cfg(test)]
mod plan730_http_upload_tests;
#[cfg(test)]
mod plan734_api_contract_tests;
#[cfg(test)]
mod plan736_http_service_tests;
#[cfg(test)]
mod plan776_http_return_tests;
#[cfg(test)]
mod plan781_budget_http_tests;
#[cfg(test)]
mod plan707_wait_tests; // PLAN-906 r2: Web SSE 测试助手依赖随迁
#[cfg(test)]
mod plan349_cors_tests; // PLAN-906 r2: CORS 三面 + real_get wire（auto_web::http_server 符号）
