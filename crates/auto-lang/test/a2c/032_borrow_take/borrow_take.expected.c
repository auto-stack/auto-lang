#include "borrow_take.h"

int main(void) {
    unknown s1 = str_new("hello", 10);
    unknown s2 = s1;
    printf("%d\n", str_len(s2));

    return 0;
}
