/**
 * 7FA4提交器浏览器扩展 - 弹出窗口脚本
 * 负责用户界面交互，向页面注入脚本并处理消息通信
 */

// 支持的7FA4主机列表
const host_list = [
  'oj.7fa4.cn', 'jx.7fa4.cn:8888', 'in.7fa4.cn:8888', '10.210.57.10:8888'
];

// 全局变量
let runtimeInstance = null;  // WASM运行时实例
let wasmModule = null;       // 已加载的WASM模块

/**
 * 加载WASM运行时模块
 * @returns {Promise<Object>} WASM模块对象
 */
async function loadWasmRuntime() {
  if (wasmModule) return wasmModule;

  try {
    // 动态导入WASM模块
    const module = await import(chrome.runtime.getURL('wasm/runtime.js'));
    // 初始化WASM模块
    await module.default({ module_or_path: chrome.runtime.getURL('wasm/runtime_bg.wasm') });
    wasmModule = module;
    return module;
  } catch (e) {
    console.error('WASM 加载失败:', e);
    throw e;
  }
}

/**
 * 设置状态消息
 * @param {string} text - 要显示的状态文本
 */
function setStatus(text) {
  const el = document.getElementById('message');
  if (el) el.textContent = text;
}

/**
 * 设置登录状态
 * @param {string} text - 登录状态文本
 */
function setLoginStatus(text) {
  const el = document.getElementById('loginStatus');
  if (el) el.textContent = text;
}

/**
 * 刷新登录状态显示
 */
function freshLoginStatus() {
  chrome.storage.sync.get('cookies', ({ cookies }) => {
    if (!cookies || !cookies.login || !cookies['connect.sid']) {
      setLoginStatus('未登录');
    } else {
      setLoginStatus('已登录');
    }
  });
}

/**
 * 获取当前活动标签页
 * @returns {Promise<Object>} 活动标签页对象
 */
async function getActiveTab() {
  return new Promise(resolve => chrome.tabs.query({ active: true, currentWindow: true }, tabs => resolve(tabs && tabs[0] ? tabs[0] : null)));
}

/**
 * 存储cookies到扩展存储
 * @param {string} cookieStr - 原始的cookie字符串
 * @param {string} origin - 页面来源
 * @returns {Promise<Object>} 操作结果
 */
async function storeCookies(cookieStr, origin) {
  try {
    const module = await loadWasmRuntime();

    // 检查parse_cookie函数是否可用
    if (typeof module.parse_cookie !== 'function') {
      throw new Error('parse_cookie 函数未找到');
    }

    // 使用WASM解析cookies
    const ci = module.parse_cookie(cookieStr, origin);
    await chrome.storage.sync.set({ cookies: ci });

    try {
      // 初始化运行时实例
      runtimeInstance = new module.Runtime(ci);
    } catch (e) {
      console.warn('Runtime 实例化失败:', e);
    }

    return { ok: true };
  } catch (e) {
    // 回退：存储原始cookie数据
    await chrome.storage.sync.set({ raw_cookie: cookieStr, raw_origin: origin });
    return { ok: false, err: String(e) };
  }
}

/**
 * 提交页面内容到7FA4
 * @param {string} url - 页面URL
 * @param {string} html - 页面HTML内容
 * @param {boolean} in_contest - 是否在比赛中
 * @returns {Promise<Object>} 提交结果
 */
async function submitPage(url, html, in_contest = false) {
  // 从存储中获取cookies
  const { cookies } = await chrome.storage.sync.get(['cookies']);
  if (!cookies) {
    return { ok: false, err: '未找到 cookies' };
  }

  try {
    const module = await loadWasmRuntime();

    // 确保运行时实例存在
    if (!runtimeInstance) {
      runtimeInstance = new module.Runtime(cookies);
    }

    // 使用WASM提取提交信息
    const extractResult = runtimeInstance.extract(url, html, in_contest);

    // 检查提取结果
    if (!extractResult?.request) {
      return { ok: false, parsed: extractResult };
    }

    // 准备HTTP请求
    const req = extractResult.request;
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

    // 发送请求到7FA4
    const response = await fetch(req.url, fetchOpts);
    const json = await response.json().catch(() => null);
    return { ok: true, resp: json, parsed: extractResult };
  } catch (e) {
    return { ok: false, err: String(e) };
  }
}

// 事件监听器

/**
 * 获取Cookies按钮点击事件
 */
document.getElementById('getCookies').addEventListener('click', async () => {
  setStatus('获取中...');
  const tab = await getActiveTab();
  if (!tab) { setStatus('找不到活动标签页'); return; }

  // 向当前页面注入脚本获取cookies
  chrome.scripting.executeScript({
    target: { tabId: tab.id },
    func: () => ({ cookie: document.cookie, origin: location.origin })
  }, async (results) => {
    if (!results?.[0]?.result) { setStatus('注入脚本失败'); return; }

    const res = results[0].result;
    const response = await storeCookies(res.cookie || '', res.origin || '');

    if (response.ok) {
      setStatus('7FA4登录信息保存成功');
      freshLoginStatus();
    } else {
      setStatus(response.err ? ('保存失败: ' + response.err) : '保存失败');
    }
  });
});

/**
 * 发送页面按钮点击事件
 */
document.getElementById('sendPage').addEventListener('click', async () => {
  setStatus('准备发送...');
  const tab = await getActiveTab();
  if (!tab) { setStatus('找不到活动标签页'); return; }

  // 向当前页面注入脚本获取页面内容
  chrome.scripting.executeScript({
    target: { tabId: tab.id },
    func: () => ({ html: document.documentElement.outerHTML, url: location.href })
  }, async (results) => {
    if (!results?.[0]?.result) { setStatus('注入脚本失败'); return; }

    const res = results[0].result;
    const response = await submitPage(res.url || '', res.html || '', false);

    // 处理提交结果
    if (response.ok) {
      setStatus('发送成功: ' + (response.resp ? JSON.stringify(response.resp) : '无返回'));
    } else {
      // 检查是否有部分解析结果
      if (response.parsed && response.parsed.partial) {
        const parsed = response.parsed;
        let msg = '部分解析成功';
        if (parsed.error) msg += ': ' + parsed.error;
        setStatus(msg);
      } else {
        // 没有提取器或其他错误
        const errorMsg = response.parsed?.error || response.err || '未知错误';
        setStatus('发送失败: ' + errorMsg);
      }
    }
  });
});

/**
 * 页面加载完成后初始化
 */
document.addEventListener('DOMContentLoaded', () => {
  freshLoginStatus();
});
