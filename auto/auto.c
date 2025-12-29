#include "auto.h"

int main(void) {
    printf("%s\n", "Hello, world!");

    char* code = "print(\"Hello, world!\")";
    struct Src src = {.content = code, .len = 22, .pos = {.line = 0, .lpos = 0, .spos = 0}};
    while (1) {
        char ch = next_char(&src);
        if (ch == -1) {
            break;
        }
        printf("%c\n", ch);
    }
    return 0;
}
