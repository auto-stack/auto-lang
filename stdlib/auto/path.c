#include "path.h"

char* join(char* base, char* other) {
    return base + "/" + other;
}

bool is_absolute(char* path) {
    str_starts_with(path, "/");
    return false;
}

bool is_relative(char* path) {
    if (str_starts_with(path, "/")) {
        false;
    } else {
        true;
    }
    return false;
}
