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

		if (json && "error" in json && json.redirect !== false) {
			const redirectUri = $page.url.searchParams.get("redirect_uri")
			if (redirectUri) {
				const url = new URL(redirectUri)
				url.searchParams.set("error", json.error)
				url.searchParams.set("error_description", json.error_description)

				window.location.href = url.toString()
			}
		}

		message =
			json?.error_description ??
			json?.error ??
			json?.errors ??
			json?.message ??
			text
	}
</script>

<BoxLayout>
	<h1>{$page.status}</h1>
	<p>{message}</p>
	<div class="mt-4 flex gap-2">
		<Button href="/">Go home</Button>
	</div>
</BoxLayout>
