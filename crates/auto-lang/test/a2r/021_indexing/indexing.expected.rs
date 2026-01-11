fn main() {
    let arr: [i32; 5] = [10, 20, 30, 40, 50];
    let first: i32 = arr[0];
    let third: i32 = arr[2];
    let last: i32 = arr[4];

    println!("First: {}", first);
    println!("Third: {}", third);
    println!("Last: {}", last);

    let matrix: [[i32; 2]; 2] = [[1, 2], [3, 4]];
    let val: i32 = matrix[0][1];
    println!("Matrix value: {}", val);
}
