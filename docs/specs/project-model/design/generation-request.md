# generation-request — I-PROJECT 生成请求数据合同（PLAN-809 SD-02，as-built）

状态：work 树 canonical 草案（随 PLAN-809 merge 入主检出）。

## 合同

一次请求承载 UI 生成所需全部工程侧数据：origin（StaticAdmitted{source,unsupported}/DynamicEvaluated{source}）、project_root、identity/display/runtime/scene/members/render（RenderDecl）/toggles/theme_decl（ThemeDeclLite）/dep_roots/package_assets（npm_deps/styles/i18n 求值 Value）/exe_name/has_rust_sidecar/front_decl/input_sources/output_root/mode/raw+raw_nodes。

不变量：
- 无副作用（构造零网络/建目录/编译）；
- 无伪造求值（is_evaluated：静态 origin 需 unsupported 空；RenderSpec::Unknown 不假称前端）；
- 无字段丢失（未知键 raw 保留；B 组键 npm_deps/styles/i18n/exe_name/front/theme 块从 raw 目录回收）；
- 路径按 source 根纯代数（resolve_against_root，cwd 无关）；
- locale 链 display_title(prefers_zh) 镜像 Pac::display_title_in。

## 消费接线（as-built）

auto-man `generation_request_adapter`：request_from_pac（A 组全量投影）+ apply_evaluated_root/apply_evaluated_props（B 组，Pac.props 即求值 root 全量 props）+ request_of 单调用入口。生成器读缝：jet/ark（JetReads/ArkReads）、rust_ui（PacSource：name/exe/window/title/dep_roots/api/external/theme 激活）、vue（VueReads 九字段+Value 等价转换）；全部 `_with_request` 入口，legacy 自扫保留为冻结 oracle（R04 退役）。automan gen/run/build/open 分派全量经请求（零遗留 grep 审计）。

## 边界（明确不在本合同）

dep 自身 pac.at 的 npm_deps/styles 合并（vue 构建内跨文件读取）、sidecar spec 装载（load_sidecar）、api_gen 的 auto_lang resolve_back_dir——单工程外，R04 迁生成器时收编。

验证：project-model generation_request_tests（8）+ project-request-consumer（3+闭包单证）+ vue plan809_v02 双源对拍 + generation_request_adapter 2 完备性。
