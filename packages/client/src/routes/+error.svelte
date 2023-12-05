<script lang="ts">
	import { page } from '$app/stores'
</script>

<div class="w-full h-screen flex flex-col items-center justify-center gap-2">
	<div class="flex">
		<img src="/logo.svg" class="w-6 h-6 mr-1 rounded-md" alt={import.meta.env.VITE_APP_NAME} />
		<span
			class="min-w-0 whitespace-nowrap overflow-hidden text-ellipsis"
			style="font-family: var(--logo-font);">{import.meta.env.VITE_APP_NAME}</span
		>
	</div>
	<h1>{$page.status}</h1>
	<p>{$page.error?.message}</p>
	<a href="/app">Go back to the main page</a>
</div>
