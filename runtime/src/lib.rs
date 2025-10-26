//! 提交器后台运行环境支持

#![allow(dead_code)]

use extractor::error;
use extractor::models::Submission;
use serde::{Deserialize, Serialize};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// 解析后的Cookie信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieInfo {
    /// login cookie 值
    pub login: Option<String>,
    /// connect.sid cookie 值
    #[serde(rename = "connect.sid")]
    pub connect_sid: Option<String>,
    /// 用于发送请求的主机 (例如 "oj.7fa4.cn" 或 "jx.7fa4.cn:8888")
    pub chost: Option<String>,
}

/// 提取成功时生成的HTTP请求规范
#[derive(Debug, Serialize, Deserialize)]
pub struct RequestSpec {
    pub url: String,
    pub method: String,
    pub headers: serde_json::Value,
    pub body: serde_json::Value,
}

/// 调用 `Runtime::extract` 或 `parse_cookie` 后返回给JS的输出
#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractOutput {
    pub success: bool,
    pub error: Option<String>,
    pub partial: Option<Submission>,
    pub request: Option<RequestSpec>,
}

impl From<error::Result<Submission>> for ExtractOutput {
    fn from(res: error::Result<Submission>) -> Self {
        match res {
            Ok(sub) => ExtractOutput {
                success: true,
                error: None,
                partial: Some(sub), // 成功时包含提取的提交数据
                request: None,
            },
            Err(e) => match e {
                error::Error::Extract(ee) => ExtractOutput {
                    success: false,
                    error: Some(format!("{ee}")),
                    partial: ee.partial.map(|b| *b),
                    request: None,
                },
                error::Error::NoExtractor(u) => ExtractOutput {
                    success: false,
                    error: Some(format!("没有找到适用于URL的提取器: {u}")),
                    partial: None,
                    request: None,
                },
            },
        }
    }
}

/// 导出到WASM的运行时。保存解析的cookie信息并生成请求规范。
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct Runtime {
    cookies: CookieInfo,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Runtime {
    /// 从解析的cookie对象创建运行时
    #[wasm_bindgen(constructor)]
    pub fn new(js_cookies: JsValue) -> Result<Runtime, JsValue> {
        let ci: CookieInfo = serde_wasm_bindgen::from_value(js_cookies)
            .map_err(|e| JsValue::from_str(&format!("无效的cookies: {e}")))?;
        Ok(Runtime { cookies: ci })
    }

    /// 在 (url, html) 上运行提取器并返回JSON友好的输出
    #[wasm_bindgen]
    pub fn extract(&self, url: &str, html: &str, in_contest: bool) -> JsValue {
        let res = extractor::extract(url, html);

        let extract_output = ExtractOutput::from(res);

        // 只有在提取成功时才创建请求
        if extract_output.success
            && let Some(sub) = extract_output.partial
        {
            // 附加 in_contest 标志
            let mut body = serde_json::to_value(&sub).unwrap_or_else(|_| serde_json::json!({}));
            if let serde_json::Value::Object(ref mut map) = body {
                map.insert(
                    "in_contest".to_string(),
                    serde_json::Value::Bool(in_contest),
                );
            }

            let chost = self
                .cookies
                .chost
                .clone()
                .unwrap_or_else(|| "oj.7fa4.cn".to_string());
            let target = format!("http://{chost}/foreign_oj");

            let cookie_header = if let (Some(login), Some(sid)) =
                (&self.cookies.login, &self.cookies.connect_sid)
            {
                format!("login={login}; connect.sid={sid}")
            } else if let Some(login) = &self.cookies.login {
                format!("login={login}")
            } else {
                String::new()
            };

            let headers = serde_json::json!({
                "Content-Type": "application/json",
                "Cookie": cookie_header
            });

            let request = RequestSpec {
                url: target,
                method: "POST".to_string(),
                headers,
                body,
            };

            let out = ExtractOutput {
                success: true,
                error: None,
                partial: Some(sub),
                request: Some(request),
            };

            return serde_wasm_bindgen::to_value(&out)
                .unwrap_or_else(|e| JsValue::from_str(&format!("序列化错误: {e}")));
        }

        // 如果提取失败，返回原始的 extract_output
        serde_wasm_bindgen::to_value(&extract_output)
            .unwrap_or_else(|e| JsValue::from_str(&format!("序列化错误: {e}")))
    }
}

/// 解析原始的 `document.cookie` 字符串和origin为CookieInfo
/// 此函数导出到JS，因此解析逻辑保留在Rust中
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn parse_cookie(cookie_str: &str, origin: &str) -> JsValue {
    let mut login: Option<String> = None;
    let mut connect_sid: Option<String> = None;

    // 解析cookie字符串
    for part in cookie_str.split(';') {
        let p = part.trim();
        if p.is_empty() {
            continue;
        }
        if let Some(idx) = p.find('=') {
            let k = p[..idx].trim();
            let v = p[idx + 1..].trim();
            match k {
                "login" => login = Some(v.to_string()),
                "connect.sid" => connect_sid = Some(v.to_string()),
                _ => (),
            }
        }
    }

    // 根据origin确定chost，匹配已知的主机模式，回退到origin主机
    let chost = if origin.contains("oj.7fa4.cn") {
        Some("oj.7fa4.cn".to_string())
    } else if origin.contains("jx.7fa4.cn") {
        Some("jx.7fa4.cn:8888".to_string())
    } else if origin.contains("in.7fa4.cn") {
        Some("in.7fa4.cn:8888".to_string())
    } else {
        // 尝试从origin提取主机
        if let Ok(u) = url::Url::parse(origin) {
            Some(
                u.host_str().unwrap_or("").to_string()
                    + &(if let Some(port) = u.port() {
                        format!(":{port}")
                    } else {
                        "".to_string()
                    }),
            )
        } else {
            None
        }
    };

    let ci = CookieInfo {
        login,
        connect_sid,
        chost,
    };
    serde_wasm_bindgen::to_value(&ci)
        .unwrap_or_else(|e| JsValue::from_str(&format!("序列化错误: {e}")))
}

// 用于本地测试 / 服务器的非 WASM 存根
#[cfg(not(feature = "wasm"))]
pub struct RuntimeNonWasm;

#[cfg(not(feature = "wasm"))]
impl RuntimeNonWasm {
    pub fn new(_cookies: CookieInfo) -> Self {
        RuntimeNonWasm {}
    }
}
