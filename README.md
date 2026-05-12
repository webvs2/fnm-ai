# fnm-ai

`fnm-ai` 是一个 Windows 优先的 Node.js 版本管理工具。它基于 [fnm](https://github.com/Schniz/fnm) 的快速版本切换能力，并增加了自然语言入口：你可以直接说“帮我切换到 Node 20”“安装最新 LTS 并使用它”“检查当前环境配置”。

这个仓库同时包含底层 `fnm` Rust 二进制和 npm wrapper。npm 包当前发布名是 `fnm-ai`，安装后提供 `fnm-api` 和 `fnm-ai` 两个可执行命令。

## 特性

- 用中文或英文管理 Node.js 版本：安装、切换、列出、查看当前版本、设置默认版本、卸载。
- 默认使用 Kimi `kimi-k2.6`，通过 OpenAI-compatible `chat/completions` 接口调用模型。
- `fnm-api` 会读取本地配置，并在缺少 fnm 环境变量时自动执行 `fnm env --json` 初始化子进程环境。
- 交互模式内置两个常用预设：切换当前项目环境、检查当前环境配置。
- AI 输出只会映射到固定 fnm 动作，不会执行模型返回的任意 shell 命令。

## 支持范围

当前 npm 包面向 Windows x64：

```json
{
  "os": ["win32"],
  "cpu": ["x64"]
}
```

README 中的示例默认使用 Windows PowerShell。

## 安装

```powershell
npm install -g fnm-ai
```

安装后推荐使用 `fnm-api`：

```powershell
fnm-api "检查当前环境配置"
```

`fnm-ai` 是同一个 npm wrapper 的别名。底层 Rust 二进制也提供 `fnm ai` 和 `fnm api`：

```powershell
fnm ai "帮我切换到 Node 20"
fnm api "检查当前环境配置"
```

普通 npm 安装场景建议优先使用 `fnm-api`，因为它会自动读取 AI 配置并补齐常见的 fnm 子进程环境。

## 配置 AI Provider

默认模型是：

```text
kimi-k2.6
```

配置 Kimi：

```powershell
fnm-api config set --base-url https://api.moonshot.ai/v1 --api-key <your-kimi-api-key>
```

如果要显式指定模型：

```powershell
fnm-api config set --base-url https://api.moonshot.ai/v1 --api-key <your-kimi-api-key> --model kimi-k2.6
```

查看配置：

```powershell
fnm-api config get
```

输出会隐藏 API key 的中间部分：

```json
{
  "baseUrl": "https://api.moonshot.ai/v1",
  "apiKey": "sk-...xxxx",
  "model": null,
  "path": "C:\\Users\\you\\AppData\\Roaming\\fnm-ai\\config.json"
}
```

查看配置文件路径：

```powershell
fnm-api config path
```

Windows 默认位置通常是：

```text
%APPDATA%\fnm-ai\config.json
```

也可以用环境变量覆盖配置文件：

```powershell
$env:FNM_AI_BASE_URL = "https://api.moonshot.ai/v1"
$env:FNM_AI_API_KEY = "<your-kimi-api-key>"
$env:FNM_AI_MODEL = "kimi-k2.6"
```

环境变量优先级高于配置文件：

- `FNM_AI_BASE_URL` 覆盖 `baseUrl`
- `FNM_AI_API_KEY` 覆盖 `apiKey`
- `FNM_AI_MODEL` 覆盖 `model`

## 快速使用

安装并切换：

```powershell
fnm-api "安装 Node 20 并马上使用"
fnm-api "install node 20 and use it"
```

切换版本，缺失时自动安装：

```powershell
fnm-api "use node 22, install it if missing"
fnm-api "切换到 Node 18"
```

查看当前版本：

```powershell
fnm-api "现在用的是哪个 Node 版本"
fnm-api "what version am I using"
```

列出版本：

```powershell
fnm-api "列出已安装版本"
fnm-api "list remote lts versions"
```

设置默认版本：

```powershell
fnm-api "把 Node 20 设置成默认版本"
```

卸载版本：

```powershell
fnm-api "卸载 Node 16"
```

## 交互模式

不带参数运行会进入交互模式：

```powershell
fnm-api
```

启动后会看到两个预设：

```text
fnm ai is ready.
Presets:
  1. switch environment
  2. check current environment config
```

预设 `1` 会根据当前目录的 `.node-version`、`.nvmrc`、`package.json engines.node` 或默认版本切换环境。

预设 `2` 会打印 fnm 目录、Node 镜像、架构、版本文件策略、corepack、resolve engines、multishell 路径、PATH 状态和当前 Node 版本。

也可以直接说：

```powershell
fnm-api "先切换环境"
fnm-api "检查当前环境配置"
```

## PowerShell 环境说明

fnm 切换 Node 版本依赖运行时环境变量和 PATH，例如：

- `FNM_MULTISHELL_PATH`
- `FNM_DIR`
- `FNM_VERSION_FILE_STRATEGY`
- `FNM_NODE_DIST_MIRROR`
- `FNM_COREPACK_ENABLED`
- `FNM_RESOLVE_ENGINES`
- `FNM_ARCH`
- `PATH`

传统 fnm 用法需要在 shell profile 中加载：

```powershell
fnm env --use-on-cd --shell powershell | Out-String | Invoke-Expression
```

`fnm-api` 的 npm wrapper 会在启动时做额外初始化：

1. 读取 `%APPDATA%\fnm-ai\config.json`。
2. 把 `baseUrl`、`apiKey`、`model` 转成 `FNM_AI_BASE_URL`、`FNM_AI_API_KEY`、`FNM_AI_MODEL`。
3. 如果缺少 `FNM_MULTISHELL_PATH`，自动执行 `fnm env --json`。
4. 合并 `fnm env --json` 返回的 `FNM_*` 变量。
5. 把 multishell 路径加入子进程 `PATH`。
6. 启动真正的 `fnm ai`。

因此，即使当前 PowerShell 没有提前加载 `fnm env`，下面的命令也能在 `fnm-api` 子进程里看到完整 fnm 环境：

```powershell
fnm-api "检查当前环境配置"
```

需要注意的是，Windows、PowerShell 和 Node.js 子进程都有同一个限制：子进程不能永久修改已经运行中的父终端环境变量。

这意味着：

- `fnm-api` 可以为自己的子进程补齐 `FNM_*` 和 `PATH`。
- `fnm-api` 可以更新 fnm 的 multishell 链接，让 fnm 知道应该使用哪个 Node 版本。
- `fnm-api` 退出后，父 PowerShell 中已经存在的 `PATH` 不一定会自动刷新。

如果你希望 `fnm-api "切换到 Node 20"` 之后，当前 PowerShell 里的 `node -v` 也马上稳定使用该版本，请把下面这行加入 PowerShell profile：

```powershell
fnm env --use-on-cd --shell powershell | Out-String | Invoke-Expression
```

创建并打开 profile：

```powershell
if (-not (Test-Path $profile)) { New-Item $profile -Force }
Invoke-Item $profile
```

写入后重启 PowerShell。

## 安全模型

AI provider 只负责把自然语言请求翻译成固定 JSON 动作。当前允许的动作是：

```text
help
exit
switch_environment
check_environment
install
use
list_local
list_remote
current
default
uninstall
```

模型不能返回任意 shell 命令并让工具执行。自然语言请求最终会映射到这些 fnm 能力：

- `fnm install`
- `fnm use`
- `fnm list`
- `fnm list-remote`
- `fnm current`
- `fnm default`
- `fnm uninstall`
- `fnm env`

## 文档

- [命令列表](./docs/commands.md)
- [AI 使用说明](./docs/ai.md)
- [fnm 配置项](./docs/configuration.md)
- [nightly 说明](./docs/nightly.md)

## 开发

安装依赖：

```powershell
pnpm install
```

构建 Rust 二进制：

```powershell
cargo build
```

运行 AI 相关单元测试：

```powershell
cargo test ai::tests
```

运行 e2e：

```powershell
pnpm test -- e2e/ai.test.ts
```

测试 npm wrapper：

```powershell
node bin\fnm-ai.mjs config set --base-url https://api.moonshot.ai/v1 --api-key test-key
node bin\fnm-ai.mjs config get
node bin\fnm-ai.mjs "检查当前环境配置"
```

打包检查：

```powershell
npm pack --dry-run
```

发布前应确保：

- `cargo fmt` 已运行。
- `cargo test ai::tests` 通过。
- 相关 e2e 测试通过。
- `.changeset/` 中包含本次变更说明。

## License

GPL-3.0
