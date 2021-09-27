FROM ianpurton/rust-fullstack-devcontainer:latest

ARG EXE_NAME=authn-proxy
ARG FOLDER=.
ARG CONTAINER_NAME=authnproxy/authnproxy

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
    EXPOSE 8080
    ENTRYPOINT ["./rust-exe"]
    SAVE IMAGE --push $CONTAINER_NAME:latest

integration-test:
    FROM +build
    COPY --dir $FOLDER/tests .
    COPY --dir migrations .
    COPY +build/$EXE_NAME ./rust-exe
    ARG DATABASE_URL=postgresql://postgres:testpassword@localhost:5432
    # Enc vrs picked up by the tests
    #ARG WEB_DRIVER_URL=http://localhost:4444/wd/hub
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
    USER root
    WITH DOCKER \
        --load $CONTAINER_NAME:latest=+docker
        RUN \
            docker run --name whoami -d --rm --network=host containous/whoami \
            # Run up postgres
            && docker run -d --rm --network=host -e POSTGRES_USER=postgres -e POSTGRES_PASSWORD=testpassword postgres:alpine \
            # Run the database migrations
            && while ! pg_isready --host=localhost --port=5432 --username=postgres; do sleep 1; done ;\
                diesel migration run \

            # Now the database is up start the exe
            && chmod +x ./rust-exe && PORT=9095  ./rust-exe & \
            # Run up selenium for browser testing.
            docker run -d --rm --network=host --shm-size="2g" selenium/standalone-chrome:4.0.0-rc-2-prerelease-20210916 \
            
            # Finally run the browser testing
            && WEB_DRIVER_DESTINATION_HOST=http://localhost:9095 \
                cargo test --release --target x86_64-unknown-linux-musl -- --nocapture \

            # Run integration tests encrypted
            && PORT=9096 USER_TABLE_NAME=keypair_users AUTH_TYPE=encrypted ./rust-exe & \
            WEB_DRIVER_DESTINATION_HOST=http://localhost:9096 \
                cargo test --release --target x86_64-unknown-linux-musl -- --nocapture
    END
