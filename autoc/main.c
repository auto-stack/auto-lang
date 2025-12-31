/**
 * Auto-lang C Compiler - Main Entry Point
 */

#include "autoc.h"
#include "trans_c.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>

// ============================================================================
// Main API Implementation
// ============================================================================

AutoRunResult autoc_run(const char* code) {
    AutoRunResult result;
    result.result = AUTOC_OK;
    result.value = NULL;
    result.error_msg = NULL;


    if (!code) {
        result.result = AUTOC_ERROR_PARSE;
        result.error_msg = strdup("Null code provided");
        return result;
    }

    // Create lexer
    Lexer* lexer = lexer_new(code);
    if (!lexer) {
        result.result = AUTOC_ERROR_LEX;
        result.error_msg = strdup("Failed to create lexer");
        return result;
    }

    // Create parser
    Parser* parser = parser_new(lexer);
    if (!parser) {
        lexer_free(lexer);
        result.result = AUTOC_ERROR_PARSE;
        result.error_msg = strdup("Failed to create parser");
        return result;
    }

    // Parse code
    Code* ast = parser_parse(parser);
    if (!ast) {
        parser_free(parser);
        lexer_free(lexer);
        result.result = AUTOC_ERROR_PARSE;
        result.error_msg = strdup("Failed to parse code");
        return result;
    }

    // Create universe and evaluator
    Universe* universe = universe_new();
    Evaler* evaler = evaler_new(universe);

    // Evaluate
    Value* value = evaler_eval(evaler, ast);

    // Save the result before cleanup (clone it to survive universe cleanup)
    result.value = value_clone(value);
    value_free(value);  // Free the original value

    // Cleanup
    evaler_free(evaler);

    universe_free(universe);

    // Note: We don't free the AST here as it may be needed
    // In a real implementation, we'd have proper reference counting

    parser_free(parser);
    lexer_free(lexer);

    return result;
}

void autorun_free(AutoRunResult* result) {
    if (!result) return;
    if (result->value) {
        value_free(result->value);
    }
    if (result->error_msg) {
        free(result->error_msg);
    }
}

// ============================================================================
// Transpilation API Implementation
// ============================================================================

// Windows compatibility
#ifdef _WIN32
#define HAS_OPEN_MEMSTREAM 0
#else
#define HAS_OPEN_MEMSTREAM 1
#endif

AutoTransResult autoc_trans(const char* code, const char* name) {
    AutoTransResult result;
    result.result = AUTOC_OK;
    result.header_code = NULL;
    result.source_code = NULL;
    result.error_msg = NULL;

    if (!code) {
        result.result = AUTOC_ERROR_PARSE;
        result.error_msg = strdup("Null code provided");
        return result;
    }

    // Create lexer
    Lexer* lexer = lexer_new(code);
    if (!lexer) {
        result.result = AUTOC_ERROR_LEX;
        result.error_msg = strdup("Failed to create lexer");
        return result;
    }

    // Create parser
    Parser* parser = parser_new(lexer);
    if (!parser) {
        lexer_free(lexer);
        result.result = AUTOC_ERROR_PARSE;
        result.error_msg = strdup("Failed to create parser");
        return result;
    }

    // Parse code
    Code* ast = parser_parse(parser);
    if (!ast) {
        parser_free(parser);
        lexer_free(lexer);
        result.result = AUTOC_ERROR_PARSE;
        result.error_msg = strdup("Failed to parse code");
        return result;
    }

    // Create universe and transpiler
    Universe* universe = universe_new();
    CTrans* trans = ctrans_new(name, universe);

#if HAS_OPEN_MEMSTREAM
    // Open string buffers for output (Unix-like systems)
    trans->header = open_memstream(&trans->header_buf, &trans->header_size);
    trans->source = open_memstream(&trans->source_buf, &trans->source_size);

    if (!trans->header || !trans->source) {
        ctrans_free(trans);
        universe_free(universe);
        parser_free(parser);
        lexer_free(lexer);
        result.result = AUTOC_ERROR_EVAL;
        result.error_msg = strdup("Failed to open output stream");
        return result;
    }

    // Transpile
    int ret = ctrans_trans(trans, ast);
    if (ret != 0) {
        result.result = AUTOC_ERROR_EVAL;
        result.error_msg = strdup("Transpilation failed");
    }

    // Flush and close streams
    fflush(trans->header);
    fflush(trans->source);
    fclose(trans->header);
    fclose(trans->source);

    // Get the generated code
    result.header_code = trans->header_buf;
    result.source_code = trans->source_buf;
    trans->header_buf = NULL;
    trans->source_buf = NULL;
#else
    // On Windows, use direct buffer writing
    trans->header_buf = (char*)malloc(4096);
    trans->source_buf = (char*)malloc(4096);
    trans->header_buf[0] = '\0';
    trans->source_buf[0] = '\0';
    trans->header_capacity = 4096;
    trans->source_capacity = 4096;
    trans->header_size = 0;
    trans->source_size = 0;

    // Create temp files for output
    FILE* hf = tmpfile();
    FILE* sf = tmpfile();
    if (!hf || !sf) {
        if (hf) fclose(hf);
        if (sf) fclose(sf);
        free(trans->header_buf);
        free(trans->source_buf);
        ctrans_free(trans);
        universe_free(universe);
        parser_free(parser);
        lexer_free(lexer);
        result.result = AUTOC_ERROR_EVAL;
        result.error_msg = strdup("Failed to create temp files");
        return result;
    }

    trans->header = hf;
    trans->source = sf;

    // Transpile
    int ret = ctrans_trans(trans, ast);
    if (ret != 0) {
        result.result = AUTOC_ERROR_EVAL;
        result.error_msg = strdup("Transpilation failed");
    }

    // Read back the generated code
    fflush(hf);
    fflush(sf);
    rewind(hf);
    rewind(sf);

    // Read header
    fseek(hf, 0, SEEK_END);
    long hsize = ftell(hf);
    rewind(hf);
    trans->header_buf = (char*)malloc(hsize + 1);
    fread(trans->header_buf, 1, hsize, hf);
    trans->header_buf[hsize] = '\0';
    trans->header_size = hsize;

    // Read source
    fseek(sf, 0, SEEK_END);
    long ssize = ftell(sf);
    rewind(sf);
    trans->source_buf = (char*)malloc(ssize + 1);
    fread(trans->source_buf, 1, ssize, sf);
    trans->source_buf[ssize] = '\0';
    trans->source_size = ssize;

    fclose(hf);
    fclose(sf);

    result.header_code = trans->header_buf;
    result.source_code = trans->source_buf;
    trans->header_buf = NULL;
    trans->source_buf = NULL;
#endif

    // Cleanup
    ctrans_free(trans);
    universe_free(universe);
    parser_free(parser);
    lexer_free(lexer);

    return result;
}

void autotrans_free(AutoTransResult* result) {
    if (!result) return;
    if (result->header_code) free(result->header_code);
    if (result->source_code) free(result->source_code);
    if (result->error_msg) free(result->error_msg);
}

// ============================================================================
// REPL
// ============================================================================

static void print_version(void) {
    printf("auto-lang C Compiler v0.1.0\n");
    printf("A C implementation of the auto-lang compiler\n");
}

static void print_usage(const char* program) {
    printf("Usage: %s [options] [file]\n", program);
    printf("\nOptions:\n");
    printf("  -e <code>    Evaluate code string\n");
    printf("  -t <code>    Transpile code to C\n");
    printf("  -o <file>    Output file (for transpilation)\n");
    printf("  -v           Show version\n");
    printf("  -h           Show this help\n");
    printf("  --repl       Start interactive REPL\n");
}

static void run_repl(void) {
    printf("auto-lang REPL (Ctrl+C to exit)\n");
    printf("> ");

    char line[4096];
    while (fgets(line, sizeof(line), stdin)) {
        if (strcmp(line, "exit\n") == 0 || strcmp(line, "quit\n") == 0) {
            break;
        }

        AutoRunResult result = autoc_run(line);
        if (result.result == AUTOC_OK && result.value) {
            const char* repr = value_repr(result.value);
            printf("%s\n", repr);
        } else if (result.error_msg) {
            fprintf(stderr, "Error: %s\n", result.error_msg);
        }
        autorun_free(&result);

        printf("> ");
    }

    printf("\nGoodbye!\n");
}

static int run_file(const char* filename) {
    FILE* file = fopen(filename, "r");
    if (!file) {
        fprintf(stderr, "Error: Cannot open file: %s\n", filename);
        return 1;
    }

    // Read file content
    fseek(file, 0, SEEK_END);
    long file_size = ftell(file);
    fseek(file, 0, SEEK_SET);

    char* code = (char*)malloc(file_size + 1);
    if (!code) {
        fprintf(stderr, "Error: Memory allocation failed\n");
        fclose(file);
        return 1;
    }

    size_t read_size = fread(code, 1, file_size, file);
    code[read_size] = '\0';
    fclose(file);

    AutoRunResult result = autoc_run(code);
    int exit_code = 0;

    if (result.result == AUTOC_OK) {
        if (result.value) {
            const char* repr = value_repr(result.value);
            printf("%s\n", repr);
        }
    } else {
        fprintf(stderr, "Error: %s\n", result.error_msg);
        exit_code = 1;
    }

    autorun_free(&result);
    free(code);
    return exit_code;
}

static int transpile_file(const char* input_file, const char* output_base) {
    FILE* file = fopen(input_file, "r");
    if (!file) {
        fprintf(stderr, "Error: Cannot open file: %s\n", input_file);
        return 1;
    }

    // Read file content
    fseek(file, 0, SEEK_END);
    long file_size = ftell(file);
    fseek(file, 0, SEEK_SET);

    char* code = (char*)malloc(file_size + 1);
    if (!code) {
        fprintf(stderr, "Error: Memory allocation failed\n");
        fclose(file);
        return 1;
    }

    size_t read_size = fread(code, 1, file_size, file);
    code[read_size] = '\0';
    fclose(file);

    // Use output_base or derive from input file
    const char* base_name = output_base ? output_base : input_file;
    char name[256];
    // Remove directory and extension from base_name
    const char* last_slash = strrchr(base_name, '/');
    const char* last_backslash = strrchr(base_name, '\\');
    const char* filename_start = last_slash > last_backslash ? last_slash + 1 :
                                 last_backslash > last_slash ? last_backslash + 1 : base_name;
    const char* last_dot = strrchr(filename_start, '.');
    size_t name_len = last_dot ? (size_t)(last_dot - filename_start) : strlen(filename_start);
    snprintf(name, sizeof(name), "%.*s", (int)name_len, filename_start);

    // Transpile
    AutoTransResult result = autoc_trans(code, name);
    int exit_code = 0;

    if (result.result == AUTOC_OK) {
        // Write header file
        char header_path[512];
        snprintf(header_path, sizeof(header_path), "%s.h", name);
        FILE* hf = fopen(header_path, "w");
        if (hf) {
            fwrite(result.header_code, 1, strlen(result.header_code), hf);
            fclose(hf);
            printf("Generated: %s\n", header_path);
        }

        // Write source file
        char source_path[512];
        snprintf(source_path, sizeof(source_path), "%s.c", name);
        FILE* sf = fopen(source_path, "w");
        if (sf) {
            fwrite(result.source_code, 1, strlen(result.source_code), sf);
            fclose(sf);
            printf("Generated: %s\n", source_path);
        }
    } else {
        fprintf(stderr, "Error: %s\n", result.error_msg);
        exit_code = 1;
    }

    autotrans_free(&result);
    free(code);
    return exit_code;
}

// ============================================================================
// Main
// ============================================================================

#ifndef AUTOC_TEST_MAIN
int main(int argc, char* argv[]) {
    if (argc < 2) {
        print_usage(argv[0]);
        return 1;
    }

    // Parse arguments
    const char* transpile_output = NULL;
    const char* transpile_code = NULL;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-v") == 0 || strcmp(argv[i], "--version") == 0) {
            print_version();
            return 0;
        } else if (strcmp(argv[i], "-h") == 0 || strcmp(argv[i], "--help") == 0) {
            print_usage(argv[0]);
            return 0;
        } else if (strcmp(argv[i], "-e") == 0) {
            if (i + 1 >= argc) {
                fprintf(stderr, "Error: -e requires an argument\n");
                return 1;
            }
            const char* code = argv[++i];
            AutoRunResult result = autoc_run(code);
            int exit_code = 0;

            if (result.result == AUTOC_OK) {
                if (result.value) {
                    const char* repr = value_repr(result.value);
                    printf("%s\n", repr);
                }
            } else {
                fprintf(stderr, "Error: %s\n", result.error_msg);
                exit_code = 1;
            }

            autorun_free(&result);
            return exit_code;
        } else if (strcmp(argv[i], "-t") == 0) {
            if (i + 1 >= argc) {
                fprintf(stderr, "Error: -t requires an argument\n");
                return 1;
            }
            transpile_code = argv[++i];
        } else if (strcmp(argv[i], "-o") == 0) {
            if (i + 1 >= argc) {
                fprintf(stderr, "Error: -o requires an argument\n");
                return 1;
            }
            transpile_output = argv[++i];
        } else if (strcmp(argv[i], "--repl") == 0) {
            run_repl();
            return 0;
        } else if (argv[i][0] == '-') {
            fprintf(stderr, "Error: Unknown option: %s\n", argv[i]);
            return 1;
        } else {
            // Treat as file
            if (transpile_code) {
                // Transpile mode
                return transpile_file(argv[i], transpile_output);
            } else {
                return run_file(argv[i]);
            }
        }
    }

    // Handle transpile code from -t option
    if (transpile_code) {
        AutoTransResult result = autoc_trans(transpile_code, transpile_output ? transpile_output : "out");
        int exit_code = 0;

        if (result.result == AUTOC_OK) {
            printf("=== Generated C Code ===\n\n");
            printf("--- Header (.h) ---\n");
            printf("%s\n", result.header_code);
            printf("\n--- Source (.c) ---\n");
            printf("%s\n", result.source_code);
        } else {
            fprintf(stderr, "Error: %s\n", result.error_msg);
            exit_code = 1;
        }

        autotrans_free(&result);
        return exit_code;
    }

    return 0;
}
#endif // AUTOC_TEST_MAIN
