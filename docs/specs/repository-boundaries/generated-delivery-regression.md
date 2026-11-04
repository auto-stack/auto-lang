# generated-delivery-regression（PLAN-930）

> **Status**: active（PLAN-930 SD-01）
> 范围：生成 UI 交付回归的长期守门——三具名原红（036 V04×2/V05a）的正确
> 成员身份、随包工具真实构建与固定负控；全链支持面不改弱交付契约。

## 正确成员身份（910 合同，V04）

- dev 模式成员目录为 `<basename>-<id8>`（路径身份、项目锚/cwd 无关、同
  basename 不同根不碰撞）；**定位只消费受管登记 member.json**（schema=
  `generated-member/v1`、role、project_path=canonical_key(项目路径)）——
  恰一个匹配才采信；缺失/错 owner/旧无身份目录/多候选一律具名拒绝，不按
  basename/mtime 认领、不 glob 第一个、不复刻 id8 哈希算法。
- dev/交付 main.rs 逐字节同源断言绑定**实际选中成员**（同一编译相位）。
- 长期守门：driver V04 两腿 + generated-delivery-controls 固定组
  （成员身份正/反控）。

## 随包工具真实构建（V05）

- 交付包 A/B 内 `cargo build --locked -p auto-ui-tools` 真实构建；B 的
  auto-ui-tool.exe 以 framed I-TOOL v1 真实外部消费（describe/validate/
  generate + 资源物化字节=包内源）；未知版本/不服务 profile 零副作用拒绝、
  坏帧流致命退出。
- 投影面：auto-ui-tools 对 auto-ui-runtime 的 `features=["autodown"]`
  请求在交付投影后失效（runtime 投影已把 autodown 剥为空 feature）——
  依赖行改写（`dep_rewrites`）剥除该请求，文档引擎回退普通编辑器面（与
  runtime 不进 SDK 的既有投影语义一致）；原形/投影形二选一校验，漂移判红。

## 固定负控与全链支持

- generated-delivery-controls 固定组（python runner）：真身份定位/产物
  比较/坏来源/子失败传播四类便宜控制；任一必需控制失败整体非 0；删 case
  判红（发现完备性自钉）。不默认发起重 Cargo/GUI、不编 ACE。
- 真实 A/B 构建与端到端仍由原 driver 显式执行（V01..V06），合成控制绿
  不替代真实构建。
- 消费侧 env 显式隔离：开发根覆盖（AUTO_*/CARGO_TARGET_DIR）剥离，
  RUSTC_WRAPPER 显式置空串（036 机器事实：置 sccache→0xc0000409、
  缺失→0xc0000142、空串→正常；不改用户机器配置/共享缓存）。
