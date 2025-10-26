//! 提取器实现

pub mod luogu;

/// 收集注册的提取器
/// 
/// 由于 linkme 分布式注册表的依赖问题, wasm 编译将报错.
/// 现已移除 linkme 并全部替换为手动实现的注册表.
pub(crate) fn registry_items() -> Vec<crate::factory::ExtractorRegistryItem> {
	vec![
		luogu::__EXTRACTOR_REGISTRY_LUOGUEXTRACTOR(),
	]
}
