//! 7fa4 提交记录

/*
 * Copyright (c) 2025 fltLi
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// 7fa4 提交记录
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Submission {
    pub code: String,
    pub pid: String,
    pub rid: String,
    pub oj: String,
    pub language: SubmissionLanguage,
    pub status: SubmissionStatus,
    #[serde(default)]
    pub total_time: i32, // ms
    #[serde(default)]
    pub max_memory: i32, // K
    pub score: i32,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SubmissionStatus {
    #[default]
    Unknown,
    Accepted,
    #[serde(rename = "Wrong Answer")]
    WrongAnswer,
    #[serde(rename = "Partially Correct")]
    PartiallyCorrect,
    #[serde(rename = "Runtime Error")]
    RuntimeError,
    #[serde(rename = "Compile Error")]
    CompileError,
    #[serde(rename = "Time Limit Exceeded")]
    TimeLimitExceeded,
    #[serde(rename = "Memory Limit Exceeded")]
    MemoryLimitExceeded,
}

impl FromStr for SubmissionStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let txt = s.replace(' ', "").to_lowercase();
        match txt.as_str() {
            "unknown" => Ok(SubmissionStatus::Unknown),
            "accepted" => Ok(SubmissionStatus::Accepted),
            "wronganswer" => Ok(SubmissionStatus::WrongAnswer),
            "partiallycorrect" => Ok(SubmissionStatus::PartiallyCorrect),
            "runtimeerror" => Ok(SubmissionStatus::RuntimeError),
            "compileerror" => Ok(SubmissionStatus::CompileError),
            "timelimitexceeded" => Ok(SubmissionStatus::TimeLimitExceeded),
            "memorylimitexceeded" => Ok(SubmissionStatus::MemoryLimitExceeded),
            other => Err(format!("unknown submission status: {other}")),
        } // 相信编译器会优化成 map !
    }
}

/// 提交语言
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SubmissionLanguage {
    #[serde(rename = "cpp14")]
    Cpp14,
    #[default]
    #[serde(rename = "cpp17")]
    Cpp17,
    #[serde(rename = "cpp11")]
    Cpp11,
    #[serde(rename = "cpp")]
    Cpp,
    #[serde(rename = "cpp-noilinux")]
    CppNoiLinux,
    #[serde(rename = "cpp11-noilinux")]
    Cpp11NoiLinux,
    #[serde(rename = "cpp11-clang")]
    Cpp11Clang,
    #[serde(rename = "cpp17-clang")]
    Cpp17Clang,
    #[serde(rename = "c")]
    C,
    #[serde(rename = "c-noilinux")]
    CNoiLinux,
}

impl FromStr for SubmissionLanguage {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let txt = s.trim().to_lowercase();
        if txt.is_empty() {
            return Err("empty language".to_string());
        }

        // 检测环境特征
        let has_clang = txt.contains("clang");
        let has_noilinux = txt.contains("noi") && txt.contains("linux");

        // 检测语言类型和版本
        if txt.contains("c++") || txt.contains("cpp") {
            match (has_clang, has_noilinux) {
                (true, _) => {
                    if txt.contains("17") {
                        Ok(SubmissionLanguage::Cpp17Clang)
                    } else {
                        Ok(SubmissionLanguage::Cpp11Clang)
                    }
                }
                (false, true) => {
                    if txt.contains("11") {
                        Ok(SubmissionLanguage::Cpp11NoiLinux)
                    } else {
                        Ok(SubmissionLanguage::CppNoiLinux)
                    }
                }
                (false, false) => {
                    if txt.contains("17") {
                        Ok(SubmissionLanguage::Cpp17)
                    } else if txt.contains("14") {
                        Ok(SubmissionLanguage::Cpp14)
                    } else if txt.contains("11") {
                        Ok(SubmissionLanguage::Cpp11)
                    } else {
                        Ok(SubmissionLanguage::Cpp)
                    }
                }
            }
        } else if txt.contains('c') && !txt.contains("c#") && !txt.contains("cs") {
            if has_noilinux {
                Ok(SubmissionLanguage::CNoiLinux)
            } else {
                Ok(SubmissionLanguage::C)
            }
        } else {
            Ok(SubmissionLanguage::Cpp17)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_parsing() {
        assert_eq!("C++".parse(), Ok(SubmissionLanguage::Cpp));
        assert_eq!("c++".parse(), Ok(SubmissionLanguage::Cpp));
        assert_eq!("C++17 O2".parse(), Ok(SubmissionLanguage::Cpp17));
        assert_eq!("C++14".parse(), Ok(SubmissionLanguage::Cpp14));
        assert_eq!("C++11".parse(), Ok(SubmissionLanguage::Cpp11));
        assert_eq!("C++".parse(), Ok(SubmissionLanguage::Cpp));
        assert_eq!("cpp".parse(), Ok(SubmissionLanguage::Cpp));
        assert_eq!("cpp17".parse(), Ok(SubmissionLanguage::Cpp17));
        assert_eq!("c++17".parse(), Ok(SubmissionLanguage::Cpp17));
        assert_eq!("C++17O2".parse(), Ok(SubmissionLanguage::Cpp17));
        assert_eq!("C++ 17".parse(), Ok(SubmissionLanguage::Cpp17));

        assert_eq!("C++17 Clang".parse(), Ok(SubmissionLanguage::Cpp17Clang));
        assert_eq!("C++11 Clang".parse(), Ok(SubmissionLanguage::Cpp11Clang));
        assert_eq!("cpp17 clang".parse(), Ok(SubmissionLanguage::Cpp17Clang));

        assert_eq!(
            "C++11 NOI Linux".parse(),
            Ok(SubmissionLanguage::Cpp11NoiLinux)
        );
        assert_eq!("C++ NOI Linux".parse(), Ok(SubmissionLanguage::CppNoiLinux));

        assert_eq!("C".parse(), Ok(SubmissionLanguage::C));
        assert_eq!("C NOI Linux".parse(), Ok(SubmissionLanguage::CNoiLinux));
        assert_eq!("c".parse(), Ok(SubmissionLanguage::C));

        assert_eq!("C#".parse(), Ok(SubmissionLanguage::Cpp17));
        assert_eq!("CSharp".parse(), Ok(SubmissionLanguage::Cpp17));
    }
}
