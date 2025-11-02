# rsubmitter-7fa4

一个 Rust 实现的 7fa4 外站题提交器.

## 特性

- **便捷操作**:  继承 submitter [^1] 的简洁用法, 提供更美观的界面.

- **详细信息**:  提供更强大的提取器实现, 能够获取更详细的提交记录信息.

## 使用

在 Chrome 内核浏览器的扩展页面, 选择开发者模式 -> 加载解压缩的扩展.

使用时, 将扩展固定, 单击图标, 在 7fa4 页面点获取登录信息, 在 oj 提交记录页面点击发送提交记录.

## 待实现

- 支持更多的 oj. (详见[submitter 支持列表](http://jx.7fa4.cn:9080/tools/submitter/-/blob/main/README.md))  
   当前支持: [洛谷](https://www.luogu.com.cn/), [信友队](https://www.xinyoudui.com/).

- 支持 vjudge 间接提交其他 oj 提交记录. (submitter 功能)

- 提供控制到字段的手动提交功能.

- 添加提取器 wasm 更新获取功能, 支持热重载.

---

[^1]:  submitter 是一个长期维护的 7fa4 外站题提交插件, 使用方便快捷.
