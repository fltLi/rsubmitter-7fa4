/**
 * 7FA4提交器 - 核心功能
 */

/**
 * Copyright (c) 2025 fltLi
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

class RSubmitterCore {
    constructor() {
        this.wasmModule = null;
    }

    async loadWasm() {
        if (this.wasmModule) return this.wasmModule;

        try {
            const module = await import(chrome.runtime.getURL('wasm/runtime.js'));
            await module.default({
                module_or_path: chrome.runtime.getURL('wasm/runtime_bg.wasm')
            });
            this.wasmModule = module;
            return module;
        } catch (e) {
            console.error('WASM 加载失败:', e);
            throw e;
        }
    }

    async init() {
        await this.loadWasm();
    }

    async getActiveTab() {
        return new Promise(resolve => {
            chrome.tabs.query({ active: true, currentWindow: true }, tabs => {
                resolve(tabs && tabs[0] ? tabs[0] : null);
            });
        });
    }

    async storeCookies(cookieStr, origin) {
        try {
            const module = await this.loadWasm();

            if (typeof module.parse_cookie !== 'function') {
                throw new Error('parse_cookie 函数未找到');
            }

            const ci = module.parse_cookie(cookieStr, origin);

            const isValid7fa4Cookies = ci.login && ci['connect.sid'] &&
                (origin.includes('7fa4.cn') || ci.chost);

            await chrome.storage.sync.set({ cookies: ci });

            if (isValid7fa4Cookies) {
                return { ok: true, message: '7FA4 登录信息保存成功' };
            } else {
                return {
                    ok: true,
                    message: 'Cookie 已保存, 但未检测到有效的 7FA4 登录信息',
                    warning: true
                };
            }
        } catch (e) {
            await chrome.storage.sync.set({
                raw_cookie: cookieStr,
                raw_origin: origin
            });
            return { ok: false, err: String(e) };
        }
    }

    async submitPage(url, html, in_contest = false, options = { mapVjudge: false, observe: false }) {
        const { cookies } = await chrome.storage.sync.get(['cookies']);
        if (!cookies) {
            return { ok: false, err: '未找到 cookies' };
        }

        try {
            const module = await this.loadWasm();

            if (typeof module.extract_submission !== 'function') {
                throw new Error('extract_submission 函数未找到');
            }

            const extractResult = module.extract_submission(url, html);

            if (!extractResult?.success || !extractResult?.partial) {
                return {
                    ok: false,
                    err: extractResult?.error || '无法提取提交信息',
                    parsed: extractResult
                };
            }

            // 可选: 若开启映射 VJudge 为来源并且提取器是 vjudge, 调用 wasm 的映射函数
            if (options.mapVjudge && extractResult.extractor_name && typeof module.map_vjudge_submission === 'function') {
                const ename = (extractResult.extractor_name || '').toString().toLowerCase();
                if (ename.includes('vj') && ename.includes('vjudge')) {
                    try {
                        const mapped = module.map_vjudge_submission(extractResult.partial);
                        if (mapped) {
                            try {
                                const arr = mapped;
                                if (Array.isArray(arr) && arr.length >= 3) {
                                    extractResult.partial.oj = arr[0];
                                    extractResult.partial.pid = arr[1];
                                    extractResult.partial.rid = arr[2];
                                }
                            } catch (e) { }
                        }
                    } catch (e) { }
                }
            }

            // 如果仅观察模式, 不发送请求, 仅输出到控制台
            if (options.observe) {
                try {
                    console.log('观测模式输出 submission:', extractResult.partial);
                } catch (e) {
                    // ignore
                }
                return {
                    ok: true,
                    resp: null,
                    parsed: extractResult,
                    businessSuccess: false,
                    statusCode: 0
                };
            }

            // 构建请求
            const request = this.buildRequest(extractResult.partial, cookies, in_contest);

            if (!request) {
                return {
                    ok: false,
                    err: '无法生成请求数据',
                    parsed: extractResult
                };
            }

            const fetchOpts = {
                method: request.method || 'POST',
                headers: request.headers,
                credentials: 'include',
                body: typeof request.body === 'string' ? request.body : JSON.stringify(request.body),
            };

            const controller = new AbortController();
            const timeoutId = setTimeout(() => controller.abort(), 8000);

            fetchOpts.signal = controller.signal;

            let response;
            try {
                response = await fetch(request.url, fetchOpts);
                clearTimeout(timeoutId);
            } catch (fetchError) {
                clearTimeout(timeoutId);
                if (fetchError.name === 'AbortError') {
                    return {
                        ok: false,
                        err: '请求超时 (8 秒)',
                        parsed: extractResult
                    };
                }
                throw fetchError;
            }

            const json = await response.json().catch(() => null);

            const businessSuccess = json && json.success === true;

            return {
                ok: true,
                resp: json,
                parsed: extractResult,
                businessSuccess: businessSuccess,
                statusCode: response.status
            };
        } catch (e) {
            return {
                ok: false,
                err: String(e),
                parsed: extractResult
            };
        }
    }

    // 仅提取提交信息, 不依赖 cookies, 也不发送任何请求. 
    async extractSubmission(url, html) {
        try {
            const module = await this.loadWasm();

            if (typeof module.extract_submission !== 'function') {
                throw new Error('extract_submission 函数未找到');
            }

            const extractResult = module.extract_submission(url, html);

            if (!extractResult?.success || !extractResult?.partial) {
                return {
                    ok: false,
                    err: extractResult?.error || '无法提取提交信息',
                    parsed: extractResult
                };
            }

            try {
                console.log('观测到的 submission:', extractResult.partial);
            } catch (e) {
                // 忽略 console 输出错误
            }

            return {
                ok: true,
                submission: extractResult.partial,
                parsed: extractResult
            };
        } catch (e) {
            return { ok: false, err: String(e) };
        }
    }

    buildRequest(submission, cookies, in_contest) {
        try {
            let body = JSON.parse(JSON.stringify(submission)); // 深拷贝

            // 添加 in_contest 字段
            body.in_contest = in_contest;

            const chost = cookies.chost || "oj.7fa4.cn";
            const target = `http://${chost}/foreign_oj`;

            // 构建 cookie header
            let cookieHeader = "";
            if (cookies.login && cookies['connect.sid']) {
                cookieHeader = `login=${cookies.login}; connect.sid=${cookies['connect.sid']}`;
            } else if (cookies.login) {
                cookieHeader = `login=${cookies.login}`;
            }

            const headers = {
                "Content-Type": "application/json",
                "Cookie": cookieHeader
            };

            return {
                url: target,
                method: "POST",
                headers: headers,
                body: body
            };
        } catch (e) {
            console.error('构建请求失败:', e);
            return null;
        }
    }

    async getLoginStatus() {
        const { cookies } = await chrome.storage.sync.get('cookies');
        if (!cookies || !cookies.login || !cookies['connect.sid']) {
            return '未登录';
        } else {
            return '已登录';
        }
    }

    async isLoggedIn() {
        const status = await this.getLoginStatus();
        return status === '已登录';
    }
}
