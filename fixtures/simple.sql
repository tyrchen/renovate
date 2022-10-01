CREATE SCHEMA tenant;
ALTER SCHEMA tenant OWNER TO superadmin;

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

CREATE SCHEMA auth;
ALTER SCHEMA auth OWNER TO superadmin;
CREATE TABLE auth.users (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    aud character varying(255),
    role character varying(64),
    email character varying(255),
    encrypted_password character varying(255),
    confirmed_at timestamp with time zone,
    invited_at timestamp with time zone,
    confirmation_token character varying(255),
    confirmation_sent_at timestamp with time zone,
    recovery_token character varying(255),
    recovery_sent_at timestamp with time zone,
    email_change_token character varying(255),
    email_change character varying(255),
    email_change_sent_at timestamp with time zone,
    last_sign_in_at timestamp with time zone,
    raw_app_meta_data jsonb,
    raw_user_meta_data jsonb,
    is_super_admin boolean,
    created_at timestamp with time zone,
    updated_at timestamp with time zone
);

ALTER TABLE auth.users OWNER TO autoapi_auth_admin;
COMMENT ON TABLE auth.users IS 'Auth: Stores user login data within a secure schema.';

ALTER TABLE ONLY auth.users
    ADD CONSTRAINT users_email_key UNIQUE (email);

ALTER TABLE ONLY auth.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);

ALTER TABLE ONLY tenant.instances
    ADD CONSTRAINT instances_name_fkey FOREIGN KEY (name) REFERENCES tenant.tenants(name) ON DELETE CASCADE;

ALTER TABLE ONLY tenant.tenants
    ADD CONSTRAINT tenants_owner_id_fkey FOREIGN KEY (owner_id) REFERENCES auth.users(id) ON DELETE RESTRICT;
