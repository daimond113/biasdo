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
import { derived, get, readable, writable } from 'svelte/store'

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

		ws = new WebSocket(`ws://${apiUrl.host}/v0/ws`)

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

export const deletedServers = writable<Set<Id>>(new Set())

export const wsServers = readable<ServerWithChannels[]>([], (_, update) => {
	return ws.subscribe((ws) => {
		if (!ws) return

		ws.addEventListener('message', (event) => {
			const data = JSON.parse(event.data) as { type: string; data: ServerWithChannels }
			if (!data.type.startsWith('server_')) return

			update((prev) => {
				if (data.type.endsWith('_delete')) {
					if (get(currentServerId) === data.data.id) {
						goto('/app')
					}

					deletedServers.update((prev) => prev.add(data.data.id))

					return prev.filter((server) => server.id !== data.data.id)
				}

				deletedServers.update((prev) => {
					prev.delete(data.data.id)
					return prev
				})

				return [...prev, data.data]
			})
		})
	})
})

export const wsChannels = readable<Channel[]>([], (_, update) => {
	return ws.subscribe((ws) => {
		if (!ws) return

		ws.addEventListener('message', (event) => {
			const data = JSON.parse(event.data) as { type: string; data: Channel }
			if (!data.type.startsWith('channel_')) return
			update((prev) => [...prev, data.data])
		})
	})
})

export const wsInvites = readable<Invite[]>([], (_, update) => {
	return ws.subscribe((ws) => {
		if (!ws) return

		ws.addEventListener('message', (event) => {
			const data = JSON.parse(event.data) as { type: string; data: Invite }
			if (!data.type.startsWith('invite_')) return
			update((prev) => [...prev, data.data])
		})
	})
})

export type APIMessage = Message & { member: Member & { user?: User } }

export const wsMessages = readable<APIMessage[]>([], (_, update) => {
	return ws.subscribe((ws) => {
		if (!ws) return

		ws.addEventListener('message', (event) => {
			const data = JSON.parse(event.data) as { type: string; data: APIMessage }
			if (!data.type.startsWith('message_')) return
			update((prev) => [...prev, data.data])
		})
	})
})

export const isMobileUI = readable<boolean>(false, (set) => {
	if (!browser) return

	const mediaQuery = window.matchMedia('(min-width: 1024px)')
	const listener = (event: MediaQueryListEvent) => set(!event.matches)

	mediaQuery.addEventListener('change', listener)
	set(!mediaQuery.matches)

	return () => mediaQuery.removeEventListener('change', listener)
})
