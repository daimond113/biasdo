CREATE TABLE User
(
    id                BIGINT UNSIGNED PRIMARY KEY,
    -- ci is very important for case-insensitive comparison in the unique constraint
    username          VARCHAR(32) COLLATE utf8mb4_unicode_ci  NOT NULL UNIQUE,
    display_name      VARCHAR(32),
    password          VARCHAR(128)                            NOT NULL,
    email             VARCHAR(255) COLLATE utf8mb4_unicode_ci NOT NULL UNIQUE,
    email_verified    BOOLEAN                                 NOT NULL DEFAULT FALSE,
    began_deletion_at TIMESTAMP
);

CREATE INDEX User_username ON User (username);

CREATE TABLE UserFriendRequest
(
    sender_id   BIGINT UNSIGNED NOT NULL,
    receiver_id BIGINT UNSIGNED NOT NULL,
    created_at  TIMESTAMP       NOT NULL DEFAULT NOW(),
    PRIMARY KEY (sender_id, receiver_id),
    FOREIGN KEY (sender_id) REFERENCES User (id) ON DELETE CASCADE,
    FOREIGN KEY (receiver_id) REFERENCES User (id) ON DELETE CASCADE
);

CREATE TABLE UserFriend
(
    user_id    BIGINT UNSIGNED NOT NULL,
    friend_id  BIGINT UNSIGNED NOT NULL,
    created_at TIMESTAMP       NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, friend_id),
    FOREIGN KEY (user_id) REFERENCES User (id) ON DELETE CASCADE,
    FOREIGN KEY (friend_id) REFERENCES User (id) ON DELETE CASCADE
);

CREATE TABLE UserSession
(
    id         CHAR(64) PRIMARY KEY,
    user_id    BIGINT UNSIGNED NOT NULL,
    created_at TIMESTAMP       NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP       NOT NULL DEFAULT (TIMESTAMPADD(DAY, 30, NOW())),
    FOREIGN KEY (user_id) REFERENCES User (id) ON DELETE CASCADE
);

CREATE INDEX UserSession_user_id ON UserSession (user_id);
CREATE INDEX UserSession_expires_at ON UserSession (expires_at);

CREATE TABLE Client
(
    id         BIGINT UNSIGNED PRIMARY KEY,
    name       VARCHAR(24)     NOT NULL,
    secret     CHAR(64) UNIQUE,
    owner_id   BIGINT UNSIGNED NOT NULL,
    client_uri VARCHAR(255),
    tos_uri    VARCHAR(255),
    policy_uri VARCHAR(255),
    FOREIGN KEY (owner_id) REFERENCES User (id) ON DELETE CASCADE
);

CREATE TABLE ClientRedirect
(
    client_id BIGINT UNSIGNED NOT NULL,
    uri       VARCHAR(255)    NOT NULL,
    PRIMARY KEY (client_id, uri),
    FOREIGN KEY (client_id) REFERENCES Client (id) ON DELETE CASCADE
);

CREATE TABLE ClientToken
(
    access_token CHAR(66) PRIMARY KEY,
    client_id    BIGINT UNSIGNED                                                                                                                            NOT NULL,
    created_at   TIMESTAMP                                                                                                                                  NOT NULL DEFAULT NOW(),
    expires_at   TIMESTAMP                                                                                                                                  NOT NULL DEFAULT (TIMESTAMPADD(MINUTE, 10, NOW())),
    scope        SET ('profile.read', 'profile.write', 'servers.read', 'servers.write', 'messages.read', 'messages.write', 'friends.read', 'friends.write') NOT NULL,
    FOREIGN KEY (client_id) REFERENCES Client (id) ON DELETE CASCADE
);

CREATE INDEX ClientToken_expires_at ON ClientToken (expires_at);

CREATE TABLE AuthorizationCode
(
    id                    CHAR(32) PRIMARY KEY,
    created_at            TIMESTAMP                                                                                                                                  NOT NULL DEFAULT NOW(),
    expires_at            TIMESTAMP                                                                                                                                  NOT NULL DEFAULT (TIMESTAMPADD(MINUTE, 10, NOW())),
    client_id             BIGINT UNSIGNED                                                                                                                            NOT NULL,
    user_id               BIGINT UNSIGNED                                                                                                                            NOT NULL,
    scope                 SET ('profile.read', 'profile.write', 'servers.read', 'servers.write', 'messages.read', 'messages.write', 'friends.read', 'friends.write') NOT NULL,
    code_challenge        VARCHAR(128),
    code_challenge_method ENUM ('plain', 'S256')                                                                                                                     NOT NULL,
    FOREIGN KEY (client_id) REFERENCES Client (id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES User (id) ON DELETE CASCADE
);

CREATE INDEX AuthorizationCode_expires_at ON AuthorizationCode (expires_at);

CREATE TABLE ClientUserTokens
(
    user_id           BIGINT UNSIGNED                                                                                                                            NOT NULL,
    client_id         BIGINT UNSIGNED                                                                                                                            NOT NULL,
    created_at        TIMESTAMP                                                                                                                                  NOT NULL DEFAULT NOW(),
    access_expires_at TIMESTAMP                                                                                                                                  NOT NULL DEFAULT (TIMESTAMPADD(MINUTE, 10, NOW())),
    expires_at        TIMESTAMP                                                                                                                                  NOT NULL DEFAULT (TIMESTAMPADD(DAY, 30, NOW())),
    auth_code         CHAR(32) UNIQUE,
    access_token      CHAR(66)                                                                                                                                   NOT NULL UNIQUE,
    refresh_token     CHAR(66)                                                                                                                                   NOT NULL UNIQUE,
    scope             SET ('profile.read', 'profile.write', 'servers.read', 'servers.write', 'messages.read', 'messages.write', 'friends.read', 'friends.write') NOT NULL,
    PRIMARY KEY (user_id, client_id),
    FOREIGN KEY (user_id) REFERENCES User (id) ON DELETE CASCADE,
    FOREIGN KEY (client_id) REFERENCES Client (id) ON DELETE CASCADE,
    FOREIGN KEY (auth_code) REFERENCES AuthorizationCode (id) ON DELETE SET NULL
);

CREATE INDEX ClientUserTokens_access_token ON ClientUserTokens (access_token, access_expires_at);
CREATE INDEX ClientUserTokens_refresh_token ON ClientUserTokens (refresh_token);
CREATE INDEX ClientUserTokens_expires_at ON ClientUserTokens (expires_at);

CREATE TABLE Server
(
    id       BIGINT UNSIGNED PRIMARY KEY,
    name     VARCHAR(32)     NOT NULL,
    owner_id BIGINT UNSIGNED NOT NULL,
    FOREIGN KEY (owner_id) REFERENCES User (id) ON DELETE CASCADE
);

CREATE TABLE ServerMember
(
    server_id  BIGINT UNSIGNED NOT NULL,
    user_id    BIGINT UNSIGNED NOT NULL,
    created_at TIMESTAMP       NOT NULL DEFAULT NOW(),
    nickname   VARCHAR(32),
    PRIMARY KEY (server_id, user_id),
    FOREIGN KEY (server_id) REFERENCES Server (id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES User (id) ON DELETE CASCADE
);

CREATE TABLE Channel
(
    id        BIGINT UNSIGNED PRIMARY KEY,
    name      VARCHAR(32)         NOT NULL,
    kind      ENUM ('text', 'DM') NOT NULL,
    server_id BIGINT UNSIGNED,
    FOREIGN KEY (server_id) REFERENCES Server (id) ON DELETE CASCADE
);

CREATE TABLE DMChannelRecipient
(
    channel_id BIGINT UNSIGNED NOT NULL,
    user_id    BIGINT UNSIGNED NOT NULL,
    PRIMARY KEY (channel_id, user_id),
    FOREIGN KEY (channel_id) REFERENCES Channel (id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES User (id) ON DELETE CASCADE
);

CREATE TABLE ChannelMessage
(
    id         BIGINT UNSIGNED PRIMARY KEY,
    updated_at TIMESTAMP,
    content    TEXT            NOT NULL,
    kind       ENUM ('text')   NOT NULL,
    channel_id BIGINT UNSIGNED NOT NULL,
    user_id    BIGINT UNSIGNED NOT NULL,
    FOREIGN KEY (channel_id) REFERENCES Channel (id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES User (id) ON DELETE CASCADE
);

CREATE TABLE ServerInvite
(
    id         CHAR(24) PRIMARY KEY,
    created_at TIMESTAMP       NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP       NOT NULL DEFAULT (TIMESTAMPADD(DAY, 7, NOW())),
    server_id  BIGINT UNSIGNED NOT NULL,
    FOREIGN KEY (server_id) REFERENCES Server (id) ON DELETE CASCADE
);

CREATE INDEX Invite_expires_at ON ServerInvite (expires_at);

CREATE EVENT expires_at_cleanup
    ON SCHEDULE EVERY 1 DAY
    DO
    BEGIN
        DELETE FROM UserSession WHERE expires_at <= NOW();
        DELETE FROM ClientToken WHERE expires_at <= NOW();
        DELETE FROM AuthorizationCode WHERE expires_at <= NOW();
        DELETE FROM ClientUserTokens WHERE expires_at <= NOW();
        DELETE FROM ServerInvite WHERE expires_at <= NOW();
    END;

CREATE EVENT user_deletion_cleanup
    ON SCHEDULE EVERY 8 HOUR
    DO
    BEGIN
        DELETE User, Channel
        FROM User
                 INNER JOIN DMChannelRecipient ON DMChannelRecipient.user_id = User.id
                 INNER JOIN Channel ON Channel.id = DMChannelRecipient.channel_id
        WHERE User.began_deletion_at <= NOW() - INTERVAL 7 DAY;
    END;