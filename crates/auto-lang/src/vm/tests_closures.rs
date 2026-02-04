// Plan 071: BigVM Closure Tests
//
// This module contains integration tests for BigVM closure support.

#[cfg(test)]
mod tests {
    // Include borrow checking tests
    include!("tests_closures_borrow_check.rs");
}
