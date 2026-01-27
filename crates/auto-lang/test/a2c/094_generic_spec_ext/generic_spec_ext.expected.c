#include "generic_spec_ext.h"


int* Heap_Get(struct Heap *self) {
}
Storage_vtable Heap_Storage_vtable = {
    .get = Heap_Get
};

Storage_int_vtable Heap_Storage_int_vtable = {
    .get = Heap_Get
};


int main(void) {
    struct Heap h = {};
    unknown p = Heap_Get(&h);
    return 0;
}
