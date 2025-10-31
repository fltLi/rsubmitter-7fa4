//! 信友队提取器

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

// 题目链接
static PROBLEM_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"题目ID：\s*(\d+)").unwrap());

// 提交记录链接
static RECORD_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"https://(?:www\.)?xinyoudui\.com/ac/contest/.*?/problem/(\d+)").unwrap()
});

// 从编译结果中提取时间和内存
static TIME_MEM_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"time: (\d+)ms, memory: (\d+)kb").unwrap());

// 从得分文本中提取分数
static SCORE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d+)\s*分").unwrap());

/// 信友队提取器
#[derive(Extractable)]
#[extractor(name = "xyd", tags = ["xinyoudui", "信友队"])]
pub struct XinyouduiExtractor;

impl XinyouduiExtractor {
    /// 提取代码
    fn extract_code(document: &Html) -> String {
        let Ok(code_selector) = Selector::parse(".cm-line") else {
            return String::new();
        };

        let code_lines: Vec<String> = document
            .select(&code_selector)
            .map(|element| {
                let text = element.text().collect::<String>();
                text.trim_end().to_string()
            })
            .collect();

        if code_lines.is_empty() {
            return String::new();
        }

        code_lines.join("\n") + "\n"
    }

    /// 提取题目 ID
    fn extract_pid(url: &str, document: &Html) -> String {
        if let Some(pid_from_page) = Self::extract_pid_from_page(document) {
            return pid_from_page;
        }

        RECORD_REGEX
            .captures(url)
            .and_then(|caps| caps.get(1))
            .map(|pid_match| pid_match.as_str().to_string())
            .unwrap_or_default()
    }

    /// 从页面中提取题目 ID
    fn extract_pid_from_page(document: &Html) -> Option<String> {
        let Ok(tag_selector) = Selector::parse(".ac-ant-tag") else {
            return None;
        };

        for element in document.select(&tag_selector) {
            let text = element.text().collect::<String>();
            if let Some(caps) = PROBLEM_REGEX.captures(&text)
                && let Some(pid_match) = caps.get(1)
            {
                return Some(pid_match.as_str().to_string());
            }
        }

        None
    }

    /// 提取提交ID
    fn extract_rid(document: &Html) -> String {
        let (Ok(selected_row_selector), Ok(td_selector)) = (
            Selector::parse("tr.ac-ant-table-row-selected"),
            Selector::parse("td"),
        ) else {
            return String::new();
        };

        if let Some(selected_row) = document.select(&selected_row_selector).next()
            && let Some(first_td) = selected_row.select(&td_selector).next()
        {
            return first_td.text().collect::<String>().trim().to_string();
        }

        String::new()
    }

    /// 提取编程语言
    fn extract_language(document: &Html) -> SubmissionLanguage {
        let (Ok(selected_row_selector), Ok(td_selector)) = (
            Selector::parse("tr.ac-ant-table-row-selected"),
            Selector::parse("td"),
        ) else {
            return SubmissionLanguage::Cpp17;
        };

        if let Some(selected_row) = document.select(&selected_row_selector).next() {
            let tds: Vec<_> = selected_row.select(&td_selector).collect();
            if tds.len() >= 2 {
                let language_text = tds[1].text().collect::<String>().trim().to_string();
                return language_text.parse().unwrap_or(SubmissionLanguage::Cpp17);
            }
        }

        SubmissionLanguage::default()
    }

    /// 提取状态和得分
    fn extract_status_and_score(document: &Html) -> (SubmissionStatus, i32) {
        let (Ok(selected_row_selector), Ok(td_selector)) = (
            Selector::parse("tr.ac-ant-table-row-selected"),
            Selector::parse("td"),
        ) else {
            return (SubmissionStatus::default(), 0);
        };

        if let Some(selected_row) = document.select(&selected_row_selector).next() {
            let tds: Vec<_> = selected_row.select(&td_selector).collect();

            // 提取状态 (第三列)
            let status = if tds.len() >= 3 {
                tds[2]
                    .text()
                    .collect::<String>()
                    .trim()
                    .parse()
                    .unwrap_or(SubmissionStatus::Unknown)
            } else {
                SubmissionStatus::default()
            };

            // 提取得分 (第四列)
            let score = if tds.len() >= 4 {
                let score_text = tds[3].text().collect::<String>();
                SCORE_REGEX
                    .captures(&score_text)
                    .and_then(|caps| caps.get(1))
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(0)
            } else {
                0
            };

            return (status, score);
        }

        (SubmissionStatus::default(), 0)
    }

    /// 提取时间和内存
    fn extract_time_and_memory(document: &Html) -> (i32, i32) {
        let Ok(compilation_selector) = Selector::parse("._compilation_1f8cm_53") else {
            return (0, 0);
        };

        if let Some(compilation_div) = document.select(&compilation_selector).next() {
            let compilation_text = compilation_div.text().collect::<String>();

            if let Some(caps) = TIME_MEM_REGEX.captures(&compilation_text) {
                let time = caps
                    .get(1)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(0);
                let memory = caps
                    .get(2)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(0);
                return (time, memory);
            }
        }

        (0, 0)
    }

    fn extract_partial(&self, url: &str, content: &str) -> Submission {
        let document = Html::parse_document(content);

        let code = Self::extract_code(&document);
        let pid = Self::extract_pid(url, &document);
        let rid = Self::extract_rid(&document);
        let language = Self::extract_language(&document);
        let (status, score) = Self::extract_status_and_score(&document);
        let (total_time, max_memory) = Self::extract_time_and_memory(&document);

        Submission {
            code,
            pid,
            rid,
            oj: "xyd".to_string(),
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

impl Extractor for XinyouduiExtractor {
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
fn test_extractor() -> Result<()> {
    let url = "https://www.xinyoudui.com/ac/contest/74700B6AA0008E906FED34/problem/15569";
    let content = r#"
        <div id="rc-tabs-0-panel-submissions">
            <div class="_overview_10upj_43">
                <div class="_top_10upj_56">
                    <div class="_left_10upj_61">
                        <div class="_tags_10upj_68 print-hide">
                            <span class="ac-ant-tag css-oxq8ps">题目ID：23051</span>
                            <span class="ac-ant-tag ac-ant-tag-blue css-oxq8ps">必做题</span>
                        </div>
                    </div>
                </div>
            </div>
            <table>
                <tbody>
                    <tr class="ac-ant-table-row ac-ant-table-row-selected">
                        <td>2542938</td>
                        <td>C++17</td>
                        <td>Accepted</td>
                        <td><strong>100 分</strong></td>
                    </tr>
                </tbody>
            </table>
            <div class="_codingArea_hyhtw_77">
                <div class="cm-theme-light _codeMirror_hyhtw_81 x-star-design-codeMirror">
                    <div class="cm-content">
                        <div class="cm-line">#include &lt;bits/stdc++.h&gt;</div>
                        <div class="cm-line">using namespace std;</div>
                        <div class="cm-line">int main() {</div>
                        <div class="cm-line">    return 0;</div>
                        <div class="cm-line">}</div>
                    </div>
                </div>
            </div>
            <div class="_compilation_1f8cm_53">
                time: 350ms, memory: 141628kb, score: 100, status: Accepted
            </div>
        </div>
        "#;

    let extractor = XinyouduiExtractor;
    let submission = extractor.extract(url, content)?;

    assert_eq!(submission.pid, "23051");
    assert_eq!(submission.rid, "2542938");
    assert_eq!(submission.language, SubmissionLanguage::Cpp17);
    assert_eq!(submission.status, SubmissionStatus::Accepted);
    assert_eq!(submission.score, 100);
    assert_eq!(submission.total_time, 350);
    assert_eq!(submission.max_memory, 141628);

    // println!("{}", submission.code);

    Ok(())
}
