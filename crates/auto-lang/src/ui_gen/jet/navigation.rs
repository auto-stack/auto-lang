//! Navigation Component Generators
//!
//! Generates Jetpack Compose navigation code from AURA routes.
//!
//! ## Supported Features
//! - NavHost with composable routes
//! - Navigation controller setup
//! - Navigate actions
//! - Support for merged routes (Plan 114: Hybrid Routing)
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
                let params_str = route.params.join(", ");
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
    pub fn generate_navigate_with_arg(&self, route: &str, arg_name: &str, arg_value: &str) -> String {
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
/// - `/` -> `HomeScreen`
/// - `/about` -> `AboutScreen`
/// - `/user/:id` -> `UserScreen`
/// - `/admin/settings` -> `AdminSettingsScreen`
fn path_to_screen_name(path: &str) -> String {
    let segments: Vec<&str> = path
        .split('/')
        .filter(|s| !s.is_empty() && !s.starts_with(':'))
        .collect();

    if segments.is_empty() {
        return "HomeScreen".to_string();
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

    format!("{}Screen", name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_route() {
        let mut gen = NavigationGenerator::new();

        gen.add_route("home", "HomeScreen");
        gen.add_route("settings", "SettingsScreen");

        let routes = gen.get_routes();
        assert_eq!(routes.len(), 2);
        assert_eq!(routes[0].name, "home");
        assert_eq!(routes[0].screen, "HomeScreen");
    }

    #[test]
    fn test_add_route_with_params() {
        let mut gen = NavigationGenerator::new();

        gen.add_route_with_params(
            "detail",
            "DetailScreen",
            vec!["id".to_string()]
        );

        let routes = gen.get_routes();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].params.len(), 1);
    }

    #[test]
    fn test_generate_nav_host() {
        let mut gen = NavigationGenerator::new();

        gen.add_route("home", "HomeScreen");
        gen.add_route("settings", "SettingsScreen");

        let result = gen.generate_nav_host("home");
        assert!(result.is_ok());

        let code = result.unwrap();
        assert!(code.contains("NavHost"));
        assert!(code.contains("startDestination = \"home\""));
        assert!(code.contains("composable(\"home\")"));
        assert!(code.contains("composable(\"settings\")"));
        assert!(code.contains("HomeScreen(navController)"));
        assert!(code.contains("SettingsScreen(navController)"));
    }

    #[test]
    fn test_generate_app_with_nav() {
        let mut gen = NavigationGenerator::new();

        gen.add_route("home", "HomeScreen");

        let result = gen.generate_app_with_nav("home");
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

        gen.add_route("home", "HomeScreen");
        let _ = gen.generate_nav_host("home");

        let imports = gen.get_imports();
        assert!(imports.iter().any(|i| i.contains("NavHostController")));
        assert!(imports.iter().any(|i| i.contains("NavHost")));
    }

    #[test]
    fn test_clear_routes() {
        let mut gen = NavigationGenerator::new();

        gen.add_route("home", "HomeScreen");
        gen.add_route("settings", "SettingsScreen");
        assert_eq!(gen.get_routes().len(), 2);

        gen.clear_routes();
        assert_eq!(gen.get_routes().len(), 0);
    }

    #[test]
    fn test_clear_imports() {
        let mut gen = NavigationGenerator::new();

        gen.add_route("home", "HomeScreen");
        let _ = gen.generate_nav_host("home");
        assert!(!gen.get_imports().is_empty());

        gen.clear_imports();
        assert!(gen.get_imports().is_empty());
    }
}
