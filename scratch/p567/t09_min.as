use.py builtins: list

fn main() {
    var ab = py_call(list("ab"), "__add__", list("cd"))
    print("made ab")
    var abn = py_call(ab, "__len__")
    print("len=" + abn.to(str))
}
