fn main() {
    let list = List.new();
    list.push(100);
    list.push(200);


    let val1: i32 = list[0] ?? 0;
    let val2: i32 = list[5] ?? 999;
}
