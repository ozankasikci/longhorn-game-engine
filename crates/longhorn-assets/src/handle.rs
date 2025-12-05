use longhorn_core::AssetId;
use std::marker::PhantomData;

/// A typed handle to an asset in the asset manager
#[derive(Debug, Clone, Copy)]
pub struct AssetHandle<T> {
    id: AssetId,
    _marker: PhantomData<T>,
}

impl<T> AssetHandle<T> {
    /// Create a new asset handle
    pub fn new(id: AssetId) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }

    /// Get the asset ID
    pub fn id(&self) -> AssetId {
        self.id
    }
}

impl<T> PartialEq for AssetHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for AssetHandle<T> {}

impl<T> std::hash::Hash for AssetHandle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct DummyAsset;

    #[test]
    fn test_handle_creation() {
        let handle = AssetHandle::<DummyAsset>::new(AssetId::new(42));
        assert_eq!(handle.id(), AssetId::new(42));
    }

    #[test]
    fn test_handle_equality() {
        let handle1 = AssetHandle::<DummyAsset>::new(AssetId::new(1));
        let handle2 = AssetHandle::<DummyAsset>::new(AssetId::new(1));
        let handle3 = AssetHandle::<DummyAsset>::new(AssetId::new(2));

        assert_eq!(handle1, handle2);
        assert_ne!(handle1, handle3);
    }
}
