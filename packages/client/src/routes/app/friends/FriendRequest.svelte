<script lang="ts">
	import { allUsers, me } from "$lib/stores"
	import type { UserFriendRequest } from "@biasdo/server-utils/src/UserFriendRequest"
	import { fetch } from "$lib/fetch"
	import { getImageUrl } from "$lib/images"

	import Check from "@lucide/svelte/icons/check"
	import UserProfile from "$lib/UserProfile.svelte"
	import X from "@lucide/svelte/icons/x"

	interface Props {
		data: UserFriendRequest
	}

	let { data }: Props = $props()

	let isOutgoing = $derived(data.sender.id === $me?.id)
	let userData = $derived(isOutgoing ? data.receiver : data.sender)
	let user = $derived($allUsers.get(userData.id) ?? userData)

	let additionalRef: HTMLButtonElement = $state()
</script>

<li
	class="bg-paper-2-bg hover:bg-paper-1-outline flex gap-2 rounded-md px-2 py-3 transition-all"
>
	<UserProfile {user} {additionalRef}>
		{#snippet children({ floatingRef, show })}
			<button
				class="contents"
				type="button"
				title="Open {user.username}'s profile"
				onclick={() => show(true)}
				bind:this={additionalRef}
			>
				<img
					class="size-6 shrink-0 rounded-sm"
					src={getImageUrl("user", user)}
					alt="{user.username}'s icon"
					use:floatingRef
				/>
				<span class="overflow-text w-full text-start">
					{#if isOutgoing}
						Outgoing request to {user.username}
					{:else}
						Incoming request from {user.username}
					{/if}</span
				>
			</button>
			{#if !isOutgoing}
				<button
					type="button"
					title="Accept request"
					onclick={() => {
						fetch(`/users/${user.id}/friend-request/accept`, {
							method: "POST",
						})
					}}
				>
					<Check class="size-6" />
				</button>
			{/if}
			<button
				type="button"
				title="Delete request"
				onclick={() => {
					fetch(`/users/${user.id}/friend-request`, {
						method: "DELETE",
					})
				}}
			>
				<X class="size-6" />
			</button>
		{/snippet}
	</UserProfile>
</li>
