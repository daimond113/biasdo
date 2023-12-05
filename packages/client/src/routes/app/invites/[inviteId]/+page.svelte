<script lang="ts">
	import Paper from '$lib/Paper.svelte'
	import Button from '$lib/Button.svelte'
	import type { PageData } from './$types'
	import { createForm } from 'felte'
	import { credentialSubmitHandler } from '$lib'
	import { goto } from '$app/navigation'

	export let data: PageData

	let formElement: HTMLFormElement

	const { form } = createForm({
		onSubmit: () => credentialSubmitHandler(formElement),
		onSuccess: () => {
			goto(`/app/servers/${data.invite.server_id}`)
		}
	})
</script>

<div class="flex justify-center items-center h-screen">
	<Paper class="w-[32rem] p-12 flex flex-col gap-4 text-center">
		<h1>You have received an invite to</h1>
		<div class="max-w-[32rem] break-words">
			{data.invite.server.name}
		</div>
		<div class="flex gap-2 w-full">
			<form
				class="contents"
				use:form
				bind:this={formElement}
				action="{import.meta.env.VITE_API_URL}/v0/invites/{data.invite.id}/join"
				method="post"
			>
				<Button class="flex-grow flex-shrink-0" type="submit">Accept</Button>
			</form>
			<Button class="flex-grow flex-shrink-0" variant="secondary" href="/app">Decline</Button>
		</div>
	</Paper>
</div>
