// stdlib/result/result.c
// Result type implementation for AutoLang C Foundation

#include "result.h"
#include <stdlib.h>
#include <stdio.h>
#include <string.h>

// Create an Ok value
Result Result_ok(void* value) {
    Result res;
    res.tag = Result_Ok;
    res.value = value;
    res.error = NULL;
    return res;
}

// Create an Err value
Result Result_err(const char* error) {
    Result res;
    res.tag = Result_Err;
    res.value = NULL;

    // Copy error message
    if (error) {
        size_t len = strlen(error) + 1;
        res.error = (char*)malloc(len);
        if (res.error) {
            memcpy(res.error, error, len);
        } else {
            fprintf(stderr, "Result_err: failed to allocate memory for error message\n");
        }
    } else {
        res.error = NULL;
    }

    return res;
}

// Check if Result is Ok
bool Result_is_ok(Result* self) {
    if (!self) return false;
    return self->tag == Result_Ok;
}

// Check if Result is Err
bool Result_is_err(Result* self) {
    if (!self) return true;
    return self->tag == Result_Err;
}

// Get the contained value (UNSAFE if Err)
void* Result_unwrap(Result* self) {
    if (!self) {
        fprintf(stderr, "Result_unwrap: NULL pointer\n");
        return NULL;
    }

    if (self->tag == Result_Err) {
        fprintf(stderr, "Result_unwrap: called on Err: %s\n", self->error);
        return NULL;
    }

    return self->value;
}

// Get the contained error message (UNSAFE if Ok)
const char* Result_unwrap_err(Result* self) {
    if (!self) {
        fprintf(stderr, "Result_unwrap_err: NULL pointer\n");
        return "NULL Result";
    }

    if (self->tag == Result_Ok) {
        fprintf(stderr, "Result_unwrap_err: called on Ok\n");
        return "called on Ok";
    }

    return self->error;
}

// Get the contained value or default
void* Result_unwrap_or(Result* self, void* default_value) {
    if (!self) return default_value;

    if (self->tag == Result_Err) {
        return default_value;
    }

    return self->value;
}

// Get the error message or default
const char* Result_unwrap_err_or(Result* self, const char* default_error) {
    if (!self) return default_error;

    if (self->tag == Result_Ok) {
        return default_error ? default_error : "no error";
    }

    return self->error;
}

// Clean up Result resources
void Result_drop(Result* self) {
    if (self) {
        // Free error message if allocated
        if (self->error) {
            free(self->error);
            self->error = NULL;
        }
    }
}
