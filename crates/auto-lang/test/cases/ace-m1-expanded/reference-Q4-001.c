// Independent PLAN-941 arithmetic reference; no generated headers are consumed.
#include <stdio.h>
#include <fcntl.h>
#include <io.h>
int main(void) {
    _setmode(_fileno(stdout), _O_BINARY);
    printf("value:%d\n", 41 + 1);
    return 0;
}
