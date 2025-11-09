/**
 * 7FA4 提交器 - UI控制器
 * 只包含逻辑, 不包含 HTML 结构定义
 */

/**
 * Copyright (c) 2025 fltLi
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

class RSubmitterUI {
    constructor(core) {
        this.core = core;
        this.initialized = false;
        this.observeMode = false;
    }

    // 设置状态消息
    setStatus(text, isError = false) {
        const el = document.getElementById('message');
        if (el) {
            el.textContent = text;
            if (isError) {
                el.className = 'message error';
            } else {
                el.className = 'message';
            }
        }
    }

    // 设置登录状态
    setLoginStatus(text) {
        const el = document.getElementById('loginStatus');
        if (el) {
            el.textContent = text;
            if (text === '已登录') {
                el.className = 'status logged-in';
            } else {
                el.className = 'status logged-out';
            }
        }
    }

    // 刷新登录状态
    async refreshLoginStatus() {
        if (!this.core) return;
        const status = await this.core.getLoginStatus();
        this.setLoginStatus(status);
    }

    // 获取 cookies 处理
    async handleGetCookies() {
        if (!this.core) return;

        this.setStatus('获取中...');
        const tab = await this.core.getActiveTab();
        if (!tab) {
            this.setStatus('找不到活动标签页', true);
            return;
        }

        try {
            chrome.scripting.executeScript({
                target: { tabId: tab.id },
                func: () => ({ cookie: document.cookie, origin: location.origin })
            }, async (results) => {
                if (!results?.[0]?.result) {
                    this.setStatus('注入脚本失败', true);
                    return;
                }

                const res = results[0].result;
                const response = await this.core.storeCookies(res.cookie || '', res.origin || '');

                if (response.ok) {
                    if (response.warning) {
                        this.setStatus(response.message, true);
                    } else {
                        this.setStatus(response.message || '7FA4 登录信息保存成功');
                    }
                    this.refreshLoginStatus();
                } else {
                    this.setStatus(response.err ? ('保存失败: ' + response.err) : '保存失败', true);
                }
            });
        } catch (error) {
            this.setStatus('执行脚本失败: ' + error.message, true);
        }
    }

    // 发送页面处理
    async handleSendPageWithOptions(options = { mapVjudge: false, observe: false }) {
        if (!this.core) return;

        if (options.observe) {
            this.setStatus('观测模式: 准备提取...');
        } else {
            this.setStatus('准备发送...');
        }

        const tab = await this.core.getActiveTab();
        if (!tab) {
            this.setStatus('找不到活动标签页', true);
            return;
        }

        try {
            chrome.scripting.executeScript({
                target: { tabId: tab.id },
                func: () => ({ html: document.documentElement.outerHTML, url: location.href })
            }, async (results) => {
                if (!results?.[0]?.result) {
                    this.setStatus('注入脚本失败', true);
                    return;
                }

                const res = results[0].result;

                if (options.observe) {
                    this.setStatus('正在提取 (观测)...');
                    const response = await this.core.extractSubmission(res.url || '', res.html || '');
                    if (response.ok) {
                        this.setStatus('提取完成 (观测) , 请查看扩展控制台');
                        try { console.log('观测模式提交 (UI):', response.submission); } catch (e) { }
                    } else {
                        this.setStatus('提取失败: ' + (response.err || '未知错误'), true);
                    }
                    return;
                }

                this.setStatus('发送中...');

                // 读取选项并调用 core.submitPage
                const response = await this.core.submitPage(res.url || '', res.html || '', false, options);

                if (response.ok) {
                    if (response.businessSuccess) {
                        this.setStatus('提交成功！');
                    } else {
                        const errorMsg = response.resp?.err || response.resp?.error ||
                            response.resp?.message || '服务器处理失败';
                        this.setStatus('提交失败: ' + errorMsg, true);
                    }
                } else {
                    let errorMessage = '发送失败: ';

                    if (response.err) {
                        errorMessage += response.err;
                    } else if (response.parsed?.error) {
                        errorMessage += response.parsed.error;
                    } else {
                        errorMessage += '未知错误';
                    }

                    if (response.parsed?.partial) {
                        errorMessage += ' (部分数据已解析)';
                    }

                    this.setStatus(errorMessage, true);
                }
            });
        } catch (error) {
            this.setStatus('执行脚本失败: ' + error.message, true);
        }
    }

    // 初始化事件
    initEvents() {
        if (this.initialized) return;

    const getCookiesBtn = document.getElementById('getCookies');
    const sendPageBtn = document.getElementById('sendPage');
    const sendOptionsToggle = document.getElementById('sendOptionsToggle');
    const sendOptions = document.getElementById('sendOptions');
    const optMapVjudge = document.getElementById('optMapVjudge');
    const optObserve = document.getElementById('optObserve');

        if (getCookiesBtn) {
            getCookiesBtn.addEventListener('click', () => this.handleGetCookies());
        }

        if (sendPageBtn) {
            sendPageBtn.addEventListener('click', () => {
                const options = {
                    mapVjudge: optMapVjudge ? optMapVjudge.checked : false,
                    observe: optObserve ? optObserve.checked : false
                };
                this.handleSendPageWithOptions(options);
            });
        }

        if (sendOptionsToggle && sendOptions) {
            sendOptionsToggle.addEventListener('click', (e) => {
                e.stopPropagation();
                // 如果已经显示则隐藏
                if (sendOptions.style.display === 'flex') {
                    sendOptions.style.display = 'none';
                    return;
                }

                // 作为流内元素显示在按钮下方，从而推动 popup 窗口高度扩展，避免裁切
                sendOptions.style.position = '';
                sendOptions.style.display = 'flex';
            });

            // 点击页面任意位置关闭 options
            document.addEventListener('click', () => { if (sendOptions) sendOptions.style.display = 'none'; });

            // 阻止 options 内部点击导致关闭
            sendOptions.addEventListener('click', (e) => { e.stopPropagation(); });
        }

        // 初始化选项状态: 从 storage 读取并应用到复选框
        chrome.storage.sync.get(['optMapVjudge', 'optObserve'], ({ optMapVjudge: storedMap, optObserve: storedObserve }) => {
            try {
                if (optMapVjudge) optMapVjudge.checked = !!storedMap;
                if (optObserve) optObserve.checked = !!storedObserve;
            } catch (e) {
                // ignore
            }
        });
        

        // 保存选项变化
        if (optObserve) {
            optObserve.addEventListener('change', () => {
                chrome.storage.sync.set({ optObserve: optObserve.checked });
            });
        }
        if (optMapVjudge) {
            optMapVjudge.addEventListener('change', () => {
                chrome.storage.sync.set({ optMapVjudge: optMapVjudge.checked });
            });
        }

        // 初始化登录状态
        this.refreshLoginStatus();

        this.initialized = true;
    }

    // 简单初始化方法 (用于嵌入场景)
    init(container) {
        if (container) {
            // 如果提供了容器, 检查是否已经有我们的 HTML 结构
            const existingUI = container.querySelector('.fluent-card');
            if (!existingUI) {
                console.warn('容器中没有找到 7FA4 提交器的 HTML 结构, 请确保已包含 popup.html 的内容');
                return;
            }
        }

        // 初始化事件
        this.initEvents();
    }
}
