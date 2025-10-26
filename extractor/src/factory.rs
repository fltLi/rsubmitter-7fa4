//! 提取器工厂

use crate::error::*;
use crate::models::*;
use crate::traits::Extractor;

use once_cell::sync::Lazy;
use std::sync::Mutex;

/// 提取器注册项
#[derive(Clone)]
pub(crate) struct ExtractorRegistryItem {
    pub(crate) rank_fn: fn(url: &str) -> u32,
    pub(crate) creator: fn() -> Box<dyn Extractor>,
}

/// 提取器工厂
pub(crate) struct ExtractorFactory {
    extractors: Vec<ExtractorRegistryItem>,
}

impl ExtractorFactory {
    /// 创建新的工厂实例
    pub fn new() -> Self {
        let mut items: Vec<ExtractorRegistryItem> = Vec::new();
        items.extend(crate::extractors::registry_items());
        Self { extractors: items }
    }

    /// 根据 URL 创建最匹配的提取器
    pub fn create_extractor(&self, url: &str) -> Result<Box<dyn Extractor>> {
        let mut candidates: Vec<_> = self
            .extractors
            .iter()
            .map(|item| ((item.rank_fn)(url), item))
            .collect();

        // 按分数降序排序
        candidates.sort_by(|a, b| b.0.cmp(&a.0));

        if let Some((highest_score, item)) = candidates.first()
            && *highest_score > 0
        {
            return Ok((item.creator)());
        }

        Err(Error::NoExtractor(url.to_string()))
    }
}

static FACTORY: Lazy<Mutex<ExtractorFactory>> = Lazy::new(|| Mutex::new(ExtractorFactory::new()));

/// 创建提取器
pub fn create_extractor(url: &str) -> Result<Box<dyn Extractor>> {
    FACTORY.lock().unwrap().create_extractor(url)
}

/// 直接提取
pub fn extract(url: &str, content: &str) -> Result<Submission> {
    FACTORY
        .lock()
        .unwrap()
        .create_extractor(url)?
        .extract(url, content)
}
