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
| ch22-chat-server | zh | 47 | 11 | 1 | 12 |
| ch22-chat-server | en | 52 | 11 | 1 | 12 |

## 每个 Run/锁定位置

| ID | 源文件:行 | 当前入口 | 静态风险线索 |
|---|---|---|---|
| zh/ch01-getting-started/listing-1-1 | tapl/ch01-getting-started.cn.md:99 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch01-getting-started/listing-1-2 | tapl/ch01-getting-started.cn.md:242 | CodeView display-only | non_auto_file_offered_as_auto_run |
| zh/ch01-getting-started/fence-014 | tapl/ch01-getting-started.cn.md:166 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch01-getting-started/fence-015 | tapl/ch01-getting-started.cn.md:178 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch01-getting-started/listing-1-1 | tapl/ch01-getting-started.md:133 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch01-getting-started/listing-1-2 | tapl/ch01-getting-started.md:319 | CodeView display-only | non_auto_file_offered_as_auto_run |
| en/ch01-getting-started/fence-014 | tapl/ch01-getting-started.md:203 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch01-getting-started/fence-015 | tapl/ch01-getting-started.md:220 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch02-variables-operators/listing-2-1 | tapl/ch02-variables-operators.cn.md:41 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch02-variables-operators/listing-2-2 | tapl/ch02-variables-operators.cn.md:158 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch02-variables-operators/listing-2-3 | tapl/ch02-variables-operators.cn.md:286 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch02-variables-operators/listing-2-4 | tapl/ch02-variables-operators.cn.md:397 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch02-variables-operators/listing-2-5 | tapl/ch02-variables-operators.cn.md:489 | CodeView run | markdown_and_listing_differ |
| zh/ch02-variables-operators/listing-2-6 | tapl/ch02-variables-operators.cn.md:597 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch02-variables-operators/fence-001 | tapl/ch02-variables-operators.cn.md:16 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch02-variables-operators/fence-002 | tapl/ch02-variables-operators.cn.md:30 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch02-variables-operators/fence-008 | tapl/ch02-variables-operators.cn.md:131 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch02-variables-operators/fence-009 | tapl/ch02-variables-operators.cn.md:152 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch02-variables-operators/fence-015 | tapl/ch02-variables-operators.cn.md:265 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch02-variables-operators/fence-016 | tapl/ch02-variables-operators.cn.md:271 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch02-variables-operators/fence-017 | tapl/ch02-variables-operators.cn.md:279 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch02-variables-operators/fence-023 | tapl/ch02-variables-operators.cn.md:378 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch02-variables-operators/fence-034 | tapl/ch02-variables-operators.cn.md:583 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch02-variables-operators/fence-035 | tapl/ch02-variables-operators.cn.md:590 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch02-variables-operators/listing-2-1 | tapl/ch02-variables-operators.md:51 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch02-variables-operators/listing-2-2 | tapl/ch02-variables-operators.md:176 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch02-variables-operators/listing-2-3 | tapl/ch02-variables-operators.md:310 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch02-variables-operators/listing-2-4 | tapl/ch02-variables-operators.md:426 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch02-variables-operators/listing-2-5 | tapl/ch02-variables-operators.md:525 | CodeView run | markdown_and_listing_differ |
| en/ch02-variables-operators/listing-2-6 | tapl/ch02-variables-operators.md:641 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch02-variables-operators/fence-001 | tapl/ch02-variables-operators.md:19 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch02-variables-operators/fence-002 | tapl/ch02-variables-operators.md:37 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch02-variables-operators/fence-008 | tapl/ch02-variables-operators.md:146 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch02-variables-operators/fence-009 | tapl/ch02-variables-operators.md:170 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch02-variables-operators/fence-015 | tapl/ch02-variables-operators.md:287 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch02-variables-operators/fence-016 | tapl/ch02-variables-operators.md:294 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch02-variables-operators/fence-017 | tapl/ch02-variables-operators.md:303 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch02-variables-operators/fence-023 | tapl/ch02-variables-operators.md:407 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch02-variables-operators/fence-034 | tapl/ch02-variables-operators.md:623 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch02-variables-operators/fence-035 | tapl/ch02-variables-operators.md:632 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/listing-3-1 | tapl/ch03-functions.cn.md:44 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/listing-3-2 | tapl/ch03-functions.cn.md:148 | CodeView run | markdown_and_listing_differ |
| zh/ch03-functions/listing-3-3 | tapl/ch03-functions.cn.md:257 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/listing-3-4 | tapl/ch03-functions.cn.md:393 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/listing-3-5 | tapl/ch03-functions.cn.md:488 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/listing-3-6 | tapl/ch03-functions.cn.md:569 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/listing-3-7 | tapl/ch03-functions.cn.md:666 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/listing-3-8 | tapl/ch03-functions.cn.md:795 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/fence-001 | tapl/ch03-functions.cn.md:20 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/fence-002 | tapl/ch03-functions.cn.md:30 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/fence-008 | tapl/ch03-functions.cn.md:133 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/fence-009 | tapl/ch03-functions.cn.md:143 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/fence-015 | tapl/ch03-functions.cn.md:235 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/fence-016 | tapl/ch03-functions.cn.md:243 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/fence-022 | tapl/ch03-functions.cn.md:377 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/fence-028 | tapl/ch03-functions.cn.md:476 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/fence-034 | tapl/ch03-functions.cn.md:560 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/fence-040 | tapl/ch03-functions.cn.md:654 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch03-functions/fence-046 | tapl/ch03-functions.cn.md:783 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/listing-3-1 | tapl/ch03-functions.md:56 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/listing-3-2 | tapl/ch03-functions.md:168 | CodeView run | markdown_and_listing_differ |
| en/ch03-functions/listing-3-3 | tapl/ch03-functions.md:284 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/listing-3-4 | tapl/ch03-functions.md:427 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/listing-3-5 | tapl/ch03-functions.md:529 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/listing-3-6 | tapl/ch03-functions.md:615 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/listing-3-7 | tapl/ch03-functions.md:717 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/listing-3-8 | tapl/ch03-functions.md:852 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/fence-001 | tapl/ch03-functions.md:27 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/fence-002 | tapl/ch03-functions.md:39 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/fence-008 | tapl/ch03-functions.md:151 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/fence-009 | tapl/ch03-functions.md:163 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/fence-015 | tapl/ch03-functions.md:261 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/fence-016 | tapl/ch03-functions.md:269 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/fence-022 | tapl/ch03-functions.md:409 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/fence-028 | tapl/ch03-functions.md:514 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/fence-034 | tapl/ch03-functions.md:605 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/fence-040 | tapl/ch03-functions.md:705 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch03-functions/fence-046 | tapl/ch03-functions.md:837 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch04-collections/listing-4-1 | tapl/ch04-collections.cn.md:25 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch04-collections/listing-4-2 | tapl/ch04-collections.cn.md:140 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch04-collections/listing-4-3 | tapl/ch04-collections.cn.md:250 | CodeView run | markdown_and_listing_differ |
| zh/ch04-collections/listing-4-4 | tapl/ch04-collections.cn.md:397 | CodeView run | markdown_and_listing_differ |
| zh/ch04-collections/listing-4-5 | tapl/ch04-collections.cn.md:542 | CodeView run | markdown_and_listing_differ |
| zh/ch04-collections/listing-4-6 | tapl/ch04-collections.cn.md:707 | CodeView run | markdown_and_listing_differ |
| zh/ch04-collections/listing-4-7 | tapl/ch04-collections.cn.md:848 | CodeView run | markdown_and_listing_differ |
| zh/ch04-collections/listing-4-8 | tapl/ch04-collections.cn.md:996 | CodeView run | markdown_and_listing_differ |
| zh/ch04-collections/listing-4-9 | tapl/ch04-collections.cn.md:1165 | CodeView run | markdown_and_listing_differ |
| zh/ch04-collections/listing-4-10 | tapl/ch04-collections.cn.md:1238 | CodeView run | markdown_and_listing_differ |
| zh/ch04-collections/listing-4-11 | tapl/ch04-collections.cn.md:1307 | CodeView run | markdown_and_listing_differ |
| zh/ch04-collections/fence-001 | tapl/ch04-collections.cn.md:19 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch04-collections/fence-008 | tapl/ch04-collections.cn.md:134 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch04-collections/fence-015 | tapl/ch04-collections.cn.md:241 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch04-collections/fence-022 | tapl/ch04-collections.cn.md:386 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch04-collections/fence-041 | tapl/ch04-collections.cn.md:842 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch04-collections/listing-4-1 | tapl/ch04-collections.md:34 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch04-collections/listing-4-2 | tapl/ch04-collections.md:157 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch04-collections/listing-4-3 | tapl/ch04-collections.md:274 | CodeView run | markdown_and_listing_differ |
| en/ch04-collections/listing-4-4 | tapl/ch04-collections.md:433 | CodeView run | markdown_and_listing_differ |
| en/ch04-collections/listing-4-5 | tapl/ch04-collections.md:589 | CodeView run | markdown_and_listing_differ |
| en/ch04-collections/listing-4-6 | tapl/ch04-collections.md:756 | CodeView run | markdown_and_listing_differ |
| en/ch04-collections/listing-4-7 | tapl/ch04-collections.md:907 | CodeView run | markdown_and_listing_differ |
| en/ch04-collections/listing-4-8 | tapl/ch04-collections.md:1062 | CodeView run | markdown_and_listing_differ |
| en/ch04-collections/listing-4-9 | tapl/ch04-collections.md:1238 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch04-collections/listing-4-10 | tapl/ch04-collections.md:1340 | CodeView run | markdown_and_listing_differ |
| en/ch04-collections/listing-4-11 | tapl/ch04-collections.md:1447 | CodeView run | markdown_and_listing_differ |
| en/ch04-collections/fence-001 | tapl/ch04-collections.md:25 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch04-collections/fence-008 | tapl/ch04-collections.md:148 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch04-collections/fence-015 | tapl/ch04-collections.md:263 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch04-collections/fence-022 | tapl/ch04-collections.md:420 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch04-collections/fence-041 | tapl/ch04-collections.md:898 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch05-guessing-game/listing-5-1 | tapl/ch05-guessing-game.cn.md:27 | CodeView run | current_lock_regex_matches, markdown_and_listing_differ |
| zh/ch05-guessing-game/listing-5-2 | tapl/ch05-guessing-game.cn.md:156 | CodeView run | current_lock_regex_matches, markdown_and_listing_differ |
| zh/ch05-guessing-game/listing-5-3 | tapl/ch05-guessing-game.cn.md:356 | CodeView run | current_lock_regex_matches, markdown_and_listing_differ |
| zh/ch05-guessing-game/listing-5-4 | tapl/ch05-guessing-game.cn.md:529 | CodeView run | current_lock_regex_matches, markdown_and_listing_differ |
| zh/ch05-guessing-game/fence-007 | tapl/ch05-guessing-game.cn.md:110 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch05-guessing-game/fence-008 | tapl/ch05-guessing-game.cn.md:118 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch05-guessing-game/fence-009 | tapl/ch05-guessing-game.cn.md:126 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch05-guessing-game/fence-016 | tapl/ch05-guessing-game.cn.md:298 | AutoFence display-only | possible_placeholder_loop, ellipsis_needs_context_review |
| zh/ch05-guessing-game/fence-017 | tapl/ch05-guessing-game.cn.md:308 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch05-guessing-game/fence-018 | tapl/ch05-guessing-game.cn.md:318 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch05-guessing-game/fence-025 | tapl/ch05-guessing-game.cn.md:501 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch05-guessing-game/fence-032 | tapl/ch05-guessing-game.cn.md:709 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch05-guessing-game/fence-033 | tapl/ch05-guessing-game.cn.md:730 | AutoFence run | ellipsis_needs_context_review |
| en/ch05-guessing-game/listing-5-1 | tapl/ch05-guessing-game.md:36 | CodeView run | current_lock_regex_matches, markdown_and_listing_differ |
| en/ch05-guessing-game/listing-5-2 | tapl/ch05-guessing-game.md:177 | CodeView run | current_lock_regex_matches, markdown_and_listing_differ |
| en/ch05-guessing-game/listing-5-3 | tapl/ch05-guessing-game.md:386 | CodeView run | current_lock_regex_matches, markdown_and_listing_differ |
| en/ch05-guessing-game/listing-5-4 | tapl/ch05-guessing-game.md:562 | CodeView run | current_lock_regex_matches, markdown_and_listing_differ |
| en/ch05-guessing-game/fence-007 | tapl/ch05-guessing-game.md:119 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch05-guessing-game/fence-008 | tapl/ch05-guessing-game.md:129 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch05-guessing-game/fence-009 | tapl/ch05-guessing-game.md:137 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch05-guessing-game/fence-016 | tapl/ch05-guessing-game.md:319 | AutoFence display-only | possible_placeholder_loop, ellipsis_needs_context_review |
| en/ch05-guessing-game/fence-017 | tapl/ch05-guessing-game.md:333 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch05-guessing-game/fence-018 | tapl/ch05-guessing-game.md:343 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch05-guessing-game/fence-025 | tapl/ch05-guessing-game.md:531 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch05-guessing-game/fence-032 | tapl/ch05-guessing-game.md:742 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch05-guessing-game/fence-033 | tapl/ch05-guessing-game.md:772 | AutoFence run | ellipsis_needs_context_review |
| zh/ch06-types/listing-6-1 | tapl/ch06-types.cn.md:23 | CodeView run | markdown_and_listing_differ |
| zh/ch06-types/listing-6-2 | tapl/ch06-types.cn.md:137 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch06-types/listing-6-3 | tapl/ch06-types.cn.md:267 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch06-types/listing-6-4 | tapl/ch06-types.cn.md:395 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch06-types/listing-6-5 | tapl/ch06-types.cn.md:524 | CodeView run | markdown_and_listing_differ |
| zh/ch06-types/listing-6-6 | tapl/ch06-types.cn.md:710 | CodeView run | markdown_and_listing_differ |
| zh/ch06-types/listing-6-7 | tapl/ch06-types.cn.md:831 | CodeView run | markdown_and_listing_differ |
| zh/ch06-types/fence-007 | tapl/ch06-types.cn.md:130 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch06-types/fence-014 | tapl/ch06-types.cn.md:257 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch06-types/fence-033 | tapl/ch06-types.cn.md:704 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch06-types/fence-040 | tapl/ch06-types.cn.md:821 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch06-types/listing-6-1 | tapl/ch06-types.md:37 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch06-types/listing-6-2 | tapl/ch06-types.md:167 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch06-types/listing-6-3 | tapl/ch06-types.md:314 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch06-types/listing-6-4 | tapl/ch06-types.md:453 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch06-types/listing-6-5 | tapl/ch06-types.md:591 | CodeView run | markdown_and_listing_differ |
| en/ch06-types/listing-6-6 | tapl/ch06-types.md:792 | CodeView run | markdown_and_listing_differ |
| en/ch06-types/listing-6-7 | tapl/ch06-types.md:928 | CodeView run | markdown_and_listing_differ |
| en/ch06-types/fence-007 | tapl/ch06-types.md:158 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch06-types/fence-014 | tapl/ch06-types.md:301 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch06-types/fence-033 | tapl/ch06-types.md:785 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch06-types/fence-040 | tapl/ch06-types.md:915 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch07-enums/listing-7-1 | tapl/ch07-enums.cn.md:23 | CodeView run | markdown_and_listing_differ |
| zh/ch07-enums/listing-7-2 | tapl/ch07-enums.cn.md:159 | CodeView run | markdown_and_listing_differ |
| zh/ch07-enums/listing-7-3 | tapl/ch07-enums.cn.md:347 | CodeView run | markdown_and_listing_differ |
| zh/ch07-enums/listing-7-4 | tapl/ch07-enums.cn.md:552 | CodeView run | markdown_and_listing_differ |
| zh/ch07-enums/listing-7-5 | tapl/ch07-enums.cn.md:765 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch07-enums/listing-7-6 | tapl/ch07-enums.cn.md:906 | CodeView run | markdown_and_listing_differ |
| zh/ch07-enums/listing-7-7 | tapl/ch07-enums.cn.md:1054 | CodeView run | markdown_and_listing_differ |
| en/ch07-enums/listing-7-1 | tapl/ch07-enums.md:35 | CodeView run | markdown_and_listing_differ |
| en/ch07-enums/listing-7-2 | tapl/ch07-enums.md:181 | CodeView run | markdown_and_listing_differ |
| en/ch07-enums/listing-7-3 | tapl/ch07-enums.md:377 | CodeView run | markdown_and_listing_differ |
| en/ch07-enums/listing-7-4 | tapl/ch07-enums.md:589 | CodeView run | markdown_and_listing_differ |
| en/ch07-enums/listing-7-5 | tapl/ch07-enums.md:809 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch07-enums/listing-7-6 | tapl/ch07-enums.md:959 | CodeView run | markdown_and_listing_differ |
| en/ch07-enums/listing-7-7 | tapl/ch07-enums.md:1112 | CodeView run | markdown_and_listing_differ |
| zh/ch08-oop/listing-8-1 | tapl/ch08-oop.cn.md:47 | CodeView run | markdown_and_listing_differ |
| zh/ch08-oop/listing-8-2 | tapl/ch08-oop.cn.md:220 | CodeView run | markdown_and_listing_differ |
| zh/ch08-oop/listing-8-3 | tapl/ch08-oop.cn.md:410 | CodeView run | markdown_and_listing_differ |
| zh/ch08-oop/listing-8-4 | tapl/ch08-oop.cn.md:602 | CodeView run | markdown_and_listing_differ |
| zh/ch08-oop/listing-8-5 | tapl/ch08-oop.cn.md:675 | CodeView run | markdown_and_listing_differ |
| zh/ch08-oop/listing-8-6 | tapl/ch08-oop.cn.md:924 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch08-oop/listing-8-7 | tapl/ch08-oop.cn.md:1047 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch08-oop/listing-8-8 | tapl/ch08-oop.cn.md:1196 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch08-oop/listing-8-1 | tapl/ch08-oop.md:60 | CodeView run | markdown_and_listing_differ |
| en/ch08-oop/listing-8-2 | tapl/ch08-oop.md:241 | CodeView run | markdown_and_listing_differ |
| en/ch08-oop/listing-8-3 | tapl/ch08-oop.md:438 | CodeView run | markdown_and_listing_differ |
| en/ch08-oop/listing-8-4 | tapl/ch08-oop.md:635 | CodeView run | markdown_and_listing_differ |
| en/ch08-oop/listing-8-5 | tapl/ch08-oop.md:715 | CodeView run | markdown_and_listing_differ |
| en/ch08-oop/listing-8-6 | tapl/ch08-oop.md:975 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch08-oop/listing-8-7 | tapl/ch08-oop.md:1107 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch08-oop/listing-8-8 | tapl/ch08-oop.md:1261 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch09-error-handling/listing-9-1 | tapl/ch09-error-handling.cn.md:51 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch09-error-handling/listing-9-2 | tapl/ch09-error-handling.cn.md:221 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch09-error-handling/listing-9-3 | tapl/ch09-error-handling.cn.md:406 | CodeView run | legacy_parse_random_names |
| zh/ch09-error-handling/listing-9-4 | tapl/ch09-error-handling.cn.md:526 | CodeView run | current_lock_regex_matches |
| zh/ch09-error-handling/listing-9-5 | tapl/ch09-error-handling.cn.md:676 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch09-error-handling/listing-9-6 | tapl/ch09-error-handling.cn.md:843 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch09-error-handling/listing-9-7 | tapl/ch09-error-handling.cn.md:981 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch09-error-handling/listing-9-8 | tapl/ch09-error-handling.cn.md:1157 | CodeView run | markdown_and_listing_differ |
| zh/ch09-error-handling/listing-9-9 | tapl/ch09-error-handling.cn.md:1245 | CodeView run | markdown_and_listing_differ |
| zh/ch09-error-handling/fence-012 | tapl/ch09-error-handling.cn.md:381 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch09-error-handling/listing-9-1 | tapl/ch09-error-handling.md:65 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch09-error-handling/listing-9-2 | tapl/ch09-error-handling.md:243 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch09-error-handling/listing-9-3 | tapl/ch09-error-handling.md:434 | CodeView run | legacy_parse_random_names |
| en/ch09-error-handling/listing-9-4 | tapl/ch09-error-handling.md:562 | CodeView run | current_lock_regex_matches |
| en/ch09-error-handling/listing-9-5 | tapl/ch09-error-handling.md:718 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch09-error-handling/listing-9-6 | tapl/ch09-error-handling.md:892 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch09-error-handling/listing-9-7 | tapl/ch09-error-handling.md:1044 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch09-error-handling/listing-9-8 | tapl/ch09-error-handling.md:1229 | CodeView run | markdown_and_listing_differ |
| en/ch09-error-handling/listing-9-9 | tapl/ch09-error-handling.md:1317 | CodeView run | markdown_and_listing_differ |
| en/ch09-error-handling/fence-012 | tapl/ch09-error-handling.md:405 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch10-modules/listing-10-1 | tapl/ch10-modules.cn.md:31 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch10-modules/listing-10-2 | tapl/ch10-modules.cn.md:131 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch10-modules/listing-10-3 | tapl/ch10-modules.cn.md:207 | CodeView run | current_lock_regex_matches, markdown_and_listing_differ |
| zh/ch10-modules/listing-10-4 | tapl/ch10-modules.cn.md:347 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch10-modules/listing-10-5 | tapl/ch10-modules.cn.md:496 | CodeView run | current_lock_regex_matches |
| zh/ch10-modules/fence-019 | tapl/ch10-modules.cn.md:322 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch10-modules/listing-10-1 | tapl/ch10-modules.md:38 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch10-modules/listing-10-2 | tapl/ch10-modules.md:142 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch10-modules/listing-10-3 | tapl/ch10-modules.md:221 | CodeView run | current_lock_regex_matches, markdown_and_listing_differ |
| en/ch10-modules/listing-10-4 | tapl/ch10-modules.md:364 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch10-modules/listing-10-5 | tapl/ch10-modules.md:520 | CodeView run | current_lock_regex_matches |
| en/ch10-modules/fence-019 | tapl/ch10-modules.md:337 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch11-references/listing-11-1 | tapl/ch11-references.cn.md:43 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch11-references/listing-11-2 | tapl/ch11-references.cn.md:166 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch11-references/listing-11-3 | tapl/ch11-references.cn.md:288 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch11-references/listing-11-4 | tapl/ch11-references.cn.md:399 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch11-references/listing-11-5 | tapl/ch11-references.cn.md:504 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch11-references/listing-11-6 | tapl/ch11-references.cn.md:670 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch11-references/fence-007 | tapl/ch11-references.cn.md:138 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch11-references/fence-038 | tapl/ch11-references.cn.md:796 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch11-references/listing-11-1 | tapl/ch11-references.md:52 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch11-references/listing-11-2 | tapl/ch11-references.md:182 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch11-references/listing-11-3 | tapl/ch11-references.md:305 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch11-references/listing-11-4 | tapl/ch11-references.md:425 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch11-references/listing-11-5 | tapl/ch11-references.md:533 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch11-references/listing-11-6 | tapl/ch11-references.md:706 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch11-references/fence-007 | tapl/ch11-references.md:148 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch11-references/fence-038 | tapl/ch11-references.md:837 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch12-memory/listing-12-1 | tapl/ch12-memory.cn.md:30 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch12-memory/listing-12-2 | tapl/ch12-memory.cn.md:146 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch12-memory/listing-12-3 | tapl/ch12-memory.cn.md:260 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch12-memory/listing-12-4 | tapl/ch12-memory.cn.md:378 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch12-memory/listing-12-5 | tapl/ch12-memory.cn.md:494 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch12-memory/listing-12-6 | tapl/ch12-memory.cn.md:643 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch12-memory/listing-12-1 | tapl/ch12-memory.md:38 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch12-memory/listing-12-2 | tapl/ch12-memory.md:160 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch12-memory/listing-12-3 | tapl/ch12-memory.md:279 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch12-memory/listing-12-4 | tapl/ch12-memory.md:403 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch12-memory/listing-12-5 | tapl/ch12-memory.md:526 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch12-memory/listing-12-6 | tapl/ch12-memory.md:684 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch13-generics/listing-13-1 | tapl/ch13-generics.cn.md:39 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch13-generics/listing-13-2 | tapl/ch13-generics.cn.md:176 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch13-generics/listing-13-3 | tapl/ch13-generics.cn.md:361 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch13-generics/listing-13-4 | tapl/ch13-generics.cn.md:517 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch13-generics/listing-13-5 | tapl/ch13-generics.cn.md:635 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch13-generics/listing-13-6 | tapl/ch13-generics.cn.md:787 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch13-generics/listing-13-7 | tapl/ch13-generics.cn.md:911 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch13-generics/fence-013 | tapl/ch13-generics.cn.md:343 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch13-generics/listing-13-1 | tapl/ch13-generics.md:49 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch13-generics/listing-13-2 | tapl/ch13-generics.md:194 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch13-generics/listing-13-3 | tapl/ch13-generics.md:384 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch13-generics/listing-13-4 | tapl/ch13-generics.md:543 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch13-generics/listing-13-5 | tapl/ch13-generics.md:665 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch13-generics/listing-13-6 | tapl/ch13-generics.md:820 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch13-generics/listing-13-7 | tapl/ch13-generics.md:950 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch13-generics/fence-013 | tapl/ch13-generics.md:364 | AutoFence run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch14-file-processor/listing-14-1 | tapl/ch14-file-processor.cn.md:38 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch14-file-processor/listing-14-2 | tapl/ch14-file-processor.cn.md:196 | CodeView run | current_lock_regex_matches, nonlocal_library_names_to_resolve |
| zh/ch14-file-processor/listing-14-3 | tapl/ch14-file-processor.cn.md:331 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch14-file-processor/listing-14-4 | tapl/ch14-file-processor.cn.md:522 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch14-file-processor/listing-14-5 | tapl/ch14-file-processor.cn.md:719 | CodeView run | current_lock_regex_matches |
| zh/ch14-file-processor/listing-14-6 | tapl/ch14-file-processor.cn.md:973 | CodeView run | markdown_and_listing_differ |
| zh/ch14-file-processor/listing-14-7 | tapl/ch14-file-processor.cn.md:1070 | CodeView run | markdown_and_listing_differ |
| zh/ch14-file-processor/listing-14-8 | tapl/ch14-file-processor.cn.md:1178 | CodeView run | markdown_and_listing_differ |
| zh/ch14-file-processor/listing-14-9 | tapl/ch14-file-processor.cn.md:1258 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch14-file-processor/fence-031 | tapl/ch14-file-processor.cn.md:950 | AutoFence run | current_lock_regex_matches |
| en/ch14-file-processor/listing-14-1 | tapl/ch14-file-processor.md:43 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch14-file-processor/listing-14-2 | tapl/ch14-file-processor.md:203 | CodeView run | current_lock_regex_matches, nonlocal_library_names_to_resolve |
| en/ch14-file-processor/listing-14-3 | tapl/ch14-file-processor.md:340 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch14-file-processor/listing-14-4 | tapl/ch14-file-processor.md:535 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch14-file-processor/listing-14-5 | tapl/ch14-file-processor.md:735 | CodeView run | current_lock_regex_matches |
| en/ch14-file-processor/listing-14-6 | tapl/ch14-file-processor.md:993 | CodeView run | markdown_and_listing_differ |
| en/ch14-file-processor/listing-14-7 | tapl/ch14-file-processor.md:1110 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch14-file-processor/listing-14-8 | tapl/ch14-file-processor.md:1238 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch14-file-processor/listing-14-9 | tapl/ch14-file-processor.md:1330 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch14-file-processor/fence-031 | tapl/ch14-file-processor.md:968 | AutoFence run | current_lock_regex_matches |
| zh/ch15-actors/listing-15-1 | tapl/ch15-actors.cn.md:43 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch15-actors/listing-15-2 | tapl/ch15-actors.cn.md:176 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch15-actors/listing-15-3 | tapl/ch15-actors.cn.md:375 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch15-actors/listing-15-4 | tapl/ch15-actors.cn.md:532 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch15-actors/listing-15-5 | tapl/ch15-actors.cn.md:717 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch15-actors/listing-15-6 | tapl/ch15-actors.cn.md:859 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch15-actors/listing-15-7 | tapl/ch15-actors.cn.md:1081 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch15-actors/listing-15-1 | tapl/ch15-actors.md:51 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch15-actors/listing-15-2 | tapl/ch15-actors.md:188 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch15-actors/listing-15-3 | tapl/ch15-actors.md:389 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch15-actors/listing-15-4 | tapl/ch15-actors.md:550 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch15-actors/listing-15-5 | tapl/ch15-actors.md:738 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch15-actors/listing-15-6 | tapl/ch15-actors.md:883 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch15-actors/listing-15-7 | tapl/ch15-actors.md:1109 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch16-async/listing-16-1 | tapl/ch16-async.cn.md:44 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch16-async/listing-16-2 | tapl/ch16-async.cn.md:186 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch16-async/listing-16-3 | tapl/ch16-async.cn.md:326 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch16-async/listing-16-4 | tapl/ch16-async.cn.md:532 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch16-async/listing-16-5 | tapl/ch16-async.cn.md:724 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch16-async/listing-16-6 | tapl/ch16-async.cn.md:978 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch16-async/listing-16-1 | tapl/ch16-async.md:51 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch16-async/listing-16-2 | tapl/ch16-async.md:199 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch16-async/listing-16-3 | tapl/ch16-async.md:344 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch16-async/listing-16-4 | tapl/ch16-async.md:556 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch16-async/listing-16-5 | tapl/ch16-async.md:752 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch16-async/listing-16-6 | tapl/ch16-async.md:1012 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch17-smart-casts/listing-17-1 | tapl/ch17-smart-casts.cn.md:45 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch17-smart-casts/listing-17-2 | tapl/ch17-smart-casts.cn.md:206 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch17-smart-casts/listing-17-3 | tapl/ch17-smart-casts.cn.md:361 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch17-smart-casts/listing-17-4 | tapl/ch17-smart-casts.cn.md:565 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch17-smart-casts/listing-17-5 | tapl/ch17-smart-casts.cn.md:721 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch17-smart-casts/listing-17-1 | tapl/ch17-smart-casts.md:51 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch17-smart-casts/listing-17-2 | tapl/ch17-smart-casts.md:216 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch17-smart-casts/listing-17-3 | tapl/ch17-smart-casts.md:378 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch17-smart-casts/listing-17-4 | tapl/ch17-smart-casts.md:590 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch17-smart-casts/listing-17-5 | tapl/ch17-smart-casts.md:750 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch18-testing/listing-18-1 | tapl/ch18-testing.cn.md:38 | CodeView test | test_runner_required |
| zh/ch18-testing/listing-18-2 | tapl/ch18-testing.cn.md:121 | CodeView test | test_runner_required |
| zh/ch18-testing/listing-18-3 | tapl/ch18-testing.cn.md:227 | CodeView test | test_runner_required |
| zh/ch18-testing/listing-18-4 | tapl/ch18-testing.cn.md:426 | CodeView test | test_runner_required |
| zh/ch18-testing/listing-18-5 | tapl/ch18-testing.cn.md:625 | CodeView test | test_runner_required |
| en/ch18-testing/listing-18-1 | tapl/ch18-testing.md:46 | CodeView test | test_runner_required |
| en/ch18-testing/listing-18-2 | tapl/ch18-testing.md:135 | CodeView test | test_runner_required |
| en/ch18-testing/listing-18-3 | tapl/ch18-testing.md:246 | CodeView test | test_runner_required |
| en/ch18-testing/listing-18-4 | tapl/ch18-testing.md:450 | CodeView test | test_runner_required |
| en/ch18-testing/listing-18-5 | tapl/ch18-testing.md:655 | CodeView test | test_runner_required |
| zh/ch19-closures/listing-19-1 | tapl/ch19-closures.cn.md:30 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch19-closures/listing-19-2 | tapl/ch19-closures.cn.md:154 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch19-closures/listing-19-3 | tapl/ch19-closures.cn.md:238 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch19-closures/listing-19-4 | tapl/ch19-closures.cn.md:349 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch19-closures/listing-19-5 | tapl/ch19-closures.cn.md:448 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch19-closures/listing-19-6 | tapl/ch19-closures.cn.md:551 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch19-closures/listing-19-7 | tapl/ch19-closures.cn.md:650 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch19-closures/listing-19-1 | tapl/ch19-closures.md:36 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch19-closures/listing-19-2 | tapl/ch19-closures.md:164 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch19-closures/listing-19-3 | tapl/ch19-closures.md:253 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch19-closures/listing-19-4 | tapl/ch19-closures.md:365 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch19-closures/listing-19-5 | tapl/ch19-closures.md:469 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch19-closures/listing-19-6 | tapl/ch19-closures.md:577 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch19-closures/listing-19-7 | tapl/ch19-closures.md:679 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch20-comptime/listing-20-1 | tapl/ch20-comptime.cn.md:43 | CodeView test | test_runner_required |
| zh/ch20-comptime/listing-20-2 | tapl/ch20-comptime.cn.md:172 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch20-comptime/listing-20-3 | tapl/ch20-comptime.cn.md:294 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch20-comptime/listing-20-4 | tapl/ch20-comptime.cn.md:426 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch20-comptime/listing-20-5 | tapl/ch20-comptime.cn.md:574 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch20-comptime/listing-20-1 | tapl/ch20-comptime.md:54 | CodeView test | test_runner_required |
| en/ch20-comptime/listing-20-2 | tapl/ch20-comptime.md:190 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch20-comptime/listing-20-3 | tapl/ch20-comptime.md:320 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch20-comptime/listing-20-4 | tapl/ch20-comptime.md:459 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch20-comptime/listing-20-5 | tapl/ch20-comptime.md:615 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch21-stdlib/listing-21-1 | tapl/ch21-stdlib.cn.md:24 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch21-stdlib/listing-21-2 | tapl/ch21-stdlib.cn.md:245 | CodeView run | current_lock_regex_matches |
| zh/ch21-stdlib/listing-21-3 | tapl/ch21-stdlib.cn.md:387 | CodeView run | current_lock_regex_matches |
| zh/ch21-stdlib/listing-21-4 | tapl/ch21-stdlib.cn.md:553 | CodeView run | markdown_and_listing_differ |
| zh/ch21-stdlib/listing-21-5 | tapl/ch21-stdlib.cn.md:788 | CodeView run | current_lock_regex_matches |
| zh/ch21-stdlib/listing-21-6 | tapl/ch21-stdlib.cn.md:950 | CodeView run | current_lock_regex_matches |
| zh/ch21-stdlib/listing-21-7 | tapl/ch21-stdlib.cn.md:1113 | CodeView run | current_lock_regex_matches |
| zh/ch21-stdlib/listing-21-8 | tapl/ch21-stdlib.cn.md:1227 | CodeView display-only | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch21-stdlib/listing-21-9 | tapl/ch21-stdlib.cn.md:1313 | CodeView run | foreign_or_package_dependency, markdown_and_listing_differ |
| zh/ch21-stdlib/listing-21-10 | tapl/ch21-stdlib.cn.md:1394 | CodeView run | foreign_or_package_dependency, markdown_and_listing_differ |
| zh/ch21-stdlib/listing-21-11 | tapl/ch21-stdlib.cn.md:1489 | CodeView run | current_lock_regex_matches |
| zh/ch21-stdlib/listing-21-12 | tapl/ch21-stdlib.cn.md:1605 | CodeView run | current_lock_regex_matches, explicit_stub_comment |
| zh/ch21-stdlib/listing-21-13 | tapl/ch21-stdlib.cn.md:1676 | CodeView run | current_lock_regex_matches |
| zh/ch21-stdlib/listing-21-14 | tapl/ch21-stdlib.cn.md:1747 | CodeView run | current_lock_regex_matches |
| en/ch21-stdlib/listing-21-1 | tapl/ch21-stdlib.md:29 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch21-stdlib/listing-21-2 | tapl/ch21-stdlib.md:252 | CodeView run | current_lock_regex_matches |
| en/ch21-stdlib/listing-21-3 | tapl/ch21-stdlib.md:397 | CodeView run | current_lock_regex_matches |
| en/ch21-stdlib/listing-21-4 | tapl/ch21-stdlib.md:566 | CodeView run | markdown_and_listing_differ |
| en/ch21-stdlib/listing-21-5 | tapl/ch21-stdlib.md:805 | CodeView run | current_lock_regex_matches |
| en/ch21-stdlib/listing-21-6 | tapl/ch21-stdlib.md:971 | CodeView run | current_lock_regex_matches |
| en/ch21-stdlib/listing-21-7 | tapl/ch21-stdlib.md:1136 | CodeView run | current_lock_regex_matches |
| en/ch21-stdlib/listing-21-8 | tapl/ch21-stdlib.md:1250 | CodeView display-only | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch21-stdlib/listing-21-9 | tapl/ch21-stdlib.md:1337 | CodeView run | foreign_or_package_dependency, markdown_and_listing_differ |
| en/ch21-stdlib/listing-21-10 | tapl/ch21-stdlib.md:1418 | CodeView run | foreign_or_package_dependency, markdown_and_listing_differ |
| en/ch21-stdlib/listing-21-11 | tapl/ch21-stdlib.md:1517 | CodeView run | current_lock_regex_matches |
| en/ch21-stdlib/listing-21-12 | tapl/ch21-stdlib.md:1634 | CodeView run | current_lock_regex_matches, explicit_stub_comment |
| en/ch21-stdlib/listing-21-13 | tapl/ch21-stdlib.md:1705 | CodeView run | current_lock_regex_matches |
| en/ch21-stdlib/listing-21-14 | tapl/ch21-stdlib.md:1776 | CodeView run | current_lock_regex_matches |
| zh/ch22-chat-server/listing-22-1 | tapl/ch22-chat-server.cn.md:51 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch22-chat-server/listing-22-2 | tapl/ch22-chat-server.cn.md:410 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch22-chat-server/listing-22-3 | tapl/ch22-chat-server.cn.md:741 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch22-chat-server/listing-22-4 | tapl/ch22-chat-server.cn.md:1002 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch22-chat-server/listing-22-5 | tapl/ch22-chat-server.cn.md:1491 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch22-chat-server/listing-22-6 | tapl/ch22-chat-server.cn.md:1865 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| zh/ch22-chat-server/listing-22-10 | tapl/ch22-chat-server.cn.md:1874 | CodeView local-only | current_lock_regex_matches |
| zh/ch22-chat-server/listing-22-11 | tapl/ch22-chat-server.cn.md:1878 | CodeView local-only | current_lock_regex_matches |
| zh/ch22-chat-server/listing-22-7 | tapl/ch22-chat-server.cn.md:2022 | CodeView run | markdown_and_listing_differ |
| zh/ch22-chat-server/listing-22-8 | tapl/ch22-chat-server.cn.md:2075 | CodeView run | foreign_or_package_dependency, markdown_and_listing_differ |
| zh/ch22-chat-server/listing-22-9 | tapl/ch22-chat-server.cn.md:2134 | CodeView run | ellipsis_needs_context_review, explicit_stub_comment, markdown_and_listing_differ |
| zh/ch22-chat-server/fence-032 | tapl/ch22-chat-server.cn.md:1918 | AutoFence missing catalog entry | test_runner_required |
| en/ch22-chat-server/listing-22-1 | tapl/ch22-chat-server.md:66 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch22-chat-server/listing-22-2 | tapl/ch22-chat-server.md:429 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch22-chat-server/listing-22-3 | tapl/ch22-chat-server.md:765 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch22-chat-server/listing-22-4 | tapl/ch22-chat-server.md:1032 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch22-chat-server/listing-22-5 | tapl/ch22-chat-server.md:1533 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch22-chat-server/listing-22-6 | tapl/ch22-chat-server.md:1906 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch22-chat-server/listing-22-10 | tapl/ch22-chat-server.md:2487 | CodeView local-only | current_lock_regex_matches |
| en/ch22-chat-server/listing-22-11 | tapl/ch22-chat-server.md:2491 | CodeView local-only | current_lock_regex_matches |
| en/ch22-chat-server/listing-22-7 | tapl/ch22-chat-server.md:2655 | CodeView run | 需实跑验证（未发现所列静态风险不等于通过） |
| en/ch22-chat-server/listing-22-8 | tapl/ch22-chat-server.md:2715 | CodeView run | foreign_or_package_dependency, markdown_and_listing_differ |
| en/ch22-chat-server/listing-22-9 | tapl/ch22-chat-server.md:2788 | CodeView run | ellipsis_needs_context_review, explicit_stub_comment, markdown_and_listing_differ |
| en/ch22-chat-server/fence-037 | tapl/ch22-chat-server.md:2536 | AutoFence missing catalog entry | test_runner_required |

非 Auto 围栏和被 Listing 替换的嵌套围栏也已逐一记录于 static-inventory.csv / .json；
input_hashes 绑定书稿、实际运行源、包声明及已有金样。
