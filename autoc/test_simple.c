#include "autoc.h"
#include <stdio.h>

int main() {
    const char* code = "var a = 10; a = 20; a";
    printf("Code: %s\n\n", code);

    AutoRunResult result = autoc_run(code);

    printf("\nResult value: ");
    if (result.value) {
        printf("%s\n", value_repr(result.value));
    } else {
        printf("(null)\n");
    }

    if (result.error_msg) {
        printf("Error: %s\n", result.error_msg);
    }

    autorun_free(&result);
    return 0;
}
