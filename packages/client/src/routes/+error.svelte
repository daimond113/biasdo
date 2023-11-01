<script lang="ts">
	import { page } from '$app/stores'
</script>

<div class="w-full h-screen flex flex-col items-center justify-center gap-2">
	<p>{import.meta.env.VITE_APP_NAME}</p>
	<h1 class="font-bold text-2xl">{$page.status}</h1>
	<p>{$page.error?.message}</p>
	<a href="/app">Go back to the main page</a>
</div>
