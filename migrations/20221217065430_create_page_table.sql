-- Add migration script here
CREATE TABLE page (
    id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    title TEXT NOT NULL UNIQUE,
    summary TEXT,
    content TEXT NOT NULL,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NOT NULL DEFAULT current_timestamp
);