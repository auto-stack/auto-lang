// py_random Auto test — TAP output format (Plan 369 Python parity).
// The parity runner sets the working directory to the library root
// (parity/libs/py_random), so `use.py random` resolves the Python stdlib
// module.
//
// Test names MUST match tests/python/test_random.py because the comparator
// joins backends by name.
//
// Design note: random numbers are non-deterministic unless seeded, so every
// test re-seeds before generating a value. The parity comparator only looks at
// TAP pass/fail, so to detect value divergences the comparison is baked into
// pass/fail: each test re-seeds, generates a value, and checks it against a
// hard-coded expected value captured from the CPython 3 oracle. If AutoVM's
// PyFFI reproduces the seeded sequence it emits `ok`; otherwise `not ok`
// (flagged as an AutoVM bug). See tests/python/test_random.py for the value
// table.
//
// random.randrange is omitted: its 1- and 3-argument forms hit the same
// multi-argument FFI corruption as math.gcd/lcm/perm (stray value leaks to
// stdout, return value wrong), which would corrupt the TAP stream. The 2-arg
// randint path works cleanly and is what this suite exercises.
//
// Plan 369 P4 (Task 16): added choice (seeded, over a string sequence) and
// uniform (seeded, asserted by exact float string match). choice returns a
// single-character string; uniform returns a float that marshals to its
// shortest round-tripping string form (deterministic for IEEE-754 doubles).
use.py random: seed, randint, choice, uniform

fn tap_ok(n int, name str) {
    print("ok " + n.to(str) + " - " + name)
}

fn tap_not_ok(n int, name str, diag str) {
    print("not ok " + n.to(str) + " - " + name + " # " + diag)
}

fn main() {
    seed(42)
    if randint(1, 100) == 82 { tap_ok(1, "test_seed_randint") } else { tap_not_ok(1, "test_seed_randint", "got " + randint(1, 100).to(str)) }

    seed(42)
    if randint(1, 1000) == 655 { tap_ok(2, "test_seed_randint_1000") } else { tap_not_ok(2, "test_seed_randint_1000", "got " + randint(1, 1000).to(str)) }

    seed(100)
    if randint(1, 100) == 19 { tap_ok(3, "test_seed100_randint") } else { tap_not_ok(3, "test_seed100_randint", "got " + randint(1, 100).to(str)) }

    seed(42)
    if randint(1, 100) == 82 { tap_ok(4, "test_seed_randint_again") } else { tap_not_ok(4, "test_seed_randint_again", "got " + randint(1, 100).to(str)) }

    seed(42)
    if randint(1, 10) == 2 { tap_ok(5, "test_seed_randint_small") } else { tap_not_ok(5, "test_seed_randint_small", "got " + randint(1, 10).to(str)) }

    seed(7)
    if randint(1, 50) == 21 { tap_ok(6, "test_seed7_randint") } else { tap_not_ok(6, "test_seed7_randint", "got " + randint(1, 50).to(str)) }

    seed(42)
    if randint(0, 5) == 5 { tap_ok(7, "test_seed_randint_zero_low") } else { tap_not_ok(7, "test_seed_randint_zero_low", "got " + randint(0, 5).to(str)) }

    seed(0)
    if randint(1, 1000) == 865 { tap_ok(8, "test_seed0_randint") } else { tap_not_ok(8, "test_seed0_randint", "got " + randint(1, 1000).to(str)) }

    // Plan 369 P4 (Task 16): seeded choice (over a string sequence) and uniform.
    seed(42)
    var c1 = choice("abcde")
    if c1 == "a" { tap_ok(9, "test_seed_choice_str") } else { tap_not_ok(9, "test_seed_choice_str", "got " + c1.to(str)) }

    seed(42)
    var c2 = choice("abcdef")
    if c2 == "f" { tap_ok(10, "test_seed_choice_str2") } else { tap_not_ok(10, "test_seed_choice_str2", "got " + c2.to(str)) }

    seed(7)
    var c3 = choice("abcdef")
    if c3 == "c" { tap_ok(11, "test_seed7_choice_str") } else { tap_not_ok(11, "test_seed7_choice_str", "got " + c3.to(str)) }

    // uniform returns a float; compare its shortest round-tripping string form.
    seed(42)
    var u1 = uniform(1.0, 10.0)
    if u1.to(str) == "6.754841186120954" { tap_ok(12, "test_seed_uniform") } else { tap_not_ok(12, "test_seed_uniform", "got " + u1.to(str)) }

    seed(42)
    var u2 = uniform(0.0, 1.0)
    if u2.to(str) == "0.6394267984578837" { tap_ok(13, "test_seed_uniform_01") } else { tap_not_ok(13, "test_seed_uniform_01", "got " + u2.to(str)) }
}
