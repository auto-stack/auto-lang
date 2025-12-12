#include "std_getpid.h"

int main(void) {
    int pid = get_pid();
    printf("%s %d\n", "Pid is:", pid);
    return 0;
}
