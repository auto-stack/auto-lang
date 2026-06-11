//! AtomStream — lazy iteration of Atoms for large datasets
//!
//! Commands like `ls -R /` can produce thousands of entries. AtomStream
//! provides a cursor-based iterator over a Vec of Atoms, avoiding the need
//! to materialize everything at once in future streaming scenarios.

use super::atom::{Atom, AtomType};

/// A cursor-based stream of Atoms.
///
/// Uses `Vec<Atom>` + position cursor for simplicity and `Clone` support.
/// In the future this could be replaced with a proper lazy iterator.
#[derive(Debug, Clone)]
pub struct AtomStream {
    /// The items in the stream (pub for read access from AtomPipeline)
    pub items: Vec<Atom>,
    pos: usize,
}

impl AtomStream {
    /// Create a new stream from a Vec of Atoms.
    pub fn new(items: Vec<Atom>) -> Self {
        Self { items, pos: 0 }
    }

    /// Create an empty stream.
    pub fn empty() -> Self {
        Self::new(Vec::new())
    }

    /// Get the next Atom from the stream, advancing the cursor.
    pub fn next(&mut self) -> Option<Atom> {
        if self.pos < self.items.len() {
            let atom = self.items[self.pos].clone();
            self.pos += 1;
            Some(atom)
        } else {
            None
        }
    }

    /// Check if there are more items.
    pub fn has_next(&self) -> bool {
        self.pos < self.items.len()
    }

    /// Get the number of remaining items.
    pub fn remaining_count(&self) -> usize {
        self.items.len().saturating_sub(self.pos)
    }

    /// Get the total number of items (including already consumed).
    pub fn total_count(&self) -> usize {
        self.items.len()
    }

    /// Collect all remaining Atoms into a Vec.
    pub fn collect_remaining(&mut self) -> Vec<Atom> {
        let remaining: Vec<Atom> = self.items[self.pos..].to_vec();
        self.pos = self.items.len();
        remaining
    }

    /// Convert the stream into an Atom with a list type tag.
    ///
    /// Collects all items (including already consumed ones marked as Void)
    /// into a single Atom. The type tag is determined by the first non-empty item.
    pub fn into_atom_list(self) -> Atom {
        // Collect remaining items
        let items: Vec<Atom> = self.items.clone();
        let values: Vec<Value> = items.iter().map(|a| a.value.clone()).collect();

        // Infer type from first item
        let atom_type = items
            .first()
            .map(|a| a.atom_type)
            .unwrap_or(AtomType::Nothing);

        use auto_val::Array;
        Atom::new(Value::Array(Array::from(values)), atom_type)
    }
}

use auto_val::Value;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_new() {
        let atoms = vec![Atom::text("a"), Atom::text("b"), Atom::text("c")];
        let mut stream = AtomStream::new(atoms);
        assert_eq!(stream.total_count(), 3);
        assert_eq!(stream.remaining_count(), 3);
        assert!(stream.has_next());
    }

    #[test]
    fn test_stream_next() {
        let atoms = vec![Atom::text("a"), Atom::text("b")];
        let mut stream = AtomStream::new(atoms);

        assert_eq!(stream.next().unwrap().as_text(), "a");
        assert_eq!(stream.remaining_count(), 1);
        assert_eq!(stream.next().unwrap().as_text(), "b");
        assert_eq!(stream.remaining_count(), 0);
        assert!(!stream.has_next());
        assert!(stream.next().is_none());
    }

    #[test]
    fn test_stream_empty() {
        let mut stream = AtomStream::empty();
        assert_eq!(stream.total_count(), 0);
        assert!(!stream.has_next());
        assert!(stream.next().is_none());
    }

    #[test]
    fn test_stream_collect_remaining() {
        let atoms = vec![Atom::text("x"), Atom::text("y"), Atom::text("z")];
        let mut stream = AtomStream::new(atoms);
        stream.next(); // consume "x"
        let remaining = stream.collect_remaining();
        assert_eq!(remaining.len(), 2);
        assert_eq!(remaining[0].as_text(), "y");
        assert_eq!(remaining[1].as_text(), "z");
        assert_eq!(stream.remaining_count(), 0);
    }
}
