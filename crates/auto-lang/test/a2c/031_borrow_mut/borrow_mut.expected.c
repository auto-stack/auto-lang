#include "borrow_mut.h"

int main(void) {
    unknown s = str_new("hello", 10);
    unknown mut_ref = s;
    str_append(mut_ref, " world");
    printf("%d\n", s);
    return 0;
}
