FROM ianpurton/rust-fullstack-devcontainer:latest AS development

ARG USERNAME=vscode

COPY *.bash .

# Add our aliases and ps1 to bashrc
RUN cat *.bash >> ~/.bashrc && sudo rm *.bash

# Enable our git hooks and set the permisisons on docker sock.
RUN echo 'git config core.hooksPath /vscode/.devcontainer/.githooks' >> ~/.bashrc \
    && echo 'sudo chmod 777 /var/run/docker.sock' >> ~/.bashrc

# Setup volumes so they have non root owner
RUN sudo mkdir -p /vscode/target && sudo chown $USERNAME:$USERNAME /vscode/target
RUN sudo mkdir -p /vscode/node_modules && sudo chown $USERNAME:$USERNAME /vscode/node_modules