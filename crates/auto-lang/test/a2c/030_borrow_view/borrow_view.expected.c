#include "borrow_view.h"

int main(void) {
    char* s = "hello";
    unknown slice = s;
    printf("%d\n", str_len(slice));
    printf("%s\n", s);
    return 0;
}
