ALTER TABLE Channel
    MODIFY kind ENUM ('Text', 'DM') NOT NULL,
    MODIFY server_id BIGINT UNSIGNED;

CREATE TABLE ChannelRecipient
(
    channel_id BIGINT UNSIGNED NOT NULL,
    user_id    BIGINT UNSIGNED NOT NULL,
    PRIMARY KEY (channel_id, user_id),
    FOREIGN KEY (channel_id) REFERENCES Channel (id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES User (id) ON DELETE CASCADE
);

ALTER TABLE Message
    ADD COLUMN user_id BIGINT UNSIGNED;

UPDATE Message
SET user_id = (SELECT user_id FROM Member WHERE Member.id = Message.member_id);

ALTER TABLE Message
    DROP FOREIGN KEY Message_ibfk_2,
    DROP COLUMN member_id,
    ADD FOREIGN KEY (user_id) REFERENCES User (id) ON DELETE SET NULL;