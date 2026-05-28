# Changelog

### [0.1.8](https://github.com/mangerle/vessel/compare/v0.1.7...v0.1.8) (2026-05-28)


### 🚀 新增功能 (Features)

* 实现容器挂起、活跃进程与Exec单次命令真实API，并修复右键菜单被截断的缺陷 ([c0db046](https://github.com/mangerle/vessel/commit/c0db0466e29e309e5b9e99cdbbeca1732d7e9172))
* 为现有镜像增加修改/追加 Tag 的操作交互入口，并同步支持防截断算法 ([7be3864](https://github.com/mangerle/vessel/commit/7be386489cddfdbdccc9ddf34d4c74be8766975e))


### 🐛 Bug 修复 (Bug Fixes)

* 解决日志流与性能统计流在关闭/切换页面后未主动注销导致的后台协程和网络套接字泄漏问题 ([7b794e7](https://github.com/mangerle/vessel/commit/7b794e79eafd41323b44dbe5c958bc42fdb16e4e))
* 修复交互终端关闭时后台协程与 Docker 长连接泄露的资源缺陷 ([9733cea](https://github.com/mangerle/vessel/commit/9733cea67a294b643a309468c92e8303abfa9338))
* 修复LS兼容模式下带空格的文件名被截断解析的Bug ([d8bdac6](https://github.com/mangerle/vessel/commit/d8bdac62e9c7691650235503088313e62a0bc0df))

### [0.1.7](https://github.com/mangerle/vessel/compare/v0.1.6...v0.1.7) (2026-05-28)


### 🚀 新增功能 (Features)

* 集成日志记录系统，持久化前后端运行日志 ([deca50f](https://github.com/mangerle/vessel/commit/deca50f1d447e6897f9d50bff3df7d60422aa2d4))
* 实现镜像流式导入导出与原生 Prune 清理，支持自定义导入标签并修复 tag_image 报错 ([8a6dcc1](https://github.com/mangerle/vessel/commit/8a6dcc116f86967973fe00254902f0751da06349))
* 实现容器提交(commit)与容器重命名功能 ([81ff959](https://github.com/mangerle/vessel/commit/81ff959c79990e36fd953e82e54552a1c42e1b33))
* 实现容器文件交互功能，包括文件浏览、双向拷贝与在线编辑 ([c5b58a8](https://github.com/mangerle/vessel/commit/c5b58a837b261b89f76ce19921a77b45967d775f))

### [0.1.6](https://github.com/mangerle/vessel/compare/v0.1.5...v0.1.6) (2026-05-28)


### 🐛 Bug 修复 (Bug Fixes)

* 修复 Docker Compose 停止/下线逻辑与数据刷新，精简菜单按钮文案 ([6a56531](https://github.com/mangerle/vessel/commit/6a5653115bfbaac1791369570c65230714e12616))
* **release:** 兼容 Windows CI 换行符，避免日志匹配失败退回到默认文本 ([cae8baf](https://github.com/mangerle/vessel/commit/cae8bafbbe6c9ffc2bc311561d4e667dbd781026))

### [0.1.5](https://github.com/mangerle/vessel/compare/v0.1.4...v0.1.5) (2026-05-28)


### 🐛 Bug 修复 (Bug Fixes)

* **release:** 升级发布日志流程，实现网页富文本与客户端纯文本差异化显示 ([008ebdb](https://github.com/mangerle/vessel/commit/008ebdb2f7494eed030aa0defad6254b257787d3))

### [0.1.4](https://github.com/mangerle/vessel/compare/v0.1.3...v0.1.4) (2026-05-27)


### 🐛 Bug 修复 (Bug Fixes)

* **updater:** 修复更新下载时因 Vue Proxy 导致私有成员读取失败的错误 ([80fec30](https://github.com/mangerle/vessel/commit/80fec30b67182fd2c8e0d223aaf2baa46a1d42dd))
