![Docker Pulls](https://img.shields.io/docker/pulls/contorsystems/contor?style=plastic)

**Contor** is a proxy you put in front of your application. It then intercepts requests and will show a logon or registration page to the user. Contor uses your database and requires minimal configuration.

## Features 

* A "No Code" solution. Zero integration.
* You can replace your existing auth solution and you get to remove code from your app. 
* Data in your region in your database and under your control.
* Small high performance Docker container built with Rust.
* Configure with environment variables.
* Works well as a Kubernetes side car.
* Works with envoy proxy as an auth module.
* Encrypted Mode - Generates ECDH and ECDSA keys client side. Uses a similar technique to Bitwarden but upgraded.
* Generates secure HTTP only cookies encrypted with GCM and additional data.
* Can replace rails devise or other popular open source auth libraries.
* TODO - WebAuthn for 2FA
* TODO - Allow user to see sessions and logout devices.

## Contor defends against the following attacks

* Account enumeration 
* Password stuffing
* Password Brute forcing
* Session Hijacking
* Data breach password brute forcing.

## Try it out 

To show how quickly you can get going with Contor let's add authentication to a small web app called [Whoami](https://hub.docker.com/r/containous/whoami). Whoami is a tiny Go webserver that prints os information and HTTP request to output and is avaible on docker hub. 

To get started cut and paste the following definition into a file called `docker-compose.yml` 

```yaml
version: '3.4'
services:

  # First let's create a postgres database
  db:
    image: postgres:alpine
    environment:
      POSTGRES_PASSWORD: testpassword
      POSTGRES_USER: postgres
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 10s
      timeout: 5s
      retries: 5

  # As an example we'll add login/registration to 
  # https://hub.docker.com/r/containous/whoami
  whoami:
    image: containous/whoami

  # Configure contor to listen on port 9090 and reverse proxy nginx on port 80.
  contor:
    image: contorsystems/contor
    environment:
        # This secret key is used to encrypt cookies.
        SECRET_KEY: 190a5bf4b3cbb6c0991967ab1c48ab30790af876720f1835cbbf3820f4f5d949
        DATABASE_URL: postgresql://postgres:testpassword@db:5432
        FORWARD_URL: whoami
        FORWARD_PORT: 80
        # Any requests that meet the following regulkar expressions
        # with pass through. i.e. They don't require auth.
        SKIP_AUTH_FOR: /api*
        REDIRECT_URL: '/'
    ports:
      - "9091:9090"
    depends_on:
      db:
        condition: service_healthy
```

## Bring up the services

```console
docker-compose up
```

And you should get output on your console like the following.

<p align="center">
  <img src="./.github/assets/compose-output.png" width="100%" />
</p>

## Intercept user requests

If you now head to `http://localhost:9091/api` in your browser you should see the following output from Whoami.

<p align="center">
  <img src="./.github/assets/api.png" width="100%" />
</p>

        
We set our environment variable `SKIP_AUTH_FOR` to `/api*` which means we allow people to access the `/api` end point without being authenticated.

## Access an endpoint that requires authentication.


Access `localhost:9091` and you'll see contor block your request.

<p align="center">
  <img src="./.github/assets/login-large.png" width="100%" />
</p>

## Add a user table

We need to add a user table to our database. Run the psql command line from docker-compose

```console
docker-compose run db psql postgres://postgres:testpassword@db:5432
```

Once you have the psql command prompt you can cut and paste the following code to create a users and sessions table.

```sql
CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE users (
    id SERIAL PRIMARY KEY, 
    email VARCHAR NOT NULL UNIQUE, 
    hashed_password VARCHAR NOT NULL, 
    reset_password_token UUID,
    reset_password_sent_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE sessions (
    id SERIAL PRIMARY KEY, 
    session_uuid UUID NOT NULL DEFAULT gen_random_uuid(), 
    user_id INT NOT NULL, 
    otp_code_encrypted VARCHAR NOT NULL;
    otp_code_attempts INTEGER NOT NULL DEFAULT 0;
    otp_code_confirmed BOOLEAN NOT NULL DEFAULT false;
    otp_code_sent BOOLEAN NOT NULL DEFAULT false;
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),

    CONSTRAINT fk_user
        FOREIGN KEY(user_id) 
        REFERENCES users(id)
);
```

<p align="center">
  <img src="./.github/assets/psql-output.png" width="100%" />
</p>

## Register a new user

Go to `localhost:9091/auth/sign_up` in your browser and register.

<p align="center">
  <img src="./.github/assets/registration.png" width="100%" />
</p>

After registration you'll be take to the whoami home page as we set `REDIRECT_URL` to `/`.

You'll then be greated with 


<p align="center">
  <img src="./.github/assets/post-registration-whoami.png" width="100%" />
</p>

There's two important things to note from the whoami screenshot.

* The Cookie header `session=` is stored client side and is an encrypted cookie based on `SECRET_KEY`
* A header called `x-user-id` is passed in. This is the primary key of the user in the Users table. Your app can retrieve this header to access the logged in user. This is not stored on the client it's only passed from Contor to your application.

## Finally take a look in the database

Run the following command in the psql shell to see the your new user registration in the database.

```console
docker-compose run db psql postgres://postgres:testpassword@db:5432
```


```console
psql=# select * from users;
 id |        email         |                                         hashed_password                                          |         created_at         |         updated_at         | reset_password_token | reset_pa
ssword_sent_at 
----+----------------------+--------------------------------------------------------------------------------------------------+----------------------------+----------------------------+----------------------+---------
---------------
  6 | some_user@gmail.com  | $argon2id$v=19$m=4096,t=3,p=1$kf7KkCPIIfbl1dgasa58yQ$WFNVT05c2ptBtBAZQI1CIGaEqbR9m3505WiZ+/oflk0 | 2021-09-07 11:03:35.663096 | 2021-09-07 11:03:35.663096 |                      | 
(1 row)
```

This is how your user table looks after a user registration.