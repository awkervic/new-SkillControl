# new-SkillControl 🚀

[中文](#中文) | [English](#english)

---

## 中文

`new-SkillControl` 是一款基于 **Tauri v2** + **Rust** + **原生 JS/CSS** 打造的高性能、高颜值 AI Agent 技能管理器。用于无缝管理、同步和分发如 Anthropic Claude Code、Reasonix 等本地代理框架的各类自定义技能。

### ✨ 核心亮点

1. **高端微动效拟物 UI**：完美适配日间/夜间模式，日间模式采用高端的 **Slate White（板岩白）毛玻璃极简设计**，夜间模式采用深邃高对比度配色，搭配细腻的微交互动效。
2. **无感 CMD 隐式执行管道**：深度重构了本地进程调用机制，使用 Rust `tokio::process::Command` 搭配 Windows 原生句柄标识 `CREATE_NO_WINDOW (0x08000000)`，杜绝了执行 Git/命令行时产生黑色控制台弹窗闪烁的问题。
3. **多仓库解耦同步机制**：支持本地技能与远程公共技能库、私人技能库的完全解耦。支持一键克隆、更新、物理备份与软删除，保障本地工作区数据安全。
4. **自愈式自适应编译系统**：在 `build.rs` 中集成了专有的 DIB（设备无关位图）转换引擎，在 Windows 编译期间动态优化应用图标为符合微软 `RC.EXE` 严格规范的传统 `.ico` 资源，杜绝图标引起的打包崩溃。
5. **一键 WebDAV 云恢复与自动备份**：采用高度可靠的异步压缩策略，支持在配置更新或仓库同步时触发静默本地压缩备份，并支持通过 WebDAV 云端一键极速重构。
6. **作用域线程安全设计**：利用作用域包装排他性句柄（如 `ZipArchive`），保证 Rust 异步期（Future）完美契合 `Send` 特性，确保高并发下的系统内存绝对安全。

### 🛠️ 技术栈与底层设计

- **前端**：原生 JavaScript (ES6+), 原生 CSS (自定义变量、HSL 调色系统、响应式网格), HTML5。拒绝臃肿框架，追求极致渲染性能。
- **后端**：Rust (Tauri v2)
- **异步框架**：Tokio
- **数据结构**：YAML Frontmatter (解析技能描述与元数据) + JSON

### 📥 快速上手与编译

#### 开发环境准备
- 安装 Rust 与 Cargo 工具链。
- 安装 Node.js (v18+) 及 NPM。
- Windows 用户需安装 C++ 生成工具 (MSVC)。

#### 启动开发服务器
```bash
npm install
npm run tauri dev
```

#### 打包生产环境包
```bash
# 打包为独立的 MSI 及 NSIS 安装程序
npm run tauri build
```

---

## English

`new-SkillControl` is a high-performance, premium desktop AI agent skill manager built on **Tauri v2**, **Rust**, and **Vanilla JS/CSS**. It enables developers and power users to seamlessly manage, synchronize, and deploy custom agent skills across multiple environments (such as Anthropic Claude Code, Reasonix, and other local agent pipelines).

### ✨ Key Features

1. **Aesthetic & Responsive UI**: Seamlessly responds to light/dark system themes. Featuring a premium **Glassmorphism Slate White** Day Mode and a sleek, high-contrast Dark Mode with micro-animations.
2. **Hidden CMD Command Pipeline**: Solves the annoying flashing black CMD windows on Windows systems by integrating standard process pipelines wrapped in Rust's `tokio::process::Command` using native Windows `CREATE_NO_WINDOW (0x08000000)` flags.
3. **Robust Decoupled Multi-Repository Syncing**: Fully decoupling local workspace targets from public and private repositories, making skill deployments safe, non-destructive, and highly traceable.
4. **Self-Healing Build System**: Integrates a dynamic DIB (Device-Independent Bitmap) packer inside `build.rs` that automatically sanitizes and rebuilds Windows `.ico` formats at compile time, circumventing Microsoft's legacy `RC.EXE` compiler failures.
5. **Secure Local & Cloud Resurrect**: Background automatic zip packing and WebDAV cloud integration to seamlessly synchronize and resurrect entire skill workspaces across devices with one click.
6. **Scoped Thread-Safe Async Operations**: Eliminates async compiler safety issues (like `Send` boundary issues in futures) by scoping zip archivers within isolated, transient code blocks.

### 🛠️ Tech Stack & Architecture

- **Frontend**: Vanilla JS (ES6+), Vanilla CSS (Custom properties, CSS grid, HSL-color tokens), HTML5. Zero bloated UI libraries.
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
# From the project root
npm install
npm run tauri dev
```

#### Packaging Production Builds
```bash
# Package into standalone MSI/NSIS installation packages
npm run tauri build
```
