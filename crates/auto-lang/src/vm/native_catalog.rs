// Plan 249: Single source of truth for native function IDs.
// Each entry: (ID, CONST_NAME, shim_function)
// Covers all functions with self.register() calls in native.rs.
// Functions registered via register_shim_by_name (stdlib.rs) are NOT here — they stay manual.

/// Master catalog of native functions with shim bindings.
/// Each entry generates a NATIVE_* constant via gen_native_constants!
/// and a shim binding via bind_shims!
#[macro_export]
macro_rules! for_each_native {
    ($mac:ident) => {
        $mac! {
            // === Print/Assert ===
            (1, NATIVE_PRINT_I32, shim_print_i32),
            (2, NATIVE_PRINT_F32, shim_print_f32),
            (4, NATIVE_PRINT_F64, shim_print_f64),
            (3, NATIVE_PRINT_STR, shim_print_str),
            (2900, NATIVE_WRITE_STR, shim_write_str),
            (4, NATIVE_ASSERT, shim_assert),
            (5, NATIVE_ASSERT_EQ, shim_assert_eq),
            (6, NATIVE_ASSERT_NE, shim_assert_ne),
            (7, NATIVE_RUNTIME_PANIC, shim_runtime_panic),

            // === List (100-110, 205, 2060-2080) ===
            (100, NATIVE_LIST_NEW, shim_list_new),
            (101, NATIVE_LIST_PUSH, shim_list_push),
            (102, NATIVE_LIST_POP, shim_list_pop),
            (103, NATIVE_LIST_LEN, shim_list_len),
            (104, NATIVE_LIST_IS_EMPTY, shim_list_is_empty),
            (105, NATIVE_LIST_CLEAR, shim_list_clear),
            (106, NATIVE_LIST_GET, shim_list_get),
            (107, NATIVE_LIST_SET, shim_list_set),
            (108, NATIVE_LIST_INSERT, shim_list_insert),
            (109, NATIVE_LIST_REMOVE, shim_list_remove),
            (110, NATIVE_LIST_DROP, shim_list_drop),
            (118, NATIVE_LIST_RESERVE, shim_list_reserve),
            (205, NATIVE_LIST_CAPACITY, shim_list_capacity),
            (2060, NATIVE_LIST_MAP, shim_list_map),
            (2061, NATIVE_LIST_FILTER, shim_list_filter),
            (2062, NATIVE_LIST_FOREACH, shim_list_for_each),
            (2063, NATIVE_LIST_FIND, shim_list_find),
            (2064, NATIVE_LIST_ANY, shim_list_any),
            (2065, NATIVE_LIST_ALL, shim_list_all),
            (2066, NATIVE_LIST_REDUCE, shim_list_reduce),
            (2067, NATIVE_LIST_SORT, shim_list_sort),
            (2068, NATIVE_LIST_SORT_BY, shim_list_sort_by),
            (2080, NATIVE_LIST_JOIN, shim_list_join),
            (2069, NATIVE_LIST_CONTAINS, shim_list_contains),

            // === Result HOF ===
            (2070, NATIVE_RESULT_MAP_ERR, shim_result_map_err),

            // === Iterator (111-118) ===
            (111, NATIVE_LIST_ITER, shim_list_iter),
            (112, NATIVE_ITERATOR_NEXT, shim_iterator_next),
            (113, NATIVE_ITERATOR_MAP, shim_iterator_map),
            (114, NATIVE_ITERATOR_FILTER, shim_iterator_filter),
            (115, NATIVE_ITERATOR_COLLECT, shim_iterator_collect),
            (116, NATIVE_ITERATOR_REDUCE, shim_iterator_reduce),
            (117, NATIVE_ITERATOR_FIND, shim_iterator_find),
            (118, NATIVE_ITERATOR_ENUMERATE, shim_iterator_enumerate),

            // === HashMap (119-128, 1290-1292) ===
            (119, NATIVE_HASHMAP_NEW, shim_hashmap_new),
            (120, NATIVE_HASHMAP_INSERT_STR, shim_hashmap_insert_str),
            (121, NATIVE_HASHMAP_INSERT_INT, shim_hashmap_insert_int),
            (122, NATIVE_HASHMAP_GET_STR, shim_hashmap_get_str),
            (123, NATIVE_HASHMAP_GET_INT, shim_hashmap_get_int),
            (124, NATIVE_HASHMAP_CONTAINS, shim_hashmap_contains),
            (125, NATIVE_HASHMAP_REMOVE, shim_hashmap_remove),
            (126, NATIVE_HASHMAP_SIZE, shim_hashmap_size),
            (127, NATIVE_HASHMAP_CLEAR, shim_hashmap_clear),
            (128, NATIVE_HASHMAP_DROP, shim_hashmap_drop),
            (1290, NATIVE_HASHMAP_IS_EMPTY, shim_hashmap_is_empty),
            (1291, NATIVE_HASHMAP_GET_OR, shim_hashmap_get_or),
            (1292, NATIVE_HASHMAP_KEYS, shim_hashmap_keys),

            // === HashSet (129-135) ===
            (129, NATIVE_HASHSET_NEW, shim_hashset_new),
            (130, NATIVE_HASHSET_INSERT, shim_hashset_insert),
            (131, NATIVE_HASHSET_CONTAINS, shim_hashset_contains),
            (132, NATIVE_HASHSET_REMOVE, shim_hashset_remove),
            (133, NATIVE_HASHSET_SIZE, shim_hashset_size),
            (134, NATIVE_HASHSET_CLEAR, shim_hashset_clear),
            (135, NATIVE_HASHSET_DROP, shim_hashset_drop),

            // === StringBuilder (160-167) ===
            (160, NATIVE_STRINGBUILDER_NEW, shim_stringbuilder_new),
            (161, NATIVE_STRINGBUILDER_APPEND, shim_stringbuilder_append),
            (162, NATIVE_STRINGBUILDER_APPEND_INT, shim_stringbuilder_append_int),
            (163, NATIVE_STRINGBUILDER_APPEND_CHAR, shim_stringbuilder_append_char),
            (164, NATIVE_STRINGBUILDER_LEN, shim_stringbuilder_len),
            (165, NATIVE_STRINGBUILDER_CLEAR, shim_stringbuilder_clear),
            (166, NATIVE_STRINGBUILDER_DROP, shim_stringbuilder_drop),
            (167, NATIVE_STRINGBUILDER_BUILD, shim_stringbuilder_build),

            // === VecDeque (136-146) ===
            (136, NATIVE_VECDEQUE_NEW, shim_vecdeque_new),
            (137, NATIVE_VECDEQUE_PUSH_BACK, shim_vecdeque_push_back),
            (138, NATIVE_VECDEQUE_PUSH_FRONT, shim_vecdeque_push_front),
            (139, NATIVE_VECDEQUE_POP_BACK, shim_vecdeque_pop_back),
            (140, NATIVE_VECDEQUE_POP_FRONT, shim_vecdeque_pop_front),
            (141, NATIVE_VECDEQUE_FRONT, shim_vecdeque_front),
            (142, NATIVE_VECDEQUE_BACK, shim_vecdeque_back),
            (143, NATIVE_VECDEQUE_SIZE, shim_vecdeque_size),
            (144, NATIVE_VECDEQUE_IS_EMPTY, shim_vecdeque_is_empty),
            (145, NATIVE_VECDEQUE_CLEAR, shim_vecdeque_clear),
            (146, NATIVE_VECDEQUE_DROP, shim_vecdeque_drop),

            // === BTreeMap (147-157) ===
            (147, NATIVE_BTREEMAP_NEW, shim_btreemap_new),
            (148, NATIVE_BTREEMAP_INSERT, shim_btreemap_insert),
            (149, NATIVE_BTREEMAP_GET, shim_btreemap_get),
            (150, NATIVE_BTREEMAP_CONTAINS, shim_btreemap_contains),
            (151, NATIVE_BTREEMAP_REMOVE, shim_btreemap_remove),
            (152, NATIVE_BTREEMAP_SIZE, shim_btreemap_size),
            (153, NATIVE_BTREEMAP_IS_EMPTY, shim_btreemap_is_empty),
            (154, NATIVE_BTREEMAP_CLEAR, shim_btreemap_clear),
            (155, NATIVE_BTREEMAP_FIRST_KEY, shim_btreemap_first_key),
            (156, NATIVE_BTREEMAP_LAST_KEY, shim_btreemap_last_key),
            (157, NATIVE_BTREEMAP_DROP, shim_btreemap_drop),

            // === String (170-186) ===
            (170, NATIVE_STR_LEN, shim_str_len),
            (171, NATIVE_STRING_LEN, shim_string_len),
            (172, NATIVE_STR_NEW, shim_str_new),
            (173, NATIVE_STR_APPEND, shim_str_append),
            (174, NATIVE_INT_STR, shim_int_str),
            (175, NATIVE_STR_UPPER, shim_str_upper),
            (176, NATIVE_STRING_FROM, shim_string_from),
            (177, NATIVE_STRING_NEW, shim_string_new),
            (178, NATIVE_STRING_PUSH, shim_string_push),
            (179, NATIVE_STRING_POP, shim_string_pop),
            (180, NATIVE_STRING_GET, shim_string_get),
            (181, NATIVE_STRING_SET, shim_string_set),
            (182, NATIVE_STRING_INSERT, shim_string_insert),
            (183, NATIVE_STRING_REMOVE, shim_string_remove),
            (184, NATIVE_STRING_CLEAR, shim_string_clear),
            (185, NATIVE_STRING_IS_EMPTY, shim_string_is_empty),
            (186, NATIVE_STRING_RESERVE, shim_string_reserve),

            // === String/Uint Extensions ===
            (235, NATIVE_STR_BYTES, shim_str_bytes),
            (236, NATIVE_UINT_TO_HEX, shim_uint_to_hex),

            // === Memory Allocation (190-192) ===
            (190, NATIVE_ALLOC_ARRAY, shim_alloc_array),
            (191, NATIVE_REALLOC_ARRAY, shim_realloc_array),
            (192, NATIVE_FREE_ARRAY, shim_free_array),

            // === Heap/Storage (195-202) ===
            (195, NATIVE_HEAP_NEW, shim_heap_new),
            (196, NATIVE_HEAP_CAPACITY, shim_heap_capacity),
            (197, NATIVE_HEAP_TRY_GROW, shim_heap_try_grow),
            (198, NATIVE_HEAP_DROP, shim_heap_drop),
            (199, NATIVE_INLINE_INT64_NEW, shim_inline_int64_new),
            (200, NATIVE_INLINE_INT64_CAPACITY, shim_inline_int64_capacity),
            (201, NATIVE_INLINE_INT64_TRY_GROW, shim_inline_int64_try_grow),
            (202, NATIVE_INLINE_INT64_DROP, shim_inline_int64_drop),

            // === Bit Operations (210-234) — Plan 178 ===
            (210, NATIVE_INT_AND, shim_int_and),
            (211, NATIVE_INT_OR, shim_int_or),
            (212, NATIVE_INT_XOR, shim_int_xor),
            (213, NATIVE_INT_NOT, shim_int_not),
            (214, NATIVE_INT_SHL, shim_int_shl),
            (215, NATIVE_INT_SHR, shim_int_shr),
            (216, NATIVE_INT_SAR, shim_int_sar),
            (217, NATIVE_INT_ROL, shim_int_rol),
            (218, NATIVE_INT_ROR, shim_int_ror),
            (220, NATIVE_INT_COUNT_ONES, shim_int_count_ones),
            (221, NATIVE_INT_LEADING_ZEROS, shim_int_leading_zeros),
            (222, NATIVE_INT_TRAILING_ZEROS, shim_int_trailing_zeros),
            (223, NATIVE_INT_BITREV, shim_int_bitrev),
            (230, NATIVE_INT_BIT_READ, shim_int_bit_read),
            (231, NATIVE_INT_BIT_TEST, shim_int_bit_test),
            (232, NATIVE_INT_BIT_ON, shim_int_bit_on),
            (233, NATIVE_INT_BIT_OFF, shim_int_bit_off),
            (234, NATIVE_INT_BIT_FLIP, shim_int_bit_flip),

            // === Rand (1850-1854) — Plan 212 ===
            (1850, NATIVE_RAND_THREAD_RNG, shim_rand_thread_rng),
            (1851, NATIVE_RNG_GEN_RANGE, shim_rng_gen_range),
            (1852, NATIVE_RNG_GEN, shim_rng_gen),
            (1853, NATIVE_RNG_DROP, shim_rng_drop),
            (1854, NATIVE_RAND_RANDOM, shim_rand_random),

            // === Log ===
            (1804, NATIVE_LOG_NOOP, shim_log_noop),

            // === Regex Opaque (2450-2459) — Plan 212 ===
            (2450, NATIVE_RE_OPAQUE_NEW, shim_re_opaque_new),
            (2451, NATIVE_RE_OPAQUE_IS_MATCH, shim_re_opaque_is_match),
            (2452, NATIVE_RE_OPAQUE_FIND, shim_re_opaque_find),
            (2453, NATIVE_RE_OPAQUE_FIND_ALL, shim_re_opaque_find_all),
            (2454, NATIVE_RE_OPAQUE_REPLACE_ALL, shim_re_opaque_replace_all),
            (2455, NATIVE_RE_OPAQUE_CAPTURES, shim_re_opaque_captures),
            (2459, NATIVE_RE_OPAQUE_DROP, shim_re_opaque_drop),

            // === URL Opaque (2500-2511) — Plan 212 ===
            (2500, NATIVE_URL_OPAQUE_PARSE, shim_url_opaque_parse),
            (2501, NATIVE_URL_OPAQUE_SCHEME, shim_url_opaque_scheme),
            (2502, NATIVE_URL_OPAQUE_HOST_STR, shim_url_opaque_host_str),
            (2503, NATIVE_URL_OPAQUE_PATH, shim_url_opaque_path),
            (2504, NATIVE_URL_OPAQUE_FRAGMENT, shim_url_opaque_fragment),
            (2505, NATIVE_URL_OPAQUE_PORT, shim_url_opaque_port),
            (2506, NATIVE_URL_OPAQUE_QUERY_PAIRS, shim_url_opaque_query_pairs),
            (2510, NATIVE_URL_OPAQUE_QUERY, shim_url_opaque_query),
            (2511, NATIVE_URL_OPAQUE_TO_STRING, shim_url_opaque_to_string),
            (2507, NATIVE_URL_OPAQUE_JOIN, shim_url_opaque_join),
            (2508, NATIVE_URL_OPAQUE_ORIGIN, shim_url_opaque_origin),
            (2509, NATIVE_URL_OPAQUE_DROP, shim_url_opaque_drop),

            // === Semver Opaque (2600-2609) — Plan 212 ===
            (2600, NATIVE_SEMVER_OPAQUE_PARSE, shim_semver_opaque_parse),
            (2601, NATIVE_SEMVER_OPAQUE_MAJOR, shim_semver_opaque_major),
            (2602, NATIVE_SEMVER_OPAQUE_MINOR, shim_semver_opaque_minor),
            (2603, NATIVE_SEMVER_OPAQUE_PATCH, shim_semver_opaque_patch),
            (2604, NATIVE_SEMVER_OPAQUE_PRE, shim_semver_opaque_pre),
            (2605, NATIVE_SEMVER_OPAQUE_TO_STRING, shim_semver_opaque_to_string),
            (2606, NATIVE_SEMVER_OPAQUE_CMP_GT, shim_semver_opaque_cmp_gt),
            (2609, NATIVE_SEMVER_OPAQUE_DROP, shim_semver_opaque_drop),

            // === Chrono Opaque (2700-2709) — Plan 212 ===
            (2700, NATIVE_CHRONO_LOCAL_NOW, shim_chrono_local_now),
            (2701, NATIVE_CHRONO_YEAR, shim_chrono_year),
            (2702, NATIVE_CHRONO_MONTH, shim_chrono_month),
            (2703, NATIVE_CHRONO_DAY, shim_chrono_day),
            (2704, NATIVE_CHRONO_HOUR, shim_chrono_hour),
            (2705, NATIVE_CHRONO_MINUTE, shim_chrono_minute),
            (2706, NATIVE_CHRONO_SECOND, shim_chrono_second),
            (2707, NATIVE_CHRONO_TIMESTAMP, shim_chrono_timestamp),
            (2708, NATIVE_CHRONO_FORMAT, shim_chrono_format),
            (2709, NATIVE_CHRONO_DROP, shim_chrono_drop),

            // === Base64 (2710-2711) — Plan 212 ===
            (2710, NATIVE_BASE64_ENCODE, shim_base64_encode),
            (2711, NATIVE_BASE64_DECODE, shim_base64_decode),

            // === Hex (2720-2721) — Plan 212 ===
            (2720, NATIVE_HEX_ENCODE, shim_hex_encode),
            (2721, NATIVE_HEX_DECODE, shim_hex_decode),

            // === SHA2 Opaque (2730-2739) — Plan 212 ===
            (2730, NATIVE_SHA2_SHA256_NEW, shim_sha2_sha256_new),
            (2731, NATIVE_SHA2_UPDATE, shim_sha2_update),
            (2732, NATIVE_SHA2_FINALIZE, shim_sha2_finalize),
            (2739, NATIVE_SHA2_DROP, shim_sha2_drop),

            // === Mime (2740) — Plan 212 ===
            (2740, NATIVE_MIME_FROM_PATH, shim_mime_from_path),

            // === Math (1710-1733) — Plan 240 VM-1 ===
            (1716, NATIVE_MATH_SIN, shim_math_sin),
            (1717, NATIVE_MATH_COS, shim_math_cos),
            (1718, NATIVE_MATH_TAN, shim_math_tan),
            (1723, NATIVE_MATH_ABS_F, shim_math_abs_f),
            (1710, NATIVE_MATH_FLOOR, shim_math_floor),
            (1711, NATIVE_MATH_CEIL, shim_math_ceil),
            (1712, NATIVE_MATH_ROUND, shim_math_round),
            (1713, NATIVE_MATH_POW, shim_math_pow),
            (1731, NATIVE_MATH_POWF, shim_math_powf),
            (1730, NATIVE_MATH_POWI, shim_math_powi),
            (1719, NATIVE_MATH_EXP, shim_math_exp),
            (1720, NATIVE_MATH_LN, shim_math_ln),
            (1721, NATIVE_MATH_LOG2, shim_math_log2),
            (1722, NATIVE_MATH_LOG10, shim_math_log10),
            (1724, NATIVE_MATH_SIGNUM, shim_math_signum),
            (1726, NATIVE_MATH_ASIN, shim_math_asin),
            (1727, NATIVE_MATH_ACOS, shim_math_acos),
            (1728, NATIVE_MATH_ATAN, shim_math_atan),
            (1729, NATIVE_MATH_ATAN2, shim_math_atan2),
            (1732, NATIVE_MATH_TO_RADIANS, shim_math_to_radians),
            (1733, NATIVE_MATH_TO_DEGREES, shim_math_to_degrees),

            // === Instant (1203-1204) — Plan 240 ===
            (1203, NATIVE_INSTANT_NOW, shim_instant_now),
            (1204, NATIVE_INSTANT_ELAPSED, shim_instant_elapsed),

            // === OnceCell (2850-2852) — Plan 240 ===
            (2850, NATIVE_ONCE_NEW, shim_once_new),
            (2851, NATIVE_ONCE_SET, shim_once_set),
            (2852, NATIVE_ONCE_GET, shim_once_get),

            // === File I/O Opaque (1010-1013) — Plan 240 ===
            (1010, NATIVE_FILE_CREATE_HANDLE, shim_file_create_handle),
            (1011, NATIVE_FILE_OPEN_HANDLE, shim_file_open_handle),
            (1012, NATIVE_FILE_WRITE_HANDLE, shim_file_write_handle),
            (1013, NATIVE_FILE_TRY_CLONE, shim_file_try_clone)
        }
    };
}

/// Generate `pub const NATIVE_XXX: u16 = NNN;` for all catalog entries.
#[macro_export]
macro_rules! gen_native_constants {
    (($id:expr, $name:ident, $fn:ident) $(, $rest:tt)*) => {
        pub const $name: u16 = $id;
        gen_native_constants!($($rest),*);
    };
    () => {};
}

/// Consumer macro for for_each_native! that generates shim bindings.
/// IMPORTANT: This macro cannot use `self` directly due to #[macro_export] hygiene.
/// Instead, use it from a local wrapper in native.rs:
///   macro_rules! __bind_all { ... self.register($name, $fn); ... }
///   for_each_native!(__bind_all);
#[macro_export]
macro_rules! bind_shims {
    (($id:expr, $name:ident, $fn:ident) $(, $rest:tt)*) => {
        self.register($name, $fn);
        $crate::bind_shims!($($rest),*);
    };
    () => {};
}
