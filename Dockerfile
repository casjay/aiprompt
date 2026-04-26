# Use the official Rust image as a base
FROM rust:1.76-slim-bookworm as builder

# Set the working directory inside the container
WORKDIR /app

# Install nightly Rust toolchain
RUN rustup install nightly
RUN rustup default nightly

# Install build dependencies for Claude CLI installer
# These packages are generally available in Debian-based images
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    bash \
    && rm -rf /var/lib/apt/lists/*

# Install Claude CLI
# The install.sh script typically places the executable in /usr/local/bin
RUN curl -fsSL https://claude.ai/install.sh | bash

# Install GitHub Copilot CLI
# The install script typically places the executable in /usr/local/bin
RUN curl -fsSL https://gh.io/copilot-install | bash

# Install Node.js and npm (NodeSource setup)
RUN apt-get update && apt-get install -y ca-certificates curl gnupg \
    && mkdir -p /etc/apt/keyrings \
    && curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key | gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg \
    && NODE_MAJOR=18 \
    && echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_$NODE_MAJOR.x nodistro main" | tee /etc/apt/sources.list.d/nodesource.list \
    && apt-get update && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# Install Gemini CLI
RUN npm install -g @google/gemini-cli

# Install Codex CLI
RUN npm install -g @openai/codex

# Install Opencode CLI
RUN curl -fsSL https://opencode.ai/install | bash

# --- TEMPORARY DEBUG COMMANDS ---
RUN find / -name gemini -type f -executable 2>/dev/null | xargs -r echo >&2
# --- END TEMPORARY DEBUG COMMANDS ---

# Copy all project files into the container
COPY . .

# Build the project
RUN cargo build --release

# Use a minimal base image for the final stage
FROM debian:bookworm-slim

# Copy the built executable from the builder stage
COPY --from=builder /app/target/release/aiprompt /usr/local/bin/aiprompt
# Copy the claude CLI executable from the builder stage
COPY --from=builder /root/.local/bin/claude /usr/local/bin/claude
# Copy the copilot CLI executable from the builder stage
COPY --from=builder /usr/local/bin/copilot /usr/local/bin/copilot
# Copy the gemini CLI executable from the builder stage
COPY --from=builder /usr/lib/node_modules/@google/gemini-cli/bin/gemini /usr/local/bin/gemini
# Copy the codex CLI executable from the builder stage
COPY --from=builder /usr/lib/node_modules/@openai/codex/bin/codex /usr/local/bin/codex
# Copy the opencode CLI executable from the builder stage
COPY --from=builder /usr/local/bin/opencode /usr/local/bin/opencode

# Set the entrypoint to run the application
ENTRYPOINT ["aiprompt"]