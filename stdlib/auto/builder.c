// StringBuilder C Implementation
// Phase 2: StringBuilder (Plan 027)

#include "builder.h"
#include "may.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

// ============================================================================
// API - Creation
// ============================================================================

May* StringBuilder_new(size_t initial_capacity) {
    StringBuilder* sb = (StringBuilder*)malloc(sizeof(StringBuilder));
    if (!sb) {
        return May_error("out of memory");
    }

    sb->buffer = (char*)malloc(initial_capacity);
    if (!sb->buffer) {
        free(sb);
        return May_error("out of memory");
    }

    sb->len = 0;
    sb->capacity = initial_capacity;
    sb->buffer[0] = '\0';

    return May_value(sb);
}

void StringBuilder_drop(StringBuilder* sb) {
    if (sb) {
        free(sb->buffer);
        free(sb);
    }
}

// ============================================================================
// API - Building
// ============================================================================

May* StringBuilder_append(StringBuilder* sb, const char* str) {
    if (!sb || !str) {
        return May_error("null argument");
    }

    size_t str_len = strlen(str);

    // Resize if needed
    while (sb->len + str_len + 1 >= sb->capacity) {
        size_t new_capacity = sb->capacity * 2;
        char* new_buffer = (char*)realloc(sb->buffer, new_capacity);
        if (!new_buffer) {
            return May_error("out of memory");
        }
        sb->buffer = new_buffer;
        sb->capacity = new_capacity;
    }

    // Append string
    memcpy(sb->buffer + sb->len, str, str_len);
    sb->len += str_len;
    sb->buffer[sb->len] = '\0';

    return May_value(sb);
}

May* StringBuilder_append_char(StringBuilder* sb, char c) {
    if (!sb) {
        return May_error("null argument");
    }

    // Resize if needed
    if (sb->len + 2 >= sb->capacity) {
        size_t new_capacity = sb->capacity * 2;
        char* new_buffer = (char*)realloc(sb->buffer, new_capacity);
        if (!new_buffer) {
            return May_error("out of memory");
        }
        sb->buffer = new_buffer;
        sb->capacity = new_capacity;
    }

    // Append character
    sb->buffer[sb->len] = c;
    sb->len++;
    sb->buffer[sb->len] = '\0';

    return May_value(sb);
}

May* StringBuilder_append_int(StringBuilder* sb, int value) {
    if (!sb) {
        return May_error("null argument");
    }

    char buffer[32];
    snprintf(buffer, sizeof(buffer), "%d", value);
    return StringBuilder_append(sb, buffer);
}

// ============================================================================
// API - Finalization
// ============================================================================

char* StringBuilder_build(StringBuilder* sb) {
    if (!sb) {
        return NULL;
    }

    // Return null-terminated string (caller owns it)
    char* result = strdup(sb->buffer);
    if (!result) {
        return NULL;
    }

    return result;
}

void StringBuilder_clear(StringBuilder* sb) {
    if (sb) {
        sb->len = 0;
        sb->buffer[0] = '\0';
    }
}

size_t StringBuilder_len(StringBuilder* sb) {
    return sb ? sb->len : 0;
}
