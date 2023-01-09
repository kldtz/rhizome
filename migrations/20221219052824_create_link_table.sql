-- Add migration script here
CREATE TABLE link (
    source INT NOT NULL,
    target INT NOT NULL,
    PRIMARY KEY (source, target),
    bidirectional BOOLEAN NOT NULL,
    CONSTRAINT fk_source
        FOREIGN KEY(source)
            REFERENCES page(id)
            ON DELETE CASCADE,
    CONSTRAINT fk_target
        FOREIGN KEY (target)
            REFERENCES page(id)
            ON DELETE CASCADE
);
