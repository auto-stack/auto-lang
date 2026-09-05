fn main() {
    var a = "a"
    var b = "b"
    var p1 = a.to(str)
    var p2 = b.to(str)
    var q1 = p1 + p2
    print("q1=" + q1)

    print("r1=" + a.to(str) + b.to(str))

    var s1 = a.to(str)
    print("s1=" + s1)
    print("a_after=" + a)

    var t = a.to(str) + a.to(str)
    print("t=" + t)

    var u = a + b.to(str)
    print("u=" + u)

    var v = a.to(str) + b
    print("v=" + v)
}
