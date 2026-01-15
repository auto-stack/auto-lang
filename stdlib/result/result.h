// stdlib/result/result.h
// Result type for AutoLang C Foundation
// Represents either success (Ok) or error (Err)

#ifndef AUTO_RESULT_H
#define AUTO_RESULT_H

#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// Result tag discriminates between Ok and Err
typedef enum {
    Result_Ok,
    Result_Err
} ResultTag;

// Result represents either success (Ok) or error (Err)
// - If tag is Result_Ok, value points to the success value
// - If tag is Result_Err, error points to the error message
typedef struct {
    ResultTag tag;
    void* value;
    char* error;
} Result;

// Create an Ok value
// Parameters:
//   value - pointer to the success value
// Returns: Result with tag=Result_Ok
Result Result_ok(void* value);

// Create an Err value
// Parameters:
//   error - error message string (will be copied)
// Returns: Result with tag=Result_Err
Result Result_err(const char* error);

// Check if Result is Ok
// Parameters:
//   self - pointer to Result
// Returns: true if Ok, false if Err
bool Result_is_ok(Result* self);

// Check if Result is Err
// Parameters:
//   self - pointer to Result
// Returns: true if Err, false if Ok
bool Result_is_err(Result* self);

// Get the contained value (UNSAFE if Err)
// Parameters:
//   self - pointer to Result
// Returns: contained value pointer if Ok, NULL if Err
// WARNING: Calling this on Err is undefined behavior
void* Result_unwrap(Result* self);

// Get the contained error message (UNSAFE if Ok)
// Parameters:
//   self - pointer to Result
// Returns: error message if Err, NULL if Ok
// WARNING: Calling this on Ok is undefined behavior
const char* Result_unwrap_err(Result* self);

// Get the contained value or default
// Parameters:
//   self - pointer to Result
//   default_value - pointer to default value (returned if Err)
// Returns: contained value pointer if Ok, default_value if Err
void* Result_unwrap_or(Result* self, void* default_value);

// Get the error message or default
// Parameters:
//   self - pointer to Result
//   default_error - default error message (returned if Ok)
// Returns: error message if Err, default_error if Ok
const char* Result_unwrap_err_or(Result* self, const char* default_error);

// Clean up Result resources
// Parameters:
//   self - pointer to Result
// NOTE: Does not free the contained value/error, only the Result itself
void Result_drop(Result* self);

#ifdef __cplusplus
}
#endif

#endif // AUTO_RESULT_H
