<script lang="ts">
	import { twMerge } from "tailwind-merge"

	let className: string | undefined = undefined

	export { className as class }

	export let onClick: (() => void) | undefined = undefined
	export let disabled: boolean = false
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	export let floatingRef: (...p: any[]) => void = () => {}

	$: resolvedClassName = twMerge(
		"bg-paper-2-bg flex h-[2.375rem] w-full shrink-0 items-center rounded-md px-2 transition-all",
		disabled
			? "opacity-50 cursor-not-allowed"
			: "cursor-pointer hover:bg-paper-1-outline active:bg-paper-2-active",
		className,
	)
</script>

{#if onClick}
	<button
		type="button"
		on:click={onClick}
		{...disabled ? { disabled: true } : {}}
		{...$$restProps}
		class={resolvedClassName}
		use:floatingRef
	>
		<slot />
	</button>
{:else}
	<div class={resolvedClassName} {...$$restProps}>
		<slot />
	</div>
{/if}
