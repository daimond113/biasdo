<script lang="ts">
	import { twMerge } from "tailwind-merge"

	interface Props {
		class?: string | undefined
		onClick?: (() => void) | undefined
		disabled?: boolean
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		floatingRef?: (...p: any[]) => void
		children?: import("svelte").Snippet
		[key: string]: any
	}

	let {
		class: className = undefined,
		onClick = undefined,
		disabled = false,
		floatingRef = () => {},
		children,
		...rest
	}: Props = $props()

	let resolvedClassName = $derived(
		twMerge(
			"bg-paper-2-bg flex h-[2.375rem] w-full shrink-0 items-center rounded-md px-2 transition-all",
			disabled
				? "opacity-50 cursor-not-allowed"
				: "cursor-pointer hover:bg-paper-1-outline active:bg-paper-2-active",
			className,
		),
	)
</script>

{#if onClick}
	<button
		type="button"
		onclick={onClick}
		{...disabled ? { disabled: true } : {}}
		{...rest}
		class={resolvedClassName}
		use:floatingRef
	>
		{@render children?.()}
	</button>
{:else}
	<div class={resolvedClassName} {...rest}>
		{@render children?.()}
	</div>
{/if}
