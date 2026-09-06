use anyhow::{Result, anyhow};
use log::debug;

use crate::methods::{
    classifiers::Classifier,
    preprocessing::{
        Document,
        tfidf::{self, TfIdf, TfIdfBuilder},
    },
};

mod classifiers;
mod preprocessing;

pub type ModelCreatingFunction<T: Document, C: Classifier> = Box<dyn FnOnce(Vec<T>) -> Result<C>>;

pub struct TrainerBuilder<T: Document, C: Classifier> {
    /// default: split_ratio = 0.8
    split_ratio: f32,
    documents: Vec<T>,
    /// default:
    builder: Option<TfIdfBuilder>,
    method: ModelCreatingFunction<T, C>,
}

impl<T: Document, C: Classifier> Default for TrainerBuilder<T, C> {
    fn default() -> Self {
        Self {
            split_ratio: 0.8,
            documents: Default::default(),
            builder: None,
            method: Box::new(|docs| Err(anyhow!("Missing model generating function!"))),
        }
    }
}

impl<T: Document, C: Classifier> TrainerBuilder<T, C> {
   pub fn split_ratio(&mut self, split_ratio: f32) -> &mut Self {
      self.split_ratio = split_ratio;
      self
   }

   /// If tf_idf_builder is not added, then a basic tfidf will be created with default values.
   pub fn tf_idf_builder(&mut self, builder: TfIdfBuilder) -> &mut Self {
      self.builder = Some(builder);
      self
   }

   pub fn documents(&mut self, documents: Vec<T>) -> &mut Self {
      self.documents = documents;
      self
   }

   pub fn method(&mut self, model_creator: ModelCreatingFunction<T, C>) -> &mut Self {
      self.method = model_creator;
      self
   }

   pub fn build(&self) {

   }
}

pub struct Trainer<T: Document, C: Classifier> {
    tf_idf: TfIdf,
    train_documents: Vec<T>,
    test_documents: Vec<T>,
    model: C,
}

impl<T: Document, C: Classifier> Trainer<T, C> {
    #[allow(non_snake_case)]
    pub fn Builder() -> TrainerBuilder<T, C> {
        TrainerBuilder::default()
    }

    fn new(
        train_documents: Vec<T>,
        test_documents: Vec<T>,
        tf_idf: TfIdf,
        method: ModelCreatingFunction<T, C>,
    ) -> Result<Self> {
        let model = match method(train_documents) {
            Ok(model) => {
                debug!("Model: {:?} has successfully been created", "123")
                model
            }
            Err(e) => return Err(e),
        };
        Ok(Self {
            tf_idf,
            train_documents,
            test_documents,
            model,
        })
    }
}
