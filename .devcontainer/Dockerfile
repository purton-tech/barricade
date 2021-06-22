FROM ianpurton/rust-devcontainer:latest AS base

FROM base AS development

# I like to have my bash history setup. It helps me remember what commands to run
COPY ./bash_history /home/vscode/.bash_history
RUN sudo chown vscode /home/vscode/.bash_history

RUN echo "alias cw='cargo watch -x fmt -x clippy -x run'"  >> ~/.bashrc \ 
    && echo "alias dmr='diesel migration run'"  >> ~/.bashrc \
    && echo "alias dml='diesel migration list'"  >> ~/.bashrc \
    && echo "alias cwl='FORWARD_PORT=8085 FORWARD_URL=host.docker.internal cargo watch -x fmt -x clippy -x run'"  >> ~/.bashrc \
    && echo "alias cwp='cargo watch --poll -x fmt -x clippy -x run'"  >> ~/.bashrc