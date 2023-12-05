<script lang="ts">
	import Paper from '$lib/Paper.svelte'
</script>

<svelte:head>
	<title>{import.meta.env.VITE_APP_NAME}</title>
</svelte:head>

<div class="w-full h-full flex flex-col justify-center items-center">
	<Paper class="p-8 text-center">
		<h1 class="mb-4">Welcome to {import.meta.env.VITE_APP_NAME}</h1>
		<p>Choose a channel from the menu on the left to start chatting</p>
	</Paper>
</div>
