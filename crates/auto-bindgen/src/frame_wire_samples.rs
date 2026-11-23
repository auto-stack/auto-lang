// The shared sample table of the `p967.frame` wire: ONE source that the two
// implementations of the frame grammar are both tested against.
//
// * the AV admission (`ace-av-core`, `validate_frame_wire`) — the gate every
//   frame passes before it is dispatched;
// * the decoder of the generated provider shim
//   (`auto-bindgen/src/frame_shim_prelude.rs`).
//
// Both test modules `include!` this file and run every sample through their
// own implementation, so the two are judged by the same bytes and the same
// expectations (PLAN-1129 review R-10: each used to be tested against its
// own hand-written samples). The file holds no code of the implementations.
// It starts with plain comments because it is `include!`d inside a module
// body, where inner doc comments are not allowed.
//
// Each sample is a complete frame text (header included) and the verdict
// both implementations must return. A `Reject(reason)` pins the reason
// string; the reasons of the typed float payloads are part of the contract
// and must be identical in both. `Rejected` only asks for a refusal: those
// are shapes of the version-1 grammar whose reason strings differ between
// the two implementations (see D-1129-10).

/// What both implementations must answer for one frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Accept,
    Reject(&'static str),
    Rejected,
}

pub const V1: &str = "p967.frame/1\n";
pub const V2: &str = "p967.frame/2\n";

const NOT_TRIPLE: &str = "typed float payload is not `<w2> <w1> <w0>`";
const NOT_DECIMAL: &str = "typed float word is not a decimal";
const OUT_OF_RANGE: &str = "typed float word is out of range";
const LEADING_ZERO: &str = "typed float word has a leading zero";
const F_OVER_32: &str = "typed float f payload exceeds 32 bits";

/// A positional value in an otherwise empty version-2 frame.
pub fn positional(tag: &str, payload: &str) -> String {
    format!("r n 0\n\np 1\n{tag} {}\n{payload}\nk 0\n", payload.len())
}

/// A value in the receiver slot.
pub fn receiver(tag: &str, payload: &str) -> String {
    format!("r {tag} {}\n{payload}\np 0\nk 0\n", payload.len())
}

/// A value under the keyword `x`.
pub fn kwarg(tag: &str, payload: &str) -> String {
    format!(
        "r n 0\n\np 0\nk 1\n1\nx{tag} {}\n{payload}\n",
        payload.len()
    )
}

fn v2(body: &str) -> String {
    format!("{V2}{body}")
}

fn v1(body: &str) -> String {
    format!("{V1}{body}")
}

/// The canonical spelling of a bit pattern: `<w2> <w1> <w0>`, 21 / 21 / 22
/// bits from the top, each word in its shortest decimal form.
pub fn canonical(bits: u64) -> String {
    format!(
        "{} {} {}",
        bits >> 43,
        (bits >> 22) & 0x1F_FFFF,
        bits & 0x3F_FFFF
    )
}

/// Every hand-written sample: the typed float payload in all three value
/// slots, the controls that must be accepted, the version-1 grammar the
/// version-2 frame keeps, and the shapes where only the refusal is pinned.
pub fn hand_written() -> Vec<(String, Verdict)> {
    use Verdict::*;
    let mut rows: Vec<(String, Verdict)> = Vec::new();
    let mut add = |frame: String, verdict: Verdict| rows.push((frame, verdict));

    // ---- canonical spellings are accepted, in every slot
    for payload in [
        "0 0 0",
        "1 0 0",
        "0 1 0",
        "0 0 1",
        "0 0 2097151",
        "2097151 2097151 4194303",
        "10 100 1000",
        "2097150 1048575 1048576",
    ] {
        add(v2(&positional("d", payload)), Accept);
    }
    for payload in [
        "0 0 0",
        "0 512 0",
        "0 1023 4194303",
        "0 509 4194303",
        "0 0 4194303",
    ] {
        add(v2(&positional("f", payload)), Accept);
        add(v2(&receiver("f", payload)), Accept);
        add(v2(&kwarg("f", payload)), Accept);
    }
    for payload in ["0 0 0", "2097151 2097151 4194303"] {
        add(v2(&receiver("d", payload)), Accept);
        add(v2(&kwarg("d", payload)), Accept);
    }
    // two typed values, both widths, in one frame
    add(
        v2("r n 0\n\np 2\nd 5\n0 0 0\nf 7\n0 512 0\nk 1\n1\nxf 7\n0 512 0\n"),
        Accept,
    );
    // a typed value beside the version-1 tags
    add(v2("r i 2\n42\np 2\nd 5\n0 0 0\nt 5\nF:2.5\nk 0\n"), Accept);

    // ---- a leading zero is a second spelling of the same value: refused
    for (tag, payload) in [
        ("d", "00 0 0"),
        ("d", "01 0 0"),
        ("d", "0 00 0"),
        ("d", "0 01 0"),
        ("d", "0 0 00"),
        ("d", "0 0 01"),
        ("d", "0000000 0 0"),
        ("d", "0 0000000 0"),
        ("d", "0 0 0000000"),
        ("d", "0000000 0000000 0000000"),
        ("d", "0100000 0000000 0000000"),
        ("d", "0000001 0000000 0000000"),
        ("d", "0123456 0065432 0419430"),
        ("f", "00 0 0"),
        ("f", "0 0512 0"),
        ("f", "0 1023 0419430"),
        ("f", "00 1023 4194303"),
        ("f", "0000000 0000000 0000000"),
    ] {
        for body in [
            positional(tag, payload),
            receiver(tag, payload),
            kwarg(tag, payload),
        ] {
            add(v2(&body), Reject(LEADING_ZERO));
        }
    }

    // ---- the f32 pattern is the low 32 bits alone
    for payload in [
        "0 1024 0",
        "0 1048576 0",
        "1 0 0",
        "2097151 0 0",
        "0 2097151 0",
    ] {
        for body in [
            positional("f", payload),
            receiver("f", payload),
            kwarg("f", payload),
        ] {
            add(v2(&body), Reject(F_OVER_32));
        }
    }
    // the same words under `d` are an ordinary f64 pattern
    add(v2(&positional("d", "0 1024 0")), Accept);

    // ---- not three words (a missing word is an empty one: see "out of range")
    for payload in ["0", "0 0", "0 0 0 0", "0 0 0 "] {
        add(v2(&positional("d", payload)), Reject(NOT_TRIPLE));
        add(v2(&positional("f", payload)), Reject(NOT_TRIPLE));
    }

    // ---- a word that is not a decimal
    for payload in ["0 +1 0", "0 -1 0", "0 0x1 0", "0 1.5 0", "0\t0 0", "0 0 ٣"] {
        add(v2(&positional("d", payload)), Reject(NOT_DECIMAL));
        add(v2(&positional("f", payload)), Reject(NOT_DECIMAL));
    }

    // ---- a word past its bound, longer than seven digits, or empty (a
    // missing word, a doubled separator)
    for payload in [
        "2097152 0 0",
        "0 2097152 0",
        "0 0 4194304",
        "12345678 0 0",
        "0 12345678 0",
        "0 0 12345678",
        "",
        "0  0",
        "0 0 ",
    ] {
        add(v2(&positional("d", payload)), Reject(OUT_OF_RANGE));
        add(v2(&positional("f", payload)), Reject(OUT_OF_RANGE));
    }
    // the bounds of the f32 words are the same 21 / 22 bits
    add(v2(&positional("f", "0 4194304 0")), Reject(OUT_OF_RANGE));

    // ---- the typed tags belong to version 2 only
    for tag in ["d", "f"] {
        for body in [
            positional(tag, "0 0 0"),
            receiver(tag, "0 0 0"),
            kwarg(tag, "0 0 0"),
        ] {
            add(v1(&body), Reject("unknown tag"));
        }
    }

    // ---- the version-1 grammar is unchanged inside a version-2 frame
    add(v2("r n 0\n\np 0\nk 0\n"), Accept);
    add(v2("r o 1\n7\np 0\nk 0\n"), Accept);
    add(v2("r n 0\n\np 2\nt 0\n\nd 5\n0 0 0\nk 0\n"), Accept);
    add(v2("r n 0\n\np 1\ni 2\n42\nk 0\n"), Accept);
    add(v2("r n 0\n\np 1\nb 1\n1\nk 0\n"), Accept);
    add(v2("r n 0\n\np 0\nk 2\n2\nxli 1\n5\n1\nyi 1\n6\n"), Accept);
    add(v2("r n 0\n\np 1\nt 5\nF:2.5\nk 0\n"), Accept);
    add(v1("r n 0\n\np 0\nk 0\n"), Accept);
    add(v1("r o 1\n7\np 0\nk 0\n"), Accept);
    add(v1("r n 0\n\np 1\nt 5\nF:2.5\nk 0\n"), Accept);
    add(v1("r n 0\n\np 1\ni 2\n42\nk 0\n"), Accept);

    // ---- refused; only the refusal is pinned (the reasons differ)
    add(v2("r n 0\n\np 1\ni 3\n4x\nk 0\n"), Rejected); // i32 payload
    add(v2("r n 0\n\np 1\no 2\n-1\nk 0\n"), Rejected); // opaque payload
    add(v2("r n 0\n\np 1\nb 1\n2\nk 0\n"), Rejected); // bool payload not 0 / 1
    add(v2("r n 5\nabc\n\np 0\nk 0\n"), Rejected); // nil with a payload
    add(v2("r x 1\n1\np 0\nk 0\n"), Rejected); // unknown tag
    add(v2("r n 0\n\np 9\nk 0\n"), Rejected); // count over the frame cap
    add(v2("r n 0\n\np 1\nd 40\nabc\nk 0\n"), Rejected); // declared length not met
    add(v2("r n 0\n\np 0\nk 0\ngarbage"), Rejected); // trailing bytes
    add(v2("r n 0\n\np 1\nd +5\n0 0 0\nk 0\n"), Reject("bad length")); // `+` is no length
    add(v2("r f 40\n0 0 0\np 0\nk 0\n"), Rejected); // payload not LF-terminated
    add(
        "p967.frame/3\nr n 0\n\np 0\nk 0\n".to_string(),
        Reject("bad magic/version header"),
    );
    add(
        "nope\nr n 0\np 0\nk 0\n".to_string(),
        Reject("bad magic/version header"),
    );

    // ---- inherited leniency of the version-1 layout (D-1129-10): the length
    // and count fields take leading zeros and a leading minus. Both
    // implementations accept them today; these rows keep the two in step and
    // make the boundary explicit. They are not an endorsement.
    add(v2("r n 0\n\np 1\nd 05\n0 0 0\nk 0\n"), Accept);
    add(v2("r n 0\n\np 1\nd 0000000000000005\n0 0 0\nk 0\n"), Accept);
    add(v2("r n 0\n\np 1\nd -5\n0 0 0\nk 0\n"), Accept);
    add(v2("r n 0\n\np 00\nk 0\n"), Accept);
    add(v2("r n 0\n\np -0\nk 0\n"), Accept);
    add(v2("r n 0\n\np 0\nk 00\n"), Accept);
    add(v2("r n 0\n\np 1\ni 3\n003\nk 0\n"), Accept);
    add(v2("r n 0\n\np 1\ni 3\n+42\nk 0\n"), Accept);
    add(v2("r n 0\n\np 1\no 3\n007\nk 0\n"), Accept);

    rows
}

/// A small deterministic generator (a 64-bit LCG), so the generated rows are
/// the same on every run and every machine.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 ^ (self.0 >> 31)
    }
}

/// The batch of bit patterns the generated rows are built from. A test can
/// also walk it directly to check that the canonical spelling decodes back
/// to the very pattern.
pub fn patterns() -> Vec<u64> {
    let mut lcg = Lcg(0x1129_2026_1124);
    (0..64).map(|_| lcg.next()).collect()
}

/// The generated rows: for the batch of bit patterns, the canonical spelling
/// is accepted, every zero-padded variant of it is refused as a leading zero,
/// and an f32 never carries bits above its 32.
pub fn generated() -> Vec<(String, Verdict)> {
    use Verdict::*;
    let mut rows = Vec::new();
    for &bits in &patterns() {
        let words: Vec<String> = canonical(bits).split(' ').map(str::to_string).collect();

        // one spelling per value: the canonical words, and only them
        for slot in 0..3 {
            for pad in 1..=(7 - words[slot].len()) {
                let mut padded = words.clone();
                let mut word = "0".repeat(pad);
                word.push_str(&words[slot]);
                padded[slot] = word;
                let body = positional("d", &padded.join(" "));
                rows.push((v2(&body), Reject(LEADING_ZERO)));
            }
        }
        // a different value is a different spelling (the words encode the
        // bits; nothing is lost)
        let other = canonical(bits ^ 1);
        if other != canonical(bits) {
            rows.push((v2(&positional("d", &other)), Accept));
        }
        rows.push((v2(&positional("d", &canonical(bits))), Accept));

        // the f32 pattern: the low 32 bits are accepted, any bit above is not
        let low = bits & 0xFFFF_FFFF;
        rows.push((v2(&positional("f", &canonical(low))), Accept));
        rows.push((
            v2(&positional("f", &canonical(low | 1 << 43))),
            Reject(F_OVER_32),
        ));
        if low < 0xFFFF_FFFF {
            rows.push((v2(&positional("f", &canonical(low + 1))), Accept));
        }
    }
    rows
}

/// Every sample, hand-written first.
pub fn all_samples() -> Vec<(String, Verdict)> {
    let mut rows = hand_written();
    rows.extend(generated());
    rows
}
