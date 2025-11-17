#include "auto/sys.h"
#include <stdio.h>

int main(void) {
    int pid = get_pid();
    printf("%s %d\n", "Pid is:", pid);
    return 0;
}
