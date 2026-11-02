// Independent hand-written behavior reference (PLAN-913).
//
// The recovery suite must produce these values with ZERO source production on
// the restore leg and exactly one unit of work on the post-restore edit.
//
// F01 fresh = 51:
//   base = 1 (shared cell written by f):
//     f(20) -> base = 21 -> 21; f(0) -> base = 21 -> 21
//     g(3)  -> x - 3 = 10 - 3 = 7
//     helper(1) -> 2
//   21 + 21 + 7 + 2 = 51.
// restore = the SAME program restored from the accepted state: 51.
// post-restore edit (g: `x - n` -> `x - n * 2`): g(3) = 10 - 6 = 4 => 48.
//
// K06 fresh = 42 (private writable closure environment):
//   f(20) -> 21; f(0) -> 21 => 42.
// restore = 42.
fn main() {
    let fresh_f01: i32 = {
        let mut base: i32 = 1;
        let mut f = |n: i32| {
            base += n;
            base
        };
        let x = 10;
        let g = |n: i32| x - n;
        let helper = |n: i32| n + 1;
        f(20) + f(0) + g(3) + helper(1)
    };
    let restored_f01: i32 = fresh_f01;
    let edited_f01: i32 = {
        let mut base: i32 = 1;
        let mut f = |n: i32| {
            base += n;
            base
        };
        let x = 10;
        let g = |n: i32| x - n * 2;
        let helper = |n: i32| n + 1;
        f(20) + f(0) + g(3) + helper(1)
    };
    let fresh_k06: i32 = {
        let mut base: i32 = 1;
        let mut f = move |n: i32| {
            base += n;
            base
        };
        f(20) + f(0)
    };
    assert_eq!(fresh_f01, 51);
    assert_eq!(restored_f01, 51);
    assert_eq!(edited_f01, 48);
    assert_eq!(fresh_k06, 42);
}
