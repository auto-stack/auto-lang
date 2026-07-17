use tokio;

async fn double(n: i32) -> i32 {
    n * 2
}

async fn add(a: i32, b: i32) -> i32 {
    a + b
}

async fn compute(a: i32, b: i32) -> i32 {
    let d = double(a).await;
    let s = add(d, b).await;
    s
}

async fn delay_value(v: i32) -> i32 {
    v
}

async fn pipeline(n: i32) -> i32 {
    let step1 = delay_value(n).await;
    let step2 = double(step1).await;
    let step3 = add(step2, 1).await;
    step3
}

#[tokio::test]
async fn test_double_5() {
    assert_eq!(double(5).await, 10);
}

#[tokio::test]
async fn test_double_0() {
    assert_eq!(double(0).await, 0);
}

#[tokio::test]
async fn test_double_negative() {
    assert_eq!(double(-3).await, -6);
}

#[tokio::test]
async fn test_add_simple() {
    assert_eq!(add(3, 4).await, 7);
}

#[tokio::test]
async fn test_add_negatives() {
    assert_eq!(add(-5, -10).await, -15);
}

#[tokio::test]
async fn test_compute_3_4() {
    assert_eq!(compute(3, 4).await, 10);
}

#[tokio::test]
async fn test_compute_0_0() {
    assert_eq!(compute(0, 0).await, 0);
}

#[tokio::test]
async fn test_compute_10_5() {
    assert_eq!(compute(10, 5).await, 25);
}

#[tokio::test]
async fn test_delay_42() {
    assert_eq!(delay_value(42).await, 42);
}

#[tokio::test]
async fn test_delay_0() {
    assert_eq!(delay_value(0).await, 0);
}

#[tokio::test]
async fn test_pipeline_5() {
    assert_eq!(pipeline(5).await, 11);
}

#[tokio::test]
async fn test_pipeline_0() {
    assert_eq!(pipeline(0).await, 1);
}

#[tokio::test]
async fn test_pipeline_10() {
    assert_eq!(pipeline(10).await, 21);
}
