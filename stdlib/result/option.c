// stdlib/result/option.c
// Option type implementation for AutoLang C Foundation

#include "option.h"
#include <stdlib.h>
#include <stdio.h>
#include <string.h>

// Create a None value
Option Option_none(void) {
    Option opt;
    opt.tag = Option_None;
    opt.value = NULL;
    return opt;
}

// Create a Some value
Option Option_some(void* value) {
    Option opt;
    opt.tag = Option_Some;
    opt.value = value;
    return opt;
}

// Check if Option contains a value
bool Option_is_some(Option* self) {
    if (!self) return false;
    return self->tag == Option_Some;
}

// Check if Option is None
bool Option_is_none(Option* self) {
    if (!self) return true;
    return self->tag == Option_None;
}

// Get the contained value (UNSAFE if None)
void* Option_unwrap(Option* self) {
    if (!self) {
        fprintf(stderr, "Option_unwrap: NULL pointer\n");
        return NULL;
    }

    if (self->tag == Option_None) {
        fprintf(stderr, "Option_unwrap: called on None\n");
        return NULL;
    }

    return self->value;
}

// Get the contained value or return default
void* Option_unwrap_or(Option* self, void* default_value) {
    if (!self) return default_value;

    if (self->tag == Option_None) {
        return default_value;
    }

    return self->value;
}

// Get the contained value or NULL
void* Option_unwrap_or_null(Option* self) {
    if (!self) return NULL;

    if (self->tag == Option_None) {
        return NULL;
    }

    return self->value;
}
