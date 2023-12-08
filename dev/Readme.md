# AlertmanagerExt

## Postgres Database

```bash
docker run -d --name postgres_alertmanager -e POSTGRES_USER=user -e POSTGRES_PASSWORD=password -e POSTGRES_DB=database -p 127.0.0.1:5432:5432 -v ${PWD}/dev/data/postgres_data:/var/lib/postgresql/data postgres

docker run -d --name postgres_x_alertmanager -e POSTGRES_USER=user -e POSTGRES_PASSWORD=password -e POSTGRES_DB=database -p 127.0.0.1:5433:5432 -v ${PWD}/dev/data/postgres_x_data:/var/lib/postgresql/data postgres

docker run -d --name postgres_sea_alertmanager -e POSTGRES_USER=user -e POSTGRES_PASSWORD=password -e POSTGRES_DB=database -p 127.0.0.1:5434:5432 -v ${PWD}/dev/data/postgres_sea_data:/var/lib/postgresql/data postgres

docker run -d --name mysql_ox_alertmanager -e MYSQL_USER=user -e MYSQL_PASSWORD=password -e MYSQL_ROOT_PASSWORD=password -e MYSQL_DATABASE=database -p 127.0.0.1:3306:3306 -v ${PWD}//dev/data/mysql_ox_data:/var/lib/mysql mysql
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
