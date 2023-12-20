import type { PageLoad } from './$types'
import { error } from '@sveltejs/kit'
import type { Invite } from '@biasdo/server-utils/src/Invite'

export const ssr = false

export const load: PageLoad = async ({ fetch, params }) => {
	const res = await fetch(`${import.meta.env.VITE_API_URL}/v0/servers/${params.serverId}/invites`, {
		credentials: 'include'
	})

	if (!res.ok) {
		error(res.status, 'An error occurred while fetching invites');
	}

	const invites = (await res.json()) as Invite[]

	return {
		invites
	}
}
