ALTER TABLE ONLY tenant.instances
    ADD CONSTRAINT instances_name_fkey FOREIGN KEY (name) REFERENCES tenant.tenants(name) ON DELETE CASCADE;

ALTER TABLE ONLY tenant.tenants
    ADD CONSTRAINT tenants_owner_id_fkey FOREIGN KEY (owner_id) REFERENCES auth.users(id) ON DELETE RESTRICT;
