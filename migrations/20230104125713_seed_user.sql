-- Add migration script here
INSERT INTO users (user_id, username, password_hash)
VALUES (
    'a4ee7112-ac8d-446f-8578-fc4fccd55ee5',
    'Tobias',
    '$argon2id$v=19$m=15000,t=2,p=1$Ms06Tueq8qAV1wQVTp1hbw$rrsboumIG/0Ir+AU1wuO4oHylx4uKy7NnIWU8L3Rn04'
);
