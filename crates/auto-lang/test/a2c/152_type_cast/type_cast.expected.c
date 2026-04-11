#include "type_cast.h"

int main(void) {
    int x = 42;
    unsigned int y = ((unsigned int)(x));
    printf("%d\n", y);
    return 0;
}
