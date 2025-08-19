// We derive from the Debug trait and the Clone trait
// Debug allows us to print the struct with {:?} for debugging
// Clone allows us to create copies of Chunk instances when needed
#[derive(Debug, Clone)]
pub struct Chunk {
    pub text: String,
    pub file: String,
    pub index: usize,
}

///
/// Read a text and chunk it down according to chunk_size
pub fn chunk_text(text: &str, chunk_size: usize, source_file: &str) -> Vec<Chunk> {
    let mut chunks = Vec::new();
    let mut current_pos = 0;
    let mut index = 0;
    let text_len = text.len();

    while current_pos < text_len {
        let mut end_pos = std::cmp::min(current_pos + chunk_size, text_len);

        // Adjust to character boundary if needed - we can't split in the middle of a UTF-8 character
        // is_char_boundary checks if this byte position is safe to split at
        while end_pos < text_len && !text.is_char_boundary(end_pos) {
            end_pos += 1;
        }

        chunks.push(Chunk {
            // we adjust end pos index because slicing in Rust
            // works with Byte Indices, not character indices
            text: text[current_pos..end_pos].to_string(),
            file: source_file.to_string(),
            index,
        });

        current_pos = end_pos;
        index += 1;
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_text_basic() {
        let text = "The quick brown fox jumps over the lazy dog.";
        let chunks = chunk_text(text, 20, "test.txt");

        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].text, "The quick brown fox ");
        assert_eq!(chunks[1].text, "jumps over the lazy ");
        assert_eq!(chunks[2].text, "dog.");
    }
}