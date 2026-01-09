# Auto-Gen 教程：生成多种文本格式

本教程展示如何使用 `auto-gen` 生成各种文本格式，包括 HTML、XML、C 代码等。每个示例都在前一个基础上逐步增加复杂度。

## 目录

1. [示例 1：简单文本问候](#示例-1简单文本问候)
2. [示例 2：HTML 页面](#示例-2html-页面)
3. [示例 3：XML 配置](#示例-3xml-配置)
4. [示例 4：JSON 数据](#示例-4json-数据)
5. [示例 5：CSV 表格](#示例-5csv-表格)
6. [示例 6：Markdown 文档](#示例-6markdown-文档)
7. [示例 7：C 头文件](#示例-7c-头文件)
8. [示例 8：C 源文件](#示例-8c-源文件)
9. [示例 9：SQL 脚本](#示例-9sql-脚本)
10. [示例 10：完整的 C 模块](#示例-10完整的-c-模块)

---

## 示例 1：简单文本问候

**输出格式：** 纯文本

创建数据文件 `data/greeting.at`：
```auto
let name = "世界"
let count = 5
```

创建模板 `templates/greet.txt.at`：
```auto
你好，$name！

你是第 $count 位访问者。
```

生成输出：
```bash
autogen -d data/greeting.at -t templates/greet.txt.at -o output/
```

**输出** (`output/greet.txt`)：
```
你好，世界！

你是第 5 位访问者。
```

---

## 示例 2：HTML 页面

**输出格式：** HTML

创建数据文件 `data/person.at`：
```auto
let title = "张三"
let role = "软件工程师"
let email = "zhangsan@example.com"
let phone = "+86-138-0000-0000"
```

创建模板 `templates/profile.html.at`：
```auto
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>$title - 个人资料</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        h1 { color: #333; }
        .contact { color: #666; }
    </style>
</head>
<body>
    <h1>$title</h1>
    <p><strong>职位：</strong>$role</p>
    <div class="contact">
        <p><strong>邮箱：</strong>$email</p>
        <p><strong>电话：</strong>$phone</p>
    </div>
</body>
</html>
```

生成：
```bash
autogen -d data/person.at -t templates/profile.html.at -o output/
```

**输出** (`output/profile.html`)：
```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>张三 - 个人资料</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        h1 { color: #333; }
        .contact { color: #666; }
    </style>
</head>
<body>
    <h1>张三</h1>
    <p><strong>职位：</strong>软件工程师</p>
    <div class="contact">
        <p><strong>邮箱：</strong>zhangsan@example.com</p>
        <p><strong>电话：</strong>+86-138-0000-0000</p>
    </div>
</body>
</html>
```

---

## 示例 3：XML 配置

**输出格式：** XML

创建数据文件 `data/config.at`：
```auto
let app_name = "我的应用"
let version = "1.0.0"
let port = 8080
let debug_mode = true
```

创建模板 `templates/config.xml.at`：
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

生成：
```bash
autogen -d data/config.at -t templates/config.xml.at -o output/
```

**输出** (`output/config.xml`)：
```xml
<?xml version="1.0" encoding="UTF-8"?>
<configuration>
    <application>
        <name>我的应用</name>
        <version>1.0.0</version>
    </application>
    <server>
        <port>8080</port>
        <debug>true</debug>
    </server>
</configuration>
```

---

## 示例 4：JSON 数据

**输出格式：** JSON

创建数据文件 `data/user.at`：
```auto
let user_id = 1001
let username = "zhangsan"
let full_name = "张三"
let is_active = true
```

创建模板 `templates/user.json.at`：
```auto
{
    "id": $user_id,
    "username": "$username",
    "fullName": "$full_name",
    "isActive": $is_active
}
```

生成：
```bash
autogen -d data/user.at -t templates/user.json.at -o output/
```

**输出** (`output/user.json`)：
```json
{
    "id": 1001,
    "username": "zhangsan",
    "fullName": "张三",
    "isActive": true
}
```

---

## 示例 5：CSV 表格

**输出格式：** CSV（逗号分隔值）

创建数据文件 `data/products.at`：
```auto
let products = [
    { name: "产品 A", price: 10.50, stock: 100 }
    { name: "产品 B", price: 25.00, stock: 50 }
    { name: "产品 C", price: 7.25, stock: 200 }
]
```

创建模板 `templates/products.csv.at`：
```auto
名称,价格,库存
$for product in $products {
$($product.name),$($product.price),$($product.stock)
}
```

生成：
```bash
autogen -d data/products.at -t templates/products.csv.at -o output/
```

**输出** (`output/products.csv`)：
```csv
名称,价格,库存
产品 A,10.5,100
产品 B,25.0,50
产品 C,7.25,200
```

---

## 示例 6：Markdown 文档

**输出格式：** Markdown

创建数据文件 `data/article.at`：
```auto
let title = "Auto-Gen 简介"
let author = "李开发者"
let date = "2025-01-09"

let sections = [
    { heading: "什么是 Auto-Gen？", content: "Auto-Gen 是一个强大的代码生成器..." }
    { heading: "主要特性", content: "支持多种输出格式..." }
    { heading: "快速开始", content: "安装 auto-gen 并创建你的第一个模板..." }
]
```

创建模板 `templates/article.md.at`：
```auto
# $title

**作者：**$author  
**日期：**$date

---

$for section in $sections {
## $($section.heading)

$($section.content)

}
```

生成：
```bash
autogen -d data/article.at -t templates/article.md.at -o output/
```

**输出** (`output/article.md`)：
```markdown
# Auto-Gen 简介

**作者：**李开发者  
**日期：**2025-01-09

---

## 什么是 Auto-Gen？

Auto-Gen 是一个强大的代码生成器...

## 主要特性

支持多种输出格式...

## 快速开始

安装 auto-gen 并创建你的第一个模板...
```

---

## 示例 7：C 头文件

**输出格式：** C 头文件（.h 文件）

创建数据文件 `data/api.at`：
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

创建模板 `templates/sensor.h.at`：
```auto
/**
 * @file sensor.h
 * @brief 传感器接口 API
 */

#ifndef SENSOR_H
#define SENSOR_H

#include <stdint.h>

#define SENSOR_VERSION_MAJOR $version_major
#define SENSOR_VERSION_MINOR $version_minor

#ifdef __cplusplus
extern "C" {
#endif

$for func in $functions {
$($func.return) $($func.name)($for i, param in $func.params {$($param)if i < len($func.params) - 1 {, }});
}

#ifdef __cplusplus
}
#endif

#endif /* SENSOR_H */
```

生成：
```bash
autogen -d data/api.at -t templates/sensor.h.at -o output/
```

**输出** (`output/sensor.h`)：
```c
/**
 * @file sensor.h
 * @brief 传感器接口 API
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

## 示例 8：C 源文件

**输出格式：** C 源文件（.c 文件）

创建数据文件 `data/errors.at`：
```auto
let module = "network"

let errors = [
    { code: 1, name: "ERR_CONNECT_FAILED", message: "连接服务器失败" }
    { code: 2, name: "ERR_TIMEOUT", message: "连接超时" }
    { code: 3, name: "ERR_INVALID_RESPONSE", message: "无效的服务器响应" }
]
```

创建模板 `templates/errors.c.at`：
```auto
/**
 * @file errors.c
 * @brief 网络错误定义
 */

#include "errors.h"

$for err in $errors {
const char* $($err.name)_STR = "$($err.message)";

}

int get_error_message(int code, char* buffer, size_t size) {
    switch (code) {
$for err in $errors {
        case $($err.code):
            snprintf(buffer, size, "%s: %s", "$($err.name)", $($err.name)_STR);
            return 0;
}
        default:
            snprintf(buffer, size, "未知错误");
            return -1;
    }
}
```

生成：
```bash
autogen -d data/errors.at -t templates/errors.c.at -o output/
```

**输出** (`output/errors.c`)：
```c
/**
 * @file errors.c
 * @brief 网络错误定义
 */

#include "errors.h"

const char* ERR_CONNECT_FAILED_STR = "连接服务器失败";

const char* ERR_TIMEOUT_STR = "连接超时";

const char* ERR_INVALID_RESPONSE_STR = "无效的服务器响应";

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
            snprintf(buffer, size, "未知错误");
            return -1;
    }
}
```

---

## 示例 9：SQL 脚本

**输出格式：** SQL

创建数据文件 `data/schema.at`：
```auto
let table_name = "users"
let columns = [
    { name: "id", type: "SERIAL", constraints: ["PRIMARY KEY"] }
    { name: "username", type: "VARCHAR(50)", constraints: ["NOT NULL", "UNIQUE"] }
    { name: "email", type: "VARCHAR(100)", constraints: ["NOT NULL"] }
    { name: "created_at", type: "TIMESTAMP", constraints: ["DEFAULT CURRENT_TIMESTAMP"] }
]
```

创建模板 `templates/schema.sql.at`：
```auto
-- 表：$($table_name)
-- 由 Auto-Gen 生成

DROP TABLE IF EXISTS $($table_name);

CREATE TABLE $($table_name) (
    $for col in $columns {
    $($col.name) $($col.type) $($for constraint in $col.constraints {$($constraint)if $constraint != "PRIMARY KEY" { }})
    if col_index < len($columns) - 1 {,}
    }
);

-- 在 email 上创建索引
CREATE INDEX idx_$($table_name)_email ON $($table_name)(email);

-- 示例数据
INSERT INTO $($table_name) (username, email) VALUES
    ('admin', 'admin@example.com'),
    ('user1', 'user1@example.com');
```

生成：
```bash
autogen -d data/schema.at -t templates/schema.sql.at -o output/
```

**输出** (`output/schema.sql`)：
```sql
-- 表：users
-- 由 Auto-Gen 生成

DROP TABLE IF EXISTS users;

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(100) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 在 email 上创建索引
CREATE INDEX idx_users_email ON users(email);

-- 示例数据
INSERT INTO users (username, email) VALUES
    ('admin', 'admin@example.com'),
    ('user1', 'user1@example.com');
```

---

## 示例 10：完整的 C 模块（带保护块）

**输出格式：** C 头文件和源文件

创建数据文件 `data/diag.at`：
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

创建头文件模板 `templates/diag.h.at`：
```auto
/**
 * @file diag.h
 * @brief UDS 诊断服务定义
 */

#ifndef DIAG_H
#define DIAG_H

#include <stdint.h>

// 服务 ID
#define DIAG_SERVICE_SID 0x$($service.to_hex())

// 诊断服务 ID
$for diag in $diagnostics {
#define DIAG_$($diag.name.to_upper()) 0x$($diag.sid.to_hex())
}

// 函数声明
$for diag in $diagnostics {
const char* diag_get_$($diag.name.to_lower())_desc(void);
}

/**
 * @brief 获取诊断服务描述
 * @param sid 服务 ID
 * @return 描述字符串，如果未找到则返回 NULL
 */
const char* diag_get_service_desc(uint8_t sid);

#endif /* DIAG_H */
```

创建源文件模板 `templates/diag.c.at`：
```auto
/**
 * @file diag.c
 * @brief UDS 诊断服务实现
 */

#include "diag.h"
#include <string.h>

/// ---------- begin of guard: <custom_includes> ---
// 在此添加自定义头文件
/// ---------- end of guard: ---

// 服务描述
static const struct {
    uint8_t sid;
    const char* name;
    const char* desc;
} service_table[] = {
$for diag in $diagnostics {
    { 0x$($diag.sid.to_hex()), "$($diag.name)", "$($diag.desc)" }if diag_index < len($diagnostics) - 1 {,}
};

const char* diag_get_service_desc(uint8_t sid) {
    for (size_t i = 0; i < sizeof(service_table) / sizeof(service_table[0]); i++) {
        if (service_table[i].sid == sid) {
            return service_table[i].desc;
        }
    }
    return NULL;
}

$for diag in $diagnostics {
const char* diag_get_$($diag.name.to_lower())_desc(void) {
    return "$($diag.desc)";
}

}

/// ---------- begin of guard: <custom_functions> ---
// 在此添加自定义函数
// 重新生成时这些代码将被保留
/// ---------- end of guard: ---
```

生成：
```bash
autogen -d data/diag.at \
    -t templates/diag.h.at \
    -t templates/diag.c.at \
    -o output/
```

**输出** (`output/diag.h`)：
```c
/**
 * @file diag.h
 * @brief UDS 诊断服务定义
 */

#ifndef DIAG_H
#define DIAG_H

#include <stdint.h>

// 服务 ID
#define DIAG_SERVICE_SID 0x10

// 诊断服务 ID
#define DIAG_DIAGNOSTICSESSIONCONTROL 0x10
#define DIAG_ECURESET 0x11
#define DIAG_READDATABYIDENTIFIER 0x22
#define DIAG_READMEMORYBYADDRESS 0x23
#define DIAG_READSCALINGDATABYIDENTIFIER 0x24

// 函数声明
const char* diag_get_diagnostic_session_control_desc(void);
const char* diag_get_ecu_reset_desc(void);
const char* diag_get_read_data_by_identifier_desc(void);
const char* diag_get_read_memory_by_address_desc(void);
const char* diag_get_read_scaling_data_by_identifier_desc(void);

/**
 * @brief 获取诊断服务描述
 * @param sid 服务 ID
 * @return 描述字符串，如果未找到则返回 NULL
 */
const char* diag_get_service_desc(uint8_t sid);

#endif /* DIAG_H */
```

**输出** (`output/diag.c`)：
```c
/**
 * @file diag.c
 * @brief UDS 诊断服务实现
 */

#include "diag.h"
#include <string.h>

/// ---------- begin of guard: <custom_includes> ---
// 在此添加自定义头文件
/// ---------- end of guard: ---

// 服务描述
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
// 在此添加自定义函数
// 重新生成时这些代码将被保留
/// ---------- end of guard: ---
```

**核心特性：** 保护块允许你添加自定义代码，重新生成时这些代码将被保留。编辑 `diag.c` 文件并在保护标记之间添加代码：

```c
/// ---------- begin of guard: <custom_functions> ---
// 你的自定义实现
void custom_helper_function(void) {
    // 你的代码在这里
}
/// ---------- end of guard: ---
```

当你使用更新的数据重新生成时，你的自定义函数将被保留！

---

## 高级特性

### 使用库 API

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
println!("生成了 {} 个文件，耗时 {:?}", report.files_generated.len(), report.duration);
```

### 配置文件

创建 `autogen_config.at`：
```auto
let output_dir = "./generated"
let fstr_note = '$'
let overwrite_guarded = false
```

使用配置文件：
```bash
autogen --config autogen_config.at -d data/diag.at -t templates/diag.h.at
```

---

## 总结

本教程演示了：
1. ✅ 简单文本生成
2. ✅ HTML 页面生成
3. ✅ XML 配置文件
4. ✅ JSON 数据结构
5. ✅ 从数组生成 CSV 表格
6. ✅ Markdown 文档
7. ✅ 带宏的 C 头文件
8. ✅ 带函数的 C 源文件
9. ✅ SQL 模式脚本
10. ✅ 带保护块的完整 C 模块

关键要点：
- **数据文件**（`.at`）包含变量和数据结构
- **模板文件**（`.txt.at`）包含输出格式，使用 `$variable` 占位符
- **保护块**在重新生成时保留自定义代码
- **F-string 符号**默认使用 `$`（可通过 `-n` 参数配置）

更多信息请参阅设计文档 `docs/language/design/autogen.md`。
