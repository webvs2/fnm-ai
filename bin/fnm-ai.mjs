#!/usr/bin/env node

import { spawnSync } from "node:child_process"
import {
  existsSync,
  mkdirSync,
  readFileSync,
  writeFileSync,
} from "node:fs"
import { dirname, join, resolve } from "node:path"
import { fileURLToPath } from "node:url"

const here = dirname(fileURLToPath(import.meta.url))
const args = process.argv.slice(2)
const binaryName = process.platform === "win32" ? "fnm.exe" : "fnm"
const candidates = [
  resolve(here, binaryName),
  resolve(here, "..", binaryName),
  resolve(here, "..", "target", "release", binaryName),
  resolve(here, "..", "target", "debug", binaryName),
]

const isWindows = process.platform === "win32"
const pathParts = (process.env.PATH ?? "").split(isWindows ? ";" : ":")
for (const part of pathParts) {
  if (part) {
    candidates.push(join(part, binaryName))
  }
}

if (args[0] === "config") {
  handleConfigCommand(args.slice(1))
  process.exit(0)
}

const fnmPath = findFnmBinary()
if (!fnmPath) {
  console.error(
    `${process.argv[1]} could not find the fnm binary. Install fnm or publish @fnm/ai with a bundled release binary.`,
  )
  process.exit(1)
}

const env = buildChildEnv(fnmPath)
const result = spawnSync(fnmPath, ["ai", ...args], {
  stdio: "inherit",
  env,
})

if (result.error) {
  console.error(result.error.message)
  process.exit(1)
}

process.exit(result.status ?? 0)

function findFnmBinary() {
  for (const candidate of candidates) {
    if (existsSync(candidate)) {
      return candidate
    }

    const result = spawnSync(candidate, ["--version"], {
      stdio: "ignore",
    })
    if (!result.error) {
      return candidate
    }

    if (result.error.code !== "ENOENT") {
      console.error(result.error.message)
      process.exit(1)
    }
  }

  return undefined
}

function buildChildEnv(fnmPath) {
  const env = { ...process.env }
  const config = readConfig()

  setIfMissing(env, "FNM_AI_BASE_URL", config.baseUrl)
  setIfMissing(env, "FNM_AI_API_KEY", config.apiKey)
  setIfMissing(env, "FNM_AI_MODEL", config.model ?? "kimi-k2.6")

  ensureFnmEnvironment(fnmPath, env)
  return env
}

function ensureFnmEnvironment(fnmPath, env) {
  if (!env.FNM_MULTISHELL_PATH) {
    const result = spawnSync(fnmPath, ["env", "--json"], {
      encoding: "utf8",
      env,
    })

    if (result.error) {
      console.error(`fnm-api could not initialize fnm environment: ${result.error.message}`)
      return
    }

    if (result.status !== 0) {
      const details = (result.stderr || result.stdout || "").trim()
      console.error(
        `fnm-api could not initialize fnm environment${details ? `: ${details}` : "."}`,
      )
      return
    }

    try {
      const fnmEnv = JSON.parse(result.stdout)
      for (const [key, value] of Object.entries(fnmEnv)) {
        setIfMissing(env, key, value)
      }
    } catch (error) {
      console.error(`fnm-api could not parse fnm env output: ${error.message}`)
      return
    }
  }

  if (env.FNM_MULTISHELL_PATH) {
    prependPath(env, process.platform === "win32" ? env.FNM_MULTISHELL_PATH : join(env.FNM_MULTISHELL_PATH, "bin"))
  }
}

function setIfMissing(env, key, value) {
  if (env[key] || typeof value !== "string" || value.trim() === "") {
    return
  }

  env[key] = value
}

function prependPath(env, entry) {
  const key = Object.keys(env).find((name) => name.toLowerCase() === "path") ?? "PATH"
  const delimiter = process.platform === "win32" ? ";" : ":"
  const existing = env[key] ?? ""
  const parts = existing.split(delimiter).filter(Boolean)
  const normalizedEntry = normalizePathForCompare(entry)
  const alreadyPresent = parts.some(
    (part) => normalizePathForCompare(part) === normalizedEntry,
  )

  if (!alreadyPresent) {
    env[key] = [entry, ...parts].join(delimiter)
  }
}

function normalizePathForCompare(value) {
  const normalized = value.replace(/[\\/]+$/, "")
  return process.platform === "win32" ? normalized.toLowerCase() : normalized
}

function handleConfigCommand(args) {
  const command = args[0]
  if (!command || command === "help" || command === "--help" || command === "-h") {
    printConfigHelp()
    return
  }

  if (command === "path") {
    console.log(configPath())
    return
  }

  if (command === "get") {
    const config = readConfig()
    console.log(
      JSON.stringify(
        {
          baseUrl: config.baseUrl ?? null,
          apiKey: maskSecret(config.apiKey),
          model: config.model ?? null,
          path: configPath(),
        },
        null,
        2,
      ),
    )
    return
  }

  if (command === "set") {
    const parsed = parseConfigSetArgs(args.slice(1))
    const current = readConfig()
    const next = {
      ...current,
      ...parsed,
    }

    if (!next.baseUrl || !next.apiKey) {
      console.error("config set requires both --base-url and --api-key.")
      process.exit(1)
    }

    writeConfig(next)
    console.log(`Saved @fnm/ai config to ${configPath()}`)
    return
  }

  console.error(`Unknown config command: ${command}`)
  printConfigHelp()
  process.exit(1)
}

function parseConfigSetArgs(args) {
  const parsed = {}

  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index]

    if (arg === "--base-url" || arg === "--baseurl") {
      parsed.baseUrl = takeValue(args, (index += 1), arg)
    } else if (arg === "--api-key" || arg === "--key") {
      parsed.apiKey = takeValue(args, (index += 1), arg)
    } else if (arg === "--model") {
      parsed.model = takeValue(args, (index += 1), arg)
    } else {
      console.error(`Unknown config option: ${arg}`)
      process.exit(1)
    }
  }

  return parsed
}

function takeValue(args, index, flag) {
  const value = args[index]
  if (!value || value.startsWith("--")) {
    console.error(`${flag} requires a value.`)
    process.exit(1)
  }

  return value
}

function readConfig() {
  try {
    return JSON.parse(readFileSync(configPath(), "utf8"))
  } catch (error) {
    if (error.code === "ENOENT") {
      return {}
    }

    console.error(`Could not read @fnm/ai config: ${error.message}`)
    return {}
  }
}

function writeConfig(config) {
  mkdirSync(configDir(), { recursive: true })
  writeFileSync(configPath(), `${JSON.stringify(config, null, 2)}\n`, {
    mode: 0o600,
  })
}

function configDir() {
  const root =
    process.env.APPDATA ??
    (process.env.USERPROFILE
      ? join(process.env.USERPROFILE, "AppData", "Roaming")
      : process.cwd())
  return join(root, "fnm-ai")
}

function configPath() {
  return join(configDir(), "config.json")
}

function maskSecret(secret) {
  if (!secret) {
    return null
  }

  if (secret.length <= 8) {
    return "********"
  }

  return `${secret.slice(0, 4)}...${secret.slice(-4)}`
}

function printConfigHelp() {
  console.log(`Usage:
  fnm-api config set --base-url https://api.moonshot.ai/v1 --api-key <key> [--model kimi-k2.6]
  fnm-api config get
  fnm-api config path

Environment variables:
  FNM_AI_BASE_URL overrides the configured baseUrl
  FNM_AI_API_KEY overrides the configured apiKey
  FNM_AI_MODEL overrides the configured model

Default model:
  kimi-k2.6`)
}
