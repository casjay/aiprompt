// Copyright (c) 2026 casapps
// Licensed under the MIT License. See LICENSE.md in the project root for license information.

mod session;
mod wrappers;
mod config;
mod normalization;

use clap::{Parser, Subcommand, CommandFactory, ValueEnum};
use clap_complete::{generate, Shell as ClapShell};
use clap_complete_nushell::Nushell;
use session::{Session, Role};
use wrappers::{Provider, ClaudeWrapper, CopilotWrapper};
use config::Config;
use anyhow::{Result, bail};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use uuid::Uuid;
use std::io::{self, IsTerminal, Write};
use std::env;
use tokio::process::Command;

shadow_rs::shadow!(build);

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum ColorOption {
    /// Automatically detect whether to use colors
    Auto,
    /// Always use colors
    Always,
    /// Never use colors
    Never,
}

#[derive(Parser)]
#[command(name = "aiprompt")]
#[command(version = build::VERSION, about = "Unified AI CLI Wrapper with Context Persistence")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Print version information including build date and commit ID
    #[arg(short = 'V', long)]
    version: bool,

    /// Shell integration options
    #[command(flatten)]
    shell_options: Option<ShellOptions>,

    /// Control colored output (auto, always, never)
    #[arg(long, default_value_t = ColorOption::Auto, value_enum)]
    color: ColorOption,

    /// Enable debug logging and output
    #[arg(long)]
    debug: bool,
}

#[derive(clap::Args)]
struct ShellOptions {
    #[command(subcommand)]
    shell_command: ShellCommands,
}

#[derive(Subcommand)]
enum ShellCommands {
    /// Shell integration
    Shell {
        #[command(subcommand)]
        action: ShellAction,
    },
}

#[derive(Subcommand)]
enum ShellAction {
    /// Print environment variable exports
    Env {
        /// Optional shell name (bash, zsh, fish, nu, powershell)
        shell: Option<String>,
    },
    /// Print full initialization script (env + completions)
    Init {
        /// Optional shell name (bash, zsh, fish, nu, powershell)
        shell: Option<String>,
    },
    /// Print shell completions only
    Completions {
        /// Optional shell name (bash, zsh, fish, nu, powershell)
        shell: Option<String>,
    },
}

#[derive(Subcommand)]
enum Commands {
    /// Start a session with Claude
    Claude,
    /// Start a session with GitHub Copilot
    Copilot,
    /// Resume the last active session
    Resume,
    /// Load a specific session by ID
    Load {
        id: Uuid,
    },
    /// List all saved sessions
    List,
    /// Start a session with Codex
    Codex,
    /// Start a session with Ollama (Direct API)
    Ollama,
    /// Start a session with Opencode
    Opencode,
    /// Authenticate with a specific AI provider
    Auth {
        /// The name of the AI provider to authenticate with (e.g., claude, copilot, gemini)
        provider_name: String,
    },
}

// ANSI color codes
const GREEN: &str = "32m";
const YELLOW: &str = "33m";
const RED: &str = "31m";
const BLUE: &str = "34m";
const BOLD: &str = "1m";

// Helper function for conditionally colored output
fn colored_text(text: &str, color_code: &str, color_enabled: bool) -> String {
    if color_enabled {
        format!("\x1b[{}{}\x1b[0m", color_code, text)
    } else {
        text.to_string()
    }
}

// Debug print macro
macro_rules! debug_println {
    ($debug_enabled:expr, $($arg:tt)*) => {
        if $debug_enabled {
            eprint!("[DEBUG] ");
            eprintln!($($arg)*);
        }
    };
}

fn print_version() {
    println!("aiprompt v{}", build::PKG_VERSION);
    println!("Build Date: {}", build::BUILD_TIME);
    println!("Commit ID:  {}", build::COMMIT_HASH);
}

fn detect_shell(provided: Option<String>) -> String {
    provided.or_else(|| env::var("SHELL").ok())
            .and_then(|s| s.split('/').last().map(|s| s.to_string()))
            .unwrap_or_else(|| "bash".to_string())
}

fn print_env_vars(shell_name: &str) {
    match shell_name {
        "fish" => {
            println!("set -gx AIPROMPT_SHELL \"fish\"");
        }
        "nu" | "nushell" => {
            println!("$env.AIPROMPT_SHELL = \"nu\"");
        }
        "powershell" | "pwsh" => {
            println!("$env:AIPROMPT_SHELL = \"powershell\"");
        }
        _ => {
            println!("export AIPROMPT_SHELL=\"{}\"", shell_name);
        }
    }
}

fn generate_completions(shell_name: &str, buf: &mut Vec<u8>) {
    let mut cmd = Cli::command();
    match shell_name {
        "bash" => generate(ClapShell::Bash, &mut cmd, "aiprompt", buf),
        "zsh" => generate(ClapShell::Zsh, &mut cmd, "aiprompt", buf),
        "fish" => generate(ClapShell::Fish, &mut cmd, "aiprompt", buf),
        "powershell" | "pwsh" => generate(ClapShell::PowerShell, &mut cmd, "aiprompt", buf),
        "nu" | "nushell" => generate(Nushell, &mut cmd, "aiprompt", buf),
        _ => generate(ClapShell::Bash, &mut cmd, "aiprompt", buf),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Determine color settings
    let mut color_enabled = match cli.color {
        ColorOption::Always => true,
        ColorOption::Never => false,
        ColorOption::Auto => io::stdout().is_terminal(),
    };

    // NO_COLOR environment variable overrides --color auto/always
    if env::var("NO_COLOR").is_ok() {
        debug_println!(cli.debug, "NO_COLOR environment variable detected. Disabling colors.");
        color_enabled = false;
    }

    let debug_enabled = cli.debug;

        debug_println!(debug_enabled, "Color enabled: {}", color_enabled);

        debug_println!(debug_enabled, "Debug enabled: {}", debug_enabled);

    

        let config = Config::load()?;

        

        // config loaded from AIPROMPT_CONFIG or default path

    

        // Apply environment variables from config

        if !config.env.is_empty() {

            for (key, value) in &config.env {

                env::set_var(key, value);

            }

        }

    

        if cli.version {

            print_version();

            return Ok(());

        }

    

        if let Some(shell_opt) = cli.shell_options {

            match shell_opt.shell_command {

                ShellCommands::Shell { action } => match action {

                    ShellAction::Env { shell: s } => {

                        let shell_name = detect_shell(s);

                        print_env_vars(&shell_name);

                        return Ok(());

                    }

                    ShellAction::Init { shell: s } => {

                        let shell_name = detect_shell(s);

                        print_env_vars(&shell_name);

                        let mut comp_buf = Vec::new();

                        generate_completions(&shell_name, &mut comp_buf);

                        io::stdout().write_all(&comp_buf)?;

                        return Ok(());

                    }

                    ShellAction::Completions { shell: s } => {

                        let shell_name = detect_shell(s);

                        let mut comp_buf = Vec::new();

                        generate_completions(&shell_name, &mut comp_buf);

                        io::stdout().write_all(&comp_buf)?;

                        return Ok(());

                    }

                }

            }

        }

    

        if let Some(command) = cli.command {

            return handle_cli_command(command, color_enabled, debug_enabled).await;

        }

    

        let mode = detect_interface_mode();

        match mode {

            InterfaceMode::Cli => {

                 println!("{}", colored_text("CLI Mode detected. Please provide a command or configure a default provider.", BLUE, color_enabled));

            }

            InterfaceMode::Tui | InterfaceMode::Gui => {

                let provider_name = if !config.provider.is_empty() {

                    config.provider.clone()

                } else {

                    "claude".to_string()

                };

    

                let mut system_message_content = String::new();

                if !config.context.always.is_empty() || !config.context.never.is_empty() {

                    system_message_content.push_str("You are an AI assistant. Follow these rules:\n");

                    for rule in &config.context.always {

                        system_message_content.push_str(&format!("- Always: {}\n", rule));

                    }

                    for rule in &config.context.never {

                        system_message_content.push_str(&format!("- Never: {}\n", rule));

                    }

                }

                let system_message = if system_message_content.is_empty() {

                    None

                } else {

                    Some(system_message_content)

                };

    

                let session = if let Some(s) = Session::get_session_for_cwd()? {

                    s

                } else {

                    Session::new(provider_name.clone(), system_message.clone())?

                };

                

                                                                let provider: Box<dyn Provider + Sync + Send> = match provider_name.as_str() {

                

                                                

                

                                                                                    "claude" => Box::new(ClaudeWrapper::new()),

                

                                                

                

                                                                                    "copilot" => Box::new(CopilotWrapper::new()),

                

                                                

                

                                                                                    "codex" => Box::new(CodexWrapper::new()),

                

                                                

                

                                                                                    "ollama" => Box::new(OllamaWrapper::new()),

                

                                                

                

                                                                                    "opencode" => Box::new(OpencodeWrapper::new()),

                

                                                

                

                                                                                    _ => bail!("Unknown default provider: {}", provider_name),

                

                                                

                

                                                                                };

                run_repl(session, provider, &config, color_enabled, debug_enabled).await?;

            }

                    Commands::Ollama => {
                        let mut system_message_content = String::new();
                        if !config.context.always.is_empty() || !config.context.never.is_empty() {
                            system_message_content.push_str("You are an AI assistant. Follow these rules:\n");
                            for rule in &config.context.always {
                                system_message_content.push_str(&format!("- Always: {}\n", rule));
                            }
                            for rule in &config.context.never {
                                system_message_content.push_str(&format!("- Never: {}\n", rule));
                            }
                        }
                        let system_message = if system_message_content.is_empty() {
                            None
                        } else {
                            Some(system_message_content)
                        };
                        let session = Session::new("ollama".to_string(), system_message)?;
                        run_repl(session, Box::new(OllamaWrapper::new()), &config, color_enabled, debug_enabled).await?;
                    }
                                                        Commands::Opencode => {
                                                            let mut system_message_content = String::new();
                                                            if !config.context.always.is_empty() || !config.context.never.is_empty() {
                                                                system_message_content.push_str("You are an AI assistant. Follow these rules:\n");
                                                                for rule in &config.context.always {
                                                                    system_message_content.push_str(&format!("- Always: {}\n", rule));
                                                                }
                                                                for rule in &config.context.never {
                                                                    system_message_content.push_str(&format!("- Never: {}\n", rule));
                                                                }
                                                            }
                                                            let system_message = if system_message_content.is_empty() {
                                                                None
                                                            } else {
                                                                Some(system_message_content)
                                                            };
                                                            let session = Session::new("opencode".to_string(), system_message)?;
                                                            run_repl(session, Box::new(OpencodeWrapper::new()), &config, color_enabled, debug_enabled).await?;
                                                        }
                                                                                            Commands::Opencode => {
                                                                                                let mut system_message_content = String::new();
                                                                                                if !config.context.always.is_empty() || !config.context.never.is_empty() {
                                                                                                    system_message_content.push_str("You are an AI assistant. Follow these rules:\n");
                                                                                                    for rule in &config.context.always {
                                                                                                        system_message_content.push_str(&format!("- Always: {}\n", rule));
                                                                                                    }
                                                                                                    for rule in &config.context.never {
                                                                                                        system_message_content.push_str(&format!("- Never: {}\n", rule));
                                                                                                    }
                                                                                                }
                                                                                                let system_message = if system_message_content.is_empty() {
                                                                                                    None
                                                                                                } else {
                                                                                                    Some(system_message_content)
                                                                                                };
                                                                                                let session = Session::new("opencode".to_string(), system_message)?;
                                                                                                run_repl(session, Box::new(OpencodeWrapper::new()), &config, color_enabled, debug_enabled).await?;
                                                                                            }
                                                                                                                                                        Commands::Opencode => {
                                                                                                                                                            let mut system_message_content = String::new();
                                                                                                                                                            if !config.context.always.is_empty() || !config.context.never.is_empty() {
                                                                                                                                                                system_message_content.push_str("You are an AI assistant. Follow these rules:\n");
                                                                                                                                                                for rule in &config.context.always {
                                                                                                                                                                    system_message_content.push_str(&format!("- Always: {}\n", rule));
                                                                                                                                                                }
                                                                                                                                                                for rule in &config.context.never {
                                                                                                                                                                    system_message_content.push_str(&format!("- Never: {}\n", rule));
                                                                                                                                                                }
                                                                                                                                                            }
                                                                                                                                                            let system_message = if system_message_content.is_empty() {
                                                                                                                                                                None
                                                                                                                                                            } else {
                                                                                                                                                                Some(system_message_content)
                                                                                                                                                            };
                                                                                                                                                            let session = Session::new("opencode".to_string(), system_message)?;
                                                                                                                                                            run_repl(session, Box::new(OpencodeWrapper::new()), &config, color_enabled, debug_enabled).await?;
                                                                                                                                                        }
                                                                                                                                                        Commands::Auth { provider_name } => {
                                                                                                                    
                                                                                                                                                            handle_auth_command(provider_name, color_enabled, debug_enabled).await?;
                                                                                                                    
                                                                                                                                                        }
                                                                                                                    
                                                                                                                                                    }                Ok(())

            }

    

    enum InterfaceMode {

        Cli,

        Tui,

        Gui,

    }

    

    fn detect_interface_mode() -> InterfaceMode {

        if !io::stdout().is_terminal() || !io::stdin().is_terminal() {

            return InterfaceMode::Cli;

        }

        let is_remote = env::var("SSH_CONNECTION").is_ok() 

            || env::var("SSH_CLIENT").is_ok() 

            || env::var("MOSH_SERVER_PID").is_ok();

        let has_display = env::var("DISPLAY").is_ok() || env::var("WAYLAND_DISPLAY").is_ok();

    

        if is_remote { return InterfaceMode::Tui; }

        if has_display { return InterfaceMode::Gui; }

        InterfaceMode::Tui

    }

    

    async fn handle_auth_command(

    

        provider_name: String,

    

        color_enabled: bool,

    

        debug_enabled: bool,

    

    ) -> Result<()> {

    

        debug_println!(debug_enabled, "Authenticating with provider: {}", provider_name);

    

    

    

        let (executable, args) = match provider_name.to_lowercase().as_str() {

    

            "claude" => ("claude", vec!["login"]),

    

            "copilot" => ("copilot", vec!["auth"]),

    

            "gemini" => ("gemini", vec!["auth", "login"]),

    

            "codex" => ("codex", vec!["login"]),

    

            "ollama" => ("ollama", vec!["signin"]),

    

            "opencode" => ("opencode", vec!["auth", "login"]),

    

            "openclaw" => ("openclaw", vec!["login"]),

    

            _ => {

    

                bail!("{}", colored_text(&format!("Unknown provider for authentication: {}", provider_name), RED, color_enabled));

    

            }

    

        };

    

    

    

        debug_println!(debug_enabled, "Executing auth command: {} {:?}", executable, args);

    

    

    

        let mut command = Command::new(executable);

    

        command.args(args);

    

        command.envs(std::env::vars()); // Inherit environment variables

    

    

    

        let output = command.output().await?;

    

    

    

        if output.status.success() {

    

            println!("{}", colored_text(&format!("Successfully authenticated with {}", provider_name), GREEN, color_enabled));

    

            io::stdout().write_all(&output.stdout)?;

    

        } else {

    

            println!("{}", colored_text(&format!("Authentication failed for {}", provider_name), RED, color_enabled));

    

            io::stderr().write_all(&output.stderr)?;

    

            bail!("Authentication command failed with status: {}", output.status);

    

        }

    

    

    

        Ok(())

    

    }

    

    

    

    async fn handle_cli_command(command: Commands, color_enabled: bool, debug_enabled: bool) -> Result<()> {

    

    

    

        debug_println!(debug_enabled, "Handling CLI command: {:?}", command);

    

    

    

        // Load config for CLI commands

    

    

    

        let config = Config::load()?;

    

    

    

    

    

    

    

                                match command {

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                    Commands::Claude => {

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                        let mut system_message_content = String::new();

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                        if !config.context.always.is_empty() || !config.context.never.is_empty() {

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                            system_message_content.push_str("You are an AI assistant. Follow these rules:\n");

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                            for rule in &config.context.always {

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                                system_message_content.push_str(&format!("- Always: {}\n", rule));

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                            }

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                            for rule in &config.context.never {

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                                system_message_content.push_str(&format!("- Never: {}\n", rule));

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                            }

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                        }

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                        let system_message = if system_message_content.is_empty() {

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                            None

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                        } else {

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                            Some(system_message_content)

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                        };

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                        let session = Session::new("claude".to_string(), system_message)?;

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                        run_repl(session, Box::new(ClaudeWrapper::new()), &config, color_enabled, debug_enabled).await?;

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                    }

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                    Commands::Copilot => {

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                        let mut system_message_content = String::new();

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                        if !config.context.always.is_empty() || !config.context.never.is_empty() {

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                            system_message_content.push_str("You are an AI assistant. Follow these rules:\n");

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                            for rule in &config.context.always {

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                                system_message_content.push_str(&format!("- Always: {}\n", rule));

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                            }

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                            for rule in &config.context.never {

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                                system_message_content.push_str(&format!("- Never: {}\n", rule));

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                            }

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                        }

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                        let system_message = if system_message_content.is_empty() {

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                            None

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                        } else {

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                            Some(system_message_content)

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                        };

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                        let session = Session::new("copilot".to_string(), system_message)?;

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                        run_repl(session, Box::new(CopilotWrapper::new()), &config, color_enabled, debug_enabled).await?;

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                    }

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                    Commands::Resume => {

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                        if let Some(session) = Session::get_session_for_cwd()? {

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                            resume_session(session, &config, color_enabled, debug_enabled).await?;

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                        } else {

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                            println!("{}", colored_text("No session found for current directory. Start a new session or load by ID.", YELLOW, color_enabled));

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                        }

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                    }

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                    Commands::Load { id } => {

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                        let session = Session::load(id)?;

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                        resume_session(session, &config, color_enabled, debug_enabled).await?;

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                    }

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                    Commands::List => {

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                        let sessions = Session::list_sessions()?;

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                        println!("{}", colored_text("Saved Sessions:", GREEN, color_enabled));

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                        for s in sessions {

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                            println!("- {} [{}] ({} msgs) - Updated: {}", colored_text(&s.id.to_string(), BLUE, color_enabled), s.provider, s.messages.len(), s.updated_at);

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                        }

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                    }

    

    

    

    

    

    

    

                                    Commands::Codex => {

    

    

    

    

    

    

    

                                        let mut system_message_content = String::new();

    

    

    

    

    

    

    

                                        if !config.context.always.is_empty() || !config.context.never.is_empty() {

    

    

    

    

    

    

    

                                            system_message_content.push_str("You are an AI assistant. Follow these rules:\n");

    

    

    

    

    

    

    

                                            for rule in &config.context.always {

    

    

    

    

    

    

    

                                                system_message_content.push_str(&format!("- Always: {}\n", rule));

    

    

    

    

    

    

    

                                            }

    

    

    

    

    

    

    

                                            for rule in &config.context.never {

    

    

    

    

    

    

    

                                                system_message_content.push_str(&format!("- Never: {}\n", rule));

    

    

    

    

    

    

    

                                            }

    

    

    

    

    

    

    

                                        }

    

    

    

    

    

    

    

                                        let system_message = if system_message_content.is_empty() {

    

    

    

    

    

    

    

                                            None

    

    

    

    

    

    

    

                                        } else {

    

    

    

    

    

    

    

                                            Some(system_message_content)

    

    

    

    

    

    

    

                                        };

    

    

    

    

    

    

    

                                        let session = Session::new("codex".to_string(), system_message)?;

    

    

    

    

    

    

    

                                        run_repl(session, Box::new(CodexWrapper::new()), &config, color_enabled, debug_enabled).await?;

    

    

    

    

    

    

    

                                    }

    

    

    

    

    

    

    

                                    Commands::Auth { provider_name } => {

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                        handle_auth_command(provider_name, color_enabled, debug_enabled).await?;

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                    }

    

    

    

    

    

    

    

                

    

    

    

    

    

    

    

                                }

    

    

    

    }

    

    

    

    async fn resume_session(session: Session, config: &Config, color_enabled: bool, debug_enabled: bool) -> Result<()> {

    

                        println!("{}", colored_text(&format!("Resuming session {} (Provider: {})", session.id, session.provider), GREEN, color_enabled));

    

                

    

                                let provider: Box<dyn Provider + Sync + Send> = match session.provider.as_str() {

    

                

    

                        

    

                

    

                                    "claude" => Box::new(ClaudeWrapper::new()),

    

                

    

                        

    

                

    

                                    "copilot" => Box::new(CopilotWrapper::new()),

    

                

    

                        

    

                

    

                                    "codex" => Box::new(CodexWrapper::new()),

    

                

    

                        

    

                

    

                                    "ollama" => Box::new(OllamaWrapper::new()),

    

                

    

                                    "opencode" => Box::new(OpencodeWrapper::new()),

    

                

    

                        

    

                

    

                                    _ => bail!("Unknown provider in saved session"),

    

                

    

                        

    

                

    

                                };

    

                

    

                        run_repl(session, provider, config, color_enabled, debug_enabled).await

    

    }

    

    

    

    async fn run_repl(mut session: Session, provider: Box<dyn Provider + Sync + Send>, config: &Config, color_enabled: bool, debug_enabled: bool) -> Result<()> {

    

        let mut rl = DefaultEditor::new()?;

    

        println!("{}", colored_text(&format!("Starting session with {}. Type 'exit' or 'quit' to stop.", provider.name()), GREEN, color_enabled));

    

    

    

        if !session.messages.is_empty() {

    

            println!("{}", colored_text("--- Context ---", BLUE, color_enabled));

    

            for msg in session.messages.iter().rev().take(5).rev() {

    

                 println!("{}: {}", colored_text(&format!("{:?}", msg.role), BLUE, color_enabled), msg.content);

    

            }

    

            println!("{}", colored_text("--- End Context ---", BLUE, color_enabled));

    

        }

    

    

    

        loop {

    

            let readline = rl.readline(">> ");

    

            match readline {

    

                Ok(line) => {

    

                    let input = line.trim();

    

                    if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {

    

                        break;

    

                    }

    

                    if input.is_empty() {

    

                        continue;

    

                    }

    

    

    

                    session.add_message(Role::User, input.to_string());

    

                    session.save()?;

    

    

    

                    print!("{}", colored_text("Thinking...", YELLOW, color_enabled));

    

                    io::stdout().flush()?;

    

    

    

                    match provider.send(&session, config).await {

    

                        Ok(response) => {

    

                            println!("\r\x1b[K{}", response);

    

    

    

                            if response.contains("git commit") || response.contains("git push") {

    

                                println!("\n{}", colored_text("⚠️  SAFETY WARNING: The AI generated a git operation command.", &format!("{};{}", BOLD, RED), color_enabled));

    

                                println!("{}", colored_text("⚠️  'git commit' and 'git push' are blocked by default configuration.", &format!("{};{}", BOLD, RED), color_enabled));

    

                                println!("{}", colored_text("⚠️  Please review the code carefully before executing.", &format!("{};{}", BOLD, RED), color_enabled));

    

                            }

    

    

    

                            session.add_message(Role::Assistant, response);

    

                            session.save()?;

    

                        }

    

                        Err(e) => {

    

                            println!("\r\x1b[K{}", colored_text(&format!("Error calling provider: {}", e), RED, color_enabled));

    

                        }

    

                    }

    

                }

    

                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,

    

                Err(err) => {

    

                    println!("{}", colored_text(&format!("Error: {:?}", err), RED, color_enabled));

    

                    break;

    

                }

    

            }

    

        }

    

        session.save()?;

    

        println!("{}", colored_text("Session saved. Goodbye!", GREEN, color_enabled));

    

        Ok(())

    

    }