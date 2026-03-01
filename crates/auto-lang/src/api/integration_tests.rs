//! API Module Integration Tests
//!
//! Plan 102 Phase 5.4: End-to-end tests for API code generation

#[cfg(test)]
mod tests {
    use crate::api::{
        ApiAnnotationParser, ApiAttrs, ApiEndpoint, ApiExtractor, ApiField, ApiModule, ApiParam,
        ApiType, Target, TypeScriptGenerator, TauriGenerator, AxumGenerator,
    };
    use crate::api::targets::TargetGenerator;

    /// Create a test API module with sample endpoints
    fn create_test_api_module() -> ApiModule {
        let mut module = ApiModule::new("user_api".to_string());

        // Add type: User
        let user_type = ApiType {
            name: "User".to_string(),
            fields: vec![
                ApiField::new("id".to_string(), "int".to_string()),
                ApiField::new("name".to_string(), "str".to_string()),
                ApiField {
                    name: "email".to_string(),
                    ty: "str".to_string(),
                    optional: true,
                    default: None,
                },
            ],
            doc: Some("User information".to_string()),
        };
        module.add_type(user_type);

        // Add endpoint: get_user
        let get_user = ApiEndpoint {
            fn_name: "get_user".to_string(),
            attrs: ApiAnnotationParser::parse(r#"method = "GET", path = "/users/:id""#),
            params: vec![
                ApiParam::new("id".to_string(), "int".to_string()),
            ],
            return_type: "User".to_string(),
            doc: Some("Get user by ID".to_string()),
        };
        module.add_endpoint(get_user);

        // Add endpoint: list_users
        let list_users = ApiEndpoint {
            fn_name: "list_users".to_string(),
            attrs: ApiAttrs::new(), // Use inference
            params: vec![],
            return_type: "[]User".to_string(),
            doc: Some("List all users".to_string()),
        };
        module.add_endpoint(list_users);

        // Add endpoint: create_user
        let create_user = ApiEndpoint {
            fn_name: "create_user".to_string(),
            attrs: ApiAnnotationParser::parse(r#"method = "POST", path = "/users""#),
            params: vec![
                ApiParam::new("name".to_string(), "str".to_string()),
                ApiParam::new("email".to_string(), "str".to_string()),
            ],
            return_type: "User".to_string(),
            doc: Some("Create a new user".to_string()),
        };
        module.add_endpoint(create_user);

        // Add endpoint: delete_user
        let delete_user = ApiEndpoint {
            fn_name: "delete_user".to_string(),
            attrs: ApiAttrs::new(),
            params: vec![
                ApiParam::new("id".to_string(), "int".to_string()),
            ],
            return_type: "bool".to_string(),
            doc: Some("Delete a user".to_string()),
        };
        module.add_endpoint(delete_user);

        module
    }

    #[test]
    fn test_full_typescript_generation() {
        let module = create_test_api_module();
        let gen = TypeScriptGenerator::new();
        let output = gen.generate(&module);

        // Verify type definitions
        assert!(output.contains("export interface User"));
        assert!(output.contains("id: number"));
        assert!(output.contains("name: string"));
        assert!(output.contains("email?: string"));

        // Verify IApi interface
        assert!(output.contains("export interface IApi"));
        assert!(output.contains("getUser(id: number): Promise<User>"));
        assert!(output.contains("listUsers(): Promise<User[]>"));
        assert!(output.contains("createUser(name: string, email: string): Promise<User>"));
        assert!(output.contains("deleteUser(id: number): Promise<boolean>"));
    }

    #[test]
    fn test_full_tauri_generation() {
        let module = create_test_api_module();
        let gen = TauriGenerator::new();
        let output = gen.generate(&module);

        // Verify type definitions
        assert!(output.contains("pub struct User"));
        assert!(output.contains("pub id: i32"));
        assert!(output.contains("pub name: String"));

        // Verify commands
        assert!(output.contains("#[tauri::command]"));
        assert!(output.contains("pub fn get_user(id: i32) -> User"));
        assert!(output.contains("pub fn list_users() -> Vec<User>"));
        assert!(output.contains("pub fn create_user(name: String, email: String) -> User"));
        assert!(output.contains("pub fn delete_user(id: i32) -> bool"));

        // Verify registration
        assert!(output.contains("invoke_handler"));
        assert!(output.contains("get_user"));
        assert!(output.contains("list_users"));
    }

    #[test]
    fn test_full_axum_generation() {
        let module = create_test_api_module();
        let gen = AxumGenerator::new();
        let output = gen.generate(&module);

        // Verify handlers exist
        assert!(output.contains("get_user_handler"));
        assert!(output.contains("list_users_handler"));
        assert!(output.contains("create_user_handler"));
        assert!(output.contains("delete_user_handler"));

        // Verify router setup
        assert!(output.contains("Router::new()"));
        assert!(output.contains(".route(\"/users/:id\", get(get_user_handler))"));
        assert!(output.contains(".route(\"/users\", post(create_user_handler))"));
    }

    #[test]
    fn test_typescript_all_files() {
        let module = create_test_api_module();
        let gen = TypeScriptGenerator::new();
        let files = gen.generate_all(&module);

        // Verify all files are generated
        assert!(files.contains_key("types.ts"));
        assert!(files.contains_key("api-interface.ts"));
        assert!(files.contains_key("api-tauri.ts"));
        assert!(files.contains_key("api-http.ts"));
        assert!(files.contains_key("api.ts"));

        // Verify types.ts
        let types_ts = &files["types.ts"];
        assert!(types_ts.contains("export interface User"));

        // Verify api-tauri.ts
        let api_tauri = &files["api-tauri.ts"];
        assert!(api_tauri.contains("invoke<User>('get_user'"));
        assert!(api_tauri.contains("invoke<User[]>('list_users'"));

        // Verify api-http.ts
        let api_http = &files["api-http.ts"];
        // Check that HTTP methods are correct
        assert!(api_http.contains("axios.get"));  // get_user uses GET
        assert!(api_http.contains("axios.post")); // create_user uses POST
        assert!(api_http.contains("BASE_URL"));

        // Verify api.ts auto-detection
        let api_ts = &files["api.ts"];
        assert!(api_ts.contains("__TAURI__"));
        assert!(api_ts.contains("isTauri ? tauriApi : httpApi"));
    }

    #[test]
    fn test_method_inference() {
        let module = create_test_api_module();

        // get_user has explicit GET method
        let get_user = module.endpoints.iter().find(|e| e.fn_name == "get_user").unwrap();
        assert_eq!(get_user.method(), "GET");
        assert_eq!(get_user.path(), "/users/:id");

        // list_users uses inference - should be GET
        let list_users = module.endpoints.iter().find(|e| e.fn_name == "list_users").unwrap();
        assert_eq!(list_users.method(), "GET");
        assert_eq!(list_users.path(), "/users"); // list_users -> /users (plural)

        // create_user has explicit POST method
        let create_user = module.endpoints.iter().find(|e| e.fn_name == "create_user").unwrap();
        assert_eq!(create_user.method(), "POST");
        assert_eq!(create_user.path(), "/users");

        // delete_user uses inference - should be DELETE
        let delete_user = module.endpoints.iter().find(|e| e.fn_name == "delete_user").unwrap();
        assert_eq!(delete_user.method(), "DELETE");
    }

    #[test]
    fn test_frontend_name_generation() {
        let module = create_test_api_module();

        let get_user = module.endpoints.iter().find(|e| e.fn_name == "get_user").unwrap();
        assert_eq!(get_user.frontend_name(), "getUser");

        let list_users = module.endpoints.iter().find(|e| e.fn_name == "list_users").unwrap();
        assert_eq!(list_users.frontend_name(), "listUsers");

        let create_user = module.endpoints.iter().find(|e| e.fn_name == "create_user").unwrap();
        assert_eq!(create_user.frontend_name(), "createUser");

        let delete_user = module.endpoints.iter().find(|e| e.fn_name == "delete_user").unwrap();
        assert_eq!(delete_user.frontend_name(), "deleteUser");
    }

    #[test]
    fn test_complex_types() {
        let mut module = ApiModule::new("complex_api".to_string());

        // Add endpoint with complex types
        let endpoint = ApiEndpoint {
            fn_name: "search_users".to_string(),
            attrs: ApiAnnotationParser::parse(r#"method = "GET", path = "/users/search""#),
            params: vec![
                ApiParam::new("query".to_string(), "str".to_string()),
                ApiParam::new("limit".to_string(), "int".to_string()),
                ApiParam {
                    name: "offset".to_string(),
                    ty: "int".to_string(),
                    default: Some("0".to_string()),
                    optional: true,
                },
            ],
            return_type: "[]User".to_string(),
            doc: None,
        };
        module.add_endpoint(endpoint);

        // Test TypeScript generation with complex types
        let ts_gen = TypeScriptGenerator::new();
        let ts_output = ts_gen.generate(&module);
        assert!(ts_output.contains("searchUsers(query: string, limit: number, offset?: number): Promise<User[]>"));

        // Test Tauri generation with complex types
        let tauri_gen = TauriGenerator::new();
        let tauri_output = tauri_gen.generate(&module);
        assert!(tauri_output.contains("pub fn search_users(query: String, limit: i32, offset: Option<i32>) -> Vec<User>"));
    }

    #[test]
    fn test_target_trait() {
        let module = create_test_api_module();

        // Test each target via the trait
        let ts_target = Target::TypeScript.generator();
        assert_eq!(ts_target.extension(), ".ts");
        assert_eq!(ts_target.subdirectory(), "frontend");
        let ts_output = ts_target.generate(&module);
        assert!(!ts_output.is_empty());

        let tauri_target = Target::Tauri.generator();
        assert_eq!(tauri_target.extension(), ".rs");
        assert_eq!(tauri_target.subdirectory(), "tauri");
        let tauri_output = tauri_target.generate(&module);
        assert!(!tauri_output.is_empty());

        let axum_target = Target::Axum.generator();
        assert_eq!(axum_target.extension(), ".rs");
        assert_eq!(axum_target.subdirectory(), "web");
        let axum_output = axum_target.generate(&module);
        assert!(!axum_output.is_empty());
    }

    #[test]
    fn test_empty_module() {
        let module = ApiModule::new("empty_api".to_string());

        let ts_gen = TypeScriptGenerator::new();
        let ts_output = ts_gen.generate(&module);
        assert!(ts_output.contains("export interface IApi"));

        let tauri_gen = TauriGenerator::new();
        let tauri_output = tauri_gen.generate(&module);
        assert!(tauri_output.contains("register_commands"));
    }
}
