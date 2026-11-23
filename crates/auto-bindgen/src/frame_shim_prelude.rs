
#[derive(Debug, Clone)]
pub enum F0Val { None, I32(i32), Bool(bool), Text(String), Opaque(u64), F64(u64), F32(u32) }
#[derive(Debug, Clone)]
pub struct F0Kwarg { pub name: String, pub value: F0Val }
#[derive(Debug, Clone)]
pub struct F0Frame { pub version: u8, pub receiver: F0Val, pub positionals: Vec<F0Val>, pub kwargs: Vec<F0Kwarg> }
fn __f0_take_line<'a>(cur: &mut &'a [u8]) -> Result<&'a [u8], &'static str> {
    let n = cur.iter().position(|&b| b == b'\n').ok_or("unterminated header line")?;
    let (l, tail) = cur.split_at(n);
    if l.is_empty() { return Err("empty header line"); }
    *cur = &tail[1..];
    Ok(l)
}
fn __f0_decimal(l: &[u8]) -> Option<(u64, &[u8])> {
    let mut n: u64 = 0; let mut i = 0;
    if l.first() == Some(&b'-') { i = 1; }
    let d = i;
    while i < l.len() && l[i].is_ascii_digit() {
        n = n.checked_mul(10)?.checked_add((l[i] - b'0') as u64)?;
        if n > (i64::MAX as u64) { return None; }
        i += 1;
    }
    if i == d { return None; }
    Some((n, &l[i..]))
}
fn __f0_tag_ok(tag: u8, version: u8) -> bool {
    matches!(tag, b'n' | b'i' | b'b' | b't' | b'o')
        || (version == 2 && matches!(tag, b'd' | b'f'))
}
fn __f0_take_value<'a>(cur: &mut &'a [u8], version: u8) -> Result<(u8, &'a [u8]), &'static str> {
    let header = __f0_take_line(cur)?;
    let tag = *header.first().ok_or("missing tag")?;
    if !__f0_tag_ok(tag, version) { return Err("unknown tag"); }
    let tail = header[1..].strip_prefix(b" ").ok_or("tag not followed by a length")?;
    let (len, tail) = __f0_decimal(tail).ok_or("bad length")?;
    if !tail.is_empty() { return Err("length not the header end"); }
    if len > 4096 { return Err("payload over the frame text cap"); }
    let len = len as usize;
    if cur.len() < len { return Err("payload truncated"); }
    let (payload, tail) = cur.split_at(len);
    let tail = tail.strip_prefix(b"\n").ok_or("payload not LF-terminated")?;
    *cur = tail;
    Ok((tag, payload))
}
/// The typed float payload `<w2> <w1> <w0>`: three unsigned decimals, the
/// 64-bit pattern split 21 / 21 / 22 bits from the top (`w0` is the low 22
/// bits; bits = `w2 << 43 | w1 << 22 | w0`). Every word has a range bound and
/// is in its shortest form, so one value has exactly one encoding (PLAN-1129
/// review R-10: `01` is not a second spelling of `1`). A tag `f` value is the
/// low 32 bits of the pattern with the upper 32 bits zero (see `__f0_val`).
fn __f0_float_bits(payload: &[u8]) -> Result<u64, &'static str> {
    let mut words = payload.split(|&b| b == b' ');
    let mut value: u64 = 0;
    let bounds = [(43u32, 1u64 << 21), (22u32, 1u64 << 21), (0u32, 1u64 << 22)];
    for (shift, limit) in bounds {
        let w = words.next().ok_or("typed float payload is not `<w2> <w1> <w0>`")?;
        if !w.iter().all(|b| b.is_ascii_digit()) {
            return Err("typed float word is not a decimal");
        }
        if w.is_empty() || w.len() > 7 {
            return Err("typed float word is out of range");
        }
        if w[0] == b'0' && w.len() > 1 {
            return Err("typed float word has a leading zero");
        }
        match __f0_decimal(w) {
            Some((n, rest)) if rest.is_empty() && n < limit => value |= n << shift,
            _ => return Err("typed float word is out of range"),
        }
    }
    if words.next().is_some() {
        return Err("typed float payload is not `<w2> <w1> <w0>`");
    }
    Ok(value)
}
fn __f0_take_name(cur: &mut &[u8]) -> Result<String, &'static str> {
    let header = __f0_take_line(cur)?;
    let (nl, t) = __f0_decimal(header).ok_or("bad name length")?;
    if !t.is_empty() { return Err("name length not the header end"); }
    if nl < 1 || nl > 64 { return Err("kwarg name length out of range"); }
    let nl = nl as usize;
    if cur.len() < nl { return Err("kwarg name truncated"); }
    let (nb, t2) = cur.split_at(nl);
    *cur = t2;
    match std::str::from_utf8(nb).ok() {
        Some(s) => Ok(s.to_string()),
        None => Err("kwarg name is not UTF-8"),
    }
}
fn __f0_val(tag: u8, payload: &[u8], version: u8) -> Result<F0Val, &'static str> {
    match tag {
        b'n' if payload.is_empty() => Ok(F0Val::None),
        b'i' => std::str::from_utf8(payload).ok().and_then(|t| t.parse::<i32>().ok()).map(F0Val::I32).ok_or("tag i payload is not an i32"),
        b'o' => std::str::from_utf8(payload).ok().and_then(|t| t.parse::<u64>().ok()).map(F0Val::Opaque).ok_or("tag o payload is not a u64"),
        b'b' if payload == b"0" => Ok(F0Val::Bool(false)),
        b'b' if payload == b"1" => Ok(F0Val::Bool(true)),
        b't' => std::str::from_utf8(payload).ok().map(|t| F0Val::Text(t.to_string())).ok_or("tag t payload is not UTF-8"),
        b'd' if version == 2 => Ok(F0Val::F64(__f0_float_bits(payload)?)),
        // An f32 is its 32-bit pattern alone: a payload that sets the upper
        // 32 bits is rejected, never truncated (one f32, one encoding).
        b'f' if version == 2 => match __f0_float_bits(payload)? {
            bits if bits >> 32 == 0 => Ok(F0Val::F32(bits as u32)),
            _ => Err("typed float f payload exceeds 32 bits"),
        },
        _ => Err("bad tagged value"),
    }
}
/// Decode a p967.frame/1 or p967.frame/2 wire frame (the AV admission
/// validated the structure pre-dispatch; this decode is the defensive
/// second gate). `F0Frame::version` records which wire header was read.
fn __f0_frame_decode(bytes: &[u8]) -> Result<F0Frame, &'static str> {
    let mut cur = bytes;
    let v1 = b"p967.frame/1\n";
    let v2 = b"p967.frame/2\n";
    let version = if cur.len() >= v1.len() && &cur[..v1.len()] == v1 {
        cur = &cur[v1.len()..];
        1
    } else if cur.len() >= v2.len() && &cur[..v2.len()] == v2 {
        cur = &cur[v2.len()..];
        2
    } else {
        return Err("bad magic/version header");
    };
    let rh = __f0_take_line(&mut cur)?;
    let rh = rh.strip_prefix(b"r ").ok_or("expected the receiver value header")?;
    let rtag = *rh.first().ok_or("missing tag")?;
    if !__f0_tag_ok(rtag, version) { return Err("unknown tag"); }
    let rtail = rh[1..].strip_prefix(b" ").ok_or("tag not followed by a length")?;
    let (rlen, rtail) = __f0_decimal(rtail).ok_or("bad length")?;
    if !rtail.is_empty() || rlen > 4096 { return Err("bad receiver header"); }
    let rlen = rlen as usize;
    if cur.len() < rlen { return Err("payload truncated"); }
    let (rpayload, t2) = cur.split_at(rlen);
    let t2 = t2.strip_prefix(b"\n").ok_or("payload not LF-terminated")?;
    cur = t2;
    let receiver = __f0_val(rtag, rpayload, version)?;
    let header = __f0_take_line(&mut cur)?;
    if header[0] != b'p' { return Err("expected the positional count line"); }
    let (n, tail) = __f0_decimal(header[1..].strip_prefix(b" ").ok_or("count not delimited")?).ok_or("bad count")?;
    if !tail.is_empty() || n > 8 { return Err("count over the frame cap"); }
    let mut positionals = Vec::with_capacity(n as usize);
    for _ in 0..n {
        let (tag, payload) = __f0_take_value(&mut cur, version)?;
        positionals.push(__f0_val(tag, payload, version)?);
    }
    let header = __f0_take_line(&mut cur)?;
    if header[0] != b'k' { return Err("expected the kwarg count line"); }
    let (n, tail) = __f0_decimal(header[1..].strip_prefix(b" ").ok_or("count not delimited")?).ok_or("bad count")?;
    if !tail.is_empty() || n > 8 { return Err("count over the frame cap"); }
    let mut kwargs = Vec::with_capacity(n as usize);
    for _ in 0..n {
        let name = __f0_take_name(&mut cur)?;
        let (tag, payload) = __f0_take_value(&mut cur, version)?;
        kwargs.push(F0Kwarg { name, value: __f0_val(tag, payload, version)? });
    }
    if !cur.is_empty() { return Err("trailing bytes after the frame"); }
    Ok(F0Frame { version, receiver, positionals, kwargs })
}
