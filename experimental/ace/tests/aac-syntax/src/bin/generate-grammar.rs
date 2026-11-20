use std::io::{self, Write};

use ace_aac_syntax_conformance::SYNTAX_PROFILES;

const GRAMMAR: &str = include_str!("../../../../grammars/auto-source.at");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    writeln!(
        out,
        "// AAC canonical grammar data. Regenerate with the aac-syntax generate-grammar binary.\n\
         // Each 128-character chunk stays within a bounded text-access window on AV.\n\
         // The conformance suite compares these bytes with the shared grammar compiler."
    )?;
    for profile in SYNTAX_PROFILES {
        let program = ace_syntax_grammar::compile_profile(GRAMMAR, profile)
            .map_err(|err| format!("compile grammar profile {profile}: {err:?}"))?;
        let name = profile.replace('-', "_");
        writeln!(
            out,
            "\n/// Canonical program bytes of profile {profile:?} (hex, 128-char chunks)."
        )?;
        writeln!(out, "pub fn {name}_parts() List<Str> {{")?;
        writeln!(out, "    var p = List<Str>.new()")?;
        for chunk in program.canonical_bytes().chunks(64) {
            write!(out, "    p.push(\"")?;
            for byte in chunk {
                write!(out, "{byte:02x}")?;
            }
            writeln!(out, "\")")?;
        }
        writeln!(out, "    return p\n}}")?;
    }
    out.flush()?;
    Ok(())
}
