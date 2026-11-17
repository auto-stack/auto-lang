// AUTO-GENERATED (PLAN-921 CA-A) — DO NOT EDIT.
#include <fcntl.h>
#include <inttypes.h>
#include <io.h>
#include <stdint.h>
#include <stdio.h>
#include "a2x_c_runtime.h"

a2x_result fn_main(void);

int main(void) {
    _setmode(_fileno(stdout), _O_BINARY);
    _setmode(_fileno(stderr), _O_BINARY);
    a2x_result outcome = fn_main();
    if (outcome.trap == A2X_TRAP_NONE) {
        printf("value:%" PRId32 "\n", outcome.value);
    } else {
        printf("trap:%s\n", a2x_trap_name(outcome.trap));
    }
    return 0;
}
