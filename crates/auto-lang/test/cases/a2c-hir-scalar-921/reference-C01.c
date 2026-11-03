/* hand-written certification reference (PLAN-921 T-10): value:42 */
#include <fcntl.h>
#include <io.h>
#include <stdio.h>

static int helper(int a) { return a * 2; }

int main(void) {
    _setmode(_fileno(stdout), _O_BINARY);
    printf("value:%d\n", helper(21));
    return 0;
}
