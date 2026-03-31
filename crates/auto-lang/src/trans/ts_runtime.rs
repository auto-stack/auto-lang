use crate::ast::*;
use crate::AutoResult;
use std::io::Write;
use super::TypeScriptTrans;

impl TypeScriptTrans {
    /// Injects the TypeScript runtime prelude (helpers like range, print, etc.)
    pub fn inject_runtime(&mut self, out: &mut impl Write) -> AutoResult<()> {
        out.write(b"/**\n * AutoLang TypeScript Runtime\n */\n")?;
        
        // Print alias for function references
        out.write(b"const print = console.log.bind(console);\n\n")?;

        // Range helper for Expr::Range
        out.write(b"function range(start: number, end: number, eq: boolean = false): number[] {\n")?;
        out.write(b"    const res: number[] = [];\n")?;
        out.write(b"    if (eq) {\n")?;
        out.write(b"        for (let i = start; i <= end; i++) res.push(i);\n")?;
        out.write(b"    } else {\n")?;
        out.write(b"        for (let i = start; i < end; i++) res.push(i);\n")?;
        out.write(b"    }\n")?;
        out.write(b"    return res;\n")?;
        out.write(b"}\n\n")?;

        Ok(())
    }
}
