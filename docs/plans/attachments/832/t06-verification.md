# PLAN-832 T-06 三仓完整 profile 与真实组合验证

冻结实现：Man `a8373e660f47ce1c0e90722c5240359155b81409`；lang 证据基线 `e4332bfaf128d587636e9822672205fbf42b1644`；UI `aee499e87093ff727f68ac8467c506e9ecf6ecdc`；down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。U831 的 archive、merge/清理收据及 ui-release 已读回，固定版本未被候选/WIP 替换。

## 1. full 安装与身份

- [t06-full-install-manifest.json.txt](t06-full-install-manifest.json.txt)：**6608 files**，双二进制、三仓/语言/down 来源与逐文件 hash；安装 metadata=37 packages、125 条本地 path 全在安装根内，`cargo metadata --locked` 通过。
- `auto.exe` SHA256=`eed5af33b2bfeef0d4f21a63a56c1ec82c5c4e3e2e3253ef6fb9292b8a2766e0`；`auto-ui-tool.exe` SHA256=`d74eba1298943ad9c2160206cf83b0fa5e417955b00b0700cc0fcbb234cfbc49`（`auto-ui-tool 0.1.0`）。
- 安装器补齐 down 非 crates 路径、patch 相对成员和正常 registry 锁解析；最终 locked metadata 与 manifest 均读回。无 UI source 被复制进 Man 仓本身，full 安装 SDK 是受管组合源。

## 2. 定向测试

- 真实 tool-client consumer：**30/30**，含 discovery、选项三层守门、真实生成/delivery、取消/超时/reader/孙进程；`verify-deps.sh` PASS。
- `plan816_cli_driver`：**7/7**——脚本 argv、真 tool gen、serve/req、watch SFC 再生、近截止 holder、根失败回收、超时 reader 真停。
- auto-man lib+全部集成：**33/33**（4+9+1+1+8+3+1+2+1+2+1），含 plan803/805/809/811/815/816。plan811 的真实媒体 fixture 已补入；include_str 指向固定语言源，不复制 api-gen。
- full auto/UI tool 均从冻结源 locked 构建成功；未运行整档 t/tv/tt/tb/tu/taa，merge 门 pending。

## 3. 安装态真实生成、构建与运行

均在清空 AUTO_* producer/root/target 覆盖后，直接使用安装目录二进制：

- `auto gen`：真实生成 `gen/front/vue` 17 文件；`auto build`：真实 pnpm/Vite 构建，产物 `gen/front/vue/dist/index.html`。
- Vue+VM：`auto run --server vm -F 4173 -B 4174` 启动成功，Playwright 首帧显示 `Hello, World!`。截图：`D:/autostack/.wt/lang-832/scratch-man-evidence-vue.png`（1280×800，不提交二进制）。
- VM/Iced：标准 `test_vm_mcp.py` 实跑，AURA snapshot 含 `Hello, World!`，select_rect 正反例与正常终止通过。截图：`D:/autostack/.wt/lang-832/scratch-man-full-consumer/001-helloworld/src/front/tests/screenshots/plan832-vm.png`（580×400，不提交二进制）。
- 独立服务：`auto service --server vm -B 4180` 的真实 `/__auto/health/ready`=200；结束后 4173/4180 均连接拒绝，未残留本次进程。

## 4. 兼容对照与资源修复

- 001 默认 `auto run`（Rust 后端要求）与 split-demo 工作区成员空请求，旧 CLI 与 Man CLI 均分别在相同阶段非零：前者 `backend Cargo.toml missing`，后者 `request declares []`。它们不是被冒称成功的正控；有效运行面采用现行支持的 `--server vm` 路径。
- plan805 九个 golden 的 EOF 字节属于冻结 oracle，先前 whitespace hygiene 已恢复；parity 通过，提交中明确登记这是有意的测试资产例外。
- 测试环境暴露的键差异（`TE_CHILD_BIN`/`AUTO_TE_CHILD_BIN`）与宿主预构建要求已按实际代码满足；开发态 delivery 用固定源，安装态全部清空覆盖。

## 5. 后续

T-07 负责独立 clone、fresh/locked/重定位、升级与坏源/半安装/取消/交换失败恢复。T-08 才切语言消费者并删除迁出原件；T-09 汇总 SD/handoff，merge 统一门 pending。
