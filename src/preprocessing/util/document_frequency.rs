use std::collections::HashMap;

use crate::preprocessing::util::term_frequency::TermFrequency;

pub struct DocumentFrequency {
    counts: HashMap<String, usize>,
    vocabulary: Vec<String>,
}

impl DocumentFrequency {
    pub fn count(term_frequencies: &[TermFrequency]) -> Self {
        let mut counts = HashMap::new();
        for term_frequency in term_frequencies {
            for word in term_frequency.keys() {
                if let Some(count) = counts.get_mut(word) {
                    *count += 1;
                } else {
                    counts.insert(word.clone(), 1);
                }
            }
        }
        let mut vocabulary: Vec<_> = counts.keys().cloned().collect();
        vocabulary.sort();
        Self { counts, vocabulary }
    }

    pub fn get(&self, word: &str) -> Option<&usize> {
        self.counts.get(word)
    }

    pub fn vocabulary(&self) -> &[String] {
        &self.vocabulary
    }

    pub fn index_of(&self, word: &str) -> Option<usize> {
        self.vocabulary
            .binary_search_by(|candidate| candidate.as_str().cmp(word))
            .ok()
    }

    pub(crate) fn maximum(&self) -> Option<usize> {
        self.counts.values().copied().max()
    }

    pub fn len(&self) -> usize {
        self.counts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.counts.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preprocessing::Document;
    use rstest::rstest;

    struct TestDocument(Vec<String>);

    impl Document for TestDocument {
        fn name(&self) -> &str {
            "test"
        }

        fn pages(&self) -> &[String] {
            &self.0
        }

        fn pages_size(&self) -> i16 {
            self.0.len() as i16
        }
    }

    #[rstest]
    #[case::no_documents(&[], &[])]
    #[case::empty_documents(&[&[] as &[&str], &["", " .?! "]], &[])]
    #[case::repeated_word_in_one_document(
        &[&["cat cat cat"] as &[&str]],
        &[("cat", 1)]
    )]
    #[case::repeated_word_across_pages(
        &[&["cat dog", "cat cat", "dog"] as &[&str]],
        &[("cat", 1), ("dog", 1)]
    )]
    #[case::overlapping_documents(
        &[&["cat cat dog"] as &[&str], &["cat bird"], &[], &["bird bird"]],
        &[("cat", 2), ("dog", 1), ("bird", 2)]
    )]
    #[case::disjoint_documents(
        &[&["cat"] as &[&str], &["dog"], &["bird"]],
        &[("cat", 1), ("dog", 1), ("bird", 1)]
    )]
    #[case::identical_documents_count_separately(
        &[&["cat dog"] as &[&str], &["cat dog"]],
        &[("cat", 2), ("dog", 2)]
    )]
    #[case::lowercase_unicode_and_numbers(
        &[&["CAFÉ café 東京 42"] as &[&str], &["café? 東京! 42 42"]],
        &[("café", 2), ("東京", 2), ("42", 2)]
    )]
    fn counts_documents(#[case] input: &[&[&str]], #[case] expected: &[(&str, usize)]) {
        let term_frequencies: Vec<TermFrequency> = input
            .iter()
            .map(|pages| {
                let document = TestDocument(pages.iter().map(|page| (*page).to_owned()).collect());
                TermFrequency::count(&document)
            })
            .collect();
        let document_frequency = DocumentFrequency::count(&term_frequencies);

        assert_eq!(document_frequency.len(), expected.len());
        for &(word, count) in expected {
            assert_eq!(document_frequency.get(word), Some(&count));
        }

        let mut expected_vocabulary: Vec<String> =
            expected.iter().map(|&(word, _)| word.to_owned()).collect();
        expected_vocabulary.sort();
        assert_eq!(document_frequency.vocabulary(), expected_vocabulary);
        for (index, word) in expected_vocabulary.iter().enumerate() {
            assert_eq!(document_frequency.index_of(word), Some(index));
        }
        assert_eq!(document_frequency.index_of("missing"), None);
    }
}
