// stdlib/result/test_option_result.c
// C tests for Option and Result types

#include <stdio.h>
#include <assert.h>
#include "option.h"
#include "result.h"

// Test Option_none
void test_option_none() {
    Option opt = Option_none();
    assert(opt.tag == Option_None);
    assert(opt.value == NULL);
    assert(Option_is_none(&opt));
    assert(!Option_is_some(&opt));
    printf("✓ test_option_none\n");
}

// Test Option_some
void test_option_some() {
    int value = 42;
    Option opt = Option_some(&value);
    assert(opt.tag == Option_Some);
    assert(opt.value == &value);
    assert(Option_is_some(&opt));
    assert(!Option_is_none(&opt));
    printf("✓ test_option_some\n");
}

// Test Option_unwrap with Some
void test_option_unwrap_some() {
    int value = 42;
    Option opt = Option_some(&value);
    void* unwrapped = Option_unwrap(&opt);
    assert(*(int*)unwrapped == 42);
    printf("✓ test_option_unwrap_some\n");
}

// Test Option_unwrap_or with Some
void test_option_unwrap_or_some() {
    int value = 42;
    int default_value = 100;
    Option opt = Option_some(&value);
    void* result = Option_unwrap_or(&opt, &default_value);
    assert(*(int*)result == 42);
    printf("✓ test_option_unwrap_or_some\n");
}

// Test Option_unwrap_or with None
void test_option_unwrap_or_none() {
    int default_value = 100;
    Option opt = Option_none();
    void* result = Option_unwrap_or(&opt, &default_value);
    assert(*(int*)result == 100);
    printf("✓ test_option_unwrap_or_none\n");
}

// Test Option_unwrap_or_null with Some
void test_option_unwrap_or_null_some() {
    int value = 42;
    Option opt = Option_some(&value);
    void* result = Option_unwrap_or_null(&opt);
    assert(result != NULL);
    assert(*(int*)result == 42);
    printf("✓ test_option_unwrap_or_null_some\n");
}

// Test Option_unwrap_or_null with None
void test_option_unwrap_or_null_none() {
    Option opt = Option_none();
    void* result = Option_unwrap_or_null(&opt);
    assert(result == NULL);
    printf("✓ test_option_unwrap_or_null_none\n");
}

// Test Result_ok
void test_result_ok() {
    int value = 42;
    Result res = Result_ok(&value);
    assert(res.tag == Result_Ok);
    assert(res.value == &value);
    assert(res.error == NULL);
    assert(Result_is_ok(&res));
    assert(!Result_is_err(&res));
    printf("✓ test_result_ok\n");
}

// Test Result_err
void test_result_err() {
    const char* error_msg = "something went wrong";
    Result res = Result_err(error_msg);
    assert(res.tag == Result_Err);
    assert(res.value == NULL);
    assert(res.error != NULL);
    assert(Result_is_err(&res));
    assert(!Result_is_ok(&res));
    printf("✓ test_result_err\n");

    // Clean up
    Result_drop(&res);
}

// Test Result_unwrap with Ok
void test_result_unwrap_ok() {
    int value = 42;
    Result res = Result_ok(&value);
    void* unwrapped = Result_unwrap(&res);
    assert(*(int*)unwrapped == 42);
    printf("✓ test_result_unwrap_ok\n");
}

// Test Result_unwrap_err with Err
void test_result_unwrap_err() {
    const char* error_msg = "test error";
    Result res = Result_err(error_msg);
    const char* error = Result_unwrap_err(&res);
    assert(error != NULL);
    printf("✓ test_result_unwrap_err\n");

    // Clean up
    Result_drop(&res);
}

// Test Result_unwrap_or with Ok
void test_result_unwrap_or_ok() {
    int value = 42;
    int default_value = 100;
    Result res = Result_ok(&value);
    void* result = Result_unwrap_or(&res, &default_value);
    assert(*(int*)result == 42);
    printf("✓ test_result_unwrap_or_ok\n");
}

// Test Result_unwrap_or with Err
void test_result_unwrap_or_err() {
    int default_value = 100;
    Result res = Result_err("error");
    void* result = Result_unwrap_or(&res, &default_value);
    assert(*(int*)result == 100);
    printf("✓ test_result_unwrap_or_err\n");

    // Clean up
    Result_drop(&res);
}

// Test Result_unwrap_err_or with Err
void test_result_unwrap_err_or_err() {
    const char* error_msg = "actual error";
    const char* default_error = "default error";
    Result res = Result_err(error_msg);
    const char* error = Result_unwrap_err_or(&res, default_error);
    assert(error != NULL);
    printf("✓ test_result_unwrap_err_or_err\n");

    // Clean up
    Result_drop(&res);
}

// Test Result_unwrap_err_or with Ok
void test_result_unwrap_err_or_ok() {
    int value = 42;
    const char* default_error = "default error";
    Result res = Result_ok(&value);
    const char* error = Result_unwrap_err_or(&res, default_error);
    assert(error == default_error);
    printf("✓ test_result_unwrap_err_or_ok\n");
}

// Test Result memory management
void test_result_memory() {
    // Create multiple errors to ensure proper allocation/deallocation
    for (int i = 0; i < 10; i++) {
        Result res = Result_err("test error");
        assert(res.error != NULL);
        Result_drop(&res);
    }
    printf("✓ test_result_memory\n");
}

// Divide function using Result type
Result divide(int a, int b) {
    if (b == 0) {
        return Result_err("division by zero");
    }
    static int result;
    result = a / b;
    return Result_ok(&result);
}

// Test divide function
void test_divide_success() {
    Result res = divide(10, 2);
    assert(Result_is_ok(&res));
    int* value = (int*)Result_unwrap(&res);
    assert(*value == 5);
    printf("✓ test_divide_success\n");
}

// Test divide by zero
void test_divide_by_zero() {
    Result res = divide(10, 0);
    assert(Result_is_err(&res));
    const char* error = Result_unwrap_err(&res);
    assert(error != NULL);
    printf("✓ test_divide_by_zero\n");

    // Clean up
    Result_drop(&res);
}

// Test NULL pointer handling
void test_null_pointers() {
    // Option with NULL
    assert(!Option_is_some(NULL));
    assert(Option_is_none(NULL));

    // Result with NULL
    assert(!Result_is_ok(NULL));
    assert(Result_is_err(NULL));

    printf("✓ test_null_pointers\n");
}

int main() {
    printf("Running Option tests...\n");
    test_option_none();
    test_option_some();
    test_option_unwrap_some();
    test_option_unwrap_or_some();
    test_option_unwrap_or_none();
    test_option_unwrap_or_null_some();
    test_option_unwrap_or_null_none();

    printf("\nRunning Result tests...\n");
    test_result_ok();
    test_result_err();
    test_result_unwrap_ok();
    test_result_unwrap_err();
    test_result_unwrap_or_ok();
    test_result_unwrap_or_err();
    test_result_unwrap_err_or_err();
    test_result_unwrap_err_or_ok();
    test_result_memory();
    test_divide_success();
    test_divide_by_zero();
    test_null_pointers();

    printf("\nAll C tests passed! (19 tests)\n");
    return 0;
}
