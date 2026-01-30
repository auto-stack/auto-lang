#include "may_patterns.h"


char* get_message(struct Result r) {
    switch (r.tag) {
    case RESULT_OK:
        {
            return "success";
        }
        break;
    case RESULT_ERR:
        {
            return "error";
        }
        break;
    }
    return "";
}

bool is_ok(struct Result r) {
    switch (r.tag) {
    case RESULT_OK:
        {
            return true;
        }
        break;
    case RESULT_ERR:
        {
            return false;
        }
        break;
    }
    return false;
}

bool is_err(struct Result r) {
    switch (r.tag) {
    case RESULT_OK:
        {
            return false;
        }
        break;
    case RESULT_ERR:
        {
            return true;
        }
        break;
    }
    return false;
}

int main(void) {
    struct Result r1 = {.tag = RESULT_OK, .as.Ok = 42};
    char* msg1 = get_message(r1);
    bool check1 = is_ok(r1);

    struct Result r2 = {.tag = RESULT_ERR, .as.Err = 1};
    char* msg2 = get_message(r2);
    bool check2 = is_err(r2);

    return check1;
}
