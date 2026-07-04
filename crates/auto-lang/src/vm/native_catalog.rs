// Plan 249: Single source of truth for native function IDs.
// Each entry: (ID, CONST_NAME, shim_function, "canonical.name")
// Covers all functions with self.register() calls in native.rs.
// Functions registered via register_shim_by_name (stdlib.rs) are NOT here — they stay manual.

/// Master catalog of native functions with shim bindings.
/// Each entry generates a NATIVE_* constant via gen_native_constants!
/// and a shim binding via bind_shims!
/// 4-tuple format: (ID, CONST_NAME, shim_function, "canonical.name")
#[macro_export]
macro_rules! for_each_native {
    ($mac:ident) => {
        $mac! {
            // === Print/Assert ===
            (1, NATIVE_PRINT_I32, shim_print_i32, "auto.print_i32"),
            (2, NATIVE_PRINT_F32, shim_print_f32, "auto.print_f32"),
            (4, NATIVE_PRINT_F64, shim_print_f64, "auto.print_f64"),
            (3, NATIVE_PRINT_STR, shim_print_str, "auto.print_str"),
            (2900, NATIVE_WRITE_STR, shim_write_str, "auto.write_str"),
            (8, NATIVE_ASSERT, shim_assert, "auto.assert"),
            (5, NATIVE_ASSERT_EQ, shim_assert_eq, "auto.assert_eq"),
            (6, NATIVE_ASSERT_NE, shim_assert_ne, "auto.assert_ne"),
            (7, NATIVE_RUNTIME_PANIC, shim_runtime_panic, "auto.runtime_panic"),

            // === List (100-110, 205, 2060-2080) ===
            (100, NATIVE_LIST_NEW, shim_list_new, "auto.list.new"),
            (101, NATIVE_LIST_PUSH, shim_list_push, "auto.list.push"),
            (102, NATIVE_LIST_POP, shim_list_pop, "auto.list.pop"),
            (103, NATIVE_LIST_LEN, shim_list_len, "auto.list.len"),
            (104, NATIVE_LIST_IS_EMPTY, shim_list_is_empty, "auto.list.is_empty"),
            (105, NATIVE_LIST_CLEAR, shim_list_clear, "auto.list.clear"),
            (106, NATIVE_LIST_GET, shim_list_get, "auto.list.get"),
            (107, NATIVE_LIST_SET, shim_list_set, "auto.list.set"),
            (108, NATIVE_LIST_INSERT, shim_list_insert, "auto.list.insert"),
            (109, NATIVE_LIST_REMOVE, shim_list_remove, "auto.list.remove"),
            (110, NATIVE_LIST_DROP, shim_list_drop, "auto.list.drop"),
            (118, NATIVE_LIST_RESERVE, shim_list_reserve, "auto.list.reserve"),
            (205, NATIVE_LIST_CAPACITY, shim_list_capacity, "auto.list.capacity"),
            (2060, NATIVE_LIST_MAP, shim_list_map, "auto.list.map"),
            (2061, NATIVE_LIST_FILTER, shim_list_filter, "auto.list.filter"),
            (2062, NATIVE_LIST_FOREACH, shim_list_for_each, "auto.list.for_each"),
            (2063, NATIVE_LIST_FIND, shim_list_find, "auto.list.find"),
            (2064, NATIVE_LIST_ANY, shim_list_any, "auto.list.any"),
            (2065, NATIVE_LIST_ALL, shim_list_all, "auto.list.all"),
            (2066, NATIVE_LIST_REDUCE, shim_list_reduce, "auto.list.reduce"),
            (2067, NATIVE_LIST_SORT, shim_list_sort, "auto.list.sort"),
            (2068, NATIVE_LIST_SORT_BY, shim_list_sort_by, "auto.list.sort_by"),
            (2080, NATIVE_LIST_JOIN, shim_list_join, "auto.list.join"),
            (2069, NATIVE_LIST_CONTAINS, shim_list_contains, "auto.list.contains"),

            // === Result HOF ===
            (2070, NATIVE_RESULT_MAP_ERR, shim_result_map_err, "auto.result.map_err"),

            // === Iterator (111-118) ===
            (111, NATIVE_LIST_ITER, shim_list_iter, "auto.list.iter"),
            (112, NATIVE_ITERATOR_NEXT, shim_iterator_next, "auto.iterator.next"),
            (113, NATIVE_ITERATOR_MAP, shim_iterator_map, "auto.iterator.map"),
            (114, NATIVE_ITERATOR_FILTER, shim_iterator_filter, "auto.iterator.filter"),
            (115, NATIVE_ITERATOR_COLLECT, shim_iterator_collect, "auto.iterator.collect"),
            (116, NATIVE_ITERATOR_REDUCE, shim_iterator_reduce, "auto.iterator.reduce"),
            (117, NATIVE_ITERATOR_FIND, shim_iterator_find, "auto.iterator.find"),
            (119, NATIVE_ITERATOR_ENUMERATE, shim_iterator_enumerate, "auto.iterator.enumerate"),

            // === HashMap (120-128, 1290-1292) ===
            (120, NATIVE_HASHMAP_NEW, shim_hashmap_new, "auto.hashmap.new"),
            (121, NATIVE_HASHMAP_INSERT_STR, shim_hashmap_insert_str, "auto.hashmap.insert_str"),
            (122, NATIVE_HASHMAP_INSERT_INT, shim_hashmap_insert_int, "auto.hashmap.insert_int"),
            (123, NATIVE_HASHMAP_GET_STR, shim_hashmap_get_str, "auto.hashmap.get_str"),
            (124, NATIVE_HASHMAP_GET_INT, shim_hashmap_get_int, "auto.hashmap.get_int"),
            (125, NATIVE_HASHMAP_CONTAINS, shim_hashmap_contains, "auto.hashmap.contains"),
            (126, NATIVE_HASHMAP_REMOVE, shim_hashmap_remove, "auto.hashmap.remove"),
            (127, NATIVE_HASHMAP_SIZE, shim_hashmap_size, "auto.hashmap.size"),
            (128, NATIVE_HASHMAP_CLEAR, shim_hashmap_clear, "auto.hashmap.clear"),
            (129, NATIVE_HASHMAP_DROP, shim_hashmap_drop, "auto.hashmap.drop"),
            (1290, NATIVE_HASHMAP_IS_EMPTY, shim_hashmap_is_empty, "auto.hashmap.is_empty"),
            (1291, NATIVE_HASHMAP_GET_OR, shim_hashmap_get_or, "auto.hashmap.get_or"),
            (1292, NATIVE_HASHMAP_KEYS, shim_hashmap_keys, "auto.hashmap.keys"),

            // === HashSet (129-135) ===
            (129, NATIVE_HASHSET_NEW, shim_hashset_new, "auto.hashset.new"),
            (130, NATIVE_HASHSET_INSERT, shim_hashset_insert, "auto.hashset.insert"),
            (131, NATIVE_HASHSET_CONTAINS, shim_hashset_contains, "auto.hashset.contains"),
            (132, NATIVE_HASHSET_REMOVE, shim_hashset_remove, "auto.hashset.remove"),
            (133, NATIVE_HASHSET_SIZE, shim_hashset_size, "auto.hashset.size"),
            (134, NATIVE_HASHSET_CLEAR, shim_hashset_clear, "auto.hashset.clear"),
            (135, NATIVE_HASHSET_DROP, shim_hashset_drop, "auto.hashset.drop"),

            // === StringBuilder (160-167) ===
            (160, NATIVE_STRINGBUILDER_NEW, shim_stringbuilder_new, "auto.stringbuilder.new"),
            (161, NATIVE_STRINGBUILDER_APPEND, shim_stringbuilder_append, "auto.stringbuilder.append"),
            (162, NATIVE_STRINGBUILDER_APPEND_INT, shim_stringbuilder_append_int, "auto.stringbuilder.append_int"),
            (163, NATIVE_STRINGBUILDER_APPEND_CHAR, shim_stringbuilder_append_char, "auto.stringbuilder.append_char"),
            (164, NATIVE_STRINGBUILDER_LEN, shim_stringbuilder_len, "auto.stringbuilder.len"),
            (165, NATIVE_STRINGBUILDER_CLEAR, shim_stringbuilder_clear, "auto.stringbuilder.clear"),
            (166, NATIVE_STRINGBUILDER_DROP, shim_stringbuilder_drop, "auto.stringbuilder.drop"),
            (167, NATIVE_STRINGBUILDER_BUILD, shim_stringbuilder_build, "auto.stringbuilder.build"),

            // === VecDeque (136-146) ===
            (136, NATIVE_VECDEQUE_NEW, shim_vecdeque_new, "auto.vecdeque.new"),
            (137, NATIVE_VECDEQUE_PUSH_BACK, shim_vecdeque_push_back, "auto.vecdeque.push_back"),
            (138, NATIVE_VECDEQUE_PUSH_FRONT, shim_vecdeque_push_front, "auto.vecdeque.push_front"),
            (139, NATIVE_VECDEQUE_POP_BACK, shim_vecdeque_pop_back, "auto.vecdeque.pop_back"),
            (140, NATIVE_VECDEQUE_POP_FRONT, shim_vecdeque_pop_front, "auto.vecdeque.pop_front"),
            (141, NATIVE_VECDEQUE_FRONT, shim_vecdeque_front, "auto.vecdeque.front"),
            (142, NATIVE_VECDEQUE_BACK, shim_vecdeque_back, "auto.vecdeque.back"),
            (143, NATIVE_VECDEQUE_SIZE, shim_vecdeque_size, "auto.vecdeque.size"),
            (144, NATIVE_VECDEQUE_IS_EMPTY, shim_vecdeque_is_empty, "auto.vecdeque.is_empty"),
            (145, NATIVE_VECDEQUE_CLEAR, shim_vecdeque_clear, "auto.vecdeque.clear"),
            (146, NATIVE_VECDEQUE_DROP, shim_vecdeque_drop, "auto.vecdeque.drop"),

            // === BTreeMap (147-157) ===
            (147, NATIVE_BTREEMAP_NEW, shim_btreemap_new, "auto.btreemap.new"),
            (148, NATIVE_BTREEMAP_INSERT, shim_btreemap_insert, "auto.btreemap.insert"),
            (149, NATIVE_BTREEMAP_GET, shim_btreemap_get, "auto.btreemap.get"),
            (150, NATIVE_BTREEMAP_CONTAINS, shim_btreemap_contains, "auto.btreemap.contains"),
            (151, NATIVE_BTREEMAP_REMOVE, shim_btreemap_remove, "auto.btreemap.remove"),
            (152, NATIVE_BTREEMAP_SIZE, shim_btreemap_size, "auto.btreemap.size"),
            (153, NATIVE_BTREEMAP_IS_EMPTY, shim_btreemap_is_empty, "auto.btreemap.is_empty"),
            (154, NATIVE_BTREEMAP_CLEAR, shim_btreemap_clear, "auto.btreemap.clear"),
            (155, NATIVE_BTREEMAP_FIRST_KEY, shim_btreemap_first_key, "auto.btreemap.first_key"),
            (156, NATIVE_BTREEMAP_LAST_KEY, shim_btreemap_last_key, "auto.btreemap.last_key"),
            (157, NATIVE_BTREEMAP_DROP, shim_btreemap_drop, "auto.btreemap.drop"),

            // === String (170-186) ===
            (170, NATIVE_STR_LEN, shim_str_len, "auto.str.len"),
            (171, NATIVE_STRING_LEN, shim_string_len, "auto.string.len"),
            (172, NATIVE_STR_NEW, shim_str_new, "auto.str.new"),
            (173, NATIVE_STR_APPEND, shim_str_append, "auto.str.append"),
            (174, NATIVE_INT_STR, shim_int_str, "auto.int.str"),
            (175, NATIVE_STR_UPPER, shim_str_upper, "auto.str.upper"),
            (176, NATIVE_STRING_FROM, shim_string_from, "auto.string.from"),
            (177, NATIVE_STRING_NEW, shim_string_new, "auto.string.new"),
            (178, NATIVE_STRING_PUSH, shim_string_push, "auto.string.push"),
            (179, NATIVE_STRING_POP, shim_string_pop, "auto.string.pop"),
            (180, NATIVE_STRING_GET, shim_string_get, "auto.string.get"),
            (181, NATIVE_STRING_SET, shim_string_set, "auto.string.set"),
            (182, NATIVE_STRING_INSERT, shim_string_insert, "auto.string.insert"),
            (183, NATIVE_STRING_REMOVE, shim_string_remove, "auto.string.remove"),
            (184, NATIVE_STRING_CLEAR, shim_string_clear, "auto.string.clear"),
            (185, NATIVE_STRING_IS_EMPTY, shim_string_is_empty, "auto.string.is_empty"),
            (186, NATIVE_STRING_RESERVE, shim_string_reserve, "auto.string.reserve"),

            // === String/Uint Extensions ===
            (235, NATIVE_STR_BYTES, shim_str_bytes, "auto.str.bytes"),
            (236, NATIVE_UINT_TO_HEX, shim_uint_to_hex, "auto.uint.to_hex"),

            // === Memory Allocation (190-192) ===
            (190, NATIVE_ALLOC_ARRAY, shim_alloc_array, "auto.alloc.array"),
            (191, NATIVE_REALLOC_ARRAY, shim_realloc_array, "auto.realloc.array"),
            (192, NATIVE_FREE_ARRAY, shim_free_array, "auto.free.array"),

            // === Heap/Storage (195-202) ===
            (195, NATIVE_HEAP_NEW, shim_heap_new, "auto.heap.new"),
            (196, NATIVE_HEAP_CAPACITY, shim_heap_capacity, "auto.heap.capacity"),
            (197, NATIVE_HEAP_TRY_GROW, shim_heap_try_grow, "auto.heap.try_grow"),
            (198, NATIVE_HEAP_DROP, shim_heap_drop, "auto.heap.drop"),
            (199, NATIVE_INLINE_INT64_NEW, shim_inline_int64_new, "auto.inline_int64.new"),
            (200, NATIVE_INLINE_INT64_CAPACITY, shim_inline_int64_capacity, "auto.inline_int64.capacity"),
            (201, NATIVE_INLINE_INT64_TRY_GROW, shim_inline_int64_try_grow, "auto.inline_int64.try_grow"),
            (202, NATIVE_INLINE_INT64_DROP, shim_inline_int64_drop, "auto.inline_int64.drop"),

            // === Bit Operations (210-234) — Plan 178 ===
            (210, NATIVE_INT_AND, shim_int_and, "auto.int.and"),
            (211, NATIVE_INT_OR, shim_int_or, "auto.int.or"),
            (212, NATIVE_INT_XOR, shim_int_xor, "auto.int.xor"),
            (213, NATIVE_INT_NOT, shim_int_not, "auto.int.not"),
            (214, NATIVE_INT_SHL, shim_int_shl, "auto.int.shl"),
            (215, NATIVE_INT_SHR, shim_int_shr, "auto.int.shr"),
            (216, NATIVE_INT_SAR, shim_int_sar, "auto.int.sar"),
            (217, NATIVE_INT_ROL, shim_int_rol, "auto.int.rol"),
            (218, NATIVE_INT_ROR, shim_int_ror, "auto.int.ror"),
            (220, NATIVE_INT_COUNT_ONES, shim_int_count_ones, "auto.int.count_ones"),
            (221, NATIVE_INT_LEADING_ZEROS, shim_int_leading_zeros, "auto.int.leading_zeros"),
            (222, NATIVE_INT_TRAILING_ZEROS, shim_int_trailing_zeros, "auto.int.trailing_zeros"),
            (223, NATIVE_INT_BITREV, shim_int_bitrev, "auto.int.bitrev"),
            (230, NATIVE_INT_BIT_READ, shim_int_bit_read, "auto.int.bit_read"),
            (231, NATIVE_INT_BIT_TEST, shim_int_bit_test, "auto.int.bit_test"),
            (232, NATIVE_INT_BIT_ON, shim_int_bit_on, "auto.int.bit_on"),
            (233, NATIVE_INT_BIT_OFF, shim_int_bit_off, "auto.int.bit_off"),
            (234, NATIVE_INT_BIT_FLIP, shim_int_bit_flip, "auto.int.bit_flip"),

            // === Rand (1850-1854) — Plan 212 ===
            (1850, NATIVE_RAND_THREAD_RNG, shim_rand_thread_rng, "auto.rand.thread_rng"),
            (1851, NATIVE_RNG_GEN_RANGE, shim_rng_gen_range, "auto.rng.gen_range"),
            (1852, NATIVE_RNG_GEN, shim_rng_gen, "auto.rng.gen"),
            (1853, NATIVE_RNG_DROP, shim_rng_drop, "auto.rng.drop"),
            (1854, NATIVE_RAND_RANDOM, shim_rand_random, "auto.rand.random"),

            // === Log ===
            (1804, NATIVE_LOG_NOOP, shim_log_noop, "auto.log.noop"),

            // === Regex Opaque (2450-2459) — Plan 212 ===
            (2450, NATIVE_RE_OPAQUE_NEW, shim_re_opaque_new, "auto.re_opaque.new"),
            (2451, NATIVE_RE_OPAQUE_IS_MATCH, shim_re_opaque_is_match, "auto.re_opaque.is_match"),
            (2452, NATIVE_RE_OPAQUE_FIND, shim_re_opaque_find, "auto.re_opaque.find"),
            (2453, NATIVE_RE_OPAQUE_FIND_ALL, shim_re_opaque_find_all, "auto.re_opaque.find_all"),
            (2454, NATIVE_RE_OPAQUE_REPLACE_ALL, shim_re_opaque_replace_all, "auto.re_opaque.replace_all"),
            (2455, NATIVE_RE_OPAQUE_CAPTURES, shim_re_opaque_captures, "auto.re_opaque.captures"),
            (2459, NATIVE_RE_OPAQUE_DROP, shim_re_opaque_drop, "auto.re_opaque.drop"),

            // === URL Opaque (2500-2511) — Plan 212 ===
            (2500, NATIVE_URL_OPAQUE_PARSE, shim_url_opaque_parse, "auto.url_opaque.parse"),
            (2501, NATIVE_URL_OPAQUE_SCHEME, shim_url_opaque_scheme, "auto.url_opaque.scheme"),
            (2502, NATIVE_URL_OPAQUE_HOST_STR, shim_url_opaque_host_str, "auto.url_opaque.host_str"),
            (2503, NATIVE_URL_OPAQUE_PATH, shim_url_opaque_path, "auto.url_opaque.path"),
            (2504, NATIVE_URL_OPAQUE_FRAGMENT, shim_url_opaque_fragment, "auto.url_opaque.fragment"),
            (2505, NATIVE_URL_OPAQUE_PORT, shim_url_opaque_port, "auto.url_opaque.port"),
            (2506, NATIVE_URL_OPAQUE_QUERY_PAIRS, shim_url_opaque_query_pairs, "auto.url_opaque.query_pairs"),
            (2510, NATIVE_URL_OPAQUE_QUERY, shim_url_opaque_query, "auto.url_opaque.query"),
            (2511, NATIVE_URL_OPAQUE_TO_STRING, shim_url_opaque_to_string, "auto.url_opaque.to_string"),
            (2507, NATIVE_URL_OPAQUE_JOIN, shim_url_opaque_join, "auto.url_opaque.join"),
            (2508, NATIVE_URL_OPAQUE_ORIGIN, shim_url_opaque_origin, "auto.url_opaque.origin"),
            (2509, NATIVE_URL_OPAQUE_DROP, shim_url_opaque_drop, "auto.url_opaque.drop"),

            // === Semver Opaque (2600-2609) — Plan 212 ===
            (2600, NATIVE_SEMVER_OPAQUE_PARSE, shim_semver_opaque_parse, "auto.semver_opaque.parse"),
            (2601, NATIVE_SEMVER_OPAQUE_MAJOR, shim_semver_opaque_major, "auto.semver_opaque.major"),
            (2602, NATIVE_SEMVER_OPAQUE_MINOR, shim_semver_opaque_minor, "auto.semver_opaque.minor"),
            (2603, NATIVE_SEMVER_OPAQUE_PATCH, shim_semver_opaque_patch, "auto.semver_opaque.patch"),
            (2604, NATIVE_SEMVER_OPAQUE_PRE, shim_semver_opaque_pre, "auto.semver_opaque.pre"),
            (2605, NATIVE_SEMVER_OPAQUE_TO_STRING, shim_semver_opaque_to_string, "auto.semver_opaque.to_string"),
            (2606, NATIVE_SEMVER_OPAQUE_CMP_GT, shim_semver_opaque_cmp_gt, "auto.semver_opaque.cmp_gt"),
            (2609, NATIVE_SEMVER_OPAQUE_DROP, shim_semver_opaque_drop, "auto.semver_opaque.drop"),
            (2610, NATIVE_SEMVER_OPAQUE_VERSIONREQ_PARSE, shim_semver_opaque_versionreq_parse, "auto.semver_opaque_versionreq.parse"),
            (2611, NATIVE_SEMVER_OPAQUE_VERSIONREQ_MATCHES, shim_semver_opaque_versionreq_matches, "auto.semver_opaque_versionreq.matches"),

            // === Chrono Opaque (2700-2709) — Plan 212 ===
            (2700, NATIVE_CHRONO_LOCAL_NOW, shim_chrono_local_now, "auto.chrono_opaque.local_now"),
            (2701, NATIVE_CHRONO_YEAR, shim_chrono_year, "auto.chrono_opaque.year"),
            (2702, NATIVE_CHRONO_MONTH, shim_chrono_month, "auto.chrono_opaque.month"),
            (2703, NATIVE_CHRONO_DAY, shim_chrono_day, "auto.chrono_opaque.day"),
            (2704, NATIVE_CHRONO_HOUR, shim_chrono_hour, "auto.chrono_opaque.hour"),
            (2705, NATIVE_CHRONO_MINUTE, shim_chrono_minute, "auto.chrono_opaque.minute"),
            (2706, NATIVE_CHRONO_SECOND, shim_chrono_second, "auto.chrono_opaque.second"),
            (2707, NATIVE_CHRONO_TIMESTAMP, shim_chrono_timestamp, "auto.chrono_opaque.timestamp"),
            (2708, NATIVE_CHRONO_FORMAT, shim_chrono_format, "auto.chrono_opaque.format"),
            (2709, NATIVE_CHRONO_DROP, shim_chrono_drop, "auto.chrono_opaque.drop"),

            // === Base64 (2710-2711) — Plan 212 ===
            (2710, NATIVE_BASE64_ENCODE, shim_base64_encode, "auto.base64.encode"),
            (2711, NATIVE_BASE64_DECODE, shim_base64_decode, "auto.base64.decode"),

            // === Hex (2720-2721) — Plan 212 ===
            (2720, NATIVE_HEX_ENCODE, shim_hex_encode, "auto.hex.encode"),
            (2721, NATIVE_HEX_DECODE, shim_hex_decode, "auto.hex.decode"),

            // === SHA2 Opaque (2730-2739) — Plan 212 ===
            (2730, NATIVE_SHA2_SHA256_NEW, shim_sha2_sha256_new, "auto.sha2_opaque.sha256_new"),
            (2731, NATIVE_SHA2_UPDATE, shim_sha2_update, "auto.sha2_opaque.update"),
            (2732, NATIVE_SHA2_FINALIZE, shim_sha2_finalize, "auto.sha2_opaque.finalize"),
            (2739, NATIVE_SHA2_DROP, shim_sha2_drop, "auto.sha2_opaque.drop"),

            // === Mime (2740) — Plan 212 ===
            (2740, NATIVE_MIME_FROM_PATH, shim_mime_from_path, "auto.mime.from_path"),

            // === Math (1710-1733) — Plan 240 VM-1 ===
            (1716, NATIVE_MATH_SIN, shim_math_sin, "auto.math.sin"),
            (1717, NATIVE_MATH_COS, shim_math_cos, "auto.math.cos"),
            (1718, NATIVE_MATH_TAN, shim_math_tan, "auto.math.tan"),
            (1723, NATIVE_MATH_ABS_F, shim_math_abs_f, "auto.math.abs_f"),
            (1710, NATIVE_MATH_FLOOR, shim_math_floor, "auto.math.floor"),
            (1711, NATIVE_MATH_CEIL, shim_math_ceil, "auto.math.ceil"),
            (1712, NATIVE_MATH_ROUND, shim_math_round, "auto.math.round"),
            (1713, NATIVE_MATH_POW, shim_math_pow, "auto.math.pow"),
            (1731, NATIVE_MATH_POWF, shim_math_powf, "auto.math.powf"),
            (1730, NATIVE_MATH_POWI, shim_math_powi, "auto.math.powi"),
            (1719, NATIVE_MATH_EXP, shim_math_exp, "auto.math.exp"),
            (1720, NATIVE_MATH_LN, shim_math_ln, "auto.math.ln"),
            (1721, NATIVE_MATH_LOG2, shim_math_log2, "auto.math.log2"),
            (1722, NATIVE_MATH_LOG10, shim_math_log10, "auto.math.log10"),
            (1724, NATIVE_MATH_SIGNUM, shim_math_signum, "auto.math.signum"),
            (1726, NATIVE_MATH_ASIN, shim_math_asin, "auto.math.asin"),
            (1727, NATIVE_MATH_ACOS, shim_math_acos, "auto.math.acos"),
            (1728, NATIVE_MATH_ATAN, shim_math_atan, "auto.math.atan"),
            (1729, NATIVE_MATH_ATAN2, shim_math_atan2, "auto.math.atan2"),
            (1732, NATIVE_MATH_TO_RADIANS, shim_math_to_radians, "auto.math.to_radians"),
            (1733, NATIVE_MATH_TO_DEGREES, shim_math_to_degrees, "auto.math.to_degrees"),

            // === Instant (1203-1204) — Plan 240 ===
            (1203, NATIVE_INSTANT_NOW, shim_instant_now, "auto.time.instant_now"),
            (1204, NATIVE_INSTANT_ELAPSED, shim_instant_elapsed, "auto.time.instant_elapsed"),

            // === OnceCell (2850-2852) — Plan 240 ===
            (2850, NATIVE_ONCE_NEW, shim_once_new, "auto.cell.once_new"),
            (2851, NATIVE_ONCE_SET, shim_once_set, "auto.cell.once_set"),
            (2852, NATIVE_ONCE_GET, shim_once_get, "auto.cell.once_get"),

            // === File I/O Opaque (1010-1013) — Plan 240 ===
            (1010, NATIVE_FILE_CREATE_HANDLE, shim_file_create_handle, "auto.file.create_handle"),
            (1011, NATIVE_FILE_OPEN_HANDLE, shim_file_open_handle, "auto.file.open_handle"),
            (1012, NATIVE_FILE_WRITE_HANDLE, shim_file_write_handle, "auto.file.write_handle"),
            (1013, NATIVE_FILE_TRY_CLONE, shim_file_try_clone, "auto.file.try_clone"),

            // === Bool to String (2750) — Plan 250 ===
            (2750, NATIVE_BOOL_TO_STR, shim_bool_to_str, "auto.bool.to_str"),

            // === Float to String (2751) — Plan 250 ===
            (2751, NATIVE_F64_TO_STR, shim_f64_to_str, "auto.f64.to_str"),

            // === Result (2760-2769) — Plan 250 ===
            (2760, NATIVE_RESULT_IS_OK, shim_result_is_ok, "auto.result.is_ok"),
            (2761, NATIVE_RESULT_IS_ERR, shim_result_is_err, "auto.result.is_err"),
            (2762, NATIVE_RESULT_UNWRAP, shim_result_unwrap, "auto.result.unwrap"),
            (2763, NATIVE_RESULT_UNWRAP_OR, shim_result_unwrap_or, "auto.result.unwrap_or"),
            (2764, NATIVE_RESULT_UNWRAP_ERR, shim_result_unwrap_err, "auto.result.unwrap_err"),
            (2765, NATIVE_RESULT_OK, shim_result_ok, "auto.result.ok"),
            (2766, NATIVE_RESULT_ERR, shim_result_err, "auto.result.err"),

            // === List Reverse (2770) — Plan 250 ===
            (2770, NATIVE_LIST_REVERSE, shim_list_reverse, "auto.list.reverse"),

            // === Random convenience (2780-2783) — Plan 250 ===
            (2780, NATIVE_RAND_INT, shim_rand_int, "auto.rand.int"),
            (2781, NATIVE_RAND_FLOAT, shim_rand_float, "auto.rand.float"),
            (2782, NATIVE_RAND_BOOL, shim_rand_bool, "auto.rand.bool"),
            (2783, NATIVE_RAND_SHUFFLE, shim_rand_shuffle, "auto.rand.shuffle"),

            // === DateTime extended (2790-2793) — Plan 250 ===
            (2790, NATIVE_CHRONO_FROM_TIMESTAMP, shim_chrono_from_timestamp, "auto.chrono_opaque.from_timestamp"),
            (2791, NATIVE_CHRONO_FROM_YMD, shim_chrono_from_ymd, "auto.chrono_opaque.from_ymd"),
            (2792, NATIVE_CHRONO_WEEKDAY, shim_chrono_weekday, "auto.chrono_opaque.weekday"),

            // === CSV (2800-2803) — Plan 250 ===
            (2800, NATIVE_CSV_PARSE, shim_csv_parse, "auto.csv.parse"),
            (2801, NATIVE_CSV_PARSE_DELIM, shim_csv_parse_delim, "auto.csv.parse_delim"),
            (2802, NATIVE_CSV_ENCODE, shim_csv_encode, "auto.csv.encode"),
            (2803, NATIVE_CSV_ENCODE_DELIM, shim_csv_encode_delim, "auto.csv.encode_delim"),

            // === Hashing (2810-2815) — Plan 250 ===
            (2810, NATIVE_HASH_MD5, shim_hash_md5, "auto.hash.md5"),
            (2811, NATIVE_HASH_SHA1, shim_hash_sha1, "auto.hash.sha1"),
            (2812, NATIVE_HASH_SHA256, shim_hash_sha256, "auto.hash.sha256"),
            (2813, NATIVE_HASH_SHA512, shim_hash_sha512, "auto.hash.sha512"),

            // === Test assertions (2820-2827) — Plan 250 ===
            (2820, NATIVE_TEST_ASSERT_TRUE, shim_test_assert_true, "auto.test.assert_true"),
            (2821, NATIVE_TEST_ASSERT_FALSE, shim_test_assert_false, "auto.test.assert_false"),
            (2822, NATIVE_TEST_ASSERT_CONTAINS, shim_test_assert_contains, "auto.test.assert_contains"),
            (2823, NATIVE_TEST_ASSERT_LEN, shim_test_assert_len, "auto.test.assert_len"),
            (2824, NATIVE_TEST_ASSERT_OK, shim_test_assert_ok, "auto.test.assert_ok"),
            (2825, NATIVE_TEST_ASSERT_ERR, shim_test_assert_err, "auto.test.assert_err"),

            // === Format (2830-2832) — Plan 250 ===
            (2830, NATIVE_FMT_SPRINTF, shim_fmt_sprintf, "auto.fmt.sprintf"),
            (2831, NATIVE_FMT_PRINTF, shim_fmt_printf, "auto.fmt.printf"),
            (2832, NATIVE_FMT_EPRINTF, shim_fmt_eprintf, "auto.fmt.eprintf"),

            // === FS extended (2840-2847) — Plan 250 ===
            (2840, NATIVE_FS_TEMP_DIR, shim_fs_temp_dir, "auto.fs.temp_dir"),
            (2841, NATIVE_FS_TEMP_FILE, shim_fs_temp_file, "auto.fs.temp_file"),
            (2842, NATIVE_FS_RENAME, shim_fs_rename, "auto.fs.rename"),
            (2843, NATIVE_FS_READ_DIR, shim_fs_read_dir, "auto.fs.read_dir"),
            (2844, NATIVE_FS_CANONICAL, shim_fs_canonical, "auto.fs.canonical"),
            (2845, NATIVE_FS_EXT, shim_fs_ext, "auto.fs.ext"),
            (2846, NATIVE_FS_STEM, shim_fs_stem, "auto.fs.stem"),
            (2847, NATIVE_FS_WALK_FILES, shim_fs_walk_files, "auto.fs.walk_files"),

            // === FS more (2860-2865) ===
            (2860, NATIVE_FS_WALK, shim_fs_walk, "auto.fs.walk"),
            (2861, NATIVE_FS_METADATA, shim_fs_metadata, "auto.fs.metadata"),
            (2862, NATIVE_FS_COPY_RECURSIVE, shim_fs_copy_recursive, "auto.fs.copy_recursive"),
            (2863, NATIVE_FS_FILENAME, shim_fs_filename, "auto.fs.filename"),
            (2864, NATIVE_FS_PARENT, shim_fs_parent, "auto.fs.parent"),
            (2865, NATIVE_FS_JOIN, shim_fs_join, "auto.fs.join"),

            // === Hash extended (2814-2816) ===
            (2814, NATIVE_HASH_HMAC_SHA256, shim_hash_hmac_sha256, "auto.hash.hmac_sha256"),
            (2815, NATIVE_HASH_FILE_MD5, shim_hash_file_md5, "auto.hash.file_md5"),
            (2816, NATIVE_HASH_FILE_SHA256, shim_hash_file_sha256, "auto.hash.file_sha256"),

            // === Random type (2870-2874) ===
            (2870, NATIVE_RANDOM_NEW, shim_random_new, "auto.random._vm_new"),
            (2871, NATIVE_RANDOM_SEEDED, shim_random_seeded, "auto.random._vm_seeded"),
            (2872, NATIVE_RANDOM_INSTANCE_INT, shim_random_instance_int, "auto.random.int"),
            (2873, NATIVE_RANDOM_INSTANCE_FLOAT, shim_random_instance_float, "auto.random.float"),
            (2874, NATIVE_RANDOM_INSTANCE_BOOL, shim_random_instance_bool, "auto.random.bool"),

            // === Fmt (2752) ===
            (2752, NATIVE_F64_DEBUG, shim_f64_debug, "auto.fmt.f64_debug"),

            // === Cmp (2880) ===
            (2880, NATIVE_STR_CMP, shim_str_cmp, "auto.cmp.str_cmp"),

            // === DateTime cmp (2794) ===
            (2794, NATIVE_DATETIME_CMP, shim_datetime_cmp, "auto.datetime.cmp")
        }
    };
}

/// Generate `pub const NATIVE_XXX: u16 = NNN;` for all catalog entries.
#[macro_export]
macro_rules! gen_native_constants {
    (($id:expr, $name:ident, $fn:ident, $canonical:expr) $(, $rest:tt)*) => {
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
    (($id:expr, $name:ident, $fn:ident, $canonical:expr) $(, $rest:tt)*) => {
        self.register($name, $fn);
        $crate::bind_shims!($($rest),*);
    };
    () => {};
}

// Plan 249 Phase 4: Unified opaque method dispatch table.
// Each entry: (method_name, canonical_native_name).
// Used by both codegen.rs (compile-time) and engine.rs (runtime).
// Constructors (new, parse, now) are included — callers decide whether to filter.

/// Returns the canonical native function name for an opaque type + method,
/// or None if not found.
/// `type_key` is the crate/module name: "regex", "url", "semver", "chrono",
/// "base64", "hex", "sha2", "mime_guess", "std".
pub fn lookup_opaque_dispatch(type_key: &str, method: &str) -> Option<&'static str> {
    let entries = match type_key {
        "regex" => &OPAQUE_DISPATCH_REGEX[..],
        "url" => &OPAQUE_DISPATCH_URL[..],
        "semver" => &OPAQUE_DISPATCH_SEMVER[..],
        "chrono" => &OPAQUE_DISPATCH_CHRONO[..],
        "base64" => &OPAQUE_DISPATCH_BASE64[..],
        "hex" => &OPAQUE_DISPATCH_HEX[..],
        "sha2" => &OPAQUE_DISPATCH_SHA2[..],
        "mime_guess" => &OPAQUE_DISPATCH_MIME[..],
        _ => return None,
    };
    for &(m, native) in entries {
        if m == method {
            return Some(native);
        }
    }
    None
}

/// Runtime opaque dispatch by heap object type name.
/// Matches substrings in the type name (e.g., "regex::Regex", "Url").
/// Returns only instance methods (no constructors).
pub fn lookup_opaque_dispatch_by_type(type_name: &str, method: &str) -> Option<&'static str> {
    let entries: &[(&[&str], &[(&str, &str)])] = &[
        (&["regex::Regex", "Regex"], &OPAQUE_DISPATCH_REGEX_METHODS),
        (&["url::Url", "Url"], &OPAQUE_DISPATCH_URL_METHODS),
        (&["semver::Version", "Version"], &OPAQUE_DISPATCH_SEMVER_METHODS),
        (&["semver::VersionReq", "VersionReq"], &OPAQUE_DISPATCH_VERSIONREQ_METHODS),
        (&["Instant", "std::time::Instant"], &[("elapsed", "auto.time.instant_elapsed")]),
        (&["OnceCell", "std::cell::OnceCell"], &[("get", "auto.cell.once_get"), ("set", "auto.cell.once_set")]),
        (&["std::fs::File", "FileWriter"], &[("write", "auto.file.write_handle"), ("try_clone", "auto.file.try_clone")]),
    ];
    for &(type_patterns, methods) in entries {
        if type_patterns.iter().any(|p| type_name.contains(p)) {
            for &(m, native) in methods {
                if m == method {
                    return Some(native);
                }
            }
        }
    }
    None
}

// Per-type dispatch tables: (method_name, canonical_native_name)
// Includes both constructors and methods — callers filter as needed.

const OPAQUE_DISPATCH_REGEX: &[(&str, &str)] = &[
    ("new", "auto.re_opaque.new"),
    ("is_match", "auto.re_opaque.is_match"),
    ("find", "auto.re_opaque.find"),
    ("find_iter", "auto.re_opaque.find_all"),
    ("find_all", "auto.re_opaque.find_all"),
    ("replace_all", "auto.re_opaque.replace_all"),
    ("captures", "auto.re_opaque.captures"),
];

/// Regex instance methods only (no constructors) — for runtime dispatch.
const OPAQUE_DISPATCH_REGEX_METHODS: &[(&str, &str)] = &[
    ("is_match", "auto.re_opaque.is_match"),
    ("find", "auto.re_opaque.find"),
    ("find_iter", "auto.re_opaque.find_all"),
    ("find_all", "auto.re_opaque.find_all"),
    ("replace_all", "auto.re_opaque.replace_all"),
    ("captures", "auto.re_opaque.captures"),
];

const OPAQUE_DISPATCH_URL: &[(&str, &str)] = &[
    ("parse", "auto.url_opaque.parse"),
    ("scheme", "auto.url_opaque.scheme"),
    ("host", "auto.url_opaque.host_str"),
    ("host_str", "auto.url_opaque.host_str"),
    ("path", "auto.url_opaque.path"),
    ("query", "auto.url_opaque.query"),
    ("fragment", "auto.url_opaque.fragment"),
    ("port", "auto.url_opaque.port"),
    ("query_pairs", "auto.url_opaque.query_pairs"),
    ("query_params", "auto.url_opaque.query_pairs"),
    ("to_string", "auto.url_opaque.to_string"),
    ("join", "auto.url_opaque.join"),
    ("origin", "auto.url_opaque.origin"),
];

/// URL instance methods only (no constructors) — for runtime dispatch.
const OPAQUE_DISPATCH_URL_METHODS: &[(&str, &str)] = &[
    ("scheme", "auto.url_opaque.scheme"),
    ("host", "auto.url_opaque.host_str"),
    ("host_str", "auto.url_opaque.host_str"),
    ("path", "auto.url_opaque.path"),
    ("query", "auto.url_opaque.query"),
    ("fragment", "auto.url_opaque.fragment"),
    ("port", "auto.url_opaque.port"),
    ("query_pairs", "auto.url_opaque.query_pairs"),
    ("query_params", "auto.url_opaque.query_pairs"),
    ("join", "auto.url_opaque.join"),
    ("origin", "auto.url_opaque.origin"),
    ("to_string", "auto.url_opaque.to_string"),
];

const OPAQUE_DISPATCH_SEMVER: &[(&str, &str)] = &[
    ("parse", "auto.semver_opaque.parse"),
    ("major", "auto.semver_opaque.major"),
    ("minor", "auto.semver_opaque.minor"),
    ("patch", "auto.semver_opaque.patch"),
    ("pre", "auto.semver_opaque.pre"),
    ("to_string", "auto.semver_opaque.to_string"),
    ("matches", "auto.semver_opaque_versionreq.matches"),
];

/// Semver instance methods only — for runtime dispatch.
const OPAQUE_DISPATCH_SEMVER_METHODS: &[(&str, &str)] = &[
    ("major", "auto.semver_opaque.major"),
    ("minor", "auto.semver_opaque.minor"),
    ("patch", "auto.semver_opaque.patch"),
    ("pre", "auto.semver_opaque.pre"),
    ("to_string", "auto.semver_opaque.to_string"),
];

/// VersionReq methods — for runtime dispatch.
const OPAQUE_DISPATCH_VERSIONREQ_METHODS: &[(&str, &str)] = &[
    ("parse", "auto.semver_opaque_versionreq.parse"),
    ("matches", "auto.semver_opaque_versionreq.matches"),
];

const OPAQUE_DISPATCH_CHRONO: &[(&str, &str)] = &[
    ("now", "auto.chrono_opaque.local_now"),
    ("year", "auto.chrono_opaque.year"),
    ("month", "auto.chrono_opaque.month"),
    ("day", "auto.chrono_opaque.day"),
    ("hour", "auto.chrono_opaque.hour"),
    ("minute", "auto.chrono_opaque.minute"),
    ("second", "auto.chrono_opaque.second"),
    ("timestamp", "auto.chrono_opaque.timestamp"),
    ("format", "auto.chrono_opaque.format"),
];

const OPAQUE_DISPATCH_BASE64: &[(&str, &str)] = &[
    ("encode", "auto.base64.encode"),
    ("decode", "auto.base64.decode"),
];

const OPAQUE_DISPATCH_HEX: &[(&str, &str)] = &[
    ("encode", "auto.hex.encode"),
    ("decode", "auto.hex.decode"),
];

const OPAQUE_DISPATCH_SHA2: &[(&str, &str)] = &[
    ("new", "auto.sha2_opaque.sha256_new"),
    ("update", "auto.sha2_opaque.update"),
    ("finalize", "auto.sha2_opaque.finalize"),
];

const OPAQUE_DISPATCH_MIME: &[(&str, &str)] = &[
    ("from_path", "auto.mime.from_path"),
];

// Plan 249 Phase 3: BIGVM native registry entries — single source of truth.
// Each entry: (canonical_name, numeric_id, return_type_tag).
// return_type_tag: Void (default) → register_with_id; others → register_with_id_and_type.
// Uses numeric IDs directly (not NATIVE_* consts) to avoid cross-module dependency.
// The IDs must match for_each_native! shim entries — verified by tests.
#[macro_export]
macro_rules! for_each_bigvm_native {
    ($mac:ident) => {
        $mac! {
            // === List (100-110, 118, 205) ===
            ("auto.list.new", 100, Void),
            ("auto.list.push", 101, Void),
            ("auto.list.pop", 102, Void),
            ("auto.list.len", 103, Void),
            ("auto.list.is_empty", 104, Void),
            ("auto.list.clear", 105, Void),
            ("auto.list.get", 106, Void),
            ("auto.list.set", 107, Void),
            ("auto.list.insert", 108, Void),
            ("auto.list.remove", 109, Void),
            ("auto.list.drop", 110, Void),
            ("auto.list.reserve", 118, Void),
            ("auto.list.capacity", 205, Void),

            // === List HOF (2060-2069) ===
            ("auto.list.map", 2060, List),
            ("auto.list.filter", 2061, List),
            ("auto.list.for_each", 2062, Void),
            ("auto.list.find", 2063, Void),
            ("auto.list.any", 2064, Bool),
            ("auto.list.all", 2065, Bool),
            ("auto.list.reduce", 2066, Void),
            ("auto.list.sort", 2067, Void),
            ("auto.list.sort_by", 2068, Void),
            ("auto.list.contains", 2069, Bool),

            // === Iterator (111-118) ===
            ("auto.list.iter", 111, Void),
            ("auto.iterator.next", 112, Void),
            ("auto.iterator.map", 113, Void),
            ("auto.iterator.filter", 114, Void),
            ("auto.iterator.collect", 115, List),
            ("auto.iterator.reduce", 116, Void),
            ("auto.iterator.find", 117, Void),
            ("auto.iterator.enumerate", 119, Void),

            // === HashMap (120-129, 1290-1292) ===
            ("auto.hashmap.new", 120, Void),
            ("Map.new", 120, Map),
            ("HashMap.new", 120, Void),
            ("auto.hashmap.insert_str", 121, Void),
            ("auto.hashmap.insert_int", 122, Void),
            ("auto.hashmap.get_str", 123, Void),
            ("auto.hashmap.get_int", 124, Void),
            ("auto.hashmap.contains", 125, Void),
            ("auto.hashmap.remove", 126, Void),
            ("auto.hashmap.size", 127, Void),
            ("auto.hashmap.clear", 128, Void),
            ("auto.hashmap.drop", 129, Void),
            ("auto.hashmap.insert", 121, Void),
            ("auto.hashmap.get", 123, Void),
            ("auto.hashmap.keys", 1292, Void),

            // === HashSet (129-135) ===
            ("auto.hashset.new", 129, Void),
            ("auto.hashset.insert", 130, Void),
            ("auto.hashset.contains", 131, Void),
            ("auto.hashset.remove", 132, Void),
            ("auto.hashset.size", 133, Void),
            ("auto.hashset.clear", 134, Void),
            ("auto.hashset.drop", 135, Void),

            // === VecDeque (136-146) ===
            ("auto.vecdeque.new", 136, Void),
            ("auto.vecdeque.push_back", 137, Void),
            ("auto.vecdeque.push_front", 138, Void),
            ("auto.vecdeque.pop_back", 139, Void),
            ("auto.vecdeque.pop_front", 140, Void),
            ("auto.vecdeque.front", 141, Void),
            ("auto.vecdeque.back", 142, Void),
            ("auto.vecdeque.size", 143, Void),
            ("auto.vecdeque.is_empty", 144, Void),
            ("auto.vecdeque.clear", 145, Void),
            ("auto.vecdeque.drop", 146, Void),

            // === BTreeMap (147-157) ===
            ("auto.btreemap.new", 147, Void),
            ("auto.btreemap.insert", 148, Void),
            ("auto.btreemap.get", 149, Void),
            ("auto.btreemap.contains", 150, Void),
            ("auto.btreemap.remove", 151, Void),
            ("auto.btreemap.size", 152, Void),
            ("auto.btreemap.is_empty", 153, Void),
            ("auto.btreemap.clear", 154, Void),
            ("auto.btreemap.first_key", 155, Void),
            ("auto.btreemap.last_key", 156, Void),
            ("auto.btreemap.drop", 157, Void),

            // === StringBuilder (160-167) ===
            ("auto.stringbuilder.new", 160, Void),
            ("auto.stringbuilder.append", 161, Void),
            ("auto.stringbuilder.append_int", 162, Void),
            ("auto.stringbuilder.append_char", 163, Void),
            ("auto.stringbuilder.len", 164, Void),
            ("auto.stringbuilder.clear", 165, Void),
            ("auto.stringbuilder.drop", 166, Void),
            ("auto.stringbuilder.build", 167, Void),

            // === Heap/Storage (190-202) ===
            ("auto.heap.new", 195, Void),
            ("auto.heap.capacity", 196, Void),
            ("auto.heap.try_grow", 197, Void),
            ("auto.heap.drop", 198, Void),
            ("auto.inline_int64.new", 199, Void),
            ("auto.inline_int64.capacity", 200, Void),
            ("auto.inline_int64.try_grow", 201, Void),
            ("auto.inline_int64.drop", 202, Void),

            // === Memory Allocation (190-192) ===
            ("auto.alloc.array", 190, Void),
            ("auto.realloc.array", 191, Void),
            ("auto.free.array", 192, Void),

            // === String operations (170-186, 1500-1520) ===
            ("auto.str.len", 1500, Int),
            ("auto.str.is_empty", 1501, Bool),
            ("auto.str.char_at", 1502, Int),
            ("auto.str.substr", 1503, String),
            ("auto.str.sub", 1503, Void),
            ("auto.str.slice", 1503, Void),
            ("auto.str.contains", 1504, Bool),
            ("auto.str.starts_with", 1505, Bool),
            ("auto.str.ends_with", 1506, Bool),
            ("auto.str.trim", 1507, String),
            ("auto.str.split", 1508, List),
            ("auto.str.repeat", 1509, String),
            ("auto.str.replace", 1510, String),
            ("auto.str.to_upper", 1511, String),
            ("auto.str.to_lower", 1512, String),
            ("auto.str.upper", 175, Void),
            ("auto.str.lower", 1512, Void),
            ("auto.str.reverse", 1513, String),
            ("auto.str.find", 1514, Int),
            ("auto.str.lines", 1515, List),
            ("auto.str.parse_int", 1516, Int),
            ("auto.str.to_int", 1516, Void),
            ("auto.str.parse_float", 1517, Float),
            ("auto.str.new", 177, Void),
            ("auto.str.push", 178, Void),
            ("auto.str.pop", 179, Void),
            ("auto.str.get", 180, Void),
            ("auto.str.set", 181, Void),
            ("auto.str.insert", 182, Void),
            ("auto.str.remove", 183, Void),
            ("auto.str.clear", 184, Void),
            ("auto.str.reserve", 186, Void),
            ("auto.str.bytes", 235, Void),

            // === Bit Operations (210-234) ===
            ("auto.int.and", 210, Void),
            ("auto.int.or", 211, Void),
            ("auto.int.xor", 212, Void),
            ("auto.int.not", 213, Void),
            ("auto.int.shl", 214, Void),
            ("auto.int.shr", 215, Void),
            ("auto.int.sar", 216, Void),
            ("auto.int.rol", 217, Void),
            ("auto.int.ror", 218, Void),
            ("auto.int.count_ones", 220, Void),
            ("auto.int.leading_zeros", 221, Void),
            ("auto.int.trailing_zeros", 222, Void),
            ("auto.int.flip", 223, Void),
            ("auto.int.bit_read", 230, Void),
            ("auto.int.bit_test", 231, Void),
            ("auto.int.bit_on", 232, Void),
            ("auto.int.bit_off", 233, Void),
            ("auto.int.bit_flip", 234, Void),

            // === File (1000-1015) ===
            ("auto.file.read_text", 1000, Void),
            ("auto.file.write_text", 1001, Void),
            ("auto.file.exists", 1002, Void),
            ("auto.file.delete", 1003, Void),
            ("auto.file.create_dir", 1004, Void),
            ("auto.file.read_bytes", 1005, Void),
            ("auto.file.write_bytes", 1006, Void),
            ("auto.file.copy", 1007, Void),
            ("auto.file.size", 1008, Void),
            ("auto.file.is_dir", 1009, Void),
            ("auto.file.walk", 1010, Void),
            ("auto.file.append_text", 1011, Void),
            ("auto.file.read_lines", 1012, Void),
            ("auto.file.remove_dir", 1014, Void),
            ("auto.file.remove_dir_all", 1015, Void),

            // === FS module aliases ===
            ("auto.fs.read_text", 1000, Void),
            ("auto.fs.read", 1000, Void),
            ("auto.fs.write_text", 1001, Void),
            ("auto.fs.write", 1001, Void),
            ("auto.fs.append_text", 1011, Void),
            ("auto.fs.append", 1011, Void),
            ("auto.fs.exists", 1002, Void),
            ("auto.fs.delete", 1003, Void),
            ("auto.fs.create_dir", 1004, Void),
            ("auto.fs.remove_dir", 1014, Void),
            ("auto.fs.remove_dir_all", 1015, Void),
            ("auto.fs.read_bytes", 1005, Void),
            ("auto.fs.write_bytes", 1006, Void),
            ("auto.fs.copy", 1007, Void),
            ("auto.fs.size", 1008, Void),
            ("auto.fs.is_dir", 1009, Void),

            // === FS extended (2860-2865) ===
            ("auto.fs.walk", 2860, String),
            ("auto.fs.metadata", 2861, String),
            ("auto.fs.copy_recursive", 2862, Void),
            ("auto.fs.filename", 2863, String),
            ("auto.fs.parent", 2864, String),
            ("auto.fs.join", 2865, String),

            // === Hash extended (2814-2816) ===
            ("auto.hash.hmac_sha256", 2814, String),
            ("auto.hash.file_md5", 2815, String),
            ("auto.hash.file_sha256", 2816, String),

            // === Random type (2870-2874) ===
            ("auto.random._vm_new", 2870, Void),
            ("auto.random._vm_seeded", 2871, Void),
            ("auto.random.int", 2872, Void),
            ("auto.random.float", 2873, Void),
            ("auto.random.bool", 2874, Void),

            // === Fmt (2752) ===
            ("auto.fmt.f64_debug", 2752, String),

            // === Cmp (2880) ===
            ("auto.cmp.str_cmp", 2880, Int),

            // === DateTime cmp (2794) ===
            ("auto.datetime.cmp", 2794, Int),

            // === File I/O opaque handles (1010-1013) ===
            ("auto.file.create_handle", 1010, Void),
            ("auto.file.open_handle", 1011, Void),
            ("auto.file.write_handle", 1012, Void),
            ("auto.file.try_clone", 1013, Void),

            // === Env (1100-1104) ===
            ("auto.env.get", 1100, Void),
            ("auto.env.set", 1101, Void),
            ("auto.env.remove", 1102, Void),
            ("auto.env.get_or", 1103, Void),
            ("Env.get_or", 1103, Void),
            ("auto.env.local_data_dir", 1104, Void),
            ("env.local_data_dir", 1104, Void),
            ("auto.env.home_dir", 1105, Void),
            ("env.home_dir", 1105, Void),

            // === Time (1200-1204) ===
            ("auto.time.now_ms", 1200, I64),
            ("auto.time.now_sec", 1201, I64),
            ("auto.time.sleep_ms", 1202, Void),
            ("auto.time.instant_now", 1203, Void),
            ("auto.time.instant_elapsed", 1204, Void),
            ("auto.time.now", 1205, String),
            ("Time.now", 1205, String),

            // === IO (1150) ===
            ("auto.io.read_line", 1150, String),
            ("IO.read_line", 1150, String),
            ("io.read_line", 1150, String),

            // === OnceCell (2850-2852) ===
            ("auto.cell.once_new", 2850, Void),
            ("auto.cell.once_set", 2851, Void),
            ("auto.cell.once_get", 2852, Void),

            // === Process (1300-1305) ===
            ("auto.process.exit", 1300, Void),
            ("auto.process.args", 1301, Void),
            ("auto.process.current_dir", 1302, Void),
            ("auto.process.set_current_dir", 1303, Void),
            ("auto.process.spawn", 1304, Void),
            ("auto.process.spawn_with_output", 1305, Void),

            // === Path (1400-1404) ===
            ("auto.path.join", 1400, Void),
            ("auto.path.parent", 1401, Void),
            ("auto.path.extension", 1402, Void),
            ("auto.path.filename", 1403, Void),
            ("auto.path.canonicalize", 1404, Void),

            // === Char (1600-1606) ===
            ("auto.char.is_alpha", 1600, Void),
            ("auto.char.is_digit", 1601, Void),
            ("auto.char.is_alphanum", 1602, Void),
            ("auto.char.is_whitespace", 1603, Void),
            ("auto.char.is_ident", 1604, Void),
            ("auto.char.to_lower", 1605, Void),
            ("auto.char.to_upper", 1606, Void),

            // === Log (1800-1804) ===
            ("auto.log.debug", 1800, Void),
            ("auto.log.info", 1801, Void),
            ("auto.log.warn", 1802, Void),
            ("auto.log.error", 1803, Void),
            ("auto.log.noop", 1804, Void),
            ("Log.debug", 1800, Void),
            ("Log.info", 1801, Void),
            ("Log.warn", 1802, Void),
            ("Log.error", 1803, Void),

            // === Math (1700-1733) ===
            ("auto.math.abs", 1700, Int),
            ("auto.math.min", 1701, Int),
            ("auto.math.max", 1702, Int),
            ("auto.math.sqrt", 1750, Void),
            ("auto.math.floor", 1710, Void),
            ("auto.math.ceil", 1711, Void),
            ("auto.math.round", 1712, Void),
            ("auto.math.pow", 1713, Void),
            ("auto.math.min_f", 1714, Void),
            ("auto.math.max_f", 1715, Void),
            ("auto.math.sin", 1716, Void),
            ("auto.math.cos", 1717, Void),
            ("auto.math.tan", 1718, Void),
            ("auto.math.exp", 1719, Void),
            ("auto.math.ln", 1720, Void),
            ("auto.math.log2", 1721, Void),
            ("auto.math.log10", 1722, Void),
            ("auto.math.abs_f", 1723, Void),
            ("auto.math.signum", 1724, Void),
            ("auto.math.clamp", 1725, Void),
            ("auto.math.asin", 1726, Void),
            ("auto.math.acos", 1727, Void),
            ("auto.math.atan", 1728, Void),
            ("auto.math.atan2", 1729, Void),
            ("auto.math.powi", 1730, Void),
            ("auto.math.powf", 1731, Void),
            ("auto.math.to_radians", 1732, Void),
            ("auto.math.to_degrees", 1733, Void),

            // === Rand (1850-1854) ===
            ("auto.rand.thread_rng", 1850, Void),
            ("auto.rng.gen_range", 1851, Void),
            ("auto.rng.gen", 1852, Void),
            ("auto.rng.drop", 1853, Void),
            ("auto.rand.random", 1854, Void),

            // === JSON (1900-1917) ===
            ("auto.json.encode", 1900, Void),
            ("auto.json.decode", 1901, Void),
            ("auto.json.parse", 1902, Void),
            ("auto.json.prettify", 1903, Void),
            ("auto.json.minify", 1904, Void),
            ("auto.json.is_valid", 1905, Void),
            ("auto.json.get", 1906, Void),
            ("auto.json.get_at", 1907, Void),
            ("auto.json.len", 1908, Void),
            ("auto.json.type_of", 1909, Void),
            ("auto.json.as_string", 1910, Void),
            ("auto.json.as_number", 1911, Void),
            ("auto.json.as_int", 1912, Void),
            ("auto.json.as_bool", 1913, Void),
            ("auto.json.is_null", 1914, Void),
            ("auto.json.keys", 1915, Void),
            ("auto.json.has_key", 1917, Void),

            // === Plan 340: JSON ↔ VM value conversion + JSON HTTP helpers (3100-3106) ===
            ("auto.json.to_value", 3100, Void),
            ("auto.json.from_value", 3101, Void),
            ("Json.to_value", 3100, Void),
            ("Json.from_value", 3101, Void),
            ("auto.http.get_json", 3102, Void),
            ("auto.http.post_json", 3103, Void),
            ("auto.http.put_json", 3104, Void),
            ("auto.http.delete_json", 3105, Void),
            ("auto.http.sse_get_stream", 3106, Void),

            // === serde_json aliases ===
            ("auto.serde_json.to_string", 1900, Void),
            ("auto.json.to_string", 1900, Void),
            ("json.to_string", 1900, Void),
            ("Json.to_string", 1900, Void),
            ("auto.serde_json.from_str", 1902, Void),

            // === TOML (2750-2751) ===
            ("auto.toml.from_str", 2750, Void),
            ("auto.toml.to_string", 2751, Void),

            // === URL (2000-2015) ===
            ("auto.url.encode", 2000, Void),
            ("auto.url.decode", 2001, Void),
            ("auto.url.encode_query", 2002, Void),
            ("auto.url.decode_query", 2003, Void),
            ("auto.url.parse", 2006, Void),
            ("auto.url.scheme", 2007, Void),
            ("auto.url.host", 2008, Void),
            ("auto.url.port", 2009, Void),
            ("auto.url.path", 2010, Void),
            ("auto.url.query", 2011, Void),
            ("auto.url.fragment", 2012, Void),
            ("auto.url.join_path", 2015, Void),

            // === Net/TCP (2100-2113) ===
            ("auto.net.tcp_bind", 2100, Void),
            ("auto.net.tcp_listener_accept", 2101, Void),
            ("auto.net.tcp_listener_local_addr", 2102, Void),
            ("auto.net.tcp_listener_close", 2103, Void),
            ("auto.net.tcp_connect", 2104, Void),
            ("auto.net.tcp_stream_read", 2105, Void),
            ("auto.net.tcp_stream_write", 2106, Void),
            ("auto.net.tcp_stream_read_all", 2107, Void),
            ("auto.net.tcp_stream_read_line", 2108, Void),
            ("auto.net.tcp_stream_write_str", 2109, Void),
            ("auto.net.tcp_stream_close", 2110, Void),
            ("auto.net.tcp_stream_peer_addr", 2111, Void),
            ("auto.net.tcp_stream_set_read_timeout", 2112, Void),
            ("auto.net.tcp_stream_set_write_timeout", 2113, Void),
            // Plan 313: TCP flush + nodelay for SSE
            ("auto.net.tcp_stream_flush", 2114, Void),
            ("auto.net.tcp_stream_set_nodelay", 2115, Void),

            // === HTTP server (2200-2215) ===
            ("auto.http.server", 2200, Void),
            ("auto.http.server_get", 2201, Void),
            ("auto.http.server_post", 2202, Void),
            ("auto.http.server_put", 2203, Void),
            ("auto.http.server_delete", 2204, Void),
            ("auto.http.server_static", 2205, Void),
            ("auto.http.server_listen", 2206, Void),
            ("auto.http.response", 2210, Void),
            ("auto.http.response_status", 2211, Void),
            ("auto.http.response_header", 2212, Void),
            ("auto.http.response_text", 2213, Void),
            ("auto.http.response_html", 2214, Void),
            ("auto.http.response_bytes", 2215, Void),
            // Plan 351: Redirect
            ("auto.http.response.redirect", 2219, Void),
            ("http.response.redirect", 2219, Void),
            // Plan 352: Session management
            ("auto.session.create", 2284, Void),
            ("session.create", 2284, Void),
            ("auto.session.get", 2285, Void),
            ("session.get", 2285, Void),
            ("auto.session.set", 2286, Void),
            ("session.set", 2286, Void),
            ("auto.session.destroy", 2287, Void),
            ("session.destroy", 2287, Void),
            // Plan 352: Middleware chain
            ("auto.http.server_use", 2288, Void),
            ("http.server.use", 2288, Void),
            // Plan 352: Template engine (SSR)
            ("auto.template.compile", 2289, Void),
            ("template.compile", 2289, Void),
            ("auto.template.render", 2290, Void),
            ("template.render", 2290, Void),
            // Plan 352: OpenAPI auto-generation
            ("auto.openapi.generate", 2291, Void),
            ("openapi.generate", 2291, Void),

            // === HTTP response access (2216-2218) ===
            ("auto.http.response.status_code", 2216, Void),
            ("auto.http.response.header_get", 2217, Void),
            ("auto.http.response.body", 2218, Void),

            // === HTTP client helpers (2220-2224) ===
            ("auto.http.ok", 2220, Void),
            ("auto.http.created", 2221, Void),
            ("auto.http.bad_request", 2222, Void),
            ("auto.http.not_found", 2223, Void),
            ("auto.http.internal_error", 2224, Void),

            // === HTTP client (2230-2239) ===
            ("auto.http.get", 2230, Void),
            ("auto.http.post", 2231, Void),
            ("auto.http.put", 2232, Void),
            ("auto.http.delete", 2233, Void),
            ("auto.http.request", 2234, Void),
            ("auto.http.request_builder_header", 2235, Void),
            ("auto.http.request_builder_body", 2236, Void),
            ("auto.http.request_builder_timeout", 2237, Void),
            ("auto.http.request_builder_json", 2238, Void),
            ("auto.http.request_builder_send", 2239, Void),

            // === HTTP streaming (2240-2258) ===
            ("auto.http_stream.get_stream", 2240, Void),
            ("auto.http_stream.post_stream", 2241, Void),
            ("auto.http_stream.stream_next", 2242, Void),
            ("auto.http_stream.stream_is_done", 2243, Void),
            ("auto.http_stream.stream_close", 2244, Void),
            ("auto.http_stream.stream_iter", 2245, Void), // Plan 321: Iter protocol
            ("auto.http.post_stream_with_headers", 2255, Void),
            ("auto.http.post_sync", 2256, Void),
            ("auto.http.last_status", 2257, Void),
            ("auto.http.post_bearer", 2258, Void),
            ("auto.http.listen", 2259, Void),

            // === RequestBuilder chaining (2260-2264) ===
            ("RequestBuilder.header", 2260, Void),
            ("RequestBuilder.body", 2261, Void),
            ("RequestBuilder.timeout", 2262, Void),
            ("RequestBuilder.json", 2263, Void),
            ("RequestBuilder.send", 2264, Void),
            // Plan 350: TLS configuration
            ("RequestBuilder.tls_ca_cert", 2265, Void),
            ("RequestBuilder.tls_skip_verify", 2266, Void),
            ("RequestBuilder.tls_client_cert", 2267, Void),
            // Plan 349: Multipart file upload
            ("RequestBuilder.multipart_file", 2268, Void),
            ("RequestBuilder.multipart_text", 2269, Void),
            // Plan 349 step 8: Cookie / retry / compression
            ("RequestBuilder.cookie_store", 2272, Void),
            ("RequestBuilder.retry", 2273, Void),
            ("RequestBuilder.gzip", 2274, Void),
            ("auto.http.upload", 2270, Void),
            ("http.upload", 2270, Void),
            // Plan 349: File download + resume + progress
            ("auto.http.download", 2271, Void),
            ("http.download", 2271, Void),
            ("auto.http.download_resume", 2272, Void),
            ("http.download_resume", 2272, Void),
            ("auto.http.download_with_progress", 2273, Void),
            ("http.download_with_progress", 2273, Void),
            // Plan 350: WebSocket client
            ("auto.ws.connect", 2280, Void),
            ("ws.connect", 2280, Void),
            ("auto.ws.send", 2281, Void),
            ("ws.send", 2281, Void),
            ("auto.ws.on_message", 2282, Void),
            ("ws.on_message", 2282, Void),
            ("auto.ws.close", 2283, Void),
            ("ws.close", 2283, Void),

            // === Task/Msg (2300-2311) ===
            ("auto.task.spawn", 2300, Void),
            ("auto.task.send", 2301, Void),
            ("auto.task.handle_is_null", 2302, Void),
            ("auto.task.handle_type", 2303, Void),
            ("auto.task.handle_id", 2304, Void),
            ("auto.task.send_await", 2308, Void),
            ("auto.task.ask", 2309, Void),
            ("auto.ctx.reply", 2310, Void),
            ("auto.task.singleton_send", 2311, Void),

            // === TaskSystem (2305-2307) ===
            ("auto.task_system.start", 2305, Void),
            ("auto.task_system.run", 2306, Void),
            ("auto.task_system.stop", 2307, Void),

            // === Regex (2400-2410) ===
            ("auto.regex.is_match", 2400, Void),
            ("auto.regex.find_all", 2401, Void),
            ("auto.regex.match", 2410, Void),

            // === System (2420-2430) ===
            ("auto.sys.exec", 2420, Void),
            ("auto.fs.is_binary", 2430, Void),

            // === Regex opaque (2450-2459) ===
            ("auto.re_opaque.new", 2450, Void),
            ("auto.re_opaque.is_match", 2451, Void),
            ("auto.re_opaque.find", 2452, Void),
            ("auto.re_opaque.find_all", 2453, Void),
            ("auto.re_opaque.replace_all", 2454, Void),
            ("auto.re_opaque.captures", 2455, Void),
            ("auto.re_opaque.drop", 2459, Void),

            // === URL opaque (2500-2511) ===
            ("auto.url_opaque.parse", 2500, Void),
            ("auto.url_opaque.scheme", 2501, Void),
            ("auto.url_opaque.host_str", 2502, Void),
            ("auto.url_opaque.path", 2503, Void),
            ("auto.url_opaque.fragment", 2504, Void),
            ("auto.url_opaque.port", 2505, Void),
            ("auto.url_opaque.query_pairs", 2506, Void),
            ("auto.url_opaque.join", 2507, Void),
            ("auto.url_opaque.origin", 2508, Void),
            ("auto.url_opaque.drop", 2509, Void),
            ("auto.url_opaque.query", 2510, Void),
            ("auto.url_opaque.to_string", 2511, Void),

            // === Semver opaque (2600-2609) ===
            ("auto.semver_opaque.parse", 2600, Void),
            ("auto.semver_opaque.major", 2601, Void),
            ("auto.semver_opaque.minor", 2602, Void),
            ("auto.semver_opaque.patch", 2603, Void),
            ("auto.semver_opaque.pre", 2604, Void),
            ("auto.semver_opaque.to_string", 2605, Void),
            ("auto.semver_opaque.cmp_gt", 2606, Void),
            ("auto.semver_opaque.drop", 2609, Void),

            // === Chrono opaque (2700-2709) ===
            ("auto.chrono_opaque.local_now", 2700, Int),
            ("auto.chrono_opaque.year", 2701, Int),
            ("auto.chrono_opaque.month", 2702, Int),
            ("auto.chrono_opaque.day", 2703, Int),
            ("auto.chrono_opaque.hour", 2704, Int),
            ("auto.chrono_opaque.minute", 2705, Int),
            ("auto.chrono_opaque.second", 2706, Int),
            ("auto.chrono_opaque.timestamp", 2707, I64),
            ("auto.chrono_opaque.format", 2708, String),
            ("auto.chrono_opaque.drop", 2709, Void),

            // === Base64 (2710-2711) ===
            ("auto.base64.encode", 2710, String),
            ("auto.base64.decode", 2711, String),

            // === Hex (2720-2721) ===
            ("auto.hex.encode", 2720, String),
            ("auto.hex.decode", 2721, String),

            // === SHA2 opaque (2730-2739) ===
            ("auto.sha2_opaque.sha256_new", 2730, Void),
            ("auto.sha2_opaque.update", 2731, Void),
            ("auto.sha2_opaque.finalize", 2732, String),
            ("auto.sha2_opaque.drop", 2739, Void),

            // === Mime (2740) ===
            ("auto.mime.from_path", 2740, String),

            // === Test runner (2826-2829) — Plan 263 Phase 2-3 ===
            ("auto.test.run_a2r_dir", 2826, Int),
            ("auto.test.run_vm_dir", 2827, Int),
            ("auto.test.run_a2c_dir", 2828, Int),
            ("auto.test.run_a2ts_dir", 2829, Int),

            // === Rust stdlib dispatch (3000) ===
            ("auto.rust_stdlib.dispatch", 3000, Void),

            // === Bare names (no canonical equivalent) ===
            ("sleep", 1202, Void),
            ("parse_sse", 2250, Void),
            ("str_new", 172, Void),
            ("str_append", 173, Void),
            ("int.str", 174, Void),
            ("uint.to_hex", 236, Void),
            ("alloc_array", 190, Void),
            ("realloc_array", 191, Void),
            ("free_array", 192, Void),

            // === ID-conflicting short names ===
            ("str.len", 170, Void),
            ("String.len", 171, Void),
            ("str.upper", 175, Void),
            ("String.from", 176, Void),
            ("String.is_empty", 185, Void),

            // === FFI shim name aliases (#[rust_fn]) ===
            ("File.read_text", 1000, Void),
            ("File.write_text", 1001, Void),
            ("File.exists", 1002, Void),
            ("File.delete", 1003, Void),
            ("File.create_dir", 1004, Void),
            ("File.read_bytes", 1005, Void),
            ("File.write_bytes", 1006, Void),
            ("File.copy", 1007, Void),
            ("File.size", 1008, Void),
            ("File.is_dir", 1009, Void),
            ("File.append_text", 1011, Void),
            ("File.remove_dir", 1014, Void),
            ("File.remove_dir_all", 1015, Void),

            // === Str.* aliases (#[rust_fn]) ===
            ("Str.len", 1500, Void),
            ("Str.is_empty", 1501, Void),
            ("Str.char_at", 1502, Void),
            ("Str.substr", 1503, Void),
            ("Str.contains", 1504, Void),
            ("Str.starts_with", 1505, Void),
            ("Str.ends_with", 1506, Void),
            ("Str.trim", 1507, Void),
            ("Str.split", 1508, Void),
            ("Str.repeat", 1509, Void),
            ("Str.replace", 1510, Void),
            ("Str.to_upper", 1511, Void),
            ("Str.to_lower", 1512, Void),
            ("Str.reverse", 1513, Void),
            ("Str.find", 1514, Void),
            ("Str.lines", 1515, Void),
            ("Str.parse_int", 1516, Void),
            ("Str.parse_float", 1517, Void),
            ("Str.split_once", 1518, Void),
            ("Str.match_count", 1519, Void),
            ("Str.replace_first", 1520, Void),
            ("Str.uuid", 1521, String),
            ("Str.from_uint", 1522, String),
            ("Str.to_uint", 1523, I64),

            // === Runtime aliases (CALL_SPEC lowercase type) ===
            ("str.len", 1500, Void),
            ("str.contains", 1504, Void),
            ("str.starts_with", 1505, Void),
            ("str.ends_with", 1506, Void),
            ("str.trim", 1507, Void),
            ("str.split", 1508, Void),
            ("str.repeat", 1509, Void),
            ("str.replace", 1510, Void),
            ("str.to_upper", 1511, Void),
            ("str.to_lower", 1512, Void),
            ("str.find", 1514, Void),
            ("str.lines", 1515, Void),
            ("str.parse_int", 1516, Void),
            ("str.parse_float", 1517, Void),
            ("str.is_empty", 1501, Void),
            ("str.substr", 1503, Void),

            // === Task.* aliases ===
            ("Task.spawn", 2300, Void),
            ("TaskHandle.send", 2301, Void),
            ("Task.singleton_send", 2311, Void),
            ("TaskHandle.send_await", 2308, Void),
            ("TaskHandle.ask", 2309, Void),
            ("TaskHandle.is_null", 2302, Void),
            ("TaskHandle.task_type", 2303, Void),
            ("TaskHandle.instance_id", 2304, Void),
            ("TaskSystem.start", 2305, Void),
            ("TaskSystem.stop", 2307, Void),

            // === Option/Result ===
            ("auto.option.or", 1550, Void),
            ("auto.option.unwrap_or", 1551, Void),
            ("auto.result.map_err", 2070, Void),
            ("auto.result.Ok.map_err", 2070, Void),
            ("auto.result.Err.map_err", 2070, Void),

            // === str.* typed entries ===
            ("str.split_once", 1518, List),
            ("str.match_count", 1519, Int),
            ("str.replace_first", 1520, String),
            ("str.uuid", 1521, String),
            ("str.from_uint", 1522, String),
            ("str.to_uint", 1523, I64)
        }
    };
}

/// Consumer: generates registration calls for each BIGVM entry.
/// Must be called inside a method that has `registry` in scope (use local macro wrapper).
/// Entry format: ("name", id, ret_type_tag).
/// Void → register_with_id; others → register_with_id_and_type.
#[macro_export]
macro_rules! register_bigvm {
    (($name:expr, $id:expr, Void) $(, $rest:tt)*) => {
        registry.register_with_id($name, $id);
        $crate::register_bigvm!($($rest),*);
    };
    (($name:expr, $id:expr, List) $(, $rest:tt)*) => {
        registry.register_with_id_and_type($name, $id, NativeRetType::List);
        $crate::register_bigvm!($($rest),*);
    };
    (($name:expr, $id:expr, Bool) $(, $rest:tt)*) => {
        registry.register_with_id_and_type($name, $id, NativeRetType::Bool);
        $crate::register_bigvm!($($rest),*);
    };
    (($name:expr, $id:expr, Int) $(, $rest:tt)*) => {
        registry.register_with_id_and_type($name, $id, NativeRetType::Int);
        $crate::register_bigvm!($($rest),*);
    };
    (($name:expr, $id:expr, I64) $(, $rest:tt)*) => {
        registry.register_with_id_and_type($name, $id, NativeRetType::I64);
        $crate::register_bigvm!($($rest),*);
    };
    (($name:expr, $id:expr, String) $(, $rest:tt)*) => {
        registry.register_with_id_and_type($name, $id, NativeRetType::String);
        $crate::register_bigvm!($($rest),*);
    };
    (($name:expr, $id:expr, Float) $(, $rest:tt)*) => {
        registry.register_with_id_and_type($name, $id, NativeRetType::Float);
        $crate::register_bigvm!($($rest),*);
    };
    (($name:expr, $id:expr, Map) $(, $rest:tt)*) => {
        registry.register_with_id_and_type($name, $id, NativeRetType::Map);
        $crate::register_bigvm!($($rest),*);
    };
    () => {};
}
// Plan 250: Known native function names → fixed IDs.
// Used by resolve_qualified() for lazy registration.
// IDs must match NATIVE_* constants in for_each_native! (shim bindings).
pub const NATIVE_ID_ENTRIES: &[(&str, u16)] = &[
    ("auto.list.new", 100),
    ("auto.list.push", 101),
    ("auto.list.pop", 102),
    ("auto.list.len", 103),
    ("auto.list.is_empty", 104),
    ("auto.list.clear", 105),
    ("auto.list.get", 106),
    ("auto.list.set", 107),
    ("auto.list.insert", 108),
    ("auto.list.remove", 109),
    ("auto.list.drop", 110),
    ("auto.list.reserve", 118),
    ("auto.list.capacity", 205),
    ("auto.list.map", 2060),
    ("auto.list.filter", 2061),
    ("auto.list.for_each", 2062),
    ("auto.list.find", 2063),
    ("auto.list.any", 2064),
    ("auto.list.all", 2065),
    ("auto.list.reduce", 2066),
    ("auto.list.sort", 2067),
    ("auto.list.sort_by", 2068),
    ("auto.list.contains", 2069),
    ("auto.list.iter", 111),
    ("auto.iterator.next", 112),
    ("auto.iterator.map", 113),
    ("auto.iterator.filter", 114),
    ("auto.iterator.collect", 115),
    ("auto.iterator.reduce", 116),
    ("auto.iterator.find", 117),
    ("auto.iterator.enumerate", 119),
    ("auto.hashmap.new", 120),
    ("Map.new", 120),
    ("HashMap.new", 120),
    ("auto.hashmap.insert_str", 121),
    ("auto.hashmap.insert_int", 122),
    ("auto.hashmap.get_str", 123),
    ("auto.hashmap.get_int", 124),
    ("auto.hashmap.contains", 125),
    ("auto.hashmap.remove", 126),
    ("auto.hashmap.size", 127),
    ("auto.hashmap.clear", 128),
    ("auto.hashmap.drop", 129),
    ("auto.hashmap.insert", 121),
    ("auto.hashmap.get", 123),
    ("auto.hashmap.keys", 1292),
    ("auto.hashset.new", 129),
    ("auto.hashset.insert", 130),
    ("auto.hashset.contains", 131),
    ("auto.hashset.remove", 132),
    ("auto.hashset.size", 133),
    ("auto.hashset.clear", 134),
    ("auto.hashset.drop", 135),
    ("auto.vecdeque.new", 136),
    ("auto.vecdeque.push_back", 137),
    ("auto.vecdeque.push_front", 138),
    ("auto.vecdeque.pop_back", 139),
    ("auto.vecdeque.pop_front", 140),
    ("auto.vecdeque.front", 141),
    ("auto.vecdeque.back", 142),
    ("auto.vecdeque.size", 143),
    ("auto.vecdeque.is_empty", 144),
    ("auto.vecdeque.clear", 145),
    ("auto.vecdeque.drop", 146),
    ("auto.btreemap.new", 147),
    ("auto.btreemap.insert", 148),
    ("auto.btreemap.get", 149),
    ("auto.btreemap.contains", 150),
    ("auto.btreemap.remove", 151),
    ("auto.btreemap.size", 152),
    ("auto.btreemap.is_empty", 153),
    ("auto.btreemap.clear", 154),
    ("auto.btreemap.first_key", 155),
    ("auto.btreemap.last_key", 156),
    ("auto.btreemap.drop", 157),
    ("auto.stringbuilder.new", 160),
    ("auto.stringbuilder.append", 161),
    ("auto.stringbuilder.append_int", 162),
    ("auto.stringbuilder.append_char", 163),
    ("auto.stringbuilder.len", 164),
    ("auto.stringbuilder.clear", 165),
    ("auto.stringbuilder.drop", 166),
    ("auto.stringbuilder.build", 167),
    ("auto.heap.new", 195),
    ("auto.heap.capacity", 196),
    ("auto.heap.try_grow", 197),
    ("auto.heap.drop", 198),
    ("auto.inline_int64.new", 199),
    ("auto.inline_int64.capacity", 200),
    ("auto.inline_int64.try_grow", 201),
    ("auto.inline_int64.drop", 202),
    ("auto.alloc.array", 190),
    ("auto.realloc.array", 191),
    ("auto.free.array", 192),
    ("auto.str.len", 1500),
    ("auto.str.is_empty", 1501),
    ("auto.str.char_at", 1502),
    ("auto.str.substr", 1503),
    ("auto.str.sub", 1503),
    ("auto.str.slice", 1503),
    ("auto.str.contains", 1504),
    ("auto.str.starts_with", 1505),
    ("auto.str.ends_with", 1506),
    ("auto.str.trim", 1507),
    ("auto.str.split", 1508),
    ("auto.str.repeat", 1509),
    ("auto.str.replace", 1510),
    ("auto.str.to_upper", 1511),
    ("auto.str.to_lower", 1512),
    ("auto.str.upper", 175),
    ("auto.str.lower", 1512),
    ("auto.str.reverse", 1513),
    ("auto.str.find", 1514),
    ("auto.str.lines", 1515),
    ("auto.str.parse_int", 1516),
    ("auto.str.to_int", 1516),
    ("auto.str.parse_float", 1517),
    ("auto.str.new", 177),
    ("auto.str.push", 178),
    ("auto.str.pop", 179),
    ("auto.str.get", 180),
    ("auto.str.set", 181),
    ("auto.str.insert", 182),
    ("auto.str.remove", 183),
    ("auto.str.clear", 184),
    ("auto.str.reserve", 186),
    ("auto.str.bytes", 235),
    ("auto.int.and", 210),
    ("auto.int.or", 211),
    ("auto.int.xor", 212),
    ("auto.int.not", 213),
    ("auto.int.shl", 214),
    ("auto.int.shr", 215),
    ("auto.int.sar", 216),
    ("auto.int.rol", 217),
    ("auto.int.ror", 218),
    ("auto.int.count_ones", 220),
    ("auto.int.leading_zeros", 221),
    ("auto.int.trailing_zeros", 222),
    ("auto.int.flip", 223),
    ("auto.int.bit_read", 230),
    ("auto.int.bit_test", 231),
    ("auto.int.bit_on", 232),
    ("auto.int.bit_off", 233),
    ("auto.int.bit_flip", 234),
    ("auto.file.read_text", 1000),
    ("auto.file.write_text", 1001),
    ("auto.file.exists", 1002),
    ("auto.file.delete", 1003),
    ("auto.file.create_dir", 1004),
    ("auto.file.read_bytes", 1005),
    ("auto.file.write_bytes", 1006),
    ("auto.file.copy", 1007),
    ("auto.file.size", 1008),
    ("auto.file.is_dir", 1009),
    ("auto.file.walk", 1010),
    ("auto.file.append_text", 1011),
    ("auto.file.read_lines", 1012),
    ("auto.file.remove_dir", 1014),
    ("auto.file.remove_dir_all", 1015),
    ("auto.fs.read_text", 1000),
    ("auto.fs.read", 1000),
    ("auto.fs.write_text", 1001),
    ("auto.fs.write", 1001),
    ("auto.fs.append_text", 1011),
    ("auto.fs.append", 1011),
    ("auto.fs.exists", 1002),
    ("auto.fs.delete", 1003),
    ("auto.fs.create_dir", 1004),
    ("auto.fs.remove_dir", 1014),
    ("auto.fs.remove_dir_all", 1015),
    ("auto.fs.read_bytes", 1005),
    ("auto.fs.write_bytes", 1006),
    ("auto.fs.copy", 1007),
    ("auto.fs.size", 1008),
    ("auto.fs.is_dir", 1009),
    ("auto.file.create_handle", 1010),
    ("auto.file.open_handle", 1011),
    ("auto.file.write_handle", 1012),
    ("auto.file.try_clone", 1013),
    ("auto.env.get", 1100),
    ("auto.env.set", 1101),
    ("auto.env.remove", 1102),
    ("auto.env.get_or", 1103),
    ("Env.get_or", 1103),
    ("auto.env.local_data_dir", 1104),
    ("env.local_data_dir", 1104),
    ("auto.env.home_dir", 1105),
    ("env.home_dir", 1105),
    ("auto.time.now_ms", 1200),
    ("auto.time.now_sec", 1201),
    ("auto.time.sleep_ms", 1202),
    ("auto.time.instant_now", 1203),
    ("auto.time.instant_elapsed", 1204),
    ("auto.cell.once_new", 2850),
    ("auto.cell.once_set", 2851),
    ("auto.cell.once_get", 2852),
    ("auto.process.exit", 1300),
    ("auto.process.args", 1301),
    ("auto.process.current_dir", 1302),
    ("auto.process.set_current_dir", 1303),
    ("auto.process.spawn", 1304),
    ("auto.process.spawn_with_output", 1305),
    ("auto.path.join", 1400),
    ("auto.path.parent", 1401),
    ("auto.path.extension", 1402),
    ("auto.path.filename", 1403),
    ("auto.path.canonicalize", 1404),
    ("auto.char.is_alpha", 1600),
    ("auto.char.is_digit", 1601),
    ("auto.char.is_alphanum", 1602),
    ("auto.char.is_whitespace", 1603),
    ("auto.char.is_ident", 1604),
    ("auto.char.to_lower", 1605),
    ("auto.char.to_upper", 1606),
    ("auto.log.debug", 1800),
    ("auto.log.info", 1801),
    ("auto.log.warn", 1802),
    ("auto.log.error", 1803),
    ("auto.log.noop", 1804),
    ("Log.debug", 1800),
    ("Log.info", 1801),
    ("Log.warn", 1802),
    ("Log.error", 1803),
    ("auto.math.abs", 1700),
    ("auto.math.min", 1701),
    ("auto.math.max", 1702),
    ("auto.math.sqrt", 1750),
    ("auto.math.floor", 1710),
    ("auto.math.ceil", 1711),
    ("auto.math.round", 1712),
    ("auto.math.pow", 1713),
    ("auto.math.min_f", 1714),
    ("auto.math.max_f", 1715),
    ("auto.math.sin", 1716),
    ("auto.math.cos", 1717),
    ("auto.math.tan", 1718),
    ("auto.math.exp", 1719),
    ("auto.math.ln", 1720),
    ("auto.math.log2", 1721),
    ("auto.math.log10", 1722),
    ("auto.math.abs_f", 1723),
    ("auto.math.signum", 1724),
    ("auto.math.clamp", 1725),
    ("auto.math.asin", 1726),
    ("auto.math.acos", 1727),
    ("auto.math.atan", 1728),
    ("auto.math.atan2", 1729),
    ("auto.math.powi", 1730),
    ("auto.math.powf", 1731),
    ("auto.math.to_radians", 1732),
    ("auto.math.to_degrees", 1733),
    ("auto.rand.thread_rng", 1850),
    ("auto.rng.gen_range", 1851),
    ("auto.rng.gen", 1852),
    ("auto.rng.drop", 1853),
    ("auto.rand.random", 1854),
    ("auto.json.encode", 1900),
    ("auto.json.decode", 1901),
    ("auto.json.parse", 1902),
    ("auto.json.prettify", 1903),
    ("auto.json.minify", 1904),
    ("auto.json.is_valid", 1905),
    ("auto.json.get", 1906),
    ("auto.json.get_at", 1907),
    ("auto.json.len", 1908),
    ("auto.json.type_of", 1909),
    ("auto.json.as_string", 1910),
    ("auto.json.as_number", 1911),
    ("auto.json.as_int", 1912),
    ("auto.json.as_bool", 1913),
    ("auto.json.is_null", 1914),
    ("auto.json.keys", 1915),
    ("auto.json.has_key", 1917),
    // Plan 340: JSON ↔ VM value + JSON HTTP helpers
    ("auto.json.to_value", 3100),
    ("auto.json.from_value", 3101),
    ("Json.to_value", 3100),
    ("Json.from_value", 3101),
    ("auto.http.get_json", 3102),
    ("auto.http.post_json", 3103),
    ("auto.http.put_json", 3104),
    ("auto.http.delete_json", 3105),
    // Plan 341: 异步 SSE 流式接收
    ("auto.http.sse_get_stream", 3106),
    ("http.sse_get_stream", 3106),
    ("http.sse_stream", 3106),
    ("auto.serde_json.to_string", 1900),
    ("auto.json.to_string", 1900),
    ("json.to_string", 1900),
    ("Json.to_string", 1900),
    ("auto.serde_json.from_str", 1902),
    ("auto.toml.from_str", 2750),
    ("auto.toml.to_string", 2751),
    ("auto.url.encode", 2000),
    ("auto.url.decode", 2001),
    ("auto.url.encode_query", 2002),
    ("auto.url.decode_query", 2003),
    ("auto.url.parse", 2006),
    ("auto.url.scheme", 2007),
    ("auto.url.host", 2008),
    ("auto.url.port", 2009),
    ("auto.url.path", 2010),
    ("auto.url.query", 2011),
    ("auto.url.fragment", 2012),
    ("auto.url.join_path", 2015),
    ("auto.net.tcp_bind", 2100),
    ("auto.net.tcp_listener_accept", 2101),
    ("auto.net.tcp_listener_local_addr", 2102),
    ("auto.net.tcp_listener_close", 2103),
    ("auto.net.tcp_connect", 2104),
    ("auto.net.tcp_stream_read", 2105),
    ("auto.net.tcp_stream_write", 2106),
    ("auto.net.tcp_stream_read_all", 2107),
    ("auto.net.tcp_stream_read_line", 2108),
    ("auto.net.tcp_stream_write_str", 2109),
    ("auto.net.tcp_stream_close", 2110),
    ("auto.net.tcp_stream_peer_addr", 2111),
    ("auto.net.tcp_stream_set_read_timeout", 2112),
    ("auto.net.tcp_stream_set_write_timeout", 2113),
    ("auto.net.tcp_stream_flush", 2114),
    ("auto.net.tcp_stream_set_nodelay", 2115),
    ("auto.http.server", 2200),
    ("auto.http.server_get", 2201),
    ("auto.http.server_post", 2202),
    ("auto.http.server_put", 2203),
    ("auto.http.server_delete", 2204),
    ("auto.http.server_static", 2205),
    ("auto.http.server_listen", 2206),
    ("auto.http.response", 2210),
    ("auto.http.response_status", 2211),
    ("auto.http.response_header", 2212),
    ("auto.http.response_text", 2213),
    ("auto.http.response_html", 2214),
    ("auto.http.response_bytes", 2215),
    ("auto.http.response.redirect", 2219),
    ("http.response.redirect", 2219),
    ("auto.session.create", 2284),
    ("session.create", 2284),
    ("auto.session.get", 2285),
    ("session.get", 2285),
    ("auto.session.set", 2286),
    ("session.set", 2286),
    ("auto.session.destroy", 2287),
    ("session.destroy", 2287),
    ("auto.http.server_use", 2288),
    ("http.server.use", 2288),
    ("auto.template.compile", 2289),
    ("template.compile", 2289),
    ("auto.template.render", 2290),
    ("template.render", 2290),
    ("auto.openapi.generate", 2291),
    ("openapi.generate", 2291),
    ("auto.http.response.status_code", 2216),
    ("auto.http.response.header_get", 2217),
    ("auto.http.response.body", 2218),
    ("auto.http.ok", 2220),
    ("auto.http.created", 2221),
    ("auto.http.bad_request", 2222),
    ("auto.http.not_found", 2223),
    ("auto.http.internal_error", 2224),
    ("auto.http.get", 2230),
    ("auto.http.post", 2231),
    ("auto.http.put", 2232),
    ("auto.http.delete", 2233),
    ("auto.http.request", 2234),
    ("auto.http.request_builder_header", 2235),
    ("auto.http.request_builder_body", 2236),
    ("auto.http.request_builder_timeout", 2237),
    ("auto.http.request_builder_json", 2238),
    ("auto.http.request_builder_send", 2239),
    ("auto.http_stream.get_stream", 2240),
    ("auto.http_stream.post_stream", 2241),
    ("auto.http_stream.stream_next", 2242),
    ("auto.http_stream.stream_is_done", 2243),
    ("auto.http_stream.stream_close", 2244),
    ("auto.http_stream.stream_iter", 2245),
    ("auto.http.post_stream_with_headers", 2255),
    ("auto.http.post_sync", 2256),
    ("auto.http.last_status", 2257),
    ("auto.http.post_bearer", 2258),
    ("auto.http.listen", 2259),
    ("RequestBuilder.header", 2260),
    ("RequestBuilder.body", 2261),
    ("RequestBuilder.timeout", 2262),
    ("RequestBuilder.json", 2263),
    ("RequestBuilder.send", 2264),
    ("RequestBuilder.tls_ca_cert", 2265),
    ("RequestBuilder.tls_skip_verify", 2266),
    ("RequestBuilder.tls_client_cert", 2267),
    ("RequestBuilder.multipart_file", 2268),
    ("RequestBuilder.multipart_text", 2269),
    ("RequestBuilder.cookie_store", 2272),
    ("RequestBuilder.retry", 2273),
    ("RequestBuilder.gzip", 2274),
    ("auto.http.upload", 2270),
    ("http.upload", 2270),
    ("auto.http.download", 2271),
    ("http.download", 2271),
    ("auto.http.download_resume", 2272),
    ("http.download_resume", 2272),
    ("auto.http.download_with_progress", 2273),
    ("http.download_with_progress", 2273),
    ("auto.ws.connect", 2280),
    ("ws.connect", 2280),
    ("auto.ws.send", 2281),
    ("ws.send", 2281),
    ("auto.ws.on_message", 2282),
    ("ws.on_message", 2282),
    ("auto.ws.close", 2283),
    ("ws.close", 2283),
    ("auto.task.spawn", 2300),
    ("auto.task.send", 2301),
    ("auto.task.handle_is_null", 2302),
    ("auto.task.handle_type", 2303),
    ("auto.task.handle_id", 2304),
    ("auto.task.send_await", 2308),
    ("auto.task.ask", 2309),
    ("auto.ctx.reply", 2310),
    ("auto.task.singleton_send", 2311),
    ("auto.task_system.start", 2305),
    ("auto.task_system.run", 2306),
    ("auto.task_system.stop", 2307),
    ("auto.regex.is_match", 2400),
    ("auto.regex.find_all", 2401),
    ("auto.regex.match", 2410),
    ("auto.sys.exec", 2420),
    ("auto.fs.is_binary", 2430),
    ("auto.re_opaque.new", 2450),
    ("auto.re_opaque.is_match", 2451),
    ("auto.re_opaque.find", 2452),
    ("auto.re_opaque.find_all", 2453),
    ("auto.re_opaque.replace_all", 2454),
    ("auto.re_opaque.captures", 2455),
    ("auto.re_opaque.drop", 2459),
    ("auto.url_opaque.parse", 2500),
    ("auto.url_opaque.scheme", 2501),
    ("auto.url_opaque.host_str", 2502),
    ("auto.url_opaque.path", 2503),
    ("auto.url_opaque.fragment", 2504),
    ("auto.url_opaque.port", 2505),
    ("auto.url_opaque.query_pairs", 2506),
    ("auto.url_opaque.join", 2507),
    ("auto.url_opaque.origin", 2508),
    ("auto.url_opaque.drop", 2509),
    ("auto.url_opaque.query", 2510),
    ("auto.url_opaque.to_string", 2511),
    ("auto.semver_opaque.parse", 2600),
    ("auto.semver_opaque.major", 2601),
    ("auto.semver_opaque.minor", 2602),
    ("auto.semver_opaque.patch", 2603),
    ("auto.semver_opaque.pre", 2604),
    ("auto.semver_opaque.to_string", 2605),
    ("auto.semver_opaque.cmp_gt", 2606),
    ("auto.semver_opaque.drop", 2609),
    ("auto.semver_opaque_versionreq.parse", 2610),
    ("auto.semver_opaque_versionreq.matches", 2611),
    ("auto.chrono_opaque.local_now", 2700),
    ("auto.chrono_opaque.year", 2701),
    ("auto.chrono_opaque.month", 2702),
    ("auto.chrono_opaque.day", 2703),
    ("auto.chrono_opaque.hour", 2704),
    ("auto.chrono_opaque.minute", 2705),
    ("auto.chrono_opaque.second", 2706),
    ("auto.chrono_opaque.timestamp", 2707),
    ("auto.chrono_opaque.format", 2708),
    ("auto.chrono_opaque.drop", 2709),
    ("auto.base64.encode", 2710),
    ("auto.base64.decode", 2711),
    ("auto.hex.encode", 2720),
    ("auto.hex.decode", 2721),
    ("auto.sha2_opaque.sha256_new", 2730),
    ("auto.sha2_opaque.update", 2731),
    ("auto.sha2_opaque.finalize", 2732),
    ("auto.sha2_opaque.drop", 2739),
    ("auto.mime.from_path", 2740),
    ("auto.test.run_a2r_dir", 2826),
    ("auto.test.run_vm_dir", 2827),
    ("auto.test.run_a2c_dir", 2828),
    ("auto.test.run_a2ts_dir", 2829),
    ("auto.rust_stdlib.dispatch", 3000),
    ("sleep", 1202),
    ("parse_sse", 2250),
    ("str_new", 172),
    ("str_append", 173),
    ("int.str", 174),
    ("uint.to_hex", 236),
    ("alloc_array", 190),
    ("realloc_array", 191),
    ("free_array", 192),
    ("str.len", 170),
    ("String.len", 171),
    ("str.upper", 175),
    ("String.from", 176),
    ("String.is_empty", 185),
    ("File.read_text", 1000),
    ("File.write_text", 1001),
    ("File.exists", 1002),
    ("File.delete", 1003),
    ("File.create_dir", 1004),
    ("File.read_bytes", 1005),
    ("File.write_bytes", 1006),
    ("File.copy", 1007),
    ("File.size", 1008),
    ("File.is_dir", 1009),
    ("File.append_text", 1011),
    ("File.remove_dir", 1014),
    ("File.remove_dir_all", 1015),
    ("Str.len", 1500),
    ("Str.is_empty", 1501),
    ("Str.char_at", 1502),
    ("Str.substr", 1503),
    ("Str.contains", 1504),
    ("Str.starts_with", 1505),
    ("Str.ends_with", 1506),
    ("Str.trim", 1507),
    ("Str.split", 1508),
    ("Str.repeat", 1509),
    ("Str.replace", 1510),
    ("Str.to_upper", 1511),
    ("Str.to_lower", 1512),
    ("Str.reverse", 1513),
    ("Str.find", 1514),
    ("Str.lines", 1515),
    ("Str.parse_int", 1516),
    ("Str.parse_float", 1517),
    ("Str.split_once", 1518),
    ("Str.match_count", 1519),
    ("Str.replace_first", 1520),
    ("Str.uuid", 1521),
    ("Str.from_uint", 1522),
    ("Str.to_uint", 1523),
    ("str.len", 1500),
    ("str.contains", 1504),
    ("str.starts_with", 1505),
    ("str.ends_with", 1506),
    ("str.trim", 1507),
    ("str.split", 1508),
    ("str.repeat", 1509),
    ("str.replace", 1510),
    ("str.to_upper", 1511),
    ("str.to_lower", 1512),
    ("str.find", 1514),
    ("str.lines", 1515),
    ("str.parse_int", 1516),
    ("str.parse_float", 1517),
    ("str.is_empty", 1501),
    ("str.substr", 1503),
    ("Task.spawn", 2300),
    ("TaskHandle.send", 2301),
    ("Task.singleton_send", 2311),
    ("TaskHandle.send_await", 2308),
    ("TaskHandle.ask", 2309),
    ("TaskHandle.is_null", 2302),
    ("TaskHandle.task_type", 2303),
    ("TaskHandle.instance_id", 2304),
    ("TaskSystem.start", 2305),
    ("TaskSystem.stop", 2307),
    ("auto.option.or", 1550),
    ("auto.option.unwrap_or", 1551),
    ("auto.result.map_err", 2070),
    ("auto.result.Ok.map_err", 2070),
    ("auto.result.Err.map_err", 2070),
    ("str.split_once", 1518),
    ("str.match_count", 1519),
    ("str.replace_first", 1520),
    ("str.uuid", 1521),
    ("str.from_uint", 1522),
    ("str.to_uint", 1523),

    // === Time (1200-1205) ===
    ("auto.time.now", 1205),
    ("Time.now", 1205),

    // === IO (1150) ===
    ("auto.io.read_line", 1150),
    ("IO.read_line", 1150),
    ("io.read_line", 1150),

    // === FS extended (2860-2865) ===
    ("auto.fs.walk", 2860),
    ("auto.fs.metadata", 2861),
    ("auto.fs.copy_recursive", 2862),
    ("auto.fs.filename", 2863),
    ("auto.fs.parent", 2864),
    ("auto.fs.join", 2865),

    // === Hash extended (2814-2816) ===
    ("auto.hash.hmac_sha256", 2814),
    ("auto.hash.file_md5", 2815),
    ("auto.hash.file_sha256", 2816),

    // === Random type (2870-2874) ===
    ("auto.random._vm_new", 2870),
    ("auto.random._vm_seeded", 2871),
    ("auto.random.int", 2872),
    ("auto.random.float", 2873),
    ("auto.random.bool", 2874),

    // === Fmt (2752) ===
    ("auto.fmt.f64_debug", 2752),

    // === Cmp (2880) ===
    ("auto.cmp.str_cmp", 2880),

    // === DateTime cmp (2794) ===
    ("auto.datetime.cmp", 2794),
];
