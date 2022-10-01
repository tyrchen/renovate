CREATE TABLE tenant.tenants (
    name character varying(64) NOT NULL,
    owner_id uuid,
    status tenant.tenant_status DEFAULT 'free'::tenant.tenant_status NOT NULL,
    next_billing_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT tenants_name_check CHECK (((name)::text ~* '^[a-z][a-z0-9]{5,}$'::text))
);

CREATE TABLE tenant.instances (
    name character varying(64) NOT NULL,
    env character varying(16) DEFAULT 'dev'::character varying NOT NULL,
    status tenant.instance_status DEFAULT 'ready'::tenant.instance_status NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT instances_env_check CHECK (((env)::text ~* '^[a-z][a-z0-9]{2,}$'::text))
);
