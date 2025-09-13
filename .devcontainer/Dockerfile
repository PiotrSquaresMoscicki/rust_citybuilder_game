
FROM mcr.microsoft.com/devcontainers/base:ubuntu

# Install Rust and prerequisites as vscode user
USER vscode
RUN sudo apt-get update && \
    sudo apt-get install -y curl build-essential && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y

# Set environment variables for Rust
ENV PATH="/home/vscode/.cargo/bin:${PATH}"

# Set working directory
WORKDIR /workspaces/rust_citybuilder_game