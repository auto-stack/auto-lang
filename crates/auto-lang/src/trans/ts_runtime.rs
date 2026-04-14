use crate::AutoResult;
use std::io::Write;
use super::TypeScriptTrans;

impl TypeScriptTrans {
    /// Generate conditional import statement for runtime symbols.
    /// Only imports what is actually needed.
    pub fn inject_runtime_import(&self, out: &mut impl Write) -> AutoResult<()> {
        if !self.needs_range && !self.needs_print {
            return Ok(());
        }

        out.write(b"import { ")?;

        let mut first = true;
        if self.needs_range {
            out.write(b"range")?;
            first = false;
        }
        if self.needs_print {
            if !first {
                out.write(b", ")?;
            }
            out.write(b"print")?;
        }

        out.write(b" } from \"")?;
        out.write_all(self.runtime_path.as_bytes())?;
        out.write(b"\";\n")?;

        Ok(())
    }

    /// Returns the content of the TypeScript runtime module.
    /// This should be written to a file at the runtime_path location.
    pub fn runtime_file_content() -> &'static str {
r#"/**
 * AutoLang TypeScript Runtime
 */

export function range(start: number, end: number, eq: boolean = false): number[] {
    const res: number[] = [];
    if (eq) {
        for (let i = start; i <= end; i++) res.push(i);
    } else {
        for (let i = start; i < end; i++) res.push(i);
    }
    return res;
}

export const print = console.log.bind(console);
"#
    }
}

/// Standalone function to get runtime file content (for use outside TypeScriptTrans)
pub fn runtime_file_content() -> &'static str {
    TypeScriptTrans::runtime_file_content()
}
