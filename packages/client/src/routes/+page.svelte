<script lang="ts">
	import Button from '$lib/Button.svelte'
</script>

<svelte:head>
	<title>{import.meta.env.VITE_APP_NAME}</title>
</svelte:head>

<div
	class="w-full px-4 lg:px-32 h-screen flex flex-col gap-3 justify-center items-center lg:items-start"
>
	<div class="flex items-center gap-2">
		<img src="/logo.svg" class="w-12 h-12" alt="{import.meta.env.VITE_APP_NAME} logo" />
		<h1 class="text-2xl" style="font-family: var(--logo-font);">
			{import.meta.env.VITE_APP_NAME}
		</h1>
	</div>
	<p>A chat app made for users, by users</p>
	<a href="https://www.daimond113.com" class="w-max">Made with ❤️ by daimond113</a>
	<div class="flex gap-3">
		<Button href="/auth?register=true">Register</Button>
		<Button href="/auth" variant="secondary">Login</Button>
	</div>
</div>

<footer
	class="fixed bottom-0 w-full px-4 lg:px-32 py-4 bg-[var(--paper-level-1)] flex flex-col gap-2"
>
	<p>
		Powered by a backend written in Rust using <a href="https://actix.rs">actix-web</a> with
		third-party usage in mind, and a <a href="https://svelte.dev">Svelte</a> frontend
	</p>
	<p class="text-sm max-w-max break-words">
		While the app is entirely free, hosting it still costs.
		<a href="https://buymeacoff.ee/daimond113">Mind buying me a coffee?</a>
	</p>
</footer>
