# -*- coding: utf-8 -*-
# upgrade the debug test to byte-level tracing
import io
p = 'crates/auto-lang/src/http_upload_service.rs'
src = io.open(p, encoding='utf-8').read()
start = src.find('    #[test]\n    fn plan730_parser_zero_chunk_sizes_debug() {')
end = src.find('    #[test]\n    fn plan730_parser_zero_byte_file() {')
assert start != -1 and end != -1 and start < end
new_test = '''    #[test]
    fn plan730_parser_zero_chunk_sizes_debug() {
        let body = b"--B\\r\\nContent-Disposition: form-data; name=\\"file\\"; filename=\\"empty\\"\\r\\n\\r\\n\\r\\n--B--\\r\\n";
        let options = opts(r#"{"mode":"multipart"}"#);
        let mut mp = mp_new("B", &options);
        for (i, b) in body.iter().enumerate() {
            let mut out = Vec::new();
            match mp.feed(std::slice::from_ref(b), &mut out, &options) {
                Ok(()) => {}
                Err(e) => panic!(
                    "byte {i} ({}) state={:?} buf={:?} hdr={:?}: {e:?}",
                    b,
                    mp.state,
                    String::from_utf8_lossy(&mp.buf),
                    String::from_utf8_lossy(&mp.header_block)
                ),
            }
        }
        mp.finish().expect("finish");
    }

'''
src = src[:start] + new_test + src[end:]
io.open(p, 'w', encoding='utf-8', newline='\n').write(src)
print('debug upgraded')
