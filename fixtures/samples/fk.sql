ALTER TABLE ONLY tenant.instances
    ADD CONSTRAINT instances_name_fkey FOREIGN KEY (name) REFERENCES tenant.tenants(name) ON DELETE CASCADE;
