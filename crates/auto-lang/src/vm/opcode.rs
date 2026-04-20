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
    RESERVE_STACK = 0x06, // Reserve stack space for n_locals (prevents stack from overwriting locals)

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
    // Plan 075: Object field manipulation
    SET_FIELD = 0x2A,     // Plan 075: Set field on object (value, field_str_idx) -> void
    SET_ELEM = 0x2B,      // Plan 073: Set element in array (array_id, index, value) -> void
    GET_ELEM = 0x2C,      // Plan 073: Get element from array (array_id, index) -> value
    GET_FIELD = 0x2D,     // Plan 073: Get field from object (obj_id, field_str_idx) -> value
    CREATE_OBJ = 0x2E,    // Plan 073: Create object from field_count -> object_id
    CREATE_ARRAY = 0x2F,   // Plan 073: Create array from elem_count -> array_id
    ARRAY_LEN = 0x48,      // Plan 089: Get array length (array_id) -> length
    MOD_F = 0x49,          // f32 % f32 -> f32
    MOD_D = 0x4A,          // f64 % f64 -> f64
    PROMOTE_F64 = 0xF1, // Plan 073: Widen f32 to f64 (1 slot -> 2 slots)
    RET_D = 0xF2,       // RET for 2-slot return values (f64, u64, i64): pops 2 slots, writes 2 slots

    CREATE_RANGE = 0x75,  // Plan 073: Create exclusive range (0..10) from (start, end) -> range_value
    CREATE_RANGE_EQ = 0x76, // Plan 073: Create inclusive range (0..=10) from (start, end) -> range_value
    BUILD_FSTR = 0x77,    // Plan 073: Build f-string from part_count -> string
    NULL_COALESCE = 0x78, // Plan 073: May<T> null coalesce: ?? operator (left ?? right) -> value
    ERROR_PROPAGATE = 0x79, // Plan 073: May<T> error propagate: .? operator (expr.?) -> unwrapped_value
    CREATE_NODE = 0x74,   // Plan 073: Create node from name_str_idx, arg_count -> node_id (changed from 0x30 to avoid conflict with ADD)
    // Plan 120: Option and Result type opcodes
    CREATE_SOME = 0x7D,   // value -> Some(value) (wrap value in Some)
    CREATE_NONE = 0x7E,   // -> None (push None onto stack)
    CREATE_OK = 0x7F,     // value -> Ok(value) (wrap value in Ok)
    CREATE_ERR = 0xE0,    // str_idx -> Err(msg) (create error from string index)
    IS_SOME = 0xE1,       // option -> bool (check if Option is Some)
    IS_OK = 0xE2,         // result -> bool (check if Result is Ok)
    UNWRAP_SOME = 0xE3,   // Some(value) -> value (unwrap Option, panic if None)
    UNWRAP_OK = 0xE4,     // Ok(value) -> value (unwrap Result, panic if Err)
    UNWRAP_ERR = 0xE5,    // Err(msg) -> msg (unwrap Result error, panic if Ok)
    // Plan 162: Type cast: expr.as(Type) — runtime type conversion
    TYPE_CAST_I32 = 0xE6,   // value -> i32 (truncate/reinterpret to i32)
    TYPE_CAST_U32 = 0xE7,   // value -> u32 (truncate/reinterpret to u32)
    TYPE_CAST_I64 = 0xE8,   // value -> i64 (extend to i64)
    TYPE_CAST_U64 = 0xE9,   // value -> u64 (extend to u64)
    TYPE_CAST_F64 = 0xEA,   // value -> f64 (convert to f64)
    TYPE_CAST_PTR = 0xEB,   // value -> pointer (no-op, just type change)
    // Plan 162: Explicit type conversion: expr.to(Type) — may allocate/parse
    TYPE_TO_STR = 0xEC,     // i32 -> string (via .to_string())
    TYPE_TO_I32 = 0xED,     // value -> i32 (parse string or truncate)
    TYPE_TO_F64 = 0xEE,     // value -> f64 (parse string or convert)
    // Plan 193: Extended type conversions
    TYPE_F64_TO_STR = 0xF3, // f64 -> string
    TYPE_I64_TO_STR = 0xF4, // i64 -> string
    TYPE_U64_TO_STR = 0xF5, // u64 -> string (hex)
    TYPE_BOOL_TO_STR = 0xF6, // bool -> string
    TYPE_F64_TO_I32 = 0xF7, // f64 -> i32 (truncate)
    TYPE_STR_TO_I64 = 0xF8, // string -> i64
    TYPE_F32_TO_STR = 0xF9, // f32 -> string
    TYPE_F32_TO_I32 = 0xFA, // f32 -> i32 (truncate)
    // Plan 075: Template string opcodes
    TO_STR = 0x7A,        // Convert any value to string
    IS_NIL = 0x7B,        // Check if value is nil (returns 1 if nil, 0 otherwise)
    STR_CAT = 0x7C,       // Concatenate two strings (optimized string joining)

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

    // 64-bit integer arithmetic (u64 stored as two i32 slots: low, high)
    MOD_U64 = 0xEF,     // u64 % u64 -> u64

    // Plan 117: Type coercion for mixed arithmetic
    I32_TO_F32 = 0x46,  // Convert i32 to f32
    I64_TO_F64 = 0x47,  // Convert i64 to f64
    U64_TO_F64 = 0x4B,  // Convert u64 to f64 (unsigned, avoids sign extension)

    // 64-bit integer arithmetic (u64 stored as two i32 slots: low, high)
    ADD_U64 = 0x4C,     // u64 + u64 -> u64 (wrapping)
    SUB_U64 = 0x4D,     // u64 - u64 -> u64 (wrapping)
    MUL_U64 = 0x4E,     // u64 * u64 -> u64 (wrapping)
    DIV_U64 = 0x4F,     // u64 / u64 -> u64

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

    // f64 comparison (each pops 2+2 slots, pushes 1 bool)
    EQ_D = 0x56,
    NE_D = 0x57,
    LT_D = 0x58,
    GT_D = 0x59,
    LE_D = 0x5A,
    GE_D = 0x5B,

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
    // Plan 126: .go postfix operator - fire-and-forget spawn
    SPAWN_GO = 0x89, // future -> void (spawn Future in background, discard result)

    // === Plan 127: Task/Msg Execution Opcodes ===
    // Task message loop and handler dispatch
    TASK_LOOP = 0x8A,    // -> void (enter message processing loop)
                         // Blocks waiting for messages, dispatches to handlers
    HANDLE_MSG = 0x8B,   // msg_value -> void (dispatch message to matched handler)
                         // Uses PatternMatcher to route to correct handler
    REPLY = 0x8C,        // value -> void (send reply via current MessageContext)
                         // Used in on(ctx) handlers for ask/reply pattern

    // === Closures (Plan 071: Direct Capture) ===
    CLOSURE = 0x90,         // func_addr, capture_count × value -> closure_id: u32
    CAPTURE_VAR = 0x91,     // -> value (load variable by name)
    LOAD_CAPTURED = 0x92,   // closure_id -> value (load captured var by name)
    STORE_CAPTURED = 0x93,  // closure_id, value -> (store captured var by name)
    CALL_CLOSURE = 0x94,    // closure_id -> (call closure with captured env)

    // === Plan 076 Phase 3: Generic List Opcodes ===
    // Type-specific list operations for monomorphized generics
    CREATE_LIST_INT = 0xA0,     // -> list_id (create List<int> with Heap storage)
    CREATE_LIST_STR = 0xA1,     // -> list_id (create List<string> with Heap storage)
    CREATE_LIST_BOOL = 0xA2,    // -> list_id (create List<bool> with Heap storage)
    LIST_PUSH_INT = 0xA3,       // list_id, value: int -> void
    LIST_POP_INT = 0xA4,        // list_id -> int
    LIST_GET_INT = 0xA5,        // list_id, index: int -> int
    LIST_SET_INT = 0xA6,        // list_id, index: int, value: int -> void

    // === Plan 076 Phase 4: Storage Strategy Opcodes ===
    // InlineInt64 storage variants (fixed 64-element capacity, no heap)
    CREATE_LIST_INT_INLINE = 0xA7,  // -> list_id (create List<int> with InlineInt64 storage)
    CREATE_LIST_STR_INLINE = 0xA8,  // -> list_id (create List<string> with InlineInt64 storage)
    CREATE_LIST_BOOL_INLINE = 0xA9, // -> list_id (create List<bool> with InlineInt64 storage)

    // === Plan 087 Phase 2: Generic Instance Opcodes ===
    // Support for user-defined generic types (type erasure)
    NEW_INSTANCE = 0xB0,      // mono_name_len, mono_name_bytes -> instance_id
                             // Create a new generic instance (uninitialized)
    CONSTRUCT_INSTANCE = 0xB1, // instance_id, field_count × value -> void
                             // Construct instance by populating fields from stack
    GET_GENERIC_FIELD = 0xB2, // instance_id, field_index -> value
                             // Get field value from generic instance
    SET_GENERIC_FIELD = 0xB3, // instance_id, field_index, value -> void
                             // Set field value in generic instance

    // === Plan 088 Phase 4: Reference Passing Opcodes ===
    // Support for parameter passing modes (view, mut, take, copy)
    LOAD_REF = 0xB4,          // var_index: u32 -> reference (load immutable reference)
                             // Load an immutable reference to a local variable
    STORE_REF = 0xB5,         // var_index: u32, value -> void (store via immutable reference)
                             // Store a value through an immutable reference (error if not supported)
    LOAD_MUT_REF = 0xB6,      // var_index: u32 -> mut_reference (load mutable reference)
                             // Load a mutable reference to a local variable
    STORE_MUT_REF = 0xB7,     // var_index: u32, value -> void (store via mutable reference)
                             // Store a value through a mutable reference

    // === Plan 088 Phase 4: Function Prologue ===
    // Function metadata for dynamic parameter counting
    FN_PROLOG = 0xB8,         // n_args: u8, n_locals: u8 -> void
                             // Function prologue: record argument and local count
                             // Used by LOAD_LOCAL/STORE_LOCAL to calculate stack offsets

    // === Plan 197 Task 15: Enum Variant Pattern Matching ===
    IS_VARIANT = 0xB9,       // instance_id, name_len:u16, name_bytes... -> bool
                             // Check if heap object is a GenericInstanceData with matching mono_name
                             // Returns true (-2147483648) or false (-2147483647)

    // === Plan 124: Async/Future/Await Opcodes ===
    // Future type operations for async system
    CREATE_FUTURE = 0xC0,    // body_code_offset: u32 -> future_id
                             // Create a Future from async block body
                             // The body is compiled separately and stored
    AWAIT_FUTURE = 0xC1,     // future_id -> value (or suspend)
                             // Wait for future completion, returns inner value
                             // If future is pending, suspends current task
    POLL_FUTURE = 0xC2,      // future_id -> (is_ready: bool, value_or_nil)
                             // Non-blocking poll: check if future is ready
                             // Returns (true, value) if ready, (false, nil) if pending

    // === Debug ===
    PRINT = 0xF0,
    HALT = 0xFF,
}

impl From<u8> for OpCode {
    fn from(v: u8) -> Self {
        unsafe { std::mem::transmute(v) }
    }
}

impl OpCode {
    /// All valid opcode values
    const VALID: &[u8] = &[
        // Stack ops
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
        // Constants
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
        0x1F, // LOAD_STR
        // Memory
        0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29,
        0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F,
        // Arithmetic
        0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39,
        0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F,
        // Comparison & bitwise
        0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A,
        0x4B, // U64_TO_F64
        // 64-bit integer arithmetic
        0x4C, 0x4D, 0x4E, 0x4F,
        // Control flow
        0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58,
        // Jump
        0x60, 0x61, 0x62, 0x63,
        // Objects/Strings
        0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79,
        0x7A, 0x7B, 0x7C, 0x7D, 0x7E, 0x7F,
        // Functions
        0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89,
        0x8A, 0x8B,
        // Closures
        0x90, 0x91, 0x92, 0x93, 0x94, 0x95,
        // Arrays
        0xA0, 0xA1, 0xA2, 0xA3,
        // Iterators
        0xB0, 0xB1, 0xB2, 0xB3, 0xB4, 0xB5,
        // FN_PROLOG + IS_VARIANT
        0xB8, 0xB9,
        // Async
        0xC0, 0xC1, 0xC2,
        // Error/Option
        0xE0, 0xE1, 0xE2, 0xE3, 0xE4, 0xE5,
        // Type casts
        0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xEB, 0xEC, 0xED, 0xEE,
        // 64-bit modulo
        0xEF,
        // Extended type conversions (Plan 193)
        0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8, 0xF9, 0xFA,
        // Debug/Misc
        0xF0, 0xF1, 0xF2, 0xFF,
    ];

    pub fn is_valid(v: u8) -> bool {
        Self::VALID.contains(&v)
    }
}
