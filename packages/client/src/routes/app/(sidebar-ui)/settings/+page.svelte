<script lang="ts">
	import Button from '$lib/Button.svelte'
	import Paper from '$lib/Paper.svelte'
	import TextField from '$lib/TextField.svelte'
	import { createForm } from 'felte'
	import type { PageData } from './$types'
	import { credentialSubmitHandler } from '$lib'
	import { goto } from '$app/navigation'

	export let data: PageData

	let formElement: HTMLFormElement

	const { form } = createForm({
		onSubmit: () => credentialSubmitHandler(formElement),
		onSuccess: () => {
			goto('/auth')
		}
	})
</script>

<svelte:head>
	<title>Settings</title>
</svelte:head>

<Paper class="w-full h-full p-6 overflow-auto flex flex-col">
	<h1 class="text-2xl font-bold mb-4">Settings</h1>

	<div class="w-full lg:w-2/3 xl:w-1/2 2xl:w-1/3">
		<TextField label="Username" value={data.me.username} readonly class="w-full" />
	</div>

	<Paper isError class="p-4 mt-auto">
		<h2 class="text-xl font-bold">Danger Zone</h2>
		<p class="text-sm">These actions are irreversible. Please be careful.</p>
		<div class="flex gap-2 mt-2">
			<Button disabled variant="error">Delete Account</Button>
			<form
				use:form
				bind:this={formElement}
				action="{import.meta.env.VITE_API_URL}/v0/logout"
				method="post"
			>
				<Button variant="error" type="submit">Log out</Button>
			</form>
		</div>
	</Paper>
</Paper>
