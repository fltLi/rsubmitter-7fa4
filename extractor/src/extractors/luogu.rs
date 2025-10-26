//! 洛谷提取器

use registry::Extractable;

use scraper::{Html, Selector};

use once_cell::sync::Lazy;
use regex::Regex;

use crate::error::*;
use crate::models::*;
use crate::traits::Extractor;

// 题目链接
static PROBLEM_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"/problem/(P?\d+)").unwrap());

// 提交记录链接
static RECORD_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:https?://(?:www\.)?luogu\.com\.cn)?/record/(\d+)").unwrap());

// 从文本中提取分数
static SCORE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d+)").unwrap());

/// 洛谷提取器
#[derive(Extractable)]
#[extractor(name = "洛谷", tags = ["luogu", "Luogu"])]
pub struct LuoguExtractor {}

impl LuoguExtractor {
    fn extract_basic_info(document: &Html) -> (String, i32, i32) {
        let mut language = String::new();
        let mut total_time: i32 = 0;
        let mut max_memory: i32 = 0;

        if let Ok(stat_sel) = Selector::parse(".stat.color-inverse")
            && let Some(stat_el) = document.select(&stat_sel).next()
            && let Ok(field_sel) = Selector::parse(".field")
        {
            for field in stat_el.select(&field_sel) {
                let key = field
                    .select(&Selector::parse(".key").unwrap())
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();
                let value = field
                    .select(&Selector::parse(".value").unwrap())
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();

                match key.as_str() {
                    "编程语言" => language = value,
                    "用时" => {
                        if let Some(v) = crate::utils::parse_time_to_ms(&value) {
                            total_time = v;
                        }
                    }
                    "内存" => {
                        if let Some(v) = crate::utils::parse_mem_to_kb(&value) {
                            max_memory = v;
                        }
                    }
                    _ => {}
                }
            }
        }

        (language, total_time, max_memory)
    }

    fn extract_code(document: &Html) -> String {
        let mut code = String::new();
        if let Ok(code_sel) = Selector::parse("code") {
            for el in document.select(&code_sel) {
                if let Some(cl) = el.value().attr("class")
                    && cl.contains("language-")
                {
                    code = el.text().collect::<String>().trim().to_string();
                    break;
                }
            }
            if code.is_empty()
                && let Some(el) = document.select(&code_sel).next()
            {
                code = el.text().collect::<String>().trim().to_string();
            }
        }
        if code.is_empty()
            && let Ok(pre_sel) = Selector::parse("pre")
            && let Some(el) = document.select(&pre_sel).next()
        {
            code = el.text().collect::<String>().trim().to_string();
        }
        code
    }

    fn extract_pid(document: &Html) -> String {
        let mut pid = String::new();
        if let Ok(a_sel) = Selector::parse("a") {
            for a in document.select(&a_sel) {
                if let Some(href) = a.value().attr("href")
                    && href.contains("/problem/")
                    && let Some(caps) = PROBLEM_REGEX.captures(href)
                    && let Some(m) = caps.get(1)
                {
                    pid = m.as_str().to_string();
                    break;
                }
            }
        }
        pid
    }

    fn extract_status_and_score(document: &Html) -> (SubmissionStatus, i32) {
        let mut status = SubmissionStatus::Unknown;
        let mut score: i32 = 0;

        if let Ok(rows_sel) = Selector::parse(".info-rows div") {
            for row in document.select(&rows_sel) {
                let row_text = row.text().collect::<String>();
                if row_text.contains("评测状态") {
                    let txt = row_text
                        .split_whitespace()
                        .last()
                        .map(|s| s.trim().to_string())
                        .unwrap_or_default();
                    status = txt
                        .parse::<SubmissionStatus>()
                        .unwrap_or(SubmissionStatus::Unknown);
                }

                if row_text.contains("评测分数")
                    && let Some(caps) = SCORE_REGEX.captures(&row_text)
                    && let Some(m) = caps.get(1)
                {
                    score = m.as_str().parse().unwrap_or(0);
                }
            }
        }

        (status, score)
    }

    fn extract_rid(url: &str) -> Option<String> {
        RECORD_REGEX
            .captures(url)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
    }

    fn validate_submission(sub: &Submission) -> Result<()> {
        if sub.pid.is_empty() {
            return Err(Error::Extract(ExtractError::new(
                ExtractErrorKind::MissingField("pid".to_string()),
            )));
        }
        if sub.rid.is_empty() {
            return Err(Error::Extract(ExtractError::new(
                ExtractErrorKind::MissingField("rid".to_string()),
            )));
        }
        if sub.code.is_empty() {
            return Err(Error::Extract(ExtractError::with_partial(
                ExtractErrorKind::Parse("source code empty".to_string()),
                sub.clone(),
            )));
        }
        Ok(())
    }
}

impl Extractor for LuoguExtractor {
    fn extract(&self, url: &str, content: &str) -> Result<Submission> {
        let document = Html::parse_document(content);

        let (language_text, total_time, max_memory) = LuoguExtractor::extract_basic_info(&document);
        let code = LuoguExtractor::extract_code(&document);
        let pid = LuoguExtractor::extract_pid(&document);
        let (status, score) = LuoguExtractor::extract_status_and_score(&document);

        let rid_val = LuoguExtractor::extract_rid(url).unwrap_or_default();
        let submission = Submission {
            code,
            pid,
            rid: rid_val,
            oj: "luogu".to_string(),
            language: if language_text.is_empty() {
                SubmissionLanguage::Cpp17
            } else {
                language_text.parse().unwrap_or(SubmissionLanguage::Cpp17)
            },
            status,
            total_time,
            max_memory,
            score,
        };

        LuoguExtractor::validate_submission(&submission)?;

        Ok(submission)
    }
}
