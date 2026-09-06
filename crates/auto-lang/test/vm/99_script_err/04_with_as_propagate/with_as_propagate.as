use.py builtins: open, int

fn main() {
    try {
        with open("with_as_guarantee.tmp", "w") as f {
            f.write("guaranteed")
            var boom = int("xyz")
        }
    } catch e {
        print("caught: " + e.to(str))
    }
    print("end")
}
