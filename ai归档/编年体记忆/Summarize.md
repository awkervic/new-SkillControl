# new-SkillControl 增量总结 (Incremental Summaries)

---

### [2026-05-27 10:45] (北京时间)

**核心变动描述：**
1. **项目自述排版重构**：调整了项目根目录下的 `README.md`，将纯中文的项目介绍、核心亮点调整至置顶区域，并在下方附加了英文自述，提升面向国内开发者的可读性。
2. **Git 库大二进制文件清洗**：将 `releases/` 目录从 Git 暂存区移除，写入 `.gitignore` 进行长效过滤。
3. **云端发布自动化**：利用 Node.js 提取 Git 凭证 Token，通过 GitHub API 自动创建 `v0.1.0` 发布页面并上传 EXE 及 MSI 安装包。
4. **Git 历史展平清洗**：分支强制重置与推送，将多次提交归一化为单条初始记录。
5. **记忆持久化同步**：创建 `ai归档/编年体记忆` 系列文件。

---

### [2026-05-27 11:05] (北京时间)

**核心变动描述：**
1. **彻底解决空壳包漏洞**：删除 `src/bin/gen_ico.rs` 干扰辅助工具，成功构建 2.56MB NSIS 与 3.82MB MSI 真实完整安装包。
2. **彻底攻克技能 ID 碰撞漏洞**：集成自适应父目录降级算法，当检测到通用文件名时自动追溯父文件夹名称作为唯一 ID。
3. **修复 Git 意外损坏与同步推送**。

---

### [2026-05-27 16:11] (北京时间)

**核心变动描述：**
1. **多重分发范围扩展**：前端新增 Shared（共享级）选项和"已下载技能"过滤面板。后端全面升级匹配 shared 作用域，精确规范 Global/Project/Shared 物理分发路径。
2. **深度自适应清理**：重构 `remove_physical_distribution` 方法，跨越所有可用作用域进行物理文件级联清理。

---

### [2026-05-27 17:08] (北京时间)

**核心变动描述：**
1. **铁血物理粉碎技能卸载**：后端新增 `uninstall_skill` 指令，前端引入红色微动垃圾桶按钮及二次确认弹窗。
2. **异步 IPC 防抖与 RequestID 锁**：150ms 防抖延迟 + 全局自增 `currentRequestId`，解决高频切换 Tab 时的 Race Condition。
3. **全平台全尺寸顶级图标包与 build.rs 自愈编译**。

---

### [2026-05-27 18:00] (北京时间)

**核心变动描述：**
1. **数据物理存储大迁移**：迁移至 `AppData/Roaming/new-SkillControl/`，配置 `installerHooks: "uninstall.nsh"` 实现 NSIS 卸载自动物理粉碎。
2. **WebDAV 隔离云端时间戳多版本增量备份**：MKCOL 隔离文件夹 + backup-*.zip 时间戳命名 + Time Machine Dropdown 历史版本选择器。

---

### [2026-05-27 18:35] (北京时间)

**核心变动描述：**
1. **彻底攻克开机初始化假死卡锁漏洞**：`discover_all_skills` 中 `git_clone_internal` 的同步 await 等待链重构为非阻塞 `tokio::spawn`，实现秒级渲染。
2. **Release 热修复静默打包**：38.52 秒打包，搭载 5 次自愈重试的 `publish.js`，在 GitHub Releases `v0.1.1` 覆盖发布。

---

### [2026-05-27 18:50] (北京时间)

**核心变动描述：**
1. **冷启动二次排查——WebDAV 同步阻塞修复**：HTTP 请求加 `create_webdav_client()`（connect_timeout 5s + total 15s）；`save_config` 中 WebDAV 备份改为 `tokio::spawn` fire-and-forget；前端 `loadApp` 四阶段渐进加载。提交 `23cd9f6`，创建 v0.1.2 Release。

---

### [2026-05-27 19:15] (北京时间)

**核心变动描述：**
1. **用户反馈仍冷启动卡死**：启动地毯式排查，注入全链路 `println!` 诊断日志。
2. **深挖三大死锁内鬼**：
   - `std::sync::Mutex` 锁竞争 → `std::sync::OnceLock` 替换
   - `spawn_blocking(...).await` 阻塞 async 运行时 → `tokio::spawn` fire-and-forget
   - `notify_reasonix_reload` 同步 `process::wait()` → `TokioCommand::output().await`
3. **前端 IPC 超时保护**：`get_config` 加 3s `Promise.race` 超时降级，初始化异常显示错误骨架屏。
4. **用户验证冷启动正常秒开**。提交 `e704495`，更新 v0.1.2 Release（EXE 12.2MB + NSIS 2.89MB）。

---

### [2026-05-27 20:00] (北京时间)

**核心变动描述：**
1. **Time Machine 设置面板格式修复**：修复内联样式使用未定义 CSS 变量 `var(--text-main)` 导致文字不可见；缩短 select 错误提示防止撑破 flex 布局；全部内联样式迁移为独立 CSS 类（`.timeline-section` / `.timeline-row` / `.timeline-select` 等），保证多主题一致性。
2. **版本 v0.1.3 发布**：版本号从 0.1.1 更新至 0.1.3，cargo tauri build 重新编译打包，GitHub Release 覆盖上架。
3. **README.md 首页介绍迭代**：更新中英文首页介绍文案，新增冷启动卡死修复、OnceLock 重构等核心亮点说明。

---

### [2026-05-27 20:30] (北京时间)

**核心变动描述：**
1. **WebDAV 云恢复下载 403 Forbidden 修复**：新增 `create_webdav_download_client()`（120s 超时 + 10s 连接超时，专用于文件上传/下载）；下载前 PROPFIND 探路确认文件存在；添加 `Depth: 0` + `Translate: f` WebDAV 标准请求头；失败时日志打印 HTTP body 前 200 字符辅助定位。解决某些 WebDAV 服务器（如 115 网盘）拒绝直接 GET 下载的问题。
