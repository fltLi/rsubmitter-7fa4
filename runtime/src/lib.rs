//! 后台运行环境支持 - WASM 版本

/*
 * Copyright (c) 2025 fltLi
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use extractor::error;
use extractor::models::Submission;
use extractor::utils;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// 解析后的 Cookie 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieInfo {
    pub login: Option<String>,
    #[serde(rename = "connect.sid")]
    pub connect_sid: Option<String>,
    pub chost: Option<String>,
}

/// 提取操作的输出结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractOutput {
    pub success: bool,
    pub error: Option<String>,
    pub partial: Option<Submission>,
    pub extractor_name: Option<String>,
}

/// 从 URL 和 HTML 内容中提取提交信息
#[wasm_bindgen]
pub fn extract_submission(url: &str, html: &str) -> JsValue {
    // 先创建合适的提取器以获取其名称 (用于区分 vjudge) 
    match extractor::create_extractor(url) {
        Ok((ext, name)) => match ext.extract(url, html) {
            Ok(sub) => {
                let out = ExtractOutput {
                    success: true,
                    error: None,
                    partial: Some(sub),
                    extractor_name: Some(name),
                };
                serde_wasm_bindgen::to_value(&out)
                    .unwrap_or_else(|e| JsValue::from_str(&format!("序列化错误: {e}")))
            }
            Err(e) => match e {
                error::Error::Extract(ee) => {
                    let out = ExtractOutput {
                        success: false,
                        error: Some(format!("{ee}")),
                        partial: ee.partial.map(|b| *b),
                        extractor_name: Some(name),
                    };
                    serde_wasm_bindgen::to_value(&out)
                        .unwrap_or_else(|e| JsValue::from_str(&format!("序列化错误: {e}")))
                }
                error::Error::NoExtractor(u) => {
                    let out = ExtractOutput {
                        success: false,
                        error: Some(format!("没有找到适用于 URL 的提取器: {u}")),
                        partial: None,
                        extractor_name: None,
                    };
                    serde_wasm_bindgen::to_value(&out)
                        .unwrap_or_else(|e| JsValue::from_str(&format!("序列化错误: {e}")))
                }
            },
        },
        Err(e) => {
            // 不能创建提取器
            let out = ExtractOutput {
                success: false,
                error: Some(format!("创建提取器失败: {e}")),
                partial: None,
                extractor_name: None,
            };
            serde_wasm_bindgen::to_value(&out)
                .unwrap_or_else(|e| JsValue::from_str(&format!("序列化错误: {e}")))
        }
    }
}

/// 将 VJudge 的提取结果映射为可能的原始 OJ (如果适用)
#[wasm_bindgen]
pub fn map_vjudge_submission(submission: &JsValue) -> JsValue {
    // 先将 JsValue 反序列化为 Submission
    let sub: Submission = match serde_wasm_bindgen::from_value(submission.clone()) {
        Ok(s) => s,
        Err(e) => return JsValue::from_str(&format!("反序列化错误: {e}")),
    };

    match utils::map_vjudge_to_origin(&sub) {
        Some((oj, pid, rid)) => serde_wasm_bindgen::to_value(&(oj, pid, rid))
            .unwrap_or_else(|e| JsValue::from_str(&format!("序列化错误: {e}"))),
        None => JsValue::NULL,
    }
}

/// 解析原始的 document.cookie 字符串和 origin 为结构化 Cookie 信息
#[wasm_bindgen]
pub fn parse_cookie(cookie_str: &str, origin: &str) -> JsValue {
    let mut login = None;
    let mut connect_sid = None;

    for part in cookie_str.split(';') {
        let p = part.trim();
        if p.is_empty() {
            continue;
        }
        if let Some(idx) = p.find('=') {
            let key = p[..idx].trim();
            let value = p[idx + 1..].trim();
            match key {
                "login" => login = Some(value.to_string()),
                "connect.sid" => connect_sid = Some(value.to_string()),
                _ => (),
            }
        }
    }

    let chost = if origin.contains("oj.7fa4.cn") {
        Some("oj.7fa4.cn".to_string())
    } else if origin.contains("jx.7fa4.cn") {
        Some("jx.7fa4.cn:8888".to_string())
    } else if origin.contains("in.7fa4.cn") {
        Some("in.7fa4.cn:8888".to_string())
    } else {
        url::Url::parse(origin).ok().and_then(|u| {
            u.host_str().map(|host| {
                if let Some(port) = u.port() {
                    format!("{host}:{port}")
                } else {
                    host.to_string()
                }
            })
        })
    };

    let ci = CookieInfo {
        login,
        connect_sid,
        chost,
    };

    serde_wasm_bindgen::to_value(&ci)
        .unwrap_or_else(|e| JsValue::from_str(&format!("序列化错误: {e}")))
}
