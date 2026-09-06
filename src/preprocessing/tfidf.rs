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
}
