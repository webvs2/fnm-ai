# @fnm/ai

`@fnm/ai` is a Windows-first npm package for managing Node.js versions with natural-language requests instead of remembering fnm commands.

## Install

```powershell
npm install -g @fnm/ai
```

This installs two npm executables:

```powershell
fnm-ai
fnm-api
```

Use `fnm-api` for day-to-day usage. npm package specifiers such as `@fnm/api` are not reliable shell commands, so the portable executable name is `fnm-api`.

## Configure Kimi

The default model is Kimi `kimi-k2.6`. Configure the OpenAI-compatible base URL and API key once:

```powershell
fnm-api config set --base-url https://api.moonshot.ai/v1 --api-key <your-kimi-api-key>
```

You can inspect the saved config without printing the full key:

```powershell
fnm-api config get
```

The config is stored at:

```powershell
fnm-api config path
```

On Windows this is usually `%APPDATA%\fnm-ai\config.json`.

You can override the saved values with environment variables:

```powershell
$env:FNM_AI_BASE_URL = "https://api.moonshot.ai/v1"
$env:FNM_AI_API_KEY = "<your-kimi-api-key>"
$env:FNM_AI_MODEL = "kimi-k2.6"
```

## Use

Run a single request:

```powershell
fnm-api "帮我切换到 Node 20"
fnm-api "安装最新 LTS，并切换过去"
fnm-api "检查当前环境配置"
fnm-api "列出已安装版本"
```

Or start the interactive prompt:

```powershell
fnm-api
```

The prompt has two built-in presets:

```text
1. switch environment
2. check current environment config
```

## Environment Variables

When launched through npm, `fnm-api` automatically runs `fnm env --json` if `FNM_MULTISHELL_PATH` is missing. It merges the returned `FNM_*` variables into the child process and prepends the multishell path to `PATH`.

This fixes the common npm-package case where `fnm-api "检查当前环境配置"` starts from a shell that has not loaded `fnm env`.

One limitation remains: a child process cannot permanently mutate the parent terminal's environment. If you want `node`, `npm`, and `pnpm` in the current PowerShell window to keep using the selected version after `fnm-api` exits, add fnm to your PowerShell profile:

```powershell
fnm env --use-on-cd --shell powershell | Out-String | Invoke-Expression
```

Without that profile setup, `fnm-api` can initialize and use the environment for its own process, and it can update fnm's multishell link, but the already-running parent shell may not refresh its own `PATH`.

## Safety

AI output is translated into a fixed set of fnm actions only:

```text
install, use, list_local, list_remote, current, default, uninstall, help, exit
```

The model cannot return arbitrary shell commands for execution.
