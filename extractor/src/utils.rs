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
    let lower = txt.to_lowercase();
    if lower.contains("mb") {
        let num = lower.replace("mb", "").trim().to_string();
        num.parse::<f64>().ok().map(|v| (v * 1024.0) as i32)
    } else if lower.contains('k') {
        let num = lower.replace('k', "").trim().to_string();
        num.parse::<f64>().ok().map(|v| v as i32)
    } else {
        txt.parse::<f64>().ok().map(|v| v as i32)
    }
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
