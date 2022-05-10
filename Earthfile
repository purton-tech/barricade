FROM purtontech/rust-on-nails-devcontainer:1.0.6

ARG EXE_NAME=barricade
ARG FOLDER=.
ARG CONTAINER_NAME=purtontech/barricade:build
ARG SELENIUM=selenium/standalone-chrome:4.1.1-20220121

WORKDIR /build

all:
    BUILD +docker
    BUILD +integration-test

npm-deps:
    COPY $FOLDER/package.json package.json
    COPY $FOLDER/package-lock.json package-lock.json
    RUN npm install
    SAVE ARTIFACT node_modules

npm-build:
    FROM +npm-deps
    COPY $FOLDER/asset-pipeline asset-pipeline
    COPY +npm-deps/node_modules node_modules
    RUN npm run release
    SAVE ARTIFACT asset-pipeline/dist

prepare-cache:
    COPY --dir $FOLDER/src $FOLDER/Cargo.lock $FOLDER/Cargo.toml .
    RUN cargo chef prepare --recipe-path recipe.json
    SAVE ARTIFACT recipe.json

build-cache:
    COPY +prepare-cache/recipe.json ./
    RUN cargo chef cook --release --target x86_64-unknown-linux-musl 
    SAVE ARTIFACT target
    SAVE ARTIFACT $CARGO_HOME cargo_home
    SAVE IMAGE --cache-hint

build:
    COPY --dir $FOLDER/src $FOLDER/Cargo.lock $FOLDER/Cargo.toml $FOLDER/build.rs .
    COPY +build-cache/cargo_home $CARGO_HOME
    COPY +build-cache/target target
    RUN mkdir asset-pipeline
    COPY --dir +npm-build/dist asset-pipeline
    RUN cargo build --release --target x86_64-unknown-linux-musl
    SAVE ARTIFACT target/x86_64-unknown-linux-musl/release/$EXE_NAME $EXE_NAME
    SAVE IMAGE --cache-hint

# The final stage after testing, build our tiny container
# hub.docker.com/r/purtontech/barricade
docker:
    FROM scratch
    COPY +build/$EXE_NAME barricade
    COPY --dir +npm-build/dist asset-pipeline/dist
    ENTRYPOINT ["./barricade"]
    # We call the image build and let semantic release handle tagging and pushing latest
    SAVE IMAGE --push $CONTAINER_NAME

integration-test:
    FROM +build
    COPY --dir $FOLDER/tests .
    COPY --dir migrations .
    COPY .devcontainer/docker-compose.earthly.yml .
    COPY .devcontainer/docker-compose.yml .
    
    # Used by tests/common/mod.rs
    ARG SECRET_KEY=50fb08b06b381c575e60c56328f66a51822320e922c7e11e264a7bb443ee22fe
    ARG DATABASE_URL=postgresql://vscode:testpassword@localhost:5432
    ARG WEB_DRIVER_URL=http://localhost:4444
    ARG WEB_DRIVER_DESTINATION_HOST=http://app:9096

    USER root
    WITH DOCKER \
        # Bring up the containers we have built
        --load $CONTAINER_NAME=+docker \
        --compose docker-compose.yml \
        --compose docker-compose.earthly.yml \
        --service db \
        --service smtp \
        --service whoami \
        --service app \
        --service selenium

        RUN while ! pg_isready --host=localhost --port=5432 --username=vscode; do sleep 1; done ;\
                diesel migration run \
            && docker ps \
            && curl http://localhost:9096/auth/sign_in \
            && docker logs -t build_app_1 \
            && cargo test --release --target x86_64-unknown-linux-musl
    END
