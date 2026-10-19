# TAPL 静态全量清单（PLAN-813）

仅静态分析；所有 Auto 执行状态为 not_run。风险标签是排查线索，不是运行失败结论。

## 逐章计数

| 章节 | 语言 | 原始围栏（全部语言） | Listing 视图 | 独立 Auto 围栏 | Run/锁定位置 |
|---|---|---:|---:|---:|---:|
| appendix-a-keywords | zh | 0 | 0 | 0 | 0 |
| appendix-a-keywords | en | 0 | 0 | 0 | 0 |
| appendix-b-operators | zh | 0 | 0 | 0 | 0 |
| appendix-b-operators | en | 0 | 0 | 0 | 0 |
| appendix-c-transpiler-quick-ref | zh | 0 | 0 | 0 | 0 |
| appendix-c-transpiler-quick-ref | en | 0 | 0 | 0 | 0 |
| appendix-d-stdlib-index | zh | 0 | 0 | 0 | 0 |
| appendix-d-stdlib-index | en | 0 | 0 | 0 | 0 |
| ch00-introduction | zh | 0 | 0 | 0 | 0 |
| ch00-introduction | en | 0 | 0 | 0 | 0 |
| ch01-getting-started | zh | 23 | 2 | 2 | 4 |
| ch01-getting-started | en | 23 | 2 | 2 | 4 |
| ch02-variables-operators | zh | 40 | 6 | 10 | 16 |
| ch02-variables-operators | en | 40 | 6 | 10 | 16 |
| ch03-functions | zh | 52 | 8 | 11 | 19 |
| ch03-functions | en | 52 | 8 | 11 | 19 |
| ch04-collections | zh | 68 | 11 | 5 | 16 |
| ch04-collections | en | 68 | 11 | 5 | 16 |
| ch05-guessing-game | zh | 34 | 4 | 9 | 13 |
| ch05-guessing-game | en | 34 | 4 | 9 | 13 |
| ch06-types | zh | 46 | 7 | 4 | 11 |
| ch06-types | en | 46 | 7 | 4 | 11 |
| ch07-enums | zh | 42 | 7 | 0 | 7 |
| ch07-enums | en | 42 | 7 | 0 | 7 |
| ch08-oop | zh | 47 | 8 | 0 | 8 |
| ch08-oop | en | 47 | 8 | 0 | 8 |
| ch09-error-handling | zh | 49 | 9 | 1 | 10 |
| ch09-error-handling | en | 49 | 9 | 1 | 10 |
| ch10-modules | zh | 32 | 5 | 1 | 6 |
| ch10-modules | en | 32 | 5 | 1 | 6 |
| ch11-references | zh | 38 | 6 | 2 | 8 |
| ch11-references | en | 38 | 6 | 2 | 8 |
| ch12-memory | zh | 36 | 6 | 0 | 6 |
| ch12-memory | en | 36 | 6 | 0 | 6 |
| ch13-generics | zh | 42 | 7 | 1 | 8 |
| ch13-generics | en | 42 | 7 | 1 | 8 |
| ch14-file-processor | zh | 51 | 9 | 1 | 10 |
| ch14-file-processor | en | 51 | 9 | 1 | 10 |
| ch15-actors | zh | 41 | 7 | 0 | 7 |
| ch15-actors | en | 41 | 7 | 0 | 7 |
| ch16-async | zh | 35 | 6 | 0 | 6 |
| ch16-async | en | 35 | 6 | 0 | 6 |
| ch17-smart-casts | zh | 30 | 5 | 0 | 5 |
| ch17-smart-casts | en | 30 | 5 | 0 | 5 |
| ch18-testing | zh | 30 | 5 | 0 | 5 |
| ch18-testing | en | 30 | 5 | 0 | 5 |
| ch19-closures | zh | 35 | 7 | 0 | 7 |
| ch19-closures | en | 35 | 7 | 0 | 7 |
| ch20-comptime | zh | 25 | 5 | 0 | 5 |
| ch20-comptime | en | 25 | 5 | 0 | 5 |
| ch21-stdlib | zh | 70 | 14 | 0 | 14 |
| ch21-stdlib | en | 70 | 14 | 0 | 14 |
| ch22-chat-server | zh | 46 | 8 | 1 | 9 |
| ch22-chat-server | en | 51 | 9 | 1 | 10 |

## 每个 Run/锁定位置

| ID | 源文件:行 | 当前入口 | 静态风险线索 |
|---|---|---|---|
| zh/ch01-getting-started/listing-1-1 | tapl/ch01-getting-started.cn.md:99 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch01-getting-started/listing-1-2 | tapl/ch01-getting-started.cn.md:242 | CodeView Run (always shown on Auto tab) | non_auto_file_offered_as_auto_run |
| zh/ch01-getting-started/fence-014 | tapl/ch01-getting-started.cn.md:166 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch01-getting-started/fence-015 | tapl/ch01-getting-started.cn.md:178 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch01-getting-started/listing-1-1 | tapl/ch01-getting-started.md:133 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch01-getting-started/listing-1-2 | tapl/ch01-getting-started.md:319 | CodeView Run (always shown on Auto tab) | non_auto_file_offered_as_auto_run |
| en/ch01-getting-started/fence-014 | tapl/ch01-getting-started.md:203 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch01-getting-started/fence-015 | tapl/ch01-getting-started.md:220 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch02-variables-operators/listing-2-1 | tapl/ch02-variables-operators.cn.md:41 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch02-variables-operators/listing-2-2 | tapl/ch02-variables-operators.cn.md:158 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch02-variables-operators/listing-2-3 | tapl/ch02-variables-operators.cn.md:286 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch02-variables-operators/listing-2-4 | tapl/ch02-variables-operators.cn.md:397 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch02-variables-operators/listing-2-5 | tapl/ch02-variables-operators.cn.md:489 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch02-variables-operators/listing-2-6 | tapl/ch02-variables-operators.cn.md:597 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch02-variables-operators/fence-001 | tapl/ch02-variables-operators.cn.md:16 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch02-variables-operators/fence-002 | tapl/ch02-variables-operators.cn.md:30 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch02-variables-operators/fence-008 | tapl/ch02-variables-operators.cn.md:131 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch02-variables-operators/fence-009 | tapl/ch02-variables-operators.cn.md:152 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch02-variables-operators/fence-015 | tapl/ch02-variables-operators.cn.md:265 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch02-variables-operators/fence-016 | tapl/ch02-variables-operators.cn.md:271 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch02-variables-operators/fence-017 | tapl/ch02-variables-operators.cn.md:279 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch02-variables-operators/fence-023 | tapl/ch02-variables-operators.cn.md:378 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch02-variables-operators/fence-034 | tapl/ch02-variables-operators.cn.md:583 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch02-variables-operators/fence-035 | tapl/ch02-variables-operators.cn.md:590 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch02-variables-operators/listing-2-1 | tapl/ch02-variables-operators.md:51 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch02-variables-operators/listing-2-2 | tapl/ch02-variables-operators.md:176 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch02-variables-operators/listing-2-3 | tapl/ch02-variables-operators.md:310 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch02-variables-operators/listing-2-4 | tapl/ch02-variables-operators.md:426 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch02-variables-operators/listing-2-5 | tapl/ch02-variables-operators.md:525 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch02-variables-operators/listing-2-6 | tapl/ch02-variables-operators.md:641 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch02-variables-operators/fence-001 | tapl/ch02-variables-operators.md:19 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch02-variables-operators/fence-002 | tapl/ch02-variables-operators.md:37 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch02-variables-operators/fence-008 | tapl/ch02-variables-operators.md:146 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch02-variables-operators/fence-009 | tapl/ch02-variables-operators.md:170 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch02-variables-operators/fence-015 | tapl/ch02-variables-operators.md:287 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch02-variables-operators/fence-016 | tapl/ch02-variables-operators.md:294 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch02-variables-operators/fence-017 | tapl/ch02-variables-operators.md:303 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch02-variables-operators/fence-023 | tapl/ch02-variables-operators.md:407 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch02-variables-operators/fence-034 | tapl/ch02-variables-operators.md:623 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch02-variables-operators/fence-035 | tapl/ch02-variables-operators.md:632 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/listing-3-1 | tapl/ch03-functions.cn.md:44 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/listing-3-2 | tapl/ch03-functions.cn.md:148 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/listing-3-3 | tapl/ch03-functions.cn.md:257 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/listing-3-4 | tapl/ch03-functions.cn.md:393 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/listing-3-5 | tapl/ch03-functions.cn.md:488 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/listing-3-6 | tapl/ch03-functions.cn.md:569 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/listing-3-7 | tapl/ch03-functions.cn.md:666 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/listing-3-8 | tapl/ch03-functions.cn.md:795 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/fence-001 | tapl/ch03-functions.cn.md:20 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/fence-002 | tapl/ch03-functions.cn.md:30 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/fence-008 | tapl/ch03-functions.cn.md:133 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/fence-009 | tapl/ch03-functions.cn.md:143 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/fence-015 | tapl/ch03-functions.cn.md:235 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/fence-016 | tapl/ch03-functions.cn.md:243 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/fence-022 | tapl/ch03-functions.cn.md:377 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/fence-028 | tapl/ch03-functions.cn.md:476 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/fence-034 | tapl/ch03-functions.cn.md:560 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/fence-040 | tapl/ch03-functions.cn.md:654 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/fence-046 | tapl/ch03-functions.cn.md:783 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/listing-3-1 | tapl/ch03-functions.md:56 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/listing-3-2 | tapl/ch03-functions.md:168 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/listing-3-3 | tapl/ch03-functions.md:284 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/listing-3-4 | tapl/ch03-functions.md:427 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/listing-3-5 | tapl/ch03-functions.md:529 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/listing-3-6 | tapl/ch03-functions.md:615 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/listing-3-7 | tapl/ch03-functions.md:717 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/listing-3-8 | tapl/ch03-functions.md:852 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/fence-001 | tapl/ch03-functions.md:27 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/fence-002 | tapl/ch03-functions.md:39 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/fence-008 | tapl/ch03-functions.md:151 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/fence-009 | tapl/ch03-functions.md:163 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/fence-015 | tapl/ch03-functions.md:261 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/fence-016 | tapl/ch03-functions.md:269 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/fence-022 | tapl/ch03-functions.md:409 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/fence-028 | tapl/ch03-functions.md:514 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/fence-034 | tapl/ch03-functions.md:605 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/fence-040 | tapl/ch03-functions.md:705 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/fence-046 | tapl/ch03-functions.md:837 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch04-collections/listing-4-1 | tapl/ch04-collections.cn.md:25 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch04-collections/listing-4-2 | tapl/ch04-collections.cn.md:140 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch04-collections/listing-4-3 | tapl/ch04-collections.cn.md:247 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch04-collections/listing-4-4 | tapl/ch04-collections.cn.md:389 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch04-collections/listing-4-5 | tapl/ch04-collections.cn.md:534 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch04-collections/listing-4-6 | tapl/ch04-collections.cn.md:691 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch04-collections/listing-4-7 | tapl/ch04-collections.cn.md:834 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch04-collections/listing-4-8 | tapl/ch04-collections.cn.md:982 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch04-collections/listing-4-9 | tapl/ch04-collections.cn.md:1151 | CodeView Run (always shown on Auto tab) | markdown_and_listing_differ |
| zh/ch04-collections/listing-4-10 | tapl/ch04-collections.cn.md:1224 | CodeView Run (always shown on Auto tab) | markdown_and_listing_differ |
| zh/ch04-collections/listing-4-11 | tapl/ch04-collections.cn.md:1293 | CodeView Run (always shown on Auto tab) | markdown_and_listing_differ |
| zh/ch04-collections/fence-001 | tapl/ch04-collections.cn.md:19 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch04-collections/fence-008 | tapl/ch04-collections.cn.md:134 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch04-collections/fence-015 | tapl/ch04-collections.cn.md:241 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch04-collections/fence-022 | tapl/ch04-collections.cn.md:383 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch04-collections/fence-041 | tapl/ch04-collections.cn.md:825 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch04-collections/listing-4-1 | tapl/ch04-collections.md:34 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch04-collections/listing-4-2 | tapl/ch04-collections.md:157 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch04-collections/listing-4-3 | tapl/ch04-collections.md:271 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch04-collections/listing-4-4 | tapl/ch04-collections.md:425 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch04-collections/listing-4-5 | tapl/ch04-collections.md:581 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch04-collections/listing-4-6 | tapl/ch04-collections.md:740 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch04-collections/listing-4-7 | tapl/ch04-collections.md:893 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch04-collections/listing-4-8 | tapl/ch04-collections.md:1048 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch04-collections/listing-4-9 | tapl/ch04-collections.md:1224 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch04-collections/listing-4-10 | tapl/ch04-collections.md:1326 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch04-collections/listing-4-11 | tapl/ch04-collections.md:1433 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch04-collections/fence-001 | tapl/ch04-collections.md:25 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch04-collections/fence-008 | tapl/ch04-collections.md:148 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch04-collections/fence-015 | tapl/ch04-collections.md:263 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch04-collections/fence-022 | tapl/ch04-collections.md:417 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch04-collections/fence-041 | tapl/ch04-collections.md:881 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch05-guessing-game/listing-5-1 | tapl/ch05-guessing-game.cn.md:27 | CodeView Run (always shown on Auto tab) | missing_input_builtin |
| zh/ch05-guessing-game/listing-5-2 | tapl/ch05-guessing-game.cn.md:156 | CodeView Run (always shown on Auto tab) | missing_input_builtin, legacy_parse_random_names |
| zh/ch05-guessing-game/listing-5-3 | tapl/ch05-guessing-game.cn.md:357 | CodeView Run (always shown on Auto tab) | missing_input_builtin, legacy_parse_random_names, nonlocal_library_names_to_resolve |
| zh/ch05-guessing-game/listing-5-4 | tapl/ch05-guessing-game.cn.md:532 | CodeView Run (always shown on Auto tab) | missing_input_builtin, legacy_parse_random_names, nonlocal_library_names_to_resolve |
| zh/ch05-guessing-game/fence-007 | tapl/ch05-guessing-game.cn.md:110 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch05-guessing-game/fence-008 | tapl/ch05-guessing-game.cn.md:118 | AutoFence Run | missing_input_builtin |
| zh/ch05-guessing-game/fence-009 | tapl/ch05-guessing-game.cn.md:126 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch05-guessing-game/fence-016 | tapl/ch05-guessing-game.cn.md:298 | AutoFence Run | possible_placeholder_loop, ellipsis_needs_context_review |
| zh/ch05-guessing-game/fence-017 | tapl/ch05-guessing-game.cn.md:308 | AutoFence Run | legacy_parse_random_names |
| zh/ch05-guessing-game/fence-018 | tapl/ch05-guessing-game.cn.md:318 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch05-guessing-game/fence-025 | tapl/ch05-guessing-game.cn.md:502 | AutoFence Run | legacy_parse_random_names, nonlocal_library_names_to_resolve |
| zh/ch05-guessing-game/fence-032 | tapl/ch05-guessing-game.cn.md:712 | AutoFence Run | legacy_parse_random_names |
| zh/ch05-guessing-game/fence-033 | tapl/ch05-guessing-game.cn.md:733 | AutoFence Run | ellipsis_needs_context_review |
| en/ch05-guessing-game/listing-5-1 | tapl/ch05-guessing-game.md:36 | CodeView Run (always shown on Auto tab) | missing_input_builtin |
| en/ch05-guessing-game/listing-5-2 | tapl/ch05-guessing-game.md:180 | CodeView Run (always shown on Auto tab) | missing_input_builtin, legacy_parse_random_names |
| en/ch05-guessing-game/listing-5-3 | tapl/ch05-guessing-game.md:395 | CodeView Run (always shown on Auto tab) | missing_input_builtin, legacy_parse_random_names, nonlocal_library_names_to_resolve |
| en/ch05-guessing-game/listing-5-4 | tapl/ch05-guessing-game.md:578 | CodeView Run (always shown on Auto tab) | missing_input_builtin, legacy_parse_random_names, nonlocal_library_names_to_resolve |
| en/ch05-guessing-game/fence-007 | tapl/ch05-guessing-game.md:119 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch05-guessing-game/fence-008 | tapl/ch05-guessing-game.md:129 | AutoFence Run | missing_input_builtin |
| en/ch05-guessing-game/fence-009 | tapl/ch05-guessing-game.md:140 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch05-guessing-game/fence-016 | tapl/ch05-guessing-game.md:322 | AutoFence Run | possible_placeholder_loop, ellipsis_needs_context_review |
| en/ch05-guessing-game/fence-017 | tapl/ch05-guessing-game.md:336 | AutoFence Run | legacy_parse_random_names |
| en/ch05-guessing-game/fence-018 | tapl/ch05-guessing-game.md:350 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch05-guessing-game/fence-025 | tapl/ch05-guessing-game.md:540 | AutoFence Run | legacy_parse_random_names, nonlocal_library_names_to_resolve |
| en/ch05-guessing-game/fence-032 | tapl/ch05-guessing-game.md:758 | AutoFence Run | legacy_parse_random_names |
| en/ch05-guessing-game/fence-033 | tapl/ch05-guessing-game.md:788 | AutoFence Run | ellipsis_needs_context_review |
| zh/ch06-types/listing-6-1 | tapl/ch06-types.cn.md:23 | CodeView Run (always shown on Auto tab) | markdown_and_listing_differ |
| zh/ch06-types/listing-6-2 | tapl/ch06-types.cn.md:137 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch06-types/listing-6-3 | tapl/ch06-types.cn.md:267 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch06-types/listing-6-4 | tapl/ch06-types.cn.md:395 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch06-types/listing-6-5 | tapl/ch06-types.cn.md:524 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch06-types/listing-6-6 | tapl/ch06-types.cn.md:710 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch06-types/listing-6-7 | tapl/ch06-types.cn.md:828 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch06-types/fence-007 | tapl/ch06-types.cn.md:130 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch06-types/fence-014 | tapl/ch06-types.cn.md:257 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch06-types/fence-033 | tapl/ch06-types.cn.md:704 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch06-types/fence-040 | tapl/ch06-types.cn.md:821 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch06-types/listing-6-1 | tapl/ch06-types.md:37 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch06-types/listing-6-2 | tapl/ch06-types.md:167 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch06-types/listing-6-3 | tapl/ch06-types.md:314 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch06-types/listing-6-4 | tapl/ch06-types.md:453 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch06-types/listing-6-5 | tapl/ch06-types.md:591 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch06-types/listing-6-6 | tapl/ch06-types.md:792 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch06-types/listing-6-7 | tapl/ch06-types.md:925 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch06-types/fence-007 | tapl/ch06-types.md:158 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch06-types/fence-014 | tapl/ch06-types.md:301 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch06-types/fence-033 | tapl/ch06-types.md:785 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch06-types/fence-040 | tapl/ch06-types.md:915 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch07-enums/listing-7-1 | tapl/ch07-enums.cn.md:23 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch07-enums/listing-7-2 | tapl/ch07-enums.cn.md:159 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch07-enums/listing-7-3 | tapl/ch07-enums.cn.md:344 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch07-enums/listing-7-4 | tapl/ch07-enums.cn.md:549 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch07-enums/listing-7-5 | tapl/ch07-enums.cn.md:762 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch07-enums/listing-7-6 | tapl/ch07-enums.cn.md:903 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch07-enums/listing-7-7 | tapl/ch07-enums.cn.md:1051 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch07-enums/listing-7-1 | tapl/ch07-enums.md:35 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch07-enums/listing-7-2 | tapl/ch07-enums.md:181 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch07-enums/listing-7-3 | tapl/ch07-enums.md:374 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch07-enums/listing-7-4 | tapl/ch07-enums.md:586 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch07-enums/listing-7-5 | tapl/ch07-enums.md:806 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch07-enums/listing-7-6 | tapl/ch07-enums.md:956 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch07-enums/listing-7-7 | tapl/ch07-enums.md:1109 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch08-oop/listing-8-1 | tapl/ch08-oop.cn.md:45 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch08-oop/listing-8-2 | tapl/ch08-oop.cn.md:218 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch08-oop/listing-8-3 | tapl/ch08-oop.cn.md:408 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch08-oop/listing-8-4 | tapl/ch08-oop.cn.md:600 | CodeView Run (always shown on Auto tab) | markdown_and_listing_differ |
| zh/ch08-oop/listing-8-5 | tapl/ch08-oop.cn.md:673 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch08-oop/listing-8-6 | tapl/ch08-oop.cn.md:920 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch08-oop/listing-8-7 | tapl/ch08-oop.cn.md:1047 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch08-oop/listing-8-8 | tapl/ch08-oop.cn.md:1198 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch08-oop/listing-8-1 | tapl/ch08-oop.md:58 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch08-oop/listing-8-2 | tapl/ch08-oop.md:239 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch08-oop/listing-8-3 | tapl/ch08-oop.md:436 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch08-oop/listing-8-4 | tapl/ch08-oop.md:633 | CodeView Run (always shown on Auto tab) | markdown_and_listing_differ |
| en/ch08-oop/listing-8-5 | tapl/ch08-oop.md:713 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch08-oop/listing-8-6 | tapl/ch08-oop.md:971 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch08-oop/listing-8-7 | tapl/ch08-oop.md:1107 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch08-oop/listing-8-8 | tapl/ch08-oop.md:1263 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch09-error-handling/listing-9-1 | tapl/ch09-error-handling.cn.md:51 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch09-error-handling/listing-9-2 | tapl/ch09-error-handling.cn.md:222 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch09-error-handling/listing-9-3 | tapl/ch09-error-handling.cn.md:410 | CodeView Run (always shown on Auto tab) | legacy_parse_random_names, explicit_stub_comment |
| zh/ch09-error-handling/listing-9-4 | tapl/ch09-error-handling.cn.md:520 | CodeView Run (always shown on Auto tab) | nonlocal_library_names_to_resolve |
| zh/ch09-error-handling/listing-9-5 | tapl/ch09-error-handling.cn.md:656 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch09-error-handling/listing-9-6 | tapl/ch09-error-handling.cn.md:821 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch09-error-handling/listing-9-7 | tapl/ch09-error-handling.cn.md:960 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch09-error-handling/listing-9-8 | tapl/ch09-error-handling.cn.md:1169 | CodeView Run (always shown on Auto tab) | foreign_or_package_dependency |
| zh/ch09-error-handling/listing-9-9 | tapl/ch09-error-handling.cn.md:1244 | CodeView Run (always shown on Auto tab) | markdown_and_listing_differ |
| zh/ch09-error-handling/fence-012 | tapl/ch09-error-handling.cn.md:384 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch09-error-handling/listing-9-1 | tapl/ch09-error-handling.md:65 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch09-error-handling/listing-9-2 | tapl/ch09-error-handling.md:244 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch09-error-handling/listing-9-3 | tapl/ch09-error-handling.md:438 | CodeView Run (always shown on Auto tab) | legacy_parse_random_names, explicit_stub_comment |
| en/ch09-error-handling/listing-9-4 | tapl/ch09-error-handling.md:556 | CodeView Run (always shown on Auto tab) | nonlocal_library_names_to_resolve |
| en/ch09-error-handling/listing-9-5 | tapl/ch09-error-handling.md:698 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch09-error-handling/listing-9-6 | tapl/ch09-error-handling.md:870 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch09-error-handling/listing-9-7 | tapl/ch09-error-handling.md:1023 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch09-error-handling/listing-9-8 | tapl/ch09-error-handling.md:1238 | CodeView Run (always shown on Auto tab) | foreign_or_package_dependency |
| en/ch09-error-handling/listing-9-9 | tapl/ch09-error-handling.md:1321 | CodeView Run (always shown on Auto tab) | markdown_and_listing_differ |
| en/ch09-error-handling/fence-012 | tapl/ch09-error-handling.md:408 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch10-modules/listing-10-1 | tapl/ch10-modules.cn.md:31 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch10-modules/listing-10-2 | tapl/ch10-modules.cn.md:131 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch10-modules/listing-10-3 | tapl/ch10-modules.cn.md:207 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch10-modules/listing-10-4 | tapl/ch10-modules.cn.md:347 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch10-modules/listing-10-5 | tapl/ch10-modules.cn.md:496 | CodeView Run (always shown on Auto tab) | current_lock_regex_matches |
| zh/ch10-modules/fence-019 | tapl/ch10-modules.cn.md:322 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch10-modules/listing-10-1 | tapl/ch10-modules.md:38 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch10-modules/listing-10-2 | tapl/ch10-modules.md:142 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch10-modules/listing-10-3 | tapl/ch10-modules.md:221 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch10-modules/listing-10-4 | tapl/ch10-modules.md:364 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch10-modules/listing-10-5 | tapl/ch10-modules.md:520 | CodeView Run (always shown on Auto tab) | current_lock_regex_matches |
| en/ch10-modules/fence-019 | tapl/ch10-modules.md:337 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch11-references/listing-11-1 | tapl/ch11-references.cn.md:41 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch11-references/listing-11-2 | tapl/ch11-references.cn.md:164 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch11-references/listing-11-3 | tapl/ch11-references.cn.md:286 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch11-references/listing-11-4 | tapl/ch11-references.cn.md:394 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch11-references/listing-11-5 | tapl/ch11-references.cn.md:499 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch11-references/listing-11-6 | tapl/ch11-references.cn.md:665 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch11-references/fence-007 | tapl/ch11-references.cn.md:136 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch11-references/fence-038 | tapl/ch11-references.cn.md:789 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch11-references/listing-11-1 | tapl/ch11-references.md:50 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch11-references/listing-11-2 | tapl/ch11-references.md:179 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch11-references/listing-11-3 | tapl/ch11-references.md:302 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch11-references/listing-11-4 | tapl/ch11-references.md:419 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch11-references/listing-11-5 | tapl/ch11-references.md:527 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch11-references/listing-11-6 | tapl/ch11-references.md:700 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch11-references/fence-007 | tapl/ch11-references.md:146 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch11-references/fence-038 | tapl/ch11-references.md:829 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch12-memory/listing-12-1 | tapl/ch12-memory.cn.md:30 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch12-memory/listing-12-2 | tapl/ch12-memory.cn.md:142 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch12-memory/listing-12-3 | tapl/ch12-memory.cn.md:256 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch12-memory/listing-12-4 | tapl/ch12-memory.cn.md:374 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch12-memory/listing-12-5 | tapl/ch12-memory.cn.md:515 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch12-memory/listing-12-6 | tapl/ch12-memory.cn.md:664 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch12-memory/listing-12-1 | tapl/ch12-memory.md:38 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch12-memory/listing-12-2 | tapl/ch12-memory.md:156 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch12-memory/listing-12-3 | tapl/ch12-memory.md:275 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch12-memory/listing-12-4 | tapl/ch12-memory.md:399 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch12-memory/listing-12-5 | tapl/ch12-memory.md:548 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch12-memory/listing-12-6 | tapl/ch12-memory.md:706 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch13-generics/listing-13-1 | tapl/ch13-generics.cn.md:39 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch13-generics/listing-13-2 | tapl/ch13-generics.cn.md:199 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch13-generics/listing-13-3 | tapl/ch13-generics.cn.md:384 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch13-generics/listing-13-4 | tapl/ch13-generics.cn.md:554 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch13-generics/listing-13-5 | tapl/ch13-generics.cn.md:679 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch13-generics/listing-13-6 | tapl/ch13-generics.cn.md:831 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch13-generics/listing-13-7 | tapl/ch13-generics.cn.md:955 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch13-generics/fence-013 | tapl/ch13-generics.cn.md:366 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch13-generics/listing-13-1 | tapl/ch13-generics.md:49 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch13-generics/listing-13-2 | tapl/ch13-generics.md:216 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch13-generics/listing-13-3 | tapl/ch13-generics.md:406 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch13-generics/listing-13-4 | tapl/ch13-generics.md:579 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch13-generics/listing-13-5 | tapl/ch13-generics.md:708 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch13-generics/listing-13-6 | tapl/ch13-generics.md:863 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch13-generics/listing-13-7 | tapl/ch13-generics.md:993 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch13-generics/fence-013 | tapl/ch13-generics.md:386 | AutoFence Run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch14-file-processor/listing-14-1 | tapl/ch14-file-processor.cn.md:38 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch14-file-processor/listing-14-2 | tapl/ch14-file-processor.cn.md:196 | CodeView Run (always shown on Auto tab) | explicit_stub_comment, nonlocal_library_names_to_resolve |
| zh/ch14-file-processor/listing-14-3 | tapl/ch14-file-processor.cn.md:324 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch14-file-processor/listing-14-4 | tapl/ch14-file-processor.cn.md:513 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch14-file-processor/listing-14-5 | tapl/ch14-file-processor.cn.md:714 | CodeView Run (always shown on Auto tab) | nonlocal_library_names_to_resolve |
| zh/ch14-file-processor/listing-14-6 | tapl/ch14-file-processor.cn.md:960 | CodeView Run (always shown on Auto tab) | markdown_and_listing_differ |
| zh/ch14-file-processor/listing-14-7 | tapl/ch14-file-processor.cn.md:1056 | CodeView Run (always shown on Auto tab) | markdown_and_listing_differ |
| zh/ch14-file-processor/listing-14-8 | tapl/ch14-file-processor.cn.md:1164 | CodeView Run (always shown on Auto tab) | markdown_and_listing_differ |
| zh/ch14-file-processor/listing-14-9 | tapl/ch14-file-processor.cn.md:1244 | CodeView Run (always shown on Auto tab) | markdown_and_listing_differ |
| zh/ch14-file-processor/fence-031 | tapl/ch14-file-processor.cn.md:942 | AutoFence Run | nonlocal_library_names_to_resolve |
| en/ch14-file-processor/listing-14-1 | tapl/ch14-file-processor.md:43 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch14-file-processor/listing-14-2 | tapl/ch14-file-processor.md:203 | CodeView Run (always shown on Auto tab) | explicit_stub_comment, nonlocal_library_names_to_resolve |
| en/ch14-file-processor/listing-14-3 | tapl/ch14-file-processor.md:333 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch14-file-processor/listing-14-4 | tapl/ch14-file-processor.md:526 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch14-file-processor/listing-14-5 | tapl/ch14-file-processor.md:730 | CodeView Run (always shown on Auto tab) | nonlocal_library_names_to_resolve |
| en/ch14-file-processor/listing-14-6 | tapl/ch14-file-processor.md:978 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch14-file-processor/listing-14-7 | tapl/ch14-file-processor.md:1094 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch14-file-processor/listing-14-8 | tapl/ch14-file-processor.md:1222 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch14-file-processor/listing-14-9 | tapl/ch14-file-processor.md:1314 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch14-file-processor/fence-031 | tapl/ch14-file-processor.md:958 | AutoFence Run | nonlocal_library_names_to_resolve |
| zh/ch15-actors/listing-15-1 | tapl/ch15-actors.cn.md:41 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch15-actors/listing-15-2 | tapl/ch15-actors.cn.md:180 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch15-actors/listing-15-3 | tapl/ch15-actors.cn.md:383 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch15-actors/listing-15-4 | tapl/ch15-actors.cn.md:546 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch15-actors/listing-15-5 | tapl/ch15-actors.cn.md:738 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch15-actors/listing-15-6 | tapl/ch15-actors.cn.md:889 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch15-actors/listing-15-7 | tapl/ch15-actors.cn.md:1119 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch15-actors/listing-15-1 | tapl/ch15-actors.md:49 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch15-actors/listing-15-2 | tapl/ch15-actors.md:192 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch15-actors/listing-15-3 | tapl/ch15-actors.md:397 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch15-actors/listing-15-4 | tapl/ch15-actors.md:563 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch15-actors/listing-15-5 | tapl/ch15-actors.md:759 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch15-actors/listing-15-6 | tapl/ch15-actors.md:913 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch15-actors/listing-15-7 | tapl/ch15-actors.md:1146 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch16-async/listing-16-1 | tapl/ch16-async.cn.md:42 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch16-async/listing-16-2 | tapl/ch16-async.cn.md:189 | CodeView Run (always shown on Auto tab) | explicit_stub_comment |
| zh/ch16-async/listing-16-3 | tapl/ch16-async.cn.md:336 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch16-async/listing-16-4 | tapl/ch16-async.cn.md:553 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch16-async/listing-16-5 | tapl/ch16-async.cn.md:751 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch16-async/listing-16-6 | tapl/ch16-async.cn.md:1022 | CodeView Run (always shown on Auto tab) | explicit_stub_comment |
| en/ch16-async/listing-16-1 | tapl/ch16-async.md:49 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch16-async/listing-16-2 | tapl/ch16-async.md:202 | CodeView Run (always shown on Auto tab) | explicit_stub_comment |
| en/ch16-async/listing-16-3 | tapl/ch16-async.md:354 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch16-async/listing-16-4 | tapl/ch16-async.md:576 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch16-async/listing-16-5 | tapl/ch16-async.md:778 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch16-async/listing-16-6 | tapl/ch16-async.md:1055 | CodeView Run (always shown on Auto tab) | explicit_stub_comment |
| zh/ch17-smart-casts/listing-17-1 | tapl/ch17-smart-casts.cn.md:43 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch17-smart-casts/listing-17-2 | tapl/ch17-smart-casts.cn.md:194 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch17-smart-casts/listing-17-3 | tapl/ch17-smart-casts.cn.md:341 | CodeView Run (always shown on Auto tab) | explicit_stub_comment |
| zh/ch17-smart-casts/listing-17-4 | tapl/ch17-smart-casts.cn.md:548 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch17-smart-casts/listing-17-5 | tapl/ch17-smart-casts.cn.md:702 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch17-smart-casts/listing-17-1 | tapl/ch17-smart-casts.md:49 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch17-smart-casts/listing-17-2 | tapl/ch17-smart-casts.md:204 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch17-smart-casts/listing-17-3 | tapl/ch17-smart-casts.md:358 | CodeView Run (always shown on Auto tab) | explicit_stub_comment |
| en/ch17-smart-casts/listing-17-4 | tapl/ch17-smart-casts.md:573 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch17-smart-casts/listing-17-5 | tapl/ch17-smart-casts.md:731 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch18-testing/listing-18-1 | tapl/ch18-testing.cn.md:36 | CodeView Run (always shown on Auto tab) | test_runner_required |
| zh/ch18-testing/listing-18-2 | tapl/ch18-testing.cn.md:119 | CodeView Run (always shown on Auto tab) | test_runner_required |
| zh/ch18-testing/listing-18-3 | tapl/ch18-testing.cn.md:225 | CodeView Run (always shown on Auto tab) | test_runner_required |
| zh/ch18-testing/listing-18-4 | tapl/ch18-testing.cn.md:424 | CodeView Run (always shown on Auto tab) | test_runner_required |
| zh/ch18-testing/listing-18-5 | tapl/ch18-testing.cn.md:628 | CodeView Run (always shown on Auto tab) | test_runner_required |
| en/ch18-testing/listing-18-1 | tapl/ch18-testing.md:44 | CodeView Run (always shown on Auto tab) | test_runner_required |
| en/ch18-testing/listing-18-2 | tapl/ch18-testing.md:133 | CodeView Run (always shown on Auto tab) | test_runner_required |
| en/ch18-testing/listing-18-3 | tapl/ch18-testing.md:244 | CodeView Run (always shown on Auto tab) | test_runner_required |
| en/ch18-testing/listing-18-4 | tapl/ch18-testing.md:448 | CodeView Run (always shown on Auto tab) | test_runner_required |
| en/ch18-testing/listing-18-5 | tapl/ch18-testing.md:658 | CodeView Run (always shown on Auto tab) | test_runner_required |
| zh/ch19-closures/listing-19-1 | tapl/ch19-closures.cn.md:23 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch19-closures/listing-19-2 | tapl/ch19-closures.cn.md:149 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch19-closures/listing-19-3 | tapl/ch19-closures.cn.md:282 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch19-closures/listing-19-4 | tapl/ch19-closures.cn.md:412 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch19-closures/listing-19-5 | tapl/ch19-closures.cn.md:511 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch19-closures/listing-19-6 | tapl/ch19-closures.cn.md:615 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch19-closures/listing-19-7 | tapl/ch19-closures.cn.md:728 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch19-closures/listing-19-1 | tapl/ch19-closures.md:28 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch19-closures/listing-19-2 | tapl/ch19-closures.md:161 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch19-closures/listing-19-3 | tapl/ch19-closures.md:300 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch19-closures/listing-19-4 | tapl/ch19-closures.md:435 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch19-closures/listing-19-5 | tapl/ch19-closures.md:539 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch19-closures/listing-19-6 | tapl/ch19-closures.md:647 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch19-closures/listing-19-7 | tapl/ch19-closures.md:765 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch20-comptime/listing-20-1 | tapl/ch20-comptime.cn.md:41 | CodeView Run (always shown on Auto tab) | test_runner_required |
| zh/ch20-comptime/listing-20-2 | tapl/ch20-comptime.cn.md:171 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch20-comptime/listing-20-3 | tapl/ch20-comptime.cn.md:295 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch20-comptime/listing-20-4 | tapl/ch20-comptime.cn.md:427 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch20-comptime/listing-20-5 | tapl/ch20-comptime.cn.md:575 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch20-comptime/listing-20-1 | tapl/ch20-comptime.md:52 | CodeView Run (always shown on Auto tab) | test_runner_required |
| en/ch20-comptime/listing-20-2 | tapl/ch20-comptime.md:189 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch20-comptime/listing-20-3 | tapl/ch20-comptime.md:321 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch20-comptime/listing-20-4 | tapl/ch20-comptime.md:460 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch20-comptime/listing-20-5 | tapl/ch20-comptime.md:616 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch21-stdlib/listing-21-1 | tapl/ch21-stdlib.cn.md:22 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch21-stdlib/listing-21-2 | tapl/ch21-stdlib.cn.md:243 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch21-stdlib/listing-21-3 | tapl/ch21-stdlib.cn.md:378 | CodeView Run (always shown on Auto tab) | nonlocal_library_names_to_resolve |
| zh/ch21-stdlib/listing-21-4 | tapl/ch21-stdlib.cn.md:554 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch21-stdlib/listing-21-5 | tapl/ch21-stdlib.cn.md:773 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch21-stdlib/listing-21-6 | tapl/ch21-stdlib.cn.md:945 | CodeView Run (always shown on Auto tab) | nonlocal_library_names_to_resolve |
| zh/ch21-stdlib/listing-21-7 | tapl/ch21-stdlib.cn.md:1127 | CodeView Run (always shown on Auto tab) | foreign_or_package_dependency, nonlocal_library_names_to_resolve |
| zh/ch21-stdlib/listing-21-8 | tapl/ch21-stdlib.cn.md:1259 | CodeView Run (always shown on Auto tab) | nonlocal_library_names_to_resolve |
| zh/ch21-stdlib/listing-21-9 | tapl/ch21-stdlib.cn.md:1356 | CodeView Run (always shown on Auto tab) | nonlocal_library_names_to_resolve |
| zh/ch21-stdlib/listing-21-10 | tapl/ch21-stdlib.cn.md:1437 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch21-stdlib/listing-21-11 | tapl/ch21-stdlib.cn.md:1534 | CodeView Run (always shown on Auto tab) | explicit_stub_comment |
| zh/ch21-stdlib/listing-21-12 | tapl/ch21-stdlib.cn.md:1656 | CodeView Run (always shown on Auto tab) | foreign_or_package_dependency, explicit_stub_comment |
| zh/ch21-stdlib/listing-21-13 | tapl/ch21-stdlib.cn.md:1741 | CodeView Run (always shown on Auto tab) | foreign_or_package_dependency |
| zh/ch21-stdlib/listing-21-14 | tapl/ch21-stdlib.cn.md:1821 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch21-stdlib/listing-21-1 | tapl/ch21-stdlib.md:27 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch21-stdlib/listing-21-2 | tapl/ch21-stdlib.md:250 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch21-stdlib/listing-21-3 | tapl/ch21-stdlib.md:388 | CodeView Run (always shown on Auto tab) | nonlocal_library_names_to_resolve |
| en/ch21-stdlib/listing-21-4 | tapl/ch21-stdlib.md:567 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch21-stdlib/listing-21-5 | tapl/ch21-stdlib.md:789 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch21-stdlib/listing-21-6 | tapl/ch21-stdlib.md:965 | CodeView Run (always shown on Auto tab) | nonlocal_library_names_to_resolve |
| en/ch21-stdlib/listing-21-7 | tapl/ch21-stdlib.md:1149 | CodeView Run (always shown on Auto tab) | foreign_or_package_dependency, nonlocal_library_names_to_resolve |
| en/ch21-stdlib/listing-21-8 | tapl/ch21-stdlib.md:1281 | CodeView Run (always shown on Auto tab) | nonlocal_library_names_to_resolve |
| en/ch21-stdlib/listing-21-9 | tapl/ch21-stdlib.md:1378 | CodeView Run (always shown on Auto tab) | nonlocal_library_names_to_resolve |
| en/ch21-stdlib/listing-21-10 | tapl/ch21-stdlib.md:1459 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch21-stdlib/listing-21-11 | tapl/ch21-stdlib.md:1558 | CodeView Run (always shown on Auto tab) | explicit_stub_comment |
| en/ch21-stdlib/listing-21-12 | tapl/ch21-stdlib.md:1681 | CodeView Run (always shown on Auto tab) | foreign_or_package_dependency, explicit_stub_comment |
| en/ch21-stdlib/listing-21-13 | tapl/ch21-stdlib.md:1766 | CodeView Run (always shown on Auto tab) | foreign_or_package_dependency |
| en/ch21-stdlib/listing-21-14 | tapl/ch21-stdlib.md:1848 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch22-chat-server/listing-22-1 | tapl/ch22-chat-server.cn.md:56 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch22-chat-server/listing-22-2 | tapl/ch22-chat-server.cn.md:437 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch22-chat-server/listing-22-3 | tapl/ch22-chat-server.cn.md:769 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch22-chat-server/listing-22-4 | tapl/ch22-chat-server.cn.md:1051 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch22-chat-server/listing-22-5 | tapl/ch22-chat-server.cn.md:1583 | CodeView Run (always shown on Auto tab) | current_lock_regex_matches, ellipsis_needs_context_review |
| zh/ch22-chat-server/listing-22-7 | tapl/ch22-chat-server.cn.md:2151 | CodeView Run (always shown on Auto tab) | markdown_and_listing_differ |
| zh/ch22-chat-server/listing-22-8 | tapl/ch22-chat-server.cn.md:2204 | CodeView Run (always shown on Auto tab) | foreign_or_package_dependency, markdown_and_listing_differ |
| zh/ch22-chat-server/listing-22-9 | tapl/ch22-chat-server.cn.md:2263 | CodeView Run (always shown on Auto tab) | ellipsis_needs_context_review, explicit_stub_comment, markdown_and_listing_differ |
| zh/ch22-chat-server/fence-031 | tapl/ch22-chat-server.cn.md:2057 | AutoFence Run | test_runner_required |
| en/ch22-chat-server/listing-22-1 | tapl/ch22-chat-server.md:66 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch22-chat-server/listing-22-2 | tapl/ch22-chat-server.md:451 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch22-chat-server/listing-22-3 | tapl/ch22-chat-server.md:786 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch22-chat-server/listing-22-4 | tapl/ch22-chat-server.md:1074 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch22-chat-server/listing-22-5 | tapl/ch22-chat-server.md:1620 | CodeView Run (always shown on Auto tab) | current_lock_regex_matches, ellipsis_needs_context_review |
| en/ch22-chat-server/listing-22-6 | tapl/ch22-chat-server.md:2056 | CodeView Run (always shown on Auto tab) | current_lock_regex_matches, ellipsis_needs_context_review |
| en/ch22-chat-server/listing-22-7 | tapl/ch22-chat-server.md:2864 | CodeView Run (always shown on Auto tab) | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch22-chat-server/listing-22-8 | tapl/ch22-chat-server.md:2924 | CodeView Run (always shown on Auto tab) | foreign_or_package_dependency, markdown_and_listing_differ |
| en/ch22-chat-server/listing-22-9 | tapl/ch22-chat-server.md:2997 | CodeView Run (always shown on Auto tab) | ellipsis_needs_context_review, explicit_stub_comment, markdown_and_listing_differ |
| en/ch22-chat-server/fence-036 | tapl/ch22-chat-server.md:2756 | AutoFence Run | test_runner_required |

非 Auto 围栏和被 Listing 替换的嵌套围栏也已逐一记录于 static-inventory.csv / .json；
input_hashes 绑定书稿、实际运行源、包声明及已有金样。
