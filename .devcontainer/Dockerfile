FROM ianpurton/rust-fullstack-devcontainer:latest AS development

ARG USERNAME=vscode

RUN sudo apt install -y lld

COPY *.bash .

RUN cat *.bash >> ~/.bashrc && sudo rm *.bash

# Setup volumes so they have non root owner
RUN sudo mkdir -p /vscode/target && sudo chown $USERNAME:$USERNAME /vscode/target
RUN sudo mkdir -p /vscode/node_modules && sudo chown $USERNAME:$USERNAME /vscode/node_modules