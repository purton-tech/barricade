FROM ianpurton/rust-fullstack-devcontainer:latest

ARG EXE_NAME=authn-proxy
ARG FOLDER=.
ARG DOCKER_HUB_DESTINATION=authnproxy/authnproxy

WORKDIR /build

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

docker:
    FROM scratch
    COPY +build/$EXE_NAME rust-exe
    COPY --dir +npm-build/dist asset-pipeline/dist
    COPY --dir $FOLDER/asset-pipeline/images asset-pipeline/images
    EXPOSE 8080
    ENTRYPOINT ["./rust-exe"]
    SAVE IMAGE --push $DOCKER_HUB_DESTINATION:latest

integration-test:
    FROM +build
    COPY --dir $FOLDER/tests .
    COPY --dir migrations .
    COPY .devcontainer/docker-compose.yml ./ 
    ARG DATABASE_URL=postgresql://postgres:testpassword@localhost:5432
    ARG WEB_DRIVER_URL=http://localhost:4444/wd/hub
    ARG WEB_DRIVER_DESTINATION_HOST=localhost:8080
    USER root
    WITH DOCKER \
        --load webui:latest=+docker
        RUN docker run -d --rm --network=host webui:latest \
            && docker run -d --rm --network=host --shm-size="2g" selenium/standalone-chrome:3.141.59 \
            && docker run -d --rm --network=host -e POSTGRES_USER=postgres -e POSTGRES_PASSWORD=testpassword postgres:alpine \
            && while ! pg_isready --host=localhost --port=5432 --username=postgres; do sleep 1; done ;\
                diesel migration run \
            && cargo test hello_world --release --target x86_64-unknown-linux-musl -- --nocapture
    END
