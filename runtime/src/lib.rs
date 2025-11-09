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
}

impl From<error::Result<Submission>> for ExtractOutput {
    fn from(res: error::Result<Submission>) -> Self {
        match res {
            Ok(sub) => ExtractOutput {
                success: true,
                error: None,
                partial: Some(sub),
            },
            Err(e) => match e {
                error::Error::Extract(ee) => ExtractOutput {
                    success: false,
                    error: Some(format!("{ee}")),
                    partial: ee.partial.map(|b| *b),
                },
                error::Error::NoExtractor(u) => ExtractOutput {
                    success: false,
                    error: Some(format!("没有找到适用于 URL 的提取器: {u}")),
                    partial: None,
                },
            },
        }
    }
}

/// 从 URL 和 HTML 内容中提取提交信息
#[wasm_bindgen]
pub fn extract_submission(url: &str, html: &str) -> JsValue {
    let res = extractor::extract(url, html);
    let extract_output = ExtractOutput::from(res);

    serde_wasm_bindgen::to_value(&extract_output)
        .unwrap_or_else(|e| JsValue::from_str(&format!("序列化错误: {e}")))
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
