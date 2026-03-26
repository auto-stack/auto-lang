//! Database operations module
//!
//! Auto-generated from: examples/api-example/back/db.at
//!
//! This module provides:
//! - Global state management (users list, next_id counter)
//! - CRUD operations for users
//! - Search functionality

use crate::types::User;
use once_cell::sync::Lazy;
use std::sync::Mutex;

// ============================================================================
// Global State (simulated database)
// In production, this would use a real database connection
// ============================================================================

/// In-memory user storage
///
/// Auto source:
/// ```auto
/// var users List<User> = List<User>.new([
///     User { id: 1, name: "Alice", email: "alice@example.com" },
///     User { id: 2, name: "Bob", email: "bob@example.com" },
///     User { id: 3, name: "Charlie", email: "charlie@example.com" },
/// ])
/// ```
static USERS: Lazy<Mutex<Vec<User>>> = Lazy::new(|| {
    Mutex::new(vec![
        User {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        },
        User {
            id: 2,
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
        },
        User {
            id: 3,
            name: "Charlie".to_string(),
            email: "charlie@example.com".to_string(),
        },
    ])
});

/// Next available user ID
///
/// Auto source:
/// ```auto
/// var nextid int = 4
/// ```
static NEXT_ID: Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(4));

// ============================================================================
// Database Operations
// ============================================================================

/// Find user by ID
///
/// Auto source:
/// ```auto
/// pub fn find_user(id int) ?User {
///     for user in users {
///         if user.id == id {
///             return Some(user)
///         }
///     }
///     return None
/// }
/// ```
pub fn find_user(id: i32) -> Option<User> {
    let users = USERS.lock().unwrap();
    for user in users.iter() {
        if user.id == id {
            return Some(user.clone());
        }
    }
    None
}

/// Get all users
///
/// Auto source:
/// ```auto
/// pub fn all_users() []User {
///     return users.to_array()
/// }
/// ```
pub fn all_users() -> Vec<User> {
    let users = USERS.lock().unwrap();
    users.clone()
}

/// Create a new user
///
/// Auto source:
/// ```auto
/// pub fn create_user(name str, email str) User {
///     let user = User {
///         id: nextid,
///         name: name,
///         email: email,
///     }
///     nextid = nextid + 1
///     users.push(user)
///     return user
/// }
/// ```
pub fn create_user(name: String, email: String) -> User {
    let mut next_id = NEXT_ID.lock().unwrap();
    let id = *next_id;
    *next_id += 1;

    let user = User { id, name, email };

    let mut users = USERS.lock().unwrap();
    users.push(user.clone());

    user
}

/// Delete a user by ID
///
/// Auto source:
/// ```auto
/// pub fn delete_user(id int) bool {
///     var removed bool = false
///     users = users.filter((u User) => {
///         if u.id == id {
///             removed = true
///             return false
///         }
///         return true
///     })
///     return removed
/// }
/// ```
pub fn delete_user(id: i32) -> bool {
    let mut users = USERS.lock().unwrap();
    let original_len = users.len();
    users.retain(|u| u.id != id);
    users.len() < original_len
}

/// Search users by name or email
///
/// Auto source:
/// ```auto
/// pub fn search_users(query str) []User {
///     let query_lower str = query.to_lower()
///     let results List<User> = List<User>.new([])
///
///     for user in users {
///         if user.name.to_lower().contains(query_lower) ||
///            user.email.to_lower().contains(query_lower) {
///             results.push(user)
///         }
///     }
///
///     return results.to_array()
/// }
/// ```
pub fn search_users(query: String) -> Vec<User> {
    let query_lower = query.to_lowercase();
    let users = USERS.lock().unwrap();

    users
        .iter()
        .filter(|u| {
            u.name.to_lowercase().contains(&query_lower)
                || u.email.to_lowercase().contains(&query_lower)
        })
        .cloned()
        .collect()
}


// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_user() {
        let user = find_user(1);
        assert!(user.is_some());
        assert_eq!(user.unwrap().name, "Alice");

        let not_found = find_user(999);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_all_users() {
        let users = all_users();
        assert!(users.len() >= 3);
    }

    #[test]
    fn test_create_and_delete_user() {
        let new_user = create_user("Test".to_string(), "test@example.com".to_string());
        assert!(new_user.id >= 4);

        let deleted = delete_user(new_user.id);
        assert!(deleted);

        let deleted_again = delete_user(new_user.id);
        assert!(!deleted_again);
    }

    #[test]
    fn test_search_users() {
        let results = search_users("ali".to_string());
        assert!(!results.is_empty());
        assert!(results.iter().any(|u| u.name == "Alice"));

        let no_results = search_users("nonexistent".to_string());
        assert!(no_results.is_empty());
    }
}
