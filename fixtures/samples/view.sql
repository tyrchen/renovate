CREATE VIEW api_catalog.schemas AS
 SELECT (pg_namespace.oid)::integer AS id,
    (pg_namespace.nspname)::character varying AS name,
    (pg_namespace.nspowner)::integer AS owner
   FROM pg_namespace
  WHERE ((pg_namespace.nspname <> ALL (ARRAY['pg_catalog'::name, 'information_schema'::name, 'pg_toast'::name, 'public'::name])) AND (pg_namespace.nspname !~ '^tn_'::text));
