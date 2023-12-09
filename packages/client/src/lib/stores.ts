import { browser } from '$app/environment'
import { goto } from '$app/navigation'
import { page } from '$app/stores'
import type { Channel } from '@biasdo/server-utils/src/Channel'
import type { Id } from '@biasdo/server-utils/src/Id'
import type { Invite } from '@biasdo/server-utils/src/Invite'
import type { Member } from '@biasdo/server-utils/src/Member'
import type { Message } from '@biasdo/server-utils/src/Message'
import type { Server } from '@biasdo/server-utils/src/Server'
import type { User } from '@biasdo/server-utils/src/User'
import { derived, get, readable, writable, type Writable } from 'svelte/store'

export const currentServerId = derived(page, ($page) => ($page.params.serverId as Id) ?? null)
export const currentChannelId = derived(page, ($page) => ($page.params.channelId as Id) ?? null)

type ServerWithChannels = Server & { channels: Channel[] }

const reopenTimeouts = [2000, 5000, 10000, 30000, 60000]

const ws = readable<WebSocket | undefined>(undefined, (set) => {
	if (!browser) return
	const apiUrl = new URL(import.meta.env.VITE_API_URL)
	let ws: WebSocket | undefined
	let reopenCount = 0
	let reopenTimeout: number | undefined
	let openPromise: Promise<void> | undefined

	const getReopenTimeout = () => {
		const n = reopenCount
		reopenCount++
		return reopenTimeouts[n >= reopenTimeouts.length - 1 ? reopenTimeouts.length - 1 : n]
	}

	const close = () => {
		if (!ws) return
		ws.close(3001, 'User closed connection')
		ws = undefined
		if (reopenTimeout) {
			clearTimeout(reopenTimeout)
			reopenTimeout = undefined
		}
	}

	const open = () => {
		if (reopenTimeout) {
			clearTimeout(reopenTimeout)
			reopenTimeout = undefined
		}

		if (openPromise) return

		ws = new WebSocket(`${apiUrl.protocol === 'https:' ? 'wss' : 'ws'}://${apiUrl.host}/v0/ws`)

		openPromise = new Promise((resolve, reject) => {
			ws!.onopen = () => {
				console.log('[WebSocket]: Connection opened')
				resolve()
				set(ws)
				openPromise = undefined
				reopenCount = 0
			}
			ws!.onerror = (event) => {
				console.log('[WebSocket]: Connection error')
				reject(event)
				openPromise = undefined
			}
		})

		ws.onclose = (event) => {
			if (event.wasClean) {
				console.log(
					`[WebSocket]: Connection closed cleanly, code=${event.code} reason=${event.reason}`
				)
			} else {
				console.log('[WebSocket]: Connection died')
			}

			if (event.code === 3001) return

			if (reopenTimeout) clearTimeout(reopenTimeout)
			reopenTimeout = setTimeout(open, getReopenTimeout())
		}
	}

	open()

	return close
})

export type APIMessage = Message & { member?: Member; user: User }

const findIndexForId = <T extends { id: Id }>(arr: T[], id: Id) => {
	// Convert id to BigInt
	const bigIntId = BigInt(id)

	let left = 0
	let right = arr.length - 1

	while (left <= right) {
		const mid = left + Math.floor((right - left) / 2)
		const midId = BigInt(arr[mid].id)

		if (midId === bigIntId) {
			return [mid] as const // Return an array to easily distinguish from indexes that are not found
		} else if (midId < bigIntId) {
			left = mid + 1
		} else {
			right = mid - 1
		}
	}

	// If no id is found that is larger than the input id, return the index where the new item should be inserted
	return left
}

const findIndexForStringId = <T extends { id: string; created_at: string }>(
	arr: T[],
	id: string,
	date: string
) => {
	let left = 0
	let right = arr.length - 1

	const dateDate = new Date(date)

	while (left <= right) {
		const mid = left + Math.floor((right - left) / 2)
		const midDate = new Date(arr[mid].created_at)

		if (arr[mid].created_at === date && arr[mid].id === id) {
			return [mid] as const // Return an array to easily distinguish from indexes that are not found
		} else if (midDate < dateDate) {
			left = mid + 1
		} else {
			right = mid - 1
		}
	}

	// If no date is found that is larger than the input date, return the index where the new item should be inserted
	return left
}

const insertBefore = <T>(array: T[], item: T, before: number) => {
	const newArray = [...array]
	newArray.splice(before, 0, item)
	return newArray
}

export const deletedServers = writable<Set<Id>>(new Set())

export const allServers = writable<ServerWithChannels[]>([], (_, update) => {
	const handler = (event: MessageEvent) => {
		const data = JSON.parse(event.data) as { type: string; data: ServerWithChannels }
		if (!data.type.startsWith('server_')) return

		if (data.type.endsWith('_delete')) {
			if (get(currentServerId) === data.data.id) {
				goto('/app', {
					invalidateAll: true
				})
			}

			deletedServers.update((prev) => prev.add(data.data.id))

			return
		}

		update((prev) => {
			deletedServers.update((prev) => {
				prev.delete(data.data.id)
				return prev
			})

			return [...prev, data.data]
		})
	}

	return ws.subscribe(
		(ws) => ws?.addEventListener('message', handler),
		(ws) => ws?.removeEventListener('message', handler)
	)
})

type APIMember = Member & { user: User }

export const allMembers = writable<APIMember[]>([], (_, update) => {
	const handler = (event: MessageEvent) => {
		const data = JSON.parse(event.data) as { type: string; data: APIMember }
		if (!data.type.startsWith('member_')) return
		update((prev) => [...prev, data.data])
	}

	return ws.subscribe(
		(ws) => ws?.addEventListener('message', handler),
		(ws) => ws?.removeEventListener('message', handler)
	)
})

type APIChannel = Channel & { recipients?: User[] }

export const allChannels = writable<APIChannel[]>([], (_, update) => {
	const handler = (event: MessageEvent) => {
		const data = JSON.parse(event.data) as { type: string; data: APIChannel }
		if (!data.type.startsWith('channel_')) return
		update((prev) => [...prev, data.data])
	}

	return ws.subscribe(
		(ws) => ws?.addEventListener('message', handler),
		(ws) => ws?.removeEventListener('message', handler)
	)
})

export const allInvites = writable<Invite[]>([], (_, update) => {
	const handler = (event: MessageEvent) => {
		const data = JSON.parse(event.data) as { type: string; data: Invite }
		if (!data.type.startsWith('invite_')) return
		update((prev) => [...prev, data.data])
	}

	return ws.subscribe(
		(ws) => ws?.addEventListener('message', handler),
		(ws) => ws?.removeEventListener('message', handler)
	)
})

const newMessageNotification = (message: APIMessage) => {
	if (message.user_id === get(page).data?.me?.id) return
	if (message.channel_id === get(currentChannelId) && document.visibilityState === 'visible') return

	if (Notification.permission === 'default') {
		Notification.requestPermission()
	}

	if (Notification.permission === 'granted') {
		const name = message.member?.nickname ?? message.user?.username ?? 'Deleted User'
		const channel = get(allChannels).find(({ id }) => id === message.channel_id)

		new Notification(`${name}${channel && channel.kind !== 'DM' ? ` | #${channel.name}` : ''}`, {
			body: message.content,
			timestamp: new Date(message.created_at).getTime(),
			icon: `/user-icons/${BigInt(message.user_id ?? 1) % BigInt(4)}.svg`
		}).addEventListener('click', () => {
			if (!channel) return

			goto(
				channel.kind === 'DM'
					? `/app/direct-messages/${channel.id}`
					: `/app/servers/${channel.server_id}/channels/${channel.id}`,
				{
					invalidateAll: true
				}
			)
		})
	}
}

export const allMessages = writable<APIMessage[]>([], (_, update) => {
	const handler = (event: MessageEvent) => {
		const data = JSON.parse(event.data) as { type: string; data: APIMessage }
		if (!data.type.startsWith('message_')) return

		newMessageNotification(data.data)

		update((prev) => [...prev, data.data])
	}

	return ws.subscribe(
		(ws) => ws?.addEventListener('message', handler),
		(ws) => ws?.removeEventListener('message', handler)
	)
})

type StringIdObject = { id: string; created_at: string }

const updateStore = <
	T extends NumericId extends true ? { id: Id } : StringIdObject,
	NumericId extends boolean = true
>(
	store: Writable<T[]>,
	newData: T[] | undefined,
	numericId: NumericId = true as NumericId
) => {
	if (!newData || newData.length <= 0) return

	store.update((prev) => {
		for (const data of newData) {
			const index = numericId
				? findIndexForId(prev as { id: Id }[], data.id as Id)
				: findIndexForStringId(
						prev as StringIdObject[],
						data.id,
						(data as StringIdObject).created_at
				  )

			if (Array.isArray(index)) {
				prev = prev.with(index[0], data)
				continue
			}
			prev = insertBefore(prev, data, index as number)
		}

		return prev
	})
}

export const populateStores = (
	data?: Partial<{
		servers: ServerWithChannels[]
		members: Member[]
		channels: Channel[]
		invites: Invite[]
		messages: APIMessage[]
	}>
) => {
	updateStore(allServers, data?.servers)
	updateStore(allMembers, data?.members)
	updateStore(allChannels, [
		...(data?.servers?.flatMap((server) => server.channels) ?? []),
		...(data?.channels ?? [])
	])
	updateStore(allMessages, data?.messages)
	updateStore(allInvites, data?.invites, false)
}

export const servers = derived([allServers, deletedServers], ([$allServers, $deletedServers]) =>
	$allServers.filter((server) => !$deletedServers.has(server.id))
)

export const members = derived([allMembers, currentServerId], ([$allMembers, $currentServerId]) =>
	$allMembers.filter((member) => member.server_id === $currentServerId)
)

export const channels = derived(
	[allChannels, currentServerId],
	([$allChannels, $currentServerId]) =>
		$allChannels.filter((channel) =>
			$currentServerId === null ? channel.kind === 'DM' : channel.server_id === $currentServerId
		)
)

export const invites = derived([allInvites, currentServerId], ([$allInvites, $currentServerId]) =>
	$allInvites.filter((invite) => invite.server_id === $currentServerId)
)

export const messages = derived(
	[allMessages, currentChannelId],
	([$allMessages, $currentChannelId]) =>
		$allMessages.filter((message) => message.channel_id === $currentChannelId)
)

export const isMobileUI = readable<boolean>(false, (set) => {
	if (!browser) return

	const mediaQuery = window.matchMedia('(min-width: 1024px)')
	const listener = (event: MediaQueryListEvent) => set(!event.matches)

	mediaQuery.addEventListener('change', listener)
	set(!mediaQuery.matches)

	return () => mediaQuery.removeEventListener('change', listener)
})
