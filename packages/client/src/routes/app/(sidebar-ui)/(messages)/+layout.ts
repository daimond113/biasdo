import type { APIMessage } from '$lib/stores'
import type { Member } from '@biasdo/server-utils/src/Member'
import type { LayoutLoad } from './$types'
import { error } from '@sveltejs/kit'

export const ssr = false

export const load: LayoutLoad = async ({ fetch, params }) => {
	const messagesRes = await fetch(
		`${import.meta.env.VITE_API_URL}/v0/channels/${params.channelId}/messages`,
		{ credentials: 'include' }
	)

	if (!messagesRes.ok) {
		throw error(
			messagesRes.status,
			messagesRes.status === 404
				? 'Invalid server/channel id.'
				: 'An error occurred while fetching messages'
		)
	}

	const messages = (await messagesRes.json()) as APIMessage[]

	let members: Member[] | undefined

	if (params.serverId) {
		const membersRes = await fetch(
			`${import.meta.env.VITE_API_URL}/v0/servers/${params.serverId}/members`,
			{ credentials: 'include' }
		)

		if (!membersRes.ok) {
			throw error(membersRes.status, 'An error occurred while fetching members')
		}

		members = (await membersRes.json()) as Member[]
	}

	return {
		messages,
		members
	}
}
