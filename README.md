[![Build Status](https://dev.azure.com/ianpurton0711/onchain.io/_apis/build/status/contor-systems.contor?branchName=master)](https://dev.azure.com/ianpurton0711/onchain.io/_build/latest?definitionId=18&branchName=master) ![Docker Pulls](https://img.shields.io/docker/pulls/contorsystems/contor?style=plastic)

**Contor** is a proxy you put in front of your application. It then intercepts requests and will show a logon or registration page to the user. Contor uses your database and requires minimal configuration.

## How it looks

<p align="center">
  <img src="./docs/images/login.png" width="400" />
  <img src="./docs/images/registration.png" width="400" />
</p>

## Features 

* Only requires you to add a users table to your application.
* Small high performance docker container.
* No code to write just configure with environment variables.
* Works well as a Kubernetes side car.
* TODO - Password reset.
* TODO - U2F
* TODO - 

## Try it out 

Cut and paste the following into a docker-compose.yml

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

  # As an example we'll add login/registration to an nginx install
  nginx:
    image: nginx:stable-alpine

  # Configure contor to listen on port 9090 and reverse proxy nginx on port 80.
  contor:
    image: contorsystems/contor
    environment:
        # This secret key is used to encrypt cookies.
        SECRET_KEY: 190a5bf4b3cbb6c0991967ab1c48ab30790af876720f1835cbbf3820f4f5d949
        DATABASE_URL: postgresql://postgres:testpassword@db:5432
        FORWARD_URL: nginx
        FORWARD_PORT: 80
        REDIRECT_URL: '/'
    ports:
      - "9090:9090"
    depends_on:
      db:
        condition: service_healthy
```

Bring up the services

```console
docker-compose up
```

We need to add a user table to our database. Run the psql command line from docker-compose

```console
docker-compose run db psql postgres://postgres:testpassword@localhost:5432
```

Once you have the psql command prompt you can cut and paste the following code to create a users table.

```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY, 
    email VARCHAR NOT NULL UNIQUE, 
    hashed_password VARCHAR NOT NULL, 
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);
```

Go to localhost:9090 in your browser and register. You should be taken to the nginx default page.

Run the following command in the psql shell to see yourself in the database.

```sql
SELECT * FROM users;
```