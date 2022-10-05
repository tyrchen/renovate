# Rewrite `create table`

- Feature Name: rewrite-create-table
- Proposal Date: 2022-10-04 21:40:31
- Start Date: (date)

## Summary

We shall support `renovate format` to rewrite `create table` statements to a compatible format that `pg_dump` uses. For example:

```sql
CREATE TABLE foo (
      id serial not null primary key check((id>5)),
      name text default 'tyrchen',
      CHECK (name ~* '^[a-z][a-z0-9]{5,}$')
 );
```

shall be rewritten to:

```sql
CREATE TABLE foo (
    id integer NOT NULL,
    name text DEFAULT 'tyrchen'::text,
    CONSTRAINT foo_id_check CHECK ((id > 5)),
    CONSTRAINT foo_name_check CHECK ((name ~* '^[a-z][a-z0-9]{5,}$'::text))
);
CREATE SEQUENCE public.foo_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;
ALTER TABLE public.foo_id_seq OWNER TO tchen;
ALTER SEQUENCE public.foo_id_seq OWNED BY public.foo.id;
ALTER TABLE ONLY public.foo ALTER COLUMN id SET DEFAULT nextval('public.foo_id_seq'::regclass);
ALTER TABLE ONLY public.foo ADD CONSTRAINT foo_pkey PRIMARY KEY (id);
```

## Motivation

## Guide-level explanation

## Reference-level explanation

## Drawbacks

## Rationale and alternatives

## Prior art

## Unresolved questions

## Future possibilities
