use.py builtins: list

fn main() {
    var ab = py_call(list("ab"), "__add__", list("cd"))
    print("step1")
    var ab2 = py_call_may(ab, "__len__")
    print("step2-raw")
    print(ab2.to(str))
    var abn = ab2.?
    print("step3=" + abn.to(str))
}
