/**
 * auto-lang C Compiler
 * A C implementation of the auto-lang compiler
 *
 * This is the main header file that includes all other headers.
 */

#ifndef AUTOC_H
#define AUTOC_H

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>

// Include all split headers
#include "astr.h"
#include "token.h"
#include "ast.h"
#include "lexer.h"
#include "parser.h"
#include "value.h"
#include "universe.h"
#include "eval.h"

// ============================================================================
// Main API
// ============================================================================

typedef enum {
    AUTOC_OK,
    AUTOC_ERROR_LEX,
    AUTOC_ERROR_PARSE,
    AUTOC_ERROR_EVAL,
} AutoResult;

typedef struct {
    AutoResult result;
    Value* value;
    char* error_msg;
} AutoRunResult;

AutoRunResult autoc_run(const char* code);
void autorun_free(AutoRunResult* result);

// ============================================================================
// Transpilation API
// ============================================================================

typedef struct {
    AutoResult result;
    char* header_code;
    char* source_code;
    char* error_msg;
} AutoTransResult;

AutoTransResult autoc_trans(const char* code, const char* name);
void autotrans_free(AutoTransResult* result);

#endif // AUTOC_H
