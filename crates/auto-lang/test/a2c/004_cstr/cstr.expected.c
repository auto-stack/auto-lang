#include <stdio.h>

void println(char* msg) {
    printf("%s\n", msg);
}

int main(void) {
    char* s = "Hello!";
    println(s);
    return 0;
}
