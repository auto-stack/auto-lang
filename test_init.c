#include "test_init.at.h"


int main(void) {
    struct MyMay x = {.tag = MYMAY_SOME, .as.some = 42};
    return 0;
}
