<script lang="ts">
	import BoxLayout from "$lib/BoxLayout.svelte"
	import Button from "$lib/Button.svelte"
	import { page } from "$app/stores"

	let message: string

	$: {
		const text = $page.error?.message
		let json
		try {
			json = text && JSON.parse(text)
		} catch {}

		message =
			json?.error_description ??
			json?.error ??
			json?.errors ??
			json?.message ??
			text
	}

	$: {
		if ($page.status === 401) {
			localStorage.removeItem("session")
		}
	}
</script>

<BoxLayout>
	<h1>{$page.status}</h1>
	<p>{message}</p>
	<div class="mt-4 flex gap-2">
		<Button href="/">Go home</Button>
		{#if $page.status === 401}
			<Button href="/login" variant="secondary">Login</Button>
		{/if}
	</div>
</BoxLayout>
