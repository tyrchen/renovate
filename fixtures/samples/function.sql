CREATE OR REPLACE FUNCTION auth.uid(name text, age integer)
RETURNS uuid
LANGUAGE SQL stable
AS $$
	SELECT coalesce(
		current_setting('request.jwt.claim.sub', true),
		(current_setting('request.jwt.claims', true)::jsonb ->> 'sub')
	)::uuid
$$;
