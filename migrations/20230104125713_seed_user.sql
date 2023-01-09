-- Add migration script here
INSERT INTO users (user_id, username, password_hash)
VALUES (
    'a4ee7112-ac8d-446f-8578-fc4fccd55ee5',
    'admin',
    '$argon2id$v=19$m=15000,t=2,p=1$RGfCsEhp71MBtlKF3XBAEw$+yK88CdbTCbtEk9usmsxshZ95qn/hlcJdkQ8mb77ioM'
);
