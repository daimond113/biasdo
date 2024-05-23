<script lang="ts">
	import type { PageData } from "./$types"
	import { createForm } from "felte"
	import { getImageUrl } from "$lib/images"

	import Button from "$lib/Button.svelte"

	export let data: PageData

	const { form, isSubmitting } = createForm({
		onSubmit: async () => {
			const { code, state } = (await fetch(data.uriAuthorize, {
				method: "POST",
				headers: {
					Authorization: localStorage.getItem("session")!,
				},
			}).then((res) => res.json())) as { code: string; state?: string }

			const redirectUri = new URL(data.redirectUri)
			redirectUri.searchParams.set("code", code)
			if (state) redirectUri.searchParams.set("state", state)

			window.location.href = redirectUri.toString()
		},
	})
</script>

<svelte:head>
	<title>Authorize - biasdo</title>
</svelte:head>

<div class="flex size-full items-center justify-center">
	<div
		class="border-paper-1-outline bg-paper-1-bg w-full max-w-[48rem] shrink-0 overflow-auto rounded-2xl border p-16"
	>
		<h1 class="mb-4">Authorize Application</h1>
		<div class="mb-4">
			<img
				src={getImageUrl("app", data.client)}
				class="mr-2 inline-block size-24 rounded-md"
				alt={`${data.client.client_name}'s icon`}
			/>
			<span class="text-xl">{data.client.client_name}</span>
		</div>
		{#if data.scopes.length > 0}
			<p>With the following permissions:</p>
			<ul class="mt-2 flex list-inside list-disc flex-col gap-2">
				{#each data.scopes as scope}
					<li>{scope}</li>
				{/each}
			</ul>
		{/if}
		<div class="mt-6 flex gap-4">
			<Button
				class="mt-4 w-full shrink"
				variant="secondary"
				href={data.uriDecline}>Deny</Button
			>
			<form use:form class="contents">
				<Button
					class="mt-4 w-full shrink"
					type="submit"
					disabled={$isSubmitting}>Authorize</Button
				>
			</form>
		</div>
	</div>
</div>
