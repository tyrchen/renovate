-- CREATE TABLE tenant.tenants (
--     name character varying(64) NOT NULL,
--     owner_id uuid,
--     status tenant.tenant_status DEFAULT 'free'::tenant.tenant_status NOT NULL,
--     next_billing_at timestamp with time zone,
--     created_at timestamp with time zone DEFAULT now() NOT NULL,
--     updated_at timestamp with time zone DEFAULT now() NOT NULL,
--     CONSTRAINT tenants_name_check CHECK (((name)::text ~* '^[a-z][a-z0-9]{5,}$'::text))
-- );
-- CREATE TABLE foo (id serial not null primary key, name text default random_name(), CHECK (check_name(name)))

CREATE TABLE public.foo (
    id integer NOT NULL,
    name text DEFAULT 'hello'::text NOT NULL
);
