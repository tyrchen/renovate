# SQL migration

- Feature Name: sql-migration
- Proposal Date: 2022-09-30 18:38:20
- Start Date: (date)

## Summary

DB migration should be as easy as possible. Users should only need change their db schema as changing the code, and the migration system should take care of the rest, just like terraform.

## Motivation

Existing solutions:

1. Use normal database migrations. This is a pretty bad experience since engineer need to keep track of what the database look like at this point. And from a list of migration files, it's super hard.
2. projects like [atlas](https://github.com/ariga/atlas). It tried to mimic the terraform experience, but given that SQL itself is a declarative language, why bother creating a new one that developers/DBAs need to learn?

Since the existing solutions are not good enough, we need to create a new one.

## Guide-level explanation

User could use the tool like this:

```bash
# dump all the schemas into a folder
$ renovate init --source postgres://user@localhost:5432/hello
Database schema has successfully dumped into ./hello.

# if schema already exists, before modifying it, it is always a good practice to fetch the latest schema. Fetch will fail if current folder is not under git or it is not up to date with remote repository.
$ renovate fetch

# do whatever schema changes you want

# then run plan to see what changes will be applied. When redirect to a file, it will just print all the SQL statements for the migration.
$ renovate plan
Table auth.users changed:

create table users(
    id uuid primary key,
    name text not null,
    email text not null,
    password text not null,
-   created_at timestamptz not null,
+   created_at timestamptz not null default now(),
+   updated_at timestamptz not null
);

The following SQLs will be applied:

    alter table users add column updated_at timestamptz not null;
    alter table users alter column created_at set default now();

# then apply the changes
$ renovate apply
Your repo is dirty. Please commit the changes before applying.

$ git commit -a -m "add updated_at column and set default value for created_at"

# now you can directly apply
# apply can use -p to run a previously saved plan or manually edited plan
# the remove schema and the plan being executed will be saved in _meta/plans/202109301022/.
$ renovate apply

The following SQLs will be applied:

    alter table users add column updated_at timestamptz not null;
    alter table users alter column created_at set default now();

Continue (y/n)? y
Successfully applied migration to postgres://user@localhost:5432/hello.
Your repo is updated with the latest schema. See `git diff HEAD~1` for details.
```

Note that not all changes could generate proper migration SQLs. Currently we only support to generate the following migration SQLs:

- create table
- alter table add/drop column
- alter table alter column set default
- alter table add/drop constraint
- grant/revoke privilege

We will gradually support more and more migrations. If certain schema changes are not supported (e.g. a table is completely removed or column type is changed), we will print a warning and ask the user to manually write the migration SQLs.

Once the migration is applied, the code will be updated to the latest schema automatically.

## Reference-level explanation

Postgres supports pg_get_xxx functions to retrieve DDL for view and function

When loading remote schema to the local directories, we will create subdirecties for each schema, types/tables/views/functions/triggers directories under the schema directory if exists. Each types/table/view/function/trigger will be stored in a separate file. The file name will be the table/view/function/trigger name. The file content will be the SQL to create the type/table/view/function/trigger.

Upon `renovate plan`, we will compare the local schema with the remote schema. The comparison algorithm looks like this:

1. use pg_dump to dump the remote schema into a temporary file, then load it to DatabaseSchema struct. The DatabaseSchema and corresponding structs are parsed from each sql statement. We use [sqlparser](https://github.com/sqlparser-rs/sqlparser-rs) to do the parsing work and `From<SqlStatement>` trait to convert SqlStatement to our own structs.
2. load the local repo into DatabaseSchema struct
3. Compare each data structure to find out: 1) newly added 2) removed 3) changed.
4. Based on the comparison result, generate the SQL statements to apply the changes.

```rust
pub struct DatabaseSchema {
    pub schemas: BTreeMap<String, Schema>,
}

pub struct Schema {
    pub types: BTreeMap<String, DataType>,
    pub tables: BTreeMap<String, Table>,
    pub views: BTreeMap<String, View>,
    pub functions: BTreeMap<String, Function>,
    pub triggers: BTreeMap<String, Trigger>,
}

pub struct DataType {

}

pub struct Table {
    pub columns: BTreeMap<String, Column>,
    pub constraints: BTreeMap<String, Constraint>,
    pub privileges: BTreeMap<String, Privilege>,
}

pub struct View {
    // for view definition, if it changed we will just drop and recreate it
    // shall we verify if the SQL is valid?
    pub sql: String,
    pub constraints: BTreeMap<String, Constraint>,
    pub privileges: BTreeMap<String, Privilege>,
}

pub struct Function {
    // for function definition, if it changed we will just drop and recreate it
    // shall we verify if the SQL is valid?
    pub sql: String,
    pub privileges: BTreeMap<String, Privilege>,
}

pub struct Trigger {
    // for trigger definition, if it changed we will just drop and recreate it
    // shall we verify if the SQL is valid?
    pub sql: String,
}
```

For each

```rust
pub trait Differ {
    fn text_diff(&self, remote: &Self) -> Vec<Diff>;
    fn ast_diff(&self, remote: &Self) -> Vec<Diff>;
}
```

```rust
pub trait Planner {
    fn diff(&self, remote: &Self) -> Vec<Diff>;
    fn plan(&self, diff: &[Diff]) -> Vec<Plan>;
}
```

When applying the migration, we will first check if the local schema is up to date with the remote schema. If not, we will print a warning and ask the user to run `renovate init` to update the local schema. Then we will apply the migration SQLs to the remote database.

This is the technical portion of the RFC. Explain the design in sufficient detail that:

- Its interaction with other features is clear.
- It is reasonably clear how the feature would be implemented.
- Corner cases are dissected by example.

The section should return to the examples given in the previous section, and explain more fully how the detailed proposal makes those examples work.

## Drawbacks

Why should we *not* do this?

## Rationale and alternatives

- Why is this design the best in the space of possible designs?
- What other designs have been considered and what is the rationale for not choosing them?
- What is the impact of not doing this?

## Prior art

Discuss prior art, both the good and the bad, in relation to this proposal.
A few examples of what this can include are:

- For language, library, cargo, tools, and compiler proposals: Does this feature exist in other programming languages and what experience have their community had?
- For community proposals: Is this done by some other community and what were their experiences with it?
- For other teams: What lessons can we learn from what other communities have done here?
- Papers: Are there any published papers or great posts that discuss this? If you have some relevant papers to refer to, this can serve as a more detailed theoretical background.

This section is intended to encourage you as an author to think about the lessons from other languages, provide readers of your RFC with a fuller picture.
If there is no prior art, that is fine - your ideas are interesting to us whether they are brand new or if it is an adaptation from other languages.

Note that while precedent set by other languages is some motivation, it does not on its own motivate an RFC.
Please also take into consideration that rust sometimes intentionally diverges from common language features.

## Unresolved questions

- What parts of the design do you expect to resolve through the RFC process before this gets merged?
- What parts of the design do you expect to resolve through the implementation of this feature before stabilization?
- What related issues do you consider out of scope for this RFC that could be addressed in the future independently of the solution that comes out of this RFC?

## Future possibilities

Think about what the natural extension and evolution of your proposal would
be and how it would affect the language and project as a whole in a holistic
way. Try to use this section as a tool to more fully consider all possible
interactions with the project and language in your proposal.
Also consider how this all fits into the roadmap for the project
and of the relevant sub-team.

This is also a good place to "dump ideas", if they are out of scope for the
RFC you are writing but otherwise related.

If you have tried and cannot think of any future possibilities,
you may simply state that you cannot think of anything.

Note that having something written down in the future-possibilities section
is not a reason to accept the current or a future RFC; such notes should be
in the section on motivation or rationale in this or subsequent RFCs.
The section merely provides additional information.
