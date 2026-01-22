#include "test_typed.at.h"


int main(void) {
    struct MyMay_int x = {.tag = MYMAY_SOME, .as.some = 42};
    return 0;
}
