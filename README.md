# Renovate: A new way to handle SQL migration

Renovate is a CLI tool to help you to work on Postgres SQL migration easily.

Example:

```bash
➜ renovate pg init postgres://localhost:5432/test
➜ cat public/tables.sql
CREATE TABLE public.todos (title text, completed boolean);⏎
➜ cat > public/tables.sql
CREATE TABLE public.todos (title text, completed boolean, created_at tstz default now());
➜ renovate pg plan
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
