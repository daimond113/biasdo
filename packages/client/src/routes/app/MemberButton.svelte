<script lang="ts">
	import { allUsers, members } from "$lib/stores"
	import { getImageUrl } from "$lib/images"
	import { twMerge } from "tailwind-merge"

	import SidebarButton from "$lib/SidebarButton.svelte"
	import UserProfile from "$lib/UserProfile.svelte"

	export let data: { user_id: `${number}`; server_id?: `${number}` }
	export let index: number

	$: user = $allUsers.get(data.user_id)!
	$: member =
		data.server_id && $members.get(`${data.server_id}-${data.user_id}`)

	$: username =
		member?.nickname ?? user?.display_name ?? user?.username ?? "Deleted User"
</script>

<UserProfile let:floatingRef let:show {user} {member}>
	<SidebarButton
		class={twMerge("group flex items-center", index !== 0 && "mt-2")}
		onClick={() => show(true)}
		{floatingRef}
	>
		<img
			class="mr-2 size-6 shrink-0 rounded-sm"
			src={getImageUrl("user", user)}
			alt="{username}'s icon"
		/>
		<span class="overflow-text">{username}</span>
	</SidebarButton>
</UserProfile>
