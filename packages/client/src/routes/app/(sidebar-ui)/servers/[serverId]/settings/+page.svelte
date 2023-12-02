<script lang="ts">
	import { page } from '$app/stores'
	import Button from '$lib/Button.svelte'
	import Paper from '$lib/Paper.svelte'
	import TextField from '$lib/TextField.svelte'
	import { currentServerId, makeStores } from '$lib/stores'
	import { createForm } from 'felte'
	import type { PageData } from './$types'
	import { credentialSubmitHandler } from '$lib'
	import { afterNavigate } from '$app/navigation'

	export let data: PageData

	let inviteFormElement: HTMLFormElement

	const {
		form: inviteForm,
		isValid: inviteIsValid,
		isValidating: inviteIsValidating,
		isSubmitting: inviteIsSubmitting
	} = createForm({
		onSubmit: () => credentialSubmitHandler(inviteFormElement)
	})

	let leaveFormElement: HTMLFormElement

	const { form: leaveForm } = createForm({
		onSubmit: () => credentialSubmitHandler(leaveFormElement)
	})

	$: ({ servers, invites } = makeStores(data))

	$: currentServerData = $servers.find(({ id }) => id === $currentServerId)
</script>

<svelte:head>
	<title>{currentServerData?.name} Settings</title>
</svelte:head>

<Paper class="w-full h-full p-6 overflow-auto flex flex-col">
	<h1 class="text-2xl font-bold mb-4">{currentServerData?.name} Settings</h1>
	<div class="w-full lg:w-2/3 xl:w-1/2 2xl:w-1/3 mb-4">
		<h2 class="text-xl font-bold mb-4">Invites</h2>
		<ul>
			{#each $invites as { id, expires_at } (id)}
				<li class="mb-4">
					<TextField
						class="w-full"
						readonly
						label="Invite"
						value={new URL(`/app/invites/${id}`, $page.url).toString()}
						withoutLabel
					>
						<Button
							onClick={() => {
								navigator.clipboard.writeText(new URL(`/app/invites/${id}`, $page.url).toString())
							}}
							class="ml-2 py-[calc(0.5rem+0.5px)]"
							variant="secondary">Copy</Button
						>
						<Button onClick={() => {}} class="ml-2 py-[calc(0.5rem+0.5px)]" variant="error" disabled
							>Delete</Button
						>
					</TextField>
					<p class="text-sm mt-2">
						Expires on: <time datetime={expires_at}
							>{new Date(expires_at).toLocaleString([], {
								timeStyle: 'short',
								dateStyle: 'short'
							})}</time
						>
					</p>
				</li>
			{/each}
		</ul>
		<form
			use:inviteForm
			bind:this={inviteFormElement}
			action="{import.meta.env.VITE_API_URL}/v0/servers/{$currentServerId}/invites"
			method="post"
		>
			<Button
				class="w-full"
				type="submit"
				disabled={data.me.id !== currentServerData?.owner_id ||
					$inviteIsValidating ||
					$inviteIsSubmitting ||
					!$inviteIsValid}>Generate</Button
			>
		</form>
	</div>

	<Paper isError class="p-4 mt-auto">
		<h2 class="text-xl font-bold">Danger Zone</h2>
		<p class="text-sm">These actions are irreversible. Please be careful.</p>
		<div class="flex gap-2 mt-2">
			<Button disabled variant="error">Delete Server</Button>
			<form
				use:leaveForm
				bind:this={leaveFormElement}
				action="{import.meta.env.VITE_API_URL}/v0/servers/{$currentServerId}/leave"
				method="post"
			>
				<Button
					onClick={() => {
						if (confirm('Are you sure you want to leave this server?')) {
							leaveFormElement.requestSubmit()
						}
					}}
					variant="error"
					disabled={currentServerData?.owner_id === data.me.id}
				>
					Leave Server
				</Button>
			</form>
		</div></Paper
	>
</Paper>
