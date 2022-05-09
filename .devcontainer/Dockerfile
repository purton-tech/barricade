FROM purtontech/rust-on-nails-devcontainer:1.0.5 AS cargo

# Configure apt and install packages
RUN sudo apt-get update \
    && sudo apt-get install -y --no-install-recommends \
        libpq-dev \
    #
    #
    # Clean up
    && sudo apt-get autoremove -y \
    && sudo apt-get clean -y \
    && sudo rm -r /var/cache/* /var/lib/apt/lists/*

RUN cargo install diesel_cli --no-default-features --features postgres && ls -la /home/vscode

FROM purtontech/rust-on-nails-devcontainer:1.0.5 AS development

ARG USERNAME=vscode

COPY --from=cargo /usr/local/cargo/bin/diesel* /usr/local/cargo/bin/

COPY *.bash .

# Add our aliases and ps1 to bashrc
RUN cat *.bash >> ~/.bashrc && sudo rm *.bash

# Enable our git hooks and set the permisisons on docker sock.
RUN echo 'git config core.hooksPath /vscode/.devcontainer/.githooks' >> ~/.bashrc \
    && echo 'sudo chmod 777 /var/run/docker.sock' >> ~/.bashrc

# Setup volumes so they have non root owner
RUN sudo mkdir -p /workspace/target && sudo chown $USERNAME:$USERNAME /workspace/target
RUN sudo mkdir -p /workspace/node_modules && sudo chown $USERNAME:$USERNAME /workspace/node_modules