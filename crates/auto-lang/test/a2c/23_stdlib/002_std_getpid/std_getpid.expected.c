#include "std_getpid.h"

int main(void) {
    unknown pid = getpid();
    printf("%s %d\n", "Pid is:", pid);
    return 0;
}
