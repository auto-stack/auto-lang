#include "io_specs.h"


char* MyReader_Read(struct MyReader *self) {
    return self->data;
}

int main(void) {
    unknown reader = MyReader("Hello, spec!");


    void* readers[1] = {reader};
    for () {
        unknown text = r.read();
        printf("%d\n", text);
    }

    printf("%s\n", "Test passed!");
    return 0;
}
