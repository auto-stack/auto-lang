#pragma once

#include <stdbool.h>
enum ResultKind {
    RESULT_OK,
    RESULT_ERR,
};

struct Result {
    enum ResultKind tag;
    union {
        int Ok;
        int Err;
    } as;
};
char* get_message(struct Result r);
bool is_ok(struct Result r);
bool is_err(struct Result r);
