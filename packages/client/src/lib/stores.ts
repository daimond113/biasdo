import {
	type Readable,
	type Writable,
	derived,
	get,
	readable,
	writable,
} from "svelte/store"
import BTreeMap from "sorted-btree"
import { getImageUrl } from "./images"
import { goto } from "$app/navigation"
import { page } from "$app/stores"

import type { Channel } from "@biasdo/server-utils/src/Channel"
import type { Invite } from "@biasdo/server-utils/src/Invite"
import type { Message } from "@biasdo/server-utils/src/Message"
import type { Server } from "@biasdo/server-utils/src/Server"
import type { ServerMember } from "@biasdo/server-utils/src/ServerMember"
import type { User } from "@biasdo/server-utils/src/User"
import type { UserFriend } from "@biasdo/server-utils/src/UserFriend"
import type { UserFriendRequest } from "@biasdo/server-utils/src/UserFriendRequest"
import type { WsUpdateEvent } from "@biasdo/server-utils/src/WsUpdateEvent"

export const currentServerId = derived(
	page,
	($page) => ($page.params.serverId as `${number}`) ?? null,
)
export const currentChannelId = derived(
	page,
	($page) => ($page.params.channelId as `${number}`) ?? null,
)

const updateHandlers = [] as ((event: WsUpdateEvent) => void)[]

const wsUpdateMessage = (handler: (event: WsUpdateEvent) => void) => {
	updateHandlers.push(handler)
}

const wsStore = writable<WebSocket | undefined>(undefined)
let reopenCount = 0
let reauthCount = 0

{
	const reopenTimeouts = [2000, 5000, 10000, 30000, 60000]

	const apiUrl = new URL(import.meta.env.VITE_API_URL)
	let reopenTimeout: number | undefined
	let openPromise: Promise<void> | undefined
	let firstConnect = true

	const getReopenTimeout = () => {
		const n = reopenCount
		reopenCount++
		return reopenTimeouts[
			n >= reopenTimeouts.length - 1 ? reopenTimeouts.length - 1 : n
		]
	}

	const open = () => {
		if (reopenTimeout) {
			clearTimeout(reopenTimeout)
			reopenTimeout = undefined
		}

		if (openPromise) return

		if (!get(page).url?.pathname?.startsWith("/app")) {
			// client side navigation won't cause the function to be reran, check in a loop
			reopenTimeout = setTimeout(open, 500)
			return
		}

		if (firstConnect) {
			firstConnect = false
			console.log("[WebSocket]: First connection attempt...")
		} else {
			console.log("[WebSocket]: Reconnecting...")
		}

		if (!localStorage.getItem("session")) {
			const shouldEnd = ++reauthCount >= 2
			console.error(
				`[WebSocket]: No session found, ${shouldEnd ? "no longer " : ""}retrying...`,
			)

			if (shouldEnd) {
				reauthCount = 0
				return goto("/")
			}

			reopenTimeout = setTimeout(open, getReopenTimeout())
			return
		}

		const ws = new WebSocket(
			`${apiUrl.protocol === "https:" ? "wss" : "ws"}://${apiUrl.host}/v0/ws`,
		)

		openPromise = new Promise<void>((resolve, reject) => {
			ws.onopen = () => {
				resolve()
				reopenCount = 0

				ws.send(
					JSON.stringify({
						type: "authenticate",
						data: localStorage.getItem("session"),
					}),
				)
			}

			ws.onerror = (event) => {
				reject(event)
			}
		})
			.then(() => {
				console.log("[WebSocket]: Connection opened")
				wsStore.set(ws)
			})
			.catch((event) => {
				console.error("[WebSocket]: Connection error", event)
			})
			.finally(() => {
				openPromise = undefined
			})

		ws.onmessage = (event) => {
			const data = JSON.parse(event.data) as WsUpdateEvent
			if (data.type === "reauthenticate") {
				ws.send(
					JSON.stringify({
						type: "authenticate",
						data: localStorage.getItem("session"),
					}),
				)

				return
			}

			updateHandlers.forEach((handler) => handler(data))
		}

		ws.onclose = (event) => {
			console.error(
				`[WebSocket]: Connection ${event.wasClean ? "closed cleanly" : "died"}`,
				event,
			)

			if (reopenTimeout) clearTimeout(reopenTimeout)
			reopenTimeout = setTimeout(open, getReopenTimeout())
		}
	}

	open()
}

type Collection<V> = BTreeMap<`${number}`, V>
type PartialNullable<T> = { [K in keyof T]?: T[K] | null }

const updateCollection = <
	K extends string,
	V,
	T extends PartialNullable<V> & { id: K },
>(
	collection: BTreeMap<K, V>,
	data: T,
	insertOnMissing = false,
): BTreeMap<K, V> => {
	const previous = collection.get(data.id)
	if (previous) {
		collection.set(data.id, { ...previous, ...data })
	} else if (insertOnMissing) {
		collection.set(data.id, data as unknown as V)
	}

	return collection
}

export const allServers = writable<Collection<Server>>(
	new BTreeMap(),
	(_, update) =>
		wsUpdateMessage((data) => {
			if (data.type === "server_create") {
				update((collection) => updateCollection(collection, data.data, true))
			} else if (data.type === "server_update") {
				update((collection) => updateCollection(collection, data.data))
			} else if (data.type === "server_delete") {
				if (get(currentServerId) === data.data.id) {
					goto("/app")
				}

				update((collection) => {
					collection.delete(data.data.id)
					return collection
				})
			}
		}),
)

export type APIMember = ServerMember & { user: User; id: `${number}-${number}` }

export const allMembers = writable<BTreeMap<`${number}-${number}`, APIMember>>(
	new BTreeMap(),
	(_, update) =>
		wsUpdateMessage((data) => {
			if (data.type === "member_create") {
				const id = `${data.data.server_id}-${data.data.user_id}` as const

				update((collection) =>
					updateCollection(collection, { ...data.data, id }, true),
				)

				const user = data.data.user
				if (user)
					allUsers.update((collection) =>
						updateCollection(collection, user, true),
					)
			} else if (data.type === "member_update") {
				const id = `${data.data.server_id}-${data.data.user_id}` as const

				update((collection) =>
					updateCollection(collection, { ...data.data, id }),
				)
			} else if (data.type === "member_delete") {
				const id = `${data.data.server_id}-${data.data.user_id}` as const

				update((collection) => {
					collection.delete(id)
					return collection
				})
			}
		}),
)

export type APIChannel = Channel & { recipients?: `${number}`[] }

export const allChannels = writable<Collection<APIChannel>>(
	new BTreeMap(),
	(_, update) =>
		wsUpdateMessage((data) => {
			if (data.type === "channel_create") {
				update((collection) =>
					updateCollection(
						collection,
						{
							...data.data,
							recipients:
								data.data.kind === "DM"
									? ([get(meId), data.data.user?.id] as never)
									: undefined,
						},
						true,
					),
				)

				const user = data.data.user
				if (user)
					allUsers.update((collection) =>
						updateCollection(collection, user, true),
					)
			} else if (data.type === "channel_update") {
				update((collection) => updateCollection(collection, data.data))
			} else if (data.type === "channel_delete") {
				update((collection) => {
					collection.delete(data.data.id)
					return collection
				})
			}
		}),
)

export const allInvites = writable<Map<string, Invite>>(
	new Map(),
	(_, update) =>
		wsUpdateMessage((data) => {
			if (data.type === "invite_create") {
				update((collection) => {
					const existing = collection.get(data.data.id)
					if (existing) {
						collection.set(data.data.id, { ...existing, ...data.data })
					} else {
						collection.set(data.data.id, data.data)
					}
					return collection
				})

				allServers.update((collection) =>
					updateCollection(collection, data.data.server, true),
				)
			} else if (data.type === "invite_delete") {
				update((collection) => {
					collection.delete(data.data.id)
					return collection
				})
			}
		}),
)

const newMessageNotification = (message: Message) => {
	if (message.user.id === get(meId)) return
	if (
		message.channel_id === get(currentChannelId) &&
		document.visibilityState === "visible"
	)
		return

	if (Notification.permission === "default") {
		Notification.requestPermission()
	}

	if (Notification.permission === "granted") {
		const name =
			message.member?.nickname ?? message.user?.username ?? "Deleted User"
		const channel = get(allChannels).get(message.channel_id)

		new Notification(
			`${name}${channel && channel.kind !== "DM" ? ` | #${channel.name}` : ""}`,
			{
				body: message.content,
				icon: getImageUrl("user", message.user),
			},
		).addEventListener("click", () => {
			if (!channel) return

			goto(
				channel.kind === "DM"
					? `/app/direct-messages/${channel.id}`
					: `/app/servers/${channel.server_id}/channels/${channel.id}`,
			)
		})
	}
}

export const allMessages = writable<Collection<Message>>(
	new BTreeMap(),
	(_, update) =>
		wsUpdateMessage((data) => {
			if (data.type === "message_create") {
				update((collection) => {
					const message = data.data
					newMessageNotification(message)
					return updateCollection(collection, message, true)
				})

				allUsers.update((collection) =>
					updateCollection(collection, data.data.user, true),
				)

				const member = data.data.member
				if (member)
					allMembers.update((collection) =>
						updateCollection(
							collection,
							{
								...member,
								id: `${member.server_id}-${member.user_id}` as const,
							},
							true,
						),
					)
			} else if (data.type === "message_update") {
				update((collection) => updateCollection(collection, data.data))
			} else if (data.type === "message_delete") {
				update((collection) => {
					collection.delete(data.data.id)
					return collection
				})
			}
		}),
)

export const allUsers = writable<Collection<User>>(
	new BTreeMap(),
	(_, update) =>
		wsUpdateMessage((data) => {
			if (data.type === "user_update") {
				update((collection) => updateCollection(collection, data.data))
			}
		}),
)

export const allFriendRequests = writable<
	Collection<UserFriendRequest & { id: `${number}` }>
>(new BTreeMap(), (_, update) =>
	wsUpdateMessage((data) => {
		if (data.type === "friend_request_create") {
			const id = [data.data.sender.id, data.data.receiver.id].find(
				(id) => id !== get(meId),
			)!

			update((collection) =>
				updateCollection(collection, { ...data.data, id }, true),
			)

			updateStore(allUsers, [data.data.sender, data.data.receiver])
		} else if (data.type === "friend_request_delete") {
			const id = [data.data.sender_id, data.data.receiver_id].find(
				(id) => id !== get(meId),
			)!

			update((collection) => {
				collection.delete(id)
				return collection
			})
		}
	}),
)

export const allFriends = writable<
	Collection<UserFriend & { id: `${number}` }>
>(new BTreeMap(), (_, update) =>
	wsUpdateMessage((data) => {
		if (data.type === "friend_create") {
			const id = [data.data.user.id, data.data.friend.id].find(
				(id) => id !== get(meId),
			)!

			update((collection) =>
				updateCollection(collection, { ...data.data, id }, true),
			)

			updateStore(allUsers, [data.data.user, data.data.friend])

			allChannels.update((collection) =>
				updateCollection(
					collection,
					{
						...data.data.channel,
						recipients: [data.data.user.id, data.data.friend.id],
					},
					true,
				),
			)
		} else if (data.type === "friend_delete") {
			const id = [data.data.user_id, data.data.friend_id].find(
				(id) => id !== get(meId),
			)!

			update((collection) => {
				collection.delete(id)
				return collection
			})
		}
	}),
)

export const updateStore = <K extends string, T extends { id: K }>(
	store: Writable<BTreeMap<K, T>>,
	newData: T[] | undefined,
) => {
	if (!newData || newData.length <= 0) return

	store.update((prev) => {
		for (const data of newData) {
			updateCollection(prev, data, true)
		}
		return prev
	})
}

export const memberId = <T extends ServerMember>(
	members?: T[],
): (T & { id: `${number}-${number}` })[] =>
	members?.map((m) => ({
		...m,
		id: `${m.server_id}-${m.user_id}` as const,
	})) as never

const friendRequestId = (
	friendRequests?: UserFriendRequest[],
): (UserFriendRequest & { id: `${number}` })[] =>
	friendRequests?.map((fr) => ({
		...fr,
		id: [fr.sender.id, fr.receiver.id].find((id) => id !== get(meId))!,
	})) as never

const friendId = (
	friends?: UserFriend[],
): (UserFriend & { id: `${number}` })[] =>
	friends?.map((f) => ({
		...f,
		id: [f.user.id, f.friend.id].find((id) => id !== get(meId))!,
	})) as never

export type TypedResponse<T> = Promise<
	Omit<Response, "json"> & { json: () => Promise<T> }
>

export const filterResponse = async <T>(
	response: Awaited<TypedResponse<T>>,
): Promise<T> => {
	const text = await response.text()
	let json
	try {
		json = JSON.parse(text)
	} catch {}

	if (!response.ok) {
		throw new Error(json ?? text, {
			code: response.status,
		})
	}

	return json
}

export const meId = writable<`${number}` | undefined>(undefined)

const awaitStore = <T>(store: Readable<T>) => {
	let unsubscribe: (() => void) | undefined

	return new Promise<NonNullable<T>>((resolve) => {
		unsubscribe = store.subscribe((value) => {
			if (value) {
				resolve(value)
			}
		})
	}).finally(() => unsubscribe?.())
}

export const populateStores = async (
	getData: () => {
		servers?: TypedResponse<Server[]>
		members?: TypedResponse<APIMember[]>
		channels?: TypedResponse<Channel[]>
		invites?: TypedResponse<Invite[] | Invite>
		messages?: TypedResponse<Message[]>
		users?: TypedResponse<User[]>
		me?: TypedResponse<User & { email: string; email_verified: boolean }>
		friendRequests?: TypedResponse<UserFriendRequest[]>
		friends?: TypedResponse<UserFriend[]>
	},
	immediate?: boolean,
): Promise<unknown> => {
	// prevent a race condition between the websocket and the initial data
	if (!immediate) await awaitStore(wsStore)
	const data = getData()

	const mePromise = // update the meId store first to prevent issues with other stores
		data.me?.then(filterResponse)?.then((me) => {
			meId.set(me.id)
			allUsers.update((prev) => updateCollection(prev, me, true))
		})

	if (!immediate)
		await Promise.race([mePromise, awaitStore(meId)].filter(Boolean))

	const promise = Promise.all([
		immediate && mePromise,
		data.servers
			?.then(filterResponse)
			?.then((servers) => updateStore(allServers, servers)),
		data.members?.then(filterResponse)?.then((members) => {
			updateStore(allMembers, memberId(members))
			updateStore(
				allUsers,
				members.map((m) => m.user),
			)
		}),
		data.channels?.then(filterResponse)?.then((channels) => {
			updateStore(
				allChannels,
				channels.map((c) => ({
					...c,
					recipients:
						c.kind === "DM" ? ([get(meId), c.user?.id] as never) : undefined,
				})),
			)
			updateStore(
				allUsers,
				channels.map((c) => c.user).filter(Boolean) as never,
			)
		}),
		data.messages?.then(filterResponse)?.then((messages) => {
			updateStore(allMessages, messages)
			updateStore(
				allUsers,
				messages.map((m) => m.user),
			)
			updateStore(
				allMembers,
				memberId(messages.map((m) => m.member).filter(Boolean) as never),
			)
		}),
		data.invites?.then(filterResponse)?.then((invites) => {
			if (Array.isArray(invites)) {
				updateStore(allInvites as never, invites)
				updateStore(allServers, invites.map((i) => i.server).filter(Boolean))
			} else {
				// don't update the server, because the invite will then redirect to the missing server
				updateStore(allInvites as never, [invites])
			}
		}),
		data.users
			?.then(filterResponse)
			?.then((users) => updateStore(allUsers, users)),
		data.friendRequests?.then(filterResponse)?.then((friendRequests) => {
			updateStore(allFriendRequests, friendRequestId(friendRequests))
			updateStore(
				allUsers,
				friendRequests.flatMap((fr) => [fr.sender, fr.receiver]),
			)
		}),
		data.friends?.then(filterResponse)?.then((friends) => {
			updateStore(allFriends, friendId(friends))
			updateStore(
				allUsers,
				friends.flatMap((f) => [f.user, f.friend]),
			)
			updateStore(
				allChannels,
				friends.map((f) => ({
					...f.channel,
					recipients: [f.user.id, f.friend.id],
				})),
			)
		}),
	])

	if (immediate) return promise

	return awaitStore(me).then(() => promise)
}

export const servers = allServers

export const members = derived(
	[allMembers, currentServerId],
	([$allMembers, $currentServerId]) =>
		$allMembers.filter((_, member) => member.server_id === $currentServerId),
)

export const channels = derived(
	[allChannels, currentServerId],
	([$allChannels, $currentServerId]) =>
		$allChannels.filter((_, channel) =>
			$currentServerId === null
				? channel.kind === "DM"
				: channel.server_id === $currentServerId,
		),
)

export const invites = derived(
	[allInvites, currentServerId],
	([$allInvites, $currentServerId]) => {
		const arr = [...$allInvites.values()].filter(
			(invite) => invite.server.id === $currentServerId,
		)
		arr.sort(
			(a, b) =>
				new Date(a.created_at).getTime() - new Date(b.created_at).getTime(),
		)
		return arr
	},
)

export const messages = derived(
	[allMessages, currentChannelId],
	([$allMessages, $currentChannelId]) =>
		$allMessages.filter(
			(_, message) => message.channel_id === $currentChannelId,
		),
)

export const invalidateAll = () => {
	allServers.update(() => new BTreeMap())
	allMembers.update(() => new BTreeMap())
	allChannels.update(() => new BTreeMap())
	allInvites.update(() => new Map())
	allMessages.update(() => new BTreeMap())
	allUsers.update(() => new BTreeMap())
	allFriendRequests.update(() => new BTreeMap())
	allFriends.update(() => new BTreeMap())
	wsStore.update((ws) => {
		ws?.close(3001)
		reopenCount = 0
		reauthCount = 0
		return undefined
	})
}

export const currentServerData = derived(
	[servers, currentServerId],
	([$servers, $currentServerId]) => $servers.get($currentServerId),
)

export const currentChannelData = derived(
	[channels, currentChannelId],
	([$channels, $currentChannelId]) => $channels.get($currentChannelId),
)

export const isMobileUI = readable<boolean>(false, (set) => {
	const mediaQuery = window.matchMedia("(min-width: 1024px)")
	const listener = (event: MediaQueryListEvent) => set(!event.matches)

	mediaQuery.addEventListener("change", listener)
	set(!mediaQuery.matches)

	return () => mediaQuery.removeEventListener("change", listener)
})

export const me = derived(
	[allUsers, meId],
	([$allUsers, $meId]) =>
		$meId &&
		($allUsers.get($meId) as User & {
			email: string
			email_verified: boolean
		}),
)

export const membersSidebarOpen = writable<boolean>(false)

// used for debugging
globalThis.__stores = {
	currentServerId,
	currentChannelId,
	allServers,
	allMembers,
	allChannels,
	allInvites,
	allMessages,
	allUsers,
	allFriendRequests,
	allFriends,
	meId,
	members,
	channels,
	invites,
	messages,
	me,
	membersSidebarOpen,
	wsStore,
	page,
}
globalThis.__storeGet = get
