# new-SkillControl 🚀

[中文](#中文) | [English](#english)

---

## 中文

`new-SkillControl` 是一款基于 **Tauri v2** + **Rust** + **原生 JS/CSS** 打造的高性能、高阶审美 AI Agent 技能管理器。用于无缝同步、管理和物理分发如 Anthropic Claude Code（agycli）、AGY 2.0、Reasonix 等本地代理框架的各类自定义 Skill。

### ✨ 核心亮点

1. **极简拟物折叠面板 UI (Accordion List)**：全面摒弃了臃肿的传统卡牌布局，重构为极致清爽的条形手风琴（Collapsible Bar）目录结构。支持二级目录平滑展开以显示技能详情、来源、开关和分发范围，头部内置状态指示灯，高保真还原 Slate White（板岩白）毛玻璃拟物美学。
2. **智能差异比对引擎 (LCS Diff Viewer)**：内置原生 JavaScript 最长公共子序列（LCS）算法。可一键拉取对比本地已安装版本与 Git 仓库最新版本，以绿底（`+` 增加）与红底（`-` 删除）高亮行直观呈现代码差异。
3. **双版本（AGY CLI / AGY 2.0）物理分发**：支持将 Skill 物理镜像同时或独立同步至 `agycli` (`.gemini/antigravity-cli`) 与最新的 `AGY 2.0` (`.gemini/antigravity`) 全局/项目/共享工作区中，并深度协同 `config.json` 索引文件。
4. **内存持久化缓存与加速 (SKILLS_CACHE)**：在 Rust 后端建立基于 `OnceLock<Mutex<Option<Vec<SkillMetadata>>>>` 的静态预加载缓存。除主动同步或安装动作外，日常页面切换、分组过滤与模糊搜索耗时降低至 0ms，大幅减少磁盘 I/O 并降低冷启动延迟。
5. **快捷键与键盘流深度集成**：深度优化开发者交互效率。支持 `Ctrl + F` 或 `/` 快捷键瞬间聚焦并全选搜索框；支持 `Esc` 键一键折叠所有展开项、退出搜索焦点及关闭设置弹窗。
6. **无感 CMD 隐式执行管道**：重构了本地命令行调用机制，使用 Rust `tokio::process::Command` 搭配 Windows 原生句柄标识 `CREATE_NO_WINDOW (0x08000000)`，彻底消除调用 Git 同步时产生控制台黑框闪烁的问题。
7. **云端拉取自适应更新与清理**：在 Git 仓库镜像拉取时执行自愈清理。若云端 Skill 有所更新，会自动触发 Staging 重分发；若某 Skill 被云端移除或改名，系统将物理粉碎本地 staging、`.gemini` 以及 `.reasonix` 中的残留文件，杜绝新旧版本 Skill 并存冲突。
8. **一键 WebDAV 云恢复与自动备份**：支持在配置更新或仓库同步时自动触发本地声明式账本（无冗余大文件体积）的静默压缩备份，并通过 WebDAV 一键极速恢复工作流。

### 🛠️ 技术栈与底层设计

- **前端**：原生 JavaScript (ES6+), 原生 CSS (自定义 HSL 调色系统、毛玻璃动效、CSS Grid & Flex), HTML5。零库依赖，追求极致的渲染效率与纯粹。
- **后端**：Rust (Tauri v2)
- **异步框架**：Tokio
- **数据序列化**：YAML Frontmatter (元数据解析) + JSON (配置账本)

### 📥 快速上手与编译

#### 开发环境准备
- 安装 Rust 与 Cargo 工具链。
- 安装 Node.js (v18+) 及 NPM。
- Windows 用户需安装 C++ 生成工具 (MSVC)。

#### 启动开发服务器
```bash
# 纯 Cargo 驱动，直接编译并启动前端
cargo tauri dev
```

#### 打包生产环境安装包
```bash
# 打包为独立的 MSI 或 NSIS 静态单文件安装包
cargo tauri build
```

---

## English

`new-SkillControl` is a high-performance, premium desktop AI agent skill manager built on **Tauri v2**, **Rust**, and **Vanilla JS/CSS**. It allows developers and power users to seamlessly manage, synchronize, and deploy custom agent skills across multiple environments (such as Anthropic Claude Code / agycli, AGY 2.0, and Reasonix).

### ✨ Key Features

1. **Elegant Accordion List UI**: Replaces the old grid cards layout with a highly compact, collapsible bar (Accordion) directory. Smoothly expands to show full details, scopes, source info, and action buttons. Features reactive active badges (AGY 1.0, AGY 2.0, Reasonix) visible directly on the collapsed header.
2. **Instant LCS Diff Viewer**: Integrates a lightweight, native JavaScript Longest Common Subsequence (LCS) diff algorithm. Highlights line-by-line additions (green `+`) and deletions (red `-`) between your locally installed staged copy and the remote repository version at any time.
3. **Multi-Platform (AGY CLI / AGY 2.0) Deployments**: Adds dedicated support for the new AGY 2.0 environment. Seamlessly toggles and deploys physical `SKILL.md` documents and synchronizes JSON configuration indexes to both `agycli` (`.gemini/antigravity-cli`) and `AGY 2.0` (`.gemini/antigravity`).
4. **Static Memory Caching (SKILLS_CACHE)**: Employs a thread-safe `OnceLock<Mutex<Option<Vec<SkillMetadata>>>>` memory cache in Rust. Normal page filtering, tab switching, and fuzzy search queries bypass the disk entirely with a 0ms response time, significantly optimizing CPU and I/O efficiency.
5. **Keyboard shortcuts integration**: Press `Ctrl + F` or `/` to focus and highlight the global search bar instantly. Press `Esc` to blur inputs, collapse all expanded directory rows, and close setting modals immediately.
6. **Background CMD Command Pipeline**: Solves the annoying flashing black CMD windows on Windows systems by integrating standard process pipelines wrapped in Rust's `tokio::process::Command` using native Windows `CREATE_NO_WINDOW (0x08000000)` flags.
7. **Cloud Auto-Update & Orphan Cleanup**: Runs repository cache scans during syncs. Automatically stages updates if cloud files change, and physically wipes out staged, `.gemini`, and `.reasonix` distributions if a skill is removed or renamed in the repository to prevent duplicates.
8. **Secure Local & Cloud Resurrect**: Background automatic zip packing and WebDAV cloud integration to seamlessly synchronize and resurrect entire skill workspaces across devices with one click.

### 🛠️ Tech Stack & Architecture

- **Frontend**: Vanilla JS (ES6+), Vanilla CSS (Custom properties, CSS grid/flex, HSL-color tokens), HTML5. Zero bloated UI libraries.
- **Backend (Tauri v2 Core)**: Written in Rust.
- **Asynchronous Engine**: Tokio.
- **Data Serialization**: YAML frontmatter + JSON config structures.

### 📥 Getting Started & Building

#### Prerequisites
- Rust & Cargo installed.
- Node.js (v18+) & NPM.
- On Windows: C++ Build Tools (MSVC).

#### Running in Development Mode
```bash
# Pure Cargo driven dev command
cargo tauri dev
```

#### Packaging Production Builds
```bash
# Package into standalone MSI/NSIS installation packages
cargo tauri build
```
