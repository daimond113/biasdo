import type { PageLoad } from './$types'
import { error, redirect } from '@sveltejs/kit'
import type { Channel } from '@biasdo/server-utils/src/Channel'

export const ssr = false

export const load: PageLoad = async ({ fetch, params }) => {
	const res = await fetch(
		`${import.meta.env.VITE_API_URL}/v0/servers/${params.serverId}/channels`,
		{
			credentials: 'include'
		}
	)

	if (!res.ok) {
		throw error(res.status, 'An error occurred while fetching channels')
	}

	const channels = (await res.json()) as Channel[]

	throw redirect(302, `/app/servers/${params.serverId}/channels/${channels[0].id}`)
}
