// Plan 010 (MS3-A): try/catch AST node.
//
// `try { <body> } catch (e) { <handler> }` — runs body; if a runtime error
// reaches the try boundary, it is caught and bound to `catch_param` (if any),
// then the handler runs. Without errors the handler is skipped.

use super::Body;
use crate::ast::AtomWriter;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Try {
    pub body: Body,
    /// Optional binding name for the caught error: `catch (e)`. None = `catch { }`.
    pub catch_param: Option<String>,
    pub catch_body: Body,
    pub new_line: bool,
}

impl fmt::Display for Try {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.catch_param {
            Some(p) => write!(f, "(try {} (catch {} {}))", self.body, p, self.catch_body),
            None => write!(f, "(try {} (catch {}))", self.body, self.catch_body),
        }
    }
}

impl AtomWriter for Try {
    fn write_atom(&self, f: &mut impl std::io::Write) -> auto_val::AutoResult<()> {
        write!(f, "try {} catch ", self.body)?;
        if let Some(p) = &self.catch_param {
            write!(f, "({}) ", p)?;
        }
        write!(f, "{}", self.catch_body)?;
        Ok(())
    }
}
