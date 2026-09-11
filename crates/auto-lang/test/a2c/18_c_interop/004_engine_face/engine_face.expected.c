#include "engine_face.h"

int scan_anchor(void* h) {
    char buf[256];
    int row = 0;
    while (row < 24) {
        if (autoterm_engine_row_text(h, row, buf, 256) > 0) {
            if (strstr(buf, "CFACE_OK") != 0) {
                return 1;
            }
        }
        row = row + 1;
    }
    return 0;
}

int main(void) {
    void* h = autoterm_engine_spawn(80, 24, "cmd");
    if (h == 0) {
        printf("%s\n", "SPAWN_FAIL");
        exit(1);
    }

    autoterm_engine_write_input(h, "echo CFACE_OK\r\n", 16);
    int dirty[64];
    int found = 0;
    int tries = 0;
    while (tries < 100) {
        if (autoterm_engine_feed_ready(h) == 1) {
            autoterm_engine_take_dirty_rows(h, dirty, 64);
            if (scan_anchor(h) == 1) {
                found = 1;
            }
        }
        if (found == 1) {
            tries = 100;
        } else {
            Sleep(50);
        }
        tries = tries + 1;
    }
    if (found == 1) {
        printf("%s\n", "CFACE_OK");
    } else {
        printf("%s\n", "ANCHOR_TIMEOUT");
    }
    autoterm_engine_kill(h);
    autoterm_engine_free(h);
    if (found == 1) {
        exit(0);
    }
    exit(2);
    return 0;
}
