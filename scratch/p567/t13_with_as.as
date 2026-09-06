use.py builtins: open, int

fn main() {
    with open("scratch/p567/t13_out.txt", "w") as f {
        f.write("hello-with-as")
    }
    print("done")
    with open("scratch/p567/t13_out2.txt", "w") as f2 {
        f2.write("partial")
        var boom = int("xyz")
    }
    print("unreachable")
}
