import type { Server } from '@biasdo/server-utils/src/Server'
import type { LayoutLoad } from './$types'
import type { Channel } from '@biasdo/server-utils/src/Channel'
import { error, redirect } from '@sveltejs/kit'
import type { User } from '@biasdo/server-utils/src/User'

export const ssr = false

export const load: LayoutLoad = async ({ fetch }) => {
	const res = await fetch(`${import.meta.env.VITE_API_URL}/v0/servers`, { credentials: 'include' })

	if (!res.ok) {
		if (res.status === 401) {
			throw redirect(302, '/auth')
		}

		throw error(res.status, 'An error occurred while fetching servers')
	}

	const servers = (await res.json()) as (Server & { channels: Channel[] })[]

	const userRes = await fetch(`${import.meta.env.VITE_API_URL}/v0/me`, {
		credentials: 'include'
	})

	if (!userRes.ok) {
		throw error(userRes.status, 'An error occurred while fetching user data')
	}

	const me = (await userRes.json()) as User

	const dmsRes = await fetch(`${import.meta.env.VITE_API_URL}/v0/direct-messages/channels`, {
		credentials: 'include'
	})

	if (!dmsRes.ok) {
		throw error(dmsRes.status, 'An error occurred while fetching direct messages')
	}

	const channels = (await dmsRes.json()) as (Channel & { kind: 'DM'; recipients: User[] })[]

	return {
		me,
		servers,
		channels
	}
}
