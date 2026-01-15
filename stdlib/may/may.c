// stdlib/may/may.c
// Unified May<T> type implementation for AutoLang

#include "may.h"
#include <stdlib.h>
#include <stdio.h>
#include <string.h>

// ==================== Creation Functions ====================

May May_empty(void) {
    May may;
    may.tag = May_Empty;
    may.data.value = NULL;
    return may;
}

May May_value(void* value) {
    May may;
    may.tag = May_Value;
    may.data.value = value;
    return may;
}

May May_error(void* error) {
    May may;
    may.tag = May_Error;
    may.data.error = error;
    return may;
}

// ==================== Inspection Functions ====================

bool May_is_empty(May* self) {
    if (!self) return true;
    return self->tag == May_Empty;
}

bool May_is_value(May* self) {
    if (!self) return false;
    return self->tag == May_Value;
}

bool May_is_error(May* self) {
    if (!self) return false;
    return self->tag == May_Error;
}

// ==================== Unwrapping Functions ====================

void* May_unwrap(May* self) {
    if (!self) {
        fprintf(stderr, "May_unwrap: NULL pointer\n");
        return NULL;
    }

    if (self->tag == May_Error) {
        fprintf(stderr, "May_unwrap: called on Error state\n");
        return NULL;
    }

    if (self->tag == May_Empty) {
        fprintf(stderr, "May_unwrap: called on Empty state\n");
        return NULL;
    }

    return self->data.value;
}

void* May_unwrap_or(May* self, void* default_value) {
    if (!self) return default_value;

    if (self->tag != May_Value) {
        return default_value;
    }

    return self->data.value;
}

void* May_unwrap_or_null(May* self) {
    return May_unwrap_or(self, NULL);
}

void* May_unwrap_error(May* self) {
    if (!self) {
        fprintf(stderr, "May_unwrap_error: NULL pointer\n");
        return NULL;
    }

    if (self->tag != May_Error) {
        fprintf(stderr, "May_unwrap_error: not in Error state\n");
        return NULL;
    }

    return self->data.error;
}

void* May_unwrap_error_or(May* self, void* default_error) {
    if (!self) return default_error;

    if (self->tag == May_Error) {
        return self->data.error;
    }

    return default_error;
}

// ==================== Cleanup Functions ====================

void May_drop(May* self) {
    if (self && self->tag == May_Error) {
        // Free error payload if it was allocated
        // Note: Value payload is owned by caller
        if (self->data.error) {
            // For now, we don't free error since we don't know if it was allocated
            // In the future, we can track this with a flag
        }
    }
}
