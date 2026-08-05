<script lang="ts">
	import { run } from "svelte/legacy"

	interface Props {
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		error: any
	}

	let { error }: Props = $props()

	run(() => {
		console.error(error)
	})
</script>

<div class="bg-error-bg text-error-text rounded-paper-1 size-full p-12">
	<p>{error.message}</p>
</div>
