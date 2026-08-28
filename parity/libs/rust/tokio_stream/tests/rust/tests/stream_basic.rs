//! Native Rust oracle for tokio_stream parity.
//!
//! Demonstrates the idiomatic Rust equivalent of Auto's `~Stream<T>` generators:
//! `async_stream::stream!` producing `impl futures::Stream<Item = i32>`, consumed
//! via `StreamExt::next().await` inside an async runtime.

use async_stream::stream;
use futures::StreamExt;

fn counter(start: i32, count: i32) -> impl futures::Stream<Item = i32> {
    stream! {
        let mut i: i32 = 0;
        while i < count {
            yield start + i;
            i += 1;
        }
    }
}

fn repeat(v: i32, count: i32) -> impl futures::Stream<Item = i32> {
    stream! {
        let mut i: i32 = 0;
        while i < count {
            yield v;
            i += 1;
        }
    }
}

#[tokio::test]
async fn test_counter_constructs() {
    let s = counter(10, 3);
    tokio::pin!(s);
    assert_eq!(s.next().await, Some(10));
    assert_eq!(s.next().await, Some(11));
    assert_eq!(s.next().await, Some(12));
    assert_eq!(s.next().await, None);
}

#[tokio::test]
async fn test_repeat_constructs() {
    let s = repeat(42, 5);
    tokio::pin!(s);
    for _ in 0..5 {
        assert_eq!(s.next().await, Some(42));
    }
    assert_eq!(s.next().await, None);
}
