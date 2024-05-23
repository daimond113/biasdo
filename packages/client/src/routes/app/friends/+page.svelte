<script lang="ts">
	import { allFriendRequests, allFriends, populateStores } from "$lib/stores"
	import { fetch } from "$lib/fetch"
	import { onMount } from "svelte"

	import ErrorPage from "$lib/ErrorPage.svelte"
	import Friend from "./Friend.svelte"
	import FriendRequest from "./FriendRequest.svelte"
	import LoadingSpinner from "$lib/LoadingSpinner.svelte"

	let data: Promise<unknown> | undefined = undefined

	onMount(() => {
		data = populateStores(() => ({
			friendRequests: fetch(`/friend-requests`),
			friends: fetch(`/friends`),
		}))
	})
</script>

{#await data}
	<div class="flex size-full items-center justify-center">
		<LoadingSpinner />
	</div>
{:then}
	<div
		class="bg-paper-1-bg border-paper-1-outline rounded-paper-1 flex size-full flex-col border p-16"
	>
		<div class="size-full max-w-[48rem]">
			<h1 class="mb-2 shrink-0">Friends</h1>
			<ul class="flex w-full grow flex-col gap-2 overflow-auto">
				{#each $allFriends.values() as friend}
					<Friend data={friend} />
				{:else}
					<li>No friends yet</li>
				{/each}
			</ul>
		</div>
		<div class="size-full max-w-[48rem]">
			<h1 class="mb-2 shrink-0">Friend Requests</h1>

			<ul class="flex w-full grow flex-col gap-2 overflow-auto">
				{#each $allFriendRequests.values() as request}
					<FriendRequest data={request} />
				{:else}
					<li>No friend requests yet</li>
				{/each}
			</ul>
		</div>
	</div>
{:catch error}
	<ErrorPage {error} />
{/await}
