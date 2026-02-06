// Copyright (c) 2026 casapps
// Licensed under the MIT License. See LICENSE.md in the project root for license information.

mod session;
mod wrappers;
mod config;

use clap::{Parser, Subcommand, CommandFactory};
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

shadow_rs::shadow!(build);

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
    let config = Config::load()?;

    // Apply environment variables from config
    if let Some(env_vars) = &config.env {
        for (key, value) in env_vars {
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
        return handle_cli_command(command).await;
    }

    let mode = detect_interface_mode();
    match mode {
        InterfaceMode::Cli => {
             println!("CLI Mode detected. Please provide a command or configure a default provider.");
        }
        InterfaceMode::Tui | InterfaceMode::Gui => {
            let provider_name = config.provider.unwrap_or_else(|| "claude".to_string());
            let session = Session::new(provider_name.clone());
            let provider: Box<dyn Provider + Sync + Send> = match provider_name.as_str() {
                "claude" => Box::new(ClaudeWrapper::new()),
                "copilot" => Box::new(CopilotWrapper::new()),
                _ => bail!("Unknown default provider: {}", provider_name),
            };
            run_repl(session, provider).await?;
        }
    }

    Ok(())
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

async fn handle_cli_command(command: Commands) -> Result<()> {
    match command {
        Commands::Claude => {
            let session = Session::new("claude".to_string());
            run_repl(session, Box::new(ClaudeWrapper::new())).await?;
        }
        Commands::Copilot => {
            let session = Session::new("copilot".to_string());
            run_repl(session, Box::new(CopilotWrapper::new())).await?;
        }
        Commands::Resume => {
            if let Some(session) = Session::get_last_session()? {
                resume_session(session).await?;
            } else {
                println!("No previous session found.");
            }
        }
        Commands::Load { id } => {
            let session = Session::load(id)?;
            resume_session(session).await?;
        }
        Commands::List => {
            let sessions = Session::list_sessions()?;
            println!("Saved Sessions:");
            for s in sessions {
                println!("- {} [{}] ({} msgs) - Updated: {}", s.id, s.provider, s.messages.len(), s.updated_at);
            }
        }
    }
    Ok(())
}

async fn resume_session(session: Session) -> Result<()> {
    println!("Resuming session {} (Provider: {})", session.id, session.provider);
    let provider: Box<dyn Provider + Sync + Send> = match session.provider.as_str() {
        "claude" => Box::new(ClaudeWrapper::new()),
        "copilot" => Box::new(CopilotWrapper::new()),
        _ => bail!("Unknown provider in saved session"),
    };
    run_repl(session, provider).await
}

async fn run_repl(mut session: Session, provider: Box<dyn Provider + Sync + Send>) -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    println!("Starting session with {}. Type 'exit' or 'quit' to stop.", provider.name());

    if !session.messages.is_empty() {
        println!("--- Context ---");
        for msg in session.messages.iter().rev().take(5).rev() {
             println!("{:?}: {}", msg.role, msg.content);
        }
        println!("--- End Context ---");
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

                print!("Thinking...");
                io::stdout().flush()?;

                match provider.send(&session).await {
                    Ok(response) => {
                        println!("\r\x1b[K{}", response);

                        if response.contains("git commit") || response.contains("git push") {
                            println!("\n⚠️  SAFETY WARNING: The AI generated a git operation command.");
                            println!("⚠️  'git commit' and 'git push' are blocked by default configuration.");
                            println!("⚠️  Please review the code carefully before executing.");
                        }

                        session.add_message(Role::Assistant, response);
                        session.save()?;
                    }
                    Err(e) => {
                        println!("\r\x1b[KError calling provider: {}", e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    session.save()?;
    println!("Session saved. Goodbye!");
    Ok(())
}
