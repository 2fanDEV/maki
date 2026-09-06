use crate::preprocessing::{Document, util::term_frequency::TermFrequency};
use rayon::prelude::*;

#[derive(Default)]
pub enum IdfWeightScheme {
    #[default]
    BASE,
    SMOOTH,
    MAXIMUM,
    PROBABILISTIC,
}

#[allow(non_camel_case_types)]
#[derive(Default)]
pub enum TfWeightScheme {
    BINARY,
    #[default]
    RAW_COUNT,
    NORMALIZATION,
    DOUBLE_NORMALIZATION_HALF,
    DOUBLE_NORMALIZATION_VARIABLE,
}

#[derive(Default)]
pub struct TfIdfBuilder {
    tf_scheme: TfWeightScheme,
    idf_scheme: IdfWeightScheme,
}

impl TfIdfBuilder {
    pub fn tf_weight_scheme(&mut self, weight_scheme: TfWeightScheme) -> &Self {
        self.tf_scheme = weight_scheme;
        self
    }

    pub fn idf_weight_scheme(&mut self, weight_scheme: IdfWeightScheme) -> &Self {
        self.idf_scheme = weight_scheme;
        self
    }

    pub fn build(self) -> TfIdf {
        TfIdf {
            tf_scheme: self.tf_scheme,
            idf_scheme: self.idf_scheme,
        }
    }
}

pub struct TfIdf {
    tf_scheme: TfWeightScheme,
    idf_scheme: IdfWeightScheme,
}

impl TfIdf {
    #[allow(non_snake_case)]
    #[inline(always)]
    pub fn Builder() -> TfIdfBuilder {
        TfIdfBuilder::default()
    }

    pub fn fit<T>(&self, documents: &[T]) {}

    /// Returns raw term counts for each document in input order.
    /// Weighting schemes are applied separately when computing TF-IDF.
    pub fn term_frequencies<T: Document>(&self, documents: &[T]) -> Vec<TermFrequency> {
        documents.par_iter().map(TermFrequency::count).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    #[case::preserves_order_and_separate_counts(
        &["Cat cat", "", "DOG cat"],
        &[&[("cat", 2usize)] as &[(&str, usize)], &[], &[("dog", 1), ("cat", 1)]]
    )]
    fn calculates_term_frequencies(#[case] input: &[&str], #[case] expected: &[&[(&str, usize)]]) {
        let documents: Vec<TestDocument> = input
            .iter()
            .map(|text| TestDocument(vec![(*text).to_owned()]))
            .collect();
        let tfidf = TfIdf::Builder().build();

        let frequencies = tfidf.term_frequencies(&documents);

        assert_eq!(frequencies.len(), expected.len());
        for (frequency, expected) in frequencies.iter().zip(expected) {
            assert_eq!(frequency.len(), expected.len());
            for &(word, count) in *expected {
                assert_eq!(frequency.get(word), Some(&count));
            }
        }
    }
}
