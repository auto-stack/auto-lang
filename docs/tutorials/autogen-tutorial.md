# Auto-Gen Tutorial: Generating Multiple Text Formats

This tutorial shows how to use `auto-gen` to generate various text formats including HTML, XML, C code, and more. Each example builds upon the previous ones, increasing in complexity.

## How Auto-Gen Works

Auto-Gen uses a two-step process:

1. **Data Section**: Creates an Atom data structure (a superset of JSON)
   - Can use simple Atom format directly: `name: "World"`, `count: 42`
   - Can use arrays: `items: [1, 2, 3]`
   - Can use objects: `user: { name: "John", age: 30 }`
   - Can also use Auto script with `let`: `let name = "World"` (then reference it as `name` in Atom)

2. **Template Section**: Queries the Atom data and generates output
   - Code lines (start with `$ `): Control logic like loops and conditionals
   - Content lines: Template output with embedded variables

## Template Syntax Rules

Before diving into examples, it's important to understand how auto-gen templates work:

1. **Code Lines** (don't produce output): Lines starting with `$ ` (dollar sign followed by a space) are Auto code statements. These lines control the template logic but don't produce output themselves.
   - Example: `$ for item in items {` - this is a loop control statement
   - In these lines, use variable names directly: `item` not `$item`

2. **Content Lines** (produce output): All other lines are template content that goes into the final output.
   - Example: `Hello, $name!` - this line produces output with the variable embedded

3. **Variable Embedding**: In content lines, use `$` to embed variables or expressions:
   - Simple variable: `$name` or `${name}`
   - Field access: `${user.name}`
   - Complex expressions: `${items.length()}`

**Example Template:**
```
$ for user in users {
User: ${user.name}
$ }
```

Breaking this down:
- Line 1: `$ for ...` - code line (starts with `$ `), doesn't produce output
- Line 2: `User: ${user.name}` - content line, produces output with embedded field access
- Line 3: `$ }` - code line, closes the for loop

---

## Table of Contents

1. [Example 1: Simple Text Greeting](#example-1-simple-text-greeting)
2. [Example 2: HTML Page](#example-2-html-page)
3. [Example 3: XML Configuration](#example-3-xml-configuration)
4. [Example 4: JSON Data](#example-4-json-data)
5. [Example 5: CSV Table](#example-5-csv-table)
6. [Example 6: Markdown Document](#example-6-markdown-document)
7. [Example 7: C Header File](#example-7-c-header-file)
8. [Example 8: C Source File](#example-8-c-source-file)
9. [Example 9: SQL Script](#example-9-sql-script)
10. [Example 10: Complete C Module](#example-10-complete-c-module)

---

## Example 1: Simple Text Greeting

**Output Format:** Plain text

Create a data file `data/greeting.at`:
```auto
let name = "World"
let count = 5
```

Create a template `templates/greet.txt.at`:
```auto
Hello, $name!

You are visitor number $count.
```

Generate the output:
```bash
autogen -d data/greeting.at -t templates/greet.txt.at -o output/
```

**Output** (`output/greet.txt`):
```
Hello, World!

You are visitor number 5.
```

---

## Example 2: HTML Page

**Output Format:** HTML

Create data file `data/person.at`:
```auto
let title = "John Doe"
let role = "Software Engineer"
let email = "john@example.com"
let phone = "+1-555-0123"
```

Create template `templates/profile.html.at`:
```auto
<!DOCTYPE html>
<html>
<head>
    <title>$title - Profile</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        h1 { color: #333; }
        .contact { color: #666; }
    </style>
</head>
<body>
    <h1>$title</h1>
    <p><strong>Role:</strong> $role</p>
    <div class="contact">
        <p><strong>Email:</strong> $email</p>
        <p><strong>Phone:</strong> $phone</p>
    </div>
</body>
</html>
```

Generate:
```bash
autogen -d data/person.at -t templates/profile.html.at -o output/
```

**Output** (`output/profile.html`):
```html
<!DOCTYPE html>
<html>
<head>
    <title>John Doe - Profile</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        h1 { color: #333; }
        .contact { color: #666; }
    </style>
</head>
<body>
    <h1>John Doe</h1>
    <p><strong>Role:</strong> Software Engineer</p>
    <div class="contact">
        <p><strong>Email:</strong> john@example.com</p>
        <p><strong>Phone:</strong> +1-555-0123</p>
    </div>
</body>
</html>
```

---

## Example 3: XML Configuration

**Output Format:** XML

Create data file `data/config.at`:
```auto
let app_name = "MyApp"
let version = "1.0.0"
let port = 8080
let debug_mode = true
```

Create template `templates/config.xml.at`:
```auto
<?xml version="1.0" encoding="UTF-8"?>
<configuration>
    <application>
        <name>$app_name</name>
        <version>$version</version>
    </application>
    <server>
        <port>$port</port>
        <debug>$debug_mode</debug>
    </server>
</configuration>
```

Generate:
```bash
autogen -d data/config.at -t templates/config.xml.at -o output/
```

**Output** (`output/config.xml`):
```xml
<?xml version="1.0" encoding="UTF-8"?>
<configuration>
    <application>
        <name>MyApp</name>
        <version>1.0.0</version>
    </application>
    <server>
        <port>8080</port>
        <debug>true</debug>
    </server>
</configuration>
```

---

## Example 4: JSON Data

**Output Format:** JSON

Create data file `data/user.at`:
```auto
let user_id = 1001
let username = "jdoe"
let full_name = "John Doe"
let is_active = true
```

Create template `templates/user.json.at`:
```auto
{
    "id": $user_id,
    "username": "$username",
    "fullName": "$full_name",
    "isActive": $is_active
}
```

Generate:
```bash
autogen -d data/user.at -t templates/user.json.at -o output/
```

**Output** (`output/user.json`):
```json
{
    "id": 1001,
    "username": "jdoe",
    "fullName": "John Doe",
    "isActive": true
}
```

---

## Example 5: CSV Table

**Output Format:** CSV (Comma-Separated Values)

Create data file `data/products.at`:
```auto
let products = [
    { name: "Widget A", price: 10.50, stock: 100 }
    { name: "Widget B", price: 25.00, stock: 50 }
    { name: "Widget C", price: 7.25, stock: 200 }
]
```

Create template `templates/products.csv.at`:
```auto
Name,Price,Stock
$ for product in products {
${product.name},${product.price},${product.stock}
}
```

Generate:
```bash
autogen -d data/products.at -t templates/products.csv.at -o output/
```

**Output** (`output/products.csv`):
```csv
Name,Price,Stock
Widget A,10.5,100
Widget B,25.0,50
Widget C,7.25,200
```

---

## Example 6: Markdown Document

**Output Format:** Markdown

Create data file `data/article.at`:
```auto
let title = "Introduction to Auto-Gen"
let author = "Jane Developer"
let date = "2025-01-09"

let sections = [
    { heading: "What is Auto-Gen?", content: "Auto-Gen is a powerful code generator..." }
    { heading: "Key Features", content: "Supports multiple output formats..." }
    { heading: "Getting Started", content: "Install auto-gen and create your first template..." }
]
```

Create template `templates/article.md.at`:
```auto
# $title

**Author:** $author  
**Date:** $date

---

$ for section in sections {
## ${section.heading}

${section.content}

}
```

Generate:
```bash
autogen -d data/article.at -t templates/article.md.at -o output/
```

**Output** (`output/article.md`):
```markdown
# Introduction to Auto-Gen

**Author:** Jane Developer  
**Date:** 2025-01-09

---

## What is Auto-Gen?

Auto-Gen is a powerful code generator...

## Key Features

Supports multiple output formats...

## Getting Started

Install auto-gen and create your first template...
```

---

## Example 7: C Header File

**Output Format:** C Header (.h file)

Create data file `data/api.at`:
```auto
let module_name = "sensor"
let version_major = 1
let version_minor = 0

let functions = [
    { name: "sensor_init", return: "int", params: ["void* config"] }
    { name: "sensor_read", return: "float", params: ["int channel"] }
    { name: "sensor_close", return: "void", params: ["void"] }
]
```

Create template `templates/sensor.h.at`:
```auto
/**
 * @file sensor.h
 * @brief Sensor interface API
 */

#ifndef SENSOR_H
#define SENSOR_H

#include <stdint.h>

#define SENSOR_VERSION_MAJOR $version_major
#define SENSOR_VERSION_MINOR $version_minor

#ifdef __cplusplus
extern "C" {
#endif

$ for func in functions {
${func.return} ${func.name}($ for i, param in func.params {${param}if i < len(func.params) - 1 {, }});
}

#ifdef __cplusplus
}
#endif

#endif /* SENSOR_H */
```

Generate:
```bash
autogen -d data/api.at -t templates/sensor.h.at -o output/
```

**Output** (`output/sensor.h`):
```c
/**
 * @file sensor.h
 * @brief Sensor interface API
 */

#ifndef SENSOR_H
#define SENSOR_H

#include <stdint.h>

#define SENSOR_VERSION_MAJOR 1
#define SENSOR_VERSION_MINOR 0

#ifdef __cplusplus
extern "C" {
#endif

int sensor_init(void* config);
float sensor_read(int channel);
void sensor_close(void);

#ifdef __cplusplus
}
#endif

#endif /* SENSOR_H */
```

---

## Example 8: C Source File

**Output Format:** C Source (.c file)

Create data file `data/errors.at`:
```auto
let module = "network"

let errors = [
    { code: 1, name: "ERR_CONNECT_FAILED", message: "Failed to connect to server" }
    { code: 2, name: "ERR_TIMEOUT", message: "Connection timed out" }
    { code: 3, name: "ERR_INVALID_RESPONSE", message: "Invalid server response" }
]
```

Create template `templates/errors.c.at`:
```auto
/**
 * @file errors.c
 * @brief Network error definitions
 */

#include "errors.h"

$ for err in errors {
const char* ${err.name}_STR = "${err.message}";

}

int get_error_message(int code, char* buffer, size_t size) {
    switch (code) {
$ for err in errors {
        case ${err.code}:
            snprintf(buffer, size, "%s: %s", "${err.name}", ${err.name}_STR);
            return 0;
}
        default:
            snprintf(buffer, size, "Unknown error");
            return -1;
    }
}
```

Generate:
```bash
autogen -d data/errors.at -t templates/errors.c.at -o output/
```

**Output** (`output/errors.c`):
```c
/**
 * @file errors.c
 * @brief Network error definitions
 */

#include "errors.h"

const char* ERR_CONNECT_FAILED_STR = "Failed to connect to server";

const char* ERR_TIMEOUT_STR = "Connection timed out";

const char* ERR_INVALID_RESPONSE_STR = "Invalid server response";

int get_error_message(int code, char* buffer, size_t size) {
    switch (code) {
        case 1:
            snprintf(buffer, size, "%s: %s", "ERR_CONNECT_FAILED", ERR_CONNECT_FAILED_STR);
            return 0;
        case 2:
            snprintf(buffer, size, "%s: %s", "ERR_TIMEOUT", ERR_TIMEOUT_STR);
            return 0;
        case 3:
            snprintf(buffer, size, "%s: %s", "ERR_INVALID_RESPONSE", ERR_INVALID_RESPONSE_STR);
            return 0;
        default:
            snprintf(buffer, size, "Unknown error");
            return -1;
    }
}
```

---

## Example 9: SQL Script

**Output Format:** SQL

Create data file `data/schema.at`:
```auto
let table_name = "users"
let columns = [
    { name: "id", type: "SERIAL", constraints: ["PRIMARY KEY"] }
    { name: "username", type: "VARCHAR(50)", constraints: ["NOT NULL", "UNIQUE"] }
    { name: "email", type: "VARCHAR(100)", constraints: ["NOT NULL"] }
    { name: "created_at", type: "TIMESTAMP", constraints: ["DEFAULT CURRENT_TIMESTAMP"] }
]
```

Create template `templates/schema.sql.at`:
```auto
-- Table: ${table_name}
-- Generated by Auto-Gen

DROP TABLE IF EXISTS ${table_name};

CREATE TABLE ${table_name} (
    $ for col in columns {
    ${col.name} ${col.type} $ for constraint in col.constraints {${constraint}if constraint != "PRIMARY KEY" { }}
    if col_index < len(columns) - 1 {,}
    }
);

-- Create index on email
CREATE INDEX idx_${table_name}_email ON ${table_name}(email);

-- Sample data
INSERT INTO ${table_name} (username, email) VALUES
    ('admin', 'admin@example.com'),
    ('user1', 'user1@example.com');
```

Generate:
```bash
autogen -d data/schema.at -t templates/schema.sql.at -o output/
```

**Output** (`output/schema.sql`):
```sql
-- Table: users
-- Generated by Auto-Gen

DROP TABLE IF EXISTS users;

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(100) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create index on email
CREATE INDEX idx_users_email ON users(email);

-- Sample data
INSERT INTO users (username, email) VALUES
    ('admin', 'admin@example.com'),
    ('user1', 'user1@example.com');
```

---

## Example 10: Complete C Module with Guard Blocks

**Output Format:** C Header and Source files

Create data file `data/diag.at`:
```auto
let service = 0x10

let diagnostics = [
    { sid: 0x10, name: "DiagnosticSessionControl", desc: "诊断会话控制" }
    { sid: 0x11, name: "EcuReset", desc: "电控单元复位" }
    { sid: 0x22, name: "ReadDataByIdentifier", desc: "通过标识符读取数据" }
    { sid: 0x23, name: "ReadMemoryByAddress", desc: "通过地址读取内存" }
    { sid: 0x24, name: "ReadScalingDataByIdentifier", desc: "读取缩放数据" }
]
```

Create header template `templates/diag.h.at`:
```auto
/**
 * @file diag.h
 * @brief UDS Diagnostic Service Definitions
 */

#ifndef DIAG_H
#define DIAG_H

#include <stdint.h>

// Service ID
#define DIAG_SERVICE_SID 0x${service.to_hex()}

// Diagnostic Service IDs
$ for diag in diagnostics {
#define DIAG_${diag.name.to_upper()} 0x${diag.sid.to_hex()}
}

// Function declarations
$ for diag in diagnostics {
const char* diag_get_${diag.name.to_lower()}_desc(void);
}

/**
 * @brief Get diagnostic service description
 * @param sid Service ID
 * @return Description string, or NULL if not found
 */
const char* diag_get_service_desc(uint8_t sid);

#endif /* DIAG_H */
```

Create source template `templates/diag.c.at`:
```auto
/**
 * @file diag.c
 * @brief UDS Diagnostic Service Implementation
 */

#include "diag.h"
#include <string.h>

/// ---------- begin of guard: <custom_includes> ---
// Add your custom includes here
/// ---------- end of guard: ---

// Service descriptions
static const struct {
    uint8_t sid;
    const char* name;
    const char* desc;
} service_table[] = {
$ for diag in diagnostics {
    { 0x${diag.sid.to_hex()}, "${diag.name}", "${diag.desc}" } if diag_index < len(diagnostics) - 1 {,}
};

const char* diag_get_service_desc(uint8_t sid) {
    for (size_t i = 0; i < sizeof(service_table) / sizeof(service_table[0]); i++) {
        if (service_table[i].sid == sid) {
            return service_table[i].desc;
        }
    }
    return NULL;
}

$ for diag in diagnostics {
const char* diag_get_${diag.name.to_lower()}_desc(void) {
    return "${diag.desc}";
}

}

/// ---------- begin of guard: <custom_functions> ---
// Add your custom functions here
// These will be preserved when regenerating
/// ---------- end of guard: ---
```

Generate:
```bash
autogen -d data/diag.at \
    -t templates/diag.h.at \
    -t templates/diag.c.at \
    -o output/
```

**Output** (`output/diag.h`):
```c
/**
 * @file diag.h
 * @brief UDS Diagnostic Service Definitions
 */

#ifndef DIAG_H
#define DIAG_H

#include <stdint.h>

// Service ID
#define DIAG_SERVICE_SID 0x10

// Diagnostic Service IDs
#define DIAG_DIAGNOSTICSESSIONCONTROL 0x10
#define DIAG_ECURESET 0x11
#define DIAG_READDATABYIDENTIFIER 0x22
#define DIAG_READMEMORYBYADDRESS 0x23
#define DIAG_READSCALINGDATABYIDENTIFIER 0x24

// Function declarations
const char* diag_get_diagnostic_session_control_desc(void);
const char* diag_get_ecu_reset_desc(void);
const char* diag_get_read_data_by_identifier_desc(void);
const char* diag_get_read_memory_by_address_desc(void);
const char* diag_get_read_scaling_data_by_identifier_desc(void);

/**
 * @brief Get diagnostic service description
 * @param sid Service ID
 * @return Description string, or NULL if not found
 */
const char* diag_get_service_desc(uint8_t sid);

#endif /* DIAG_H */
```

**Output** (`output/diag.c`):
```c
/**
 * @file diag.c
 * @brief UDS Diagnostic Service Implementation
 */

#include "diag.h"
#include <string.h>

/// ---------- begin of guard: <custom_includes> ---
// Add your custom includes here
/// ---------- end of guard: ---

// Service descriptions
static const struct {
    uint8_t sid;
    const char* name;
    const char* desc;
} service_table[] = {
    { 0x10, "DiagnosticSessionControl", "诊断会话控制" },
    { 0x11, "EcuReset", "电控单元复位" },
    { 0x22, "ReadDataByIdentifier", "通过标识符读取数据" },
    { 0x23, "ReadMemoryByAddress", "通过地址读取内存" },
    { 0x24, "ReadScalingDataByIdentifier", "读取缩放数据" }
};

const char* diag_get_service_desc(uint8_t sid) {
    for (size_t i = 0; i < sizeof(service_table) / sizeof(service_table[0]); i++) {
        if (service_table[i].sid == sid) {
            return service_table[i].desc;
        }
    }
    return NULL;
}

const char* diag_get_diagnostic_session_control_desc(void) {
    return "诊断会话控制";
}

const char* diag_get_ecu_reset_desc(void) {
    return "电控单元复位";
}

const char* diag_get_read_data_by_identifier_desc(void) {
    return "通过标识符读取数据";
}

const char* diag_get_read_memory_by_address_desc(void) {
    return "通过地址读取内存";
}

const char* diag_get_read_scaling_data_by_identifier_desc(void) {
    return "读取缩放数据";
}

/// ---------- begin of guard: <custom_functions> ---
// Add your custom functions here
// These will be preserved when regenerating
/// ---------- end of guard: ---
```

**Key Feature:** Guard blocks allow you to add custom code that will be preserved when regenerating. Edit the `diag.c` file and add code between the guard markers:

```c
/// ---------- begin of guard: <custom_functions> ---
// Your custom implementation
void custom_helper_function(void) {
    // Your code here
}
/// ---------- end of guard: ---
```

When you regenerate with updated data, your custom function will be preserved!

---

## Advanced Features

### Using the Library API

```rust
use auto_gen::{CodeGenerator, GenerationSpec, GeneratorConfig, DataSource, TemplateSpec};

let config = GeneratorConfig {
    output_dir: "./output".into(),
    dry_run: false,
    fstr_note: '$',
    overwrite_guarded: false,
};

let mut generator = CodeGenerator::new(config);

let spec = GenerationSpec {
    data_source: DataSource::AutoFile("./data/diag.at".into()),
    templates: vec![
        TemplateSpec {
            template_path: "./templates/diag.h.at".into(),
            output_name: Some("diag.h".into()),
            rename: false,
        },
        TemplateSpec {
            template_path: "./templates/diag.c.at".into(),
            output_name: Some("diag.c".into()),
            rename: false,
        },
    ],
};

let report = generator.generate(&spec)?;
println!("Generated {} files in {:?}", report.files_generated.len(), report.duration);
```

### Configuration File

Create `autogen_config.at`:
```auto
let output_dir = "./generated"
let fstr_note = '$'
let overwrite_guarded = false
```

Use it:
```bash
autogen --config autogen_config.at -d data/diag.at -t templates/diag.h.at
```

---

## Summary

This tutorial demonstrated:
1. ✅ Simple text generation
2. ✅ HTML page generation
3. ✅ XML configuration files
4. ✅ JSON data structures
5. ✅ CSV tables from arrays
6. ✅ Markdown documents
7. ✅ C header files with macros
8. ✅ C source files with functions
9. ✅ SQL schema scripts
10. ✅ Complete C modules with guard blocks

The key takeaways:
- **Data files** (`.at`) contain your variables and data structures
- **Templates** (`.txt.at`) contain the output format with `$variable` placeholders
- **Guard blocks** preserve custom code between regenerations
- **F-string notation** uses `$` by default (configurable with `-n` flag)

For more information, see the design document at `docs/language/design/autogen.md`.
