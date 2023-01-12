-- Add migration script here
CREATE TABLE users (
    user_id uuid PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at timestamp NOT NULL DEFAULT current_timestamp
);
