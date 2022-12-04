# Rewrite `create table`

- Feature Name: rewrite-create-table
- Proposal Date: 2022-10-04 21:40:31
- Start Date: (date)

## Summary

We shall support `renovate format` to rewrite `create table` statements to a compatible format that `pg_dump` uses. For example:

```sql
CREATE TABLE foo (
    id1 int generated always as identity,
    id2 serial not null primary key check((id2>5)),
    name text default 'tyrchen',
    CHECK (name ~* '^[a-z][a-z0-9]{5,}$')
);
```

shall be rewritten to:

```sql
CREATE TABLE public.foo (
    id1 integer NOT NULL,
    id2 integer NOT NULL,
    name text DEFAULT 'tyrchen'::text,
    CONSTRAINT foo_id2_check CHECK ((id2 > 5)),
    CONSTRAINT foo_name_check CHECK ((name ~* '^[a-z][a-z0-9]{5,}$'::text))
);
ALTER TABLE public.foo OWNER TO CURRENT_USER;

ALTER TABLE public.foo ALTER COLUMN id1 ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.foo_id1_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);

CREATE SEQUENCE public.foo_id2_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER TABLE public.foo_id2_seq OWNER TO CURRENT_USER;
ALTER SEQUENCE public.foo_id2_seq OWNED BY public.foo.id2;
ALTER TABLE ONLY public.foo ALTER COLUMN id2 SET DEFAULT nextval('public.foo_id2_seq'::regclass);
ALTER TABLE ONLY public.foo ADD CONSTRAINT foo_pkey PRIMARY KEY (id2);
```

## Motivation

Better compatibility with `pg_dump`.

## Guide-level explanation

To achieve this, we need to alter the AST of `create table`. High level thoughts:

1. if column level has constraints other than Not Null and Default, we move those constraints to table level and give then a name. For example, `check((id>5))` becomes `CONSTRAINT foo_id_check CHECK ((id > 5))`.
2. if table level has constraints other than Check, we move those constraints to `alter table` statements. For example, `primary key (id)` becomes `ALTER TABLE ONLY public.foo ADD CONSTRAINT foo_pkey PRIMARY KEY (id);`

Note we won't support constraints like `GENERATED`
## Reference-level explanation

## Drawbacks

## Rationale and alternatives

## Prior art

## Unresolved questions

## Future possibilities
