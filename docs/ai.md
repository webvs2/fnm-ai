# fnm-ai

`fnm-ai` is a Windows-first npm wrapper around `fnm` for managing Node.js
versions with natural-language requests.

It installs two executable names:

```powershell
fnm-api
fnm-ai
```

Use `fnm-api` for normal npm-based usage. The lower-level Rust binary also
exposes the same feature as `fnm ai` and `fnm api`.

## Install

```powershell
npm install -g fnm-ai
```

The npm package currently targets Windows x64:

```json
{
  "os": ["win32"],
  "cpu": ["x64"]
}
```

## Configure the AI provider

Common requests are parsed locally first. If the local parser cannot understand
the request, `fnm` calls an OpenAI-compatible `chat/completions` provider.

The default model is Kimi `kimi-k2.6`. Save the provider config once:

```powershell
fnm-api config set --base-url https://api.moonshot.ai/v1 --api-key <your-kimi-api-key>
```

You can also set the model explicitly:

```powershell
fnm-api config set --base-url https://api.moonshot.ai/v1 --api-key <your-kimi-api-key> --model kimi-k2.6
```

Inspect the saved config without printing the full API key:

```powershell
fnm-api config get
```

Print the config file path:

```powershell
fnm-api config path
```

On Windows this is usually:

```text
%APPDATA%\fnm-ai\config.json
```

Environment variables override the saved config:

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
fnm-api "use node 22, install it if missing"
fnm-api "检查当前环境配置"
fnm-api "列出已安装版本"
```

Start the interactive prompt:

```powershell
fnm-api
```

The prompt has two built-in presets:

```text
1. switch environment
2. check current environment config
```

Preset `1` applies the current project or default Node version using
`.node-version`, `.nvmrc`, `package.json#engines.node`, or the `default` alias.

Preset `2` prints the fnm directory, Node mirror, architecture, version-file
strategy, corepack setting, resolve-engines setting, multishell path, PATH
status, and current Node version.

## Environment bootstrap

When launched through npm, `fnm-api` reads `%APPDATA%\fnm-ai\config.json` and
maps it to `FNM_AI_BASE_URL`, `FNM_AI_API_KEY`, and `FNM_AI_MODEL`.

If `FNM_MULTISHELL_PATH` is missing, `fnm-api` also runs:

```powershell
fnm env --json
```

It merges the returned `FNM_*` variables into the child process and prepends the
multishell path to `PATH`. This lets `fnm-api "检查当前环境配置"` work even when
the current PowerShell session has not loaded `fnm env`.

One limitation remains: a child process cannot permanently mutate the parent
terminal environment. If you want `node`, `npm`, and `pnpm` in the current
PowerShell window to keep using the selected version after `fnm-api` exits, add
fnm to your PowerShell profile:

```powershell
fnm env --use-on-cd --shell powershell | Out-String | Invoke-Expression
```

Without that profile setup, `fnm-api` can initialize and use the environment for
its own process and update fnm's multishell link, but the already-running parent
shell may not refresh its own `PATH`.

## Safety

AI output is translated into a fixed set of actions:

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

The model cannot return arbitrary shell commands for execution.
