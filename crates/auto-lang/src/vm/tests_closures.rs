// Plan 071: AutoVM Closure Tests
//
// This module contains integration tests for AutoVM closure support.

#[cfg(test)]
mod tests {
    // Include borrow checking tests
    include!("tests_closures_borrow_check.rs");
}
