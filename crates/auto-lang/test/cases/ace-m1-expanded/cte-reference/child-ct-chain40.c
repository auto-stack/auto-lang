/* Independent PLAN-941 finite source-CT reference. */
#include <stdio.h>
#include <fcntl.h>
#include <io.h>
int main(void) {
    _setmode(_fileno(stdout), _O_BINARY);
    printf("value:%d\n", (int)(1 ? 40 : 42));
    return 0;
}
