CREATE TABLE User
(
    id         BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    created_at TIMESTAMP       NOT NULL DEFAULT NOW(),
    username   VARCHAR(16)     NOT NULL,
    password   CHAR(60) BINARY NOT NULL
);

CREATE INDEX User_username ON User (username);

CREATE TABLE Session
(
    id         CHAR(24) PRIMARY KEY,
    created_at TIMESTAMP       NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP       NOT NULL DEFAULT (TIMESTAMPADD(MONTH, 1, NOW())),
    user_id    BIGINT UNSIGNED NOT NULL,
    FOREIGN KEY (user_id) REFERENCES User (id) ON DELETE CASCADE
);

CREATE INDEX Session_expires_at ON Session (expires_at);

CREATE TABLE Server
(
    id         BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    created_at TIMESTAMP       NOT NULL DEFAULT NOW(),
    name       VARCHAR(32)     NOT NULL,
    owner_id   BIGINT UNSIGNED NOT NULL,
    FOREIGN KEY (owner_id) REFERENCES User (id) ON DELETE CASCADE
);

CREATE TABLE Member
(
    id          BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    created_at  TIMESTAMP       NOT NULL DEFAULT NOW(),
    server_id   BIGINT UNSIGNED NOT NULL,
    user_id     BIGINT UNSIGNED,
    nickname    VARCHAR(32),
    FOREIGN KEY (server_id) REFERENCES Server (id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES User (id) ON DELETE SET NULL
);

CREATE TABLE Channel
(
    id         BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    created_at TIMESTAMP       NOT NULL DEFAULT NOW(),
    name       VARCHAR(32)     NOT NULL,
    kind       ENUM ('Text')   NOT NULL,
    server_id  BIGINT UNSIGNED NOT NULL,
    FOREIGN KEY (server_id) REFERENCES Server (id) ON DELETE CASCADE
);

CREATE TABLE Message
(
    id         BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    created_at TIMESTAMP       NOT NULL DEFAULT NOW(),
    content    TEXT            NOT NULL,
    kind       ENUM ('Text')   NOT NULL,
    channel_id BIGINT UNSIGNED NOT NULL,
    member_id  BIGINT UNSIGNED NOT NULL,
    FOREIGN KEY (channel_id) REFERENCES Channel (id) ON DELETE CASCADE,
    FOREIGN KEY (member_id) REFERENCES Member (id) ON DELETE CASCADE
);

CREATE TABLE Invite
(
    id         BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    created_at TIMESTAMP       NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP       NOT NULL DEFAULT (TIMESTAMPADD(DAY, 7, NOW())),
    server_id  BIGINT UNSIGNED NOT NULL,
    FOREIGN KEY (server_id) REFERENCES Server (id) ON DELETE CASCADE
);

CREATE INDEX Invite_expires_at ON Invite (expires_at);