#include "format.h"


bool is_first(struct Format m) {
    switch (m.tag) {
    case FORMAT_TEXT:
        {
            return true;
        }
        break;
    case FORMAT_BINARY:
        {
            return false;
        }
        break;
    }
    return false;
}

int main(void) {
    struct Format s = {.tag = FORMAT_TEXT, .as.Text = 0};
    return is_first(s);
}
