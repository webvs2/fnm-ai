# @fnm/ai

`@fnm/ai` 是一个面向 Windows 的 Node.js 版本管理工具。它基于 fnm，但把主要交互方式从命令行参数升级为自然语言对话：你可以直接说“帮我切换到 Node 20”“检查当前环境配置”“安装最新 LTS 并使用它”。

当前项目从 `1.0.0` 开始发布，npm 包名为 `@fnm/ai`。

## 核心能力

- 使用自然语言管理 Node.js 版本：安装、切换、查看当前版本、列出版本、设置默认版本、卸载版本。
- 默认使用 Kimi 模型 `kimi-k2.6`，只需要配置 OpenAI-compatible `baseUrl` 和 API key。
- npm 安装后提供 `fnm-api` 和 `fnm-ai` 两个可执行命令。
- 内置两个交互预设：先切换环境、检查当前环境配置。
- npm wrapper 会在缺少 fnm 环境变量时自动执行 `fnm env --json`，让当前子进程获得可用的 `FNM_*` 和 `PATH`。
- AI 输出只会被映射到固定的 fnm 动作，不会执行模型返回的任意 shell 命令。

## 当前支持范围

当前 npm 包面向 Windows x64：

```json
{
  "os": ["win32"],
  "cpu": ["x64"]
}
```

后续可以再扩展为通用 npm 包。当前 README 以 Windows PowerShell 使用方式为主。

## 快速开始

安装：

```powershell
npm install -g fnm-ai
```

配置 Kimi：

```powershell
fnm-api config set --base-url https://api.moonshot.ai/v1 --api-key <your-kimi-api-key>
```

开始使用：

```powershell
fnm-api "帮我切换到 Node 20"
fnm-api "安装最新 LTS，并切换过去"
fnm-api "检查当前环境配置"
fnm-api "列出已安装的 Node 版本"
```

启动交互模式：

```powershell
fnm-api
```

## 命令名称说明

npm 包名是 `@fnm/ai`，但安装后的可执行命令是：

```powershell
fnm-api
fnm-ai
```

`@fnm/api` 这种带 slash 的名字更适合作为 npm 包名或包 specifier，不适合作为跨平台 shell 命令。npm 在 Windows 上生成 bin shim 时也不会把它稳定暴露成 `@fnm/api` 这种命令形态，所以这里使用 `fnm-api` 作为推荐入口。

如果你直接使用底层 fnm 二进制，也可以使用：

```powershell
fnm ai "帮我切换到 Node 20"
fnm api "检查当前环境配置"
```

推荐普通用户使用 `fnm-api`，因为它会自动读取 `@fnm/ai` 配置文件，并处理 npm 场景下常见的环境变量缺失问题。

## AI 配置

默认模型是 Kimi：

```text
kimi-k2.6
```

默认推荐 base URL：

```text
https://api.moonshot.ai/v1
```

保存配置：

```powershell
fnm-api config set --base-url https://api.moonshot.ai/v1 --api-key <your-kimi-api-key>
```

指定其他模型：

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

Windows 默认配置文件位置通常是：

```text
%APPDATA%\fnm-ai\config.json
```

## 环境变量覆盖

你也可以不用配置文件，直接用环境变量：

```powershell
$env:FNM_AI_BASE_URL = "https://api.moonshot.ai/v1"
$env:FNM_AI_API_KEY = "<your-kimi-api-key>"
$env:FNM_AI_MODEL = "kimi-k2.6"
```

环境变量优先级高于配置文件：

- `FNM_AI_BASE_URL` 覆盖配置里的 `baseUrl`
- `FNM_AI_API_KEY` 覆盖配置里的 `apiKey`
- `FNM_AI_MODEL` 覆盖配置里的 `model`

如果未设置 `FNM_AI_MODEL`，`fnm-api` 会默认使用 `kimi-k2.6`。

## 自然语言示例

安装并切换：

```powershell
fnm-api "安装 Node 20 并马上使用"
fnm-api "install node 20 and use it"
```

只切换版本：

```powershell
fnm-api "切换到 Node 18"
fnm-api "use node 22, install it if missing"
```

查看当前状态：

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

## 交互模式和两个预设

运行：

```powershell
fnm-api
```

会进入交互模式：

```text
fnm ai is ready.
Presets:
  1. switch environment
  2. check current environment config
```

预设 `1`：切换环境。

它会根据当前目录的 `.node-version`、`.nvmrc` 或默认版本执行切换，等价于用自然语言说“先切换环境”。

预设 `2`：检查当前环境配置。

它会输出 fnm 目录、Node 镜像、架构、版本文件策略、corepack、resolve engines、`FNM_MULTISHELL_PATH`、multishell 是否在 `PATH` 中，以及当前 Node 版本。

你也可以直接说：

```powershell
fnm-api "先切换环境"
fnm-api "检查当前环境配置"
```

## 环境变量如何被加载

fnm 切换 Node 版本依赖这些运行时环境：

- `FNM_MULTISHELL_PATH`
- `FNM_DIR`
- `FNM_VERSION_FILE_STRATEGY`
- `FNM_LOGLEVEL`
- `FNM_NODE_DIST_MIRROR`
- `FNM_COREPACK_ENABLED`
- `FNM_RESOLVE_ENGINES`
- `FNM_ARCH`
- `PATH`

传统 fnm 需要你在 shell profile 中执行：

```powershell
fnm env --use-on-cd --shell powershell | Out-String | Invoke-Expression
```

`@fnm/ai` 的 npm wrapper 做了一层改进：

1. 启动时读取 `%APPDATA%\fnm-ai\config.json`。
2. 把 `baseUrl`、`apiKey`、`model` 转成 `FNM_AI_BASE_URL`、`FNM_AI_API_KEY`、`FNM_AI_MODEL`。
3. 如果缺少 `FNM_MULTISHELL_PATH`，自动执行 `fnm env --json`。
4. 合并 `fnm env --json` 返回的 `FNM_*`。
5. 把 multishell 路径追加到子进程 `PATH` 前面。
6. 再启动真正的 `fnm ai`。

这意味着即使你的当前 PowerShell 没有提前加载 `fnm env`，下面的命令也能在 `fnm-api` 子进程里看到完整环境：

```powershell
fnm-api "检查当前环境配置"
```

## 重要限制：不能永久修改父 shell

Windows、PowerShell 和 Node.js 子进程都有同一个限制：子进程不能永久修改已经运行中的父终端环境变量。

因此：

- `fnm-api` 可以为自己的子进程自动补齐 `FNM_*` 和 `PATH`。
- `fnm-api` 可以更新 fnm 的 multishell 链接，让 fnm 知道当前应该使用哪个 Node 版本。
- 但 `fnm-api` 退出后，父 PowerShell 里已经存在的 `PATH` 不一定会自动刷新。

如果你希望在 `fnm-api "切换到 Node 20"` 之后，当前 PowerShell 里的 `node -v` 也马上稳定使用该版本，请把下面这行加入 PowerShell profile：

```powershell
fnm env --use-on-cd --shell powershell | Out-String | Invoke-Expression
```

创建 profile 文件：

```powershell
if (-not (Test-Path $profile)) { New-Item $profile -Force }
```

打开 profile 文件：

```powershell
Invoke-Item $profile
```

写入后重启 PowerShell。

## 安全模型

AI provider 只负责把自然语言翻译成固定 JSON 动作。工具只接受以下动作：

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

模型不能返回任意 shell 命令并让工具执行。

当前本地解析器能识别常见中英文请求；当本地解析器不理解请求，并且已经配置了 `FNM_AI_BASE_URL` 与 `FNM_AI_API_KEY`，才会调用 OpenAI-compatible chat/completions 接口。

## 底层 fnm 命令

自然语言请求最终会映射到 fnm 的稳定能力：

- `fnm install`
- `fnm use`
- `fnm list`
- `fnm list-remote`
- `fnm current`
- `fnm default`
- `fnm uninstall`
- `fnm env`

完整底层命令文档仍可参考：

- [命令列表](./docs/commands.md)
- [AI 使用说明](./docs/ai.md)
- [fnm 配置项](./docs/configuration.md)

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
