---
"fnm-ai": minor
---

Initial Windows release of `fnm-ai`, with `fnm ai`/`fnm api` and npm `fnm-ai`/`fnm-api` entrypoints for natural-language Node.js version management.

`fnm-api` can save an OpenAI-compatible base URL and API key, defaults to Kimi `kimi-k2.6`, and bootstraps missing fnm environment variables for its child process by running `fnm env --json`.

Examples:

```sh-session
$ fnm-api config set --base-url https://api.moonshot.ai/v1 --api-key <your-kimi-api-key>
$ fnm-api "install node 20 and use it"
$ fnm-api "检查当前环境配置"
```
