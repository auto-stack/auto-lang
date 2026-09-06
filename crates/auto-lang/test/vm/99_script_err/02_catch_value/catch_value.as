use.py builtins: int

fn main() {
    try {
        var x = int("abc")
        print("no-error")
    } catch e {
        print("caught: " + e.to(str))
    }
    print("end")
}
