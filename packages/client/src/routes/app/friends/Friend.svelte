<script lang="ts">
	import { allUsers, me } from "$lib/stores"
	import type { UserFriend } from "@biasdo/server-utils/src/UserFriend"
	import { fetch } from "$lib/fetch"
	import { getImageUrl } from "$lib/images"

	import UserProfile from "$lib/UserProfile.svelte"
	import X from "lucide-svelte/icons/x"

	export let data: UserFriend

	$: userData = data.user.id === $me?.id ? data.friend : data.user
	$: user = $allUsers.get(userData.id) ?? userData

	let additionalRef: HTMLButtonElement
</script>

<li
	class="bg-paper-2-bg hover:bg-paper-1-outline flex gap-2 rounded-md px-2 py-3 transition-all"
>
	<UserProfile let:floatingRef let:show {user} {additionalRef}>
		<button
			class="contents"
			type="button"
			title="Open {user.username}'s profile"
			on:click={() => show(true)}
			bind:this={additionalRef}
		>
			<img
				class="size-6 shrink-0 rounded-sm"
				src={getImageUrl("user", user)}
				alt="{user.username}'s icon"
				use:floatingRef
			/>
			<span class="overflow-text w-full text-start">
				{user.username}
			</span>
		</button>
		<button
			type="button"
			title="Delete friend"
			on:click={() => {
				fetch(`/friends/${user.id}`, {
					method: "DELETE",
				})
			}}
		>
			<X class="size-6" />
		</button>
	</UserProfile>
</li>
