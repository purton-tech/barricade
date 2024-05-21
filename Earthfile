VERSION 0.8
FROM purtontech/rust-on-nails-devcontainer:1.3.1

ARG --global EXE_NAME=barricade
ARG --global EXE_FOLDER=crates/actix-server
ARG --global GRPC_API_FOLDER=crates/grpc-api
ARG --global DB_FOLDER=crates/db
ARG --global CONTAINER_NAME=purtontech/barricade:build
ARG --global SELENIUM=selenium/standalone-chrome:4.1.1-20220121
ARG --global PIPELINE_FOLDER=crates/asset-pipeline

WORKDIR /build

all:
    BUILD +app-container
    BUILD +integration-test

npm-deps:
    COPY $PIPELINE_FOLDER/package.json $PIPELINE_FOLDER/package.json
    COPY $PIPELINE_FOLDER/package-lock.json $PIPELINE_FOLDER/package-lock.json
    RUN cd $PIPELINE_FOLDER && npm install
    SAVE ARTIFACT $PIPELINE_FOLDER/node_modules

npm-build:
    FROM +npm-deps
    COPY $PIPELINE_FOLDER $PIPELINE_FOLDER
    COPY --if-exists $GRPC_API_FOLDER $GRPC_API_FOLDER
    COPY +npm-deps/node_modules $PIPELINE_FOLDER/node_modules
    RUN cd $PIPELINE_FOLDER && npm run release
    SAVE ARTIFACT $PIPELINE_FOLDER/dist

prepare-cache:
    # Copy in all our crates
    COPY --dir crates crates
    COPY Cargo.lock Cargo.toml .
    RUN cargo chef prepare --recipe-path recipe.json --bin $EXE_FOLDER
    SAVE ARTIFACT recipe.json

build-cache:
    COPY +prepare-cache/recipe.json ./
    RUN cargo chef cook --release --target x86_64-unknown-linux-musl
    SAVE ARTIFACT target
    SAVE ARTIFACT $CARGO_HOME cargo_home
    SAVE IMAGE --cache-hint


build:
    # Copy in all our crates
    COPY --dir crates crates
    COPY --dir Cargo.lock Cargo.toml .
    COPY +build-cache/cargo_home $CARGO_HOME
    COPY +build-cache/target target
    COPY --dir +npm-build/dist $PIPELINE_FOLDER/
    # We need to run inside docker as we need postgres running for cornucopia
    ARG DATABASE_URL=postgresql://postgres:testpassword@localhost:5432/postgres?sslmode=disable
    USER root
    WITH DOCKER \
        --pull postgres:alpine
        RUN docker run -d --rm --network=host -e POSTGRES_PASSWORD=testpassword postgres:alpine \
            && while ! pg_isready --host=localhost --port=5432 --username=postgres; do sleep 1; done ;\
                dbmate --migrations-dir $DB_FOLDER/migrations up \
            && cargo build --release --target x86_64-unknown-linux-musl --bin $EXE_NAME
    END
    SAVE ARTIFACT target/x86_64-unknown-linux-musl/release/$EXE_NAME

# The final stage after testing, build our tiny container
# hub.docker.com/r/purtontech/barricade
app-container:
    FROM scratch
    COPY +build/$EXE_NAME barricade
    ENTRYPOINT ["./barricade"]
    # We call the image build and let semantic release handle tagging and pushing latest
    SAVE IMAGE --push $CONTAINER_NAME

integration-test:
    FROM +build
    COPY .devcontainer/docker-compose.yml ./ 
    COPY .devcontainer/docker-compose.earthly.yml ./ 

    ARG SECRET_KEY=50fb08b06b381c575e60c56328f66a51822320e922c7e11e264a7bb443ee22fe
    ARG DATABASE_URL=postgresql://vscode:testpassword@localhost:5432/postgres?sslmode=disable

    # We expose selenium to localhost
    ARG WEB_DRIVER_URL='http://localhost:4444' 
    # The selenium container will connect to the envoy container
    ARG WEB_DRIVER_DESTINATION_HOST='http://app:9096' 

    USER root
    WITH DOCKER \
        # Bring up the containers we have built
        --load $CONTAINER_NAME=+app-container \
        --compose docker-compose.yml \
        --compose docker-compose.earthly.yml \
        --service db \
        --service smtp \
        --service whoami \
        # Record our selenium session
        --service selenium \
        --service app \
        --pull selenium/video:ffmpeg-4.3.1-20220208

        # Force to command to always be succesful so the artifact is saved. 
        # https://github.com/earthly/earthly/issues/988

        RUN while ! pg_isready --host=localhost --port=5432 --username=postgres; do sleep 1; done ;\
            dbmate --migrations-dir $DB_FOLDER/migrations up \
            && cargo test --no-run --release --target x86_64-unknown-linux-musl \
            && docker run -d --name video --network=default_default -e DISPLAY_CONTAINER_NAME=build_selenium_1 -e FILE_NAME=chrome-video.mp4 -v /build/tmp:/videos selenium/video:ffmpeg-4.3.1-20220208 \
            && (cargo test --release --target x86_64-unknown-linux-musl -- --nocapture || echo fail > ./tmp/fail) \
            && docker stop video
    END
    # You need the tmp/* if you use just tmp earthly will overwrite the folder
    SAVE ARTIFACT tmp/* AS LOCAL ./tmp/earthly/