/*
 * Copyright (c) 2025 fltLi
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::ToTokens;
use quote::{format_ident, quote};
use regex::Regex;
use syn::{Attribute, DeriveInput, parse_macro_input};

/// 提取器属性
///
/// # 使用示例
///
/// 在提取器类型上添加 `#[derive(Extractable)]` 并使用 `#[extractor(...)]` 属性指定元数据:
///
/// ```rust
/// #[derive(Extractable)]
/// #[extractor(name = "洛谷", tags = ["luogu", "Luogu"]) ]
/// pub struct LuoguExtractor;
/// ```
///
/// 支持的属性:
/// - `name = "..."`: 提取器显示名称 (必须)
/// - `tags = ["t1", "t2"]`: 用于基于 URL 的匹配标签 (可选)
///
/// 该宏会为类型生成 `ExtractorRegistry` 的实现, 并把提取器注册到 `crate::factory::EXTRACTOR_REGISTRY` 分布式切片中.
#[derive(Debug)]
struct ExtractorAttributes {
    name: String,
    tags: Vec<String>,
}

impl ExtractorAttributes {
    fn from_attrs(attrs: &[Attribute]) -> Result<Self, syn::Error> {
        // 使用简单的字符串解析方式 (兼容不同版本的 syn) :
        // attr.tokens 的文本里包含形如: (name = "xxx", tags = ["a","b"]).
        let mut name = None;
        let mut tags = Vec::new();

        let name_re = Regex::new(r#"name\s*=\s*\"([^\"]+)\""#).unwrap();
        let tags_re = Regex::new(r"tags\s*=\s*\[(?P<inner>[^\]]*)\]").unwrap();

        for attr in attrs {
            if attr.path().is_ident("extractor") {
                // 将 Attribute 转为 token 字符串以便用正则解析
                let mut ts = proc_macro2::TokenStream::new();
                attr.to_tokens(&mut ts);
                let s = ts.to_string();
                if name.is_none()
                    && let Some(cap) = name_re.captures(&s)
                {
                    name = Some(cap.get(1).unwrap().as_str().to_string());
                }
                if let Some(cap) = tags_re.captures(&s) {
                    let inner = cap.name("inner").unwrap().as_str();
                    for part in inner.split(',') {
                        let t = part.trim().trim_matches('"').trim().to_string();
                        if !t.is_empty() {
                            tags.push(t);
                        }
                    }
                }
            }
        }

        Ok(ExtractorAttributes {
            name: name.ok_or_else(|| {
                syn::Error::new_spanned(attrs.first().unwrap(), "Missing required attribute 'name'")
            })?,
            tags,
        })
    }
}

#[proc_macro_derive(Extractable, attributes(extractor))]
pub fn derive_extractable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let attrs = match ExtractorAttributes::from_attrs(&input.attrs) {
        Ok(attrs) => attrs,
        Err(e) => return e.to_compile_error().into(),
    };

    let extractor_name = attrs.name;
    let tags = attrs.tags;

    // 为每个提取器生成唯一的静态变量名 (全部大写以符合静态变量命名规范)
    let registry_item_name =
        format_ident!("__EXTRACTOR_REGISTRY_{}", name.to_string().to_uppercase());

    // 生成 rank 方法的实现
    let rank_impl = generate_rank_impl(&extractor_name, &tags);

    let expanded = quote! {
        impl crate::traits::ExtractorRegistry for #name {
            fn rank(&self, url: &str) -> u32 {
                #rank_impl
            }

            fn new() -> Box<dyn crate::traits::Extractor> {
                Box::new(Self {})
            }
        }

        // 生成一个返回注册项的函数, 由手动注册表收集调用
        #[allow(non_snake_case)]
        pub fn #registry_item_name() -> crate::factory::ExtractorRegistryItem {
            crate::factory::ExtractorRegistryItem {
                rank_fn: |url: &str| -> u32 {
                    #rank_impl
                },
                creator: || -> Box<dyn crate::traits::Extractor> {
                    Box::new(#name {})
                },
            }
        }
    };

    expanded.into()
}

/// 生成 rank 方法的实现
fn generate_rank_impl(name: &str, tags: &[String]) -> proc_macro2::TokenStream {
    let tag_checks: Vec<_> = tags
        .iter()
        .map(|tag| {
            let tag_lower = tag.to_lowercase();
            quote! {
                if url.to_lowercase().contains(#tag_lower) {
                    score += 10;
                }
            }
        })
        .collect();

    let name_lower = name.to_lowercase();
    quote! {
        let mut score = 0u32;

        // 基于标签匹配
        #(#tag_checks)*

        // 基于名称的精确匹配
        if url.to_lowercase().contains(#name_lower) {
            score += 20;
        }

        score
    }
}
