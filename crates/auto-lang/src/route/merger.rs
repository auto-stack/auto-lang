//! Route Merger - Merge convention-based and config-based routes
//!
//! This module implements the merge strategy for combining routes from
//! different sources (file system discovery and config blocks).
//!
//! ## Merge Strategy
//!
//! | Scenario          | Result                              |
//! |-------------------|-------------------------------------|
//! | File only         | Use file-based route                |
//! | Config only       | Use config route                    |
//! | Both file + config| Config wins, merges extra props     |
//! | Duplicate paths   | Config wins                         |
//!
//! ## Usage
//!
//! ```rust
//! use auto_lang::route::{RouteMerger, RouteDef, RouteSource};
//!
//! let discovered = vec![
//!     RouteDef::new("/about", "about").with_source(RouteSource::Convention),
//! ];
//!
//! let config = vec![
//!     RouteDef::new("/admin", "admin").with_source(RouteSource::Config),
//! ];
//!
//! let merged = RouteMerger::merge(discovered, config);
//! assert_eq!(merged.len(), 2);
//! ```

use crate::route::{RouteDef, RouteSource};
use std::collections::HashMap;

/// Route merger for combining convention and config routes
///
/// Implements the merge strategy where config routes take precedence
/// over convention-based routes when paths match.
pub struct RouteMerger;

impl RouteMerger {
    /// Merge discovered routes with config routes
    ///
    /// # Merge Strategy
    ///
    /// 1. Start with all discovered (convention) routes
    /// 2. Add/override with config routes
    /// 3. When paths match, config wins and merges metadata
    ///
    /// # Arguments
    ///
    /// * `discovered` - Routes discovered from file system
    /// * `config` - Routes defined in config block
    ///
    /// # Returns
    ///
    /// Merged list of routes with no duplicate paths
    ///
    /// # Example
    ///
    /// ```rust
    /// use auto_lang::route::{RouteMerger, RouteDef, RouteSource};
    ///
    /// let discovered = vec![
    ///     RouteDef::new("/about", "about").with_source(RouteSource::Convention),
    ///     RouteDef::new("/admin", "admin_file").with_source(RouteSource::Convention),
    /// ];
    ///
    /// let config = vec![
    ///     RouteDef::new("/admin", "admin_config")
    ///         .with_source(RouteSource::Config)
    ///         .with_meta("auth", "true"),
    /// ];
    ///
    /// let merged = RouteMerger::merge(discovered, config);
    ///
    /// // Config route for /admin wins
    /// let admin_route = merged.iter().find(|r| r.path == "/admin").unwrap();
    /// assert_eq!(admin_route.module, "admin_config");
    /// assert_eq!(admin_route.source, RouteSource::Config);
    /// assert_eq!(admin_route.meta.get("auth"), Some(&"true".to_string()));
    /// ```
    pub fn merge(discovered: Vec<RouteDef>, config: Vec<RouteDef>) -> Vec<RouteDef> {
        let mut routes_by_path: HashMap<String, RouteDef> = HashMap::new();

        // Phase 1: Add all discovered routes
        for route in discovered {
            routes_by_path.insert(route.path.clone(), route);
        }

        // Phase 2: Merge config routes (config wins on conflicts)
        for route in config {
            let path = route.path.clone();

            if let Some(existing) = routes_by_path.get_mut(&path) {
                // Merge: config wins, but preserve metadata from both
                Self::merge_route_with(existing, route);
            } else {
                // New route from config
                routes_by_path.insert(path, route);
            }
        }

        // Convert to sorted vector for consistent output
        let mut routes: Vec<RouteDef> = routes_by_path.into_values().collect();
        routes.sort_by(|a, b| a.path.cmp(&b.path));

        routes
    }

    /// Merge a config route into an existing route
    ///
    /// Config route takes precedence, but metadata is merged.
    fn merge_route_with(existing: &mut RouteDef, config: RouteDef) {
        // Config wins for module and source
        existing.module = config.module;
        existing.source = RouteSource::Config;

        // Merge metadata (config values override)
        for (key, value) in config.meta {
            existing.meta.insert(key, value);
        }

        // Update params from config if they differ
        if !config.params.is_empty() {
            existing.params = config.params;
        }
    }

    /// Merge and return statistics
    ///
    /// Returns merged routes and statistics about the merge.
    pub fn merge_with_stats(
        discovered: Vec<RouteDef>,
        config: Vec<RouteDef>,
    ) -> (Vec<RouteDef>, MergeStats) {
        let stats = MergeStats {
            discovered_count: discovered.len(),
            config_count: config.len(),
            overridden_count: Self::count_overrides(&discovered, &config),
        };

        let merged = Self::merge(discovered, config);
        (merged, stats)
    }

    /// Count how many discovered routes were overridden by config
    fn count_overrides(discovered: &[RouteDef], config: &[RouteDef]) -> usize {
        let discovered_paths: std::collections::HashSet<_> =
            discovered.iter().map(|r| r.path.as_str()).collect();

        config
            .iter()
            .filter(|r| discovered_paths.contains(r.path.as_str()))
            .count()
    }

    /// Check for conflicting routes (same path, different modules)
    ///
    /// Returns a list of conflicts where the path matches but the module differs.
    pub fn find_conflicts(discovered: &[RouteDef], config: &[RouteDef]) -> Vec<RouteConflict> {
        let discovered_map: HashMap<&str, &RouteDef> = discovered
            .iter()
            .map(|r| (r.path.as_str(), r))
            .collect();

        let mut conflicts = Vec::new();

        for config_route in config {
            if let Some(discovered_route) = discovered_map.get(config_route.path.as_str()) {
                if discovered_route.module != config_route.module {
                    conflicts.push(RouteConflict {
                        path: config_route.path.clone(),
                        discovered_module: discovered_route.module.clone(),
                        config_module: config_route.module.clone(),
                    });
                }
            }
        }

        conflicts
    }

    /// Filter routes by source
    pub fn filter_by_source(routes: &[RouteDef], source: RouteSource) -> Vec<RouteDef> {
        routes
            .iter()
            .filter(|r| r.source == source)
            .cloned()
            .collect()
    }

    /// Get unique paths from both discovered and config routes
    pub fn get_all_paths(discovered: &[RouteDef], config: &[RouteDef]) -> Vec<String> {
        let mut paths: std::collections::HashSet<String> = std::collections::HashSet::new();

        for route in discovered {
            paths.insert(route.path.clone());
        }

        for route in config {
            paths.insert(route.path.clone());
        }

        let mut result: Vec<String> = paths.into_iter().collect();
        result.sort();
        result
    }
}

/// Statistics about route merge operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeStats {
    /// Number of discovered routes
    pub discovered_count: usize,
    /// Number of config routes
    pub config_count: usize,
    /// Number of discovered routes overridden by config
    pub overridden_count: usize,
}

/// Information about a route conflict
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteConflict {
    /// The conflicting path
    pub path: String,
    /// Module name from discovered route
    pub discovered_module: String,
    /// Module name from config route
    pub config_module: String,
}

impl std::fmt::Display for RouteConflict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Path '{}' has conflicting modules: '{}' (file) vs '{}' (config)",
            self.path, self.discovered_module, self.config_module
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_empty() {
        let merged = RouteMerger::merge(vec![], vec![]);
        assert!(merged.is_empty());
    }

    #[test]
    fn test_merge_discovered_only() {
        let discovered = vec![
            RouteDef::new("/about", "about").with_source(RouteSource::Convention),
            RouteDef::new("/contact", "contact").with_source(RouteSource::Convention),
        ];

        let merged = RouteMerger::merge(discovered, vec![]);
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn test_merge_config_only() {
        let config = vec![
            RouteDef::new("/about", "about").with_source(RouteSource::Config),
            RouteDef::new("/admin", "admin").with_source(RouteSource::Config),
        ];

        let merged = RouteMerger::merge(vec![], config);
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn test_merge_config_overrides() {
        let discovered = vec![
            RouteDef::new("/admin", "admin_file").with_source(RouteSource::Convention),
        ];

        let config = vec![
            RouteDef::new("/admin", "admin_config")
                .with_source(RouteSource::Config)
                .with_meta("auth", "true"),
        ];

        let merged = RouteMerger::merge(discovered, config);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].module, "admin_config");
        assert_eq!(merged[0].source, RouteSource::Config);
        assert_eq!(merged[0].meta.get("auth"), Some(&"true".to_string()));
    }

    #[test]
    fn test_merge_metadata_preserved() {
        let discovered = vec![RouteDef::new("/admin", "admin")
            .with_source(RouteSource::Convention)
            .with_meta("layout", "main")];

        let config = vec![RouteDef::new("/admin", "admin")
            .with_source(RouteSource::Config)
            .with_meta("auth", "true")];

        let merged = RouteMerger::merge(discovered, config);

        assert_eq!(merged.len(), 1);
        // Both metadata should be present
        assert_eq!(merged[0].meta.get("layout"), Some(&"main".to_string()));
        assert_eq!(merged[0].meta.get("auth"), Some(&"true".to_string()));
    }

    #[test]
    fn test_merge_no_conflict_different_paths() {
        let discovered = vec![
            RouteDef::new("/about", "about").with_source(RouteSource::Convention),
        ];

        let config = vec![
            RouteDef::new("/admin", "admin").with_source(RouteSource::Config),
        ];

        let merged = RouteMerger::merge(discovered, config);
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn test_merge_sorted_output() {
        let discovered = vec![
            RouteDef::new("/zebra", "zebra").with_source(RouteSource::Convention),
            RouteDef::new("/about", "about").with_source(RouteSource::Convention),
        ];

        let config = vec![
            RouteDef::new("/admin", "admin").with_source(RouteSource::Config),
        ];

        let merged = RouteMerger::merge(discovered, config);

        // Should be sorted alphabetically
        assert_eq!(merged[0].path, "/about");
        assert_eq!(merged[1].path, "/admin");
        assert_eq!(merged[2].path, "/zebra");
    }

    #[test]
    fn test_merge_with_stats() {
        let discovered = vec![
            RouteDef::new("/about", "about").with_source(RouteSource::Convention),
            RouteDef::new("/admin", "admin_file").with_source(RouteSource::Convention),
        ];

        let config = vec![
            RouteDef::new("/admin", "admin_config").with_source(RouteSource::Config),
            RouteDef::new("/settings", "settings").with_source(RouteSource::Config),
        ];

        let (merged, stats) = RouteMerger::merge_with_stats(discovered, config);

        assert_eq!(merged.len(), 3);
        assert_eq!(stats.discovered_count, 2);
        assert_eq!(stats.config_count, 2);
        assert_eq!(stats.overridden_count, 1); // /admin was overridden
    }

    #[test]
    fn test_find_conflicts() {
        let discovered = vec![
            RouteDef::new("/admin", "admin_file").with_source(RouteSource::Convention),
        ];

        let config = vec![
            RouteDef::new("/admin", "admin_config").with_source(RouteSource::Config),
        ];

        let conflicts = RouteMerger::find_conflicts(&discovered, &config);

        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].path, "/admin");
        assert_eq!(conflicts[0].discovered_module, "admin_file");
        assert_eq!(conflicts[0].config_module, "admin_config");
    }

    #[test]
    fn test_find_no_conflicts() {
        let discovered = vec![
            RouteDef::new("/about", "about").with_source(RouteSource::Convention),
        ];

        let config = vec![
            RouteDef::new("/admin", "admin").with_source(RouteSource::Config),
        ];

        let conflicts = RouteMerger::find_conflicts(&discovered, &config);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_find_conflicts_same_module() {
        // Same path and same module = no conflict
        let discovered = vec![
            RouteDef::new("/admin", "admin").with_source(RouteSource::Convention),
        ];

        let config = vec![
            RouteDef::new("/admin", "admin").with_source(RouteSource::Config),
        ];

        let conflicts = RouteMerger::find_conflicts(&discovered, &config);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_filter_by_source() {
        let routes = vec![
            RouteDef::new("/about", "about").with_source(RouteSource::Convention),
            RouteDef::new("/admin", "admin").with_source(RouteSource::Config),
            RouteDef::new("/contact", "contact").with_source(RouteSource::Convention),
        ];

        let convention = RouteMerger::filter_by_source(&routes, RouteSource::Convention);
        assert_eq!(convention.len(), 2);

        let config = RouteMerger::filter_by_source(&routes, RouteSource::Config);
        assert_eq!(config.len(), 1);
    }

    #[test]
    fn test_get_all_paths() {
        let discovered = vec![
            RouteDef::new("/about", "about").with_source(RouteSource::Convention),
        ];

        let config = vec![
            RouteDef::new("/admin", "admin").with_source(RouteSource::Config),
            RouteDef::new("/about", "about_override").with_source(RouteSource::Config),
        ];

        let paths = RouteMerger::get_all_paths(&discovered, &config);

        assert_eq!(paths.len(), 2); // /about and /admin
        assert!(paths.contains(&"/about".to_string()));
        assert!(paths.contains(&"/admin".to_string()));
    }

    #[test]
    fn test_route_conflict_display() {
        let conflict = RouteConflict {
            path: "/admin".to_string(),
            discovered_module: "admin_file".to_string(),
            config_module: "admin_config".to_string(),
        };

        let display = format!("{}", conflict);
        assert!(display.contains("/admin"));
        assert!(display.contains("admin_file"));
        assert!(display.contains("admin_config"));
    }

    #[test]
    fn test_merge_params_updated_from_config() {
        let discovered = vec![RouteDef::new("/user/:id", "user").with_source(RouteSource::Convention)];

        let config = vec![RouteDef::new("/user/:id", "user")
            .with_source(RouteSource::Config)];

        let merged = RouteMerger::merge(discovered, config);

        assert_eq!(merged.len(), 1);
        // Config params should be used
        assert_eq!(merged[0].params, vec!["id"]);
    }
}
