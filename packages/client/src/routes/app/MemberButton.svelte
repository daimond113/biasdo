<script lang="ts">
	import { allUsers, members } from "$lib/stores"
	import { getImageUrl } from "$lib/images"
	import { twMerge } from "tailwind-merge"

	import SidebarButton from "$lib/SidebarButton.svelte"
	import UserProfile from "$lib/UserProfile.svelte"

	interface Props {
		data: { user_id: `${number}`; server_id?: `${number}` }
		index: number
	}

	let { data, index }: Props = $props()

	let user = $derived($allUsers.get(data.user_id)!)
	let member = $derived(
		data.server_id && $members.get(`${data.server_id}-${data.user_id}`),
	)

	let username = $derived(
		member?.nickname ?? user?.display_name ?? user?.username ?? "Deleted User",
	)
</script>

<UserProfile {user} {member}>
	{#snippet children({ floatingRef, show })}
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
	{/snippet}
</UserProfile>
