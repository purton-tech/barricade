FROM purtontech/rust-on-nails-devcontainer:1.0.6 AS development

ARG USERNAME=vscode

COPY *.bash .

# Add our aliases and ps1 to bashrc
RUN cat *.bash >> ~/.bashrc && sudo rm *.bash

# Enable our git hooks and set the permisisons on docker sock.
RUN echo 'git config core.hooksPath /vscode/.devcontainer/.githooks' >> ~/.bashrc \
    && echo 'sudo chmod 777 /var/run/docker.sock' >> ~/.bashrc

# Setup volumes so they have non root owner
RUN sudo mkdir -p /workspace/target && sudo chown $USERNAME:$USERNAME /workspace/target
RUN sudo mkdir -p /workspace/node_modules && sudo chown $USERNAME:$USERNAME /workspace/node_modules