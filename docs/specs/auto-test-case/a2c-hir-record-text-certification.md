# a2c-hir-record-text 认证合同（PLAN-925 CA-B 首例）

> recipe `ac-a2c/hir-record-text-emit-v1` | cert `ace.a2c-hir.record-text.cert/1`
> | fileset `auto.a2x.c.record-text.fileset/1` | 层 AXC2/AXP2

## 日常通道（spawn 零）

- 首例套件 `a2c-hir-record-text-925`（固定组同名，tools=[case-parent, a2x]）：
  worker `daily-emit-c` 单腿 emit，父适配器断言 recipe 回显、
  `spawned_rustc==0 && spawned_cl==0`、路径域审查后 4 文件 golden 字节判
  （main.c/a2x_c_rt_runtime.h/generated.c/manifest.json）。五例=R07/R08/T03/
  T05/C04（oracle 42/42/23/42/42）；对 nonce 确定。
- **golden manifest 的 .gitignore 豁免已登记**
  （`!.../a2c-hir-record-text-925/golden-*/manifest.json`）——merge 阶段抓出
  `*.json` 母规则吞掉 5 个 manifest（未跟踪即新鲜检出复现 PLAN-904 型
  inventory 红），已补跟踪并以"删盘→git 恢复→组绿"验证自足。

## 认证通道（真 cl 双腿，strict 配对证书）

- `certify-c`（profile record/text/text-record）：actual（生成集）与 caller 供
  **手写** C reference 各自真实 cl 编译运行，均须等于同一闭式 oracle
  （value:n / trap:Name）；reference 不匹配=`a2x.oracle` 拒。
- **证书身份**（`ace.a2c-hir.record-text.cert/1`）：沿 scalar cert 全字段 +
  **schema/recipe 严格按通道配对**——scalar recipe 配 RT schema（或反向）=
  REJECTED tamper；旧/外域 schema（ace.m1.cert/1、ace.a2r-hir.*）拒；裁
  actual_files、非 PASS verdict 拒。cert 绑 hir_domain 实际值（auto.hir.text-record
  等按 profile）。
- **覆盖面（测试为证，7 测）**：五源全双腿；modes/方法矩阵 8 例（Copy 快照/
  双 mut/view record/mut 文本/借返/owned 返/命名构造/双 owner 同名）各配内联
  手写 reference；trap 轴（TextIndex×2/InvalidScalar）kind=trap 认证+错 trap
  名拒；**坏 reference 反控**——错值（编译过、值错）拒于 oracle 腿、坏语法
  拒于编译腿（回归形状=reference 首版真实缺陷：字面 `\n` 落 C 源+c.len 字节/
  标量混算——独立 oracle 抓出）。
- 认证墙与采样：正式基面 512 真实不同修订全预认证（680s 错峰单实例）；
  日常不因缓存 PASS/旧 guest 值替现场判决。

## 账本分离

- 旧 M1 acceptance 不变；C RT 首例独立登记（套件+固定组+本合同）；CA-E 才
  定义总签章。旧 A2R cert-gap 豁免**不适用**于新 C 认证。
