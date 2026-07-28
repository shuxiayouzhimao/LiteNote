# LiteNote 灵光记事本（Tauri + Vue 3 版）

一款极轻量、现代化（毛玻璃 / 圆角 / 无边框 / 双主题）的 Windows 桌面记事客户端。
使用 **Tauri v2（Rust 后端）+ Vue 3（TypeScript 前端）** 构建，安装包体积约 3~8MB，秒开、低内存。

---

## 一、项目结构

```
LiteNote/
├── index.html                  # 前端 HTML 入口
├── package.json                # 前端依赖与脚本
├── vite.config.ts              # Vite 配置
├── tsconfig.json               # TypeScript 配置
│
├── src/                        # === 前端源码（Vue 3）===
│   ├── main.ts                 # Vue 应用入口
│   ├── App.vue                 # 根组件（三栏布局 + 快捷键 + 事件监听）
│   ├── vite-env.d.ts           # 类型声明
│   ├── api/
│   │   ├── index.ts            # 封装所有 invoke 后端调用
│   │   └── types.ts            # 前后端共享类型定义
│   ├── stores/                 # Pinia 状态管理（相当于控制器层）
│   │   ├── notes.ts            # 笔记状态：列表/选中/自动保存/搜索
│   │   └── config.ts           # 配置状态：主题/自启
│   ├── components/             # Vue 组件（界面层）
│   │   ├── TitleBar.vue        # 自定义标题栏（拖动/主题/窗口按钮）
│   │   ├── Sidebar.vue         # 侧边栏导航
│   │   ├── NoteList.vue        # 笔记列表 + 搜索 + 右键菜单
│   │   ├── Editor.vue          # 编辑区
│   │   ├── SettingsDialog.vue  # 设置对话框
│   │   └── Toast.vue           # 轻提示
│   ├── composables/
│   │   ├── useToast.ts         # Toast 组合式函数
│   │   └── utils.ts            # 工具函数（时间/文本格式化）
│   └── styles/
│       └── global.css          # 全局样式 + 双主题 CSS 变量
│
└── src-tauri/                  # === 后端源码（Rust）===
    ├── Cargo.toml              # Rust 依赖
    ├── build.rs                # 构建脚本
    ├── tauri.conf.json         # Tauri 配置（窗口/打包/图标）
    ├── capabilities/
    │   └── default.json        # 权限声明
    ├── icons/                  # 应用图标
    └── src/
        ├── main.rs             # 可执行入口
        ├── lib.rs              # 应用组装（插件/命令/托盘/快捷键）
        ├── db.rs               # SQLite 数据库操作
        ├── commands.rs         # Tauri 命令（前端可调用接口）
        ├── config.rs           # 配置读写
        └── tray.rs             # 系统托盘
```

**架构说明**：前端（界面 + 状态）通过 `invoke` 调用后端命令 → 后端 `commands.rs` → `db.rs` 操作 SQLite。职责清晰分离为「界面（components）/ 状态控制（stores）/ 数据（Rust db）」三层。

---

## 二、环境准备

> **关于"虚拟环境"**：Tauri 项目没有 Python 的 venv。前端依赖天然隔离在项目本地的 `node_modules`，Rust 依赖隔离在 `src-tauri/target`，都不污染全局，等价于隔离环境。无需额外操作。

1. **Node.js 18+**
2. **Rust 工具链**：
   - 访问 https://rustup.rs 下载 `rustup-init.exe` 并安装
   - 安装后重启终端，验证：`rustc --version` 和 `cargo --version`
3. **Windows 构建依赖**：
   - **MSVC 生成工具**：安装 [Visual Studio 2022 生成工具](https://visualstudio.microsoft.com/downloads/)，勾选「使用 C++ 的桌面开发」
   - **WebView2 Runtime**：Windows 10/11 一般已内置；若无，构建产物会自动下载安装

---

## 三、安装依赖

```bash
# 进入项目目录
cd LiteNote

# 安装前端依赖（隔离在 node_modules，不污染全局）
npm install
```

> 首次运行 `cargo tauri dev` 时会自动下载并编译 Rust 依赖（较慢，之后有缓存）。

---

## 四、开发运行

```bash
npm run tauri dev
```

- 启动后自动打开应用窗口，支持前端热重载
- Rust 代码修改后会自动重新编译

---

## 五、打包发布

```bash
npm run tauri build
```

产物位于：

```
src-tauri/target/release/bundle/
├── msi/LiteNote_1.0.0_x64_en-US.msi     # MSI 安装包
└── nsis/LiteNote_1.0.0_x64-setup.exe    # NSIS 安装程序
```

单文件绿色版可执行文件位于：

```
src-tauri/target/release/litenote.exe
```

---

## 六、功能一览

| 分类 | 功能 |
|------|------|
| 核心 | 新建 / 编辑（自动保存）/ 删除 / 列表 / SQLite 本地存储 / 实时搜索 / 系统托盘 / 双主题 |
| 进阶 | 收藏 ⭐ / 回收站（恢复+永久删除）/ 导出 .txt/.md / 置顶 📌 |
| 加分 | 开机自启 / 全局快捷键 Ctrl+Shift+N / 笔记数与字数统计 |

### 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+N` / `Ctrl+Shift+N` | 新建笔记 |
| `Ctrl+F` | 聚焦搜索框 |
| `Ctrl+S` | 手动保存 |
| `Ctrl+W` | 关闭当前笔记 |
| `Delete` | 删除选中笔记 |
| `Esc` | 取消选中 / 退出搜索 |

---

## 七、数据与配置存储

```
%APPDATA%/com.litenote.app/   （或 LiteNote 相关目录）
├── data.db          # SQLite 数据库
└── config.json      # 主题偏好、自启、上次打开笔记等
```

---

## 八、自定义应用图标（可选）

`src-tauri/icons/` 内为程序自动生成的占位紫色图标。如需自定义，准备一张 `1024x1024` 的 PNG，然后运行：

```bash
npm run tauri icon path/to/your-icon.png
```

Tauri 会自动生成各平台所需的全套图标（含 macOS `.icns`）。

---

*文档版本：v2.0（Tauri 版）· 适用平台：Windows 10/11*
