// stdlib/result/option.h
// Option type for AutoLang C Foundation
// Represents an optional value: either Some(T) or None

#ifndef AUTO_OPTION_H
#define AUTO_OPTION_H

#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// Option tag discriminates between None and Some
typedef enum {
    Option_None,
    Option_Some
} OptionTag;

// Option represents an optional value
// - If tag is Option_None, value is NULL
// - If tag is Option_Some, value points to the contained data
typedef struct {
    OptionTag tag;
    void* value;
} Option;

// Create a None value
// Returns: Option with tag=Option_None, value=NULL
Option Option_none(void);

// Create a Some value
// Parameters:
//   value - pointer to the contained value (must remain valid for Option's lifetime)
// Returns: Option with tag=Option_Some, value=value
Option Option_some(void* value);

// Check if Option contains a value
// Parameters:
//   self - pointer to Option
// Returns: true if Some, false if None
bool Option_is_some(Option* self);

// Check if Option is None
// Parameters:
//   self - pointer to Option
// Returns: true if None, false if Some
bool Option_is_none(Option* self);

// Get the contained value (UNSAFE if None)
// Parameters:
//   self - pointer to Option
// Returns: contained value pointer, or NULL if None
// WARNING: Calling this on None is undefined behavior
void* Option_unwrap(Option* self);

// Get the contained value or return default
// Parameters:
//   self - pointer to Option
//   default_value - pointer to default value (returned if None)
// Returns: contained value pointer if Some, default_value if None
void* Option_unwrap_or(Option* self, void* default_value);

// Get the contained value or NULL
// Parameters:
//   self - pointer to Option
// Returns: contained value pointer if Some, NULL if None
void* Option unwrap_or_null(Option* self);

#ifdef __cplusplus
}
#endif

#endif // AUTO_OPTION_H
