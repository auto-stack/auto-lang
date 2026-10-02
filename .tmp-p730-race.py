# -*- coding: utf-8 -*-
# tolerate terminal-chunk write race in chunked test
import io

p = 'crates/auto-lang/src/tests/plan730_http_upload_tests.rs'
src = io.open(p, encoding='utf-8').read()
BS = chr(92)
old = '        stream.write_all(b"0' + BS + 'r' + BS + 'n' + BS + 'r' + BS + 'n").unwrap();\n        let (status, _h, b) = read_response(&mut stream);\n        assert_eq!(status, 201, "{}", String::from_utf8_lossy(&b));\n        assert_eq!(std::fs::read(root.join("raw/data.bin")).unwrap(), payload);'
new = ('        // 终块写入与 hyper 快速应答后的连接关闭存在竞态（body 已全部落地、\n'
       '        // 201 已回）——容忍终块 RST，以响应与磁盘字节为准。\n'
       '        let _ = stream.write_all(b"0' + BS + 'r' + BS + 'n' + BS + 'r' + BS + 'n");\n'
       '        let (status, _h, b) = read_response(&mut stream);\n'
       '        assert_eq!(status, 201, "{}", String::from_utf8_lossy(&b));\n'
       '        assert_eq!(std::fs::read(root.join("raw/data.bin")).unwrap(), payload);')
assert old in src, 'anchor'
src = src.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(src)
print('race tolerated')
