# R2 运行中只读预算观测

同一 acceptance 调用的预算测试进程 PID 23116；开始 2026-10-16 13:34:34 +08:00。未启动第二份大负载，未修改测试缓存。

在 `C:/Users/zhaop/AppData/Local/Temp/ac787-budget-64m-23116/journal.db` 以 SQLite URI mode=ro 连接，执行：

```sql
SELECT count(*) FROM artifacts;
SELECT payload_chunk FROM artifacts WHERE key_canonical LIKE '%declidx-%' LIMIT 1;
```

以得到的 chunk id 读取 `blocks/blobs/<id前两字符>/<id>`：

```text
artifact_count 10128
declindex_frame_bytes 2968982
declindex_payload_count 50000
```

后两项分别是文件 len 与 `len(json.loads(frame[48:])['decls'])`。单张目录页的 payload 即 2,968,934 字节，超过 512KiB。实际读入口 `CstView::decl_entry` 对一次查询读取/解码整页；它不进入 debug_max_page_payload 的计量。该观测只证明现有读放大和漏计，不把进程某一时刻的 private bytes 当全程峰值。
