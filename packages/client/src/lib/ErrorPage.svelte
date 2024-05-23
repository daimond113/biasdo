<script lang="ts">
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	export let error: any
</script>

<div class="bg-error-bg text-error-text rounded-paper-1 size-full p-12">
	<p>{error.message}</p>
</div>
