<script lang="ts">
	import { me } from "$lib/stores"
</script>

<svelte:head>
	<title>Home - biasdo</title>
</svelte:head>

<div class="flex size-full flex-col items-center justify-center">
	<div
		class="border-paper-1-outline bg-paper-1-bg w-full max-w-[48rem] flex-shrink-0 overflow-auto rounded-2xl border p-16"
	>
		<h1 class="mb-4">Hi, {$me?.display_name ?? $me?.username}!</h1>
		<p>
			Choose a server from the top bar, or a direct message channel from the
			left to get started.
		</p>
	</div>
</div>
