# AAVM v2(纯 Rust 模式)

Plan 432 起按 `docs/specs/aavm/` 规范逐模块移植写入本目录。
旧 v1 已归档至 `auto/lib-legacy/`。

Plan 434 增 `a2r.at`(AA2R:Auto 版 a2r 核心子集)——终极自举闭环:
Auto 写的 a2r 转译含自身的七文件 lib,产物为可独立 cargo build 的纯 Rust
(零 a2r_std),该 VM 运行 corpus_m4 30/30 与参考一致;五方对比矩阵见
`docs/specs/aavm/design/matrix-434.html`。

每个 `.at` 文件头使用 Snapshot 模板(见 docs/specs/aavm/design/divergence-rules.md §5)。

Plan 511(2026-09-01)增中阶语言能力:struct 类型定义(NEW_INSTANCE 族四层
收编)、全局变量、for-in 数组/表达式、字符串下标、一元负、`use` 模块化
(多编译单元+链接器+ev_run_files 多文件入口)。语料:c05/p25/p26/p27/t08/
b34–b43/corpus_use 六用例+errors 三件,五闸+多文件双闸+错误通道全绿;
Auto 侧单测 `test/vm/aavm2/99_unit/`(`scripts/gen-aavm2-unit.py` 再生成)。
规格:`docs/specs/aavm/design/midlang-w0-archaeology.md`。

Plan 514(2026-09-02)风格二期+方法化 γ4:AA2R 方法族发射(type 体方法/
ext/static new/接收者 .field 简写);lib 六文件方法化裁定完成——P(14)/
CG(28)/Ar(23)自有操作入 type 体,产生式与纯表函数保留自由函数;管道
算子 `x |> .m()`(方法形+字段投影形);塔顶自举=方法化 AA2R 自己转译
自己,rustc 零错。写法规范:docs/specs/aavm/design/divergence-rules.md
§4b;corpus_a2r g01–g17 逐字符对拍+五方矩阵 46/46。
