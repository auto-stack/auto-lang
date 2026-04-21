use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct Example {
    pub name: String,
    pub source: String,
}

#[derive(Serialize)]
pub struct ExamplesResponse {
    pub examples: Vec<Example>,
}

pub async fn examples_handler() -> Json<ExamplesResponse> {
    Json(ExamplesResponse {
        examples: vec![
            Example {
                name: "Hello World".into(),
                source: r#"print("Hello, World!")"#.into(),
            },
            Example {
                name: "Variables".into(),
                source: r#"let x = 42
let name = "Auto"
print(f"Hello, $name! The answer is $x")"#.into(),
            },
            Example {
                name: "Functions".into(),
                source: r#"fn add(a int, b int) int {
    a + b
}

let result = add(3, 4)
print(result)"#.into(),
            },
            Example {
                name: "Fibonacci".into(),
                source: r#"fn fib(n int) int {
    if n <= 1 {
        return n
    }
    fib(n - 1) + fib(n - 2)
}

for i in 0..10 {
    print(fib(i))
}"#.into(),
            },
            Example {
                name: "Enums".into(),
                source: r#"enum Color {
    Red
    Green
    Blue
}

let c = Color.Red
is c {
    Color.Red -> print("red")
    Color.Green -> print("green")
    Color.Blue -> print("blue")
}"#.into(),
            },
        ],
    })
}
