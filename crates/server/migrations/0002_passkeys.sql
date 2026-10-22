-- The one learner this server belongs to. WebAuthn needs a stable user id.
CREATE TABLE owner (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    user_id TEXT NOT NULL -- UUID
);

-- The learner's passkeys. `passkey` is webauthn_rs::Passkey as JSON.
CREATE TABLE passkeys (
    id INTEGER PRIMARY KEY,
    cred_id TEXT NOT NULL UNIQUE,
    passkey TEXT NOT NULL,
    created_at INTEGER NOT NULL, -- unix milliseconds
    last_used_at INTEGER
);
