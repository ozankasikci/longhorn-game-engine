/// Fixed-size ring buffer for event history debugging.
/// When full, oldest events are overwritten.
pub struct RingBuffer<T, const N: usize> {
    buffer: [Option<T>; N],
    head: usize,  // Next write position
    len: usize,   // Current number of items
}

impl<T, const N: usize> RingBuffer<T, N> {
    /// Create a new empty ring buffer.
    pub fn new() -> Self {
        Self {
            buffer: std::array::from_fn(|_| None),
            head: 0,
            len: 0,
        }
    }

    /// Push an item, overwriting oldest if full.
    pub fn push(&mut self, item: T) {
        self.buffer[self.head] = Some(item);
        self.head = (self.head + 1) % N;
        if self.len < N {
            self.len += 1;
        }
    }

    /// Number of items currently stored.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Clear all items.
    pub fn clear(&mut self) {
        for slot in &mut self.buffer {
            *slot = None;
        }
        self.head = 0;
        self.len = 0;
    }

    /// Iterate over items in order (oldest to newest).
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        let start = if self.len < N {
            0
        } else {
            self.head
        };

        (0..self.len).map(move |i| {
            let idx = (start + i) % N;
            self.buffer[idx].as_ref().unwrap()
        })
    }
}

impl<T, const N: usize> Default for RingBuffer<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_iter() {
        let mut rb: RingBuffer<i32, 4> = RingBuffer::new();
        rb.push(1);
        rb.push(2);
        rb.push(3);

        let items: Vec<_> = rb.iter().copied().collect();
        assert_eq!(items, vec![1, 2, 3]);
    }

    #[test]
    fn test_overwrites_when_full() {
        let mut rb: RingBuffer<i32, 3> = RingBuffer::new();
        rb.push(1);
        rb.push(2);
        rb.push(3);
        rb.push(4); // Overwrites 1

        let items: Vec<_> = rb.iter().copied().collect();
        assert_eq!(items, vec![2, 3, 4]);
    }

    #[test]
    fn test_len_and_is_empty() {
        let mut rb: RingBuffer<i32, 4> = RingBuffer::new();
        assert!(rb.is_empty());
        assert_eq!(rb.len(), 0);

        rb.push(1);
        assert!(!rb.is_empty());
        assert_eq!(rb.len(), 1);
    }

    #[test]
    fn test_clear() {
        let mut rb: RingBuffer<i32, 4> = RingBuffer::new();
        rb.push(1);
        rb.push(2);
        rb.clear();

        assert!(rb.is_empty());
        assert_eq!(rb.len(), 0);
    }
}
