ALTER TABLE ONLY test.message ALTER COLUMN id SET DEFAULT nextval('test.message_id_seq'::regclass);
