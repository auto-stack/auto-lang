#include "borrow_conflicts.h"

int main(void) {
    char* s = "hello";

    unknown v1 = s;
    unknown v2 = s;

    printf("%d\n", v1);
    printf("%d\n", v2);
    return 0;
}
