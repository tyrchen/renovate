CREATE TRIGGER tenant_instance_modified_time_trigger BEFORE UPDATE ON tenant.instances FOR EACH ROW EXECUTE FUNCTION extensions.moddatetime('updated_at');
