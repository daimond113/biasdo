<script lang="ts">
	import { run } from "svelte/legacy"

	import { allInvites, allServers, populateStores } from "$lib/stores"
	import { createForm } from "felte"
	import { fetch } from "$lib/fetch"
	import { getImageUrl } from "$lib/images"
	import { goto } from "$app/navigation"
	import { page } from "$app/stores"

	import Button from "$lib/Button.svelte"
	import ErrorPage from "$lib/ErrorPage.svelte"
	import LoadingSpinner from "$lib/LoadingSpinner.svelte"

	let currentInviteId = $derived($page.params.inviteId)
	let currentInviteData = $derived($allInvites.get(currentInviteId))

	const { form, isSubmitting } = createForm({
		onSubmit: async () =>
			await fetch(`/invites/${currentInviteId}`, {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
				},
			}),
		onSuccess: () => {
			goto(`/app/servers/${currentInviteData?.server.id}`)
		},
	})

	let data: Promise<unknown> | undefined = $state(undefined)
	let abortController: AbortController | undefined = $state(undefined)

	run(() => {
		abortController?.abort("Navigation interrupted")
		abortController = new AbortController()

		if (currentInviteId) {
			data = populateStores(() => ({
				invites: fetch(`/invites/${currentInviteId}`, {
					signal: abortController!.signal,
				}),
			}))
		}
	})

	run(() => {
		if (
			$allServers
				.valuesArray()
				.some(({ id }) => currentInviteData?.server.id === id)
		) {
			goto(`/app/servers/${currentInviteData?.server.id}`)
		}
	})
</script>

{#await data}
	<div class="flex size-full items-center justify-center">
		<LoadingSpinner />
	</div>
{:then}
	<div class="flex size-full items-center justify-center">
		<div
			class="border-paper-1-outline bg-paper-1-bg w-full max-w-[48rem] shrink-0 overflow-auto rounded-2xl border p-16"
		>
			<h1 class="mb-2">Server Invite</h1>
			<p class="mb-4">You've been invited to join:</p>
			<div class="mb-4">
				<img
					src={getImageUrl("server", currentInviteData?.server)}
					class="mr-2 inline-block size-24 rounded-md"
					alt={`${currentInviteData?.server.name}'s icon`}
				/>
				<span class="text-xl">{currentInviteData?.server.name}</span>
			</div>
			<div class="mt-6 flex gap-4">
				<Button class="mt-4 w-full shrink" variant="secondary" href="/app"
					>Deny</Button
				>
				<form use:form class="contents">
					<Button
						class="mt-4 w-full shrink"
						type="submit"
						disabled={$isSubmitting}>Accept</Button
					>
				</form>
			</div>
		</div>
	</div>
{:catch error}
	<ErrorPage {error} />
{/await}
