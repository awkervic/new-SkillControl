# new-SkillControl 增量总结 (Incremental Summaries)

---

### ### [2026-05-27 10:45] (北京时间)

**核心变动描述：**
1. **项目自述排版重构**：调整了项目根目录下的 `README.md`，将纯中文的项目介绍、核心亮点（自愈式编译、隐藏控制台管道、毛玻璃拟物 UI 等）调整至置顶区域，并在下方附加了英文自述，提升了面向国内开发者的可读性。
2. **Git 库大二进制文件清洗**：鉴于直接向 Git 代码库提交二进制大包（EXE/MSI）会导致仓库臃肿，我们将 `releases/` 目录从 Git 暂存区移除，并写入了 `.gitignore` 中进行长效过滤，确保仓库的轻量性。
3. **云端发布自动化**：利用 Node.js 提取本地已授权的 Git 凭证安全 Token，通过 GitHub API 自动将打包好且带有控制面板卸载注册功能的 `v0.1.0` EXE 及 MSI 安装程序发布至 GitHub Releases 官方托管页。
4. **Git 历史展平清洗**：利用分支强制重置与推送（Force Push），将包含大文件历史的多次提交归一化合并（Squash）为唯一一条极致纯净的 `feat: initial premium release of new-SkillControl` 初始记录。
5. **记忆持久化同步**：启动 `/ai-neat-skill` 整理动作，在项目根目录创建 `ai归档/编年体记忆` 系列文件，实现项目运行内存到文件的永久固化。

---

### ### [2026-05-27 11:05] (北京时间)

**核心变动描述：**
1. **彻底解决空壳包漏洞**：删除了 `src/bin/gen_ico.rs` 干扰辅助工具，防止 Tauri 打包器误将其作为主二进制打包成 200KB 的空壳，锁定并成功构建了含有 11MB+ 原生界面引擎与 IPC 核心的 **2.56MB NSIS（EXE）** 与 **3.82MB MSI** 真实完整安装包，并通过 Node.js API 实现了 GitHub Releases 云端的覆盖发布。
2. **彻底攻克技能下载联动与标记错误漏洞（ID 碰撞修复）**：
   - *漏洞成因*：由于多个技能的默认物理文件均为 `SKILL.md` 且其 yaml frontmatter 中没有配置 `id` 属性，系统先前默认截取文件名导致多技能唯一识别 ID 均退化为 `"SKILL"`，下载一个就会在前端全部亮起。
   - *修复方案*：在 `lib.rs` 的 Markdown 解析器中集成了**自适应父目录降级算法**——若检测到文件名是通用的 `SKILL`、`README` 等，系统会自动向上追溯其父文件夹名称（如 `ai-neat-skill`，`think-same-skill`）作为唯一 ID，从而完美实现技能卡片的**独立下载、独立分发、独立标记与零名冲突状态管理**。
3. **修复 Git 意外损坏与同步推送**：重新初始化并成功修复了受损的本地 Git 跟踪状态，保留了全部修改过的核心代码并同步提交发布上云。

---

### [2026-05-27 16:11] (北京时间)

**核心变动描述：**
1. **多重分发范围扩展 (Shared Scope Support)**：
   - 前端 (`ui/index.js`) 扩展了分发范围选择器，新增了 `Shared`（共享级）选项，为用户提供 Global、Project、Shared 三种维度分发能力。同时在侧边栏新增了专门的 `已下载技能` (Downloaded Skills) 过滤面板，可一键筛选本地已下载安装的技能。
   - 后端 (`src-tauri/src/lib.rs`) 全面升级以匹配 `shared` 作用域，并精确规范了各作用域的物理分发路径：
     - Global：`C:\Users\<username>\.gemini\antigravity-cli\skills\<skill_id>\SKILL.md`
     - Project/Workspace：`<project_root>\.agents\skills\<skill_id>\SKILL.md`
     - Shared：`C:\Users\<username>\.gemini\skills\<skill_id>\SKILL.md`
2. **深度自适应清理与多线程安全同步**：
   - 彻底重构了 `remove_physical_distribution` 方法。在进行技能卸载或 Scope 范围切换时，跨越所有可用作用域（包括已弃用的 Legacy `antigravity` 目录）进行物理文件的级联彻底清理和父文件夹递归安全擦除，彻底杜绝切换分发范围时的多重残留。
   - 优化了技能状态检测。通过同时分析 `staging` 暂存区与 `my-brain` 根目录的物理存在性，智能解析并动态绑定 `is_downloaded` / `is_installed` 元数据字段。


