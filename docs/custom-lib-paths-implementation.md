# 自定义库路径 API 实现总结

## 修改的文件

### 1. `crates/auto-lang/src/eval.rs`

**新增方法**（在 Evaler 中）：
```rust
pub fn add_lib_path(&mut self, path: std::path::PathBuf) {
    self.lib_paths.push(path);
}

pub fn lib_paths(&self) -> &[std::path::PathBuf] {
    &self.lib_paths
}
```

**位置**: 第 143-157 行（在 `set_lib_paths()` 之后）

---

### 2. `crates/auto-lang/src/interp.rs`

**新增方法**（在 Interpreter 中）：
```rust
pub fn add_lib_path(&mut self, path: impl Into<std::path::PathBuf>) {
    let path = path.into();
    self.lib_paths.push(path.clone());
    self.evaler.add_lib_path(path);
}

pub fn lib_paths(&self) -> &[std::path::PathBuf] {
    &self.lib_paths
}
```

**位置**: 第 488-508 行（在 `set_lib_paths()` 之后）

**特点**:
- `add_lib_path()` 接受 `impl Into<PathBuf>`，更灵活
- 同时更新 Interpreter 和 Evaler 的路径
- `lib_paths()` 返回不可变引用

---

### 3. `crates/auto-gen/src/template.rs`

**新增方法**（在 TemplateEngine 中）：
```rust
pub fn add_lib_path(&mut self, path: impl Into<PathBuf>) {
    self.lib_paths.push(path.into());
}

pub fn lib_paths(&self) -> &[PathBuf] {
    &self.lib_paths
}
```

**位置**: 第 40-52 行（在 `set_lib_paths()` 之后）

---

### 4. `crates/auto-gen/src/data.rs`

**新增方法**（在 DataLoader 中）：
```rust
pub fn add_lib_path(&mut self, path: impl Into<PathBuf>) {
    self.lib_paths.push(path.into());
}

pub fn lib_paths(&self) -> &[PathBuf] {
    &self.lib_paths
}
```

**位置**: 第 37-49 行（在 `set_lib_paths()` 之后）

---

### 5. `crates/auto-gen/src/generator.rs`

**修改的结构体**：
```rust
pub struct CodeGeneratorBuilder {
    config: GeneratorConfig,
    data_source: Option<DataSource>,
    templates: Vec<TemplateSpec>,
    lib_paths: Vec<PathBuf>,  // 新增字段
}
```

**新增方法**（在 CodeGeneratorBuilder 中）：
```rust
pub fn lib_paths(mut self, paths: Vec<PathBuf>) -> Self {
    self.lib_paths = paths;
    self
}

pub fn add_lib_path(mut self, path: impl Into<PathBuf>) -> Self {
    self.lib_paths.push(path.into());
    self
}

pub fn create_spec(self) -> GenResult<GenerationSpec> {
    let data_source = self.data_source.ok_or_else(|| {
        GenError::Other("data_source is required to create a GenerationSpec".to_string())
    })?;

    Ok(GenerationSpec {
        data_source,
        templates: self.templates,
        lib_files: Vec::new(),
        lib_paths: self.lib_paths,  // 使用 Builder 中的 lib_paths
    })
}
```

**位置**:
- 字段添加：第 206 行
- `lib_paths()`: 第 237-242 行
- `add_lib_path()`: 第 244-251 行
- `create_spec()`: 第 258-281 行

---

## API 设计模式

### 1. **Builder 模式** (CodeGeneratorBuilder)
- 支持链式调用
- `lib_paths()` 设置多个路径（替换）
- `add_lib_path()` 添加单个路径（追加）
- `create_spec()` 创建 GenerationSpec

### 2. **可变借用模式** (Interpreter, TemplateEngine, DataLoader)
- `set_lib_paths()` 替换所有路径
- `add_lib_path()` 追加单个路径
- `lib_paths()` 读取路径（不可变借用）

### 3. **灵活的参数类型**
- `add_lib_path()` 接受 `impl Into<PathBuf>`
- 可以传递 `&str`, `String`, `PathBuf` 等

---

## 使用示例

### Interpreter
```rust
let mut inter = Interpreter::new();

// 添加单个路径
inter.add_lib_path("./utils");
inter.add_lib_path("./templates");

// 读取路径
let paths = inter.lib_paths();
assert_eq!(paths.len(), 2);

// 设置多个路径（替换）
inter.set_lib_paths(vec![PathBuf::from("./new")]);
```

### TemplateEngine
```rust
let mut engine = TemplateEngine::new();

engine.add_lib_path("./templates");
engine.add_lib_path("./macros");
```

### DataLoader
```rust
let mut loader = DataLoader::new();

loader.add_lib_path("./data");
loader.add_lib_path("./schemas");
```

### CodeGeneratorBuilder
```rust
let spec = CodeGenerator::builder()
    .data_source(source)
    .add_template("t.at", "out.txt")
    .add_lib_path("./utils")          // 添加单个路径
    .add_lib_path("./macros")          // 继续添加
    .create_spec()?;
```

---

## 搜索优先级

完整搜索顺序（从高到低）：

1. **用户自定义 lib_paths** ← 新接口控制的路径
2. `~/.auto/libs/`
3. `/usr/local/lib/auto`
4. `/usr/lib/auto`
5. `.` (当前目录)

---

## 文档

详细使用文档：
- [docs/custom-lib-paths-api.md](custom-lib-paths-api.md)

相关计划：
- [docs/plans/074-use-statement-multi-dir-search.md](plans/074-use-statement-multi-dir-search.md)

---

## 测试文件

测试示例（位于 `tmp/test_custom_lib_paths/`）：
- `test_interpreter.rs` - Interpreter API 测试
- `test_template_engine.rs` - TemplateEngine API 测试
- `test_data_loader.rs` - DataLoader API 测试

---

## 向后兼容性

✅ **完全向后兼容**
- 所有现有 API 保持不变
- `set_lib_paths()` 仍然可用
- 新接口是增量添加，不破坏现有代码

---

## 未来改进

可能的增强功能：

1. **路径验证**
   ```rust
   pub fn add_lib_path_verified(&mut self, path: PathBuf) -> AutoResult<()> {
       if !path.exists() {
           return Err(format!("Path does not exist: {}", path.display()).into());
       }
       self.lib_paths.push(path);
       Ok(())
   }
   ```

2. **路径优先级**
   ```rust
   pub fn add_lib_path_first(&mut self, path: PathBuf) {
       self.lib_paths.insert(0, path);
   }
   ```

3. **路径别名**
   ```rust
   pub fn add_lib_alias(&mut self, alias: &str, path: PathBuf) {
       // 添加路径别名，如 "stdlib" -> "/usr/lib/auto"
   }
   ```

4. **环境变量支持**
   ```rust
   pub fn add_lib_paths_from_env(&mut self, var: &str) {
       // 从环境变量读取路径，如 "AUTO_LIB_PATH"
   }
   ```

---

## 实现日期

**2025-02-04**

实现者：Claude (Anthropic)

状态：✅ 完成，待测试
