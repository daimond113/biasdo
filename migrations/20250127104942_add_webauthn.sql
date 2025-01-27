CREATE TABLE WebauthnUserCredential(
    user_id BIGINT UNSIGNED NOT NULL,
    cred_id VARBINARY(1023) NOT NULL,
    cred JSON NOT NULL,
    PRIMARY KEY (user_id, cred_id),
    FOREIGN KEY (user_id) REFERENCES User (id) ON DELETE CASCADE
);

CREATE TABLE WebauthnPasskeyRegistration(
    user_id BIGINT UNSIGNED NOT NULL,
    reg_id CHAR(32) NOT NULL,
    reg_state JSON NOT NULL,
    PRIMARY KEY (user_id, reg_id),
    FOREIGN KEY (user_id) REFERENCES User (id) ON DELETE CASCADE
)