use crate::vm::engine::AutoVM;
use crate::vm::opcode::OpCode;
use crate::vm::virt_memory::VirtualFlash;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_channel_simple() {
    let mut code = Vec::new();

    // --- Main Task (Sender) at offset 0 ---
    // 0: CHAN_NEW -> [chan_id]
    code.push(OpCode::CHAN_NEW as u8);

    // 1: DUP -> [chan_id, chan_id]
    // DUP is 0x03. Stack op.
    code.push(OpCode::DUP as u8);

    // 2: SPAWN <ReceiverAddr=20> <ArgCount=1>
    // Pops one chan_id. Pushes task_id.
    // Length 6 bytes.
    code.push(OpCode::SPAWN as u8);
    code.extend_from_slice(&(20u32).to_le_bytes()); // Target address 20
    code.push(1); // Arg count

    // 8: POP -> Discard task_id. [chan_id]
    code.push(OpCode::POP as u8);

    // 9: CONST_I32 42 -> [chan_id, 42]
    code.push(OpCode::CONST_I32 as u8);
    code.extend_from_slice(&(42i32).to_le_bytes());

    // 14: SEND (pops 42, pops chan_id)
    code.push(OpCode::SEND as u8);
    // 15: HALT
    code.push(OpCode::HALT as u8);

    // Pad with NOPs until 20?
    // Current size: 1 (CHAN) + 1 (DUP) + 6 (SPAWN) + 1 (POP) + 5 (CONST) + 1 (SEND) + 1 (HALT) = 16 bytes.
    // Receiver starts at 20.
    // Need 4 padding bytes (NOP is 0x00).
    for _ in 0..4 {
        code.push(OpCode::NOP as u8);
    }

    // --- Receiver Task at offset 20 ---
    // Stack: [chan_id] (passed by SPAWN)
    // 20: RECV (pops chan_id) -> pushes val
    code.push(OpCode::RECV as u8);
    // 21: HALT
    code.push(OpCode::HALT as u8);

    // Init VM
    let flash = VirtualFlash::new_with_code(code);
    let mut vm = AutoVM::new(flash, 1024);

    let main_id = vm.spawn_task(0, 1024);
    // Run
    tokio::select! {
        _ = vm.run_task_loop() => {},
        _ = sleep(Duration::from_secs(2)) => {
            panic!("VM timed out");
        }
    }

    // Check Receiver (Task 1)
    let receiver_id = main_id + 1;
    if let Some(task_arc) = vm.tasks.get(&receiver_id).map(|r| r.value().clone()) {
        let task = task_arc.lock().await;
        // Result should be on stack.
        // Stack was [chan_id] -> RECV -> [val].
        if task.ram.sp > 0 {
            let val = task.ram.top().unwrap();
            assert_eq!(val, 42, "Receiver should have received 42");
        } else {
            panic!("Receiver stack empty, RECV might have failed or not run");
        }
    } else {
        panic!("Receiver task not found");
    }
}
