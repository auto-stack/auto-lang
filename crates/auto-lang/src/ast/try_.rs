// Plan 010 (MS3-A): try/catch AST node.
// Plan 012 P2 (gap 4): optional `finally { <cleanup> }` clause.
//
// `try { <body> } catch (e) { <handler> } [finally { <cleanup> }]` — runs
// body; if a runtime error reaches the try boundary, it is caught and bound
// to `catch_param` (if any), then the handler runs. Without errors the
// handler is skipped. The finally body (when present) runs after the body on
// the normal path AND after the catch handler on the error path.

use super::Body;
use crate::ast::AtomWriter;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Try {
    pub body: Body,
    /// Optional binding name for the caught error: `catch (e)`. None = `catch { }`.
    pub catch_param: Option<String>,
    pub catch_body: Body,
    /// Optional `finally { }` body (Plan 012 P2). None = no finally clause.
    pub finally_body: Option<Body>,
    pub new_line: bool,
}

impl fmt::Display for Try {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let finally = match &self.finally_body {
            Some(fb) => format!(" (finally {})", fb),
            None => String::new(),
        };
        match &self.catch_param {
            Some(p) => write!(f, "(try {} (catch {} {}){})", self.body, p, self.catch_body, finally),
            None => write!(f, "(try {} (catch {}){})", self.body, self.catch_body, finally),
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
        if let Some(fb) = &self.finally_body {
            write!(f, " finally {}", fb)?;
        }
        Ok(())
    }
}
