# Mixed-Mode Project Examples

This document provides complete examples of mixed-mode projects using AutoLang's mode selection feature.

## Example 1: Embedded Firmware with HAL

### Project Structure

```
embedded_firmware/
├── pac.at              # Project manifest
├── src/
│   ├── main.at         # Main AutoVM application
│   └── hal.at          # Hardware abstraction (C transpilation)
└── target/
    ├── main.bc         # AutoVM bytecode
    ├── hal.c           # Generated C code
    ├── hal.h           # Generated C header
    └── hal.o           # Compiled C object
```

### pac.at

```auto
name: "embedded_firmware"
version: "1.0.0"
mode: "autovm"  # Main app uses AutoVM

app("embedded_firmware") {
    dependencies: [
        "std:core",
        ("hal", mode: "c"),  # Hardware layer in C
    ]
}
```

### src/hal.at (C Transpilation)

```auto
// Hardware Abstraction Layer - transpiled to C

// GPIO functions
#[c]
fn gpio_init(pin int) int {
    // C implementation will be generated
    // Returns 0 on success
    0
}

#[c]
fn gpio_set_direction(pin int, direction int) int {
    0
}

#[c]
fn gpio_write(pin int, value int) int {
    0
}

#[c]
fn gpio_read(pin int) int {
    0
}

// SPI functions
#[c]
fn spi_init() int {
    0
}

#[c]
fn spi_transfer(data int) int {
    0
}

// I2C functions
#[c]
fn i2c_init(address int) int {
    0
}

#[c]
fn i2c_write(data int) int {
    0
}

#[c]
fn i2c_read() int {
    0
}
```

### src/main.at (AutoVM)

```auto
// Main application - runs on AutoVM
extern "c" {
    // GPIO functions from HAL
    fn gpio_init(pin int) int;
    fn gpio_set_direction(pin int, direction int) int;
    fn gpio_write(pin int, value int) int;
    fn gpio_read(pin int) int;

    // SPI functions from HAL
    fn spi_init() int;
    fn spi_transfer(data int) int;

    // I2C functions from HAL
    fn i2c_init(address int) int;
    fn i2c_write(data int) int;
    fn i2c_read() int;
}

// Constants
const LED_PIN = 13
const OUTPUT = 1
const INPUT = 0
const HIGH = 1
const LOW = 0

fn setup() int {
    // Initialize LED pin
    let result = gpio_init(LED_PIN);
    if result != 0 {
        say("Failed to initialize GPIO")
        return result
    }

    result = gpio_set_direction(LED_PIN, OUTPUT);
    if result != 0 {
        say("Failed to set GPIO direction")
        return result
    }

    // Initialize SPI
    result = spi_init();
    if result != 0 {
        say("Failed to initialize SPI")
        return result
    }

    say("Setup complete")
    0
}

fn loop() {
    // Blink LED
    gpio_write(LED_PIN, HIGH);
    // delay(1000);  // TODO: Implement delay

    gpio_write(LED_PIN, LOW);
    // delay(1000);

    // Read sensor via SPI
    let sensor_data = spi_transfer(0x00);
    say("Sensor data: " + sensor_data)
}

fn main() {
    let setup_result = setup();
    if setup_result == 0 {
        // Main loop
        loop()
    }
}
```

### Build Process

```bash
#!/bin/bash
# build.sh

# 1. Transpile HAL to C
auto trans_c src/hal.at -o target/hal

# 2. Compile HAL to object file
gcc -c -o target/hal.o target/hal.c

# 3. Compile main app to AutoVM bytecode
auto build src/main.at -o target/main.bc

# 4. Link everything together
# This step depends on your specific runtime
# (AutoVM runtime + HAL object file)

echo "Build complete!"
```

## Example 2: Server Application with Crypto

### Project Structure

```
secure_server/
├── pac.at
├── src/
│   ├── main.at         # Server logic (AutoVM)
│   ├── crypto.at       # Crypto library (Rust transpilation)
│   └── database.at     # Database layer (AutoVM)
└── target/
    ├── main.bc
    ├── crypto.rs
    └── database.bc
```

### pac.at

```auto
name: "secure_server"
version: "1.0.0"
mode: "autovm"

app("secure_server") {
    dependencies: [
        "std:core",
        "std:io",
        "std:net",
        ("crypto", mode: "rust"),  # High-performance crypto
        "database",                # Database logic in AutoVM
    ]
}
```

### src/crypto.at (Rust Transpilation)

```auto
// Cryptographic functions - transpiled to Rust

#[rust]
fn sha256_hash(data str) str {
    // Rust implementation will be generated
    "hash_placeholder"
}

#[rust]
fn aes_encrypt(plaintext str, key str) str {
    "encrypted_placeholder"
}

#[rust]
fn aes_decrypt(ciphertext str, key str) str {
    "decrypted_placeholder"
}

#[rust]
fn hmac_sign(message str, key str) str {
    "signature_placeholder"
}

#[rust]
fn hmac_verify(message str, signature str, key str) int {
    1  # Return 1 for valid
}
```

### src/database.at (AutoVM)

```auto
// Database layer - pure AutoVM

type User struct {
    id int
    username str
    email str
    password_hash str
}

type Database struct {
    users List<User>
}

fn database_new() Database {
    Database {
        users: List.new()
    }
}

fn (db Database) add_user(username str, email str, password str) int {
    let user = User {
        id: db.users.len() + 1,
        username: username,
        email: email,
        password_hash: password,  # TODO: Hash with crypto module
    }
    db.users.push(user)
    user.id
}

fn (db Database) get_user(id int) Option<User> {
    if id >= 0 && id < db.users.len() {
        Some(db.users.get(id))
    } else {
        None
    }
}

fn (db Database) authenticate(username str, password str) int {
    let user_id = -1

    for i in 0..db.users.len() {
        let user = db.users.get(i)
        if user.username == username && user.password_hash == password {
            user_id = user.id
            break
        }
    }

    user_id
}
```

### src/main.at (AutoVM)

```auto
extern "rust" {
    // Crypto functions from Rust library
    fn sha256_hash(data str) str;
    fn aes_encrypt(plaintext str, key str) str;
    fn aes_decrypt(ciphertext str, key str) str;
    fn hmac_sign(message str, key str) str;
    fn hmac_verify(message str, signature str, key str) int;
}

use database::{Database, User}

type Server struct {
    db Database
    secret_key str
}

fn server_new() Server {
    let db = database_new()
    let key = "super_secret_key_12345"

    Server {
        db: db,
        secret_key: key
    }
}

fn (s Server) handle_register(username str, email str, password str) str {
    // Hash password with SHA-256
    let password_hash = sha256_hash(password)

    // Add user to database
    let user_id = s.db.add_user(username, email, password_hash)

    // Sign response
    let message = "User registered: " + username
    let signature = hmac_sign(message, s.secret_key)

    // Return signed response
    message + "|" + signature
}

fn (s Server) handle_login(username str, password str) str {
    // Hash password
    let password_hash = sha256_hash(password)

    // Authenticate
    let user_id = s.db.authenticate(username, password_hash)

    if user_id >= 0 {
        let user = s.db.get_user(user_id)
        let message = "Login successful: " + user.username
        let signature = hmac_sign(message, s.secret_key)
        message + "|" + signature
    } else {
        let message = "Login failed"
        let signature = hmac_sign(message, s.secret_key)
        message + "|" + signature
    }
}

fn (s Server) handle_encrypt(data str) str {
    aes_encrypt(data, s.secret_key)
}

fn (s Server) handle_decrypt(data str) str {
    aes_decrypt(data, s.secret_key)
}

fn main() {
    let server = server_new()

    // Register a test user
    let response = server.handle_register("alice", "alice@example.com", "password123")
    say("Register response: " + response)

    // Test login
    let login_response = server.handle_login("alice", "password123")
    say("Login response: " + login_response)

    // Test encryption
    let encrypted = server.handle_encrypt("Hello, World!")
    say("Encrypted: " + encrypted)

    let decrypted = server.handle_decrypt(encrypted)
    say("Decrypted: " + decrypted)
}
```

### Build Process

```bash
#!/bin/bash
# build.sh

# 1. Transpile crypto to Rust
auto trans_rust src/crypto.at -o target/crypto

# 2. Compile Rust library
cargo build --release --lib -p crypto

# 3. Compile database to AutoVM bytecode
auto build src/database.at -o target/database.bc

# 4. Compile main app to AutoVM bytecode
auto build src/main.at -o target/main.bc

echo "Build complete!"
```

## Example 3: GUI Application

### Project Structure

```
desktop_app/
├── pac.at
├── src/
│   ├── main.at         # Application logic (AutoVM)
│   ├── graphics.at     # Graphics engine (C transpilation)
│   └── ui.at           # UI framework (AutoVM)
└── target/
    ├── main.bc
    ├── graphics.c
    ├── graphics.h
    └── ui.bc
```

### pac.at

```auto
name: "desktop_app"
version: "1.0.0"
mode: "autovm"

app("desktop_app") {
    dependencies: [
        "std:core",
        "std:io",
        ("graphics", mode: "c"),  # Low-level graphics in C
        "ui",                     # UI framework in AutoVM
    ]
}
```

### src/graphics.at (C Transpilation)

```auto
// Graphics engine - transpiled to C

#[c]
fn window_init(width int, height int, title str) int {
    0
}

#[c]
fn window_close() int {
    0
}

#[c]
fn window_poll_events() int {
    0
}

#[c]
fn graphics_clear(r int, g int, b int) int {
    0
}

#[c]
fn graphics_present() int {
    0
}

#[c]
fn rectangle_draw(x int, y int, width int, height int, color int) int {
    0
}

#[c]
fn text_draw(x int, y int, text str, color int) int {
    0
}
```

### src/ui.at (AutoVM)

```auto
// UI framework - pure AutoVM

use std::core::{List, String}

type Button struct {
    x int
    y int
    width int
    height int
    text str
    onclick fn()
    enabled int
}

type ButtonState struct {
    hovered int
    pressed int
}

fn button_new(x int, y int, width int, height int, text str) Button {
    Button {
        x: x,
        y: y,
        width: width,
        height: height,
        text: text,
        onclick: fn() {},
        enabled: 1
    }
}

fn (b Button) draw() {
    let color = 0xCCCCCC  # Gray

    if b.enabled {
        color = 0x00FF00  # Green
    }

    rectangle_draw(b.x, b.y, b.width, b.height, color)
    text_draw(b.x + 10, b.y + 10, b.text, 0x000000)
}

fn (b Button) is_clicked(mouse_x int, mouse_y int) int {
    if mouse_x >= b.x && mouse_x <= b.x + b.width {
        if mouse_y >= b.y && mouse_y <= b.y + b.height {
            return 1
        }
    }
    0
}

type Window struct {
    title str
    width int
    height int
    buttons List<Button>
}

fn window_new(title str, width int, height int) Window {
    extern "c" {
        fn window_init(width int, height int, title str) int;
    }

    window_init(width, height, title);

    Window {
        title: title,
        width: width,
        height: height,
        buttons: List.new()
    }
}

fn (w Window) add_button(button Button) {
    w.buttons.push(button)
}

fn (w Window) run() {
    extern "c" {
        fn window_poll_events() int;
        fn graphics_clear(r int, g int, b int) int;
        fn graphics_present() int;
    }

    while window_poll_events() != 0 {
        graphics_clear(50, 50, 50)  # Dark gray background

        # Draw all buttons
        for i in 0..w.buttons.len() {
            let button = w.buttons.get(i)
            button.draw()
        }

        graphics_present()
    }
}
```

### src/main.at (AutoVM)

```auto
extern "c" {
    fn window_init(width int, height int, title str) int;
    fn window_close() int;
    fn window_poll_events() int;
    fn graphics_clear(r int, g int, b int) int;
    fn graphics_present() int;
    fn rectangle_draw(x int, y int, width int, height int, color int) int;
    fn text_draw(x int, y int, text str, color int) int;
}

use ui::{Window, Button, button_new}

fn on_click_button1() {
    say("Button 1 clicked!")
}

fn on_click_button2() {
    say("Button 2 clicked!")
}

fn main() {
    # Create window
    let window = window_new("My Application", 800, 600)

    # Create buttons
    let button1 = button_new(100, 100, 200, 50, "Click Me!")
    button1.onclick = on_click_button1

    let button2 = button_new(100, 200, 200, 50, "Click Me Too!")
    button2.onclick = on_click_button2

    # Add buttons to window
    window.add_button(button1)
    window.add_button(button2)

    # Run window
    window.run()

    # Cleanup
    window_close()
}
```

## Example 4: Pure AutoVM Project (Simplest)

### Project Structure

```
simple_app/
├── pac.at
├── src/
│   └── main.at
└── target/
    └── main.bc
```

### pac.at

```auto
name: "simple_app"
version: "1.0.0"
mode: "autovm"  # AutoVM is default, but explicit is good

app("simple_app") {
    dependencies: [
        "std:core",
        "std:io",
    ]
}
```

### src/main.at

```auto
fn main() {
    say("Hello, AutoVM!")

    let numbers = [1, 2, 3, 4, 5]
    let sum = 0

    for i in 0..len(numbers) {
        sum = sum + numbers[i]
    }

    say("Sum: " + sum)
}
```

### Build Process

```bash
#!/bin/bash
# build.sh

# Build to AutoVM bytecode
auto build src/main.at -o target/main.bc

# Run
auto run target/main.bc
```

## Summary

These examples demonstrate:

1. **Embedded Firmware**: HAL in C, main app in AutoVM
2. **Secure Server**: Crypto in Rust, app logic in AutoVM
3. **Desktop GUI**: Graphics engine in C, UI in AutoVM
4. **Simple App**: Everything in AutoVM

### Key Takeaways

- ✅ **Flexibility**: Mix and match modes as needed
- ✅ **Performance**: Use native code for critical paths
- ✅ **Productivity**: Use AutoVM for business logic
- ✅ **Integration**: FFI bridge connects everything

### Next Steps

1. Choose the mode for each part of your project
2. Create `pac.at` with mode specifications
3. Implement each module in its chosen mode
4. Use `extern "c"` or `extern "rust"` for cross-mode calls
5. Build and test!

---

**See Also**:
- [Mode Selection Guide](../guides/mode-selection-guide.md)
- [FFI Usage Guide](../guides/ffi-usage-guide.md)
- [Plan 081](../plans/081-autovm-default-mode.md)
