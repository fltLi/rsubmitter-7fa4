//! 提取器实现

/*
 * Copyright (c) 2025 fltLi
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod luogu;
pub mod vjudge;
pub mod xyd;

/// 收集注册的提取器
/// 
/// 由于 linkme 分布式注册表的依赖问题, wasm 编译将报错.
/// 现已移除 linkme 并全部替换为手动实现的注册表.
pub(crate) fn registry_items() -> Vec<crate::factory::ExtractorRegistryItem> {
	vec![
		luogu::__EXTRACTOR_REGISTRY_LUOGUEXTRACTOR(),
		vjudge::__EXTRACTOR_REGISTRY_VJUDGEEXTRACTOR(),
		xyd::__EXTRACTOR_REGISTRY_XINYOUDUIEXTRACTOR(),
	]
}
