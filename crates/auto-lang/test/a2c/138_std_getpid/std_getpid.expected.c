#include "std_getpid.h"

int main(void) {
    int pid = getpid();
    printf("%s %d\n", "Pid is:", pid);
    return 0;
}
