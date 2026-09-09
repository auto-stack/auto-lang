# url_real — 真三方 url 动态加载绿面语料（PLAN-594）

动态加载真实 crates.io url（类型面走 VM native_catalog，零构建秒级）。

- 上游版本：url 2.5.4（dep 声明与 oracle Cargo.toml 同版；锁定勿改版重跑）。
- 绿面：`Url.parse(..).unwrap()` + `scheme()/path()`（&str 直读）+
  `host_str().unwrap()`（Option 构造器链 unwrap 形态，双侧实证）。
- 红面（登记 DIV-DEP-8+）：`u.to(str)`（VM=`<url::Url>` 占位 vs a2r=Debug 结构体转储）、
  `host_str()` 不 unwrap 直出（VM 裸文本 vs a2r `Some(..)`）、`port()`（Option<u16>）、
  借用链/Origin/Serde 面。详见命中率报告。
