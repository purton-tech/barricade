FROM ianpurton/rust-fullstack-devcontainer:latest

ARG EXE_NAME=barricade
ARG FOLDER=.
ARG CONTAINER_NAME=purtontech/barricade
ARG SELENIUM=selenium/standalone-chrome:4.1.1-20220121

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
    SAVE IMAGE --cache-hint

integration-test:
    FROM +build
    COPY --dir $FOLDER/tests .
    COPY --dir migrations .
    COPY +build/$EXE_NAME ./$EXE_NAME
    ARG DATABASE_URL=postgresql://postgres:testpassword@localhost:5432
    
    # Env vars used by the integration tests
    ARG WEB_DRIVER_URL=http://localhost:4444
    ARG WEB_DRIVER_DESTINATION_HOST=http://localhost:9095
    
    # Env vars for the app
    ARG SECRET_KEY=50fb08b06b381c575e60c56328f66a51822320e922c7e11e264a7bb443ee22fe
    ARG FORWARD_URL=localhost
    ARG FORWARD_PORT=80
    ARG REDIRECT_URL=/
    ARG ENABLE_EMAIL_OTP='true'
    ARG ENABLE_HEADLESS=1
    ARG PORT=9095
    ARG USER_TABLE_NAME=bcrypt_users
    ARG WEB_DRIVER_DESTINATION_HOST=http://localhost:9096
    
    USER root
    WITH DOCKER \
        --pull postgres:alpine \
        --pull containous/whoami \
        # Record our selenium session
        --pull $SELENIUM
        RUN \
            docker run --name whoami -d --rm --network=host containous/whoami \
            # Run up postgres
            && docker run -d --rm --network=host -e POSTGRES_USER=postgres -e POSTGRES_PASSWORD=testpassword postgres:alpine \
            # Run the database migrations
            && while ! pg_isready --host=localhost --port=5432 --username=postgres; do sleep 1; done ;\
                diesel migration run \
            && chmod +x ./$EXE_NAME \
            # Now the database is up start the exe in normal mode
            && ./$EXE_NAME & \
            # Start exe in encrypted mode
            PORT=9096 USER_TABLE_NAME=keypair_users AUTH_TYPE=encrypted ./$EXE_NAME & \
            # Run up selenium for browser testing.
            docker run -d --rm --network=host --name selenium --shm-size="2g" $SELENIUM \
            # Finally run the browser testing
            && cargo test --release --target x86_64-unknown-linux-musl -- --nocapture \
            && ls -la /build/tmp 
    END
    SAVE ARTIFACT /build/tmp AS LOCAL ./tmp/earthly
    SAVE ARTIFACT ./$EXE_NAME $EXE_NAME
    SAVE IMAGE --cache-hint

# The final stage after testing, build our tiny container
app-container:
    FROM scratch
    COPY +integration-test/$EXE_NAME barricade
    COPY --dir +npm-build/dist asset-pipeline/dist
    EXPOSE 8080
    ENTRYPOINT ["./barricade"]
    # We call the image build and let semantic release handle tagging and pushing latest
    SAVE IMAGE --push $CONTAINER_NAME:build
