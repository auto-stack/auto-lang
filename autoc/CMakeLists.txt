cmake_minimum_required(VERSION 3.10)
project(autoc C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

# Source files
set(AUTOC_SOURCES
    astr.c
    ast.c
    value.c
    lexer.c
    parser.c
    universe.c
    eval.c
    trans_c.c
    main.c
)

# Header files
set(AUTOC_HEADERS
    autoc.h
)

# Create executable
add_executable(autoc ${AUTOC_SOURCES} ${AUTOC_HEADERS})

# Test executable
add_executable(autoc_test
    astr.c
    ast.c
    value.c
    lexer.c
    parser.c
    universe.c
    eval.c
    main.c
    test.c
    trans_c.c
)
target_compile_definitions(autoc_test PRIVATE AUTOC_TEST_MAIN)

# Compiler flags
if(MSVC)
    target_compile_options(autoc PRIVATE /W4)
    target_compile_options(autoc_test PRIVATE /W4)
else()
    target_compile_options(autoc PRIVATE -Wall -Wextra -pedantic)
    target_compile_options(autoc_test PRIVATE -Wall -Wextra -pedantic)
endif()

# Installation
install(TARGETS autoc DESTINATION bin)

# Enable testing
enable_testing()
add_test(NAME autoc_test COMMAND autoc_test)

# AST representation test executable
add_executable(test_ast_repr
    astr.c
    ast.c
    value.c
    lexer.c
    parser.c
    universe.c
    eval.c
    trans_c.c
    test_ast_repr.c
)

# Parser test runner executable
add_executable(test_parser
    astr.c
    ast.c
    value.c
    lexer.c
    parser.c
    universe.c
    eval.c
    trans_c.c
    test_markdown.c
    test_parser.c
)

# Lexer test runner executable
add_executable(test_lexer
    astr.c
    ast.c
    value.c
    lexer.c
    parser.c
    universe.c
    eval.c
    trans_c.c
    test_markdown.c
    test_lexer.c
)

# Simple f-string test
add_executable(test_fstr_simple
    astr.c
    ast.c
    value.c
    lexer.c
    parser.c
    universe.c
    eval.c
    trans_c.c
    test_fstr_simple.c
)
