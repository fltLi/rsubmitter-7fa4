/**
 * 7FA4提交器 - 主入口
 * 支持独立运行和被嵌入
 */

/**
 * Copyright (c) 2025 fltLi
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

// 创建全局应用对象
window.RSubmitter7FA4 = {
  core: null,
  ui: null,

  // 初始化应用
  async init(container) {
    try {
      // 初始化核心
      this.core = new RSubmitterCore();
      await this.core.init();

      // 初始化UI
      this.ui = new RSubmitterUI(this.core);

      // 初始化事件
      this.ui.init(container);

      console.log('7FA4 提交器初始化完成');
    } catch (error) {
      console.error('初始化失败: ', error);
      this.setStatus('初始化失败: ' + error.message);
    }
  },

  // 便捷方法: 设置状态
  setStatus(text) {
    if (this.ui) {
      this.ui.setStatus(text);
    }
  }
};

// 自动初始化 (独立运行模式)
if (typeof document !== 'undefined' && document.readyState !== 'loading') {
  window.RSubmitter7FA4.init();
} else {
  document.addEventListener('DOMContentLoaded', function () {
    window.RSubmitter7FA4.init();
  });
}
