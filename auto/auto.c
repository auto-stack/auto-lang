#include "auto.h"

int main(void) {
    printf("%s\n", "Hello, world!");

    char* code = "print(\"Hello, world!\")";
    struct Src src = {.content = code, .len = 22, .pos = {.line = 0, .at = 0, .total = 0}};
    while (1) {
        char ch = Src_NextChar(&src);
        if (ch == -1) {
            break;
        }
        printf("%c\n", ch);
    }
    return 0;
}
