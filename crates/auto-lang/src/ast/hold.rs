use crate::ast::{AtomWriter, Body, Expr, ToAtom, ToAtomStr};
use auto_val::{AutoResult, AutoStr};
use std::{fmt, io as stdio};

/// Hold expression - temporarily bind a path for modification (Phase 3)
///
/// # Syntax
///
/// ```auto
/// hold x.y.z as value {
///     value.field = new_value
/// }
/// ```
///
/// # Semantics
///
/// Hold is syntactic sugar for:
/// ```auto
/// {
///     let value = mut x.y.z
///     // body
///     // value's lifetime ends here
/// }
/// ```
///
/// The borrow is automatically released at the end of the hold block.
#[derive(Debug, Clone)]
pub struct Hold {
    /// Path expression to borrow (e.g., x.y.z)
    pub path: Box<Expr>,
    /// Temporary binding name
    pub name: AutoStr,
    /// Body to execute with the borrow active
    pub body: Body,
    /// Span for error reporting (optional)
    pub span: Option<(usize, usize)>,
}

impl Hold {
    pub fn new(path: Expr, name: AutoStr, body: Body) -> Self {
        Self {
            path: Box::new(path),
            name,
            body,
            span: None,
        }
    }

    pub fn with_span(mut self, start: usize, end: usize) -> Self {
        self.span = Some((start, end));
        self
    }
}

impl fmt::Display for Hold {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(hold {} as {} {})", self.path, self.name, self.body)
    }
}

impl ToAtom for Hold {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hold_display() {
        let hold = Hold::new(
            Expr::Ident("x.y.z".into()),
            "value".into(),
            Body::new(),
        );
        let display = format!("{}", hold);
        assert!(display.contains("hold"));
        assert!(display.contains("x.y.z"));
        assert!(display.contains("value"));
    }
}

impl AtomWriter for Hold {
    fn write_atom(&self, f: &mut impl stdio::Write) -> AutoResult<()> {
        write!(f, "(hold {} as {})", self.path, self.name)?;
        write!(f, "{}", self.body)?;
        write!(f, ")")?;
        Ok(())
    }
}
