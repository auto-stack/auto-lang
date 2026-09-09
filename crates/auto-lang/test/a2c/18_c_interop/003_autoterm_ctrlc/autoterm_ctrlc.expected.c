#include "autoterm_ctrlc.h"

int send_evt(unsigned int evt) {
    if (GenerateConsoleCtrlEvent(evt, 0) == 0) {
        return 2;
    }
    return 0;
}

int main(void) {
    char* cmd = GetCommandLineA();
    char* sp = strchr(cmd, 32);
    if (sp == 0) {
        exit(3);
    }
    int pid = atoi(sp);
    if (pid == 0) {
        exit(3);
    }
    FreeConsole();
    if (AttachConsole(pid) == 0) {
        exit(2);
    }

    SetConsoleCtrlHandler(closure_0, 1);

    int ret = send_evt(1);
    Sleep(50);
    int c = send_evt(0);
    if (ret == 0) {
        if (c != 0) {
            ret = c;
        }
    }
    exit(ret);
    return 0;
}

int closure_0(int evt) {
    return 1;
}
