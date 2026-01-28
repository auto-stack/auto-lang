#include "storage_usage.h"


struct Heap Heap_New(struct Heap *self) {
}
void* Heap_Data(struct Heap *self) {
}
void Heap_Capacity(struct Heap *self) {
}
bool Heap_TryGrow(struct Heap *self, unknown) {
}
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


struct InlineInt64 InlineInt64_New(struct InlineInt64 *self) {
}
int* InlineInt64_Data(struct InlineInt64 *self) {
}
void InlineInt64_Capacity(struct InlineInt64 *self) {
}
bool InlineInt64_TryGrow(struct InlineInt64 *self, unknown) {
}
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


int main(void) {

    unknown heap = Heap.new();
    unknown heap_data = heap.data();
    unknown heap_cap = heap.capacity();
    unknown can_grow_heap = heap.try_grow(100);


    unknown inline = InlineInt64.new();
    unknown inline_data = inline.data();
    unknown inline_cap = inline.capacity();
    unknown can_grow_inline = inline.try_grow(50);

    
    return 0;
}
