//! oj 解析逻辑

#![allow(dead_code)]

pub mod extractors;
mod factory;
pub mod models;
mod traits;
mod utils;

pub use factory::{create_extractor, extract};
pub use traits::Extractor;

pub(crate) mod constants {
    //! 常量
}

pub mod error {
    //! 错误类型

    use crate::models::*;

    pub type Result<T> = std::result::Result<T, Error>;

    /// 通用错误
    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("no extractor found for url: {0}")]
        NoExtractor(String),
        #[error("extract error: {0}")]
        Extract(ExtractError),
    }

    /// 提取错误
    #[derive(Debug, thiserror::Error)]
    #[error("Extract failed: {kind}")]
    pub struct ExtractError {
        #[source]
        pub kind: ExtractErrorKind,
        pub partial: Option<Box<Submission>>,
    }

    impl ExtractError {
        pub fn new(kind: ExtractErrorKind) -> Self {
            Self {
                kind,
                partial: None,
            }
        }

        pub fn with_partial(kind: ExtractErrorKind, partial: Submission) -> Self {
            Self {
                kind,
                partial: Some(Box::new(partial)),
            }
        }
    }

    /// 提取错误类型
    #[derive(Debug, thiserror::Error)]
    pub enum ExtractErrorKind {
        #[error("no extractor found for url: {0}")]
        NoExtractor(String),
        #[error("parse error: {0}")]
        Parse(String),
        #[error("convert error: {0}")]
        Convert(String),
        #[error("missing field: {0}")]
        MissingField(String),
        #[error("regex mismatch: {0}")]
        RegexMismatch(String),
        #[error("other: {0}")]
        Other(String),
    }
}
