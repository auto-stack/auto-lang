use.py builtins: int

fn getnum(s str) {
    return int(s)
}
fn main() {
    print("start")
    var n = getnum("42")
    print("n=" + n.to(str))
}
