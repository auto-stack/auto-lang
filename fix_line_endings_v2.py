#!/usr/bin/env python3
"""
Fix line endings in expected files for tests 046-070.
"""

test_names = [
    "046_binary", "047_tristate", "048_direction", "049_status", "050_mode",
    "051_result", "052_phase", "053_level", "054_state", "055_type",
    "056_side", "057_flow", "058_gate", "059_path", "060_color",
    "061_size", "062_speed", "063_power", "064_signal", "065_zone",
    "066_mode2", "067_link", "068_source", "069_target", "070_format"
]

for test_name in test_names:
    test_dir = f"crates/auto-lang/test/a2c/{test_name}"
    base_name = test_name.split('_', 1)[1]  # Get name after underscore

    for ext in ['.c', '.h']:
        expected_file = f"{test_dir}/{base_name}.expected{ext}"

        try:
            # Read file as text
            with open(expected_file, 'r', newline='') as f:
                content = f.read()

            # Write back with LF line endings only
            with open(expected_file, 'w', newline='\n') as f:
                f.write(content)

            print(f"Fixed: {expected_file}")
        except FileNotFoundError:
            print(f"Not found: {expected_file}")

print(f"\nProcessed {len(test_names)} test directories")
