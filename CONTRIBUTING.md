
## Setup for Development

This project uses the [Visual Studio Code Remote - Containers](https://code.visualstudio.com/docs/remote/containers) extension so we can define the runtime and development stack with code. The configuration is in the `.devcontainer` folder and uses a `docker-compose` configuration to setup the tools and database ready for development.

Make sure you have Docker Desktop installed and Visual Studio Code Remote. Make sure you have the Remote Containers extension installed. 

<p align="center">
  <img src="./.github/assets/dev-containers.png" width="100%" />
</p>

After you have run `git clone` on this repository open the folder for the project in Visual Studio Code.

Then click on the green square in the bottom left hand corner of VSCode. (It's the gree square with < and > in the screenshot above). A menu pops down, choose `Remote-Containers: Reopen in Container`

It will take a while for the containers to download.

## Sanity check your dev environment.

Open up the terminal using Use the `View > Terminal` menu command or ``CTRL/CMD ` ``

You can type the following commands in the Linux command prompt.

* rustc --version
* npm -v 
* psql -V

## Running Database Migrations

According to wikipedia

> In software engineering, schema migration (also database migration, database change management) refers to the management of incremental, reversible changes and version control to relational database schemas. A schema migration is performed on a database whenever it is necessary to update or revert that database's schema to some newer or older version.

> Migrations are performed programmatically by using a schema migration tool. When invoked with a specified desired schema version, the tool automates the successive application or reversal of an appropriate sequence of schema changes until it is brought to the desired state.

We are using the [Diesel database migration tool](https://docs.rs/diesel_migrations/1.4.0/diesel_migrations/) for our migrations which are stored in the `migrations` folder.

Run `diesel migration list` to see a list of the migrations we have created so far.

To apply the migrations run `diesel migration run`. You should see all the tables being created as below.

<p align="center">
  <img src="./.github/assets/run-migrations.png" width="100%" />
</p>

You can log into PSQL and see the tables for yourself by running `psql $DATABASE_URL`

<p align="center">
  <img src="./.github/assets/psql-users.png" width="100%" />
</p>

## Setup env dot file

Copy the following into a file called `.env`

```
SECRET_KEY=50fb08b06b381c575e60c56328f66a51822320e922c7e11e264a7bb443ee22fe
FORWARD_URL=whoami
FORWARD_PORT=80
REDIRECT_URL=/
PORT=9095
USER_TABLE_NAME=bcrypt_users
```

## Building Contor

For a full stack web development you need an [asset pipeline](https://www.theodinproject.com/courses/ruby-on-rails/lessons/the-asset-pipeline). This is responsible for compiling Typescript to Javascript, Compiling our SCSS and managing dependencies.

To compile our asset pipeline run 

* `npm install` To install all our front end dependencies
* `npm run start` To build all our assets. This will sit around waiting for any assets to change and will re-compile if thats the case. You can leave it running.

Build and run the [Actix Web](https://github.com/actix/actix-web) server.

* Open another terminal.
* `cargo watch -x fmt -x clippy -x run`

This will compile the rust code and will re-compile the code when it detects changes.

You should now be able to access the web application from `localhost:9095' in your browser. Hopefully it looks like the screenshot below.

<p align="center">
  <img src="./.github/assets/login-large.png" width="100%" />
</p>

## Running the browser tests (Integration Testing)

The Dev container has a selenium standalone headless chrome browser for testing.

To run the integration tests

`cargo test`

## Debugging browser tests with the browser on your host machine

If you want to see the tests running you can install chromedriver on your host machine https://chromedriver.chromium.org/downloads

From your host machine 

```
chromedriver --whitelisted-ips=""
```

The tests use an env var called `WEB_DRIVER_URL` to locate selenium (or any webdriver). To point this at the debug server on your host create a `.env` file.

```
echo 'WEB_DRIVER_URL=http://host.docker.internal:9515' >> .env
echo 'DISABLE_HEADLESS=true' >> .env
```


Run `cargo test` and chrome should popup and you'll see the browser automated.