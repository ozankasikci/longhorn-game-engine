// Memory management utilities

pub struct MemoryPool<T> {
    items: Vec<Option<T>>,
    free_indices: Vec<usize>,
}

impl<T> MemoryPool<T> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            free_indices: Vec::new(),
        }
    }

    pub fn allocate(&mut self, item: T) -> usize {
        if let Some(index) = self.free_indices.pop() {
            self.items[index] = Some(item);
            index
        } else {
            let index = self.items.len();
            self.items.push(Some(item));
            index
        }
    }

    pub fn deallocate(&mut self, index: usize) -> Option<T> {
        if index < self.items.len() {
            if let Some(item) = self.items[index].take() {
                self.free_indices.push(index);
                Some(item)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index)?.as_ref()
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.items.get_mut(index)?.as_mut()
    }
}
