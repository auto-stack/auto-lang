# -*- coding: utf-8 -*-
# PLAN-730 T-06 fix: AsRef<str> facade + drop .as_str() emission + futures dep
import io

# ── a2r-std facade: AsRef<str> params (literal/String/&str all legal) ──
p = 'crates/a2r-std/src/http/server_upload.rs'
src = io.open(p, encoding='utf-8').read()

def rep(old, new):
    global src
    assert old in src, 'A2R NOT FOUND: ' + old[:80]
    src = src.replace(old, new)

rep("""pub async fn upload_receive(
    req: UploadRequest,
    root: &str,
    staging_root: &str,
    options_json: &str,
) -> UploadSession {""",
"""pub async fn upload_receive(
    req: UploadRequest,
    root: impl AsRef<str>,
    staging_root: impl AsRef<str>,
    options_json: impl AsRef<str>,
) -> UploadSession {""")
# body uses &root etc — bind locals
rep("""    let Some(exec) = executor() else {
        return failed_session(
            UploadErrorKind::NoExecutor,
            "upload executor not installed (requires the HTTP server host)",
        );
    };
    let hard = UploadServeLimits::from_env();
    let options = match parse_upload_receive_options(options_json, &hard) {
        Ok(o) => o,
        Err(message) => {
            return failed_session(UploadErrorKind::InvalidOptions, message);
        }
    };
    exec.receive(req, root, staging_root, options, Arc::new(|_| {})).await
}""",
"""    let Some(exec) = executor() else {
        return failed_session(
            UploadErrorKind::NoExecutor,
            "upload executor not installed (requires the HTTP server host)",
        );
    };
    let hard = UploadServeLimits::from_env();
    let options = match parse_upload_receive_options(options_json.as_ref(), &hard) {
        Ok(o) => o,
        Err(message) => {
            return failed_session(UploadErrorKind::InvalidOptions, message);
        }
    };
    let root = root.as_ref().to_string();
    let staging_root = staging_root.as_ref().to_string();
    exec.receive(req, &root, &staging_root, options, Arc::new(|_| {}))
        .await
}""")

rep("""pub async fn upload_commit(session: UploadSession, relative_target: &str) -> UploadReceipt {
    if !session.is_received() {
        return UploadReceipt::failed(
            UploadErrorKind::SessionConflict,
            "upload_commit: cannot commit a failed receive session",
        );
    }
    if let Err(message) = validate_relative_path(relative_target) {
        return UploadReceipt::failed(UploadErrorKind::ForbiddenPath, message);
    }
    let Some(exec) = executor() else {
        return UploadReceipt::failed(
            UploadErrorKind::NoExecutor,
            "upload executor not installed (requires the HTTP server host)",
        );
    };
    exec.commit(session.id, relative_target).await
}""",
"""pub async fn upload_commit(
    session: UploadSession,
    relative_target: impl AsRef<str>,
) -> UploadReceipt {
    if !session.is_received() {
        return UploadReceipt::failed(
            UploadErrorKind::SessionConflict,
            "upload_commit: cannot commit a failed receive session",
        );
    }
    let target = relative_target.as_ref();
    if let Err(message) = validate_relative_path(target) {
        return UploadReceipt::failed(UploadErrorKind::ForbiddenPath, message);
    }
    let Some(exec) = executor() else {
        return UploadReceipt::failed(
            UploadErrorKind::NoExecutor,
            "upload executor not installed (requires the HTTP server host)",
        );
    };
    exec.commit(session.id, target).await
}""")

rep("""pub async fn upload_reject(session: UploadSession, status: i64, message: &str) -> UploadReceipt {""",
"""pub async fn upload_reject(
    session: UploadSession,
    status: i64,
    message: impl AsRef<str>,
) -> UploadReceipt {""")
rep("""    if let UploadSessionState::Failed { kind, .. } = &session.state {
        return UploadReceipt {
            status: kind.suggested_status(),
            kind: kind.clone(),
            json: format!(
                "{{\\"ok\\":false,\\"kind\\":\\"{}\\",\\"message\\":\\"{}\\"}}",
                kind.as_str(),
                json_escape(message),
            ),
        };
    }""",
"""    let message = message.as_ref();
    if let UploadSessionState::Failed { kind, .. } = &session.state {
        return UploadReceipt {
            status: kind.suggested_status(),
            kind: kind.clone(),
            json: format!(
                "{{\\"ok\\":false,\\"kind\\":\\"{}\\",\\"message\\":\\"{}\\"}}",
                kind.as_str(),
                json_escape(message),
            ),
        };
    }""")

rep("""pub fn upload_error(status: i64, message: &str) -> UploadReceipt {""",
"""pub fn upload_error(status: i64, message: impl AsRef<str>) -> UploadReceipt {
    let message = message.as_ref();""")

io.open(p, 'w', encoding='utf-8', newline='\n').write(src)

# ── trans: drop .as_str() emissions ──
p = 'crates/auto-lang/src/trans/rust.rs'
src = io.open(p, encoding='utf-8').read()

def rep2(old, new):
    global src
    assert old in src, 'TRANS NOT FOUND: ' + old[:80]
    src = src.replace(old, new)

rep2("""                        let str_positions: &[usize] = match method.as_str() {
                            "upload_receive" => &[1, 2, 3],
                            "upload_commit" => &[1],
                            _ => &[2], // upload_reject: (session, status, message)
                        };
                        write!(out, "a2r_std::http::{}(", method)?;
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 { write!(out, ", ")?; }
                            if let Arg::Pos(expr) = arg {
                                self.expr(expr, out)?;
                                if str_positions.contains(&i) {
                                    write!(out, ".as_str()")?;
                                }
                            } else {
                                self.arg(arg, out)?;
                            }
                        }
                        write!(out, ").await")?;""",
"""                        // facade 形参为 impl AsRef<str>——字面量/String/&str
                        // 直传，无需收敛发射。
                        write!(out, "a2r_std::http::{}(", method)?;
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 { write!(out, ", ")?; }
                            if let Arg::Pos(expr) = arg {
                                self.expr(expr, out)?;
                            } else {
                                self.arg(arg, out)?;
                            }
                        }
                        write!(out, ").await")?;""")
rep2("""                            if let Arg::Pos(expr) = arg {
                                self.expr(expr, out)?;
                                if i == 1 {
                                    write!(out, ".as_str()")?;
                                }
                            } else {
                                self.arg(arg, out)?;
                            }
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    // PLAN-729 T-05: 服务端文件响应构造——零 I/O 描述符，""",
"""                            if let Arg::Pos(expr) = arg {
                                self.expr(expr, out)?;
                            } else {
                                self.arg(arg, out)?;
                            }
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    // PLAN-729 T-05: 服务端文件响应构造——零 I/O 描述符，""")

io.open(p, 'w', encoding='utf-8', newline='\n').write(src)

# ── api_gen: futures dep unconditional in production template + fixture ──
p = 'crates/auto-man/src/api_gen.rs'
src = io.open(p, encoding='utf-8').read()

def rep3(old, new):
    global src
    assert old in src, 'APIGEN NOT FOUND: ' + old[:80]
    src = src.replace(old, new)

rep3('''    let sse_deps = if has_sse { "
async-stream = \\"0.3\\"
futures = \\"0.3\\"" } else { "" };''',
'''    // PLAN-730 T-06: futures 无条件依赖——上传 glue（Request body 流投影）
    // 与 SSE 共用；SSE 独占 async-stream。
    let sse_deps = if has_sse { "
async-stream = \\"0.3\\"" } else { "" };
    let futures_deps = "
futures = \\"0.3\\"";''')
rep3('''tower-http.workspace = true{}{}{}
"#,
        safe_name, runtime_deps, db_deps, sse_deps''',
'''tower-http.workspace = true{}{}{}{}
"#,
        safe_name, runtime_deps, db_deps, futures_deps, sse_deps''')

# fixture Cargo.toml: add futures
rep3('''axum = "0.7"
tokio = {{ version = "1", features = ["full"] }}
tokio-util = {{ version = "0.7", features = ["io"] }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
tower-http = {{ version = "0.5", features = ["cors"] }}
auto-lang = {{ path = {auto_lang:?}, features = ["ui", "image-pipeline"] }}
a2r-std = {{ path = {a2r_std:?} }}

[[bin]]
name = "plan730-gen-e2e"''',
'''axum = "0.7"
tokio = {{ version = "1", features = ["full"] }}
tokio-util = {{ version = "0.7", features = ["io"] }}
futures = "0.3"
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
tower-http = {{ version = "0.5", features = ["cors"] }}
auto-lang = {{ path = {auto_lang:?}, features = ["ui", "image-pipeline"] }}
a2r-std = {{ path = {a2r_std:?} }}

[[bin]]
name = "plan730-gen-e2e"''')

io.open(p, 'w', encoding='utf-8', newline='\n').write(src)
print('AsRef+deps patched OK')
