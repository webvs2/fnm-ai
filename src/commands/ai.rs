use super::command::Command;
use super::{
    current::Current,
    default::Default,
    install::Install,
    ls_local::LsLocal,
    ls_remote::{LsRemote, SortingMethod},
    r#use::Use,
    uninstall::Uninstall,
};
use crate::config::FnmConfig;
use crate::current_version::current_version;
use crate::lts::LtsType;
use crate::progress::ProgressConfig;
use crate::user_version::UserVersion;
use crate::user_version_reader::UserVersionReader;
use crate::version::Version;
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, IsTerminal, Write};
use std::str::FromStr;
use std::time::Duration;
use thiserror::Error;

const DEFAULT_AI_MODEL: &str = "kimi-k2.6";
const AI_SYSTEM_PROMPT: &str = r#"Translate natural-language Node.js version-management requests into one safe fnm action.
Return strict JSON only, with this schema:
{"action":"help|exit|switch_environment|check_environment|install|use|list_local|list_remote|current|default|uninstall","version":string|null,"install_if_missing":boolean,"use_after":boolean,"lts":boolean,"latest":boolean}
Rules:
- Never return shell commands or arbitrary text.
- Use only the allowed action names.
- Use check_environment for PATH, environment-variable, fnm config, or current setup checks.
- Use switch_environment when the user asks to apply the current project/default Node version.
- version can be null, a Node version like "20" or "20.14.0", "latest", "lts", "lts/iron", "system", or "default"."#;

#[derive(clap::Parser, Debug)]
#[clap(trailing_var_arg = true)]
pub struct Ai {
    /// A natural language request. Omit it to start an interactive conversation.
    #[clap(value_name = "MESSAGE", allow_hyphen_values = true)]
    pub message: Vec<String>,
}

impl Command for Ai {
    type Error = Error;

    fn apply(self, config: &FnmConfig) -> Result<(), Self::Error> {
        if self.message.is_empty() {
            run_conversation(config)
        } else {
            handle_message(&self.message.join(" "), config).map(|_| ())
        }
    }
}

fn run_conversation(config: &FnmConfig) -> Result<(), Error> {
    let stdin = io::stdin();

    if stdin.is_terminal() {
        print_welcome();
        let mut line = String::new();

        loop {
            print!("fnm ai> ");
            io::stdout().flush()?;
            line.clear();

            if stdin.read_line(&mut line)? == 0 {
                return Ok(());
            }

            if matches!(handle_message(&line, config)?, Flow::Exit) {
                return Ok(());
            }
        }
    }

    for line in stdin.lock().lines() {
        if matches!(handle_message(&line?, config)?, Flow::Exit) {
            break;
        }
    }

    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
enum Flow {
    Continue,
    Exit,
}

fn handle_message(message: &str, config: &FnmConfig) -> Result<Flow, Error> {
    match parse_intent(message).or_else(|err| match err {
        Error::UnknownRequest { .. } => parse_intent_with_provider(message),
        _ => Err(err),
    })? {
        Intent::Exit => Ok(Flow::Exit),
        Intent::Help => {
            print_help();
            Ok(Flow::Continue)
        }
        Intent::Preset(preset) => {
            run_preset(preset, config)?;
            Ok(Flow::Continue)
        }
        Intent::Install { version, use_after } => {
            Install {
                version,
                lts: false,
                latest: false,
                progress: ProgressConfig::Never,
                r#use: use_after,
            }
            .apply(config)
            .map_err(|source| Error::Install { source })?;
            Ok(Flow::Continue)
        }
        Intent::Use {
            version,
            install_if_missing,
        } => {
            Use {
                version: version.map(UserVersionReader::Direct),
                install_if_missing,
                silent_if_unchanged: false,
                info_to_stderr: false,
            }
            .apply(config)
            .map_err(|source| Error::Use { source })?;
            Ok(Flow::Continue)
        }
        Intent::ListLocal => {
            LsLocal {}
                .apply(config)
                .map_err(|source| Error::ListLocal { source })?;
            Ok(Flow::Continue)
        }
        Intent::ListRemote { lts, latest } => {
            LsRemote {
                filter: None,
                lts: lts.then_some(None),
                sort: SortingMethod::Ascending,
                latest,
            }
            .apply(config)
            .map_err(|source| Error::ListRemote { source })?;
            Ok(Flow::Continue)
        }
        Intent::Current => {
            Current {}
                .apply(config)
                .map_err(|source| Error::Current { source })?;
            Ok(Flow::Continue)
        }
        Intent::Default { version } => {
            Default::new(version)
                .apply(config)
                .map_err(|source| Error::Default { source })?;
            Ok(Flow::Continue)
        }
        Intent::Uninstall { version } => {
            Uninstall::new(Some(version))
                .apply(config)
                .map_err(|source| Error::Uninstall { source })?;
            Ok(Flow::Continue)
        }
    }
}

fn print_welcome() {
    println!("fnm ai is ready.");
    println!("Presets:");
    println!("  1. switch environment");
    println!("  2. check current environment config");
    println!("Ask to install, use, list, uninstall, or show current. Type exit to quit.");
}

fn print_help() {
    println!("Try requests like:");
    println!("  1");
    println!("  2");
    println!("  install node 20");
    println!("  install latest lts and use it");
    println!("  switch to node 18");
    println!("  use node 20, install it if missing");
    println!("  list installed versions");
    println!("  what version am I using?");
    println!("  set default to 20");
    println!("  uninstall node 16");
}

fn run_preset(preset: Preset, config: &FnmConfig) -> Result<(), Error> {
    match preset {
        Preset::SwitchEnvironment => {
            let version = std::env::current_dir().ok().and_then(|current_dir| {
                UserVersionReader::Path(current_dir).into_user_version(config)
            });

            if let Some(version) = &version {
                println!("Switching environment to {version}.");
            } else {
                println!("Switching environment using the current project/default version.");
            }

            Use {
                version: version.map(UserVersionReader::Direct),
                install_if_missing: false,
                silent_if_unchanged: false,
                info_to_stderr: false,
            }
            .apply(config)
            .map_err(|source| Error::Use { source })?;
        }
        Preset::CheckEnvironment => print_environment_config(config),
    }

    Ok(())
}

fn print_environment_config(config: &FnmConfig) {
    println!(
        "fnm directory: {}",
        config.base_dir_with_default().display()
    );
    println!("node mirror: {}", config.node_dist_mirror);
    println!("architecture: {}", config.arch);
    println!("version file strategy: {}", config.version_file_strategy());
    println!("corepack enabled: {}", config.corepack_enabled());
    println!("resolve engines: {}", config.resolve_engines());

    match config.multishell_path() {
        Some(path) => {
            println!("multishell path: {}", path.display());
            println!("multishell exists: {}", path.exists());
            println!("multishell on PATH: {}", multishell_path_is_on_path(path));
        }
        None => {
            println!("multishell path: missing");
            println!("environment sourced: false");
        }
    }

    match current_version(config) {
        Ok(Some(version)) => println!("current version: {version}"),
        Ok(None) => println!("current version: none"),
        Err(err) => println!("current version: unavailable ({err})"),
    }
}

fn multishell_path_is_on_path(multishell_path: &std::path::Path) -> bool {
    let Some(path_var) = std::env::var_os("PATH") else {
        return false;
    };

    let bin_path = if cfg!(unix) {
        multishell_path.join("bin")
    } else {
        multishell_path.to_path_buf()
    };

    let fixed_path = bin_path
        .to_str()
        .and_then(crate::shell::maybe_fix_windows_path);
    let fixed_path = fixed_path.as_deref();

    std::env::split_paths(&path_var).any(|path| bin_path == path || fixed_path == path.to_str())
}

#[derive(Debug)]
enum Intent {
    Help,
    Exit,
    Preset(Preset),
    Install {
        version: Option<UserVersion>,
        use_after: bool,
    },
    Use {
        version: Option<UserVersion>,
        install_if_missing: bool,
    },
    ListLocal,
    ListRemote {
        lts: bool,
        latest: bool,
    },
    Current,
    Default {
        version: Option<UserVersion>,
    },
    Uninstall {
        version: UserVersion,
    },
}

#[derive(Debug)]
enum Preset {
    SwitchEnvironment,
    CheckEnvironment,
}

fn parse_intent(message: &str) -> Result<Intent, Error> {
    let request = Request::new(message);

    if request.is_empty() {
        return Err(Error::EmptyRequest);
    }

    if request.wants_exit() {
        return Ok(Intent::Exit);
    }

    if request.wants_help() {
        return Ok(Intent::Help);
    }

    if let Some(preset) = request.preset() {
        return Ok(Intent::Preset(preset));
    }

    let version = request.requested_version();

    if request.wants_uninstall() {
        let version = version.ok_or(Error::MissingVersion {
            action: "uninstall",
        })?;
        return Ok(Intent::Uninstall { version });
    }

    if request.wants_default() && !request.wants_use() && !request.wants_install() {
        if request.wants_set() || version.is_some() {
            let version = version.ok_or(Error::MissingVersion {
                action: "set the default",
            })?;
            return Ok(Intent::Default {
                version: Some(version),
            });
        }

        return Ok(Intent::Default { version: None });
    }

    if request.wants_current() {
        return Ok(Intent::Current);
    }

    if request.wants_list() {
        if request.wants_remote() {
            return Ok(Intent::ListRemote {
                lts: request.wants_lts(),
                latest: request.wants_latest(),
            });
        }

        return Ok(Intent::ListLocal);
    }

    if request.wants_use() && request.wants_install_if_missing() {
        return Ok(Intent::Use {
            version,
            install_if_missing: true,
        });
    }

    if request.wants_install() {
        return Ok(Intent::Install {
            version,
            use_after: request.wants_use(),
        });
    }

    if request.wants_use() {
        let version = if request.wants_default() {
            Some(UserVersion::Full(Version::Alias("default".into())))
        } else {
            version
        };
        return Ok(Intent::Use {
            version,
            install_if_missing: request.wants_install_if_missing(),
        });
    }

    Err(Error::UnknownRequest {
        message: request.original.trim().to_string(),
    })
}

fn parse_intent_with_provider(message: &str) -> Result<Intent, Error> {
    let config = match AiProviderConfig::from_env()? {
        Some(config) => config,
        None => {
            return Err(Error::UnknownRequest {
                message: message.trim().to_string(),
            })
        }
    };

    request_provider_intent(message, &config)?.try_into_intent()
}

#[derive(Debug)]
struct AiProviderConfig {
    base_url: String,
    api_key: String,
    model: String,
}

impl AiProviderConfig {
    fn from_env() -> Result<Option<Self>, Error> {
        let base_url = optional_env("FNM_AI_BASE_URL");
        let api_key = optional_env("FNM_AI_API_KEY");
        let model = optional_env("FNM_AI_MODEL").unwrap_or_else(|| DEFAULT_AI_MODEL.to_string());

        match (base_url, api_key) {
            (Some(base_url), Some(api_key)) => Ok(Some(Self {
                base_url,
                api_key,
                model,
            })),
            (None, None) => Ok(None),
            _ => Err(Error::MissingAiProviderConfig),
        }
    }
}

fn optional_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest<'a> {
    model: &'a str,
    messages: [ChatMessage<'a>; 2],
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct ChatMessage<'a> {
    role: &'static str,
    content: &'a str,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct ChatChoiceMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct ProviderIntent {
    action: String,
    version: Option<String>,
    #[serde(default)]
    install_if_missing: bool,
    #[serde(default)]
    use_after: bool,
    #[serde(default)]
    lts: bool,
    #[serde(default)]
    latest: bool,
}

fn request_provider_intent(
    message: &str,
    config: &AiProviderConfig,
) -> Result<ProviderIntent, Error> {
    let url = chat_completions_url(&config.base_url);
    let request = ChatCompletionRequest {
        model: &config.model,
        messages: [
            ChatMessage {
                role: "system",
                content: AI_SYSTEM_PROMPT,
            },
            ChatMessage {
                role: "user",
                content: message,
            },
        ],
        temperature: 0.0,
    };

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|source| Error::AiProviderRequest { source })?;
    let response = client
        .post(url)
        .bearer_auth(&config.api_key)
        .header(
            reqwest::header::USER_AGENT,
            concat!("@fnm/ai ", env!("CARGO_PKG_VERSION")),
        )
        .json(&request)
        .send()
        .map_err(|source| Error::AiProviderRequest { source })?;

    let status = response.status();
    let body = response
        .text()
        .map_err(|source| Error::AiProviderRequest { source })?;

    if !status.is_success() {
        return Err(Error::AiProviderStatus {
            status,
            body: truncate_for_error(&body),
        });
    }

    let response: ChatCompletionResponse =
        serde_json::from_str(&body).map_err(|source| Error::AiProviderResponse { source })?;
    let content = response
        .choices
        .into_iter()
        .next()
        .map(|choice| choice.message.content)
        .ok_or(Error::AiProviderEmptyResponse)?;

    let json = extract_json_object(&content).ok_or_else(|| Error::AiProviderInvalidJson {
        content: truncate_for_error(&content),
    })?;

    serde_json::from_str(json).map_err(|source| Error::AiProviderIntent {
        source,
        content: truncate_for_error(&content),
    })
}

fn chat_completions_url(base_url: &str) -> String {
    let base_url = base_url.trim_end_matches('/');

    if base_url.ends_with("/chat/completions") {
        base_url.to_string()
    } else {
        format!("{base_url}/chat/completions")
    }
}

fn truncate_for_error(value: &str) -> String {
    const MAX_LEN: usize = 500;

    if value.len() <= MAX_LEN {
        return value.to_string();
    }

    let mut end = MAX_LEN;
    while !value.is_char_boundary(end) {
        end -= 1;
    }

    format!("{}...", &value[..end])
}

fn extract_json_object(content: &str) -> Option<&str> {
    let trimmed = content.trim();

    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        return Some(trimmed);
    }

    let start = trimmed.find('{')?;
    let end = trimmed.rfind('}')?;
    (start < end).then_some(&trimmed[start..=end])
}

impl ProviderIntent {
    fn try_into_intent(self) -> Result<Intent, Error> {
        let action = self.action.trim().to_ascii_lowercase();

        match action.as_str() {
            "help" => Ok(Intent::Help),
            "exit" => Ok(Intent::Exit),
            "switch_environment" => Ok(Intent::Preset(Preset::SwitchEnvironment)),
            "check_environment" => Ok(Intent::Preset(Preset::CheckEnvironment)),
            "install" => Ok(Intent::Install {
                version: self.version_for_action("install")?,
                use_after: self.use_after,
            }),
            "use" => Ok(Intent::Use {
                version: self.version_for_action("use")?,
                install_if_missing: self.install_if_missing,
            }),
            "list_local" => Ok(Intent::ListLocal),
            "list_remote" => Ok(Intent::ListRemote {
                lts: self.lts || self.version_is_lts(),
                latest: self.latest || self.version_is_latest(),
            }),
            "current" => Ok(Intent::Current),
            "default" => {
                let version = self.version_for_action("set the default")?;
                if self.version_is_empty() {
                    Ok(Intent::Default { version: None })
                } else {
                    let version = version.ok_or(Error::MissingVersion {
                        action: "set the default",
                    })?;
                    Ok(Intent::Default {
                        version: Some(version),
                    })
                }
            }
            "uninstall" => {
                let version =
                    self.version_for_action("uninstall")?
                        .ok_or(Error::MissingVersion {
                            action: "uninstall",
                        })?;
                Ok(Intent::Uninstall { version })
            }
            _ => Err(Error::AiProviderUnsupportedAction {
                action: self.action,
            }),
        }
    }

    fn version_for_action(&self, action: &'static str) -> Result<Option<UserVersion>, Error> {
        if self.lts && self.version_is_empty() {
            return Ok(Some(UserVersion::Full(Version::Lts(LtsType::Latest))));
        }

        if self.latest && self.version_is_empty() {
            return Ok(Some(UserVersion::Full(Version::Latest)));
        }

        let Some(version) = self
            .version
            .as_deref()
            .map(str::trim)
            .filter(|version| !version.is_empty())
        else {
            return Ok(None);
        };

        parse_provider_version(version).map(Some).map_err(|source| {
            Error::AiProviderInvalidVersion {
                action,
                source,
                version: version.to_string(),
            }
        })
    }

    fn version_is_empty(&self) -> bool {
        self.version
            .as_deref()
            .is_none_or(|version| version.trim().is_empty())
    }

    fn version_is_lts(&self) -> bool {
        self.version
            .as_deref()
            .is_some_and(|version| normalize_provider_version(version).starts_with("lts"))
    }

    fn version_is_latest(&self) -> bool {
        self.version
            .as_deref()
            .is_some_and(|version| normalize_provider_version(version) == "latest")
    }
}

fn parse_provider_version(version: &str) -> Result<UserVersion, node_semver::SemverError> {
    let normalized = normalize_provider_version(version);

    match normalized.as_str() {
        "lts" | "lts/latest" | "lts-latest" | "latest-lts" | "lts/*" => {
            Ok(UserVersion::Full(Version::Lts(LtsType::Latest)))
        }
        _ => UserVersion::from_str(&normalized),
    }
}

fn normalize_provider_version(version: &str) -> String {
    version.trim().trim_matches('"').to_ascii_lowercase()
}

struct Request<'a> {
    original: &'a str,
    lower: String,
    tokens: Vec<String>,
}

impl<'a> Request<'a> {
    fn new(original: &'a str) -> Self {
        let lower = original.to_lowercase();
        let tokens = tokenize(&lower);
        Self {
            original,
            lower,
            tokens,
        }
    }

    fn is_empty(&self) -> bool {
        self.original.trim().is_empty()
    }

    fn has_word(&self, words: &[&str]) -> bool {
        self.tokens
            .iter()
            .any(|token| words.iter().any(|word| token == word))
    }

    fn has_text(&self, phrases: &[&str]) -> bool {
        phrases.iter().any(|phrase| self.lower.contains(phrase))
    }

    fn wants_exit(&self) -> bool {
        self.has_word(&["exit", "quit", "bye"]) || self.has_text(&["退出", "再见"])
    }

    fn wants_help(&self) -> bool {
        self.has_word(&["help", "?"]) || self.has_text(&["帮助", "怎么用"])
    }

    fn preset(&self) -> Option<Preset> {
        if self.has_word(&["1"]) || self.has_text(&["切换环境", "先切换", "switch environment"])
        {
            Some(Preset::SwitchEnvironment)
        } else if self.has_word(&["2"])
            || self.has_text(&[
                "检查当前环境配置",
                "检查环境",
                "当前环境配置",
                "check current environment config",
                "check environment",
            ])
        {
            Some(Preset::CheckEnvironment)
        } else {
            None
        }
    }

    fn wants_install(&self) -> bool {
        self.has_word(&["install", "download", "add"]) || self.has_text(&["安装", "下载"])
    }

    fn wants_use(&self) -> bool {
        self.has_word(&["use", "switch", "change", "activate", "select"])
            || self.has_text(&["使用", "切换", "换到", "切到", "启用"])
    }

    fn wants_uninstall(&self) -> bool {
        self.has_word(&["uninstall", "remove", "delete"])
            || self.has_text(&["卸载", "删除", "移除"])
    }

    fn wants_default(&self) -> bool {
        self.has_word(&["default"]) || self.has_text(&["默认"])
    }

    fn wants_set(&self) -> bool {
        self.has_word(&["set", "make", "mark"]) || self.has_text(&["设置", "设为", "改成"])
    }

    fn wants_current(&self) -> bool {
        self.has_word(&["current", "active"])
            || self.has_text(&[
                "what version",
                "which version",
                "currently using",
                "当前",
                "现在用",
                "正在用",
            ])
    }

    fn wants_list(&self) -> bool {
        self.has_word(&["list", "ls"])
            || self.has_text(&["列出", "有哪些"])
            || (self.has_word(&["show"]) && !self.wants_current())
    }

    fn wants_remote(&self) -> bool {
        self.has_word(&["remote", "available", "downloadable"])
            || self.has_text(&["远程", "可安装", "可用"])
    }

    fn wants_latest(&self) -> bool {
        self.has_word(&["latest", "newest", "stable"]) || self.has_text(&["最新"])
    }

    fn wants_lts(&self) -> bool {
        self.has_word(&["lts"]) || self.has_text(&["长期支持"])
    }

    fn wants_system(&self) -> bool {
        self.has_word(&["system"]) || self.has_text(&["系统"])
    }

    fn wants_install_if_missing(&self) -> bool {
        self.has_text(&[
            "if missing",
            "if needed",
            "when missing",
            "install it if",
            "没有就安装",
            "不存在就安装",
            "缺失",
        ])
    }

    fn requested_version(&self) -> Option<UserVersion> {
        if self.wants_lts() {
            if let Some(version) = self
                .tokens
                .iter()
                .find_map(|token| parse_lts_version_token(token))
            {
                return Some(version);
            }

            return Some(UserVersion::Full(Version::Lts(LtsType::Latest)));
        }

        if self.wants_latest() {
            return Some(UserVersion::Full(Version::Latest));
        }

        if self.wants_system() {
            return Some(UserVersion::Full(Version::Bypassed));
        }

        self.tokens
            .iter()
            .find_map(|token| parse_versionish_token(token))
            .or_else(|| parse_inline_numeric_version(&self.lower))
    }
}

fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        if is_token_separator(ch) {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
        } else {
            current.push(ch);
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn is_token_separator(ch: char) -> bool {
    ch.is_whitespace()
        || matches!(
            ch,
            ',' | ';'
                | ':'
                | '?'
                | '!'
                | '"'
                | '\''
                | '('
                | ')'
                | '['
                | ']'
                | '{'
                | '}'
                | '，'
                | '。'
                | '？'
                | '！'
                | '、'
                | '：'
                | '；'
        )
}

fn parse_lts_version_token(token: &str) -> Option<UserVersion> {
    let token = trim_version_token(token);
    if token == "lts" {
        return Some(UserVersion::Full(Version::Lts(LtsType::Latest)));
    }

    if token.starts_with("lts/") || token.starts_with("lts-") {
        return UserVersion::from_str(token).ok();
    }

    None
}

fn parse_versionish_token(token: &str) -> Option<UserVersion> {
    let token = trim_version_token(token);
    let token = strip_node_prefix(token).unwrap_or(token);

    if token == "latest" || token == "newest" || token == "stable" {
        return Some(UserVersion::Full(Version::Latest));
    }

    if token == "system" {
        return Some(UserVersion::Full(Version::Bypassed));
    }

    if let Some(lts) = parse_lts_version_token(token) {
        return Some(lts);
    }

    if looks_like_numeric_version(token) {
        return UserVersion::from_str(token).ok();
    }

    None
}

fn trim_version_token(token: &str) -> &str {
    token.trim_matches(|ch: char| {
        !(ch.is_ascii_alphanumeric() || matches!(ch, '.' | '/' | '-' | '*' | '_'))
    })
}

fn strip_node_prefix(token: &str) -> Option<&str> {
    for prefix in ["node.js", "nodejs", "node"] {
        let Some(rest) = token.strip_prefix(prefix) else {
            continue;
        };
        let rest = rest.trim_start_matches(['-', '_']);
        if looks_like_numeric_version(rest) {
            return Some(rest);
        }
    }

    None
}

fn looks_like_numeric_version(token: &str) -> bool {
    let plain = token.strip_prefix('v').unwrap_or(token);

    plain.chars().next().is_some_and(|ch| ch.is_ascii_digit())
        && plain
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '*'))
}

fn parse_inline_numeric_version(text: &str) -> Option<UserVersion> {
    let bytes = text.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        let starts_with_v = bytes[index].eq_ignore_ascii_case(&b'v')
            && bytes.get(index + 1).is_some_and(u8::is_ascii_digit);
        let starts_with_digit = bytes[index].is_ascii_digit();

        if starts_with_v || starts_with_digit {
            let start = index;
            if starts_with_v {
                index += 1;
            }

            while index < bytes.len()
                && (bytes[index].is_ascii_digit() || matches!(bytes[index], b'.'))
            {
                index += 1;
            }

            if let Some(version) = parse_versionish_token(&text[start..index]) {
                return Some(version);
            }
        }

        index += 1;
    }

    None
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Please tell fnm ai what you want to do.")]
    EmptyRequest,
    #[error("I couldn't understand '{message}'. Try `fnm ai help` for examples.")]
    UnknownRequest { message: String },
    #[error("I understood the action, but not the Node.js version to {action}.")]
    MissingVersion { action: &'static str },
    #[error("FNM_AI_BASE_URL and FNM_AI_API_KEY must be configured together.")]
    MissingAiProviderConfig,
    #[error("AI provider request failed: {source}")]
    AiProviderRequest { source: reqwest::Error },
    #[error("AI provider returned HTTP {status}: {body}")]
    AiProviderStatus {
        status: reqwest::StatusCode,
        body: String,
    },
    #[error("AI provider returned an invalid chat/completions response: {source}")]
    AiProviderResponse { source: serde_json::Error },
    #[error("AI provider returned no choices.")]
    AiProviderEmptyResponse,
    #[error("AI provider did not return JSON: {content}")]
    AiProviderInvalidJson { content: String },
    #[error("AI provider returned invalid action JSON: {source}. Content: {content}")]
    AiProviderIntent {
        source: serde_json::Error,
        content: String,
    },
    #[error("AI provider returned unsupported action '{action}'.")]
    AiProviderUnsupportedAction { action: String },
    #[error("AI provider returned invalid Node.js version '{version}' for {action}: {source}")]
    AiProviderInvalidVersion {
        action: &'static str,
        version: String,
        source: node_semver::SemverError,
    },
    #[error(transparent)]
    Io {
        #[from]
        source: std::io::Error,
    },
    #[error(transparent)]
    Install { source: super::install::Error },
    #[error(transparent)]
    Use { source: super::r#use::Error },
    #[error(transparent)]
    ListLocal { source: super::ls_local::Error },
    #[error(transparent)]
    ListRemote { source: super::ls_remote::Error },
    #[error(transparent)]
    Current {
        source: crate::current_version::Error,
    },
    #[error(transparent)]
    Default { source: super::alias::Error },
    #[error(transparent)]
    Uninstall { source: super::uninstall::Error },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed_version(message: &str) -> Option<String> {
        match parse_intent(message).unwrap() {
            Intent::Install { version, .. }
            | Intent::Use { version, .. }
            | Intent::Default { version } => version.map(|version| version.to_string()),
            Intent::Uninstall { version } => Some(version.to_string()),
            Intent::Help
            | Intent::Exit
            | Intent::Preset(_)
            | Intent::ListLocal
            | Intent::ListRemote { .. }
            | Intent::Current => None,
        }
    }

    #[test]
    fn parses_install_and_use() {
        match parse_intent("please install Node v8.11.3 and use it").unwrap() {
            Intent::Install {
                version,
                use_after: true,
            } => assert_eq!(
                version.map(|version| version.to_string()),
                Some("v8.11.3".into())
            ),
            other => panic!("unexpected intent: {other:?}"),
        }
    }

    #[test]
    fn parses_use_with_install_if_missing() {
        match parse_intent("use node 20 and install it if missing").unwrap() {
            Intent::Use {
                version,
                install_if_missing: true,
            } => assert_eq!(
                version.map(|version| version.to_string()),
                Some("v20.x.x".into())
            ),
            other => panic!("unexpected intent: {other:?}"),
        }
    }

    #[test]
    fn parses_chinese_use_request() {
        assert_eq!(parsed_version("切换到 Node 18"), Some("v18.x.x".into()));
    }

    #[test]
    fn parses_chinese_inline_version() {
        assert_eq!(parsed_version("帮我安装v8.11.3"), Some("v8.11.3".into()));
    }

    #[test]
    fn parses_latest_lts_before_latest() {
        assert_eq!(
            parsed_version("install latest lts"),
            Some("lts-latest".into())
        );
    }

    #[test]
    fn parses_current_request() {
        assert!(matches!(
            parse_intent("what version am I using?").unwrap(),
            Intent::Current
        ));
    }

    #[test]
    fn parses_list_remote_lts() {
        assert!(matches!(
            parse_intent("list remote lts versions").unwrap(),
            Intent::ListRemote {
                lts: true,
                latest: false
            }
        ));
    }

    #[test]
    fn parses_switch_environment_preset() {
        assert!(matches!(
            parse_intent("1").unwrap(),
            Intent::Preset(Preset::SwitchEnvironment)
        ));
        assert!(matches!(
            parse_intent("先切换环境").unwrap(),
            Intent::Preset(Preset::SwitchEnvironment)
        ));
    }

    #[test]
    fn parses_check_environment_preset() {
        assert!(matches!(
            parse_intent("2").unwrap(),
            Intent::Preset(Preset::CheckEnvironment)
        ));
        assert!(matches!(
            parse_intent("检查当前环境配置").unwrap(),
            Intent::Preset(Preset::CheckEnvironment)
        ));
    }

    #[test]
    fn appends_chat_completions_to_provider_base_url() {
        assert_eq!(
            chat_completions_url("https://api.moonshot.ai/v1"),
            "https://api.moonshot.ai/v1/chat/completions"
        );
        assert_eq!(
            chat_completions_url("https://api.moonshot.ai/v1/chat/completions"),
            "https://api.moonshot.ai/v1/chat/completions"
        );
    }

    #[test]
    fn extracts_json_object_from_markdown_response() {
        assert_eq!(
            extract_json_object("```json\n{\"action\":\"current\"}\n```"),
            Some("{\"action\":\"current\"}")
        );
    }

    #[test]
    fn converts_provider_use_intent() {
        let intent = ProviderIntent {
            action: "use".into(),
            version: Some("20".into()),
            install_if_missing: true,
            use_after: false,
            lts: false,
            latest: false,
        }
        .try_into_intent()
        .unwrap();

        match intent {
            Intent::Use {
                version,
                install_if_missing: true,
            } => assert_eq!(
                version.map(|version| version.to_string()),
                Some("v20.x.x".into())
            ),
            other => panic!("unexpected intent: {other:?}"),
        }
    }

    #[test]
    fn converts_provider_lts_install_intent() {
        let intent = ProviderIntent {
            action: "install".into(),
            version: None,
            install_if_missing: false,
            use_after: true,
            lts: true,
            latest: false,
        }
        .try_into_intent()
        .unwrap();

        match intent {
            Intent::Install {
                version,
                use_after: true,
            } => assert_eq!(
                version.map(|version| version.to_string()),
                Some("lts-latest".into())
            ),
            other => panic!("unexpected intent: {other:?}"),
        }
    }
}
