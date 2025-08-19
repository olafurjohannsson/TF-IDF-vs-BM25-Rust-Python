use crate::chunker::{chunk_text, Chunk};

/// Search for chunks containing the query string
// Takes query as &str (borrowed string slice) and files as a slice of tuples
// &[(String, String)] is a borrowed slice of tuples, where each tuple is (filename, content)
// The & means we're borrowing the data, not taking ownership
pub fn search_chunks(query: &str, files: &[(String, String)]) -> Vec<Chunk> {
    let mut all_chunks = Vec::new();

    // First, chunk all files into 500-character segments
    for (filename, content) in files {
        // chunk_text returns Vec<Chunk>, we're using 500 chars (roughly 75-100 tokens)
        let chunks = chunk_text(content, 500, filename);
        // extend() adds all elements from chunks to all_chunks
        // Unlike push() which adds one item, extend() adds multiple items
        all_chunks.extend(chunks);
    }

    // Search within chunks using iterator chains
    all_chunks
        .into_iter() // into_iter() consumes the vector, taking ownership (we won't need all_chunks after this)
        .filter(|chunk| chunk.text.to_lowercase().contains(&query.to_lowercase())) // filter keeps only chunks containing our query
        .collect() // collect() consumes the iterator and builds a new Vec<Chunk> from filtered results
}

/// Search for lines containing the query string
/// Returns Vec<(String, Vec<String>)> - a vector of tuples containing (filename, matching_lines)
pub fn search_files(query: &str, files: &[(String, String)]) -> Vec<(String, Vec<String>)> {
    let mut results = Vec::new();
    // Convert query to lowercase once, outside the loop for efficiency
    let lowercase_query = query.to_lowercase();

    for (filename, content) in files {
        let matches: Vec<String> = content
            .lines() // lines() splits the string by newlines, returns an iterator of &str
            .filter(|line| line.to_lowercase().contains(&lowercase_query)) // keep only lines containing query
            .map(|line| line.to_string()) // convert &str to owned String (needed because we're storing them)
            .collect(); // build Vec<String> from the filtered lines

        if !matches.is_empty() {
            // clone() creates a copy of filename since we need an owned String
            // We can't move filename because it's borrowed from files
            results.push((filename.clone(), matches));
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_chunks_vs_search_files() {
        // Create test data that demonstrates the difference between line and chunk search
        let files = vec![(
            "test.txt".to_string(),
            "This is a long paragraph about Rust programming. \
             It contains multiple sentences and spans several lines. \
             The word 'programming' appears here and provides good context \
             for understanding what this text is about.".to_string()
        )];

        // Search for 'programming' using both methods
        let line_results = search_files("programming", &files);
        let chunk_results = search_chunks("programming", &files);

        // Line search finds the specific line
        assert_eq!(line_results.len(), 1);

        // Chunk search finds the chunk with more context
        assert_eq!(chunk_results.len(), 1);
        assert_eq!(chunk_results[0].text.len(), line_results[0].1[0].len());
        assert!(chunk_results[0].text.contains("Rust programming"));
        assert!(chunk_results[0].text.contains("context"));
    }
}