-- Blind is a user-directory visibility flag, not access to E2B G.k.2.5 data.
BEGIN;
SELECT set_current_user_context('00000000-0000-0000-0000-000000000001');

UPDATE users
SET access_blind_allowed = FALSE;

ALTER TABLE users
    ALTER COLUMN access_blind_allowed SET DEFAULT FALSE,
    ALTER COLUMN access_blind_allowed SET NOT NULL;

COMMENT ON COLUMN users.access_blind_allowed IS
    'Legacy name: TRUE hides the account from default user listings; unrelated to E2B G.k.2.5.';

COMMIT;
