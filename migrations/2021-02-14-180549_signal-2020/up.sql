----
-- 1. Rename X to X_old

ALTER TABLE sentq RENAME TO old_sentq;
ALTER TABLE message RENAME TO old_message;
ALTER TABLE session RENAME TO old_session;

-----
-- 2. Create the new structures

-- `recipients` contains the registered persons that we can talk to. Signal Android
-- interprets this table differently; they also consider a "group" as a recipient,
-- while we store groups as *separate* entities, and instead abstract over these
-- using the `session` table.
CREATE TABLE recipients (
    id INTEGER PRIMARY KEY NOT NULL,

    -- Recipient identification with Signal
    e164 VARCHAR(25) UNIQUE,
    uuid VARCHAR(36) UNIQUE,
    username TEXT UNIQUE,
    email TEXT UNIQUE,

    is_blocked BOOLEAN DEFAULT FALSE NOT NULL,

    -- Signal profile
    profile_key BLOB, -- Signal Android stores these as base64
    profile_key_credential BLOB,
    profile_given_name TEXT,
    profile_family_name TEXT,
    profile_joined_name TEXT,
    signal_profile_avatar TEXT, -- This is a pointer to the avatar, not the real thing.
    profile_sharing_enabled BOOLEAN DEFAULT FALSE NOT NULL,
    last_profile_fetch TIMESTAMP,

    unidentified_access_mode TINYINT DEFAULT 0 NOT NULL, -- 0 is UNKNOWN

    storage_service_id BLOB,
    storage_proto BLOB, -- This is set when an account update contains unknown fields

    capabilities INTEGER DEFAULT 0 NOT NULL, -- These are flags

    last_gv1_migrate_reminder TIMESTAMP,

    last_session_reset TIMESTAMP,

    -- Either e164 or uuid should be entered in recipients
    CHECK(NOT(e164 == NULL AND uuid == NULL))
);

-- Create index on UUID and e164 and other identifiers
CREATE INDEX recipient_e164 ON recipients(e164);
CREATE INDEX recipient_uuid ON recipients(uuid);
CREATE INDEX recipient_username ON recipients(username);
CREATE INDEX recipient_email ON recipients(email);

CREATE INDEX recipient_last_session_reset ON recipients(last_session_reset DESC);

-- The `v1_group` table contains the spontaneous V1 groups.
CREATE TABLE group_v1s (
    id VARCHAR(32) PRIMARY KEY NOT NULL, -- This is hex encoded. Sqlite has no HEX-decode.
    name TEXT NOT NULL
    -- Yes. Group V1 is that simple.
);

CREATE TABLE group_v1_members (
    group_v1_id VARCHAR(32) NOT NULL,
    recipient_id INTEGER NOT NULL,
    member_since TIMESTAMP, -- not sure whether we'll use this

    -- artificial primary key
    FOREIGN KEY(recipient_id) REFERENCES recipients(id), -- on delete RESTRICT because we shouldn't delete a group member because we don't like the receiver.
    PRIMARY KEY(group_v1_id, recipient_id)
);

-- The `sessions` table is a superclass of groupv1/groupv2/1:1 messages
-- When GroupV2 gets implemented, this table will be replaces once again, because
-- the constraints cannot be altered.
CREATE TABLE sessions (
    id INTEGER PRIMARY KEY NOT NULL,

    -- Exactly one of these two (later three with groupv2) should be filed
    direct_message_recipient_id INTEGER,
    group_v1_id VARCHAR(32),

    draft TEXT,

    -- Deleting recipients should be separate from deleting sessions. ON DELETE RESTRICT
    FOREIGN KEY(direct_message_recipient_id) REFERENCES recipients(id),
    FOREIGN KEY(group_v1_id) REFERENCES group_v1s(id),

    -- Either a session is dm, gv1 or gv2
    CHECK (NOT(direct_message_recipient_id == NULL AND group_v1_id == NULL))
);

-- The actual messages
CREATE TABLE messages (
    id INTEGER PRIMARY KEY NOT NULL,
    session_id INTEGER NOT NULL,
    text TEXT,

    -- for group messages, this refers to the sender.
    sender_recipient_id INTEGER,

    received_timestamp TIMESTAMP,
    sent_timestamp TIMESTAMP,
    server_timestamp TIMESTAMP,

    -- This `is_read` flag indicates that the local user read the incoming message.
    is_read BOOLEAN DEFAULT FALSE NOT NULL,
    is_outbound BOOLEAN NOT NULL,
    flags INTEGER NOT NULL,

    -- expiring messages
    -- NOT NULL means that the message gets deleted at `expires_in + expiry_started`.
    expires_in INTEGER,
    expiry_started TIMESTAMP,

    -- misc flags
    use_unidentified BOOLEAN DEFAULT FALSE NOT NULL,
    is_remote_deleted BOOLEAN DEFAULT FALSE NOT NULL,

    FOREIGN KEY(sender_recipient_id) REFERENCES recipients(id) ON DELETE CASCADE,
    FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

-- Index the timestamps of message
CREATE INDEX message_received ON messages(received_timestamp);
CREATE INDEX message_sent ON messages(sent_timestamp);
CREATE INDEX message_server ON messages(server_timestamp);

CREATE TABLE attachments (
    id INTEGER PRIMARY KEY NOT NULL,
    json TEXT,
    message_id INTEGER NOT NULL,
    content_type TEXT DEFAULT "" NOT NULL,
    name TEXT,
    content_disposition TEXT,
    content_location TEXT,
    attachment_path TEXT,
    is_pending_upload BOOLEAN DEFAULT FALSE NOT NULL,
    transfer_file_path TEXT,
    size INTEGER,
    file_name TEXT,
    unique_id TEXT,
    digest TEXT,
    is_voice_note BOOLEAN NOT NULL,
    is_borderless BOOLEAN NOT NULL,
    is_quote BOOLEAN NOT NULL,

    width INTEGER,
    height INTEGER,

    sticker_pack_id TEXT DEFAULT NULL,
    sticker_pack_key BLOB DEFAULT NULL,
    sticker_id INTEGER DEFAULT NULL,
    sticker_emoji TEXT DEFAULT NULL,

    data_hash BLOB,
    visual_hash TEXT,
    transform_properties TEXT,

    -- This is the encrypted file, used for resumable uploads (#107)
    transfer_file TEXT,
    display_order INTEGER DEFAULT 0 NOT NULL,
    -- default is timestamp of this migration.
    upload_timestamp TIMESTAMP DEFAULT "2021-02-14T18:05:49Z" NOT NULL,
    cdn_number INTEGER DEFAULT 0,

    FOREIGN KEY(message_id) REFERENCES messages(id) ON DELETE CASCADE,
    FOREIGN KEY(sticker_pack_id, sticker_id) REFERENCES stickers(pack_id, sticker_id) ON DELETE CASCADE
);

CREATE TABLE stickers (
    pack_id TEXT,
    sticker_id INTEGER NOT NULL,
    -- Cover is the ID of the sticker of this pack to be used as "cover".
    cover_sticker_id INTEGER NOT NULL,

    key BLOB NOT NULL,

    title TEXT NOT NULL,
    author TEXT NOT NULL,

    pack_order INTEGER NOT NULL,
    emoji TEXT NOT NULL,
    content_type TEXT,
    last_used TIMESTAMP NOT NULL,
    installed TIMESTAMP NOT NULL,
    file_path TEXT NOT NULL,
    file_length INTEGER NOT NULL,
    file_random BLOB NOT NULL,

    PRIMARY KEY(pack_id, sticker_id),
    FOREIGN KEY(pack_id, cover_sticker_id) REFERENCES stickers(pack_id, sticker_id) ON DELETE CASCADE,
    UNIQUE(pack_id, sticker_id, cover_sticker_id)
);


---
-- 3. Copy over the data

-- 4. Drop the old tables

DROP TABLE old_sentq;
DROP TABLE old_message;
DROP TABLE old_session;
