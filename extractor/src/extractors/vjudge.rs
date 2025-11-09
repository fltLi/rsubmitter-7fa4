//! VJudge 提取器

/*
 * Copyright (c) 2025 fltLi
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use once_cell::sync::Lazy;
use regex::Regex;
use registry::Extractable;
use scraper::{Html, Selector};

use crate::error::*;
use crate::models::*;
use crate::traits::Extractor;
use crate::utils::*;

// 提交记录链接
static RECORD_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"https://vjudge\.net/solution/(\d+)").unwrap());

// 题目链接正则
static PROBLEM_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"/problem/([^/]+)").unwrap());

// 远程提交 ID 提取
static REMOTE_RUN_ID_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"[a-f0-9]{24}").unwrap());

/// VJudge 提取器
#[derive(Extractable)]
#[extractor(name = "vj", tags = ["vjudge", "Virtual Judge"])]
pub struct VjudgeExtractor;

impl VjudgeExtractor {
    /// 提取代码
    fn extract_code(document: &Html) -> String {
        let Ok(code_selector) = Selector::parse("pre code") else {
            return String::new();
        };

        if let Some(code_element) = document.select(&code_selector).next() {
            return code_element.text().collect::<String>().trim().to_string();
        }

        // 备用选择器
        let Ok(pre_selector) = Selector::parse("pre") else {
            return String::new();
        };

        document
            .select(&pre_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default()
    }

    /// 提取题目 ID
    fn extract_pid(document: &Html) -> String {
        // 从 modal title 中提取题目链接
        let Ok(modal_title_selector) = Selector::parse(".modal-title a") else {
            return String::new();
        };

        for link in document.select(&modal_title_selector) {
            if let Some(href) = link.value().attr("href")
                && href.contains("/problem/")
                && let Some(caps) = PROBLEM_REGEX.captures(href)
                && let Some(pid_match) = caps.get(1)
            {
                return pid_match.as_str().to_string();
            }
        }

        String::new()
    }

    /// 提取提交 ID
    fn extract_rid(url: &str, document: &Html) -> String {
        // 首先尝试从 URL 中提取
        if let Some(caps) = RECORD_REGEX.captures(url)
            && let Some(rid_match) = caps.get(1)
        {
            return rid_match.as_str().to_string();
        }

        // 备用方案: 从模态框标题中提取
        let Ok(modal_title_selector) = Selector::parse(".modal-title a[href^='/solution/']") else {
            return String::new();
        };

        for link in document.select(&modal_title_selector) {
            if let Some(href) = link.value().attr("href") {
                if let Some(caps) = RECORD_REGEX.captures(href)
                    && let Some(rid_match) = caps.get(1)
                {
                    return rid_match.as_str().to_string();
                }
                // 备用: 直接解析 /solution/ 后面的数字
                if href.starts_with("/solution/")
                    && let Some(rid) = href.strip_prefix("/solution/")
                {
                    return rid.to_string();
                }
            }
        }

        // 从表格行的 id 属性中提取
        let Ok(row_selector) = Selector::parse("tr[id]") else {
            return String::new();
        };

        for row in document.select(&row_selector) {
            if let Some(id) = row.value().attr("id") {
                // 检查 id 是否是纯数字 (提交ID) 
                if id.chars().all(|c| c.is_ascii_digit()) {
                    return id.to_string();
                }
            }
        }

        String::new()
    }

    /// 提取远程提交 ID
    fn extract_remote_run_id(document: &Html) -> String {
        let Ok(remote_run_selector) = Selector::parse(".remote-run-id a") else {
            return String::new();
        };

        if let Some(link) = document.select(&remote_run_selector).next() {
            let text = link.text().collect::<String>();
            if let Some(caps) = REMOTE_RUN_ID_REGEX.captures(&text) {
                return caps
                    .get(0)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
            }
        }

        String::new()
    }

    /// 提取编程语言
    fn extract_language(document: &Html) -> SubmissionLanguage {
        let Ok(info_table_selector) = Selector::parse("#info-panel table tbody tr") else {
            return SubmissionLanguage::default();
        };

        for row in document.select(&info_table_selector) {
            let (Ok(th_selector), Ok(td_selector)) = (Selector::parse("th"), Selector::parse("td"))
            else {
                continue;
            };

            if let Some(th) = row.select(&th_selector).next() {
                let header_text = th.text().collect::<String>().to_lowercase();
                if header_text.contains("语言")
                    && let Some(td) = row.select(&td_selector).next()
                {
                    let lang_text = td.text().collect::<String>().trim().to_string();
                    return lang_text.parse().unwrap_or(SubmissionLanguage::Cpp17);
                }
            }
        }

        // 备用: 从语言列的 tooltip 中提取
        let Ok(lang_tooltip_selector) = Selector::parse(".language div[data-original-title]")
        else {
            return SubmissionLanguage::default();
        };

        if let Some(lang_div) = document.select(&lang_tooltip_selector).next()
            && let Some(tooltip) = lang_div.value().attr("data-original-title")
        {
            return tooltip.parse().unwrap_or(SubmissionLanguage::Cpp17);
        }

        SubmissionLanguage::default()
    }

    /// 提取评测状态
    fn extract_status(document: &Html) -> SubmissionStatus {
        let Ok(status_selector) = Selector::parse(".status .view-solution") else {
            return SubmissionStatus::default();
        };

        if let Some(status_div) = document.select(&status_selector).next() {
            let status_text = status_div.text().collect::<String>().trim().to_string();
            return status_text.parse().unwrap_or(SubmissionStatus::Unknown);
        }

        // 从 info panel 中提取
        let Ok(info_table_selector) = Selector::parse("#info-panel table tbody tr") else {
            return SubmissionStatus::default();
        };

        for row in document.select(&info_table_selector) {
            let (Ok(th_selector), Ok(td_selector)) = (Selector::parse("th"), Selector::parse("td"))
            else {
                continue;
            };

            if let Some(th) = row.select(&th_selector).next() {
                let header_text = th.text().collect::<String>().to_lowercase();
                if header_text.contains("评测结果")
                    && let Some(td) = row.select(&td_selector).next()
                {
                    let status_text = td.text().collect::<String>().trim().to_string();
                    return status_text.parse().unwrap_or(SubmissionStatus::Unknown);
                }
            }
        }

        SubmissionStatus::default()
    }

    /// 提取时间和内存
    fn extract_time_and_memory(document: &Html) -> (i32, i32) {
        let mut total_time = 0;
        let mut max_memory = 0;

        // 从表格中提取
        let Ok(runtime_selector) = Selector::parse(".runtime") else {
            return (total_time, max_memory);
        };
        let Ok(memory_selector) = Selector::parse(".memory") else {
            return (total_time, max_memory);
        };

        if let Some(runtime_td) = document.select(&runtime_selector).next() {
            let time_text = runtime_td.text().collect::<String>().trim().to_string();
            total_time = parse_time_to_ms(&time_text).unwrap_or(0);
        }

        if let Some(memory_td) = document.select(&memory_selector).next() {
            let mem_text = memory_td.text().collect::<String>().trim().to_string();
            max_memory = parse_mem_to_kb(&mem_text).unwrap_or(0);
        }

        // 从 info panel 中提取 (备用)
        if total_time == 0 || max_memory == 0 {
            let Ok(info_table_selector) = Selector::parse("#info-panel table tbody tr") else {
                return (total_time, max_memory);
            };

            for row in document.select(&info_table_selector) {
                let (Ok(th_selector), Ok(td_selector)) =
                    (Selector::parse("th"), Selector::parse("td"))
                else {
                    continue;
                };

                if let Some(th) = row.select(&th_selector).next() {
                    let header_text = th.text().collect::<String>().to_lowercase();
                    if let Some(td) = row.select(&td_selector).next() {
                        let value_text = td.text().collect::<String>().trim().to_string();

                        if header_text.contains("耗时") {
                            total_time = parse_time_to_ms(&value_text).unwrap_or(total_time);
                        } else if header_text.contains("内存消耗") {
                            max_memory = parse_mem_to_kb(&value_text).unwrap_or(max_memory);
                        }
                    }
                }
            }
        }

        (total_time, max_memory)
    }

    /// 提取得分
    fn extract_score(status: &SubmissionStatus) -> i32 {
        match status {
            SubmissionStatus::Accepted => 100,
            SubmissionStatus::PartiallyCorrect => 50, // 部分正确的情况
            _ => 0,
        }
    }

    /// 提取 OJ 名称
    fn extract_oj(document: &Html) -> String {
        let Ok(oj_selector) = Selector::parse(".oj") else {
            return "vj".to_string();
        };

        if let Some(oj_td) = document.select(&oj_selector).next() {
            return oj_td.text().collect::<String>().trim().to_string();
        }

        "vj".to_string()
    }

    fn extract_partial(&self, url: &str, content: &str) -> Submission {
        let document = Html::parse_document(content);

        let code = Self::extract_code(&document);
        let pid = Self::extract_pid(&document);
        let rid = Self::extract_rid(url, &document);
        let language = Self::extract_language(&document);
        let status = Self::extract_status(&document);
        let (total_time, max_memory) = Self::extract_time_and_memory(&document);
        let score = Self::extract_score(&status);
        let oj = Self::extract_oj(&document);

        Submission {
            code,
            pid,
            rid,
            oj,
            language,
            status,
            total_time,
            max_memory,
            score,
        }
    }

    /// 验证提取结果
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

impl Extractor for VjudgeExtractor {
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
    let url = "https://vjudge.net/solution/65377961";
    let content = r#"
        <div class="modal-content">
            <div class="modal-header">
                <h5 class="modal-title">
                    <a href="/solution/65377961">#65377961</a>
                    <a href="/problem/UESTC-126">[UESTC-126]</a>
                </h5>
            </div>
            <div class="modal-body">
                <div id="info-panel">
                    <table>
                        <tbody>
                            <tr>
                                <th>评测结果</th>
                                <td class="status">Accepted</td>
                            </tr>
                            <tr>
                                <th>耗时</th>
                                <td class="time">1886ms</td>
                            </tr>
                            <tr>
                                <th>内存消耗</th>
                                <td class="memory">10752kB</td>
                            </tr>
                            <tr>
                                <th>语言</th>
                                <td class="lang">C++17 (O2)</td>
                            </tr>
                        </tbody>
                    </table>
                </div>
                <div id="code-panel">
                    <pre>
                        <code>
                        #include &lt;bits/stdc++.h&gt;
                        auto main() -> int { return 0; }
                        </code>
                    </pre>
                </div>
            </div>
        </div>
        <table>
            <tbody>
                <tr>
                    <td class="oj">UESTC</td>
                    <td class="status">Accepted</td>
                    <td class="runtime">1886</td>
                    <td class="memory">10.8</td>
                </tr>
            </tbody>
        </table>
    "#;

    let extractor = VjudgeExtractor;
    let submission = extractor.extract(url, content)?;

    assert_eq!(submission.pid, "UESTC-126");
    assert_eq!(submission.rid, "65377961");
    assert_eq!(submission.oj, "UESTC");
    assert_eq!(submission.language, SubmissionLanguage::Cpp17);
    assert_eq!(submission.status, SubmissionStatus::Accepted);
    assert_eq!(submission.total_time, 1886);
    assert_eq!(submission.max_memory, 10752);
    assert_eq!(submission.score, 100);

    // println!("{}", submission.code);

    Ok(())
}
