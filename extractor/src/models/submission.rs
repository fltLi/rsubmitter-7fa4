//! 7fa4 提交记录

use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// 7fa4 提交记录
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

impl Default for Submission {
    fn default() -> Self {
        Self {
            code: String::new(),
            pid: String::new(),
            rid: String::new(),
            oj: String::new(),
            language: SubmissionLanguage::Cpp17,
            status: SubmissionStatus::Unknown,
            total_time: 0,
            max_memory: 0,
            score: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SubmissionStatus {
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
        let txt = s.trim().to_lowercase();
        match txt.as_str() {
            "unknown" => Ok(SubmissionStatus::Unknown),
            "accepted" | "通过" => Ok(SubmissionStatus::Accepted),
            "wronganswer" | "错误答案" | "wrong answer" => Ok(SubmissionStatus::WrongAnswer),
            "partiallycorrect" | "部分正确" | "partially correct" => {
                Ok(SubmissionStatus::PartiallyCorrect)
            }
            "runtimeerror" | "运行时错误" | "runtime error" => {
                Ok(SubmissionStatus::RuntimeError)
            }
            "compileerror" | "编译错误" | "compile error" => Ok(SubmissionStatus::CompileError),
            "timelimitexceeded" | "超时" | "time limit exceeded" => {
                Ok(SubmissionStatus::TimeLimitExceeded)
            }
            "memorylimitexceeded" | "内存" | "memory limit exceeded" => {
                Ok(SubmissionStatus::MemoryLimitExceeded)
            }
            other => Err(format!("unknown submission status: {other}")),
        }
    }
}

/// 提交语言
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SubmissionLanguage {
    #[serde(rename = "cpp14")]
    Cpp14,
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

        let mut n = txt.replace(' ', "");
        n = n.replace("c++", "cpp");
        n = n.replace('+', "");
        n = n.replace('\r', "");
        n = n.replace('\n', "");
        n = n.replace('-', "");

        match n.as_str() {
            "cpp14" => Ok(SubmissionLanguage::Cpp14),
            "cpp17" => Ok(SubmissionLanguage::Cpp17),
            "cpp11" => Ok(SubmissionLanguage::Cpp11),
            "cpp" => Ok(SubmissionLanguage::Cpp),
            "cppnoilinux" => Ok(SubmissionLanguage::CppNoiLinux),
            "cpp11noilinux" => Ok(SubmissionLanguage::Cpp11NoiLinux),
            "cpp11clang" => Ok(SubmissionLanguage::Cpp11Clang),
            "cpp17clang" => Ok(SubmissionLanguage::Cpp17Clang),
            "c" => Ok(SubmissionLanguage::C),
            "cnoilinux" => Ok(SubmissionLanguage::CNoiLinux),
            other => Err(format!("unknown submission language: {other}")),
        }
    }
}
