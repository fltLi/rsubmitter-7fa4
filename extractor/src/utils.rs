//! 辅助工具

/*
 * Copyright (c) 2025 fltLi
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

/// 将时间字符串解析为毫秒
pub fn parse_time_to_ms(s: &str) -> Option<i32> {
    let txt = s.trim();
    if txt.is_empty() {
        return None;
    }
    let lower = txt.to_lowercase();
    if lower.contains("ms") {
        let num = lower.replace("ms", "").trim().to_string();
        num.parse::<f64>().ok().map(|v| v as i32)
    } else if lower.contains('s') {
        let num = lower.replace('s', "").trim().to_string();
        num.parse::<f64>().ok().map(|v| (v * 1000.0) as i32)
    } else {
        txt.parse::<f64>().ok().map(|v| v as i32)
    }
}

/// 将内存字符串解析为 KB
pub fn parse_mem_to_kb(s: &str) -> Option<i32> {
    let txt = s.trim();
    if txt.is_empty() {
        return None;
    }

    // 统一用小写做后缀判断, 但保留原始数字子串解析
    let lower = txt.to_lowercase();

    // 处理常见后缀: mb, m (兆 / MB), kb, k (千 / KB), b (字节)
    if lower.ends_with("mb") || lower.ends_with('m') {
        let num = lower
            .trim_end_matches("mb")
            .trim_end_matches('m')
            .trim()
            .to_string();
        return num.parse::<f64>().ok().map(|v| (v * 1024.0) as i32);
    }

    if lower.ends_with("kb") || lower.ends_with('k') {
        let num = lower
            .trim_end_matches("kb")
            .trim_end_matches('k')
            .trim()
            .to_string();
        return num.parse::<f64>().ok().map(|v| v as i32);
    }

    // 单位为字节 (e.g. "1024b" 或 "1024B"), 转换为 KB
    if lower.ends_with('b') {
        let num = lower.trim_end_matches('b').trim().to_string();
        return num.parse::<f64>().ok().map(|v| (v / 1024.0) as i32);
    }

    // 没有单位, 按 KB 处理 (兼容历史行为)
    txt.parse::<f64>().ok().map(|v| v as i32)
}

/// 如果 submission 来源于 VJudge, 尝试将其映射为真实的源 OJ (参考 extension/popup.js 中的逻辑)
/// 输入: submission 的部分结果
/// 输出: (mapped_oj, mapped_pid, mapped_rid) 三元组, 未映射时返回 None
pub fn map_vjudge_to_origin(sub: &crate::models::Submission) -> Option<(String, String, String)> {
    // 仅在 oj 字段看起来像 vjudge 或包含 vjudge 标识时尝试映射
    let oj_lower = sub.oj.to_lowercase();
    if !oj_lower.contains("vjudge") && !oj_lower.contains("virtual") {
        return None;
    }

    // pid 可能像 "UESTC-126" 或包含原始链接信息
    let pid = sub.pid.trim();
    // 常见情况: PID 形如 "OJNAME-123" 或 "ojname/problem/123" 等
    // 先尝试分解 PID 中的 "-" 分割 (如 UESTC-126)
    if let Some(idx) = pid.find('-') {
        let oj = pid[..idx].to_string();
        let pid_only = pid[idx + 1..].to_string();
        // rid 有时包含在 sub.rid, 或者 remote run id
        let rid = if !sub.rid.is_empty() {
            sub.rid.clone()
        } else {
            String::new()
        };
        return Some((oj, pid_only, rid));
    }

    // 备选: pid 本身可能就是原题目的 id (例如 UESTC-126 中的完整形式)
    if !pid.is_empty() {
        // 试图从 pid 中提取 OJ 前缀 (以非数字分隔)
        let parts: Vec<&str> = pid.split(&['/', '_', ':'][..]).collect();
        if parts.len() >= 2 {
            let oj = parts[0].to_string();
            let pid_only = parts[1].to_string();
            let rid = sub.rid.clone();
            return Some((oj, pid_only, rid));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time() {
        assert_eq!(parse_time_to_ms("100ms"), Some(100));
        assert_eq!(parse_time_to_ms("0.2s"), Some(200));
        assert_eq!(parse_time_to_ms("  50  "), Some(50));
    }

    #[test]
    fn test_parse_mem() {
        assert_eq!(parse_mem_to_kb("1MB"), Some(1024));
        assert_eq!(parse_mem_to_kb("512K"), Some(512));
        assert_eq!(parse_mem_to_kb("256"), Some(256));
    }
}
