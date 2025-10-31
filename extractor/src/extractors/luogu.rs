//! 洛谷提取器

use registry::Extractable;

use scraper::{Html, Selector};

use once_cell::sync::Lazy;
use regex::Regex;

use crate::error::*;
use crate::models::*;
use crate::traits::Extractor;
use crate::utils::*;

// 题目链接
static PROBLEM_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"/problem/(P?\d+)").unwrap());

// 提交记录链接
static RECORD_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:https?://(?:www\.)?luogu\.com\.cn)?/record/(\d+)").unwrap());

// 从文本中提取分数
static SCORE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d+)").unwrap());

/// 洛谷提取器
#[derive(Extractable)]
#[extractor(name = "luogu", tags = ["洛谷"])]
pub struct LuoguExtractor {}

impl LuoguExtractor {
    fn extract_basic_info(document: &Html) -> (String, i32, i32) {
        let mut language = String::new();
        let mut total_time = 0;
        let mut max_memory = 0;

        let Ok(stat_sel) = Selector::parse(".stat.color-inverse") else {
            return (language, total_time, max_memory);
        };

        if let Some(stat_el) = document.select(&stat_sel).next() {
            let Ok(field_sel) = Selector::parse(".field") else {
                return (language, total_time, max_memory);
            };

            for field in stat_el.select(&field_sel) {
                let (Ok(key_sel), Ok(value_sel)) =
                    (Selector::parse(".key"), Selector::parse(".value"))
                else {
                    continue;
                };

                let key = field
                    .select(&key_sel)
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();
                let value = field
                    .select(&value_sel)
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();

                match key.as_str() {
                    "编程语言" => language = value,
                    "用时" => total_time = parse_time_to_ms(&value).unwrap_or(0),
                    "内存" => max_memory = parse_mem_to_kb(&value).unwrap_or(0),
                    _ => {}
                }
            }
        }

        (language, total_time, max_memory)
    }

    fn extract_code(document: &Html) -> String {
        let Ok(code_sel) = Selector::parse("code") else {
            return String::new();
        };

        for el in document.select(&code_sel) {
            if let Some(cl) = el.value().attr("class")
                && cl.contains("language-")
            {
                return el.text().collect::<String>().trim().to_string();
            }
        }

        if let Some(el) = document.select(&code_sel).next() {
            return el.text().collect::<String>().trim().to_string();
        }

        let Ok(pre_sel) = Selector::parse("pre") else {
            return String::new();
        };

        document
            .select(&pre_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default()
    }

    fn extract_pid(document: &Html) -> String {
        let Ok(a_sel) = Selector::parse("a") else {
            return String::new();
        };

        for a in document.select(&a_sel) {
            if let Some(href) = a.value().attr("href")
                && href.contains("/problem/")
                && let Some(caps) = PROBLEM_REGEX.captures(href)
                && let Some(m) = caps.get(1)
            {
                return m.as_str().to_string();
            }
        }

        String::new()
    }

    fn extract_status_and_score(document: &Html) -> (SubmissionStatus, i32) {
        let mut status = SubmissionStatus::Unknown;
        let mut score = 0;

        let Ok(rows_sel) = Selector::parse(".info-rows div") else {
            return (status, score);
        };

        for row in document.select(&rows_sel) {
            let row_text = row.text().collect::<String>();
            if row_text.contains("评测状态") {
                let txt = row_text
                    .split_whitespace()
                    .last()
                    .map(|s| s.trim())
                    .unwrap_or("");
                status = txt.parse().unwrap_or(SubmissionStatus::Unknown);
            }

            if row_text.contains("评测分数")
                && let Some(caps) = SCORE_REGEX.captures(&row_text)
            {
                score = caps
                    .get(1)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(0);
            }
        }

        (status, score)
    }

    fn extract_rid(url: &str) -> String {
        RECORD_REGEX
            .captures(url)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_default()
    }

    fn extract_partial(&self, url: &str, content: &str) -> Submission {
        let document = Html::parse_document(content);

        let (language_text, total_time, max_memory) = Self::extract_basic_info(&document);
        let code = Self::extract_code(&document);
        let pid = Self::extract_pid(&document);
        let (status, score) = Self::extract_status_and_score(&document);
        let rid = Self::extract_rid(url);

        let language = language_text.parse().unwrap_or_default();

        Submission {
            code,
            pid,
            rid,
            oj: "luogu".to_string(),
            language,
            status,
            total_time,
            max_memory,
            score,
        }
    }

    fn validate_submission(sub: &Submission) -> Result<()> {
        if sub.pid.is_empty() {
            return Err(Error::Extract(ExtractError::with_partial(
                ExtractErrorKind::MissingField("pid".to_string()),
                sub.clone(),
            )));
        }
        if sub.rid.is_empty() {
            return Err(Error::Extract(ExtractError::with_partial(
                ExtractErrorKind::MissingField("rid".to_string()),
                sub.clone(),
            )));
        }
        if sub.code.is_empty() {
            return Err(Error::Extract(ExtractError::with_partial(
                ExtractErrorKind::MissingField("code".to_string()),
                sub.clone(),
            )));
        }
        Ok(())
    }
}

impl Extractor for LuoguExtractor {
    fn extract(&self, url: &str, content: &str) -> Result<Submission> {
        if content.trim().is_empty() {
            return Err(Error::Extract(ExtractError::new(
                ExtractErrorKind::EmptyContent,
            )));
        }

        let submission = self.extract_partial(url, content);

        Self::validate_submission(&submission)?;
        Ok(submission)
    }
}

#[test]
fn test_extract() -> Result<()> {
    let url = "https://www.luogu.com.cn/record/241494617";
    let content = r#"
        <!DOCTYPE html>
        <html>
        <body>
            <div class="stat color-inverse">
                <div class="field">
                    <span class="key">编程语言</span>
                    <span class="value">C++17 O2</span>
                </div>
                <div class="field">
                    <span class="key">用时</span>
                    <span class="value">2.33s</span>
                </div>
                <div class="field">
                    <span class="key">内存</span>
                    <span class="value">1.55MB</span>
                </div>
            </div>

            <div class="info-rows">
                <div>
                    <span>评测状态</span>
                    <span style="color: rgb(82, 196, 26);">Accepted</span>
                </div>
                <div>
                    <span>评测分数</span>
                    <span style="font-weight: bold; color: rgb(82, 196, 26);">100</span>
                </div>
            </div>

            <a href="/problem/P4198">P4198 楼房重建</a>

            <pre><code class="language-cpp">
                #include &lt;bits/stdc++.h&gt;
                using u32 = uint32_t; using u64 = uint64_t;
                constexpr u32 N = 1e5 + 10, M = 320;
                template &lt;typename T&gt;
                void read(T&amp; v) {
                    v = 0; char ch;
                    while (!isdigit(ch = getchar()));
                    do { v = (v &lt;&lt; 1) + (v &lt;&lt; 3) + (ch ^ '0'); } while (isdigit(ch = getchar()));
                }

                struct Block {
                    u32 max;
                    std::vector&lt;u32&gt; cnt;
                };

                u32 n, b, cnt, h[N];
                Block par[M];

                auto main() -&gt; int {
                    u32 m, u, v, cnt = 0;
                    read(n), read(m), b = sqrt(n);
                    while (m--) {
                        read(u), read(v);
                        printf("%u\n", modify(u, v) ? cnt = count() : cnt);
                    }
                }
            </code></pre>
        </body>
        </html>"#;

    let submission = LuoguExtractor {}.extract(url, content)?;

    assert_eq!(submission.pid, "P4198".to_string());
    assert_eq!(submission.rid, "241494617".to_string());
    assert_eq!(submission.language, SubmissionLanguage::Cpp17);
    assert_eq!(submission.status, SubmissionStatus::Accepted);
    assert_eq!(submission.max_memory, parse_mem_to_kb("1.55MB").unwrap());
    assert_eq!(submission.total_time, parse_time_to_ms("2.33s").unwrap());

    // println!("{}", submission.code);

    Ok(())
}
