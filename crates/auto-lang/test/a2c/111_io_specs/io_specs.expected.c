#include "io_specs.h"


str MyReader_Read(struct MyReader *self) {
    return self->data;
}
Reader_vtable MyReader_Reader_vtable = {
    .read = MyReader_Read
};


int main(void) {
    struct MyReader reader = {.data = "Hello, spec!"};


    unknown readers[0] = {reader};
    for (int i = 0; i < 0; i++) {
        unknown r = readers[i];
        unknown text = r.read();
        printf("%d\n", text);
    }

    printf("%s\n", "Test passed!");
    return 0;
}
