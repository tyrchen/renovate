CREATE POLICY "Can only view own audit data." ON audit.logged_actions FOR SELECT USING ((auth.uid() = ((user_info ->> 'sub'::text))::uuid));
