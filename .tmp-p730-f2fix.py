# -*- coding: utf-8 -*-
# R1 F-2 clean rewrite (corrected): 0B + quoted boundary + total-timeout tests
import io

p = 'crates/auto-lang/src/http_upload_service.rs'
src = io.open(p, encoding='utf-8').read()

start = src.find('    /// R1 F-2：0B 文件 part 合法（空文件；size=0、无内容块产出）。')
if start == -1:
    start = src.find('    #[test]\n    fn plan730_parser_zero_chunk_sizes_debug() {')
    start = src.rfind('    ///', 0, start)
assert start != -1, 'F-2 block start'
end = src.find('    /// preamble/epilogue 字节不参与内容（计入 wire 由调用方负责）。')
assert end != -1 and start < end, 'F-2 block end'

BS = chr(92)
new_tests = (
    '    /// R1 F-2：0B 文件 part 合法（空文件；size=0、无内容块产出）——\n'
    '    /// 任意切块粒度（1..=body.len()）内容一致。\n'
    '    #[test]\n'
    '    fn plan730_parser_zero_byte_file_any_chunking() {\n'
    '        let crlf = format!("{BS}r{BS}n");\n'
    '        let body = format!(\n'
    '            "--B{crlf}Content-Disposition: form-data; name={BS}"file{BS}"; filename={BS}"empty{BS}"{crlf}{crlf}{crlf}--B--{crlf}",\n'
    '            crlf = crlf,\n'
    '        )\n'
    '        .into_bytes();\n'
    '        let options = opts(r#"{"mode":"multipart"}"#);\n'
    '        for cs in 1..=body.len() {\n'
    '            let (chunks, mp) = drive_parser("B", &body, cs, &options)\n'
    '                .unwrap_or_else(|e| panic!("chunk {cs}: {e:?}"));\n'
    '            assert!(chunks.is_empty(), "no file bytes for 0B file (cs={cs})");\n'
    '            assert_eq!(mp.file_size, 0, "cs={cs}");\n'
    '            let meta = mp.snapshot("file");\n'
    '            assert_eq!(meta.size, 0);\n'
    '            assert_eq!(meta.filename.as_deref(), Some("empty"));\n'
    '        }\n'
    '    }\n'
    '\n'
    '    /// R1 F-2：quoted boundary——run_receive 提取剥引号后同非引号解析\n'
    '    ///（此处锁解析面；wire 面 quoted-boundary 用例在 tests/plan730 R1 增补）。\n'
    '    #[test]\n'
    '    fn plan730_parser_quoted_boundary_accepted() {\n'
    '        let options = opts(r#"{"mode":"multipart"}"#);\n'
    '        let crlf = format!("{BS}r{BS}n");\n'
    '        let body = format!(\n'
    '            "--QB{crlf}Content-Disposition: form-data; name={BS}"file{BS}"{crlf}{crlf}DATA{crlf}--QB--{crlf}",\n'
    '            crlf = crlf,\n'
    '        )\n'
    '        .into_bytes();\n'
    '        let (chunks, mp) = drive_parser("QB", &body, 4, &options).unwrap();\n'
    '        assert_eq!(chunks.concat(), b"DATA");\n'
    '        assert_eq!(mp.file_size, 4);\n'
    '    }\n'
    '\n'
    '    /// R1 F-5：total 期限到期（旋钮 300ms + 滴流 stream）→ 408 total_timeout。\n'
    '    #[test]\n'
    '    fn plan730_total_timeout_fails_with_cleanup() {\n'
    '        std::env::set_var("AUTO_HTTP_UPLOAD_TOTAL_MS", "300");\n'
    '        install();\n'
    '        let (root, staging) = temp_roots("total");\n'
    '        let runtime = rt();\n'
    '        // 滴流 body：每 40ms 一字节，永不主动 EOF——total 300ms 必然先到。\n'
    '        let trickle: UploadBodyStream = Box::pin(futures::stream::unfold((), |()| async move {\n'
    '            tokio::time::sleep(Duration::from_millis(40)).await;\n'
    '            Some((Ok(vec![b' + "'t'" + ']), ()))\n'
    '        }));\n'
    '        let req = upload_request_from_parts(\n'
    '            "POST",\n'
    '            "/up",\n'
    '            vec![("content-type".to_string(), "application/octet-stream".to_string())],\n'
    '            trickle,\n'
    '        );\n'
    '        let session = runtime.block_on(a2r_std::http::upload_receive(\n'
    '            req,\n'
    '            root.to_str().unwrap(),\n'
    '            staging.to_str().unwrap(),\n'
    '            r#"{"mode":"raw"}"#,\n'
    '        ));\n'
    '        let meta = a2r_std::http::upload_metadata_json(&session);\n'
    '        assert!(meta.contains("total_timeout"), "{meta}");\n'
    '        assert!(meta.contains("{BS}"suggested_status{BS}":408"), "{meta}");\n'
    '        assert!(drive_until(\n'
    '            &runtime,\n'
    '            || std::fs::read_dir(&staging).unwrap().next().is_none(),\n'
    '            5\n'
    '        ), "staging cleaned after total timeout");\n'
    '        assert_eq!(upload_session_count(), 0);\n'
    '    }\n'
    '\n'
)
new_tests = new_tests.replace('{BS}', BS)
src = src[:start] + new_tests + src[end:]
io.open(p, 'w', encoding='utf-8', newline='\n').write(src)
print('F-2/F-5 clean rewrite done')
