/// BigVM Execution Engine
/// The core loop that executes AutoByteCode (ABC).
use crate::vm::opcode::OpCode;
use crate::vm::virt_memory::{VirtualFlash, VirtualRAM};

#[derive(Debug)]
pub enum VMError {
    StackOverflow,
    StackUnderflow,
    InvalidOpCode(u8),
    DivisionByZero,
    Halt,
}

pub struct BigVM {
    pub flash: VirtualFlash,
    pub ram: VirtualRAM,
    pub ip: usize, // Instruction Pointer
    pub bp: usize, // Base Pointer (Start of current stack frame)
}

impl BigVM {
    pub fn new(flash: VirtualFlash, ram_size: usize) -> Self {
        Self {
            flash,
            ram: VirtualRAM::new(ram_size),
            ip: 0,
            bp: 0,
        }
    }

    /// Run the VM until completion or error
    pub fn run(&mut self) -> Result<(), VMError> {
        loop {
            // 1. Fetch
            if self.ip >= self.flash.memory.len() {
                // End of code
                return Ok(());
            }

            let op_byte = self.flash.read_u8(self.ip);
            self.ip += 1;

            let op: OpCode = op_byte.into();

            // 2. Decode & Execute
            match op {
                OpCode::NOP => {
                    // Do nothing
                }

                // === Constants ===
                OpCode::CONST_I32 => {
                    let val = self.flash.read_i32(self.ip);
                    self.ip += 4;
                    self.ram.push_i32(val);
                }
                OpCode::CONST_0 => {
                    self.ram.push_i32(0);
                }
                OpCode::CONST_1 => {
                    self.ram.push_i32(1);
                }

                // === Arithmetic ===
                OpCode::ADD => {
                    let b = self.ram.pop_i32();
                    let a = self.ram.pop_i32();
                    self.ram.push_i32(a.wrapping_add(b));
                }
                OpCode::SUB => {
                    let b = self.ram.pop_i32();
                    let a = self.ram.pop_i32();
                    self.ram.push_i32(a.wrapping_sub(b));
                }
                OpCode::MUL => {
                    let b = self.ram.pop_i32();
                    let a = self.ram.pop_i32();
                    self.ram.push_i32(a.wrapping_mul(b));
                }
                OpCode::DIV => {
                    let b = self.ram.pop_i32();
                    let a = self.ram.pop_i32();
                    if b == 0 {
                        return Err(VMError::DivisionByZero);
                    }
                    self.ram.push_i32(a.wrapping_div(b));
                }

                // === Control Flow ===
                OpCode::CALL => {
                    let target = self.flash.read_u32(self.ip) as usize;
                    self.ip += 4;

                    // Push Return Address (IP)
                    self.ram.push_i32(self.ip as i32);
                    // Push Old Stack Frame (BP)
                    self.ram.push_i32(self.bp as i32);

                    // New BP points to the saved BP location (SP - 1)
                    // SP is currently "next free", so the top item is at SP-1.
                    self.bp = self.ram.sp - 1;

                    // Jump
                    self.ip = target;
                }
                OpCode::RET => {
                    // Spec: RET n_args
                    let n_args = self.flash.read_u8(self.ip) as usize;
                    self.ip += 1;

                    // Expect Result on Top of Stack
                    let result = self.ram.pop_i32();

                    // BP points to [SavedBP].
                    // Stack: [Arg1] ... [ArgN] [SavedIP] [SavedBP] [Locals...] [Result]
                    //                                      ^ BP    ^ SP

                    // We want to restore SP to: Address of Arg1.
                    // Address of [SavedBP] is self.bp.
                    // Address of [SavedIP] is self.bp - 1.
                    // Address of [ArgN] is self.bp - 2.
                    // Address of [Arg1] is self.bp - 1 - n_args.

                    // Wait, let's trace CALL logic.
                    // CALL: Push IP, Push BP. BP = SP-1.
                    // So at entry of Function:
                    // Stack: [Arg1] ... [ArgN] [SavedIP] [SavedBP]
                    //                                      ^ BP / Top

                    // Then locals are pushed.
                    // Stack: ... [SavedBP] [L0] [L1] ... [Top]

                    // RET execution:
                    // 1. Pop Result.
                    // 2. Read n_args.
                    // 3. We want to pop locals, SavedBP, SavedIP, AND args.
                    // 4. And push Result.

                    let old_bp = self.ram.read_i32(self.bp) as usize;
                    let ret_ip = self.ram.read_i32(self.bp - 1) as usize;

                    // Calculate new SP.
                    // We want SP to be below SavedIP (which is BP-1).
                    // Specifically, we want to remove 'n_args' slots below SavedIP.
                    // Target SP = (self.bp - 1) - n_args.
                    // Since SP points to the *next free slot* (or top? No, auto-val stack usually: sp is usage count).
                    // My VirtualRAM implementation: sp is index of next free slot.
                    // So if stack has 1 element, sp=1. Top is at sp-1.

                    // If BP points to SavedBP slot.
                    // That slot index is BP.
                    // The slot index of SavedIP is BP - 1.
                    // The slot index of Last Argument is BP - 2.
                    // The slot index of First Argument is BP - 1 - n_args.

                    // We want the stack to end up containing [Result] at the position of First Argument.
                    // So new SP should be (BP - 1 - n_args) + 1.
                    // = BP - n_args.

                    // Let's verify.
                    // Args=2. BP=10.
                    // SavedBP at 10. SavedIP at 9.
                    // Arg2 at 8. Arg1 at 7.
                    // We want Arg1 (7) to be replaced by Result.
                    // So we write Result to 7.
                    // And SP should be 8.
                    // Formula: BP - n_args = 10 - 2 = 8. Correct.

                    let new_sp = self.bp - n_args;

                    // Write result to the new top (which is at new_sp - 1? No, new_sp is usage).
                    // Wait, if SP=8, valid indices are 0..7.
                    // We want Result at 7.

                    // Safety check for underflow
                    if self.bp < n_args {
                        panic!("Stack Underflow during RET argument cleanup");
                    }

                    self.ram.write_i32(new_sp - 1, result); // Wait, this writes to [7] NOT [8-1=7]? Yes.
                                                            // But wait, the loop above used:
                                                            // self.ram.sp = self.bp + 1;
                                                            // which means clean up locals only.

                    // Now we effectively do:
                    self.bp = old_bp;
                    self.ip = ret_ip;
                    self.ram.sp = new_sp;

                    // Wait, I need to WRITE the result *before* changing SP safely?
                    // actually, `self.ram.write_i32(addr, val)` doesn't check SP, just length.
                    // So we can write to `new_sp - 1` (which is `self.bp - n_args - 1`? No `self.bp - n_args - 1`... wait).
                    // If SP=8. Top index is 7.
                    // We calculated new_sp = 8.
                    // So we write to 7.
                    // Correct.

                    self.ram.write_i32(new_sp - 1, result);
                }

                // === Local Variables ===
                OpCode::LOAD_LOCAL => {
                    let idx = self.flash.read_u8(self.ip) as usize;
                    self.ip += 1;
                    let val = self.ram.read_i32(self.bp + idx);
                    self.ram.push_i32(val);
                }
                OpCode::STORE_LOCAL => {
                    let idx = self.flash.read_u8(self.ip) as usize;
                    self.ip += 1;
                    let val = self.ram.pop_i32();
                    self.ram.write_i32(self.bp + idx, val);
                }
                OpCode::LOAD_LOC_0 => {
                    let val = self.ram.read_i32(self.bp + 0);
                    self.ram.push_i32(val);
                }
                OpCode::LOAD_LOC_1 => {
                    let val = self.ram.read_i32(self.bp + 1);
                    self.ram.push_i32(val);
                }
                OpCode::LOAD_LOC_2 => {
                    let val = self.ram.read_i32(self.bp + 2);
                    self.ram.push_i32(val);
                }
                OpCode::STORE_LOC_0 => {
                    let val = self.ram.pop_i32();
                    self.ram.write_i32(self.bp + 0, val);
                }
                OpCode::STORE_LOC_1 => {
                    let val = self.ram.pop_i32();
                    self.ram.write_i32(self.bp + 1, val);
                }

                // === Stack ===
                OpCode::POP => {
                    self.ram.pop_i32();
                }

                // === Comparison ===
                OpCode::EQ => {
                    let b = self.ram.pop_i32();
                    let a = self.ram.pop_i32();
                    self.ram.push_i32(if a == b { 1 } else { 0 });
                }
                OpCode::NE => {
                    let b = self.ram.pop_i32();
                    let a = self.ram.pop_i32();
                    self.ram.push_i32(if a != b { 1 } else { 0 });
                }
                OpCode::LT => {
                    let b = self.ram.pop_i32();
                    let a = self.ram.pop_i32();
                    self.ram.push_i32(if a < b { 1 } else { 0 });
                }
                OpCode::GT => {
                    let b = self.ram.pop_i32();
                    let a = self.ram.pop_i32();
                    self.ram.push_i32(if a > b { 1 } else { 0 });
                }
                OpCode::LE => {
                    let b = self.ram.pop_i32();
                    let a = self.ram.pop_i32();
                    self.ram.push_i32(if a <= b { 1 } else { 0 });
                }
                OpCode::GE => {
                    let b = self.ram.pop_i32();
                    let a = self.ram.pop_i32();
                    self.ram.push_i32(if a >= b { 1 } else { 0 });
                }

                // === Control Flow ===
                OpCode::JMP => {
                    // JMP i16 offset
                    // Offset is relative to the *start* of the instruction?
                    // ABC Spec: "Offset = Target Address - (Current IP + 3)"
                    // But here, self.ip has already been incremented by 1 (opcode fetch).
                    // And reading i16 will increment it by 2 more.
                    // So after reading offset, self.ip points to Next Instruction (Instruction End).
                    // So if offset is calculated relative to (IP+3), and our IP IS (IP+3),
                    // then new_ip = current_ip + offset.

                    let offset = self.flash.read_i16(self.ip);
                    let after_read_ip = self.ip + 2;

                    // Logic: self.ip is currently opcode+1.
                    // read_i16 reads from ip, ip+1.
                    // We want to land at `after_read_ip + offset`.
                    // Wait, if offset is -3 (jump to self), valid?
                    // Opcode(1) + I16(2) = 3 bytes.
                    // If we are at N+3. N is start.
                    // Target = N.
                    // Offset = N - (N+3) = -3.
                    // So Target = (N+3) + (-3) = N.
                    // So yes, logic is: new_ip = (address after operand) + offset.

                    // Our read_i16 does NOT auto-increment self.ip in VirtualFlash helpers if not using a stream?
                    // Let's check: self.flash.read_i32 just reads. It doesn't modify self.ip.
                    // CONST_I32: self.ip += 4; is done manually.

                    // So:
                    // 1. Read offset.
                    // 2. Advance IP past operand.
                    // 3. Apply offset.

                    let offset = self.flash.read_i16(self.ip) as isize;
                    self.ip += 2; // Advance past operand

                    let new_ip = (self.ip as isize) + offset;

                    if new_ip < 0 || new_ip as usize >= self.flash.memory.len() {
                        // For now panic or return error.
                        // Returning Halt for now on bad jump to prevent loop
                        return Err(VMError::InvalidOpCode(0xFF)); // Using 0xFF as generic error for now or add InvalidJump
                    }

                    self.ip = new_ip as usize;
                }
                OpCode::JMP_IF_Z => {
                    let offset = self.flash.read_i16(self.ip) as isize;
                    self.ip += 2;

                    let cond = self.ram.pop_i32();
                    if cond == 0 {
                        let new_ip = (self.ip as isize) + offset;
                        if new_ip < 0 || new_ip as usize >= self.flash.memory.len() {
                            return Err(VMError::InvalidOpCode(0xFF));
                        }
                        self.ip = new_ip as usize;
                    }
                }
                OpCode::JMP_IF_NZ => {
                    let offset = self.flash.read_i16(self.ip) as isize;
                    self.ip += 2;

                    let cond = self.ram.pop_i32();
                    if cond != 0 {
                        let new_ip = (self.ip as isize) + offset;
                        if new_ip < 0 || new_ip as usize >= self.flash.memory.len() {
                            return Err(VMError::InvalidOpCode(0xFF));
                        }
                        self.ip = new_ip as usize;
                    }
                }

                // === Debug ===
                OpCode::HALT => {
                    return Err(VMError::Halt);
                }

                _ => {
                    // Unimplemented opcodes for Phase 1
                    return Err(VMError::InvalidOpCode(op_byte));
                }
            }
        }
    }
}
