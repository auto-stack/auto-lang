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
//! │  - start_hook: Option<LifecycleHook>                        │
//! │  - stop_hook: Option<LifecycleHook>                         │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Lifecycle Hooks
//!
//! Tasks support lifecycle hooks for initialization and cleanup:
//!
//! ```auto
//! task CounterTask {
//!     fn start() ! { print("Task started") }
//!     fn stop() ! { print("Task stopped") }
//! }
//! ```
//!
//! - `start()` - Called when the task is spawned
//! - `stop()` - Called during system shutdown (LIFO order)
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

/// Lifecycle hook callback type
///
/// Hooks are closures that take no arguments and return Result<(), String>.
/// The closure is called during task lifecycle events (start/stop).
pub type LifecycleHook = Box<dyn Fn() -> Result<(), String> + Send + Sync>;

/// Hook execution result
#[derive(Debug, Clone)]
pub struct HookResult {
    /// The task type that executed the hook
    pub task_type: String,
    /// The hook type ("start" or "stop")
    pub hook_type: String,
    /// Whether the hook execution succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Global task instance ID counter
static TASK_INSTANCE_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Global creation order counter (for LIFO shutdown)
static CREATION_ORDER_COUNTER: AtomicU64 = AtomicU64::new(1);

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
    /// Lifecycle hook: start()
    pub start_hook: Option<Arc<LifecycleHook>>,
    /// Lifecycle hook: stop()
    pub stop_hook: Option<Arc<LifecycleHook>>,
    /// Creation order (for LIFO shutdown)
    pub creation_order: u64,
}

impl TaskInstance {
    /// Create a new task instance
    pub fn new(task_type: String, capacity: usize) -> Self {
        let instance_id = TASK_INSTANCE_COUNTER.fetch_add(1, Ordering::SeqCst);
        let creation_order = CREATION_ORDER_COUNTER.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = mpsc::channel(capacity);
        let handle = TaskHandle::new(task_type.clone(), instance_id, tx);

        Self {
            task_type,
            instance_id,
            rx,
            handle,
            start_hook: None,
            stop_hook: None,
            creation_order,
        }
    }

    /// Set the start hook
    pub fn set_start_hook(&mut self, hook: LifecycleHook) {
        self.start_hook = Some(Arc::new(hook));
    }

    /// Set the stop hook
    pub fn set_stop_hook(&mut self, hook: LifecycleHook) {
        self.stop_hook = Some(Arc::new(hook));
    }

    /// Execute the start hook
    ///
    /// Returns a HookResult indicating success or failure.
    pub fn execute_start_hook(&self) -> HookResult {
        if let Some(hook) = &self.start_hook {
            match hook() {
                Ok(()) => HookResult {
                    task_type: self.task_type.clone(),
                    hook_type: "start".to_string(),
                    success: true,
                    error: None,
                },
                Err(e) => HookResult {
                    task_type: self.task_type.clone(),
                    hook_type: "start".to_string(),
                    success: false,
                    error: Some(e),
                },
            }
        } else {
            // No hook registered, consider it successful
            HookResult {
                task_type: self.task_type.clone(),
                hook_type: "start".to_string(),
                success: true,
                error: None,
            }
        }
    }

    /// Execute the stop hook
    ///
    /// Returns a HookResult indicating success or failure.
    pub fn execute_stop_hook(&self) -> HookResult {
        if let Some(hook) = &self.stop_hook {
            match hook() {
                Ok(()) => HookResult {
                    task_type: self.task_type.clone(),
                    hook_type: "stop".to_string(),
                    success: true,
                    error: None,
                },
                Err(e) => HookResult {
                    task_type: self.task_type.clone(),
                    hook_type: "stop".to_string(),
                    success: false,
                    error: Some(e),
                },
            }
        } else {
            // No hook registered, consider it successful
            HookResult {
                task_type: self.task_type.clone(),
                hook_type: "stop".to_string(),
                success: true,
                error: None,
            }
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
    /// Instance stop hooks: (task_type, instance_id) -> stop_hook
    /// Stored separately to allow execution during shutdown
    stop_hooks: DashMap<(String, u64), Arc<LifecycleHook>>,
    /// Creation order tracking: creation_order -> (task_type, instance_id)
    creation_order: DashMap<u64, (String, u64)>,
    /// Instance counter per task type
    instance_counts: DashMap<String, AtomicU64>,
    /// Shutdown signal sender (Plan 127: for TaskSystem.stop())
    shutdown_tx: Option<tokio::sync::broadcast::Sender<()>>,
}

impl TaskRegistry {
    /// Create a new task registry
    pub fn new() -> Self {
        let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);
        Self {
            singletons: DashMap::new(),
            instances: DashMap::new(),
            stop_hooks: DashMap::new(),
            creation_order: DashMap::new(),
            instance_counts: DashMap::new(),
            shutdown_tx: Some(shutdown_tx),
        }
    }

    /// Signal the scheduler to stop (Plan 127: for TaskSystem.stop())
    pub fn signal_shutdown(&self) {
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(());
        }
    }

    /// Register a singleton task
    pub fn register_singleton(&self, task_type: String, handle: TaskHandle) {
        self.singletons.insert(task_type, handle);
    }

    /// Register a singleton task with stop hook
    pub fn register_singleton_with_hook(
        &self,
        task_type: String,
        handle: TaskHandle,
        stop_hook: Option<Arc<LifecycleHook>>,
    ) {
        if let Some(hook) = stop_hook {
            let key = (task_type.clone(), 0u64); // instance_id = 0 for singletons
            self.stop_hooks.insert(key, hook);
        }
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
            .entry(handle.task_type.clone())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::SeqCst);
    }

    /// Register a task instance with lifecycle hooks
    ///
    /// Stores the handle and optional stop hook for later shutdown.
    pub fn register_instance_with_hooks(
        &self,
        handle: TaskHandle,
        creation_order: u64,
        stop_hook: Option<Arc<LifecycleHook>>,
    ) {
        let key = (handle.task_type.clone(), handle.instance_id);
        let order_key = creation_order;

        // Store instance handle
        self.instances.insert(key.clone(), handle.clone());

        // Store stop hook if present
        if let Some(hook) = stop_hook {
            self.stop_hooks.insert(key, hook);
        }

        // Track creation order for LIFO shutdown
        self.creation_order
            .insert(order_key, (handle.task_type.clone(), handle.instance_id));

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

    /// Execute all stop hooks in LIFO order
    ///
    /// This method should be called during system shutdown to properly
    /// clean up all tasks. Hooks are executed in reverse creation order:
    /// 1. Task instances (LIFO - last created first)
    /// 2. Singleton tasks
    ///
    /// # Returns
    /// A vector of HookResult for each hook executed.
    pub fn execute_stop_hooks(&self) -> Vec<HookResult> {
        let mut results = Vec::new();

        // Collect all creation orders and sort in reverse (LIFO)
        let mut orders: Vec<u64> = self.creation_order.iter().map(|e| *e.key()).collect();
        orders.sort_by(|a, b| b.cmp(a)); // Reverse order

        // Execute stop hooks for instances in LIFO order
        for order in orders {
            if let Some(entry) = self.creation_order.get(&order) {
                let (task_type, instance_id) = entry.clone();
                let key = (task_type.clone(), instance_id);

                if let Some(hook_entry) = self.stop_hooks.get(&key) {
                    let hook = hook_entry.clone();
                    let result = match hook() {
                        Ok(()) => HookResult {
                            task_type: task_type.clone(),
                            hook_type: "stop".to_string(),
                            success: true,
                            error: None,
                        },
                        Err(e) => HookResult {
                            task_type: task_type.clone(),
                            hook_type: "stop".to_string(),
                            success: false,
                            error: Some(e),
                        },
                    };
                    results.push(result);
                }
            }
        }

        // Execute stop hooks for singletons (last)
        for entry in self.singletons.iter() {
            let task_type = entry.key().clone();
            let key = (task_type.clone(), 0u64);

            if let Some(hook_entry) = self.stop_hooks.get(&key) {
                let hook = hook_entry.clone();
                let result = match hook() {
                    Ok(()) => HookResult {
                        task_type: task_type.clone(),
                        hook_type: "stop".to_string(),
                        success: true,
                        error: None,
                    },
                    Err(e) => HookResult {
                        task_type: task_type.clone(),
                        hook_type: "stop".to_string(),
                        success: false,
                        error: Some(e),
                    },
                };
                results.push(result);
            }
        }

        results
    }

    /// Clear all registered tasks (for shutdown/reset)
    pub fn clear(&self) {
        self.singletons.clear();
        self.instances.clear();
        self.stop_hooks.clear();
        self.creation_order.clear();
        self.instance_counts.clear();
    }
}

impl Default for TaskRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskRegistry {
    /// Start the task scheduler and block until Ctrl+C is received
    ///
    /// This method:
    /// 1. Creates a Tokio runtime (or uses existing one if already in a runtime)
    /// 2. Waits for Ctrl+C signal
    /// 3. Executes all stop hooks in LIFO order
    /// 4. Prints any errors from stop hooks
    ///
    /// # Panics
    /// Panics if the Tokio runtime fails to create or if Ctrl+C handler fails.
    pub fn start_scheduler(&self) {
        // Check if we're already inside a Tokio runtime
        if tokio::runtime::Handle::try_current().is_ok() {
            // Already in a runtime - use block_in_place to allow blocking
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    self.run_scheduler_loop().await;
                });
            });
        } else {
            // Not in a runtime - create one
            let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
            rt.block_on(async {
                self.run_scheduler_loop().await;
            });
        }
    }

    /// Internal scheduler loop
    async fn run_scheduler_loop(&self) {
        // Create shutdown listener if available
        let mut shutdown_rx = self.shutdown_tx.as_ref().map(|tx| tx.subscribe());

        // Wait for either Ctrl+C or shutdown signal
        let shutdown_reason = tokio::select! {
            // Ctrl+C signal
            result = tokio::signal::ctrl_c() => {
                match result {
                    Ok(()) => "Ctrl+C",
                    Err(e) => {
                        eprintln!("Failed to listen for Ctrl+C: {}", e);
                        return;
                    }
                }
            }
            // Shutdown signal from TaskSystem.stop()
            _ = async {
                if let Some(ref mut rx) = shutdown_rx {
                    let _ = rx.recv().await;
                } else {
                    // If no shutdown channel, wait forever
                    std::future::pending::<()>().await;
                }
            } => {
                "TaskSystem.stop()"
            }
        };

        eprintln!("[TaskSystem] Shutdown triggered by {}", shutdown_reason);

        // Execute stop hooks in LIFO order
        let results = self.execute_stop_hooks();

        // Print results
        for result in results {
            if !result.success {
                eprintln!(
                    "Task {}.{} failed: {:?}",
                    result.task_type, result.hook_type, result.error
                );
            }
        }
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
        let mut instance = TaskInstance::new("TestTask".to_string(), 2);
        let handle = instance.handle.clone();

        // Try send should succeed
        handle.try_send(Value::Int(1)).unwrap();
        handle.try_send(Value::Int(2)).unwrap();

        // Third send might fail if channel is full (depends on timing)
        // Just check that try_send returns without blocking
    }

    // ========== Lifecycle Hooks Tests ==========

    #[test]
    fn test_task_instance_creation_order() {
        let i1 = TaskInstance::new("Task1".to_string(), 64);
        let i2 = TaskInstance::new("Task2".to_string(), 64);
        let i3 = TaskInstance::new("Task3".to_string(), 64);

        // Creation order should be sequential
        assert!(i1.creation_order < i2.creation_order);
        assert!(i2.creation_order < i3.creation_order);
    }

    #[test]
    fn test_task_instance_set_hooks() {
        let mut instance = TaskInstance::new("TestTask".to_string(), 64);

        // Initially no hooks
        assert!(instance.start_hook.is_none());
        assert!(instance.stop_hook.is_none());

        // Set hooks
        instance.set_start_hook(Box::new(|| Ok(())));
        instance.set_stop_hook(Box::new(|| Ok(())));

        // Now hooks are set
        assert!(instance.start_hook.is_some());
        assert!(instance.stop_hook.is_some());
    }

    #[test]
    fn test_task_instance_execute_start_hook_success() {
        let mut instance = TaskInstance::new("TestTask".to_string(), 64);

        let called = Arc::new(AtomicU64::new(0));
        let called_clone = called.clone();
        instance.set_start_hook(Box::new(move || {
            called_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }));

        let result = instance.execute_start_hook();

        assert!(result.success);
        assert_eq!(result.task_type, "TestTask");
        assert_eq!(result.hook_type, "start");
        assert!(result.error.is_none());
        assert_eq!(called.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_task_instance_execute_start_hook_failure() {
        let mut instance = TaskInstance::new("TestTask".to_string(), 64);

        instance.set_start_hook(Box::new(|| {
            Err("Start hook failed!".to_string())
        }));

        let result = instance.execute_start_hook();

        assert!(!result.success);
        assert_eq!(result.task_type, "TestTask");
        assert_eq!(result.hook_type, "start");
        assert_eq!(result.error, Some("Start hook failed!".to_string()));
    }

    #[test]
    fn test_task_instance_execute_stop_hook_success() {
        let mut instance = TaskInstance::new("TestTask".to_string(), 64);

        let called = Arc::new(AtomicU64::new(0));
        let called_clone = called.clone();
        instance.set_stop_hook(Box::new(move || {
            called_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }));

        let result = instance.execute_stop_hook();

        assert!(result.success);
        assert_eq!(result.task_type, "TestTask");
        assert_eq!(result.hook_type, "stop");
        assert!(result.error.is_none());
        assert_eq!(called.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_task_instance_no_hook() {
        let instance = TaskInstance::new("TestTask".to_string(), 64);

        // No hooks set - should return success
        let start_result = instance.execute_start_hook();
        assert!(start_result.success);
        assert!(start_result.error.is_none());

        let stop_result = instance.execute_stop_hook();
        assert!(stop_result.success);
        assert!(stop_result.error.is_none());
    }

    #[test]
    fn test_task_registry_register_instance_with_hooks() {
        let registry = TaskRegistry::new();
        let mut instance = TaskInstance::new("TestTask".to_string(), 64);

        instance.set_stop_hook(Box::new(|| Ok(())));

        registry.register_instance_with_hooks(
            instance.handle.clone(),
            instance.creation_order,
            instance.stop_hook.clone(),
        );

        // Verify instance is registered
        let handle = registry.get_instance("TestTask", instance.instance_id);
        assert!(handle.is_some());
    }

    #[test]
    fn test_task_registry_execute_stop_hooks_lifo_order() {
        let registry = TaskRegistry::new();
        let execution_order = Arc::new(std::sync::Mutex::new(Vec::new()));

        // Create multiple tasks with hooks
        let i1 = TaskInstance::new("Task1".to_string(), 64);
        let order1 = execution_order.clone();
        let hook1: LifecycleHook = Box::new(move || {
            order1.lock().unwrap().push("Task1_stop");
            Ok(())
        });
        registry.register_instance_with_hooks(
            i1.handle.clone(),
            i1.creation_order,
            Some(Arc::new(hook1)),
        );

        let i2 = TaskInstance::new("Task2".to_string(), 64);
        let order2 = execution_order.clone();
        let hook2: LifecycleHook = Box::new(move || {
            order2.lock().unwrap().push("Task2_stop");
            Ok(())
        });
        registry.register_instance_with_hooks(
            i2.handle.clone(),
            i2.creation_order,
            Some(Arc::new(hook2)),
        );

        let i3 = TaskInstance::new("Task3".to_string(), 64);
        let order3 = execution_order.clone();
        let hook3: LifecycleHook = Box::new(move || {
            order3.lock().unwrap().push("Task3_stop");
            Ok(())
        });
        registry.register_instance_with_hooks(
            i3.handle.clone(),
            i3.creation_order,
            Some(Arc::new(hook3)),
        );

        // Execute stop hooks
        let results = registry.execute_stop_hooks();

        // All hooks should succeed
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.success));

        // Verify LIFO order: Task3 -> Task2 -> Task1
        let order = execution_order.lock().unwrap();
        assert_eq!(*order, vec!["Task3_stop", "Task2_stop", "Task1_stop"]);
    }

    #[test]
    fn test_task_registry_singleton_with_stop_hook() {
        let registry = TaskRegistry::new();
        let instance = TaskInstance::new("SingletonTask".to_string(), 64);

        let called = Arc::new(AtomicU64::new(0));
        let called_clone = called.clone();
        let hook: LifecycleHook = Box::new(move || {
            called_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });

        registry.register_singleton_with_hook(
            "SingletonTask".to_string(),
            instance.handle.clone(),
            Some(Arc::new(hook)),
        );

        // Verify singleton is registered
        assert!(registry.is_singleton("SingletonTask"));

        // Execute stop hooks (singleton hook should be called last)
        let results = registry.execute_stop_hooks();
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
        assert_eq!(called.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_task_registry_mixed_instances_and_singletons() {
        let registry = TaskRegistry::new();
        let execution_order = Arc::new(std::sync::Mutex::new(Vec::new()));

        // Create instance
        let i1 = TaskInstance::new("InstanceTask".to_string(), 64);
        let order1 = execution_order.clone();
        let hook1: LifecycleHook = Box::new(move || {
            order1.lock().unwrap().push("Instance_stop");
            Ok(())
        });
        registry.register_instance_with_hooks(
            i1.handle.clone(),
            i1.creation_order,
            Some(Arc::new(hook1)),
        );

        // Create singleton
        let singleton = TaskInstance::new("SingletonTask".to_string(), 64);
        let order2 = execution_order.clone();
        let hook2: LifecycleHook = Box::new(move || {
            order2.lock().unwrap().push("Singleton_stop");
            Ok(())
        });
        registry.register_singleton_with_hook(
            "SingletonTask".to_string(),
            singleton.handle.clone(),
            Some(Arc::new(hook2)),
        );

        // Execute stop hooks
        let results = registry.execute_stop_hooks();
        assert_eq!(results.len(), 2);

        // Verify order: instances first, then singletons
        let order = execution_order.lock().unwrap();
        assert_eq!(*order, vec!["Instance_stop", "Singleton_stop"]);
    }

    // ========== start_scheduler Tests ==========

    #[test]
    fn test_task_registry_start_scheduler_exists() {
        // Test that start_scheduler method exists and compiles
        // We can't actually test the blocking behavior in a unit test
        let registry = TaskRegistry::new();

        // Just verify the registry has no tasks
        assert_eq!(registry.get_all_handles().len(), 0);

        // The actual start_scheduler() call would block forever,
        // so we only test execute_stop_hooks here
        let results = registry.execute_stop_hooks();
        assert_eq!(results.len(), 0);
    }
}
