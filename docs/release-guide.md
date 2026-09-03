# Vessel 打包与自动发布操作全流程指南

本指南详细介绍如何通过 GitHub Actions 自动编译、自动签名加签，并将 Vessel 安装包直接发布到当前仓库 (`mangerle/vessel`) 的 Releases 中，同时支持客户端自动检查更新与静默升级。

---

## 整体发布架构

```mermaid
graph TD
    A[本地源码] -- git commit & push --> B(GitHub 仓库 mangerle/vessel)
    B -- 触发 Tag push v* --> C{GitHub Actions}
    C -- 编译/加签/打包 --> D[安装包与 latest.json]
    D -- 自动发布 Release --> E(当前仓库 Releases)
    E -- 自动更新接口 --> F[用户客户端升级]
```

---

## 全流程操作步骤

### 第一步：本地生成 Tauri 签名密钥对 (用于更新加签)
Tauri 的安全更新机制要求所有的安装包必须经过私钥数字签名，客户端再通过公钥进行验签，防止升级包被劫持篡改。
1. 在本地项目根目录 `d:\coding\rust\vessel` 下打开终端。
2. 运行免密密钥对生成命令：
   ```bash
   npx tauri signer generate --ci
   ```
3. 终端会直接打印出生成好的密钥对：
   - **`Public key` (公钥)**：较短的字符串，用于客户端验签。
   - **`Private key` (私钥)**：较长的字符串，用于打包时签名。
4. **配置公钥到项目**：
   复制输出的 `Public key`，打开项目中的 [tauri.conf.json](file:///d:/coding/rust/vessel/src-tauri/tauri.conf.json)，确认更新端点与公钥配置：
   ```json
   "plugins": {
     "updater": {
       "endpoints": [
         "https://github.com/mangerle/vessel/releases/latest/download/latest.json"
       ],
       "pubkey": "您的公钥字符串"
     }
   }
   ```

### 第二步：在 GitHub 仓库中配置 Secrets (密钥托管)
为了在 Actions 运行期间安全调用私钥对安装包进行签名，需要在当前 GitHub 仓库中注册 Actions Secrets：
1. 打开当前 GitHub 仓库：`https://github.com/mangerle/vessel`。
2. 依次进入 **Settings** -> **Secrets and variables** -> **Actions**。
3. 点击 **New repository secret** 按钮，添加以下机密项：
   - **`TAURI_SIGNING_PRIVATE_KEY`**：填入第一步中生成的 **私钥 (Private key)** 字符串。
   - *(注：若在生成密钥时您设置了私钥密码，则还需要新建 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` 填入密码，未设密码则无需添加)*。
   - *(注：因直接在当前仓库发布，Actions 使用自带的 `GITHUB_TOKEN` 即可，无需配置跨仓库个人访问令牌)*。

---

## 🛠️ 触发打包发布流程 (每次版本迭代)

当您开发完新功能并准备发布新版本时，请执行以下标准操作：

### 1. 保持版本号三合一一致
发布前，请务必确认以下三个文件中的版本号是一致的（例如准备发布 `0.1.2` 版本）：
- [package.json](file:///d:/coding/rust/vessel/package.json) 中的 `"version": "0.1.2"`
- [tauri.conf.json](file:///d:/coding/rust/vessel/src-tauri/tauri.conf.json) 中的 `"version": "0.1.2"`
- [Cargo.toml](file:///d:/coding/rust/vessel/src-tauri/Cargo.toml) 中的 `version = "0.1.2"`

### 2. 提交代码至 master 分支
确保本地的所有修改均已提交，并且推送到远程默认分支：
```bash
git add .
git commit -m "feat: 迭代新版本 0.1.2"
git push origin master
```

### 3. 创建并推送版本 Tag
在本地终端基于当前提交打上 `v` 开头的 Tag 并推送到远程，**这是触发自动构建的唯一开关**：
```bash
# 打上 v0.1.2 的版本标签
git tag v0.1.2

# 将此标签推送到远程仓库
git push origin v0.1.2
```

### 4. 观察云端构建与自动发布
- 推送 Tag 后，打开 GitHub 仓库的 **Actions** 选项卡，会看到一个名为 `自动加签发布 Release` 的工作流正在运行。
- 该工作流会拉取 Node.js 与 Rust 环境，编译生成安装包并使用您配置的 `TAURI_SIGNING_PRIVATE_KEY` 进行加签。
- 构建成功后，工作流将使用 `GITHUB_TOKEN` 自动将编译出的安装包、签名文件以及更新描述文件 `latest.json` 发布到当前仓库对应版本的 Releases 中。

---

## 🔄 客户端更新侦测逻辑

在发布成功后，客户端的关于面板会自动检测更新：
1. 前端通过 [Settings.vue](file:///d:/coding/rust/vessel/src/views/Settings.vue) 中的真实 `check()` API，去拉取当前仓库 `mangerle/vessel` 的 `latest.json` 文件。
2. 校验 `latest.json` 中的 `version` 是否大于当前版本（例如 `v0.1.2` 是否大于 `v0.1.1`）。
3. 校验通过后，使用公钥比对云端签名，安全地拉取安装包进行后台静默解压和覆盖安装。
4. 安装完成后，为用户呈现“立即重启应用”按钮，点击后调用 `relaunch()` 重启软件即完成热更新。
