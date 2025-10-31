//! oj 解析逻辑

/*
 * Copyright (c) 2025 fltLi
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

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
        #[error("selector parse error: {0}")]
        SelectorParse(String),
        #[error("time parse error: {0}")]
        TimeParse(String),
        #[error("memory parse error: {0}")]
        MemoryParse(String),
        #[error("language parse error: {0}")]
        LanguageParse(String),
        #[error("status parse error: {0}")]
        StatusParse(String),
        #[error("invalid url: {0}")]
        InvalidUrl(String),
        #[error("empty content")]
        EmptyContent,
        #[error("not in submission page: {0}")]
        NotInSubmissionPage(String),
        #[error("no submission selected: {0}")]
        NoSubmissionSelected(String),
        #[error("other: {0}")]
        Other(String),
    }
}
