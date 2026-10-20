// =============================================================================
// Plan 069 Phase 6: Concurrency Validation Tests
// =============================================================================
//
// These tests validate the M:N green thread scheduling, channel communication,
// and async sleep operations in BigVM.

use crate::vm::codegen::Codegen;
use crate::vm::engine::BigVM;
use crate::vm::opcode::OpCode;
use crate::vm::virt_memory::VirtualFlash;

#[tokio::test]
async fn test_01_interleaved_execution() {
    // Test: Two tasks, one sleeps 10ms, one sleeps 5ms
    // Expected: Both tasks complete successfully

    let mut codegen = Codegen::new();

    // Function: task_a - sleeps 10ms, returns 1
    let func_a_start = codegen.code.len();
    codegen.code.push(OpCode::SLEEP as u8);
    codegen.code.extend_from_slice(&10u32.to_le_bytes());
    codegen.code.push(OpCode::CONST_I32 as u8);
    codegen.code.extend_from_slice(&1i32.to_le_bytes());
    codegen.code.push(OpCode::RET as u8);
    codegen.code.push(0); // 0 args

    // Function: task_b - sleeps 5ms, returns 2
    let func_b_start = codegen.code.len();
    codegen.code.push(OpCode::SLEEP as u8);
    codegen.code.extend_from_slice(&5u32.to_le_bytes());
    codegen.code.push(OpCode::CONST_I32 as u8);
    codegen.code.extend_from_slice(&2i32.to_le_bytes());
    codegen.code.push(OpCode::RET as u8);
    codegen.code.push(0); // 0 args

    // Main function: spawn both tasks, then join them
    let main_start = codegen.code.len();
    codegen.code.push(OpCode::SPAWN as u8);
    codegen.code.extend_from_slice(&(func_a_start as u32).to_le_bytes());
    codegen.code.push(0); // 0 args

    codegen.code.push(OpCode::SPAWN as u8);
    codegen.code.extend_from_slice(&(func_b_start as u32).to_le_bytes());
    codegen.code.push(0); // 0 args

    codegen.code.push(OpCode::JOIN as u8); // Join task B
    codegen.code.push(OpCode::JOIN as u8); // Join task A

    codegen.code.push(OpCode::HALT as u8);

    // Create VM and run
    let flash = VirtualFlash::new_with_code(codegen.code);
    let mut vm = BigVM::new(flash, 4096);
    vm.load_strings(codegen.strings);

    // Spawn main task
    vm.spawn_task(main_start, 4096);

    // Run VM
    vm.run_task_loop().await;

    // Verify both tasks completed
    assert_eq!(vm.tasks.len(), 3); // main + task_a + task_b

    // Check that all tasks are terminated
    for task_entry in vm.tasks.iter() {
        let task = task_entry.value().clone();
        let task_locked = task.lock().await;
        assert_eq!(
            format!("{:?}", task_locked.status),
            "Terminated",
            "All tasks should be terminated"
        );
    }
}

#[tokio::test]
async fn test_02_channel_send_in_spawned_task() {
    // Test: Spawned task sends value to channel
    // This tests that channel operations work correctly in spawned tasks

    let mut codegen = Codegen::new();

    // Function: sender - sends value 42 to channel, returns success (1)
    let sender_start = codegen.code.len();
    codegen.code.push(OpCode::LOAD_LOC_0 as u8); // Load channel_id from arg
    codegen.code.push(OpCode::CONST_I32 as u8);
    codegen.code.extend_from_slice(&42i32.to_le_bytes());
    codegen.code.push(OpCode::SEND as u8);
    codegen.code.push(OpCode::CONST_I32 as u8);
    codegen.code.extend_from_slice(&1i32.to_le_bytes()); // Return 1 on success
    codegen.code.push(OpCode::RET as u8);
    codegen.code.push(1); // 1 arg (channel_id)

    // Main function: Create channel, spawn sender, join it
    let main_start = codegen.code.len();
    codegen.code.push(OpCode::CHAN_NEW as u8); // Stack: [chan_id]

    codegen.code.push(OpCode::SPAWN as u8); // Spawn sender (consumes chan_id)
    codegen.code.extend_from_slice(&(sender_start as u32).to_le_bytes());
    codegen.code.push(1); // 1 arg, Stack: [sender_id]

    codegen.code.push(OpCode::JOIN as u8); // Join sender, Stack: [sender_result]

    codegen.code.push(OpCode::CONST_I32 as u8); // Expected result
    codegen.code.extend_from_slice(&1i32.to_le_bytes());
    codegen.code.push(OpCode::EQ as u8); // Check if equal

    codegen.code.push(OpCode::HALT as u8);

    // Create VM and run
    let flash = VirtualFlash::new_with_code(codegen.code);
    let mut vm = BigVM::new(flash, 4096);
    vm.load_strings(codegen.strings);

    // Spawn main task
    let main_task_id = vm.spawn_task(main_start, 4096);

    // Run VM
    vm.run_task_loop().await;

    // Verify sender completed successfully (EQ result should be 1)
    let main_task_arc = vm.tasks.get(&main_task_id).map(|r| r.value().clone());
    if let Some(main_task) = main_task_arc {
        let task = main_task.lock().await;
        let result = task.ram.top();
        assert_eq!(result, Some(1), "Sender should have completed successfully, got {:?}", result);
    }
}

#[tokio::test]
async fn test_03_channel_recv_in_spawned_task() {
    // Test: Spawned task receives value from channel
    // Main task sends, spawned task receives

    let mut codegen = Codegen::new();

    // Function: receiver - receives from channel, returns received value
    let receiver_start = codegen.code.len();
    codegen.code.push(OpCode::LOAD_LOC_0 as u8); // Load channel_id from arg
    codegen.code.push(OpCode::RECV as u8);
    codegen.code.push(OpCode::RET as u8);
    codegen.code.push(1); // 1 arg (channel_id)

    // Main function:
    // 1. Create channel
    // 2. Spawn receiver (passing chan_id)
    // 3. Send value to channel
    // 4. Join receiver and get result

    let main_start = codegen.code.len();

    // To pass chan_id to both SPAWN and SEND, we need it twice
    // Solution: Create channel, DUP, spawn receiver (uses one copy), SEND (uses other copy)

    // But WAIT - after SPAWN, the chan_id is consumed and replaced with receiver_id
    // So we can't use the second copy for SEND.
    //
    // Solution: Send first (using chan_id directly), then spawn receiver
    // But we need chan_id for the spawn too...

    // Simplest: Just test that receiver can receive from a pre-filled channel
    // We'll do this by having main send, then spawning receiver which receives

    codegen.code.push(OpCode::CHAN_NEW as u8); // Stack: [chan_id]
    codegen.code.push(OpCode::DUP as u8); // Stack: [chan_id, chan_id]

    // Send value using first chan_id
    codegen.code.push(OpCode::CONST_I32 as u8);
    codegen.code.extend_from_slice(&99i32.to_le_bytes());
    codegen.code.push(OpCode::SEND as u8); // Consumes chan_id and value, Stack: [chan_id]

    // Now spawn receiver using second chan_id
    codegen.code.push(OpCode::SPAWN as u8);
    codegen.code.extend_from_slice(&(receiver_start as u32).to_le_bytes());
    codegen.code.push(1); // 1 arg, consumes chan_id, Stack: [receiver_id]

    codegen.code.push(OpCode::JOIN as u8); // Join receiver, Stack: [received_value]

    codegen.code.push(OpCode::CONST_I32 as u8);
    codegen.code.extend_from_slice(&99i32.to_le_bytes()); // Expected value
    codegen.code.push(OpCode::EQ as u8); // Check

    codegen.code.push(OpCode::HALT as u8);

    // Create VM and run
    let flash = VirtualFlash::new_with_code(codegen.code);
    let mut vm = BigVM::new(flash, 4096);
    vm.load_strings(codegen.strings);

    // Spawn main task
    let main_task_id = vm.spawn_task(main_start, 4096);

    // Run VM
    vm.run_task_loop().await;

    // Verify receiver got the value
    let main_task_arc = vm.tasks.get(&main_task_id).map(|r| r.value().clone());
    if let Some(main_task) = main_task_arc {
        let task = main_task.lock().await;
        let result = task.ram.top();
        assert_eq!(result, Some(1), "Receiver should have received 99, got {:?}", result);
    }
}

#[tokio::test]
async fn test_04_try_recv_nonblocking() {
    // Test: TRY_RECV returns 0 immediately when channel is empty
    // Expected: No busy-wait, immediate return with 0

    let mut codegen = Codegen::new();

    // Function: consumer - uses TRY_RECV on empty channel
    let consumer_start = codegen.code.len();
    codegen.code.push(OpCode::LOAD_LOC_0 as u8); // Load channel_id from arg
    codegen.code.push(OpCode::TRY_RECV as u8); // Try receive (should return 0)
    codegen.code.push(OpCode::RET as u8);
    codegen.code.push(1); // 1 arg (channel_id)

    // Main function
    let main_start = codegen.code.len();
    codegen.code.push(OpCode::CHAN_NEW as u8); // Create empty channel

    codegen.code.push(OpCode::SPAWN as u8);
    codegen.code.extend_from_slice(&(consumer_start as u32).to_le_bytes());
    codegen.code.push(1); // 1 arg (channel_id)

    codegen.code.push(OpCode::JOIN as u8); // Join consumer

    codegen.code.push(OpCode::HALT as u8);

    // Create VM and run
    let flash = VirtualFlash::new_with_code(codegen.code);
    let mut vm = BigVM::new(flash, 4096);
    vm.load_strings(codegen.strings);

    // Spawn main task
    let main_task_id = vm.spawn_task(main_start, 4096);

    // Run VM (should complete quickly, no blocking)
    vm.run_task_loop().await;

    // Verify consumer returned 0 (empty channel)
    let main_task_arc = vm.tasks.get(&main_task_id).map(|r| r.value().clone());
    if let Some(main_task) = main_task_arc {
        let task = main_task.lock().await;
        let result = task.ram.top();
        assert_eq!(result, Some(0), "TRY_RECV on empty channel should return 0, got {:?}", result);
    }
}

#[tokio::test]
async fn test_05_stress_test_many_tasks() {
    // Test: Spawn 100 tasks, each does simple addition
    // Expected: All tasks complete without deadlock

    let mut codegen = Codegen::new();

    // Function: simple_task - adds 1 + 2, returns result
    let task_start = codegen.code.len();
    codegen.code.push(OpCode::CONST_I32 as u8);
    codegen.code.extend_from_slice(&1i32.to_le_bytes());
    codegen.code.push(OpCode::CONST_I32 as u8);
    codegen.code.extend_from_slice(&2i32.to_le_bytes());
    codegen.code.push(OpCode::ADD as u8); // 1 + 2 = 3
    codegen.code.push(OpCode::RET as u8);
    codegen.code.push(0); // 0 args

    // Main function: spawn 100 tasks
    let main_start = codegen.code.len();
    for _ in 0..100 {
        codegen.code.push(OpCode::SPAWN as u8);
        codegen.code.extend_from_slice(&(task_start as u32).to_le_bytes());
        codegen.code.push(0); // 0 args
    }

    // Join all 100 tasks
    for _ in 0..100 {
        codegen.code.push(OpCode::JOIN as u8);
    }

    codegen.code.push(OpCode::HALT as u8);

    // Create VM and run
    let flash = VirtualFlash::new_with_code(codegen.code);
    let mut vm = BigVM::new(flash, 4096 * 10); // Larger RAM for many tasks
    vm.load_strings(codegen.strings);

    // Spawn main task
    vm.spawn_task(main_start, 4096);

    // Run VM
    vm.run_task_loop().await;

    // Verify all 101 tasks completed (main + 100 workers)
    assert_eq!(vm.tasks.len(), 101, "Should have 101 tasks total");

    // Verify all tasks terminated
    for task_entry in vm.tasks.iter() {
        let task = task_entry.value().clone();
        let task_locked = task.lock().await;
        assert_eq!(
            format!("{:?}", task_locked.status),
            "Terminated",
            "All tasks should be terminated"
        );
    }
}

#[tokio::test]
async fn test_06_task_id_opcode() {
    // Test: TASK_ID opcode returns correct task ID

    let mut codegen = Codegen::new();

    // Function: get_my_id - returns task ID
    let task_start = codegen.code.len();
    codegen.code.push(OpCode::TASK_ID as u8);
    codegen.code.push(OpCode::RET as u8);
    codegen.code.push(0); // 0 args

    // Main function
    let main_start = codegen.code.len();
    codegen.code.push(OpCode::SPAWN as u8);
    codegen.code.extend_from_slice(&(task_start as u32).to_le_bytes());
    codegen.code.push(0); // 0 args

    codegen.code.push(OpCode::JOIN as u8); // Join and get result

    codegen.code.push(OpCode::HALT as u8);

    // Create VM and run
    let flash = VirtualFlash::new_with_code(codegen.code);
    let mut vm = BigVM::new(flash, 4096);
    vm.load_strings(codegen.strings);

    // Spawn main task
    let main_task_id = vm.spawn_task(main_start, 4096);

    // Run VM
    vm.run_task_loop().await;

    // Verify spawned task returned its ID (should be task 1, main is 0)
    let main_task_arc = vm.tasks.get(&main_task_id).map(|r| r.value().clone());
    if let Some(main_task) = main_task_arc {
        let task = main_task.lock().await;
        let result = task.ram.top();
        assert_eq!(result, Some(1), "Spawned task should have ID 1, got {:?}", result);
    }
}
