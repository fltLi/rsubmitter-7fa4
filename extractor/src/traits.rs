//! 提取器特型

use crate::error::*;
use crate::models::*;

/// 提取器
pub trait Extractor {
    /// 解析提交记录, 返回 7fa4 格式
    fn extract(&self, url: &str, content: &str) -> Result<Submission>;
}

/// 工厂注册用提取器
pub(crate) trait ExtractorRegistry: Extractor + Sync + Send {
    /// 依据 url 计算相似度
    fn rank(&self, url: &str) -> u32;

    /// 装箱提取器
    #[allow(clippy::new_ret_no_self)]
    fn new() -> Box<dyn Extractor>;
}
