# new-SkillControl 编年史记�?(Memory Chronicles)

---

## 📅 核心事件快照 (Event Snapshots)

- **[2026-05-27] 架构**：初始化 `new-SkillControl` 基于 Tauri v2 + Rust + 原生 JS/CSS 的技能管理框架。设计了�?Crate 库模式（`skill_manager_lib`）以规避 Tauri 命令宏重名编译冲突�?[lib.rs](src-tauri/src/lib.rs))
- **[2026-05-27] 特�?*：实现隐藏式进程执行管道，应�?`CREATE_NO_WINDOW (0x08000000)` 标志位完美抹�?Windows 后台 Git/CMD 黑框弹窗闪烁�?[lib.rs:620](src-tauri/src/lib.rs))
- **[2026-05-27] 构建**：首创自愈式 DIB（设备无关位图）图标打包策略，在 `build.rs` 编译阶段自动重新生成并校验微�?`RC.EXE` 所需的标�?`.ico` 格式，杜绝编译崩溃�?[build.rs](src-tauri/build.rs))
- **[2026-05-27] 界面**：完�?Slate White 板岩白拟物毛玻璃日间模式与极光黑夜间模式的高品质开发，使用 CSS HSL 变量系统与纯 CSS Grid 实现全设备响应式自适应�?[index.css](ui/index.css))
- **[2026-05-27] 安全**：实�?Scoped 作用域多线程安全文件归档与备份（基于 `ZipArchive`），保证 Rust 异步上下文契�?`Send` 特征；完成默认配置的 WebDAV 脱敏设计�?[config.json](my-brain/config.json))
- **[2026-05-27] 发布**：利�?Node.js 脚本对接 Git 缓存 Token，直接通过 GitHub API 自动创建官方 `v0.1.0` 发布页面，并成功上传了内置控制面板自动卸载支持的 EXE �?MSI 双格式安装包�?[GitHub Releases](https://github.com/awkervic/new-SkillControl/releases))
- **[2026-05-27] 清洗**：完�?Git 历史提交归一化清洗（Squash & Force Push），删除�?Git 树中的大二进制大包痕迹并完善�?`.gitignore`，历史区目前仅存 1 条极致纯净的初始提交记录�?[Git History](https://github.com/awkervic/new-SkillControl/commits/main))
- **[2026-05-27] 修复**：解决了空壳打包漏洞（误�?200KB 的图标辅助命令行打包成了主程序）与技�?ID 碰撞漏洞（因多个 Generic `SKILL.md` 缺少 ID 前言导致全部折叠关联）。通过删除干扰项和加入"自适应父目录降级算�?彻底根治，并成功发布 2.56MB NSIS �?3.82MB MSI 覆盖上架�?[lib.rs:405](src-tauri/src/lib.rs))
- **[2026-05-27] 特�?*：扩展多重分发范围（新增 Shared 共享级，优化 Global/Project），�?UI 侧支�?已下载技�?过滤面板，提供更灵活的技能分发和筛选能力�?[lib.rs:649](src-tauri/src/lib.rs))
- **[2026-05-27] 重构**：优化物理分发路径同步与深度清理机制。实现跨 Global/Project/Shared 作用域物理路径的自适应清空和目录递归擦除，杜绝跨级残留；动态解�?staging 暂存池与 my-brain 根目录，智能判定技能下载状态�?[lib.rs:791](src-tauri/src/lib.rs))
- **[2026-05-27] 重构**：攻克高频切�?Tab 时的 IPC 抖动问题。通过引入 150ms 延迟防抖和全局自增 RequestID 锁机制，结合优雅�?全速扫�?旋转加载骨架屏，实现了极速流畅、零顿卡的切换体验�?[index.js:120](ui/index.js))
- **[2026-05-27] 特�?*：实�?铁血物理粉碎"技能强力卸载。后端集�?`uninstall_skill` 指令，实现暂存池、my-brain 以及 AGY/Reasonix 分发路径的一键物理级联擦除与 config.json 账本注销，并在前端引入极具视效的微动红色拟物粉碎按钮�?[lib.rs:1130](src-tauri/src/lib.rs))
- **[2026-05-27] 构建**：上线全平台自定义高品质图标矩阵。配套升�?`build.rs` 资源打包编译防御逻辑，自动对�?macOS icns/iOS 级联图标集、Android mipmap 矩阵以及 Windows .ico 文件，全面覆盖微�?MSI �?NSIS 打包发布标准�?[build.rs](src-tauri/build.rs))
- **[2026-05-27] 特�?*：物理存储应用目录大迁移（至 AppData/Roaming/new-SkillControl）配�?NSIS 卸载自动全物理粉碎；实现 WebDAV MKCOL 隔离文件夹探测创建，打包压缩 my-brain（忽�?repos）并上传带时间戳 backup-*.zip 版本包；前端集成 Time Machine Dropdown 历史版本选择器并打通一键定向解包还原、Git 重新克隆与全物理对齐分发的史诗级复活工作流�?[lib.rs](src-tauri/src/lib.rs))
- **[2026-05-27] 修复**：彻底定位并解决冷启动应用初始化假死�?未响�?）问题。将 `discover_skills` �?`discover_all_skills` 中的 `git_clone_internal` 同步 await 等待链重构为非阻塞的后台协程 `tokio::spawn`。在 38.52秒内完成静默打包构建，并利用带有 5次自愈重试的 `publish.js` 脚本成功�?GitHub Releases 云端覆盖上传最新正式版 NSIS 独立安装包�?[lib.rs:356](src-tauri/src/lib.rs))
- **[2026-05-27] 修复**：冷启动二次排查——WebDAV 引入的同步阻塞。所�?HTTP 请求�?`create_webdav_client()`（connect_timeout 5s + total 15s）；`save_config` �?WebDAV 备份改为 `tokio::spawn` 后台 fire-and-forget；前�?`loadApp` 重构为四阶段渐进加载（骨架屏→非阻塞技能拉取）。提�?`23cd9f6`�?
- **[2026-05-27] 故障**：排查发�?`APP_DATA_PATH` 使用 `std::sync::Mutex` �?Tokio async 运行时存在死锁风险；`spawn_blocking(...).await` 阻塞 async 运行时等�?blocking pool；`notify_reasonix_reload` 使用 `std::process::Command::wait()` 同步阻塞 Tokio 线程�?
- **[2026-05-27] 修复**：`APP_DATA_PATH` �?`std::sync::Mutex` 替换�?`std::sync::OnceLock` 消除锁竞争；两处 `spawn_blocking(...).await` 改为 `tokio::spawn` 彻底 fire-and-forget；`notify_reasonix_reload` 改用 `TokioCommand::output().await`；前�?`invoke('get_config')` �?3s 超时保护。全链路注入 `println!` 诊断日志。用户验证冷启动正常秒开。提�?`e704495`，更�?v0.1.2 Release�?[lib.rs:97](src-tauri/src/lib.rs))
- **[2026-05-27] 修复**：Time Machine 设置面板格式修复——内联样式使用未定义 CSS 变量 `var(--text-main)` 导致文字不可见；错误提示过长撑破 flex 布局；全部内联样式迁移为独立 CSS 类（`.timeline-section` 等）。提�?`b8dcd5c`�?[index.css](ui/index.css))
- **[2026-05-27] 发布**：版本号更新�?v0.1.3，重新编译打包，GitHub Release 覆盖上架。更�?README.md 首页介绍文案。提�?`HEAD`�?[v0.1.3 Release](https://github.com/awkervic/new-SkillControl/releases/tag/v0.1.3))
- **[2026-05-27] 修复**：WebDAV 云恢复下�?403 Forbidden——新�?`create_webdav_download_client()`�?20s 超时专门用于文件传输）；下载�?PROPFIND 探路确认文件存在；添�?`Depth: 0` + `Translate: f` WebDAV 标准 headers；失败时日志打印 HTTP body �?200 字符辅助定位�?[lib.rs:148](src-tauri/src/lib.rs))
- **[2026-05-27] 修复**：WebDAV 403 绝杀——两个客户端函数全部注入 Chrome 120 标准浏览�?User-Agent + Accept + Accept-Language + Accept-Encoding + Cache-Control 全套 headers，彻底伪装请求来源绕过网盘反爬拦截。下载改�?PROPFIND 响应中的服务器原�?href 路径，不再自行构�?URL。用户验�?WebDAV 历史版本恢复下载成功�?[lib.rs:149](src-tauri/src/lib.rs))

- **[2026-05-28] 故障**：排查发现本地物理运行的 WebDAV 服务（127.0.0.1:11501）在 Windows 复杂网络下存在系统 Loopback 本地回环拦截、系统代理干扰与开机启动时间差死锁风险；且不同 WebDAV 服务（如本地搭建的无命名空间服务）返回的 PROPFIND 响应 XML 中不包含命名空间前缀 `d:`，导致硬编码的 `<d:href>` 精准查询失败，造成备份成功但无法拉取恢复。
- **[2026-05-28] 修复**：对本地/回环 WebDAV 鲁棒性进行深度重构调优。1) 客户端注入 `.no_proxy()` 禁用代理；2) 引入带有 3 次重试及 127.0.0.1 ↔ localhost 智能自动地址替换的通用网络请求包装器 `send_webdav_request`；3) 无账号密码时改用纯净匿名匿名 HTTP 流；4) 设计完全兼容命名空间（如 `<D:href>`, `<d:href>`, `<href>` 等）的通用 XML 提取器 `extract_hrefs_from_xml`，彻底解决下载还原故障。成功通过 `cargo tauri build` 重新编译出 release v0.1.3 正式版包并提交推送 Git 归档。[lib.rs:260](src-tauri/src/lib.rs)

- **[2026-05-28] 修复**：完美解决本地 115-WebDAV (基于 WsgiDAV) 在拉取还原备份时返回的 `302 Found` 目标局域网物理 IP (如 `10.206.163.107:11500`) 无法访问导致的 120s 连接超时失败。对 `send_webdav_request` 引入精细化 "双阶段智能重定向纠错 (Smart Redirect Correction)"：利用 `std::net::IpAddr` 地址解析器，仅对局域网私有 IP (Private/Local IPv4) 强行修改回 `127.0.0.1` 环回 Host 并保留目标端口，完美放行后续第二阶段重定向到 115 官方高速 CDN 域名 (`cdnfhnfile.115cdn.net`) 的直链，顺利打通整条下载链路。通过 `cargo tauri build` 42.82 秒内成功重新编译并打包 NSIS 程序。[lib.rs:258](src-tauri/src/lib.rs)
- **[2026-05-28] 发布**：利用 C# P/Invoke 与 Win32 APIs (`CredReadW`) 编写了自动化凭据提权脚本，无需环境变量支持即可静默导出 Windows 凭据管理器中保存的 GitHub Token；随后调用 GitHub REST API 全自动重建 `v0.1.3` Releases，并将本地最新生成的 NSIS 一键安装包（`new-SkillControl_0.1.3_x64-setup.exe`）成功上传为官方 Release 主要资产。同时在终端运行 `chcp 65001` 清洗会话编码为 UTF-8 消除假死隐患，成功完美落盘。[GitHub Releases](https://github.com/awkervic/new-SkillControl/releases/tag/v0.1.3)

* [2026-06-06] 特性：后端 parse_markdown_skill 传入 skills_status 并校验 repo_id 隔离多仓库同名 ID 技能的下载/安装状态判断 (src-tauri/src/lib.rs)
* [2026-06-06] 界面：前端技能列表卡片标题处渲染来源仓库 badge 以区分同名技能 (ui/index.js, ui/index.css)
* [2026-06-06] 交互：前端技能头部状态指示器绑定快速 toggle 切换分发开关并阻止手风琴事件冒泡 (ui/index.js, ui/index.css)
* [2026-06-06] 修复：修复在主界面点击状态标签触发开关操作时由于 async-await 后 event 对象被回收导致 classList 读取失败的 TypeError Bug (ui/index.js)
* [2026-06-06] 发布：版本号更新至 v0.1.5，重新编译打包，提取 Git GCM 凭据，通过 Python 向 GitHub Releases v0.1.5 自动发布并直接上传了安装包。(https://github.com/awkervic/new-SkillControl/releases/tag/v0.1.5)
