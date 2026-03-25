//! Navigation Component Generators
//!
//! Generates Jetpack Compose navigation code from AURA routes.
//!
//! ## Supported Features
//! - NavHost with composable routes
//! - Navigation controller setup
//! - Navigate actions
//! - Support for merged routes (Plan 114: Hybrid Routing)
//! - Tabs components (Plan 147, Task 1.4)
//!
//! ## Tabs Components
//!
//! The Tabs component family provides tabbed navigation:
//! - `generate_tabs()` - Complete tabs with state management
//! - `generate_tab_row()` - Just the TabRow (state managed externally)
//! - `generate_tab()` - Single tab component
//! - `generate_tabs_content()` - Content switcher
//!
//! ## Example
//!
//! ```kotlin
//! @Composable
//! fun AppNavHost(
//!     navController: NavHostController,
//!     modifier: Modifier = Modifier
//! ) {
//!     NavHost(
//!         navController = navController,
//!         startDestination = "home",
//!         modifier = modifier
//!     ) {
//!         composable("home") { HomeScreen(navController) }
//!         composable("settings") { SettingsScreen(navController) }
//!     }
//! }
//! ```

use crate::route::RouteDef;
use crate::ui_gen::GenResult;

/// Navigation route definition
#[derive(Debug, Clone)]
pub struct NavRoute {
    /// Route name/path
    pub name: String,
    /// Screen component name
    pub screen: String,
    /// Optional parameters
    pub params: Vec<String>,
}

/// Navigation generator
pub struct NavigationGenerator {
    /// Track imports needed for navigation
    imports: Vec<String>,
    /// Routes defined in the app
    routes: Vec<NavRoute>,
}

impl NavigationGenerator {
    /// Create a new navigation generator
    pub fn new() -> Self {
        Self {
            imports: Vec::new(),
            routes: Vec::new(),
        }
    }

    /// Get required imports for generated navigation components
    pub fn get_imports(&self) -> &[String] {
        &self.imports
    }

    /// Clear imports for fresh generation
    pub fn clear_imports(&mut self) {
        self.imports.clear();
    }

    /// Add import if not already present
    fn add_import(&mut self, import: &str) {
        if !self.imports.iter().any(|i| i == import) {
            self.imports.push(import.to_string());
        }
    }

    /// Add a route to the navigation
    pub fn add_route(&mut self, name: &str, screen: &str) {
        self.routes.push(NavRoute {
            name: name.to_string(),
            screen: screen.to_string(),
            params: Vec::new(),
        });
    }

    /// Add a route with parameters
    pub fn add_route_with_params(&mut self, name: &str, screen: &str, params: Vec<String>) {
        self.routes.push(NavRoute {
            name: name.to_string(),
            screen: screen.to_string(),
            params,
        });
    }

    /// Add a route from a merged RouteDef (Plan 114: Hybrid Routing)
    ///
    /// Converts the RouteDef to NavRoute format and adds it.
    pub fn add_route_from_def(&mut self, route: &RouteDef) {
        // Convert path to screen name: /user/:id -> UserScreen
        let screen = path_to_screen_name(&route.path);

        self.routes.push(NavRoute {
            name: route.path.clone(),
            screen,
            params: route.params.clone(),
        });
    }

    /// Add multiple routes from merged RouteDefs (Plan 114: Hybrid Routing)
    pub fn add_routes_from_defs(&mut self, routes: &[RouteDef]) {
        for route in routes {
            self.add_route_from_def(route);
        }
    }

    /// Add a route from an AuraRoute
    pub fn add_route_from_aura(&mut self, route: &crate::aura::AuraRoute) {
        let screen = path_to_screen_name(&route.path);

        self.routes.push(NavRoute {
            name: route.path.clone(),
            screen,
            params: route.params.clone(),
        });
    }

    /// Add multiple routes from AuraRoutes
    pub fn add_routes_from_aura(&mut self, routes: &[crate::aura::AuraRoute]) {
        for route in routes {
            self.add_route_from_aura(route);
        }
    }

    /// Clear all routes
    pub fn clear_routes(&mut self) {
        self.routes.clear();
    }

    /// Get all routes
    pub fn get_routes(&self) -> &[NavRoute] {
        &self.routes
    }

    /// Generate NavHost composable function
    pub fn generate_nav_host(&mut self, start_destination: &str) -> GenResult<String> {
        self.add_import("androidx.navigation.NavHostController");
        self.add_import("androidx.navigation.compose.NavHost");
        self.add_import("androidx.navigation.compose.composable");
        self.add_import("androidx.compose.runtime.Composable");
        self.add_import("androidx.compose.ui.Modifier");

        let mut route_composables = Vec::new();

        for route in &self.routes {
            if route.params.is_empty() {
                route_composables.push(format!(
                    "        composable(\"{}\") {{\n            {}(navController)\n        }}",
                    route.name, route.screen
                ));
            } else {
                // Route with parameters
                let _params_str = route.params.join(", ");
                route_composables.push(format!(
                    "        composable(\n            \"{}\"\n        ) {{ backStackEntry ->\n            {}(navController)\n        }}",
                    route.name, route.screen
                ));
            }
        }

        let routes_block = route_composables.join("\n");

        Ok(format!(
r#"@Composable
fun AppNavHost(
    navController: NavHostController,
    modifier: Modifier = Modifier
) {{
    NavHost(
        navController = navController,
        startDestination = "{}",
        modifier = modifier
    ) {{
{}
    }}
}}"#,
            start_destination, routes_block
        ))
    }

    /// Generate the main App composable with navigation
    pub fn generate_app_with_nav(&mut self, start_destination: &str) -> GenResult<String> {
        self.add_import("androidx.navigation.compose.rememberNavController");
        self.add_import("androidx.compose.material3.MaterialTheme");
        self.add_import("androidx.compose.material3.Surface");

        let nav_host = self.generate_nav_host(start_destination)?;

        Ok(format!(
r#"@Composable
fun App() {{
    val navController = rememberNavController()

    MaterialTheme {{
        Surface {{
            AppNavHost(navController)
        }}
    }}
}}

{}"#,
            nav_host
        ))
    }

    /// Generate a navigate call
    pub fn generate_navigate_call(&self, route: &str) -> String {
        format!("navController.navigate(\"{}\")", route)
    }

    /// Generate a navigate with argument call
    pub fn generate_navigate_with_arg(&self, route: &str, _arg_name: &str, arg_value: &str) -> String {
        format!(
            "navController.navigate(\"{}/{{$}}\".format({}))",
            route, arg_value
        )
    }

    /// Generate pop back stack call
    pub fn generate_pop_back_stack(&self) -> String {
        "navController.popBackStack()".to_string()
    }

    /// Generate navigate up call
    pub fn generate_navigate_up(&self) -> String {
        "navController.navigateUp()".to_string()
    }

    /// Generate a clickable modifier with navigation
    pub fn generate_clickable_navigate(&self, route: &str) -> String {
        format!("clickable {{ {} }}", self.generate_navigate_call(route))
    }

    /// Generate navigation imports
    pub fn generate_nav_imports(&self) -> String {
        let mut imports = vec![
            "import androidx.navigation.NavHostController".to_string(),
            "import androidx.navigation.compose.NavHost".to_string(),
            "import androidx.navigation.compose.composable".to_string(),
            "import androidx.navigation.compose.rememberNavController".to_string(),
        ];

        imports.extend(self.imports.clone());

        imports.sort();
        imports.dedup();

        imports.join("\n")
    }

    // =========================================================================
    // Tabs Components (Plan 147, Task 1.4)
    // =========================================================================

    /// Generate a complete Tabs component with state management
    ///
    /// # Arguments
    /// - `tab_ids`: List of tab identifiers (used for state and content matching)
    /// - `tab_labels`: List of tab display labels
    /// - `content_blocks`: List of content blocks for each tab
    ///
    /// # Returns
    /// Complete Kotlin code for a Tabs component with TabRow and content switching
    pub fn generate_tabs(&mut self, tab_ids: &[&str], tab_labels: &[&str], content_blocks: &[&str]) -> GenResult<String> {
        self.add_import("androidx.compose.material3.TabRow");
        self.add_import("androidx.compose.material3.Tab");
        self.add_import("androidx.compose.material3.Text");
        self.add_import("androidx.compose.foundation.layout.Column");
        self.add_import("androidx.compose.runtime.mutableStateOf");
        self.add_import("androidx.compose.runtime.remember");
        self.add_import("androidx.compose.runtime.getValue");
        self.add_import("androidx.compose.runtime.setValue");

        let tabs_list = tab_ids.iter().zip(tab_labels.iter()).enumerate()
            .map(|(i, (_id, label))| {
                format!(
                    r#"Tab(
            selected = activeTab == {},
            onClick = {{ activeTab = {} }},
            text = {{ Text("{}") }}
        )"#,
                    i, i, label
                )
            })
            .collect::<Vec<_>>()
            .join("\n        ");

        let content_switch = tab_ids.iter().enumerate()
            .map(|(i, _)| {
                let content = content_blocks.get(i).copied().unwrap_or("Text(\"Content\")");
                format!("{} -> {{\n            {}\n        }}", i, content)
            })
            .collect::<Vec<_>>()
            .join("\n        ");

        Ok(format!(
            r#"var activeTab by remember {{ mutableStateOf(0) }}

    Column {{
        TabRow(selectedTabIndex = activeTab) {{
            {}
        }}

        when (activeTab) {{
            {}
        }}
    }}"#,
            tabs_list, content_switch
        ))
    }

    /// Generate just the TabRow component (without state management)
    ///
    /// Use this when you want to manage the activeTab state yourself.
    pub fn generate_tab_row(&mut self, tab_labels: &[&str], active_index: usize) -> GenResult<String> {
        self.add_import("androidx.compose.material3.TabRow");
        self.add_import("androidx.compose.material3.Tab");
        self.add_import("androidx.compose.material3.Text");

        let tabs = tab_labels.iter().enumerate()
            .map(|(i, label)| {
                format!(
                    r#"Tab(
            selected = activeTab == {},
            onClick = {{ activeTab = {} }},
            text = {{ Text("{}") }}
        )"#,
                    i, i, label
                )
            })
            .collect::<Vec<_>>()
            .join("\n        ");

        Ok(format!(
            "TabRow(selectedTabIndex = {}) {{\n        {}\n    }}",
            active_index, tabs
        ))
    }

    /// Generate a single Tab component
    pub fn generate_tab(&mut self, label: &str, index: usize) -> GenResult<String> {
        self.add_import("androidx.compose.material3.Tab");
        self.add_import("androidx.compose.material3.Text");

        Ok(format!(
            r#"Tab(
        selected = activeTab == {},
        onClick = {{ activeTab = {} }},
        text = {{ Text("{}") }}
    )"#,
            index, index, label
        ))
    }

    /// Generate a TabsContent container with conditional rendering
    ///
    /// # Arguments
    /// - `active_tab`: The currently active tab index
    /// - `contents`: List of (tab_index, content) pairs
    pub fn generate_tabs_content(&mut self, contents: &[(usize, &str)]) -> GenResult<String> {
        let cases = contents.iter()
            .map(|(idx, content)| {
                format!("{} -> {{\n        {}\n    }}", idx, content)
            })
            .collect::<Vec<_>>()
            .join("\n    ");

        Ok(format!("when (activeTab) {{\n    {}\n}}", cases))
    }
}

impl Default for NavigationGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// =========================================================================
// Helper Functions
// =========================================================================

/// Convert a route path to a screen name
///
/// # Examples
/// - `/` -> `IndexPage`
/// - `/about` -> `AboutPage`
/// - `/user/:id` -> `UserPage`
/// - `/admin/settings` -> `AdminSettingsPage`
fn path_to_screen_name(path: &str) -> String {
    let segments: Vec<&str> = path
        .split('/')
        .filter(|s| !s.is_empty() && !s.starts_with(':'))
        .collect();

    if segments.is_empty() {
        return "IndexPage".to_string();
    }

    let name: String = segments
        .iter()
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect();

    format!("{}Page", name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_route() {
        let mut gen = NavigationGenerator::new();

        gen.add_route("/", "IndexPage");
        gen.add_route("/settings", "SettingsPage");

        let routes = gen.get_routes();
        assert_eq!(routes.len(), 2);
        assert_eq!(routes[0].name, "/");
        assert_eq!(routes[0].screen, "IndexPage");
    }

    #[test]
    fn test_add_route_with_params() {
        let mut gen = NavigationGenerator::new();

        gen.add_route_with_params(
            "detail",
            "DetailPage",
            vec!["id".to_string()]
        );

        let routes = gen.get_routes();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].params.len(), 1);
    }

    #[test]
    fn test_generate_nav_host() {
        let mut gen = NavigationGenerator::new();

        gen.add_route("/", "IndexPage");
        gen.add_route("/settings", "SettingsPage");

        let result = gen.generate_nav_host("/");
        assert!(result.is_ok());

        let code = result.unwrap();
        assert!(code.contains("NavHost"));
        assert!(code.contains("startDestination = \"/\""));
        assert!(code.contains("composable(\"/\")"));
        assert!(code.contains("composable(\"/settings\")"));
        assert!(code.contains("IndexPage(navController)"));
        assert!(code.contains("SettingsPage(navController)"));
    }

    #[test]
    fn test_generate_app_with_nav() {
        let mut gen = NavigationGenerator::new();

        gen.add_route("/", "IndexPage");

        let result = gen.generate_app_with_nav("/");
        assert!(result.is_ok());

        let code = result.unwrap();
        assert!(code.contains("@Composable"));
        assert!(code.contains("fun App()"));
        assert!(code.contains("rememberNavController()"));
        assert!(code.contains("MaterialTheme"));
    }

    #[test]
    fn test_generate_navigate_call() {
        let gen = NavigationGenerator::new();

        let call = gen.generate_navigate_call("settings");
        assert_eq!(call, "navController.navigate(\"settings\")");
    }

    #[test]
    fn test_generate_navigate_with_arg() {
        let gen = NavigationGenerator::new();

        let call = gen.generate_navigate_with_arg("detail", "id", "itemId");
        assert!(call.contains("navController.navigate"));
        assert!(call.contains("detail"));
    }

    #[test]
    fn test_generate_pop_back_stack() {
        let gen = NavigationGenerator::new();

        let call = gen.generate_pop_back_stack();
        assert_eq!(call, "navController.popBackStack()");
    }

    #[test]
    fn test_generate_navigate_up() {
        let gen = NavigationGenerator::new();

        let call = gen.generate_navigate_up();
        assert_eq!(call, "navController.navigateUp()");
    }

    #[test]
    fn test_generate_clickable_navigate() {
        let gen = NavigationGenerator::new();

        let modifier = gen.generate_clickable_navigate("settings");
        assert!(modifier.contains("clickable"));
        assert!(modifier.contains("navController.navigate(\"settings\")"));
    }

    #[test]
    fn test_generate_nav_imports() {
        let mut gen = NavigationGenerator::new();

        gen.add_route("home", "HomeScreen");
        let _ = gen.generate_nav_host("home");

        let imports = gen.generate_nav_imports();
        assert!(imports.contains("NavHostController"));
        assert!(imports.contains("rememberNavController"));
        assert!(imports.contains("composable"));
    }

    #[test]
    fn test_import_collection() {
        let mut gen = NavigationGenerator::new();

        gen.add_route("/", "IndexPage");
        let _ = gen.generate_nav_host("/");

        let imports = gen.get_imports();
        assert!(imports.iter().any(|i| i.contains("NavHostController")));
        assert!(imports.iter().any(|i| i.contains("NavHost")));
    }

    #[test]
    fn test_clear_routes() {
        let mut gen = NavigationGenerator::new();

        gen.add_route("/", "IndexPage");
        gen.add_route("/settings", "SettingsPage");
        assert_eq!(gen.get_routes().len(), 2);

        gen.clear_routes();
        assert_eq!(gen.get_routes().len(), 0);
    }

    #[test]
    fn test_clear_imports() {
        let mut gen = NavigationGenerator::new();

        gen.add_route("/", "IndexPage");
        let _ = gen.generate_nav_host("/");
        assert!(!gen.get_imports().is_empty());

        gen.clear_imports();
        assert!(gen.get_imports().is_empty());
    }

    // =========================================================================
    // Tabs Tests (Plan 147, Task 1.4)
    // =========================================================================

    #[test]
    fn test_generate_tabs_basic() {
        let mut gen = NavigationGenerator::new();

        let tab_ids = vec!["preview", "code", "notes"];
        let tab_labels = vec!["Preview", "Code", "Notes"];
        let contents = vec!["Text(\"Preview content\")", "Text(\"Code content\")", "Text(\"Notes content\")"];

        let result = gen.generate_tabs(&tab_ids, &tab_labels, &contents);
        assert!(result.is_ok());
        let code = result.unwrap();

        // Check structure
        assert!(code.contains("var activeTab by remember"));
        assert!(code.contains("mutableStateOf(0)"));
        assert!(code.contains("TabRow(selectedTabIndex = activeTab)"));
        assert!(code.contains("when (activeTab)"));
    }

    #[test]
    fn test_generate_tabs_with_labels() {
        let mut gen = NavigationGenerator::new();

        let tab_ids = vec!["tab1", "tab2"];
        let tab_labels = vec!["First Tab", "Second Tab"];
        let contents = vec!["Text(\"Content 1\")", "Text(\"Content 2\")"];

        let result = gen.generate_tabs(&tab_ids, &tab_labels, &contents);
        assert!(result.is_ok());
        let code = result.unwrap();

        // Check labels
        assert!(code.contains("Text(\"First Tab\")"));
        assert!(code.contains("Text(\"Second Tab\")"));
    }

    #[test]
    fn test_generate_tabs_content_switching() {
        let mut gen = NavigationGenerator::new();

        let tab_ids = vec!["a", "b", "c"];
        let tab_labels = vec!["A", "B", "C"];
        let contents = vec!["Text(\"A content\")", "Text(\"B content\")", "Text(\"C content\")"];

        let result = gen.generate_tabs(&tab_ids, &tab_labels, &contents);
        assert!(result.is_ok());
        let code = result.unwrap();

        // Check content switching
        assert!(code.contains("0 -> {"));
        assert!(code.contains("1 -> {"));
        assert!(code.contains("2 -> {"));
        assert!(code.contains("A content"));
        assert!(code.contains("B content"));
        assert!(code.contains("C content"));
    }

    #[test]
    fn test_generate_tabs_imports() {
        let mut gen = NavigationGenerator::new();

        let tab_ids = vec!["preview", "code"];
        let tab_labels = vec!["Preview", "Code"];
        let contents = vec!["Text(\"Preview\")", "Text(\"Code\")"];

        let _ = gen.generate_tabs(&tab_ids, &tab_labels, &contents);

        let imports = gen.get_imports();
        assert!(imports.iter().any(|i| i.contains("TabRow")));
        assert!(imports.iter().any(|i| i.contains("Tab")));
        assert!(imports.iter().any(|i| i.contains("mutableStateOf")));
        assert!(imports.iter().any(|i| i.contains("remember")));
    }

    #[test]
    fn test_generate_tab_row() {
        let mut gen = NavigationGenerator::new();

        let tab_labels = vec!["Home", "Settings", "Profile"];

        let result = gen.generate_tab_row(&tab_labels, 0);
        assert!(result.is_ok());
        let code = result.unwrap();

        assert!(code.contains("TabRow(selectedTabIndex = 0)"));
        assert!(code.contains("Text(\"Home\")"));
        assert!(code.contains("Text(\"Settings\")"));
        assert!(code.contains("Text(\"Profile\")"));
    }

    #[test]
    fn test_generate_single_tab() {
        let mut gen = NavigationGenerator::new();

        let result = gen.generate_tab("My Tab", 2);
        assert!(result.is_ok());
        let code = result.unwrap();

        assert!(code.contains("selected = activeTab == 2"));
        assert!(code.contains("onClick = { activeTab = 2 }"));
        assert!(code.contains("Text(\"My Tab\")"));
    }

    #[test]
    fn test_generate_tabs_content() {
        let mut gen = NavigationGenerator::new();

        let contents = vec![
            (0, "PreviewScreen()"),
            (1, "CodeScreen()"),
            (2, "NotesScreen()"),
        ];

        let result = gen.generate_tabs_content(&contents);
        assert!(result.is_ok());
        let code = result.unwrap();

        assert!(code.contains("when (activeTab)"));
        assert!(code.contains("0 -> {"));
        assert!(code.contains("PreviewScreen()"));
        assert!(code.contains("CodeScreen()"));
        assert!(code.contains("NotesScreen()"));
    }
}
