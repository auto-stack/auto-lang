#include "ext_builtin_type.at.h"

int str_len(char* self) {
    return self->size;
}

int main(void) {
    char* s = "hello";
    unknown length = str_len(s);
    printf("%d\n", length);
    return 0;
}
