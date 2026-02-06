# AI Project Specification: aiprompt (The SPEC)

## PROJECT RULES
*   **No Inline Comments:** Comments must always be placed *above* the code they describe, never on the same line.
*   **No AI Attribution:** Never add signatures, comments, or headers attributing work to an AI. All work is performed on behalf of the user.
*   **Follow SPEC:** Rigorously adhere to this specification. Use `grep`, `search`, and other discovery tools to ensure full context and verify requirements before acting.
*   **No Assumptions:** NEVER assume or guess. If any requirement, state, or path is ambiguous, **ASK THE USER** for clarification.
*   **Git Restrictions:** NEVER execute `git commit` or `git push`.
*   **Commit Documentation:** You may create or update the file `.git/COMMIT_MESS`. This file must accurately reflect the actual git state and current changes.

## Project Goal
To create a robust, fault-tolerant, self-contained Rust binary named `aiprompt` that acts as a unified wrapper and context manager for various AI CLI tools. It provides a multimodal interface (CLI, TUI, GUI) that automatically adapts to the user's environment.

## 1. Core Mandates
*   **Hybrid Architecture:** 
    *   **Wrappers:** Orchestrates existing CLI tools (`claude`, `copilot`, `gemini`, `codex`, `opencode`, `openclaw`).
    *   **Direct API:** Interacts directly with HTTP APIs where preferred (`ollama`).
*   **Multimodal Interface (Smart Detection):**
    *   **CLI Mode:** Default for scripts, pipes, or when flags are present.
    *   **TUI Mode:** Interactive terminal sessions (SSH, Mosh, or standard terminal) using `ratatui`.
    *   **GUI Mode:** Local display sessions (Wayland/X11) using `iced`. Forbidden over SSH/Mosh.
*   **Static Cross-Platform Binary:**
    *   **Targets:** 8 Platforms x 2 Architectures = 16 Binaries.
    *   **Platforms:** Linux, Darwin (macOS), Windows, FreeBSD, OpenBSD, NetBSD, DragonFlyBSD, Solaris/Illumos.
    *   **Architectures:** amd64 (x86_64), arm64.
    *   **Artifact Naming:** `{projectname}-{platform}-{arch}` (e.g., `aiprompt-freebsd-arm64`). Suffixes like `-musl` are always stripped. Windows gets `.exe`.
    *   **Release Archive:** Source archive must exclude all VCS files (`.git`, `.gitignore`, etc.).
*   **Context & Persistence:** 
    *   Conversation history is serialized to JSON immediately after every user input and AI response.
    *   **Per-Directory Context:** Scoped session resumption based on CWD via `directory_map.json`.
    *   Crash recovery allows resuming the last session via `aiprompt resume`.

## 2. Configuration Example (`config.yaml`)

# The default AI provider to use when no provider is explicitly specified
provider: claude

# Environment variables to be exported to the underlying AI tools
env:
  ANTHROPIC_API_KEY: "sk-ant-..."
  GEMINI_API_KEY: "..."
  OLLAMA_HOST: "http://localhost:11434"

# Detailed permission settings for the AI tools
permissions:
  # File access rules: uses glob patterns like /** for granular file matching
  files:
    - allow: all
    - deny: ["/etc/**", "/usr/**"]
  # Directory access rules: inherently recursive
  # Denying a path blocks everything inside it.
  directories:
    - allow: ["/"]
    - deny: ["/etc/mail"]
  # Command execution rules
  commands:
    - allow: all
    - deny: ["shell(git:commit)", "shell(git:push)"]
  # Web/Network access rules
  web:
    - allow: all
    - deny: ""

# Mandatory behavioral rules injected into every session
context:
  # Rules that must always be followed
  always:
    - Follow SPEC (use grep/search ensure full context)
    - Ask the user if unsure
  # Prohibited behaviors
  never:
    - add AI attribution
    - assume or guess.

## 3. Platform Directory Structure
The binary must ensure these exist with correct permissions (`0700` for dirs, `0600` for files).

| Platform | Config Directory | Data Directory (Sessions) | Log Directory |
| :--- | :--- | :--- | :--- |
| **Linux/BSD/Solaris** | `~/.config/aiprompt/` | `~/.local/share/aiprompt/` | `~/.local/log/aiprompt/` |
| **macOS (Darwin)** | `~/Library/Application Support/aiprompt/` | `~/Library/Application Support/aiprompt/` | `~/Library/Logs/aiprompt/` |
| **Windows** | `%AppData%\aiprompt\` | `%LocalAppData%\aiprompt\` | `%LocalAppData%\aiprompt\logs\` |
| **Root User** | `/root/.config/aiprompt/` | `/root/.local/share/aiprompt/` | `/root/.local/log/aiprompt/` |

## 4. Normalization & Translation Engine
The engine normalizes user inputs and translates them into provider-specific flags.

### Translation Logic Examples
| aiprompt Input | Internal Normalized | Translated Copilot Flag |
| :--- | :--- | :--- |
| `deny: "shell(git:commit)"` | `git commit` | `--deny-tool 'shell(git commit)'` |
| `deny: "git push"` | `git push` | `--deny-tool 'shell(git push)'` |
| `allow: "url:google.com"` | `google.com` | `--allow-url google.com` |
| `allow: "all"` | `all` | `--allow-all-tools --allow-all-urls` |

### Path Matching Logic
*   **Files:** Uses glob pattern matching (e.g., `/**`).
*   **Directories:** Uses recursive prefix matching. If a path is denied, all descendants are denied unless a more specific allow rule exists (e.g., allow `/` but deny `/etc/mail`).

## 5. Sandbox Enforcement
*   **Allowed Paths:** `/tmp`, `~`, `/root`, `/usr/local`, `/var`.
*   **Denied Paths:** `/etc`, `/usr` (except `/usr/local`), `/`.
*   **Forbidden Commands:** `git commit`, `git push`.

## 6. Integrated AI Services

| Provider | integration | Binary Name | Auth Command |
| :--- | :--- | :--- | :--- |
| **Claude** | Wrapper | `claude` | `claude login` |
| **Copilot** | Wrapper | `copilot` | `copilot auth` |
| **Gemini** | Wrapper | `gemini` | `gemini auth login` |
| **Codex** | Wrapper | `codex` | `codex login` |
| **Ollama** | Direct API | `ollama` | `ollama signin` |
| **Opencode** | Wrapper | `opencode` | `opencode auth login` |
| **OpenClaw** | Wrapper | `openclaw` | `openclaw login` |

## 7. Architecture Components
*   **Command Dispatcher:** Routes commands based on provider selection.
*   **Interface Manager:** Detects env (SSH, TTY, Display) and launches CLI/TUI/GUI.
*   **Session Manager:** Manages per-directory session mapping (`directory_map.json`).
*   **Filesystem Manager:** Cross-platform path resolution and permission enforcement.
*   **Provider Engine:** Subprocess execution with environment variable injection.
*   **Sandbox Engine:** Active interceptor for forbidden paths and commands.