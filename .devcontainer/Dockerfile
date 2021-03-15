FROM ianpurton/rust-devcontainer:latest AS base

FROM base AS development

# I like to have my bash history setup. It helps me remember what commands to run
COPY ./bash_history /home/vscode/.bash_history
RUN sudo chown vscode /home/vscode/.bash_history