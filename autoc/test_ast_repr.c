/**
 * AST Representation Test
 * Tests the AutoLang atom format for AST nodes
 */

#include "autoc.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Helper function to parse code and get the AST
static Code* parse_code(const char* code) {
    Lexer* lexer = lexer_new(code);
    Parser* parser = parser_new(lexer);
    Code* ast = parser_parse(parser);

    parser_free(parser);
    lexer_free(lexer);

    return ast;
}

// Helper function to free AST
static void free_code(Code* code) {
    if (!code) return;
    for (size_t i = 0; i < code->count; i++) {
        // Note: This is a simplified free that doesn't recursively free the AST
        // In production, you'd want a proper ast_free function
        free(code->stmts[i]);
    }
    free(code->stmts);
    free(code);
}

int main() {
    printf("=============================================================\n");
    printf("  AST Representation Test - AutoLang Atom Format\n");
    printf("=============================================================\n\n");

    // Test 1: Integer literal
    printf("Test 1: Integer Literal\n");
    printf("Input: 42\n");
    Code* ast1 = parse_code("42");
    if (ast1 && ast1->count > 0) {
        Stmt* stmt = ast1->stmts[0];
        if (stmt->kind == STMT_EXPR) {
            printf("Output: %s\n\n", expr_repr(stmt->u.expr));
        }
    }
    free_code(ast1);

    // Test 2: Identifier
    printf("Test 2: Identifier\n");
    printf("Input: x\n");
    Code* ast2 = parse_code("x");
    if (ast2 && ast2->count > 0) {
        Stmt* stmt = ast2->stmts[0];
        if (stmt->kind == STMT_EXPR) {
            printf("Output: %s\n\n", expr_repr(stmt->u.expr));
        }
    }
    free_code(ast2);

    // Test 3: Binary operation
    printf("Test 3: Binary Operation\n");
    printf("Input: 1 + 2\n");
    Code* ast3 = parse_code("1 + 2");
    if (ast3 && ast3->count > 0) {
        Stmt* stmt = ast3->stmts[0];
        if (stmt->kind == STMT_EXPR) {
            printf("Output: %s\n\n", expr_repr(stmt->u.expr));
        }
    }
    free_code(ast3);

    // Test 4: Array literal
    printf("Test 4: Array Literal\n");
    printf("Input: [1, 2, 3]\n");
    Code* ast4 = parse_code("[1, 2, 3]");
    if (ast4 && ast4->count > 0) {
        Stmt* stmt = ast4->stmts[0];
        if (stmt->kind == STMT_EXPR) {
            printf("Output: %s\n\n", expr_repr(stmt->u.expr));
        }
    }
    free_code(ast4);

    // Test 5: Variable declaration
    printf("Test 5: Variable Declaration\n");
    printf("Input: var x = 42\n");
    Code* ast5 = parse_code("var x = 42");
    if (ast5 && ast5->count > 0) {
        printf("Output: %s\n\n", stmt_repr(ast5->stmts[0]));
    }
    free_code(ast5);

    // Test 6: For loop
    printf("Test 6: For Loop\n");
    printf("Input: for i in 0..3 { i }\n");
    Code* ast6 = parse_code("for i in 0..3 { i }");
    if (ast6 && ast6->count > 0) {
        printf("Output: %s\n\n", stmt_repr(ast6->stmts[0]));
    }
    free_code(ast6);

    // Test 7: Block statement
    printf("Test 7: Block Statement\n");
    printf("Input: { var x = 1; x + 2 }\n");
    Code* ast7 = parse_code("{ var x = 1; x + 2 }");
    if (ast7 && ast7->count > 0) {
        printf("Output: %s\n\n", stmt_repr(ast7->stmts[0]));
    }
    free_code(ast7);

    // Test 8: Multiple statements (Code)
    printf("Test 8: Code with Multiple Statements\n");
    printf("Input:\n");
    printf("  var x = 42\n");
    printf("  x\n");
    printf("  x + 1\n");
    Code* ast8 = parse_code("var x = 42\nx\nx + 1");
    printf("Output: %s\n\n", code_repr(ast8));
    free_code(ast8);

    // Test 9: Range expression
    printf("Test 9: Range Expression\n");
    printf("Input: 0..10\n");
    Code* ast9 = parse_code("0..10");
    if (ast9 && ast9->count > 0) {
        Stmt* stmt = ast9->stmts[0];
        if (stmt->kind == STMT_EXPR) {
            printf("Output: %s\n\n", expr_repr(stmt->u.expr));
        }
    }
    free_code(ast9);

    // Test 10: Function call
    printf("Test 10: Function Call\n");
    printf("Input: print(42)\n");
    Code* ast10 = parse_code("print(42)");
    if (ast10 && ast10->count > 0) {
        Stmt* stmt = ast10->stmts[0];
        if (stmt->kind == STMT_EXPR) {
            printf("Output: %s\n\n", expr_repr(stmt->u.expr));
        }
    }
    free_code(ast10);

    printf("=============================================================\n");
    printf("  All tests completed!\n");
    printf("=============================================================\n");

    return 0;
}
