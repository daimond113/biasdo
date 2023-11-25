import type { APIMessage } from '$lib/stores'
import type { LayoutLoad } from './$types'
import { error } from '@sveltejs/kit'

export const ssr = false

export const load: LayoutLoad = async ({ fetch, params }) => {
	console.log("I'm being called")
	const res = await fetch(
		`${import.meta.env.VITE_API_URL}/v0/servers/${params.serverId}/channels/${
			params.channelId
		}/messages`,
		{ credentials: 'include' }
	)

	if (!res.ok) {
		throw error(
			res.status,
			res.status === 404
				? 'Invalid server/channel id.'
				: 'An error occurred while fetching messages'
		)
	}

	const messages = (await res.json()) as APIMessage[]

	return {
		messages
	}
}
