# Changelog

## [0.2.0](https://github.com/mangerle/vessel/compare/v0.1.15...v0.2.0) (2026-08-21)

### [0.1.15](https://github.com/mangerle/vessel/compare/v0.1.14...v0.1.15) (2026-08-02)


### 🚀 新增功能 (Features)

* 适配 Linux Ubuntu 平台支持、托盘图标、IPC与 GitHub Actions CI 构建矩阵 ([9c8f99a](https://github.com/mangerle/vessel/commit/9c8f99af1fe5c7f66ba3390f20c9e6bb41cea185))

### [0.1.14](https://github.com/mangerle/vessel/compare/v0.1.13...v0.1.14) (2026-06-15)

### [0.1.13](https://github.com/mangerle/vessel/compare/v0.1.9...v0.1.13) (2026-06-15)


### ⚡ 性能优化 (Performance)

* **frontend:** 修复 前端代码性能问题（ECharts/日志/脏检查） ([d60fb68](https://github.com/mangerle/vessel/commit/d60fb68a2281619fe9deb2452abf6ee2bb14d9ea))
* **frontend:** 修复大量性能问题 ([7c186c0](https://github.com/mangerle/vessel/commit/7c186c0503c9905dd338ec5fe924699720b0ce93))
* **rust:** 修复大量性能问题 ([cf5408e](https://github.com/mangerle/vessel/commit/cf5408e868a5fe90f95f63c095c457250a6e3db2))
* **rust:** 修复rust性能问题（事件流节流/IO 异步化/连接幂等） ([55c7f3b](https://github.com/mangerle/vessel/commit/55c7f3bc64220d29b5522074731d50751c3ce9d7))
* **rust:** 优化连接管理锁竞争并增强类型安全 ([69bcb00](https://github.com/mangerle/vessel/commit/69bcb0020581da57cd36bc30b505dea1e0074766))
* **settings:** saveSettings 仅写变更 key，避免每次全量落盘 ([59eef4c](https://github.com/mangerle/vessel/commit/59eef4ce2ed326138edfe729aafa44428431a629))


### 🚀 新增功能 (Features)

* **api:** 命令名/事件名/类型常量统一前后端协议层 ([1a841bd](https://github.com/mangerle/vessel/commit/1a841bd16f777e2f66b78220f387648f3e617b8c))
* **api:** 优化容器伪终端创建接口用户身份处理 ([9108001](https://github.com/mangerle/vessel/commit/9108001f8b756854abe670e396699099cab2b434))
* **compose:** 优化ContainerFileBrowser文件加载逻辑 ([23c4440](https://github.com/mangerle/vessel/commit/23c444097e1ccf65f0d896f2bba3604c98a7ba36))
* **connection:** 前后端连接配置 emit 同步通道 ([9f71767](https://github.com/mangerle/vessel/commit/9f7176700a15f14d72c098a93d42f658df91d37f))
* **connection:** 统一支持 SSH 远程 Docker 与多连接引擎 ([a03f7fc](https://github.com/mangerle/vessel/commit/a03f7fc00f7ce919d90d55d8f26babe81d7a5354))
* **log:** 完善日志记录 ([cd39917](https://github.com/mangerle/vessel/commit/cd3991715a98a21e634684283612aac918d3adfa))
* **ssh:** 增加远端 Docker 环境一键诊断 ([b259351](https://github.com/mangerle/vessel/commit/b2593514de8b201115a2f35570cf0bec01092a70))
* **ssh:** 支持 sudo 提升权限以兼容非 root 用户 ([43ba373](https://github.com/mangerle/vessel/commit/43ba373a83ae861c373ce1751e18282fbb6746bb))


### 🐛 Bug 修复 (Bug Fixes)

* 事件名常量化/zip-slip/askpass 残留/日志 key/死代码 ([59a716b](https://github.com/mangerle/vessel/commit/59a716bb97bd9ea6f8c23dc0413536598f3c74e8))
* 修复 CSS 语法错误、内存泄漏并增强 CSP 安全策略 ([600dd85](https://github.com/mangerle/vessel/commit/600dd85599c15829d837c8135ff6d5f6974559f7))
* **compose:** 优化容器日志流的节流处理 ([8182500](https://github.com/mangerle/vessel/commit/818250032e31da0269716c9a7ed68e631bd281b5))
* **connection:** 启动/切换串行化，根除连接 race 与列表残留 ([1f2ce40](https://github.com/mangerle/vessel/commit/1f2ce400eb1f0dc10742c7720ebe07ecf6bbf918))
* **connection:** 原子化 ensure_proxy 避免并发端口泄漏 ([9c922fd](https://github.com/mangerle/vessel/commit/9c922fdcb4b0e2c881b88eabd4207796ffb51674))
* **docker:** 优化 docker compose 命令参数传递方式 ([aaa6a05](https://github.com/mangerle/vessel/commit/aaa6a05c8d50ab50a306d9c93854d5b6f60e0186))
* **docker:** 优化错误处理与代码简洁性 ([3d6b772](https://github.com/mangerle/vessel/commit/3d6b772308db7713272554d5aeea7aa37c2b305c))
* ECharts 预拆/写锁不跨 await/emit 按 name/compose cmd_id ([fba7ee0](https://github.com/mangerle/vessel/commit/fba7ee0fac9d187ff51b1c1e33838af33b734d27))
* known_hosts/askpass 改环境变量/keyring 集成 ([854eba1](https://github.com/mangerle/vessel/commit/854eba10989ae82cd2f48cc2680cfb982eb47c62))
* **rust:** 修复安全/规范问题 ([c02166e](https://github.com/mangerle/vessel/commit/c02166e719d6d8d60c90e1cf496b6b6f2913fa8a))
* **rust:** 修复死锁风险、命令注入漏洞及终端锁阻塞问题 ([e007da1](https://github.com/mangerle/vessel/commit/e007da107c88a0780c4b53b87419044b43084495))
* **rust:** 移除后端代码中潜在的 unwrap() panic 风险 ([6a1a5e3](https://github.com/mangerle/vessel/commit/6a1a5e3ef5f9d09dc0fd9bfd322a8e77dc936ea2))
* **settings:** 修复 lastSavedSnapshot 引用复用导致增量同步失效 ([0f501e3](https://github.com/mangerle/vessel/commit/0f501e3d8c55c1bd1ea0100f89b3feeff07ccf5b))
* **tauri:** 修复 build 模式 inline style 失效导致图标错乱的问题 ([3f1ae44](https://github.com/mangerle/vessel/commit/3f1ae447fed40e5fa99f9d67abbcd1e2fe6a60d7))
* Volumes.vue 下推到 api 层 + 容器 fs 路径白名单 ([25444d3](https://github.com/mangerle/vessel/commit/25444d33da5ce83c57cfa6fd25d13c403a889d5b))
* **wsl:** 移除 client_socket 多余的 mut 标记 ([88beca8](https://github.com/mangerle/vessel/commit/88beca8db6617bbc78776deda9efd884c9f8a61b))

### [0.1.12](https://github.com/mangerle/vessel/compare/v0.1.9...v0.1.12) (2026-06-13)


### ⚡ 性能优化 (Performance)

* **frontend:** 修复 前端代码性能问题（ECharts/日志/脏检查） ([d60fb68](https://github.com/mangerle/vessel/commit/d60fb68a2281619fe9deb2452abf6ee2bb14d9ea))
* **frontend:** 修复大量性能问题 ([7c186c0](https://github.com/mangerle/vessel/commit/7c186c0503c9905dd338ec5fe924699720b0ce93))
* **rust:** 修复大量性能问题 ([cf5408e](https://github.com/mangerle/vessel/commit/cf5408e868a5fe90f95f63c095c457250a6e3db2))
* **rust:** 修复rust性能问题（事件流节流/IO 异步化/连接幂等） ([55c7f3b](https://github.com/mangerle/vessel/commit/55c7f3bc64220d29b5522074731d50751c3ce9d7))
* **rust:** 优化连接管理锁竞争并增强类型安全 ([69bcb00](https://github.com/mangerle/vessel/commit/69bcb0020581da57cd36bc30b505dea1e0074766))
* **settings:** saveSettings 仅写变更 key，避免每次全量落盘 ([59eef4c](https://github.com/mangerle/vessel/commit/59eef4ce2ed326138edfe729aafa44428431a629))


### 🚀 新增功能 (Features)

* **api:** 命令名/事件名/类型常量统一前后端协议层 ([1a841bd](https://github.com/mangerle/vessel/commit/1a841bd16f777e2f66b78220f387648f3e617b8c))
* **api:** 优化容器伪终端创建接口用户身份处理 ([9108001](https://github.com/mangerle/vessel/commit/9108001f8b756854abe670e396699099cab2b434))
* **compose:** 优化ContainerFileBrowser文件加载逻辑 ([23c4440](https://github.com/mangerle/vessel/commit/23c444097e1ccf65f0d896f2bba3604c98a7ba36))
* **connection:** 前后端连接配置 emit 同步通道 ([9f71767](https://github.com/mangerle/vessel/commit/9f7176700a15f14d72c098a93d42f658df91d37f))
* **connection:** 统一支持 SSH 远程 Docker 与多连接引擎 ([a03f7fc](https://github.com/mangerle/vessel/commit/a03f7fc00f7ce919d90d55d8f26babe81d7a5354))
* **log:** 完善日志记录 ([cd39917](https://github.com/mangerle/vessel/commit/cd3991715a98a21e634684283612aac918d3adfa))
* **ssh:** 增加远端 Docker 环境一键诊断 ([b259351](https://github.com/mangerle/vessel/commit/b2593514de8b201115a2f35570cf0bec01092a70))
* **ssh:** 支持 sudo 提升权限以兼容非 root 用户 ([43ba373](https://github.com/mangerle/vessel/commit/43ba373a83ae861c373ce1751e18282fbb6746bb))


### 🐛 Bug 修复 (Bug Fixes)

* 事件名常量化/zip-slip/askpass 残留/日志 key/死代码 ([59a716b](https://github.com/mangerle/vessel/commit/59a716bb97bd9ea6f8c23dc0413536598f3c74e8))
* 修复 CSS 语法错误、内存泄漏并增强 CSP 安全策略 ([600dd85](https://github.com/mangerle/vessel/commit/600dd85599c15829d837c8135ff6d5f6974559f7))
* **compose:** 优化容器日志流的节流处理 ([8182500](https://github.com/mangerle/vessel/commit/818250032e31da0269716c9a7ed68e631bd281b5))
* **connection:** 启动/切换串行化，根除连接 race 与列表残留 ([1f2ce40](https://github.com/mangerle/vessel/commit/1f2ce400eb1f0dc10742c7720ebe07ecf6bbf918))
* **connection:** 原子化 ensure_proxy 避免并发端口泄漏 ([9c922fd](https://github.com/mangerle/vessel/commit/9c922fdcb4b0e2c881b88eabd4207796ffb51674))
* **docker:** 优化 docker compose 命令参数传递方式 ([aaa6a05](https://github.com/mangerle/vessel/commit/aaa6a05c8d50ab50a306d9c93854d5b6f60e0186))
* **docker:** 优化错误处理与代码简洁性 ([3d6b772](https://github.com/mangerle/vessel/commit/3d6b772308db7713272554d5aeea7aa37c2b305c))
* ECharts 预拆/写锁不跨 await/emit 按 name/compose cmd_id ([fba7ee0](https://github.com/mangerle/vessel/commit/fba7ee0fac9d187ff51b1c1e33838af33b734d27))
* known_hosts/askpass 改环境变量/keyring 集成 ([854eba1](https://github.com/mangerle/vessel/commit/854eba10989ae82cd2f48cc2680cfb982eb47c62))
* **rust:** 修复安全/规范问题 ([c02166e](https://github.com/mangerle/vessel/commit/c02166e719d6d8d60c90e1cf496b6b6f2913fa8a))
* **rust:** 修复死锁风险、命令注入漏洞及终端锁阻塞问题 ([e007da1](https://github.com/mangerle/vessel/commit/e007da107c88a0780c4b53b87419044b43084495))
* **rust:** 移除后端代码中潜在的 unwrap() panic 风险 ([6a1a5e3](https://github.com/mangerle/vessel/commit/6a1a5e3ef5f9d09dc0fd9bfd322a8e77dc936ea2))
* **settings:** 修复 lastSavedSnapshot 引用复用导致增量同步失效 ([0f501e3](https://github.com/mangerle/vessel/commit/0f501e3d8c55c1bd1ea0100f89b3feeff07ccf5b))
* Volumes.vue 下推到 api 层 + 容器 fs 路径白名单 ([25444d3](https://github.com/mangerle/vessel/commit/25444d33da5ce83c57cfa6fd25d13c403a889d5b))
* **wsl:** 移除 client_socket 多余的 mut 标记 ([88beca8](https://github.com/mangerle/vessel/commit/88beca8db6617bbc78776deda9efd884c9f8a61b))

### [0.1.11](https://github.com/mangerle/vessel/compare/v0.1.9...v0.1.11) (2026-06-13)


### ⚡ 性能优化 (Performance)

* **frontend:** 修复 前端代码性能问题（ECharts/日志/脏检查） ([d60fb68](https://github.com/mangerle/vessel/commit/d60fb68a2281619fe9deb2452abf6ee2bb14d9ea))
* **frontend:** 修复大量性能问题 ([7c186c0](https://github.com/mangerle/vessel/commit/7c186c0503c9905dd338ec5fe924699720b0ce93))
* **rust:** 修复大量性能问题 ([cf5408e](https://github.com/mangerle/vessel/commit/cf5408e868a5fe90f95f63c095c457250a6e3db2))
* **rust:** 修复rust性能问题（事件流节流/IO 异步化/连接幂等） ([55c7f3b](https://github.com/mangerle/vessel/commit/55c7f3bc64220d29b5522074731d50751c3ce9d7))
* **rust:** 优化连接管理锁竞争并增强类型安全 ([69bcb00](https://github.com/mangerle/vessel/commit/69bcb0020581da57cd36bc30b505dea1e0074766))
* **settings:** saveSettings 仅写变更 key，避免每次全量落盘 ([59eef4c](https://github.com/mangerle/vessel/commit/59eef4ce2ed326138edfe729aafa44428431a629))


### 🚀 新增功能 (Features)

* **api:** 命令名/事件名/类型常量统一前后端协议层 ([1a841bd](https://github.com/mangerle/vessel/commit/1a841bd16f777e2f66b78220f387648f3e617b8c))
* **api:** 优化容器伪终端创建接口用户身份处理 ([9108001](https://github.com/mangerle/vessel/commit/9108001f8b756854abe670e396699099cab2b434))
* **compose:** 优化ContainerFileBrowser文件加载逻辑 ([23c4440](https://github.com/mangerle/vessel/commit/23c444097e1ccf65f0d896f2bba3604c98a7ba36))
* **connection:** 前后端连接配置 emit 同步通道 ([9f71767](https://github.com/mangerle/vessel/commit/9f7176700a15f14d72c098a93d42f658df91d37f))
* **connection:** 统一支持 SSH 远程 Docker 与多连接引擎 ([a03f7fc](https://github.com/mangerle/vessel/commit/a03f7fc00f7ce919d90d55d8f26babe81d7a5354))
* **log:** 完善日志记录 ([cd39917](https://github.com/mangerle/vessel/commit/cd3991715a98a21e634684283612aac918d3adfa))
* **ssh:** 增加远端 Docker 环境一键诊断 ([b259351](https://github.com/mangerle/vessel/commit/b2593514de8b201115a2f35570cf0bec01092a70))
* **ssh:** 支持 sudo 提升权限以兼容非 root 用户 ([43ba373](https://github.com/mangerle/vessel/commit/43ba373a83ae861c373ce1751e18282fbb6746bb))


### 🐛 Bug 修复 (Bug Fixes)

* 修复 CSS 语法错误、内存泄漏并增强 CSP 安全策略 ([600dd85](https://github.com/mangerle/vessel/commit/600dd85599c15829d837c8135ff6d5f6974559f7))
* **compose:** 优化容器日志流的节流处理 ([8182500](https://github.com/mangerle/vessel/commit/818250032e31da0269716c9a7ed68e631bd281b5))
* **connection:** 原子化 ensure_proxy 避免并发端口泄漏 ([9c922fd](https://github.com/mangerle/vessel/commit/9c922fdcb4b0e2c881b88eabd4207796ffb51674))
* **docker:** 优化 docker compose 命令参数传递方式 ([aaa6a05](https://github.com/mangerle/vessel/commit/aaa6a05c8d50ab50a306d9c93854d5b6f60e0186))
* **docker:** 优化错误处理与代码简洁性 ([3d6b772](https://github.com/mangerle/vessel/commit/3d6b772308db7713272554d5aeea7aa37c2b305c))
* **rust:** 修复安全/规范问题 ([c02166e](https://github.com/mangerle/vessel/commit/c02166e719d6d8d60c90e1cf496b6b6f2913fa8a))
* **rust:** 修复死锁风险、命令注入漏洞及终端锁阻塞问题 ([e007da1](https://github.com/mangerle/vessel/commit/e007da107c88a0780c4b53b87419044b43084495))
* **rust:** 移除后端代码中潜在的 unwrap() panic 风险 ([6a1a5e3](https://github.com/mangerle/vessel/commit/6a1a5e3ef5f9d09dc0fd9bfd322a8e77dc936ea2))
* **settings:** 修复 lastSavedSnapshot 引用复用导致增量同步失效 ([0f501e3](https://github.com/mangerle/vessel/commit/0f501e3d8c55c1bd1ea0100f89b3feeff07ccf5b))
* **wsl:** 移除 client_socket 多余的 mut 标记 ([88beca8](https://github.com/mangerle/vessel/commit/88beca8db6617bbc78776deda9efd884c9f8a61b))

### [0.1.10](https://github.com/mangerle/vessel/compare/v0.1.9...v0.1.10) (2026-06-12)


### 🚀 新增功能 (Features)

* **compose:** 优化ContainerFileBrowser文件加载逻辑 ([23c4440](https://github.com/mangerle/vessel/commit/23c444097e1ccf65f0d896f2bba3604c98a7ba36))
* **connection:** 统一支持 SSH 远程 Docker 与多连接引擎 ([a03f7fc](https://github.com/mangerle/vessel/commit/a03f7fc00f7ce919d90d55d8f26babe81d7a5354))
* **log:** 完善日志记录 ([cd39917](https://github.com/mangerle/vessel/commit/cd3991715a98a21e634684283612aac918d3adfa))
* **ssh:** 增加远端 Docker 环境一键诊断 ([b259351](https://github.com/mangerle/vessel/commit/b2593514de8b201115a2f35570cf0bec01092a70))
* **ssh:** 支持 sudo 提升权限以兼容非 root 用户 ([43ba373](https://github.com/mangerle/vessel/commit/43ba373a83ae861c373ce1751e18282fbb6746bb))


### ⚡ 性能优化 (Performance)

* **rust:** 优化连接管理锁竞争并增强类型安全 ([69bcb00](https://github.com/mangerle/vessel/commit/69bcb0020581da57cd36bc30b505dea1e0074766))
* **settings:** saveSettings 仅写变更 key，避免每次全量落盘 ([59eef4c](https://github.com/mangerle/vessel/commit/59eef4ce2ed326138edfe729aafa44428431a629))


### 🐛 Bug 修复 (Bug Fixes)

* 修复 CSS 语法错误、内存泄漏并增强 CSP 安全策略 ([600dd85](https://github.com/mangerle/vessel/commit/600dd85599c15829d837c8135ff6d5f6974559f7))
* **compose:** 优化容器日志流的节流处理 ([8182500](https://github.com/mangerle/vessel/commit/818250032e31da0269716c9a7ed68e631bd281b5))
* **docker:** 优化 docker compose 命令参数传递方式 ([aaa6a05](https://github.com/mangerle/vessel/commit/aaa6a05c8d50ab50a306d9c93854d5b6f60e0186))
* **rust:** 修复死锁风险、命令注入漏洞及终端锁阻塞问题 ([e007da1](https://github.com/mangerle/vessel/commit/e007da107c88a0780c4b53b87419044b43084495))
* **rust:** 移除后端代码中潜在的 unwrap() panic 风险 ([6a1a5e3](https://github.com/mangerle/vessel/commit/6a1a5e3ef5f9d09dc0fd9bfd322a8e77dc936ea2))
* **wsl:** 移除 client_socket 多余的 mut 标记 ([88beca8](https://github.com/mangerle/vessel/commit/88beca8db6617bbc78776deda9efd884c9f8a61b))

### [0.1.9](https://github.com/mangerle/vessel/compare/v0.1.8...v0.1.9) (2026-05-30)


### 🚀 新增功能 (Features)

* 添加 image、container api service ([564063e](https://github.com/mangerle/vessel/commit/564063ebb2a6bfb7566473049de7ce94493059c0))
* 增加软件启动自动检查更新功能，并支持跳转设置页更新 ([072e214](https://github.com/mangerle/vessel/commit/072e214bca3391ab0548d0f464feb39d65f2b6e6))
* **backend:** 添加open_log_dir Tauri指令 ([fab63d3](https://github.com/mangerle/vessel/commit/fab63d3e21e79494739ee2f935600b50667cf267))
* **frontend:** 添加 useApi 钩子以实现标准化 API 处理 ([ee25e53](https://github.com/mangerle/vessel/commit/ee25e53115cd8c25b3d43f56a67f78a9af26a281))
* **logging:** 为设置添加前端操作日志 ([890abe7](https://github.com/mangerle/vessel/commit/890abe75d0d9cf588dcd4d8da357a2e3ac59f1c7))
* **ui:** 补全终端、图表和元数据详情的定制化右键菜单 ([275263d](https://github.com/mangerle/vessel/commit/275263d6d269416afc09fbd90228ab4ceb0fe5d3))
* **ui:** 为文件浏览器添加差异化右键菜单 ([d04d9a2](https://github.com/mangerle/vessel/commit/d04d9a2e6353fc453dfcb798af37515f3db57e15))
* **ui:** 为文件浏览器添加差异化右键菜单并完善交互逻辑 ([e5fa882](https://github.com/mangerle/vessel/commit/e5fa882def96d91982bbcb1049db967c1b4f9674))
* **ui:** 为运行日志视图添加专属右键菜单 ([50a0330](https://github.com/mangerle/vessel/commit/50a033023ae7521cc11a59f7fb2f2c5ab54e55b8))
* **ui:** 在设置中添加“打开日志目录”按钮 ([f3ea132](https://github.com/mangerle/vessel/commit/f3ea1326f459277904215bfed53ec782195af76a))
* **ui:** 在资源视图和关于部分中用应用程序标志替换占位符图标 ([234ba98](https://github.com/mangerle/vessel/commit/234ba9844fe28e74ae9e86a624417a6c05f19289))


### 🐛 Bug 修复 (Bug Fixes)

* **build:** 移除未使用的图标导入以修复编译错误 ([6b13d62](https://github.com/mangerle/vessel/commit/6b13d62467298705f5210dabe92b91d8b7388cb7))
* **ui:** 完善日志复制的错误处理与用户提示 ([f6d06a3](https://github.com/mangerle/vessel/commit/f6d06a30f702c926e592d0cce9b1f2ab59337452))
* **ui:** 修复侧边列表空白处右键菜单因事件冒泡未阻止导致原生菜单干扰和坐标计算异常的问题 ([4825943](https://github.com/mangerle/vessel/commit/48259437c31468804bd7e2e7865395f2ca365b1f))
* **ui:** 修复日志菜单因为点击失去焦点导致无法获取选中文本的问题 ([7f5034f](https://github.com/mangerle/vessel/commit/7f5034f6798d54bb90ee83fb6da78a45ab7f456b))
* **ui:** 修复终端光标颜色在不同主题下不可见的问题 ([bb53873](https://github.com/mangerle/vessel/commit/bb53873a0fab7edeec4a720d99fb657cf1381111))
* **ui:** 修复终端和图表视图下右键菜单无效并被全局背景菜单覆盖的问题 ([c7153da](https://github.com/mangerle/vessel/commit/c7153da397f789c11f502f5c2f7bd45a0829a2de))
* **ui:** 修复终端内无法使用右键选中并复制文本的问题 ([05a9c53](https://github.com/mangerle/vessel/commit/05a9c530388630175be997ddc7f7e8124115f371))
* **ui:** 修复终端右键复制因为焦点丢失导致无法获取选中文本的问题 ([dce53ae](https://github.com/mangerle/vessel/commit/dce53ae5bbdb51d5421a2435690a2d9cfe9936ce))
* **ui:** 修复终端右键执行复制或粘贴后失去输入焦点的问题 ([63f06bd](https://github.com/mangerle/vessel/commit/63f06bda0b993dcfaef8207bc7b437e14707ce79))
* **ui:** 运行日志和终端新增复制粘贴，文件浏览器去除多余的操作按钮 ([5573465](https://github.com/mangerle/vessel/commit/55734650cd5f804388083a7d7b292df2ec391a02))

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
