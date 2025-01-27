CREATE TABLE WebauthnUserCredential
(
    user_id BIGINT UNSIGNED        NOT NULL,
    cred_id VARBINARY(1023) UNIQUE NOT NULL,
    cred    JSON                   NOT NULL,
    PRIMARY KEY (user_id, cred_id),
    FOREIGN KEY (user_id) REFERENCES User (id) ON DELETE CASCADE
);

CREATE TABLE WebauthnPasskeyRegistration
(
    user_id    BIGINT UNSIGNED NOT NULL,
    reg_id     CHAR(32) UNIQUE NOT NULL,
    reg_state  JSON            NOT NULL,
    expires_at TIMESTAMP       NOT NULL DEFAULT (TIMESTAMPADD(SECOND, 300, NOW())),
    PRIMARY KEY (user_id, reg_id),
    FOREIGN KEY (user_id) REFERENCES User (id) ON DELETE CASCADE
);

CREATE TABLE WebauthnAuthState
(
    user_id    BIGINT UNSIGNED NOT NULL,
    auth_id    CHAR(32) UNIQUE NOT NULL,
    state      JSON            NOT NULL,
    expires_at TIMESTAMP       NOT NULL DEFAULT (TIMESTAMPADD(SECOND, 300, NOW())),
    PRIMARY KEY (user_id, auth_id),
    FOREIGN KEY (user_id) REFERENCES User (id) ON DELETE CASCADE
);

CREATE EVENT webauthn_cleanup
    ON SCHEDULE EVERY 1 DAY
    DO
    BEGIN
        DELETE FROM WebauthnPasskeyRegistration WHERE expires_at <= NOW();
        DELETE FROM WebauthnAuthState WHERE expires_at <= NOW();
    END;
