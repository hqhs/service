# DB models for the project

Crate uses diesel for DB access. Install diesel_cli:

```bash
cargo install diesel_cli
```

if you run into an error like: 

```bash
note: ld: library not found for -lmysqlclient
clang: error: linker command failed with exit code 1 (use -v to see invocation)
```

This means you are missing the client library needed for a database backend – mysqlclient in this case. You can resolve this issue by either installing the library (using the usual way to do this depending on your operating system) or by excluding the undesired default library with the --no-default-features flag.

By default diesel CLI depends on the following client libraries:

    libpq for the PostgreSQL backend
    libmysqlclient for the Mysql backend
    libsqlite3 for the SQLite backend

If you are not sure on how to install those dependencies please consult the documentation of the corresponding dependency or your distribution package manager.

For example, if you only have PostgreSQL installed, you can use this to install diesel_cli with only PostgreSQL:

```bash
cargo install diesel_cli --no-default-features --features postgres
```

Or with multiple database backends: 

```bash
cargo install diesel_cli --no-default-features --features "postgres sqlite"
```


## adding new migration

```bash
diesel migration generate create_posts
```
