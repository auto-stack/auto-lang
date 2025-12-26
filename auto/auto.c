#include "auto.h"

int main(void) {
    printf("%s\n", "Hello, world!");

    char* code = "print('Hello, world!')";
    struct Src src = {.content = code, .pos = {.line = 0, .lpos = 0, .spos = 0}};
    char ch = next_char(&src);
    printf("%c\n", ch);
    return 0;
}
