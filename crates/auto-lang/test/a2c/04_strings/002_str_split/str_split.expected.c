#include "str_split.h"

int main(void) {
    char* text = "hello world";
    unknown words = str_split(text);
    printf("%d\n", words[0]);
    return 0;
}
