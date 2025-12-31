/**
 * Parser Test Runner
 * Reads test cases from parser_tests.md and runs them
 */

#include "autoc.h"
#include "test.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>

// Parse code and get AST representation
static char* get_ast_repr(const char* code) {
    if (!code) return strdup("Error: NULL code");

    Lexer* lexer = lexer_new(code);
    if (!lexer) return strdup("Error: Failed to create lexer");

    Parser* parser = parser_new(lexer);
    if (!parser) {
        lexer_free(lexer);
        return strdup("Error: Failed to create parser");
    }

    Code* ast = parser_parse(parser);
    if (!ast) {
        parser_free(parser);
        lexer_free(lexer);
        return strdup("Error: Failed to parse");
    }

    char* result = NULL;
    if (!ast->stmts) {
        result = strdup("Error: No statements");
    } else if (ast->count == 0) {
        result = strdup("Code(count: 0)");
    } else if (ast->count == 1) {
        // Single statement - use stmt or expr repr
        Stmt* stmt = ast->stmts[0];
        if (!stmt) {
            result = strdup("Error: NULL statement");
        } else if (stmt->kind == STMT_EXPR) {
            result = strdup(expr_repr(stmt->u.expr));
        } else {
            result = strdup(stmt_repr(stmt));
        }
    } else {
        // Multiple statements - use code repr
        result = strdup(code_repr(ast));
    }

    // Cleanup
    for (size_t i = 0; i < ast->count; i++) {
        if (ast->stmts[i]) {
            free(ast->stmts[i]);
        }
    }
    if (ast->stmts) free(ast->stmts);
    free(ast);
    parser_free(parser);
    lexer_free(lexer);

    return result ? result : strdup("Error: Unknown error");
}

// Test runner function
static bool run_parser_test(MarkdownTestCase* tc, TestStatistics* stats) {
    char* actual = get_ast_repr(tc->input);

    stats->run++;
    if (!compare_ignore_ws(actual, tc->expected)) {
        fprintf(stderr, "  \033[31mFAILED\033[0m: %s\n", tc->name);
        fprintf(stderr, "    Expected: %s\n", tc->expected);
        fprintf(stderr, "    Actual:   %s\n", actual);
        stats->failed++;
        free(actual);
        return false;
    }
    stats->passed++;
    printf("  \033[32mPASSED\033[0m: %s\n", tc->name);
    free(actual);
    return true;
}

int main() {
    return run_markdown_test_suite("tests/parser_tests.md", "Parser Test Runner", run_parser_test);
}
