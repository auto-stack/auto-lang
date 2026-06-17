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
    SLICE = 0x5C,          // Slice: (container, start, end) -> new_container, -1 = from start/end
    CREATE_TUPLE = 0x5D,   // Plan 200: Create tuple from elem_count -> tuple_id (heap object)
    GET_TUPLE_FIELD = 0x5E, // Plan 200: Get tuple field (tuple_id, field_index) -> value
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
    JMP_FAR = 0x64,

    // === Function Call ===
    CALL = 0x70,
    RET = 0x71,
    CALL_NAT = 0x72,
    CALL_SPEC = 0x73,  // Dynamic dispatch: spec_name_idx:u16, method_name_idx:u16 -> call vtable

    // === Concurrency ===
    SPAWN = 0x80,    // func_id: u32, arg_count: u8 -> task_id: u32
    TASK_ID = 0x81,  // -> task_id: u32
    YIELD_TASK = 0x82, // -> void (yield CPU back to task scheduler; NOT generator yield)
    SLEEP = 0x83,    // ms: u32 -> void
    JOIN = 0x84,     // task_id: u32 -> result
    CHAN_NEW = 0x85, // -> channel_id: u32
    SEND = 0x86,     // channel_id: u32, data: i32 -> void
    RECV = 0x87,     // channel_id: u32 -> data: i32
    TRY_RECV = 0x88, // channel_id: u32 -> data: i32 | 0 (non-blocking)
    // Plan 126: .go postfix operator - fire-and-forget spawn
    SPAWN_GO = 0x89, // future -> void (spawn Future in background, discard result)
    // Generator yield: pushes a value to the caller and suspends the generator
    // frame. Resumed by Iterator::Generator next(). Used by ~Iter<T> / ~Stream<T>.
    YIELD_VAL = 0x8D, // value -> void (to caller; generator frame suspended)

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
    SOURCE_LINE = 0xFE, // line: u16 -> void (Plan 199: record current source line)
    PRINT = 0xF0,
    PUSH_NIL = 0xFB,    // -> nil marker (TAG_NULL in nanbox, i32::MIN+1 otherwise)
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
        0x5C, // SLICE
        0x5D, // CREATE_TUPLE
        0x5E, // GET_TUPLE_FIELD
        // 64-bit integer arithmetic
        0x4C, 0x4D, 0x4E, 0x4F,
        // Control flow
        0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x5B,
        // Jump
        0x60, 0x61, 0x62, 0x63, 0x64,
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
        0xFB, // PUSH_NIL
        // Debug/Misc
        0xF0, 0xF1, 0xF2, 0xFE, 0xFF,
    ];

    pub fn is_valid(v: u8) -> bool {
        Self::VALID.contains(&v)
    }

    /// Plan 199: Get human-readable mnemonic for this opcode (for disassembly)
    pub fn to_mnemonic(self) -> &'static str {
        match self {
            Self::NOP => "nop",
            Self::POP => "pop",
            Self::POP_N => "pop.n",
            Self::DUP => "dup",
            Self::SWAP => "swap",
            Self::DROP => "drop",
            Self::RESERVE_STACK => "reserve",
            Self::CONST_I32 => "const.i32",
            Self::CONST_U8 => "const.u8",
            Self::CONST_0 => "const.0",
            Self::CONST_1 => "const.1",
            Self::CONST_F32 => "const.f32",
            Self::CONST_F64 => "const.f64",
            Self::CONST_I64 => "const.i64",
            Self::CONST_U64 => "const.u64",
            Self::LOAD_STR => "load.str",
            Self::SET_FIELD => "set.field",
            Self::SET_ELEM => "set.elem",
            Self::GET_ELEM => "get.elem",
            Self::GET_FIELD => "get.field",
            Self::CREATE_OBJ => "create.obj",
            Self::CREATE_ARRAY => "create.arr",
            Self::ARRAY_LEN => "arr.len",
            Self::MOD_F => "mod.f",
            Self::MOD_D => "mod.d",
            Self::SLICE => "slice",
            Self::CREATE_TUPLE => "create.tuple",
            Self::GET_TUPLE_FIELD => "get.tuple.field",
            Self::PROMOTE_F64 => "promote.f64",
            Self::RET_D => "ret.d",
            Self::CREATE_RANGE => "create.range",
            Self::CREATE_RANGE_EQ => "create.range.eq",
            Self::BUILD_FSTR => "build.fstr",
            Self::NULL_COALESCE => "null.coalesce",
            Self::ERROR_PROPAGATE => "error.propagate",
            Self::CREATE_NODE => "create.node",
            Self::CREATE_SOME => "create.some",
            Self::CREATE_NONE => "create.none",
            Self::CREATE_OK => "create.ok",
            Self::CREATE_ERR => "create.err",
            Self::IS_SOME => "is.some",
            Self::IS_OK => "is.ok",
            Self::UNWRAP_SOME => "unwrap.some",
            Self::UNWRAP_OK => "unwrap.ok",
            Self::UNWRAP_ERR => "unwrap.err",
            Self::TYPE_CAST_I32 => "cast.i32",
            Self::TYPE_CAST_U32 => "cast.u32",
            Self::TYPE_CAST_I64 => "cast.i64",
            Self::TYPE_CAST_U64 => "cast.u64",
            Self::TYPE_CAST_F64 => "cast.f64",
            Self::TYPE_CAST_PTR => "cast.ptr",
            Self::TYPE_TO_STR => "to.str",
            Self::TYPE_TO_I32 => "to.i32",
            Self::TYPE_TO_F64 => "to.f64",
            Self::TYPE_F64_TO_STR => "f64.to.str",
            Self::TYPE_I64_TO_STR => "i64.to.str",
            Self::TYPE_U64_TO_STR => "u64.to.str",
            Self::TYPE_BOOL_TO_STR => "bool.to.str",
            Self::TYPE_F64_TO_I32 => "f64.to.i32",
            Self::TYPE_STR_TO_I64 => "str.to.i64",
            Self::TYPE_F32_TO_STR => "f32.to.str",
            Self::TYPE_F32_TO_I32 => "f32.to.i32",
            Self::TO_STR => "to_str",
            Self::IS_NIL => "is.nil",
            Self::STR_CAT => "str.cat",
            Self::LOAD_LOCAL => "load.local",
            Self::STORE_LOCAL => "store.local",
            Self::LOAD_LOC_0 => "load.loc.0",
            Self::LOAD_LOC_1 => "load.loc.1",
            Self::LOAD_LOC_2 => "load.loc.2",
            Self::STORE_LOC_0 => "store.loc.0",
            Self::STORE_LOC_1 => "store.loc.1",
            Self::ADD => "add",
            Self::SUB => "sub",
            Self::MUL => "mul",
            Self::DIV => "div",
            Self::MOD => "mod",
            Self::NEG => "neg",
            Self::ADD_F => "add.f",
            Self::SUB_F => "sub.f",
            Self::MUL_F => "mul.f",
            Self::DIV_F => "div.f",
            Self::NEG_F => "neg.f",
            Self::ADD_D => "add.d",
            Self::SUB_D => "sub.d",
            Self::MUL_D => "mul.d",
            Self::DIV_D => "div.d",
            Self::NEG_D => "neg.d",
            Self::MOD_U64 => "mod.u64",
            Self::I32_TO_F32 => "i32.to.f32",
            Self::I64_TO_F64 => "i64.to.f64",
            Self::U64_TO_F64 => "u64.to.f64",
            Self::ADD_U64 => "add.u64",
            Self::SUB_U64 => "sub.u64",
            Self::MUL_U64 => "mul.u64",
            Self::DIV_U64 => "div.u64",
            Self::AND => "and",
            Self::OR => "or",
            Self::XOR => "xor",
            Self::NOT => "not",
            Self::SHL => "shl",
            Self::SHR => "shr",
            Self::EQ => "eq",
            Self::NE => "ne",
            Self::LT => "lt",
            Self::GT => "gt",
            Self::LE => "le",
            Self::GE => "ge",
            Self::EQ_D => "eq.d",
            Self::NE_D => "ne.d",
            Self::LT_D => "lt.d",
            Self::GT_D => "gt.d",
            Self::LE_D => "le.d",
            Self::GE_D => "ge.d",
            Self::JMP => "jmp",
            Self::JMP_IF_Z => "jmp.z",
            Self::JMP_IF_NZ => "jmp.nz",
            Self::JMP_L => "jmp.l",
            Self::JMP_FAR => "jmp.far",
            Self::CALL => "call",
            Self::RET => "ret",
            Self::CALL_NAT => "call.nat",
            Self::CALL_SPEC => "call.spec",
            Self::SPAWN => "spawn",
            Self::TASK_ID => "task.id",
            Self::YIELD_TASK => "yield.task",
            Self::SLEEP => "sleep",
            Self::JOIN => "join",
            Self::CHAN_NEW => "chan.new",
            Self::SEND => "send",
            Self::RECV => "recv",
            Self::TRY_RECV => "try.recv",
            Self::SPAWN_GO => "spawn.go",
            Self::YIELD_VAL => "yield.val",
            Self::TASK_LOOP => "task.loop",
            Self::HANDLE_MSG => "handle.msg",
            Self::REPLY => "reply",
            Self::CLOSURE => "closure",
            Self::CAPTURE_VAR => "capture.var",
            Self::LOAD_CAPTURED => "load.captured",
            Self::STORE_CAPTURED => "store.captured",
            Self::CALL_CLOSURE => "call.closure",
            Self::CREATE_LIST_INT => "create.list.int",
            Self::CREATE_LIST_STR => "create.list.str",
            Self::CREATE_LIST_BOOL => "create.list.bool",
            Self::LIST_PUSH_INT => "list.push.int",
            Self::LIST_POP_INT => "list.pop.int",
            Self::LIST_GET_INT => "list.get.int",
            Self::LIST_SET_INT => "list.set.int",
            Self::CREATE_LIST_INT_INLINE => "create.list.int.inline",
            Self::CREATE_LIST_STR_INLINE => "create.list.str.inline",
            Self::CREATE_LIST_BOOL_INLINE => "create.list.bool.inline",
            Self::NEW_INSTANCE => "new.instance",
            Self::CONSTRUCT_INSTANCE => "construct.instance",
            Self::GET_GENERIC_FIELD => "get.generic.field",
            Self::SET_GENERIC_FIELD => "set.generic.field",
            Self::LOAD_REF => "load.ref",
            Self::STORE_REF => "store.ref",
            Self::LOAD_MUT_REF => "load.mut.ref",
            Self::STORE_MUT_REF => "store.mut.ref",
            Self::FN_PROLOG => "fn.prolog",
            Self::IS_VARIANT => "is.variant",
            Self::CREATE_FUTURE => "create.future",
            Self::AWAIT_FUTURE => "await.future",
            Self::POLL_FUTURE => "poll.future",
            Self::SOURCE_LINE => ".line",
            Self::PRINT => "print",
            Self::PUSH_NIL => "push.nil",
            Self::HALT => "halt",
        }
    }

    /// Plan 226: Parse a mnemonic string back to an OpCode (for ABT assembler)
    pub fn from_mnemonic(m: &str) -> Option<Self> {
        match m {
            "nop" => Some(Self::NOP),
            "pop" => Some(Self::POP),
            "pop.n" => Some(Self::POP_N),
            "dup" => Some(Self::DUP),
            "swap" => Some(Self::SWAP),
            "drop" => Some(Self::DROP),
            "reserve" => Some(Self::RESERVE_STACK),
            "const.i32" => Some(Self::CONST_I32),
            "const.u8" => Some(Self::CONST_U8),
            "const.0" => Some(Self::CONST_0),
            "const.1" => Some(Self::CONST_1),
            "const.f32" => Some(Self::CONST_F32),
            "const.f64" => Some(Self::CONST_F64),
            "const.i64" => Some(Self::CONST_I64),
            "const.u64" => Some(Self::CONST_U64),
            "load.str" => Some(Self::LOAD_STR),
            "set.field" => Some(Self::SET_FIELD),
            "set.elem" => Some(Self::SET_ELEM),
            "get.elem" => Some(Self::GET_ELEM),
            "get.field" => Some(Self::GET_FIELD),
            "create.obj" => Some(Self::CREATE_OBJ),
            "create.arr" => Some(Self::CREATE_ARRAY),
            "arr.len" => Some(Self::ARRAY_LEN),
            "mod.f" => Some(Self::MOD_F),
            "mod.d" => Some(Self::MOD_D),
            "slice" => Some(Self::SLICE),
            "create.tuple" => Some(Self::CREATE_TUPLE),
            "get.tuple.field" => Some(Self::GET_TUPLE_FIELD),
            "promote.f64" => Some(Self::PROMOTE_F64),
            "ret.d" => Some(Self::RET_D),
            "create.range" => Some(Self::CREATE_RANGE),
            "create.range.eq" => Some(Self::CREATE_RANGE_EQ),
            "build.fstr" => Some(Self::BUILD_FSTR),
            "null.coalesce" => Some(Self::NULL_COALESCE),
            "error.propagate" => Some(Self::ERROR_PROPAGATE),
            "create.node" => Some(Self::CREATE_NODE),
            "create.some" => Some(Self::CREATE_SOME),
            "create.none" => Some(Self::CREATE_NONE),
            "create.ok" => Some(Self::CREATE_OK),
            "create.err" => Some(Self::CREATE_ERR),
            "is.some" => Some(Self::IS_SOME),
            "is.ok" => Some(Self::IS_OK),
            "unwrap.some" => Some(Self::UNWRAP_SOME),
            "unwrap.ok" => Some(Self::UNWRAP_OK),
            "unwrap.err" => Some(Self::UNWRAP_ERR),
            "cast.i32" => Some(Self::TYPE_CAST_I32),
            "cast.u32" => Some(Self::TYPE_CAST_U32),
            "cast.i64" => Some(Self::TYPE_CAST_I64),
            "cast.u64" => Some(Self::TYPE_CAST_U64),
            "cast.f64" => Some(Self::TYPE_CAST_F64),
            "cast.ptr" => Some(Self::TYPE_CAST_PTR),
            "to.str" => Some(Self::TYPE_TO_STR),
            "to.i32" => Some(Self::TYPE_TO_I32),
            "to.f64" => Some(Self::TYPE_TO_F64),
            "f64.to.str" => Some(Self::TYPE_F64_TO_STR),
            "i64.to.str" => Some(Self::TYPE_I64_TO_STR),
            "u64.to.str" => Some(Self::TYPE_U64_TO_STR),
            "bool.to.str" => Some(Self::TYPE_BOOL_TO_STR),
            "f64.to.i32" => Some(Self::TYPE_F64_TO_I32),
            "str.to.i64" => Some(Self::TYPE_STR_TO_I64),
            "f32.to.str" => Some(Self::TYPE_F32_TO_STR),
            "f32.to.i32" => Some(Self::TYPE_F32_TO_I32),
            "to_str" => Some(Self::TO_STR),
            "is.nil" => Some(Self::IS_NIL),
            "str.cat" => Some(Self::STR_CAT),
            "load.local" => Some(Self::LOAD_LOCAL),
            "store.local" => Some(Self::STORE_LOCAL),
            "load.loc.0" => Some(Self::LOAD_LOC_0),
            "load.loc.1" => Some(Self::LOAD_LOC_1),
            "load.loc.2" => Some(Self::LOAD_LOC_2),
            "store.loc.0" => Some(Self::STORE_LOC_0),
            "store.loc.1" => Some(Self::STORE_LOC_1),
            "add" => Some(Self::ADD),
            "sub" => Some(Self::SUB),
            "mul" => Some(Self::MUL),
            "div" => Some(Self::DIV),
            "mod" => Some(Self::MOD),
            "neg" => Some(Self::NEG),
            "add.f" => Some(Self::ADD_F),
            "sub.f" => Some(Self::SUB_F),
            "mul.f" => Some(Self::MUL_F),
            "div.f" => Some(Self::DIV_F),
            "neg.f" => Some(Self::NEG_F),
            "add.d" => Some(Self::ADD_D),
            "sub.d" => Some(Self::SUB_D),
            "mul.d" => Some(Self::MUL_D),
            "div.d" => Some(Self::DIV_D),
            "neg.d" => Some(Self::NEG_D),
            "mod.u64" => Some(Self::MOD_U64),
            "i32.to.f32" => Some(Self::I32_TO_F32),
            "i64.to.f64" => Some(Self::I64_TO_F64),
            "u64.to.f64" => Some(Self::U64_TO_F64),
            "add.u64" => Some(Self::ADD_U64),
            "sub.u64" => Some(Self::SUB_U64),
            "mul.u64" => Some(Self::MUL_U64),
            "div.u64" => Some(Self::DIV_U64),
            "and" => Some(Self::AND),
            "or" => Some(Self::OR),
            "xor" => Some(Self::XOR),
            "not" => Some(Self::NOT),
            "shl" => Some(Self::SHL),
            "shr" => Some(Self::SHR),
            "eq" => Some(Self::EQ),
            "ne" => Some(Self::NE),
            "lt" => Some(Self::LT),
            "gt" => Some(Self::GT),
            "le" => Some(Self::LE),
            "ge" => Some(Self::GE),
            "eq.d" => Some(Self::EQ_D),
            "ne.d" => Some(Self::NE_D),
            "lt.d" => Some(Self::LT_D),
            "gt.d" => Some(Self::GT_D),
            "le.d" => Some(Self::LE_D),
            "ge.d" => Some(Self::GE_D),
            "jmp" => Some(Self::JMP),
            "jmp.z" => Some(Self::JMP_IF_Z),
            "jmp.nz" => Some(Self::JMP_IF_NZ),
            "jmp.l" => Some(Self::JMP_L),
            "jmp.far" => Some(Self::JMP_FAR),
            "call" => Some(Self::CALL),
            "ret" => Some(Self::RET),
            "call.nat" => Some(Self::CALL_NAT),
            "call.spec" => Some(Self::CALL_SPEC),
            "spawn" => Some(Self::SPAWN),
            "task.id" => Some(Self::TASK_ID),
            "yield.task" => Some(Self::YIELD_TASK),
            "sleep" => Some(Self::SLEEP),
            "join" => Some(Self::JOIN),
            "chan.new" => Some(Self::CHAN_NEW),
            "send" => Some(Self::SEND),
            "recv" => Some(Self::RECV),
            "try.recv" => Some(Self::TRY_RECV),
            "spawn.go" => Some(Self::SPAWN_GO),
            "yield.val" => Some(Self::YIELD_VAL),
            "task.loop" => Some(Self::TASK_LOOP),
            "handle.msg" => Some(Self::HANDLE_MSG),
            "reply" => Some(Self::REPLY),
            "closure" => Some(Self::CLOSURE),
            "capture.var" => Some(Self::CAPTURE_VAR),
            "load.captured" => Some(Self::LOAD_CAPTURED),
            "store.captured" => Some(Self::STORE_CAPTURED),
            "call.closure" => Some(Self::CALL_CLOSURE),
            "create.list.int" => Some(Self::CREATE_LIST_INT),
            "create.list.str" => Some(Self::CREATE_LIST_STR),
            "create.list.bool" => Some(Self::CREATE_LIST_BOOL),
            "list.push.int" => Some(Self::LIST_PUSH_INT),
            "list.pop.int" => Some(Self::LIST_POP_INT),
            "list.get.int" => Some(Self::LIST_GET_INT),
            "list.set.int" => Some(Self::LIST_SET_INT),
            "create.list.int.inline" => Some(Self::CREATE_LIST_INT_INLINE),
            "create.list.str.inline" => Some(Self::CREATE_LIST_STR_INLINE),
            "create.list.bool.inline" => Some(Self::CREATE_LIST_BOOL_INLINE),
            "new.instance" => Some(Self::NEW_INSTANCE),
            "construct.instance" => Some(Self::CONSTRUCT_INSTANCE),
            "get.generic.field" => Some(Self::GET_GENERIC_FIELD),
            "set.generic.field" => Some(Self::SET_GENERIC_FIELD),
            "load.ref" => Some(Self::LOAD_REF),
            "store.ref" => Some(Self::STORE_REF),
            "load.mut.ref" => Some(Self::LOAD_MUT_REF),
            "store.mut.ref" => Some(Self::STORE_MUT_REF),
            "fn.prolog" => Some(Self::FN_PROLOG),
            "is.variant" => Some(Self::IS_VARIANT),
            "create.future" => Some(Self::CREATE_FUTURE),
            "await.future" => Some(Self::AWAIT_FUTURE),
            "poll.future" => Some(Self::POLL_FUTURE),
            ".line" => Some(Self::SOURCE_LINE),
            "print" => Some(Self::PRINT),
            "push.nil" => Some(Self::PUSH_NIL),
            "halt" => Some(Self::HALT),
            _ => None,
        }
    }
}
