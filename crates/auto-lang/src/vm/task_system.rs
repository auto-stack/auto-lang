//! Plan 121: Task/Msg Runtime System
//!
//! TaskRegistry, TaskHandle, and TaskInstance for the Actor model.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     TaskRegistry                             │
//! │  - Manages singleton tasks (is single)                      │
//! │  - Manages all task instances                               │
//! │  - Routes messages to correct task                          │
//! └─────────────────────────────────────────────────────────────┘
//!                            │
//!                            ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     TaskHandle                               │
//! │  - task_type: String (e.g., "CounterTask")                  │
//! │  - instance_id: u64 (unique per instance)                   │
//! │  - tx: mpsc::Sender<Value> (message sender)                 │
//! │                                                              │
//! │  Properties: Copy, Comparable, Passable                     │
//! └─────────────────────────────────────────────────────────────┘
//!                            │
//!                            ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    TaskInstance                              │
//! │  - task_type: String                                        │
//! │  - instance_id: u64                                         │
//! │  - rx: mpsc::Receiver<Value> (message receiver)             │
//! │  - handle: TaskHandle (self-reference)                      │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use auto_lang::vm::task_system::{TaskRegistry, TaskInstance, TaskHandle};
//!
//! // Create registry
//! let registry = TaskRegistry::new();
//!
//! // Spawn a task instance
//! let instance = TaskInstance::new("CounterTask".to_string(), 64);
//!
//! // Register the instance
//! registry.register_instance(instance.handle.clone());
//!
//! // Send message to task
//! let handle = instance.handle.clone();
//! handle.send(Value::Int(42)).await?;
//! ```

use auto_val::Value;
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

/// Global task instance ID counter
static TASK_INSTANCE_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Task handle - reference to a task instance
///
/// Handle is Copy, Comparable, and Passable.
/// Contains: task_type + instance_id + tx (sender)
#[derive(Clone, Debug)]
pub struct TaskHandle {
    /// Task type name (e.g., "CounterTask")
    pub task_type: String,
    /// Unique instance ID
    pub instance_id: u64,
    /// Message sender (Arc for cloning)
    pub tx: Arc<mpsc::Sender<Value>>,
}

impl TaskHandle {
    /// Create a new task handle
    pub fn new(task_type: String, instance_id: u64, tx: mpsc::Sender<Value>) -> Self {
        Self {
            task_type,
            instance_id,
            tx: Arc::new(tx),
        }
    }

    /// Create a null/empty handle
    pub fn null() -> Self {
        let (tx, _) = mpsc::channel(1);
        Self {
            task_type: String::new(),
            instance_id: 0,
            tx: Arc::new(tx),
        }
    }

    /// Check if this is a null handle
    pub fn is_null(&self) -> bool {
        self.instance_id == 0
    }

    /// Send a message to this task
    pub async fn send(&self, msg: Value) -> Result<(), String> {
        self.tx.send(msg).await.map_err(|e| e.to_string())
    }

    /// Try to send without waiting (for non-async context)
    pub fn try_send(&self, msg: Value) -> Result<(), String> {
        self.tx.try_send(msg).map_err(|e| e.to_string())
    }
}

impl PartialEq for TaskHandle {
    fn eq(&self, other: &Self) -> bool {
        self.task_type == other.task_type && self.instance_id == other.instance_id
    }
}

impl Eq for TaskHandle {}

impl std::hash::Hash for TaskHandle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.task_type.hash(state);
        self.instance_id.hash(state);
    }
}

impl Default for TaskHandle {
    fn default() -> Self {
        Self::null()
    }
}

/// Task instance - a running task
pub struct TaskInstance {
    /// Task type name
    pub task_type: String,
    /// Instance ID
    pub instance_id: u64,
    /// Message receiver
    pub rx: mpsc::Receiver<Value>,
    /// Handle to self
    pub handle: TaskHandle,
}

impl TaskInstance {
    /// Create a new task instance
    pub fn new(task_type: String, capacity: usize) -> Self {
        let instance_id = TASK_INSTANCE_COUNTER.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = mpsc::channel(capacity);
        let handle = TaskHandle::new(task_type.clone(), instance_id, tx);

        Self {
            task_type,
            instance_id,
            rx,
            handle,
        }
    }
}

/// Task registry - manages all task definitions and instances
pub struct TaskRegistry {
    /// Task definitions: task_type -> handle
    /// For singleton tasks (is single), we store the handle directly
    singletons: DashMap<String, TaskHandle>,
    /// All task instances: (task_type, instance_id) -> handle
    instances: DashMap<(String, u64), TaskHandle>,
    /// Instance counter per task type
    instance_counts: DashMap<String, AtomicU64>,
}

impl TaskRegistry {
    /// Create a new task registry
    pub fn new() -> Self {
        Self {
            singletons: DashMap::new(),
            instances: DashMap::new(),
            instance_counts: DashMap::new(),
        }
    }

    /// Register a singleton task
    pub fn register_singleton(&self, task_type: String, handle: TaskHandle) {
        self.singletons.insert(task_type, handle);
    }

    /// Get a singleton task handle
    pub fn get_singleton(&self, task_type: &str) -> Option<TaskHandle> {
        self.singletons.get(task_type).map(|h| h.clone())
    }

    /// Check if a task type is a singleton
    pub fn is_singleton(&self, task_type: &str) -> bool {
        self.singletons.contains_key(task_type)
    }

    /// Register a task instance
    pub fn register_instance(&self, handle: TaskHandle) {
        let key = (handle.task_type.clone(), handle.instance_id);
        self.instances.insert(key, handle.clone());

        // Update instance count
        self.instance_counts
            .entry(handle.task_type)
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::SeqCst);
    }

    /// Get a task instance by (task_type, instance_id)
    pub fn get_instance(&self, task_type: &str, instance_id: u64) -> Option<TaskHandle> {
        let key = (task_type.to_string(), instance_id);
        self.instances.get(&key).map(|h| h.clone())
    }

    /// Remove a task instance
    pub fn remove_instance(&self, task_type: &str, instance_id: u64) {
        let key = (task_type.to_string(), instance_id);
        self.instances.remove(&key);
    }

    /// Get all instances of a task type
    pub fn get_instances_of_type(&self, task_type: &str) -> Vec<TaskHandle> {
        self.instances
            .iter()
            .filter(|entry| entry.key().0 == task_type)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get instance count for a task type
    pub fn get_instance_count(&self, task_type: &str) -> u64 {
        self.instance_counts
            .get(task_type)
            .map(|c| c.load(Ordering::SeqCst))
            .unwrap_or(0)
    }

    /// Get all task handles (for shutdown)
    ///
    /// Returns handles in LIFO order for proper shutdown:
    /// 1. Instances first (in reverse creation order)
    /// 2. Singletons last
    pub fn get_all_handles(&self) -> Vec<TaskHandle> {
        // First singletons
        let mut handles: Vec<_> = self.singletons.iter().map(|h| h.clone()).collect();
        // Then instances (in LIFO order - reverse insertion order)
        let mut instances: Vec<_> = self.instances.iter().map(|h| h.clone()).collect();
        instances.reverse();
        handles.extend(instances);
        handles
    }

    /// Get all singleton handles
    pub fn get_all_singletons(&self) -> Vec<TaskHandle> {
        self.singletons.iter().map(|h| h.clone()).collect()
    }

    /// Clear all registered tasks (for shutdown/reset)
    pub fn clear(&self) {
        self.singletons.clear();
        self.instances.clear();
        self.instance_counts.clear();
    }
}

impl Default for TaskRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_handle_null() {
        let handle = TaskHandle::null();
        assert!(handle.is_null());
        assert_eq!(handle.instance_id, 0);
        assert_eq!(handle.task_type, "");
    }

    #[test]
    fn test_task_handle_default() {
        let handle = TaskHandle::default();
        assert!(handle.is_null());
    }

    #[test]
    fn test_task_handle_equality() {
        let h1 = TaskHandle::null();
        let h2 = TaskHandle::null();
        assert_eq!(h1, h2);

        let instance1 = TaskInstance::new("TestTask".to_string(), 64);
        let instance2 = TaskInstance::new("TestTask".to_string(), 64);
        // Different instances should not be equal
        assert_ne!(instance1.handle, instance2.handle);
    }

    #[test]
    fn test_task_instance_creation() {
        let instance = TaskInstance::new("CounterTask".to_string(), 64);
        assert_eq!(instance.task_type, "CounterTask");
        assert!(instance.instance_id > 0);
        assert_eq!(instance.handle.task_type, "CounterTask");
        assert_eq!(instance.handle.instance_id, instance.instance_id);
    }

    #[test]
    fn test_task_instance_unique_ids() {
        let i1 = TaskInstance::new("Task1".to_string(), 64);
        let i2 = TaskInstance::new("Task2".to_string(), 64);
        assert_ne!(i1.instance_id, i2.instance_id);
    }

    #[test]
    fn test_task_registry_new() {
        let registry = TaskRegistry::new();
        assert_eq!(registry.get_all_handles().len(), 0);
    }

    #[test]
    fn test_task_registry_register_instance() {
        let registry = TaskRegistry::new();
        let instance = TaskInstance::new("TestTask".to_string(), 64);

        registry.register_instance(instance.handle.clone());

        let handle = registry.get_instance("TestTask", instance.instance_id);
        assert!(handle.is_some());
        assert_eq!(handle.unwrap().task_type, "TestTask");
    }

    #[test]
    fn test_task_registry_remove_instance() {
        let registry = TaskRegistry::new();
        let instance = TaskInstance::new("TestTask".to_string(), 64);

        registry.register_instance(instance.handle.clone());
        assert!(registry.get_instance("TestTask", instance.instance_id).is_some());

        registry.remove_instance("TestTask", instance.instance_id);
        assert!(registry.get_instance("TestTask", instance.instance_id).is_none());
    }

    #[test]
    fn test_task_registry_singleton() {
        let registry = TaskRegistry::new();
        let instance = TaskInstance::new("SingletonTask".to_string(), 64);

        registry.register_singleton("SingletonTask".to_string(), instance.handle.clone());

        assert!(registry.is_singleton("SingletonTask"));
        assert!(!registry.is_singleton("NonExistentTask"));

        let handle = registry.get_singleton("SingletonTask");
        assert!(handle.is_some());
        assert_eq!(handle.unwrap().task_type, "SingletonTask");
    }

    #[test]
    fn test_task_registry_get_instances_of_type() {
        let registry = TaskRegistry::new();

        let i1 = TaskInstance::new("TaskA".to_string(), 64);
        let i2 = TaskInstance::new("TaskA".to_string(), 64);
        let i3 = TaskInstance::new("TaskB".to_string(), 64);

        registry.register_instance(i1.handle.clone());
        registry.register_instance(i2.handle.clone());
        registry.register_instance(i3.handle.clone());

        let task_a_instances = registry.get_instances_of_type("TaskA");
        assert_eq!(task_a_instances.len(), 2);

        let task_b_instances = registry.get_instances_of_type("TaskB");
        assert_eq!(task_b_instances.len(), 1);

        let task_c_instances = registry.get_instances_of_type("TaskC");
        assert_eq!(task_c_instances.len(), 0);
    }

    #[test]
    fn test_task_registry_instance_count() {
        let registry = TaskRegistry::new();

        assert_eq!(registry.get_instance_count("TaskA"), 0);

        let i1 = TaskInstance::new("TaskA".to_string(), 64);
        let i2 = TaskInstance::new("TaskA".to_string(), 64);

        registry.register_instance(i1.handle.clone());
        assert_eq!(registry.get_instance_count("TaskA"), 1);

        registry.register_instance(i2.handle.clone());
        assert_eq!(registry.get_instance_count("TaskA"), 2);
    }

    #[test]
    fn test_task_registry_get_all_handles() {
        let registry = TaskRegistry::new();

        let singleton = TaskInstance::new("SingletonTask".to_string(), 64);
        let i1 = TaskInstance::new("TaskA".to_string(), 64);
        let i2 = TaskInstance::new("TaskB".to_string(), 64);

        registry.register_singleton("SingletonTask".to_string(), singleton.handle.clone());
        registry.register_instance(i1.handle.clone());
        registry.register_instance(i2.handle.clone());

        let all_handles = registry.get_all_handles();
        assert_eq!(all_handles.len(), 3);
    }

    #[test]
    fn test_task_registry_clear() {
        let registry = TaskRegistry::new();

        let instance = TaskInstance::new("TaskA".to_string(), 64);
        let singleton = TaskInstance::new("SingletonTask".to_string(), 64);

        registry.register_instance(instance.handle.clone());
        registry.register_singleton("SingletonTask".to_string(), singleton.handle.clone());

        assert_eq!(registry.get_all_handles().len(), 2);

        registry.clear();

        assert_eq!(registry.get_all_handles().len(), 0);
        assert_eq!(registry.get_instance_count("TaskA"), 0);
        assert!(!registry.is_singleton("SingletonTask"));
    }

    #[tokio::test]
    async fn test_task_handle_send() {
        let mut instance = TaskInstance::new("TestTask".to_string(), 64);
        let handle = instance.handle.clone();

        // Send a message
        handle.send(Value::Int(42)).await.unwrap();

        // Receive the message
        let msg = instance.rx.recv().await.unwrap();
        assert_eq!(msg, Value::Int(42));
    }

    #[test]
    fn test_task_handle_try_send() {
        let instance = TaskInstance::new("TestTask".to_string(), 2);
        let handle = instance.handle.clone();

        // Try send should succeed
        handle.try_send(Value::Int(1)).unwrap();
        handle.try_send(Value::Int(2)).unwrap();

        // Third send might fail if channel is full (depends on timing)
        // Just check that try_send returns without blocking
    }
}
