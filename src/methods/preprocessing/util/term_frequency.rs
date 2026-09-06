use rayon::prelude::*;
use regex::Regex;
use std::collections::HashMap;
use std::collections::hash_map::{Iter, Keys, Values};
use std::sync::LazyLock;

use crate::methods::preprocessing::Document;
use crate::methods::preprocessing::util::merge_counts;

static WORDS: LazyLock<Regex> = LazyLock::new(|| {
    // Start with a Unicode letter or number, then allow combining accents too.
    Regex::new(r"[\p{Letter}\p{Number}][\p{Letter}\p{Mark}\p{Number}]*").expect("valid word regex")
});

/// Read-only term occurrence counts, constructed from one or more documents.
pub struct TermFrequency {
    counts: HashMap<String, usize>,
}

impl TermFrequency {
    /// Sums term occurrences across all documents, not document frequencies for IDF.
    pub fn count_documents<T: Document>(documents: &[T]) -> Self {
        let counts = documents
            .par_iter()
            .map(|document| Self::count(document).counts)
            .reduce(HashMap::new, merge_counts);
        Self { counts }
    }

    /// Counts lowercase Unicode words and numbers across all pages. Punctuation
    /// separates tokens; common words are retained without a stoplist.
    pub fn count<T: Document>(document: &T) -> Self {
        let counts = document
            .pages()
            .par_iter()
            .map_init(
                || WORDS.clone(),
                |words, page| {
                    let mut counts = HashMap::<String, usize>::new();
                    let lowercase = page.to_lowercase();
                    for word in words.find_iter(&lowercase) {
                        if let Some(count) = counts.get_mut(word.as_str()) {
                            *count += 1;
                        } else {
                            counts.insert(word.as_str().to_owned(), 1);
                        }
                    }
                    counts
                },
            )
            .reduce(HashMap::new, merge_counts);
        Self { counts }
    }

    pub fn keys(&self) -> Keys<'_, String, usize> {
        self.counts.keys()
    }

    pub fn values(&self) -> Values<'_, String, usize> {
        self.counts.values()
    }

    pub fn iter(&self) -> Iter<'_, String, usize> {
        self.counts.iter()
    }

    pub fn get(&self, word: &str) -> Option<&usize> {
        self.counts.get(word)
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
    use rstest::rstest;

    struct TestDocument(Vec<String>);

    fn assert_counts(term_frequency: &TermFrequency, expected: &[(&str, usize)]) {
        assert_eq!(term_frequency.len(), expected.len());
        for &(word, count) in expected {
            assert_eq!(term_frequency.get(word), Some(&count));
        }
    }

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
    #[case::pages_and_common_words(
        &["The cat,cat.\tdog\n42?", "CAT! the dog 42"],
        &[("the", 2), ("cat", 3), ("dog", 2), ("42", 2)]
    )]
    #[case::unicode(
        &["CAFÉ café 東京 東京 cafe\u{301} ١٢ ١٢"],
        &[("café", 2), ("東京", 2), ("cafe\u{301}", 1), ("١٢", 2)]
    )]
    #[case::no_pages(&[], &[])]
    #[case::empty_and_punctuation(&["", " .?!\n\t—_ "], &[])]
    #[case::punctuation_separates(
        &["Can't re-enter? hello_world!"],
        &[("can", 1), ("t", 1), ("re", 1), ("enter", 1), ("hello", 1), ("world", 1)]
    )]
    fn counts_document(#[case] input: &[&str], #[case] expected: &[(&str, usize)]) {
        let document = TestDocument(input.iter().map(|page| (*page).to_owned()).collect());
        assert_counts(&TermFrequency::count(&document), expected);
    }

    #[rstest]
    #[case::multiple_documents(
        &[&["The cat cat", "DOG"] as &[&str], &[], &["cat dog dog bird"]],
        &[("the", 1), ("cat", 3), ("dog", 3), ("bird", 1)]
    )]
    #[case::no_documents(&[], &[])]
    #[case::empty_documents(&[&[] as &[&str], &[""]], &[])]
    fn counts_documents(#[case] input: &[&[&str]], #[case] expected: &[(&str, usize)]) {
        let documents: Vec<TestDocument> = input
            .iter()
            .map(|pages| TestDocument(pages.iter().map(|page| (*page).to_owned()).collect()))
            .collect();
        assert_counts(&TermFrequency::count_documents(&documents), expected);
    }
}
