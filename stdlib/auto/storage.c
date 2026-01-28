#include "storage.at.h"


Storage_vtable Heap_Storage_vtable = {
    .data = Heap_Data
    .capacity = Heap_Capacity
    .try_grow = Heap_TryGrow
};

Storage_void__vtable Heap_Storage_void__vtable = {
    .data = Heap_Data
    .capacity = Heap_Capacity
    .try_grow = Heap_TryGrow
};

