# -*- coding: utf-8 -*-
# PLAN-734 R1 F-1: convert the 8 remaining production contains sites to contract kinds
import io

# ── api_gen.rs (6 sites) ──
p = 'crates/auto-man/src/api_gen.rs'
src = io.open(p, encoding='utf-8').read()

def rep(old, new):
    global src
    assert old in src, 'NOT FOUND: ' + old[:90]
    src = src.replace(old, new, 1)

# 1. endpoint_body_params (:1158)
rep('''        // PLAN-730 T-06: UploadRequest 是宿主注入参数（Request 提取器），
        // 不进 JSON body 结构/绑定。
        if p.ty.contains("UploadRequest") { return false; }''',
'''        // PLAN-730 T-06 / 734 R1 F-1: UploadRequest 是宿主注入参数（Request
        // 提取器），不进 JSON body 结构/绑定（契约身份判定）。
        if auto_lang::api::contract::is_upload_param(&p.ty) { return false; }''')

# 2. db-delegating file arm (:2422)
rep('''        let ret = endpoint.return_type.trim();
        // PLAN-729 T-05：委派路径（route A/db 覆盖）的文件端点——同样生成
        // 真实文件 adapter：method/headers 提取器 + 委派调用 + 宿主 serve。
        if ret.contains("FileResponse") {''',
'''        let ret = endpoint.return_type.trim();
        // PLAN-729 T-05 / 734 R1 F-1：委派路径（route A/db 覆盖）的文件端点——
        // 同样生成真实文件 adapter（契约身份判定）。
        if auto_lang::api::contract::ResponseKind::from_return_string(ret)
            == auto_lang::api::contract::ResponseKind::File
        {''')

# 3. main file arm (:2859)
rep('''        // PLAN-729 T-05: 文件端点——返回 `FileResponse`/`Future<FileResponse>`
        // 的 #[api] handler 生成真实文件 adapter（不经 JsonResponse，不落
        // CRUD 模板；转译失败 = 诊断 500，保留位置信息）。
        if endpoint.return_type.contains("FileResponse") {''',
'''        // PLAN-729 T-05 / 734 R1 F-1: 文件端点——返回 `FileResponse`/
        // `Future<FileResponse>` 的 #[api] handler 生成真实文件 adapter
        // （契约身份判定；不经 JsonResponse，转译失败 = 诊断 500）。
        if auto_lang::api::contract::ResponseKind::from_return_string(&endpoint.return_type)
            == auto_lang::api::contract::ResponseKind::File
        {''')

# 4+5. upload arm find + skip (:2949/:2978)
rep('''        let upload_param = endpoint
            .params
            .iter()
            .find(|p| p.ty.contains("UploadRequest"))''',
'''        let upload_param = endpoint
            .params
            .iter()
            .find(|p| auto_lang::api::contract::is_upload_param(&p.ty))''')
rep('''                if p.ty.contains("UploadRequest") || is_meta_param(p) {''',
'''                if auto_lang::api::contract::is_upload_param(&p.ty) || is_meta_param(p) {''')

# 6. main.rs HEAD automation (:3703)
rep('''            if e.return_type.contains("FileResponse")''',
'''            if auto_lang::api::contract::ResponseKind::from_return_string(&e.return_type)
                == auto_lang::api::contract::ResponseKind::File''')

io.open(p, 'w', encoding='utf-8', newline='\n').write(src)
print('api_gen 6 sites converted')

# ── typescript.rs (2 sites) ──
p = 'crates/auto-lang/src/api/targets/typescript.rs'
src = io.open(p, encoding='utf-8').read()

def rep2(old, new):
    global src
    assert old in src, 'TS NOT FOUND: ' + old[:90]
    src = src.replace(old, new, 1)

rep2('''        let upload_param_removed: Vec<&ApiParam> = endpoint
            .params
            .iter()
            .filter(|p| !p.ty.contains("UploadRequest"))''',
'''        let upload_param_removed: Vec<&ApiParam> = endpoint
            .params
            .iter()
            .filter(|p| !crate::api::contract::is_upload_param(&p.ty))''')

rep2('''        if endpoint.return_type.contains("FileResponse") {
            lines.push(format!("{}return response;", self.indent));''',
'''        if crate::api::contract::ResponseKind::from_return_string(&endpoint.return_type)
            == crate::api::contract::ResponseKind::File
        {
            lines.push(format!("{}return response;", self.indent));''')

io.open(p, 'w', encoding='utf-8', newline='\n').write(src)
print('typescript 2 sites converted')
