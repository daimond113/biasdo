import type { Invite } from '@biasdo/server-utils/src/Invite'
import type { PageLoad } from './$types'
import { error, redirect } from '@sveltejs/kit'
import type { Server } from '@biasdo/server-utils/src/Server'

export const ssr = false

export const load: PageLoad = async ({ fetch, params }) => {
	const res = await fetch(`${import.meta.env.VITE_API_URL}/v0/invites/${params.inviteId}`, {
		credentials: 'include'
	})

	if (!res.ok) {
		if (res.status === 409) {
			throw redirect(302, `/app/servers/${(await res.json()).server_id}`)
		}

		if (res.status === 401) {
			throw redirect(302, '/auth')
		}

		throw error(res.status, 'An error occurred while fetching servers')
	}

	const invite = (await res.json()) as Invite & { server: Server }

	return {
		invite
	}
}
