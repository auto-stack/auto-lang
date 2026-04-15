#include "borrow_conflicts.h"

int main(void) {
    char* s = "hello";

    char* v1 = &(s);
    char* v2 = &(s);

    printf("%s\n", v1);
    printf("%s\n", v2);
    return 0;
}
