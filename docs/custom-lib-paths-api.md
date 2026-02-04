# Custom Library Paths API

AutoLang 现在支持在多个级别自定义库文件搜索路径，让你可以灵活地组织和共享模块。

## 新增接口

### 1. Interpreter API

```rust
use auto_lang::interp::Interpreter;
use std::path::PathBuf;

let mut inter = Interpreter::new();

// 方法 1: 设置多个路径（替换现有路径）
inter.set_lib_paths(vec![
    PathBuf::from("./project/utils"),
    PathBuf::from("/usr/local/my_modules")
]);

// 方法 2: 添加单个路径
inter.add_lib_path("./shared/libs");
inter.add_lib_path("./company/templates");

// 方法 3: 获取当前路径
let paths = inter.lib_paths();
println!("Current search paths: {:?}", paths);

// 使用自定义路径导入模块
inter.interpret(r#"
    use myutil: helper
    say(helper())
"#)?;
```

**搜索顺序**：
1. 用户自定义的 `lib_paths`（优先级最高）
2. `~/.auto/libs/`
3. `/usr/local/lib/auto`
4. `/usr/lib/auto`
5. `.` (当前目录)

### 2. TemplateEngine API (auto-gen)

```rust
use auto_gen::template::TemplateEngine;
use std::path::PathBuf;

let mut engine = TemplateEngine::new();

// 添加自定义模板库路径
engine.add_lib_path("./templates/common");
engine.add_lib_path("./company/utils");

// 或者一次性设置多个路径
engine.set_lib_paths(vec![
    PathBuf::from("./generator/macros"),
    PathBuf::from("/usr/local/templates")
]);

// 渲染模板
let template = engine.load(&PathBuf::from("report.at"))?;
let output = engine.render_with_universe(&template, &universe)?;
```

### 3. DataLoader API (auto-gen)

```rust
use auto_gen::data::DataLoader;
use std::path::PathBuf;

let mut loader = DataLoader::new();

// 添加数据文件库路径
loader.add_lib_path("./data/models");
loader.add_lib_path("./schemas");

// 或者设置多个路径
loader.set_lib_paths(vec![
    PathBuf::from("./input/formats")
]);

let data = loader.load(DataSource::AutoFile("data.at".into()))?;
```

### 4. CodeGeneratorBuilder API (auto-gen)

```rust
use auto_gen::generator::{CodeGenerator, GenerationSpec};
use std::path::PathBuf;

// 方式 1: 使用 Builder（推荐）
let spec = CodeGenerator::builder()
    .data_source(DataSource::AutoFile("models.at".into()))
    .add_template("templates/report.at", "report.txt")
    .add_lib_path("./generator/utils")         // 添加单个路径
    .add_lib_path("./shared/macros")           // 继续添加
    .create_spec()?;

let mut generator = CodeGenerator::new(GeneratorConfig::default());
let report = generator.generate(&spec)?;

// 方式 2: 直接创建 GenerationSpec
let spec = GenerationSpec {
    data_source: DataSource::AutoFile("models.at".into()),
    templates: vec![/* ... */],
    lib_files: vec![],
    lib_paths: vec![                             // 设置多个路径
        PathBuf::from("./generator/utils"),
        PathBuf::from("./shared/macros")
    ],
};

let mut generator = CodeGenerator::new(GeneratorConfig::default());
let report = generator.generate(&spec)?;
```

## 使用场景

### 场景 1: 项目级别工具库

```
my_project/
├── src/
│   └── main.at              # 主程序
├── utils/                    # 项目工具库
│   └── helpers.at
└── templates/                # 模板库
    └── common.at
```

**main.at**:
```auto
use utils.helpers: format_data
use templates.common: header

fn main() {
    say(header())
    say(format_data(data))
}
```

**Rust 代码**:
```rust
let mut inter = Interpreter::new();
inter.add_lib_path("./utils");
inter.add_lib_path("./templates");
inter.load_file("src/main.at")?;
```

### 场景 2: 公司级代码生成

```
company_generator/
├── bin/
│   └── generate.rs          # 生成器程序
├── templates/
│   ├── api/
│   │   └── endpoint.at
│   └── models/
│       └── crud.at
├── lib/
│   └── company_utils.at
└── specs/
    └── user.json
```

**generate.rs**:
```rust
use auto_gen::generator::CodeGenerator;

fn main() -> GenResult<()> {
    let spec = CodeGenerator::builder()
        .data_source(DataSource::AutoFile("specs/user.json".into()))
        .add_template("templates/api/endpoint.at", "user_endpoint.rs")
        .add_template("templates/models/crud.at", "user_crud.rs")
        .add_lib_path("./lib")                 // 公司工具库
        .create_spec()?;

    let mut generator = CodeGenerator::new(GeneratorConfig::default());
    generator.generate(&spec)?;

    Ok(())
}
```

### 场景 3: 多项目共享模块

```
workspace/
├── shared/                   # 共享模块
│   ├── validators.at
│   └── formatters.at
├── project_a/
│   └── main.at
└── project_b/
    └── main.at
```

**project_a/main.at**:
```auto
use shared.validators: check_email
use shared.formatters: format_date

fn process() {
    if check_email(input) {
        say(format_email(input))
    }
}
```

**Rust 代码**:
```rust
// 在 workspace 根目录
let mut inter = Interpreter::new();
inter.add_lib_path("./shared");
inter.load_file("project_a/main.at")?;
```

### 场景 4: REPL 交互式开发

```rust
use auto_lang::interp::Interpreter;
use std::path::PathBuf;

let mut inter = Interpreter::new();

// 添加常用工具库路径
inter.add_lib_path("./devtools");
inter.add_lib_path("./experiments");

// 现在可以在 REPL 中直接使用这些模块
inter.interpret(r#"
    use devtools: debug
    use experiments: new_feature

    debug.log(new_feature.test())
"#)?;
```

## API 对比

### 设置 vs 添加

```rust
let mut inter = Interpreter::new();

// set_lib_paths() - 替换所有路径
inter.set_lib_paths(vec![PathBuf::from("./utils")]);
// 结果: ["./utils"]

// add_lib_path() - 追加到现有路径
inter.add_lib_path("./templates");
// 结果: ["./utils", "./templates"]

inter.add_lib_path("./lib");
// 结果: ["./utils", "./templates", "./lib"]
```

### 读取路径

```rust
let inter = Interpreter::new();
inter.add_lib_path("./utils");

// 获取路径的不可变引用
let paths: &[PathBuf] = inter.lib_paths();
println!("Search paths: {:?}", paths);
// 输出: Search paths: ["./utils"]
```

## 搜索优先级

当执行 `use mymodule: func` 时，搜索顺序为：

1. **用户自定义路径** (优先级最高)
   - 通过 `set_lib_paths()` 或 `add_lib_path()` 添加的路径
   - 按添加顺序搜索

2. **用户本地库**
   - `~/.auto/libs/`

3. **系统库**
   - `/usr/local/lib/auto`
   - `/usr/lib/auto`

4. **当前目录**
   - `.` (最低优先级)

### 优先级示例

```
目录结构:
~/.auto/libs/mymodule.at        # 版本 A (系统用户库)
/usr/lib/auto/mymodule.at       # 版本 B (系统库)
./project/libs/mymodule.at      # 版本 C (项目库)
./mymodule.at                   # 版本 D (当前目录)
```

```rust
let mut inter = Interpreter::new();
inter.add_lib_path("./project/libs");

inter.interpret("use mymodule: func")?;
// 会使用版本 C (project/libs/mymodule.at)
```

## 最佳实践

### 1. 项目结构

```
project/
├── lib/                      # 项目库（在 .gitignore 之外）
│   ├── utils.at
│   └── config.at
├── templates/                # 模板文件
│   └── ...
└── main.at
```

```rust
let mut inter = Interpreter::new();
inter.add_lib_path("./lib");  // 添加项目库
inter.load_file("main.at")?;
```

### 2. 共享工具链

```rust
// 创建公司级工具库
const COMPANY_LIBS: &str = "/usr/local/company/autolib";

let mut inter = Interpreter::new();
inter.add_lib_path(COMPANY_LIBS);  // 公司共享库
inter.add_lib_path("./project/lib"); // 项目库
```

### 3. 开发 vs 生产

```rust
use std::path::PathBuf;

let mut inter = Interpreter::new();

if cfg!(debug_assertions) {
    // 开发模式：使用本地实验性库
    inter.add_lib_path("./dev/experimental");
} else {
    // 生产模式：只使用稳定的库
    inter.add_lib_path("./lib/stable");
}

inter.add_lib_path("./shared");  // 共享库
```

## 完整示例

创建一个带有自定义库路径的代码生成器：

```rust
use auto_gen::generator::{CodeGenerator, GeneratorConfig, DataSource};
use std::path::PathBuf;

fn main() -> GenResult<()> {
    // 创建生成器配置
    let config = GeneratorConfig {
        output_dir: PathBuf::from("./output"),
        dry_run: false,
        fstr_note: '$',
        overwrite_guarded: false,
    };

    // 构建生成规范
    let spec = CodeGenerator::builder()
        // 数据源
        .data_source(DataSource::AutoFile("specs/models.json".into()))

        // 模板
        .add_template("templates/api.at", "api.rs")
        .add_template("templates/model.at", "model.rs")

        // 自定义库路径
        .add_lib_path("./generator/macros")      // 生成器宏
        .add_lib_path("./shared/validators")     // 共享验证器
        .add_lib_path("./company/formatters")    // 公司格式化工具

        .create_spec()?;

    // 运行生成器
    let mut generator = CodeGenerator::new(config);
    let report = generator.generate(&spec)?;

    println!("Generated {} files", report.files_generated.len());
    for error in &report.errors {
        eprintln!("Error: {}", error);
    }

    Ok(())
}
```

## 注意事项

1. **路径顺序很重要**：先添加的路径优先级更高
2. **相对路径**：相对于进程的工作目录解析
3. **路径验证**：不会自动检查路径是否存在，只在查找时检查
4. **线程安全**：`Interpreter` 和 `TemplateEngine` 不是 `Send` 或 `Sync`
5. **性能**：添加过多搜索路径会影响模块查找性能

## 相关文档

- [Plan 074: Multi-Directory Module Search](../plans/074-use-statement-multi-dir-search.md)
- [AutoGen Tutorial](../tutorials/autogen-tutorial.cn.md)
- [CLAUDE.md](../CLAUDE.md)
