# aiprompt

`aiprompt` is a robust, multimodal, and fault-tolerant Rust binary that provides a unified interface for multiple AI code generation tools. It acts as a smart orchestrator, managing conversation context, handling persistent sessions, and providing a safety layer over official AI command-line interfaces.

## Key Features

- **Unified Multimodal UI:** Automatically adapts to your environment:
    - **CLI Mode:** Optimized for scripts, pipes, and rapid commands.
    - **TUI Mode:** Rich terminal interface (`ratatui`) for SSH, Mosh, and interactive console use.
    - **GUI Mode:** Native graphical window (`iced`) for local Wayland/X11 sessions.
- **Context Persistence (Per-Directory):** Automatically maintains conversation history scoped to your current working directory. `aiprompt --resume` brings you back to the relevant context for your project.
- **Crash Recovery:** Sessions are serialized to disk in real-time. Never lose your conversation state to a terminal crash or network timeout.
- **Unified Permission Engine:** A structured configuration system to allow or deny file, directory, command, and web access across all AI providers.
- **Git Safety Guard:** Strictly prohibits AI agents from executing `git commit` and `git push` by default.
- **Cross-Platform Static Binaries:** Single-file executables with zero runtime dependencies. Supports Linux, macOS, Windows, and 4 BSD variants on both `amd64` and `arm64`.

## Supported Providers

| Provider | Integration | Binary | Installation |
| :--- | :--- | :--- | :--- |
| **Claude** | Wrapper | `claude` | `curl .../bootstrap.sh | bash` |
| **Copilot** | Wrapper | `copilot` | Standalone binary |
| **Gemini** | Wrapper | `gemini` | `npm install -g @google/gemini-cli` |
| **Codex** | Wrapper | `codex` | `npm install -g @openai/codex` |
| **Ollama** | Direct API | `ollama` | [ollama.com](https://ollama.com) |

## Installation

Download the static binary for your platform and architecture from the [Releases](https://github.com/casapps/aiprompt/releases) page.

**Artifact Naming:** `aiprompt-{platform}-{arch}` (e.g., `aiprompt-linux-amd64`).

## Quick Start

### Start/Resume a Session
```bash
# Start/Resume session in the current directory using default provider
aiprompt

# Start a session with a specific provider
aiprompt claude
```

### Unified Shell Integration
Add this to your shell profile (e.g., `.bashrc`, `.zshrc`, `config.fish`):
```bash
eval $(aiprompt --shell init)
```

## Configuration
Configuration is stored in platform-standard locations (e.g., `~/.config/aiprompt/config.yaml`).

```yaml
provider: claude
env:
  ANTHROPIC_API_KEY: "sk-..."
permissions:
  commands:
    - allow: all
    - deny: ["shell(git:commit)", "shell(git:push)"]
```

## Development

- **Specification:** See [AI.md](AI.md) for the full technical specification and project rules.
- **Roadmap:** See [TODO.AI.md](TODO.AI.md) for implementation status.

## License
MIT License. See [LICENSE.md](LICENSE.md) for details and attributions.
