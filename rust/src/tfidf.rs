use std::collections::HashMap;
use indicatif::{ProgressBar, ProgressStyle};
use crate::chunker::Chunk;


/// Calculate term frequency: how often does this term appear in this text?
/// Returns a value between 0.0 and 1.0
pub fn term_frequency(term: &str, text: &str) -> f32 {

    if text.is_empty() || term.is_empty() {
        return 0.0;
    }
    let text_lower: String = text.to_lowercase();
    let term_lower: String = term.to_lowercase();

    let words: Vec<&str> = text_lower.split_whitespace().collect();

    let count = words.iter()
        .filter(|&&w| {
            // Remove common punctuation from the end
            let cleaned = w.trim_end_matches(|c: char| !c.is_alphanumeric());
            cleaned.to_lowercase().contains(term_lower.as_str())
        })
        .count() as f32;
    count / words.len() as f32 // Normalize by document length
}

/// Calculate inverse document frequency: how rare is this term across all chunks?
/// Returns higher values for rarer terms
pub fn inverse_document_frequency(term: &str, chunks: &[Chunk]) -> f32 {
    let term_lower = term.to_lowercase();

    let chunks_with_term = chunks
        .iter()
        .filter(|chunk| chunk.text.to_lowercase().contains(&term_lower))
        .count() as f32;

    if chunks_with_term == 0.0 {
        // Term doesn't appear anywhere - return high but finite IDF
        return ((chunks.len() as f32) / 1.0).ln();
    }

    ((chunks.len() as f32) / chunks_with_term).ln()
}

/// Calculate TF-IDF score for a term in a specific chunk
pub fn tfidf_score(term: &str, chunk: &Chunk, all_chunks: &[Chunk]) -> f32 {
    term_frequency(term, &chunk.text) * inverse_document_frequency(term, all_chunks)
}

/// Score chunks using TF-IDF for a multi-word query
pub fn score_chunks_tfidf(query: &str, chunks: &[Chunk]) -> Vec<(Chunk, f32)> {
    let query_terms: Vec<&str> = query.split_whitespace().collect();
    let pb = ProgressBar::new(chunks.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} TF-IDF [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-")
    );
    // Pre-calculate IDFs for performance (this is the key improvement)
    let mut term_idfs: HashMap<&str, f32> = HashMap::new();
    for term in &query_terms {
        term_idfs.insert(term, inverse_document_frequency(term, chunks));
    }

    let mut scored_chunks: Vec<(Chunk, f32)> = chunks
        .iter()
        .map(|chunk| {
            pb.inc(1);
            // Sum TF-IDF scores for all query terms
            let score: f32 = query_terms
                .iter()
                .map(|term| {
                    let tf = term_frequency(term, &chunk.text);
                    let idf = term_idfs[term];  // Use pre-calculated IDF
                    tf * idf
                })
                .sum();
            (chunk.clone(), score) // Clone instead of borrowing
        })
        .filter(|(_, score)| *score > 0.0)  // Only keep chunks with positive scores
        .collect();
    pb.finish_with_message("TF IDF complete!");
    // Sort by score, highest first
    scored_chunks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    scored_chunks
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    // Helper function to create test chunks
    fn create_chunk(text: &str) -> Chunk {
        Chunk {
            text: text.to_string(),
            file: "".to_string(),
            index: 1
        }
    }


    #[test]
    fn bench_tfidf_performance() {
        // Create 1000 test chunks
        let chunks: Vec<Chunk> = (0..10000)
            .map(|i| Chunk {
                text: format!("test document {}", i),
                file: "test.txt".to_string(),
                index: i,
            })
            .collect();

        let start = Instant::now();
        let _ = score_chunks_tfidf("test", &chunks);
        println!("1000 chunks took: {:?}", start.elapsed());
    }
    #[test]
    fn test_tfidf_score() {
        let chunk1 = create_chunk("the quick brown fox jumps");
        let chunk2 = create_chunk("lazy dog sleeps peacefully"); // Removed "the" to make it rarer
        let chunks = vec![chunk1.clone(), chunk2.clone()];

        // Debug the components
        let tf = term_frequency("the", &chunk1.text);
        let idf = inverse_document_frequency("the", &chunks);
        println!("TF for 'the' in chunk1: {}", tf);
        println!("IDF for 'the' across chunks: {}", idf);
        println!("Chunk1 text: '{}'", chunk1.text);
        println!("Chunk2 text: '{}'", chunk2.text);

        // Test TF-IDF for "the" in first chunk
        let score = tfidf_score("the", &chunk1, &chunks);
        println!("TF-IDF score for 'the': {}", score);

        // Let's test with a term that definitely exists
        let score_brown = tfidf_score("brown", &chunk1, &chunks);
        println!("TF-IDF score for 'brown': {}", score_brown);
        assert!(score_brown > 0.0);

        // Test TF-IDF for non-existent term
        let score_missing = tfidf_score("elephant", &chunk1, &chunks);
        assert_eq!(score_missing, 0.0);
    }

}