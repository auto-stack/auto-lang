#![allow(non_camel_case_types)]

/// AutoByteCode (ABC) OpCode Definitions
/// Based on docs/design/abc.md

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OpCode {
    // === Stack Manipulation ===
    NOP = 0x00,
    POP = 0x01,
    POP_N = 0x02,
    DUP = 0x03,
    SWAP = 0x04,
    DROP = 0x05, // RAII cleanup: pops and frees owned value

    // === Constants ===
    CONST_I32 = 0x10,
    CONST_U8 = 0x11,
    CONST_0 = 0x12,
    CONST_1 = 0x13,
    CONST_F32 = 0x14,
    CONST_F64 = 0x15,     // Plan 073: Double precision
    CONST_I64 = 0x16,     // Plan 073: 64-bit integer
    CONST_U64 = 0x17,     // Plan 073: 64-bit unsigned
    LOAD_STR = 0x1F,
    CREATE_OBJ = 0x2E,    // Plan 073: Create object from field_count -> object_id
    GET_FIELD = 0x2F,     // Plan 073: Get field from object (obj_id, field_str_idx) -> value

    // === Local Variables ===
    LOAD_LOCAL = 0x20,
    STORE_LOCAL = 0x21,
    LOAD_LOC_0 = 0x22,
    LOAD_LOC_1 = 0x23,
    LOAD_LOC_2 = 0x24,
    STORE_LOC_0 = 0x25,
    STORE_LOC_1 = 0x26,

    // === Arithmetic & Logic ===
    ADD = 0x30,
    SUB = 0x31,
    MUL = 0x32,
    DIV = 0x33,
    MOD = 0x34,
    NEG = 0x35,

    // Plan 073: Floating-point arithmetic
    ADD_F = 0x36,     // f32 + f32 -> f32
    SUB_F = 0x37,     // f32 - f32 -> f32
    MUL_F = 0x38,     // f32 * f32 -> f32
    DIV_F = 0x39,     // f32 / f32 -> f32
    NEG_F = 0x3A,     // -f32 -> f32

    // Plan 073: Double precision arithmetic
    ADD_D = 0x3B,     // f64 + f64 -> f64
    SUB_D = 0x3C,     // f64 - f64 -> f64
    MUL_D = 0x3D,     // f64 * f64 -> f64
    DIV_D = 0x3E,     // f64 / f64 -> f64
    NEG_D = 0x3F,     // -f64 -> f64

    AND = 0x40,
    OR = 0x41,
    XOR = 0x42,
    NOT = 0x43,
    SHL = 0x44,
    SHR = 0x45,

    // === Comparison ===
    EQ = 0x50,
    NE = 0x51,
    LT = 0x52,
    GT = 0x53,
    LE = 0x54,
    GE = 0x55,

    // === Control Flow ===
    JMP = 0x60,
    JMP_IF_Z = 0x61,
    JMP_IF_NZ = 0x62,
    JMP_L = 0x63,

    // === Function Call ===
    CALL = 0x70,
    RET = 0x71,
    CALL_NAT = 0x72,

    // === Concurrency ===
    SPAWN = 0x80,    // func_id: u32, arg_count: u8 -> task_id: u32
    TASK_ID = 0x81,  // -> task_id: u32
    YIELD = 0x82,    // -> void
    SLEEP = 0x83,    // ms: u32 -> void
    JOIN = 0x84,     // task_id: u32 -> result
    CHAN_NEW = 0x85, // -> channel_id: u32
    SEND = 0x86,     // channel_id: u32, data: i32 -> void
    RECV = 0x87,     // channel_id: u32 -> data: i32
    TRY_RECV = 0x88, // channel_id: u32 -> data: i32 | 0 (non-blocking)

    // === Closures (Plan 071: Direct Capture) ===
    CLOSURE = 0x90,         // func_addr, capture_count × value -> closure_id: u32
    CAPTURE_VAR = 0x91,     // -> value (load variable by name)
    LOAD_CAPTURED = 0x92,   // closure_id -> value (load captured var by name)
    STORE_CAPTURED = 0x93,  // closure_id, value -> (store captured var by name)
    CALL_CLOSURE = 0x94,    // closure_id -> (call closure with captured env)

    // === Debug ===
    PRINT = 0xF0,
    HALT = 0xFF,
}

impl From<u8> for OpCode {
    fn from(v: u8) -> Self {
        unsafe { std::mem::transmute(v) }
    }
}
