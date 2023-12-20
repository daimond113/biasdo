import type { APIMessage } from '$lib/stores'
import type { Member } from '@biasdo/server-utils/src/Member'
import type { LayoutLoad } from './$types'
import { error } from '@sveltejs/kit'

export const ssr = false

export const load: LayoutLoad = async ({ fetch, params }) => {
	const [messagesRes, membersRes] = await Promise.all([
		fetch(`${import.meta.env.VITE_API_URL}/v0/channels/${params.channelId}/messages`, {
			credentials: 'include'
		}).then(async (res) => ({
			ok: res.ok,
			status: res.status,
			response: (await res.json()) as APIMessage[]
		})),

		params.serverId
			? fetch(`${import.meta.env.VITE_API_URL}/v0/servers/${params.serverId}/members`, {
					credentials: 'include'
			  }).then(async (res) => ({
					ok: res.ok,
					status: res.status,
					response: (await res.json()) as Member[]
			  }))
			: undefined
	])

	if (!messagesRes.ok) {
		error(
			messagesRes.status,
			messagesRes.status === 404
				? 'Invalid server/channel id.'
				: 'An error occurred while fetching messages'
		)
	}

	if (membersRes && !membersRes.ok) {
		error(membersRes.status, 'An error occurred while fetching members')
	}

	return {
		messages: messagesRes.response,
		members: membersRes?.response
	}
}
