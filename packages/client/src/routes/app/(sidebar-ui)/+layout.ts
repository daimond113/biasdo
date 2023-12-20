import type { Server } from '@biasdo/server-utils/src/Server'
import type { LayoutLoad } from './$types'
import type { Channel } from '@biasdo/server-utils/src/Channel'
import { error, redirect } from '@sveltejs/kit'
import type { User } from '@biasdo/server-utils/src/User'

export const ssr = false

export const load: LayoutLoad = async ({ fetch }) => {
	const [serversRes, userRes, dmsRes] = await Promise.all([
		fetch(`${import.meta.env.VITE_API_URL}/v0/servers`, { credentials: 'include' }).then(
			async (res) => ({
				ok: res.ok,
				status: res.status,
				response: (await res.json()) as (Server & { channels: Channel[] })[]
			})
		),

		fetch(`${import.meta.env.VITE_API_URL}/v0/me`, {
			credentials: 'include'
		}).then(async (res) => ({
			ok: res.ok,
			status: res.status,
			response: (await res.json()) as User
		})),

		fetch(`${import.meta.env.VITE_API_URL}/v0/direct-messages/channels`, {
			credentials: 'include'
		}).then(async (res) => ({
			ok: res.ok,
			status: res.status,
			response: (await res.json()) as (Channel & { kind: 'DM'; recipients: User[] })[]
		}))
	])

	if (!serversRes.ok) {
		if (serversRes.status === 401) {
			redirect(302, '/auth')
		}

		error(serversRes.status, 'An error occurred while fetching servers')
	}

	if (!userRes.ok) {
		error(userRes.status, 'An error occurred while fetching user data')
	}

	if (!dmsRes.ok) {
		error(dmsRes.status, 'An error occurred while fetching direct messages')
	}

	return {
		me: userRes.response,
		servers: serversRes.response,
		channels: dmsRes.response
	}
}
