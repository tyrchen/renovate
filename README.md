# Renovate: A new way to handle SQL migration

WARNING: This project still lacks of many features. It is not ready for production use. Feel free to try it for your local development and let me know what migration features you need. Please be noted some of the generated migrations are not safe to apply. If you have better ideas on how those migrations should be, please submit an issue.

Renovate is a CLI tool to help you to work on Postgres SQL migration easily.

Example:

```console,ignore
➜ renovate schema init postgres://localhost:5432/test
➜ cat public/tables.sql
CREATE TABLE public.todos (title text, completed boolean);⏎
➜ cat > public/tables.sql
CREATE TABLE public.todos (title text, completed boolean, created_at tstz default now());
➜ renovate schema plan
Table public.todos is changed:

1        |-CREATE TABLE public.todos (title text, completed boolean)
    1    |+CREATE TABLE public.todos (
    2    |+    title text,
    3    |+    completed boolean,
    4    |+    created_at tstz DEFAULT NOW()
    5    |+)

The following SQLs will be applied:

  ALTER TABLE public.todos ADD COLUMN created_at tstz DEFAULT NOW();
```

For more information, see the [initial thoughts](./rfcs/0001-sql-migration.md).

Below is a simple demo:

[![asciicast](https://asciinema.org/a/N7Pd3gDPGFcpCddREJKAKTtbx.svg)](https://asciinema.org/a/N7Pd3gDPGFcpCddREJKAKTtbx)
