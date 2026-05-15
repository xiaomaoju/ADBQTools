# AndroidQTools 设计文档

## 概述

AndroidQTools 是一个面向团队内部使用的 Android 开发辅助桌面工具，使用 Tauri 2 + Rust + Svelte + TypeScript 构建，支持 Windows 和 macOS 双平台。核心功能包括 Logcat 实时查看、APK 安装、AAB 安装，并针对 Unity 项目做了专门优化。工具内嵌 adb、bundletool、JRE，开箱即用，零外部依赖。

## 技术栈

- **桌面框架**：Tauri 2
- **后端**：Rust + Tokio（异步运行时）
- **前端**：Svelte + TypeScript
- **日志存储**：SQLite（通过 rusqlite）
- **打包资源**：内嵌 adb 二进制、bundletool.jar、Eclipse Temurin JRE

## 架构

采用方案 A：Rust 通过 `std::process::Command` / `tokio::process::Command` 调用内嵌 adb 二进制，所有设备通信、logcat 流、安装操作在 Rust 侧完成，通过 Tauri Commands（请求/响应）和 Tauri Events（实时推流）与前端通信。

```
┌──────────────────────────────────────────────┐
│              Svelte + TS 前端                 │
│  ┌──────────┐ ┌──────────┐ ┌──────────────┐  │
│  │ 设备管理  │ │ Logcat   │ │ 安装器       │  │
│  │ 面板     │ │ 查看器    │ │ (APK/AAB)    │  │
│  └────┬─────┘ └────┬─────┘ └──────┬───────┘  │
│       └────────────┼──────────────┘           │
│                    │ Tauri Commands / Events   │
├────────────────────┼─────────────────────────-┤
│              Rust 后端                        │
│  ┌──────────┐ ┌──────────┐ ┌──────────────┐  │
│  │ Device   │ │ Logcat   │ │ Installer    │  │
│  │ Manager  │ │ Engine   │ │              │  │
│  └────┬─────┘ └────┬─────┘ └──────┬───────┘  │
│       └────────────┼──────────────┘           │
│        内嵌 adb + bundletool + JRE             │
└──────────────────────────────────────────────┘
```

## 模块设计

### 1. 设备管理模块（device_manager.rs）

**职责**：管理所有已连接的 Android 设备，支持 USB、WiFi、多设备。

**Rust 侧实现**：
- 每 2 秒轮询 `adb devices -l`，解析出 serial、型号、状态
- WiFi 连接：`adb tcpip 5555` + `adb connect <ip>:<port>`
- Android 11+ 无线调试配对：`adb pair <ip>:<port> <code>`
- 全局设备状态：`Arc<Mutex<HashMap<String, Device>>>`，设备变更时通过 Tauri Event 推送

**数据结构**：
```rust
enum TransportType { Usb, Wifi }
enum DeviceStatus { Online, Offline, Unauthorized }

struct Device {
    serial: String,
    model: String,
    product: String,
    transport: TransportType,
    status: DeviceStatus,
}
```

**前端交互**：
- 左侧边栏设备列表：设备名、型号、连接方式图标（USB/WiFi）、状态指示灯
- 点击设备切换"当前活跃设备"
- 顶部"WiFi 连接"按钮弹出对话框输入 IP:Port 和配对码

### 2. Logcat 查看器模块（logcat_engine.rs）

**职责**：多设备实时 logcat 查看，支持过滤、搜索、导出、持久化，针对 Unity 项目优化。

**Rust 侧实现**：
- 每个设备独立启动 `adb -s <serial> logcat -v threadtime` 子进程
- 用 `tokio::process::Command` 异步逐行读取 stdout
- 解析每行为结构化 `LogEntry`
- 批次推送到前端：每 100ms 或累积 50 条触发一次
- 暂停/恢复：暂停时缓存到内存环形缓冲区（上限 10 万条）
- 持久化：异步写入 SQLite（按设备+日期分表）
- 支持 clear、export（txt/csv）

**数据结构**：
```rust
enum LogLevel { Verbose, Debug, Info, Warn, Error, Fatal }
enum LogSource { System, Unity, Il2Cpp, Mono }

struct LogEntry {
    timestamp: String,
    pid: u32,
    tid: u32,
    level: LogLevel,
    tag: String,
    message: String,
    source: LogSource,
    stack_frames: Option<Vec<StackFrame>>,
    unity_script_info: Option<ScriptInfo>,
}

struct StackFrame {
    module: String,
    class_name: String,
    method_name: String,
    file: Option<String>,
    line: Option<u32>,
}

struct ScriptInfo {
    file: String,
    line: u32,
}
```

**Unity 适配**：
- 自动识别 Unity 相关 Tag：`Unity`、`CRASH`、`Il2Cpp`、`Mono`
- 解析 Unity `Debug.Log/LogWarning/LogError` 的二级结构，提取 C# 脚本名和行号
- 识别 IL2CPP 和 Mono 崩溃堆栈，合并连续堆栈帧为可折叠组
- 识别 native crash 信号（SIGABRT、SIGSEGV 等）并高亮

**前端交互**：
- 虚拟滚动列表（处理百万行不卡顿）
- 按级别着色：V 灰 / D 蓝 / I 绿 / W 黄 / E 红 / F 紫
- 过滤工具栏：级别下拉、Tag 多选、关键字/正则搜索、PID/进程名过滤
- "Unity 模式"快捷按钮：一键过滤 Unity/Il2Cpp/Mono/CRASH Tag
- 预置过滤模板：Unity All、Unity Errors、Unity Scripting、Unity Rendering、Unity Network
- Unity 日志行特殊图标标识
- 崩溃堆栈可折叠/展开，C# 类名和方法名高亮
- 搜索支持按 C# 类名/方法名匹配
- 标签页切换多设备 logcat
- 工具栏：暂停/恢复、清屏、导出、自动滚动锁定
- 历史回查面板：选择日期范围查询 SQLite 历史日志

### 3. 安装器模块（installer.rs）

**职责**：安装 APK 和 AAB 文件到目标设备。

**APK 安装**：
- 调用 `adb -s <serial> install -r -d <path>`，`-r` 覆盖安装，`-d` 允许降级
- 解析安装进度通过 Tauri Event 实时推送
- 安装前用 Rust 解析 APK ZIP 内的 AndroidManifest.xml（二进制 AXML 格式）提取应用信息（包名、版本、图标、最低 SDK），不依赖 aapt2

**AAB 安装**：
- 调用内嵌 JRE 运行 `bundletool build-apks --bundle=<path> --output=<tmp>.apks --connected-device --adb=<内嵌adb路径>`
- 再调用 `bundletool install-apks --apks=<tmp>.apks --adb=<内嵌adb路径>`
- 临时 .apks 文件安装后自动清理
- 支持签名配置（keystore 路径、别名、密码），保存到本地加密配置

**内嵌资源结构**：
```
resources/
├── windows/
│   ├── adb.exe + AdbWinApi.dll
│   ├── bundletool.jar
│   └── jre/          # Temurin JRE (Windows x64)
└── macos/
    ├── adb
    ├── bundletool.jar
    └── jre/          # Temurin JRE (macOS arm64 + x64)
```

- 运行时根据 `std::env::consts::OS` 选择对应平台路径
- 首次启动解压 JRE 到 `app_data_dir`，后续直接使用

**前端交互**：
- 拖拽 APK/AAB 文件到窗口或点击按钮选择文件
- 应用预览卡片：图标、包名、版本号、文件大小
- 安装进度条 + 状态文字（解析中 → 构建 APKs → 安装中 → 完成/失败）
- AAB 签名配置区域（可记住上次配置）
- 安装历史记录列表：时间、文件名、目标设备、结果

### 4. 内嵌资源管理模块（embedded.rs）

**职责**：管理内嵌的 adb、bundletool、JRE 资源。

- 按平台打包对应二进制到 Tauri resources 目录
- 运行时解压 JRE 到 `app_data_dir`（首次启动时）
- 提供统一的路径解析 API，屏蔽平台差异
- macOS 需要处理 adb 二进制的可执行权限（`chmod +x`）

## UI 设计

### 整体布局

```
┌──────────────────────────────────────────────────┐
│  标题栏：AndroidQTools                ─ □ ×      │
├────────┬─────────────────────────────────────────┤
│        │  工具栏：[Logcat] [安装器]   设备选择▼    │
│ 设     ├─────────────────────────────────────────┤
│ 备     │                                         │
│ 列     │         主内容区                         │
│ 表     │   (Logcat 查看器 / 安装器 切换)           │
│        │                                         │
│ ────── │                                         │
│ WiFi   │                                         │
│ 连接   │                                         │
│        │                                         │
├────────┴─────────────────────────────────────────┤
│  状态栏：已连接 2 台设备 | ADB v37.0 | JRE ✓      │
└──────────────────────────────────────────────────┘
```

### 视觉规范

- **主题**：深色主题，背景 `#1e1e1e`，面板 `#252526`，边框 `#3c3c3c`
- **强调色**：`#007acc`（蓝色）
- **字体**：Logcat 区使用 `JetBrains Mono` 等宽字体，界面区使用系统 UI 字体
- **日志级别色**：V `#6a6a6a` / D `#4fc1ff` / I `#4ec9b0` / W `#cca700` / E `#f44747` / F `#c586c0`
- **交互**：按钮 hover 变亮、点击细微动画、拖拽区域虚线边框高亮

### 响应式

- 左侧设备面板可折叠，小屏自动收起为图标栏
- Logcat 区域占满剩余空间

### 组件

- 自研轻量组件：按钮、输入框、下拉菜单、标签页、对话框、进度条
- CSS 变量管理主题色，为亮色主题预留扩展

## 项目结构

```
AndroidQTools/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs              # Tauri 入口
│   │   ├── device_manager.rs    # 设备管理
│   │   ├── logcat_engine.rs     # Logcat 流引擎
│   │   ├── installer.rs         # APK/AAB 安装
│   │   ├── embedded.rs          # 内嵌资源管理
│   │   ├── db.rs                # SQLite 日志持久化
│   │   └── commands.rs          # Tauri Command 注册
│   ├── resources/               # 内嵌二进制资源
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/
│   ├── App.svelte               # 主应用
│   ├── lib/
│   │   ├── components/
│   │   │   ├── DevicePanel.svelte
│   │   │   ├── LogcatViewer.svelte
│   │   │   ├── Installer.svelte
│   │   │   ├── Toolbar.svelte
│   │   │   ├── StatusBar.svelte
│   │   │   └── ui/              # 通用 UI 组件
│   │   ├── stores/
│   │   │   ├── devices.ts       # 设备状态 store
│   │   │   ├── logcat.ts        # 日志 store
│   │   │   └── installer.ts     # 安装状态 store
│   │   └── utils/
│   │       ├── tauri.ts         # Tauri IPC 封装
│   │       └── format.ts        # 格式化工具
│   ├── styles/
│   │   ├── global.css           # 全局样式
│   │   └── variables.css        # CSS 变量/主题
│   └── app.html
├── package.json
├── svelte.config.js
├── tsconfig.json
└── vite.config.ts
```

## 跨平台考虑

- **路径分隔符**：Rust 侧统一使用 `std::path::PathBuf`，自动适配
- **可执行权限**：macOS 打包后需 `chmod +x` 内嵌 adb，在 `embedded.rs` 首次初始化时处理
- **JRE 架构**：macOS 同时打包 arm64（Apple Silicon）和 x64（Intel），运行时检测；Windows 打包 x64
- **签名与公证**：macOS 发布需 Apple Developer 证书签名 + 公证；Windows 可选 EV 代码签名
- **安装包格式**：macOS 生成 `.dmg`，Windows 生成 `.msi`/`.exe`（Tauri 内置支持）

## Rust 关键依赖

- `tauri` 2.x — 桌面框架
- `tokio` — 异步运行时
- `rusqlite` — SQLite 绑定
- `serde` / `serde_json` — 序列化
- `regex` — 日志解析
- `dirs` — 跨平台目录路径
