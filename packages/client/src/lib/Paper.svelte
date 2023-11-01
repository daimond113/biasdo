<script lang="ts">
	import { getPaperStyles } from '$lib'
	import { cn } from './cn'

	export let level: 1 = 1
	export let isError = false
	export let div: HTMLDivElement | undefined = undefined
	let className: string | undefined = undefined

	export { className as class }

	$: resolvedClassName = getPaperStyles(isError ? 'error' : level, className)
</script>

<div class={resolvedClassName} bind:this={div}>
	<slot />
</div>
