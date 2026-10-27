# Plan 857 R1 修复与定向验证

复审基线 `ecf9939c5b2e369f9ce7d6ea25b0b66439bb7da6`；修复提交 `e20b95e175f9184e13c00c78efa7b63349165276`；唯一工作树 `D:/autostack/.wt/lang-857/auto-lang`、分支 `plan-857-dev`。SDK `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`、两锁不变。三项复审原反例在基线分别红，日志及以下结果的路径/SHA见 [evidence.json.txt](evidence.json.txt)。

| finding | 原不变量、修复和验证 | 当前结果/边界 |
|---|---|---|
| F-857-R1-01 | 实际presented矩阵的拾取和冻结轴射线使用双精度逆与计算，保留公开 `stable_inverse` 的原 f32 返回类型；宽视图用可测量的世界轴方向生成原54px手柄，避免单世界单位亚像素时误隐藏。 | 原1024/4096可见对象GPU/Host反例均绿，4096时有真实Auto Y轴拖动；±60000目标在正交/透视均可拾取并建立轴平面，原广视野/独立ray-plane、真零/非有限矩阵拒绝通过。日志 `large-target-controls.log.txt`；三项永久测试 `repair857_visible.rs`。不宣称任意极小对象可分辨。 |
| F-857-R1-02 | 组后代包围盒的12条边在齐次空间与近/远及正w裁剪，只有实际交叉深度范围的几何参与组框；不可用原因涵盖后代裁剪。 | 原正交近平面外/远平面外、透视远平面外不再有手柄；正交/透视可见控制与真实Host中跨近/远平面控制保持可操作；纯几何跨两边/完全外侧亦有控制。永久测试 `repair857_clip.rs`，旧3D组层级原回归 `hierarchy-group3-exact.log.txt` 通过。 |
| F-857-R1-03 | 3D捕获记录既有的orbit/pan来源；中键释放只结束中键pan，右键释放只结束右键orbit，其他release不改变捕获。 | 原Host双方向交叉release及正控通过；原有零移动释放、取消/blur/resize路径通过。实际OS→winit→Host窗口探针增加两方向交叉按键，34阶段及partial控通过；临时去掉来源判断后同一窗口门在phase28 `middle release ended right capture`具名红，恢复后绿。永久测试 `repair857_buttons.rs`、生产/消费端窗口报告见索引。 |
| F-857-R1-04 | 用856合入冻结producer `4150a8bea7b66aff9eca87a319d5cd1c6b4b50a7`、保留runner SHA `05372dfc...` 与修复提交的runner，同设备/1200×800/sample/80帧分别实际运行。 | 3D纹理SHA均 `3fd9b3ff3edf44dc58bcf392ea03bcb228ee223128f57af01e8ba29789f0acd7`；最终合成图的800×400三维区域，默认及真实Home重置逐RGB像素相等；单像素颜色变异被比较器拒绝。`default-comparison-committed/summary.json`、`compare-default-committed.log.txt`，脚本 [compare-default.py.txt](compare-default.py.txt)。UI面板可能不同，不宣称整张画面无差异；第一次比较器错误地把RGBA alpha差异用于getbbox，负控抓出后改RGB并保留失败记录。 |
| F-857-R1-05 | 从855交付 `05fdc81c546aff735eefdb6c39fbc5fb35dc546a` 已核来源复制隔离测量副本，与857隔离副本按相同窗口逻辑尺寸/场景/8热身+40帧、串行交替两轮观察CPU壁钟、进程工作集、静态mesh上传、VM/网格弱owner释放。 | [性能对照](../repair-r1-performance.json.txt) 含8次实际记录及SHA。855默认 p50 2071/1415μs、857默认1998/2101μs；857导航2968–2989μs、网格+导航3044–3182μs；记录逐次工作集，静态mesh均14次/136952字节且测量期不重传，owner关闭后释放。855无新导航，导航与grid是附加工作量，不能当等价负载；进程工作集波动大、无GPU timestamp、不推导跨机器预算。基准857副本稍早于最终公开兼容包装和组说明更改；性能场景无选中、拾取或组说明，证据文件明确列出源码差异和复用边界。 |

定向检查：`cargo check --locked`、受影响 `editor_viewport3d`、`viewport3d_decision`、`authoring_interaction`、三份复审反例测试及既有3D组回归；最终独立consumer从空target离线构建，20+1+3+3项通过，实际窗口及partial通过。来源清单475项、SDK 681文件/17 local/479包/29边和篡改/缺失恢复对照通过。旧SDK 146项warnings仍在，不称其被清偿。合入整档/完整双端/AGP/native-owner-content门均未执行，属于merge阶段；用户仅授权work修复，下一步独立复审本提交。
