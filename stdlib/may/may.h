// stdlib/may/may.h
// Unified May<T> type for AutoLang
// Three-state enum: Empty, Value, Error
// Combines semantics of Option<T> and Result<T, E>

#ifndef AUTO_MAY_H
#define AUTO_MAY_H

#include <stdint.h>
#include <stdbool.h>

// Three-state tag
typedef enum {
    May_Empty = 0x00,  // No value (like None)
    May_Value = 0x01,  // Has value (like Some/Ok)
    May_Error = 0x02   // Has error (like Err)
} MayTag;

// Generic May type (using void* for generic value/error)
typedef struct {
    uint8_t tag;
    union {
        void* value;    // Valid data when tag = May_Value
        void* error;    // Error payload when tag = May_Error
    } data;
} May;

// ==================== Creation Functions ====================

// Create an Empty May (no value)
May May_empty(void);

// Create a May with a value
May May_value(void* value);

// Create a May with an error
May May_error(void* error);

// ==================== Inspection Functions ====================

// Check if May is Empty
bool May_is_empty(May* self);

// Check if May has a Value
bool May_is_value(May* self);

// Check if May has an Error
bool May_is_error(May* self);

// ==================== Unwrapping Functions ====================

// Get the value (unsafe if Empty or Error)
void* May_unwrap(May* self);

// Get the value or return default
void* May_unwrap_or(May* self, void* default_value);

// Get the value or return NULL
void* May_unwrap_or_null(May* self);

// Get the error message (unsafe if not Error)
void* May_unwrap_error(May* self);

// Get the error or return default_error
void* May_unwrap_error_or(May* self, void* default_error);

// ==================== Cleanup Functions ====================

// Clean up May resources
void May_drop(May* self);

#endif // AUTO_MAY_H
