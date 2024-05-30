// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { Channel } from "./Channel";
import type { Invite } from "./Invite";
import type { Message } from "./Message";
import type { Server } from "./Server";
import type { ServerMember } from "./ServerMember";
import type { UserFriend } from "./UserFriend";
import type { UserFriendRequest } from "./UserFriendRequest";

export type WsUpdateEvent = { "type": "reauthenticate" } | { "type": "server_create", "data": Server } | { "type": "server_update", "data": { id: `${number}`, name: string | null, } } | { "type": "server_delete", "data": { id: `${number}`, } } | { "type": "channel_create", "data": Channel } | { "type": "channel_update", "data": { id: `${number}`, name: string | null, } } | { "type": "channel_delete", "data": { id: `${number}`, } } | { "type": "message_create", "data": Message } | { "type": "message_update", "data": { id: `${number}`, updated_at: string, content: string | null, } } | { "type": "message_delete", "data": { id: `${number}`, } } | { "type": "invite_create", "data": Invite } | { "type": "invite_delete", "data": { id: string, } } | { "type": "member_create", "data": ServerMember } | { "type": "member_update", "data": { user_id: `${number}`, server_id: `${number}`, nickname: string | null | null, } } | { "type": "member_delete", "data": { user_id: `${number}`, server_id: `${number}`, } } | { "type": "user_update", "data": { id: `${number}`, username: string | null, display_name: string | null | null, } } | { "type": "friend_request_create", "data": UserFriendRequest } | { "type": "friend_request_delete", "data": { sender_id: `${number}`, receiver_id: `${number}`, } } | { "type": "friend_create", "data": UserFriend } | { "type": "friend_delete", "data": { user_id: `${number}`, friend_id: `${number}`, } };