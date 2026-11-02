// Independent hand-written behavior reference (PLAN-912).
// fresh: base=1 (shared cell): f(20) -> base=21 -> 21; f(0) -> 21;
//        g(3) = x - 3 = 7; helper(1) = 2  =>  21 + 21 + 7 + 2 = 51.
// body-edit (g: `x - n` -> `x - n * 2`): g(3) = 10 - 6 = 4  =>  48.
// owner-body edit (x: 10 -> 20): g(3) = 17  =>  61.
fn main() {
    let fresh: i32 = {
        let mut base: i32 = 1;
        let mut f = |n: i32| { base += n; base };
        let x = 10;
        let g = |n: i32| x - n;
        let helper = |n: i32| n + 1;
        f(20) + f(0) + g(3) + helper(1)
    };
    let edited: i32 = {
        let mut base: i32 = 1;
        let mut f = |n: i32| { base += n; base };
        let x = 10;
        let g = |n: i32| x - n * 2;
        let helper = |n: i32| n + 1;
        f(20) + f(0) + g(3) + helper(1)
    };
    let owner_edit: i32 = {
        let mut base: i32 = 1;
        let mut f = |n: i32| { base += n; base };
        let x = 20;
        let g = |n: i32| x - n;
        let helper = |n: i32| n + 1;
        f(20) + f(0) + g(3) + helper(1)
    };
    assert_eq!((fresh, edited, owner_edit), (51, 48, 61));
    println!("i32:{fresh}/{edited}/{owner_edit}");
}
