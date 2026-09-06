use anyhow::{Result, anyhow, bail};
use rayon::prelude::*;
use sprs::CsMat;

use crate::methods::preprocessing::{
    Document,
    util::{document_frequency::DocumentFrequency, term_frequency::TermFrequency},
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum IdfWeightScheme {
    #[default]
    BASE,
    SMOOTH,
    MAXIMUM,
    PROBABILISTIC,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TfWeightScheme {
    BINARY,
    #[default]
    RAW_COUNT,
    NORMALIZATION,
    DOUBLE_NORMALIZATION_HALF,
    DOUBLE_NORMALIZATION_VARIABLE,
}

pub struct TfIdfBuilder {
    tf_scheme: TfWeightScheme,
    idf_scheme: IdfWeightScheme,
    double_normalization_k: f64,
}

impl Default for TfIdfBuilder {
    fn default() -> Self {
        Self {
            tf_scheme: TfWeightScheme::default(),
            idf_scheme: IdfWeightScheme::default(),
            double_normalization_k: 0.5,
        }
    }
}

impl TfIdfBuilder {
    pub fn tf_weight_scheme(&mut self, weight_scheme: TfWeightScheme) -> &mut Self {
        self.tf_scheme = weight_scheme;
        self
    }

    pub fn idf_weight_scheme(&mut self, weight_scheme: IdfWeightScheme) -> &mut Self {
        self.idf_scheme = weight_scheme;
        self
    }

    pub fn double_normalization_k(&mut self, k: f64) -> &mut Self {
        self.double_normalization_k = k;
        self
    }

    pub fn build(&self) -> Result<TfIdf> {
        if !self.double_normalization_k.is_finite()
            || !(0.0..=1.0).contains(&self.double_normalization_k)
        {
            bail!("double-normalization k must be finite and within [0, 1]");
        }

        Ok(TfIdf {
            tf_scheme: self.tf_scheme,
            idf_scheme: self.idf_scheme,
            double_normalization_k: self.double_normalization_k,
            fitted_state: None,
        })
    }
}

struct FittedState {
    document_frequency: DocumentFrequency,
    idf_weights: Vec<f64>,
}

pub struct TfIdf {
    tf_scheme: TfWeightScheme,
    idf_scheme: IdfWeightScheme,
    double_normalization_k: f64,
    fitted_state: Option<FittedState>,
}

impl TfIdf {
    #[allow(non_snake_case)]
    #[inline(always)]
    pub fn Builder() -> TfIdfBuilder {
        TfIdfBuilder::default()
    }

    pub fn fit<T: Document>(&mut self, documents: &[T]) -> Result<()> {
        let term_frequencies = self.term_frequencies(documents);
        self.replace_fitted_state(&term_frequencies, documents.len());
        Ok(())
    }

    pub fn transform<T: Document>(&self, documents: &[T]) -> Result<CsMat<f64>> {
        self.fitted_state()?;
        let term_frequencies = self.term_frequencies(documents);
        self.transform_term_frequencies(&term_frequencies)
    }

    pub fn fit_transform<T: Document>(&mut self, documents: &[T]) -> Result<CsMat<f64>> {
        let term_frequencies = self.term_frequencies(documents);
        self.replace_fitted_state(&term_frequencies, documents.len());
        self.transform_term_frequencies(&term_frequencies)
    }

    pub fn vocabulary(&self) -> Result<&[String]> {
        Ok(self.fitted_state()?.document_frequency.vocabulary())
    }

    pub fn idf_weights(&self) -> Result<&[f64]> {
        Ok(&self.fitted_state()?.idf_weights)
    }

    /// Returns raw term counts for each document in input order.
    /// Weighting schemes are applied separately when computing TF-IDF.
    pub fn term_frequencies<T: Document>(&self, documents: &[T]) -> Vec<TermFrequency> {
        documents.par_iter().map(TermFrequency::count).collect()
    }

    fn fitted_state(&self) -> Result<&FittedState> {
        self.fitted_state
            .as_ref()
            .ok_or_else(|| anyhow!("TF-IDF must be fitted before accessing fitted state"))
    }

    fn replace_fitted_state(&mut self, term_frequencies: &[TermFrequency], document_count: usize) {
        let document_frequency = DocumentFrequency::count(term_frequencies);
        let maximum_document_frequency = document_frequency.maximum().unwrap_or(0);
        let idf_weights = document_frequency
            .vocabulary()
            .iter()
            .map(|word| {
                let frequency = *document_frequency
                    .get(word)
                    .expect("vocabulary terms have document frequencies");
                self.calculate_idf_weight(document_count, frequency, maximum_document_frequency)
            })
            .collect();

        self.fitted_state = Some(FittedState {
            document_frequency,
            idf_weights,
        });
    }

    fn transform_term_frequencies(&self, term_frequencies: &[TermFrequency]) -> Result<CsMat<f64>> {
        let fitted_state = self.fitted_state()?;
        let vocabulary_size = fitted_state.document_frequency.len();
        let rows: Vec<Vec<(usize, f64)>> = term_frequencies
            .par_iter()
            .map(|term_frequency| {
                let total_count = term_frequency.values().copied().sum();
                let maximum_count = term_frequency.values().copied().max().unwrap_or(0);
                let mut row = Vec::with_capacity(term_frequency.len());

                for word in term_frequency.keys() {
                    let Some(column) = fitted_state.document_frequency.index_of(word) else {
                        continue;
                    };
                    let count = *term_frequency
                        .get(word)
                        .expect("term-frequency keys have counts");
                    let tf_weight = self.calculate_tf_weight(count, total_count, maximum_count);
                    let tf_idf = tf_weight * fitted_state.idf_weights[column];
                    if tf_idf != 0.0 {
                        row.push((column, tf_idf));
                    }
                }

                row.sort_by_key(|(column, _)| *column);
                row
            })
            .collect();

        let non_zero_count = rows.iter().map(Vec::len).sum();
        let mut index_pointers = Vec::with_capacity(rows.len() + 1);
        let mut column_indices = Vec::with_capacity(non_zero_count);
        let mut values = Vec::with_capacity(non_zero_count);
        index_pointers.push(0);

        for row in rows {
            for (column, value) in row {
                column_indices.push(column);
                values.push(value);
            }
            index_pointers.push(values.len());
        }

        Ok(CsMat::new(
            (term_frequencies.len(), vocabulary_size),
            index_pointers,
            column_indices,
            values,
        ))
    }

    fn calculate_tf_weight(&self, count: usize, total_count: usize, maximum_count: usize) -> f64 {
        if count == 0 {
            return 0.0;
        }

        match self.tf_scheme {
            TfWeightScheme::BINARY => 1.0,
            TfWeightScheme::RAW_COUNT => count as f64,
            TfWeightScheme::NORMALIZATION => count as f64 / total_count as f64,
            TfWeightScheme::DOUBLE_NORMALIZATION_HALF => {
                0.5 + 0.5 * count as f64 / maximum_count as f64
            }
            TfWeightScheme::DOUBLE_NORMALIZATION_VARIABLE => {
                self.double_normalization_k
                    + (1.0 - self.double_normalization_k) * count as f64 / maximum_count as f64
            }
        }
    }

    fn calculate_idf_weight(
        &self,
        document_count: usize,
        document_frequency: usize,
        maximum_document_frequency: usize,
    ) -> f64 {
        match self.idf_scheme {
            IdfWeightScheme::BASE => (document_count as f64 / document_frequency as f64).ln(),
            IdfWeightScheme::SMOOTH => {
                (document_count as f64 / (1 + document_frequency) as f64).ln() + 1.0
            }
            IdfWeightScheme::MAXIMUM => {
                (maximum_document_frequency as f64 / (1 + document_frequency) as f64).ln()
            }
            IdfWeightScheme::PROBABILISTIC => {
                if document_frequency >= document_count {
                    0.0
                } else {
                    (((document_count - document_frequency) as f64) / document_frequency as f64)
                        .ln()
                        .max(0.0)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[derive(Clone)]
    struct TestDocument(Vec<String>);

    impl TestDocument {
        fn new(text: &str) -> Self {
            Self(vec![text.to_owned()])
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

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-12,
            "expected {expected}, got {actual}"
        );
    }

    fn tfidf_with_schemes(tf_scheme: TfWeightScheme, idf_scheme: IdfWeightScheme, k: f64) -> TfIdf {
        let mut builder = TfIdf::Builder();
        builder
            .tf_weight_scheme(tf_scheme)
            .idf_weight_scheme(idf_scheme)
            .double_normalization_k(k);
        builder.build().unwrap()
    }

    #[rstest]
    #[case::binary(TfWeightScheme::BINARY, 2, 4, 2, 0.5, 1.0)]
    #[case::raw_count(TfWeightScheme::RAW_COUNT, 2, 4, 2, 0.5, 2.0)]
    #[case::normalization(TfWeightScheme::NORMALIZATION, 2, 4, 2, 0.5, 0.5)]
    #[case::double_normalization_half(
        TfWeightScheme::DOUBLE_NORMALIZATION_HALF,
        1,
        4,
        2,
        0.5,
        0.75
    )]
    #[case::double_normalization_variable(
        TfWeightScheme::DOUBLE_NORMALIZATION_VARIABLE,
        1,
        4,
        2,
        0.25,
        0.625
    )]
    #[case::absent_term_is_zero(TfWeightScheme::DOUBLE_NORMALIZATION_HALF, 0, 0, 0, 0.5, 0.0)]
    fn calculates_tf_weights(
        #[case] scheme: TfWeightScheme,
        #[case] count: usize,
        #[case] total: usize,
        #[case] maximum: usize,
        #[case] k: f64,
        #[case] expected: f64,
    ) {
        let tfidf = tfidf_with_schemes(scheme, IdfWeightScheme::BASE, k);
        assert_close(tfidf.calculate_tf_weight(count, total, maximum), expected);
    }

    #[rstest]
    #[case::base(IdfWeightScheme::BASE, 10, 2, 8, (5.0_f64).ln())]
    #[case::smooth(IdfWeightScheme::SMOOTH, 10, 2, 8, (10.0_f64 / 3.0).ln() + 1.0)]
    #[case::maximum(IdfWeightScheme::MAXIMUM, 10, 2, 8, (8.0_f64 / 3.0).ln())]
    #[case::probabilistic(IdfWeightScheme::PROBABILISTIC, 10, 2, 8, (4.0_f64).ln())]
    #[case::probabilistic_clamps_negative(IdfWeightScheme::PROBABILISTIC, 10, 6, 8, 0.0)]
    #[case::probabilistic_all_documents(IdfWeightScheme::PROBABILISTIC, 10, 10, 10, 0.0)]
    fn calculates_idf_weights(
        #[case] scheme: IdfWeightScheme,
        #[case] document_count: usize,
        #[case] document_frequency: usize,
        #[case] maximum_document_frequency: usize,
        #[case] expected: f64,
    ) {
        let tfidf = tfidf_with_schemes(TfWeightScheme::RAW_COUNT, scheme, 0.5);
        assert_close(
            tfidf.calculate_idf_weight(
                document_count,
                document_frequency,
                maximum_document_frequency,
            ),
            expected,
        );
    }

    #[rstest]
    #[case(-0.1)]
    #[case(1.1)]
    #[case(f64::NAN)]
    #[case(f64::INFINITY)]
    fn rejects_invalid_double_normalization_k(#[case] k: f64) {
        let mut builder = TfIdf::Builder();
        builder.double_normalization_k(k);
        assert!(builder.build().is_err());
    }

    #[rstest]
    #[case(0.0)]
    #[case(0.5)]
    #[case(1.0)]
    fn accepts_valid_double_normalization_k(#[case] k: f64) {
        let mut builder = TfIdf::Builder();
        builder.double_normalization_k(k);
        assert!(builder.build().is_ok());
    }

    #[test]
    fn fits_and_returns_sparse_matrix_with_stable_columns() {
        let documents = [
            TestDocument::new("cat cat dog"),
            TestDocument::new("cat bird"),
        ];
        let mut tfidf = TfIdf::Builder().build().unwrap();

        let matrix = tfidf.fit_transform(&documents).unwrap();

        assert!(matrix.is_csr());
        assert_eq!(matrix.shape(), (2, 3));
        assert_eq!(tfidf.vocabulary().unwrap(), ["bird", "cat", "dog"]);
        assert_eq!(matrix.nnz(), 2);

        let dense = matrix.to_dense();
        let ln_two = 2.0_f64.ln();
        assert_close(dense[[0, 0]], 0.0);
        assert_close(dense[[0, 1]], 0.0);
        assert_close(dense[[0, 2]], ln_two);
        assert_close(dense[[1, 0]], ln_two);
        assert_close(dense[[1, 1]], 0.0);
        assert_close(dense[[1, 2]], 0.0);
    }

    #[test]
    fn fit_then_transform_matches_fit_transform() {
        let documents = [
            TestDocument::new("cat cat dog"),
            TestDocument::new("cat bird"),
        ];
        let mut separately_fitted = TfIdf::Builder().build().unwrap();
        separately_fitted.fit(&documents).unwrap();
        let transformed = separately_fitted.transform(&documents).unwrap();

        let mut fitted_while_transforming = TfIdf::Builder().build().unwrap();
        let fit_transformed = fitted_while_transforming.fit_transform(&documents).unwrap();

        assert_eq!(transformed.to_dense(), fit_transformed.to_dense());
    }

    #[test]
    fn ignores_terms_outside_the_fitted_vocabulary() {
        let mut tfidf = TfIdf::Builder().build().unwrap();
        tfidf.fit(&[TestDocument::new("cat")]).unwrap();

        let matrix = tfidf.transform(&[TestDocument::new("bird")]).unwrap();

        assert_eq!(matrix.shape(), (1, 1));
        assert_eq!(matrix.nnz(), 0);
    }

    #[test]
    fn empty_documents_produce_empty_sparse_rows() {
        let documents = [TestDocument::new(""), TestDocument::new(".?!")];
        let mut tfidf = TfIdf::Builder().build().unwrap();

        let matrix = tfidf.fit_transform(&documents).unwrap();

        assert_eq!(matrix.shape(), (2, 0));
        assert_eq!(matrix.nnz(), 0);
        assert!(tfidf.vocabulary().unwrap().is_empty());
        assert!(tfidf.idf_weights().unwrap().is_empty());
    }

    #[test]
    fn no_documents_produce_an_empty_matrix() {
        let documents: [TestDocument; 0] = [];
        let mut tfidf = TfIdf::Builder().build().unwrap();

        let matrix = tfidf.fit_transform(&documents).unwrap();

        assert_eq!(matrix.shape(), (0, 0));
        assert_eq!(matrix.nnz(), 0);
    }

    #[test]
    fn refitting_replaces_vocabulary_and_idf_weights() {
        let mut tfidf = TfIdf::Builder().build().unwrap();
        tfidf
            .fit(&[TestDocument::new("cat dog"), TestDocument::new("cat")])
            .unwrap();
        assert_eq!(tfidf.vocabulary().unwrap(), ["cat", "dog"]);
        assert_eq!(tfidf.idf_weights().unwrap().len(), 2);

        tfidf.fit(&[TestDocument::new("bird")]).unwrap();

        assert_eq!(tfidf.vocabulary().unwrap(), ["bird"]);
        assert_eq!(tfidf.idf_weights().unwrap(), [0.0]);
    }

    #[test]
    fn fitted_access_and_transform_fail_before_fit() {
        let tfidf = TfIdf::Builder().build().unwrap();
        let documents = [TestDocument::new("cat")];

        assert!(tfidf.vocabulary().is_err());
        assert!(tfidf.idf_weights().is_err());
        assert!(tfidf.transform(&documents).is_err());
    }

    #[rstest]
    #[case::no_documents(&[], &[])]
    #[case::preserves_order_and_separate_counts(
        &["Cat cat", "", "DOG cat"],
        &[&[("cat", 2usize)] as &[(&str, usize)], &[], &[("dog", 1), ("cat", 1)]]
    )]
    fn calculates_term_frequencies(#[case] input: &[&str], #[case] expected: &[&[(&str, usize)]]) {
        let documents: Vec<TestDocument> =
            input.iter().map(|text| TestDocument::new(text)).collect();
        let tfidf = TfIdf::Builder().build().unwrap();

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
