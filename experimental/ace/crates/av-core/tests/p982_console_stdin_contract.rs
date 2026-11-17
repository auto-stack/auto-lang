//! PLAN-982（EXEC 第一步的宿主面）：`console.stdin` 的**默认拒绝**合同。
//!
//! EXEC 要的 stdin 面是"四件联动"（能力键／宿主 provider／文本返回／CLI 分派）。
//! 本轮先钉住 trait 的**默认行为**：没有输入面的宿主必须**具名拒绝**，而不是
//! 假装读到空行——否则"没有 stdin"会被静默变成空字符串，正是 969 T-08 禁止的
//! "静默丢弃输入"。
//!
//! 覆盖完 trait 的提供者仍可用默认 `read_line`，本用例就是这样一个提供者。

use ace_av_core::av::CancelFlag;
use ace_av_core::host::{ConsoleHost, HostError, Stream};

/// 只实现写面的提供者（典型：`auto <script.at>` 候选腿当前形态）。
struct WriteOnlyHost {
    lines: usize,
}

impl ConsoleHost for WriteOnlyHost {
    fn retained_bytes(&self) -> usize {
        self.lines
    }

    fn write(
        &mut self,
        _stream: Stream,
        _bytes: &[u8],
        _cancel: &CancelFlag,
    ) -> Result<(), HostError> {
        self.lines += 1;
        Ok(())
    }
}

#[test]
fn p982_console_stdin_default_is_a_named_refusal() {
    let mut host = WriteOnlyHost { lines: 0 };
    let cancel = CancelFlag::default();
    let err = host
        .read_line(&cancel)
        .expect_err("a write-only host must refuse stdin by name");
    let message = format!("{err:?}");
    assert!(
        message.contains("console.stdin/unsupported"),
        "the refusal names the missing surface: {message}"
    );
    // 写面不受影响（默认读实现不改变既有合同）。
    host.write(Stream::Stdout, b"x\n", &cancel).expect("write");
    assert_eq!(host.retained_bytes(), 1);
}
