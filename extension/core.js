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
        this.runtimeInstance = null;
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

            try {
                this.runtimeInstance = new module.Runtime(ci);
            } catch (e) {
                console.warn('Runtime 实例化失败:', e);
            }

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

    async submitPage(url, html, in_contest = false) {
        const { cookies } = await chrome.storage.sync.get(['cookies']);
        if (!cookies) {
            return { ok: false, err: '未找到 cookies' };
        }

        try {
            const module = await this.loadWasm();

            if (!this.runtimeInstance) {
                this.runtimeInstance = new module.Runtime(cookies);
            }

            const extractResult = this.runtimeInstance.extract(url, html, in_contest);

            if (!extractResult?.request && !extractResult?.partial) {
                return {
                    ok: false,
                    err: '无法提取提交信息',
                    parsed: extractResult
                };
            }

            const req = extractResult.request;
            if (!req) {
                return {
                    ok: false,
                    err: extractResult.error || '无法生成请求数据',
                    parsed: extractResult
                };
            }

            const headers = new Headers();
            if (req.headers) {
                Object.keys(req.headers).forEach(k => headers.set(k, req.headers[k]));
            }

            const fetchOpts = {
                method: req.method || 'POST',
                headers,
                credentials: 'include',
                body: typeof req.body === 'string' ? req.body : JSON.stringify(req.body),
            };

            const controller = new AbortController();
            const timeoutId = setTimeout(() => controller.abort(), 8000);

            fetchOpts.signal = controller.signal;

            let response;
            try {
                response = await fetch(req.url, fetchOpts);
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
