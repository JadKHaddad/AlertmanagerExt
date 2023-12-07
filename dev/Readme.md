# AlertmanagerExt

## Postgres Database

```bash
docker run -d --name postgres_alertmanager -e POSTGRES_USER=user -e POSTGRES_PASSWORD=password -e POSTGRES_DB=database -p 127.0.0.1:5432:5432 -v ${PWD}/postgres_data:/var/lib/postgresql/data postgres

docker run -d --name postgres_x_alertmanager -e POSTGRES_USER=user -e POSTGRES_PASSWORD=password -e POSTGRES_DB=database -p 127.0.0.1:5433:5432 -v ${PWD}/postgres_x_data:/var/lib/postgresql/data postgres
```

```bash
pgcli postgresql://user:password@localhost:5432/database
```

## GPG -_-

```bash
export GPG_TTY=$(tty)
```

## Diesel Cli

```bash
cargo install diesel_cli --no-default-features --features "postgres sqlite-bundled"
```
