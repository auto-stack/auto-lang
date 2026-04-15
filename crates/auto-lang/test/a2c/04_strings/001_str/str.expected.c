#include "str.h"

int main(void) {
    char* s = "Hello!";
    printf("%s\n", s);
    printf("%c\n", s[0]);
    for (int i = 1; i < 3; i++) {
        printf("%c", s[i]);
    }
    printf("\n");
    return 0;
}
