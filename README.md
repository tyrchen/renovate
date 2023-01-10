# Renovate: A new way to handle Postgres schema migration

Database schema designs will evolve as the products they support evolves over time. It is important to be able to migrate the schema safely and reliably to incorporate the product changes.

Traditionally,  migration systems like ActiveRecord or sqlx will allow you to write migration files to describe what you want to execute to move the database from the current state to a new state. For example, if you want to add a `created_at` column to the todos table, you need to write a migration script first:

```sql
ALTER TABLE todos ADD COLUMN created_at timestamptz DEFAULT NOW();
```

The system will track what migration files were applied by using a migration table like this:

```sql
CREATE TABLE public._sqlx_migrations (
    version bigint NOT NULL PRIMARY KEY,
    description text NOT NULL,
    installed_on timestamp WITH time zone DEFAULT NOW() NOT NULL,
    success boolean NOT NULL,
    CHECKSUM bytea NOT NULL,
    execution_time bigint NOT NULL
);
```

When you apply the above migration script successfully, a new record will be added to the migration table to indicate it has been applied. The next time you run the migration, the system will check the migration table and skip the migration script if it has been applied before.

This approach is reliable but it defers the burden of writing migration scripts to the developers. Sometimes the migration scripts are not easy to write, especially when the schema is complex and the change is not straightforward. It is also hard to review the migration scripts since the reviewers need to understand what the current state is and what the new state it wants to achieve.

Newer migration systems like [atlas](https://github.com/ariga/atlas) can *understand* database schemas and generate migrations for you based on their understanding. To achieve this, normally the system needs to know the local state that users want to transit to and the remote state that is currently running in the database. Once the system gathered both states, it could *diff* them to decide what changes are needed. This approach is widely used by the tools to manage cloud resources, for example, [terraform](https://www.terraform.io/). It is more convenient for developers, and tools like terraform proved their reliability and safety.

Renovate is a tool that falls into the second category. Unlike atlas, it doesn't use a new language (say HCL) to describe the local state (UPDATE: [atlas 0.9.0 add SQL as a first class citizen](https://atlasgo.io/blog/2023/01/05/atlas-v090)). Instead, it uses the existing SQL DDL to describe the schema state. You could use `renovate schema init` to start a new project from an existing database. Renovate will retrieve the schema (if any) from the database server and properly organize the SQLs in different folders in a newly created git repo. And this will be your local state. You can do whatever modifications you'd like to do to it. Once you're satisfied with the schema, you can use `renovate schema plan` to get the migration plan. Renovate will use `pg_dump` to retrieve the remote state from the database server, and then diff the AST between the local state and the remote state to find out the right migration plan. If you're satisfied with the migration plan, you can apply it to the database server via `renovate schema apply`.

The benefit of using Renovate:

1. Declarative schema definition.
2. Just use SQL DDL. You don't need to learn a new language.
3. Renovate doesn't limit you when you introduce the tool. You can use it to start a new database project, or you can use it to migrate an existing database project. You don't need to start from scratch. You don't need to do any hacks for the existing database.
4. Renovate won't limit you on what tool you use. Feel free to use other tools or approaches to migrate database schema while you're using Renovate. You just need to do a simple `renovate schema fetch` to update the local state whenever you updated the database schema outside Renovate. Then you're good to go. No hacks are needed.

Below is an example:

Example:

```console,ignore
➜ renovate schema init postgres://localhost:5432/test
➜ cat public/04_tables.sql
CREATE TABLE public.todos (title text, completed boolean);⏎
➜ cat > public/04_tables.sql
CREATE TABLE public.todos (title text, completed boolean, created_at timestamptz default now());
➜ renovate schema plan
Table public.todos is changed:

1        |-CREATE TABLE public.todos (title text, completed boolean)
    1    |+CREATE TABLE public.todos (
    2    |+    title text,
    3    |+    completed boolean,
    4    |+    created_at timestamptz DEFAULT NOW()
    5    |+)

The following SQLs will be applied:

  ALTER TABLE public.todos ADD COLUMN created_at timestamptz DEFAULT NOW();
```

If that inspires you, here's a more detailed demo:

![demo](docs/images/demo.gif)

WARNING: This project still lacks many features. It is not ready for production use yet. Please be noted some of the generated migrations (e.g. changing fields in composite type) are not safe to apply at this moment. If you have better ideas on how those migrations should be, please submit an issue.

## How it works

Under the hood, Renovate uses [pg_query](https://github.com/pganalyze/pg_query.rs) to parse the Postgres SQL DDL to AST and uses [pg_dump](https://www.postgresql.org/docs/current/app-pgdump.html) to retrieve the remote state from the database server. The below figure shows the workflow of Renovate:

![arch](docs/images/renovate.png)

For more information, see the [initial thoughts](./rfcs/0001-sql-migration.md). Or you can also check the [architecture](./docs/architecture.md).

## Installation

Currently, Renovate only support installation from source:

```console,ignore
$ cargo install renovate
```

## Renovate CLI

```console
$ renovate schema
? 2
renovate-schema[..]
Schema migration

USAGE:
    renovate schema [OPTIONS] <SUBCOMMAND>

OPTIONS:
        --drop-on-exit    drop database on exit (for testing purpose only)
    -h, --help            Print help information

SUBCOMMANDS:
    apply        apply the migration plan to the remote database server
    fetch        fetch the most recent schema from the remote database server
    help         Print this message or the help of the given subcommand(s)
    init         init a database migration repo
    normalize    normalize local schema via a temp local database
    plan         diff the local change and remote state, then make a migration plan

```

## Folder structure for local state

Once you run `renovate schema init`, it will create a git repo with a folder structure like below example based on what it retrieved from the database server:

```console,ignore
➜ tree
.
├── public
│   └── 04_tables.sql
├── renovate.yml
└── rsvp
    ├── 02_enums.sql
    ├── 03_sequences.sql
    ├── 04_tables.sql
    └── 07_functions.sql

2 directories, 6 files
```

As you can see, the folder structure is organized by the schema name. The files are named with a prefix number to indicate the order of execution. For example, if you have a table `todos` and a table `rsvp.users`, the `public/04_tables.sql` will contain`todos` and `rsvp/04_tables.sql` will contain `rsvp.users`.

Renovate is very opinionated on how files should be laid out. If you create a random file (or use any existing SQL file) with the following SQL code:

```sql
CREATE TABLE hello.world (title text, completed boolean);
```

Once you `renovate schema apply` the change, Renovate will remove the code from the file and move it to `hello/04_tables.sql` (also delete any unnecessary files) since it's the right place for it to be.

## What has been supported

- [x] Type
  - [x] composite type add/remove
  - [ ] composite type change (destructive change only)
  - [x] enum type add/remove
  - [x] enum type add values
  - [x] enum type rename value (limited 1 rename at a time)
  - [ ] enum type change values (destructive change only)
- [x] Table
  - [x] column add/remove
  - [x] column type change
  - [x] column constraint change (default, not null, unique, check)
  - [x] table constraint add/remove/change
  - [x] table index add/remove/change
  - [x] table trigger add/remove/change
  - [x] table RLS
  - [x] table policy add/remove/change
  - [x] table owner change
- [x] View add/remove/change
- [x] Materialized view add/remove/change
- [x] Function add/remove/change
- [x] Sequence add/remove/change
- [x] Privilege add/remove/change

## FAQ

Q: How to use Renovate to roll back my schema change?

A: Unlike traditional schema migration tools, Renovate doesn't have a concept of "rollback". You could just change the schema back to the desired state (e.g. `git reset`), and then run `renovate schema plan` to get the migration plan as usual. Then you could apply the migration plan to the remote database server.

Q: What if my change to the schema is not supported?

A: Please submit an issue to let us know. We will try to support it as soon as possible. Meanwhile, you could manually apply that change to the remote database, then run `renovate schema fetch` to update the local state.

Q: Can I use Renovate as a library?

A: Yes, you can include it as a dependency in your project. Please exclude `cli` feature if you just want to use the core. But it is not recommended to use it as a library at this moment. The API is not stable yet.

Q: What is the plan or roadmap for Renovate?

A: I don't have a roadmap now. I need to get as much feedback as possible from you. My priority, for now, is to make it stable and reliable. The project has a decent number of unit tests (57 unit tests + 1 CLI test) at the time of writing, but it still lacks coverage for many scenarios. I haven't produced any user guide yet, and it is important.

## License

Renovate is released under an MIT license. See [LICENSE](./LICENSE) for details.
