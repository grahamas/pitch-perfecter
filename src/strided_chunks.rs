//! # Strided Chunks Iterator
//! This module provides an iterator for creating strided chunks from a vector.
//! It allows you to create overlapping chunks of a specified size and step size
//! 
//! NOTE: The iterator will stop when it can no longer create a full chunk.

pub struct StridedChunks<T> {
    data: Vec<T>,
    chunk_size: usize,
    step_size: usize,
    current_index: usize,
}

impl<T> StridedChunks<T> {
    pub fn new(data: Vec<T>, chunk_size: usize, step_size: usize) -> Self {
        StridedChunks {
            data,
            chunk_size,
            step_size,
            current_index: 0,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strided_chunks_basic() {
        let data = vec![1, 2, 3, 4, 5];
        let mut iter = StridedChunks::new(data, 2, 1);
        assert_eq!(iter.next(), Some(vec![1, 2]));
        assert_eq!(iter.next(), Some(vec![2, 3]));
        assert_eq!(iter.next(), Some(vec![3, 4]));
        assert_eq!(iter.next(), Some(vec![4, 5]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_strided_chunks_step_size() {
        let data = vec![10, 20, 30, 40, 50, 60];
        let mut iter = StridedChunks::new(data, 3, 2);
        assert_eq!(iter.next(), Some(vec![10, 20, 30]));
        assert_eq!(iter.next(), Some(vec![30, 40, 50]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_strided_chunks_chunk_too_large() {
        let data = vec![1, 2, 3];
        let mut iter = StridedChunks::new(data, 5, 1);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_strided_chunks_exact_fit() {
        let data = vec![1, 2, 3, 4];
        let mut iter = StridedChunks::new(data, 2, 2);
        assert_eq!(iter.next(), Some(vec![1, 2]));
        assert_eq!(iter.next(), Some(vec![3, 4]));
        assert_eq!(iter.next(), None);
    }
}

impl<T: Clone> Iterator for StridedChunks<T> {
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index + self.chunk_size > self.data.len() {
            return None;
        }
        let chunk = self.data[self.current_index..self.current_index + self.chunk_size].to_vec();
        self.current_index += self.step_size;
        Some(chunk)
    }
}