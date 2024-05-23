import type { PageLoad } from "./$types"
import { fetch } from "$lib/fetch"
import { redirect } from "@sveltejs/kit"

export const load: PageLoad = async ({ params: { serverId } }) => {
	const channels = await fetch(`/servers/${serverId}/channels`).then((res) =>
		res.json(),
	)

	redirect(302, `/app/servers/${serverId}/channels/${channels[0].id}`)
}
