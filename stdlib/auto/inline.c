#include "inline.at.h"


Storage_vtable InlineInt64_Storage_vtable = {
    .data = InlineInt64_Data
    .capacity = InlineInt64_Capacity
    .try_grow = InlineInt64_TryGrow
};

Storage_int_vtable InlineInt64_Storage_int_vtable = {
    .data = InlineInt64_Data
    .capacity = InlineInt64_Capacity
    .try_grow = InlineInt64_TryGrow
};


Storage_vtable InlineU8_256_Storage_vtable = {
    .data = InlineU8_256_Data
    .capacity = InlineU8_256_Capacity
    .try_grow = InlineU8_256_TryGrow
};

Storage_unknown_vtable InlineU8_256_Storage_unknown_vtable = {
    .data = InlineU8_256_Data
    .capacity = InlineU8_256_Capacity
    .try_grow = InlineU8_256_TryGrow
};

