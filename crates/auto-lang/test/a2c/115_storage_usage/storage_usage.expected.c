#include "storage_usage.h"


struct Heap Heap_New(struct Heap *self) {
}
void* Heap_Data(struct Heap *self) {
}
unsigned int Heap_Capacity(struct Heap *self) {
}
bool Heap_TryGrow(struct Heap *self, unsigned int) {
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
unsigned int InlineInt64_Capacity(struct InlineInt64 *self) {
}
bool InlineInt64_TryGrow(struct InlineInt64 *self, unsigned int) {
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

    struct Heap heap = Heap_New();
    void* heap_data = Heap_Data(&heap);
    unsigned int heap_cap = Heap_Capacity(&heap);
    bool can_grow_heap = Heap_TryGrow(&heap, 100);


    struct InlineInt64 inline = InlineInt64_New();
    int* inline_data = InlineInt64_Data(&inline);
    unsigned int inline_cap = InlineInt64_Capacity(&inline);
    bool can_grow_inline = InlineInt64_TryGrow(&inline, 50);

    return 0;
}
