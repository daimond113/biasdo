<script lang="ts">
	import { me, populateStores } from "$lib/stores"
	import { fetch } from "$lib/fetch"
	import { onMount } from "svelte"

	import Button from "$lib/Button.svelte"

	onMount(() => {
		const abortController = new AbortController()

		if (localStorage.getItem("session"))
			populateStores(
				() => ({
					me: fetch(`/users/@me`, {
						signal: abortController.signal,
					}),
				}),
				true,
			)

		return () => abortController.abort()
	})
</script>

<div class="flex h-screen flex-col">
	<div
		class="flex w-full grow flex-col items-center justify-center px-4 lg:items-start lg:px-32"
	>
		<img src="/logotype.svg" class="w-64" alt="biasdo logo" />
		<p class="mt-2 text-xl font-medium">A chat app made for users, by users</p>
		<div class="mt-4 flex gap-3">
			<Button href="/register" variant="primary">Register</Button>
			<Button href="/login" variant="secondary">Login</Button>
			{#if $me}
				<Button href="/app" variant="secondary">Go to app</Button>
			{/if}
		</div>
	</div>

	<footer
		class="bg-paper-1-bg border-paper-1-outline fixed bottom-0 flex w-full shrink-0 flex-col gap-2 border-t px-4 py-4 lg:px-32"
	>
		<p>
			Made with ❤️ by <a href="https://www.daimond113.com">daimond113</a>. If
			you enjoy the app, consider
			<a href="https://buymeacoff.ee/daimond113">donating</a> to help keep it running.
		</p>
		<p class="max-w-max break-words text-sm">
			Use of this app is subject to the <a href="/tos" rel="terms-of-service"
				>Terms of Service</a
			>.
		</p>
	</footer>
</div>
