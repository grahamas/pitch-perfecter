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

impl<T: Clone> Iterator for StridedChunks<T> {
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index > self.data.len() {
            return None;
        } else if self.current_index + self.chunk_size > self.data.len() {
            let remaining = self.data.len() - self.current_index;
            if remaining == 0 {
                return None;
            }
            let chunk = self.data[self.current_index..].to_vec();
            self.current_index += remaining; // Move to the end
            return Some(chunk);
        }
        let chunk = self.data[self.current_index..self.current_index + self.chunk_size].to_vec();
        self.current_index += self.step_size;
        Some(chunk)
    }
}