FROM ianpurton/rust-slim-devcontainer:latest AS cargo

# Configure apt and install packages
RUN sudo apt-get update \
    && sudo apt-get install -y --no-install-recommends \
        # CA certificates - used by curl i.e. for rustup
        #ca-certificates \
        libpq-dev \
    #
    #
    # Clean up
    && sudo apt-get autoremove -y \
    && sudo apt-get clean -y \
    && sudo rm -r /var/cache/* /var/lib/apt/lists/*

RUN cargo install diesel_cli --no-default-features --features postgres && ls -la /home/vscode

FROM ianpurton/rust-slim-devcontainer:latest AS development

ARG USERNAME=vscode

COPY --from=cargo /usr/local/cargo/bin/diesel* /usr/local/cargo/bin/

# Configure apt and install packages
RUN sudo apt-get update \
    && sudo apt-get install -y --no-install-recommends \
        # CA certificates - used by curl i.e. for rustup
        #ca-certificates \
        libpq-dev \
        postgresql-client \
        # required by parcel or you can't npm install 
        build-essential \
    #
    #
    # Clean up
    && sudo apt-get autoremove -y \
    && sudo apt-get clean -y \
    && sudo rm -r /var/cache/* /var/lib/apt/lists/*

# Install node.
RUN curl -fsSL https://deb.nodesource.com/setup_16.x | sudo -E bash -
RUN sudo apt-get install -y nodejs

COPY *.bash .

RUN cat *.bash >> ~/.bashrc && sudo rm *.bash

# Setup volumes so they have non root owner
RUN sudo mkdir -p /vscode/target && sudo chown $USERNAME:$USERNAME /vscode/target
RUN sudo mkdir -p /vscode/node_modules && sudo chown $USERNAME:$USERNAME /vscode/node_modules