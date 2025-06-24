//! Type-safe resource handles with weak/strong reference semantics

use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// Unique identifier for a resource
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceId(u64);

impl ResourceId {
    /// Create a new resource ID
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the underlying ID value
    pub fn get(&self) -> u64 {
        self.0
    }

    /// Generate a random resource ID
    pub fn generate() -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::time::{SystemTime, UNIX_EPOCH};

        let mut hasher = DefaultHasher::new();
        hasher.write_u64(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
        );
        Self(hasher.finish())
    }
}

impl fmt::Display for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ResourceId({})", self.0)
    }
}

/// Strong reference to a resource of type T
/// Keeps the resource loaded in memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceHandle<T> {
    id: ResourceId,
    #[serde(skip)]
    _phantom: PhantomData<T>,
}

impl<T> ResourceHandle<T> {
    /// Create a new resource handle
    pub fn new(id: ResourceId) -> Self {
        Self {
            id,
            _phantom: PhantomData,
        }
    }

    /// Get the resource ID
    pub fn id(&self) -> ResourceId {
        self.id
    }

    /// Create a weak reference to this resource
    pub fn downgrade(&self) -> WeakResourceHandle<T> {
        WeakResourceHandle::new(self.id)
    }

    /// Check if this handle refers to the same resource as another
    pub fn same_resource(&self, other: &ResourceHandle<T>) -> bool {
        self.id == other.id
    }
}

impl<T> PartialEq for ResourceHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for ResourceHandle<T> {}

impl<T> Hash for ResourceHandle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T> fmt::Display for ResourceHandle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ResourceHandle<{}>({})",
            std::any::type_name::<T>(),
            self.id
        )
    }
}

/// Weak reference to a resource of type T
/// Does not keep the resource loaded in memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeakResourceHandle<T> {
    id: ResourceId,
    #[serde(skip)]
    _phantom: PhantomData<T>,
}

impl<T> WeakResourceHandle<T> {
    /// Create a new weak resource handle
    pub fn new(id: ResourceId) -> Self {
        Self {
            id,
            _phantom: PhantomData,
        }
    }

    /// Get the resource ID
    pub fn id(&self) -> ResourceId {
        self.id
    }

    /// Try to upgrade this weak reference to a strong reference
    /// This would be implemented by the resource manager
    pub fn upgrade(&self) -> Option<ResourceHandle<T>> {
        // This is a placeholder - actual implementation would check if resource is still loaded
        Some(ResourceHandle::new(self.id))
    }

    /// Check if this handle refers to the same resource as another
    pub fn same_resource(&self, other: &WeakResourceHandle<T>) -> bool {
        self.id == other.id
    }
}

impl<T> PartialEq for WeakResourceHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for WeakResourceHandle<T> {}

impl<T> Hash for WeakResourceHandle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T> fmt::Display for WeakResourceHandle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "WeakResourceHandle<{}>({})",
            std::any::type_name::<T>(),
            self.id
        )
    }
}

/// Convert between handle types
impl<T> From<ResourceHandle<T>> for WeakResourceHandle<T> {
    fn from(handle: ResourceHandle<T>) -> Self {
        handle.downgrade()
    }
}

/// Trait for types that can be used as resources
pub trait Resource: Send + Sync + 'static {
    /// Get a human-readable name for this resource type
    fn resource_type_name() -> &'static str;

    /// Get the memory size of this resource in bytes (approximate)
    fn memory_size(&self) -> usize
    where
        Self: Sized,
    {
        std::mem::size_of::<Self>()
    }
}

// Implement Resource for common types
impl Resource for Vec<u8> {
    fn resource_type_name() -> &'static str {
        "RawData"
    }

    fn memory_size(&self) -> usize {
        self.len()
    }
}

impl Resource for String {
    fn resource_type_name() -> &'static str {
        "Text"
    }

    fn memory_size(&self) -> usize {
        self.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct TestResource {
        data: Vec<u8>,
    }

    impl Resource for TestResource {
        fn resource_type_name() -> &'static str {
            "TestResource"
        }

        fn memory_size(&self) -> usize {
            self.data.len()
        }
    }

    #[test]
    fn test_resource_id() {
        let id1 = ResourceId::new(42);
        let id2 = ResourceId::new(42);
        let id3 = ResourceId::new(43);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
        assert_eq!(id1.get(), 42);
    }

    #[test]
    fn test_resource_handle() {
        let id = ResourceId::new(123);
        let handle: ResourceHandle<TestResource> = ResourceHandle::new(id);

        assert_eq!(handle.id(), id);

        let weak = handle.downgrade();
        assert_eq!(weak.id(), id);

        let strong_again = weak.upgrade().unwrap();
        assert!(handle.same_resource(&strong_again));
    }

    #[test]
    fn test_handle_equality() {
        let id = ResourceId::new(456);
        let handle1: ResourceHandle<TestResource> = ResourceHandle::new(id);
        let handle2: ResourceHandle<TestResource> = ResourceHandle::new(id);
        let handle3: ResourceHandle<TestResource> = ResourceHandle::new(ResourceId::new(789));

        assert_eq!(handle1, handle2);
        assert_ne!(handle1, handle3);
    }

    #[test]
    fn test_resource_trait() {
        let resource = TestResource {
            data: vec![1, 2, 3, 4, 5],
        };
        assert_eq!(TestResource::resource_type_name(), "TestResource");
        assert_eq!(resource.memory_size(), 5);
    }
}
