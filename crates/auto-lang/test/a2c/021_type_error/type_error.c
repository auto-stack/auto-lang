#include "type_error.at.h"


int main(void) {
    struct target t = {.name = "foo", .at = 42};
    printf("%d\n", t.name);
    return 0;
}
