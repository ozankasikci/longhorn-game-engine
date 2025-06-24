use serde::Serialize;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone, Default)]
pub struct ProcessorContext {
    metadata: HashMap<String, serde_json::Value>,
}

impl ProcessorContext {
    pub fn new() -> Self {
        Self {
            metadata: HashMap::new(),
        }
    }

    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.metadata
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    pub fn set<T: Serialize>(&mut self, key: String, value: T) -> Result<(), serde_json::Error> {
        let json_value = serde_json::to_value(value)?;
        self.metadata.insert(key, json_value);
        Ok(())
    }
}

pub trait AssetProcessor: Send + Sync {
    type Input: Send;
    type Output: Send;

    /// Process an asset, transforming it from Input to Output
    fn process(
        &self,
        asset: Self::Input,
        context: &ProcessorContext,
    ) -> Result<Self::Output, Box<dyn Error>>;

    /// Returns the name of this processor
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

/// A chain of processors that can be applied sequentially
pub struct ProcessorChain<I, O> {
    processors: Vec<Box<dyn AssetProcessor<Input = I, Output = O>>>,
}

impl<I: Send, O: Send> ProcessorChain<I, O> {
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
        }
    }

    pub fn add_processor(&mut self, processor: Box<dyn AssetProcessor<Input = I, Output = O>>) {
        self.processors.push(processor);
    }
}

impl<I: Send, O: Send> Default for ProcessorChain<I, O> {
    fn default() -> Self {
        Self::new()
    }
}

// Common processor implementations

pub struct IdentityProcessor<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Default for IdentityProcessor<T> {
    fn default() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Send + Sync> AssetProcessor for IdentityProcessor<T> {
    type Input = T;
    type Output = T;

    fn process(
        &self,
        asset: Self::Input,
        _context: &ProcessorContext,
    ) -> Result<Self::Output, Box<dyn Error>> {
        Ok(asset)
    }
}
