use.py builtins: open

fn main() {
    with open("out.txt", "w") as f {
        f.write("hi")
    }
}
