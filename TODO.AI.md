# TODO for aiprompt Project

## Phase 1: Core Architecture & Session Management
- [x] Define `Session` struct.
- [x] Implement `Session` persistence.
- [ ] **Config Management:**
    - [x] Basic YAML implementation.
    - [ ] Update to support structured `permissions` and `context` blocks.
    - [ ] Implement `AIPROMPT_CONFIG` override.
- [ ] **Filesystem Management:** 
    - [ ] Implement platform-specific path detection (Linux, Darwin, Windows, 4xBSD, Solaris).
    - [ ] Implement robust directory auto-creation with `0700` permissions.
    - [ ] Implement `0600` file permissions for config and sessions.
- [ ] **Directory-Aware Resuming:**
    - [ ] Implement `directory_map.json` to track CWD -> Session UUID.
    - [ ] Update `Session` loader to use mapped IDs automatically.
- [ ] **Context Engine:**
    - [ ] Implement logic to aggregate `context.always` and `context.never` rules from config.
    - [ ] Implement system-prompt injection for all 7 providers.

## Phase 2: Tool Integration (The Wrappers)
- [ ] **Normalization & Translation Engine:**
    - [ ] Implement input normalization (handle colons, quotes, parentheses).
    - [ ] Implement mapping tables for all 7 providers (flags, subcommands).
    - [ ] Build argument generator for subprocess calls.
- [ ] **Provider Integrations:**
    - [x] Anthropic Claude (Pending Translation Engine update).
    - [x] GitHub Copilot (Standalone binary, pending Translation Engine update).
    - [ ] Google Gemini (`gemini` binary).
    - [ ] OpenAI Codex (`codex` binary).
    - [ ] Ollama (Direct HTTP API via `reqwest`).
    - [ ] Opencode (`opencode` binary).
    - [ ] OpenClaw (`openclaw` binary).
- [ ] **Auth Command:** Implement `aiprompt [provider] auth login` for all 7 tools.

## Phase 3: Multimodal Interface
- [x] **Environment Detection:** Logic for SSH, TTY, Wayland/X11 detection.
- [ ] **CLI Mode:** Refine for pipes and non-interactive use.
- [ ] **TUI Mode:** Implement rich terminal UI using `ratatui`.
- [ ] **GUI Mode:** Implement graphical window using `iced`.

## Phase 4: Robustness, Safety & Build
- [x] **Git Guard:** Block `commit` and `push` commands.
- [ ] **Sandbox Engine:** Active path enforcement (`/etc`, `/usr` block; `/tmp`, `~`, `/root`, etc. allow).
- [ ] **Unified Flags:** Implement `--model`, `--temp`, `--tokens`, `--context`.
- [ ] **Build System:** 
    - [ ] Set up GitHub Actions for 16-binary release matrix.
    - [ ] Implement `{project}-{platform}-{arch}` naming logic.
    - [ ] Automate source archive creation (excluding VCS files).
