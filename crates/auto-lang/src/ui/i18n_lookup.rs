//! PLAN-050 T9 (C7): VM 轨 `t()`/`i18n.t()` 插值最小查表。
//!
//! vue 轨的 i18n 走宿主 JS（useI18n composable + i18n/{lang}.json），VM 轨
//! 无宿主——此前 `text i18n.t("settings.title")` 一类文本节点求值恒空
//! （musk 设置面板选项文案全空，KD-048 UPSTREAM④ 同族现象）。
//!
//! 约定（与 musk 现行目录一致，不引入 musk 专属耦合）：front 根目录下
//! `i18n/{lang}.json` 平铺 key→文案。VM 装载器（build_dynamic_component）
//! 按 `AUTO_LOCALE`（默认 zh）装载进进程级注册表；求值臂对 `t("k")` /
//! `i18n.t("k")` 直查。目录缺失 = 空表（查不中回落 key 本身）。
//!
//! 已知边界：运行期切语言（settings 的 ChangeLocale）不在本批——装载期
//! 定 locale，env 重启生效；登记 PLAN-050 待澄清。

use std::collections::HashMap;
use std::path::Path;
use std::sync::RwLock;

static TABLE: RwLock<Option<HashMap<String, String>>> = RwLock::new(None);

/// 装载 `i18n/{lang}.json`（lang = AUTO_LOCALE env，默认 zh）。文件缺失或
/// 解析失败 = 清空表（回落 key 字面量），保持静默——i18n 缺席不是错误。
/// musk 的 zh.json 为命名空间嵌套（settings.title → …），平铺成点分 key。
pub fn load_from_dir(base_dir: &Path) {
    let lang = std::env::var("AUTO_LOCALE").unwrap_or_else(|_| "zh".to_string());
    let path = base_dir.join("i18n").join(format!("{lang}.json"));
    let raw = std::fs::read_to_string(&path).ok().and_then(|raw| {
        let value: serde_json::Value = serde_json::from_str(&raw).ok()?;
        let mut map = HashMap::new();
        flatten("", &value, &mut map);
        Some(map)
    });
    if let Ok(mut guard) = TABLE.write() {
        *guard = raw;
    }
}

fn flatten(prefix: &str, value: &serde_json::Value, out: &mut HashMap<String, String>) {
    match value {
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                let key = if prefix.is_empty() { k.clone() } else { format!("{prefix}.{k}") };
                flatten(&key, v, out);
            }
        }
        serde_json::Value::String(s) => {
            out.insert(prefix.to_string(), s.clone());
        }
        _ => {}
    }
}

/// vue-i18n 字面量插值转义：`{'@'}` → `@`。消息层此前不消费该转义 →
/// musk 输入框 placeholder 直出 "{@} 呼出 agent"（PLAN-054 T3/R4）。
/// 语法 `{'<literal>'}` 整段替换为字面量；无 `'}` 闭合时原样保留。
pub fn unescape_literals(msg: &str) -> String {
    if !msg.contains("{'") {
        return msg.to_string();
    }
    let mut out = String::with_capacity(msg.len());
    let mut rest = msg;
    while let Some(i) = rest.find("{'") {
        out.push_str(&rest[..i]);
        let after = &rest[i + 2..];
        if let Some(end) = after.find("'}") {
            out.push_str(&after[..end]);
            rest = &after[end + 2..];
        } else {
            out.push_str("{'");
            rest = after;
        }
    }
    out.push_str(rest);
    out
}

/// 查一个 key。空表/未命中返回 None（调用方决定回落形态）。
/// PLAN-051 P2-②b: 模板参数插值——`{k}` 占位按 params 逐名替换；未提供
/// 的参数原样保留（回落可见性优于静默清除）。
/// PLAN-054 T3: 替换后消费 `{'x'}` 字面量转义（param 值也可能带转义）。
pub fn substitute_params(template: &str, params: &[(String, String)]) -> String {
    let mut out = template.to_string();
    for (k, v) in params {
        let placeholder = format!("{{{}}}", k);
        if out.contains(&placeholder) {
            out = out.replace(&placeholder, v);
        }
    }
    unescape_literals(&out)
}

pub fn lookup(key: &str) -> Option<String> {
    TABLE
        .read()
        .ok()
        .and_then(|guard| guard.as_ref().and_then(|m| m.get(key).cloned()))
        .map(|msg| unescape_literals(&msg))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan050_i18n_lookup_loads_flat_json_and_misses_gracefully() {
        let dir = std::env::var("CARGO_MANIFEST_DIR")
            .map(std::path::PathBuf::from)
            .expect("manifest")
            .join("test/ui/plan050_stub_nil/src/front");
        load_from_dir(&dir);
        assert_eq!(lookup("settings.title").as_deref(), Some("设置"));
        assert_eq!(lookup("no.such.key"), None);
        // 未命中回落 key（求值臂行为,此处钉 lookup None 契约）
    }

    /// PLAN-054 T3 (R4): vue-i18n 字面量插值转义 `{'@'}` → `@`——
    /// musk inputPlaceholder 直出 "{@} 呼出 agent" 的根因。
    #[test]
    fn plan054_literal_interpolation_escape_unescaped() {
        assert_eq!(
            unescape_literals("(Enter 发送, {'@'} 呼出 agent, /relay 命令)"),
            "(Enter 发送, @ 呼出 agent, /relay 命令)"
        );
        // 多处 + 连续转义
        assert_eq!(unescape_literals("{'a'}{'b'}"), "ab");
        // 无闭合原样保留（防误吞用户文本）
        assert_eq!(unescape_literals("{'unclosed"), "{'unclosed");
        // 无转义零改动
        assert_eq!(unescape_literals("普通文案 {count} 条"), "普通文案 {count} 条");
        // substitute_params 出口同样消费（param 值路径）
        assert_eq!(
            substitute_params("{'@'} 呼出 {n} 个", &[("n".into(), "3".into())]),
            "@ 呼出 3 个"
        );
    }
}
