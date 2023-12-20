import type { PageLoad } from './$types'
import { error } from '@sveltejs/kit'
import type { User } from '@biasdo/server-utils/src/User'

export const ssr = false

export const load: PageLoad = async ({ fetch }) => {
	const userRes = await fetch(`${import.meta.env.VITE_API_URL}/v0/me`, {
		credentials: 'include'
	})

	if (!userRes.ok && userRes.status !== 401) {
		error(userRes.status, 'An error occurred while fetching user data');
	}

	const me = userRes.ok ? ((await userRes.json()) as User) : null

	return {
		me
	}
}
