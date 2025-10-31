/**
 * 7FA4 提交器 - UI控制器
 * 只包含逻辑, 不包含 HTML 结构定义
 */
class RSubmitterUI {
    constructor(core) {
        this.core = core;
        this.initialized = false;
    }

    // 设置状态消息
    setStatus(text) {
        const el = document.getElementById('message');
        if (el) el.textContent = text;
    }

    // 设置登录状态
    setLoginStatus(text) {
        const el = document.getElementById('loginStatus');
        if (el) el.textContent = text;
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
            this.setStatus('找不到活动标签页');
            return;
        }

        chrome.scripting.executeScript({
            target: { tabId: tab.id },
            func: () => ({ cookie: document.cookie, origin: location.origin })
        }, async (results) => {
            if (!results?.[0]?.result) {
                this.setStatus('注入脚本失败');
                return;
            }

            const res = results[0].result;
            const response = await this.core.storeCookies(res.cookie || '', res.origin || '');

            if (response.ok) {
                if (response.warning) {
                    this.setStatus(response.message);
                } else {
                    this.setStatus(response.message || '7FA4 登录信息保存成功');
                }
                this.refreshLoginStatus();
            } else {
                this.setStatus(response.err ? ('保存失败: ' + response.err) : '保存失败');
            }
        });
    }

    // 发送页面处理
    async handleSendPage() {
        if (!this.core) return;

        this.setStatus('准备发送...');
        const tab = await this.core.getActiveTab();
        if (!tab) {
            this.setStatus('找不到活动标签页');
            return;
        }

        chrome.scripting.executeScript({
            target: { tabId: tab.id },
            func: () => ({ html: document.documentElement.outerHTML, url: location.href })
        }, async (results) => {
            if (!results?.[0]?.result) {
                this.setStatus('注入脚本失败');
                return;
            }

            const res = results[0].result;
            const response = await this.core.submitPage(res.url || '', res.html || '', false);

            if (response.ok) {
                if (response.businessSuccess) {
                    this.setStatus('提交成功！');
                } else {
                    const errorMsg = response.resp?.err || response.resp?.error || '服务器处理失败';
                    this.setStatus('提交失败: ' + errorMsg);
                }
            } else {
                if (response.parsed && response.parsed.partial) {
                    const parsed = response.parsed;
                    let msg = '部分解析成功但发送失败';
                    if (parsed.error) msg += ': ' + parsed.error;
                    this.setStatus(msg);
                } else {
                    const errorMsg = response.parsed?.error || response.err || '未知错误';
                    this.setStatus('发送失败: ' + errorMsg);
                }
            }
        });
    }

    // 初始化事件
    initEvents() {
        if (this.initialized) return;

        const getCookiesBtn = document.getElementById('getCookies');
        const sendPageBtn = document.getElementById('sendPage');

        if (getCookiesBtn) {
            getCookiesBtn.addEventListener('click', () => this.handleGetCookies());
        }

        if (sendPageBtn) {
            sendPageBtn.addEventListener('click', () => this.handleSendPage());
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
                console.warn('容器中没有找到 7FA4 提交器的HTML结构, 请确保已包含 popup.html 的内容');
                return;
            }
        }

        // 初始化事件
        this.initEvents();
    }
}
