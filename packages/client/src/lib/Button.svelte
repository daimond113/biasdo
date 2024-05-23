<script lang="ts">
	import { twMerge } from "tailwind-merge"

	export let onClick: (() => void) | undefined = undefined
	export let className: string | undefined = undefined
	export let disabled: boolean = false
	export let type: "button" | "submit" | "reset" = "button"
	export let variant: "primary" | "secondary" | "error" = "primary"
	export let autofocus = false
	export let href: string | undefined = undefined

	$: resolvedClassName = twMerge(
		variant === "primary"
			? "bg-paper-2-active"
			: variant === "error"
				? "bg-error-bg text-error-text"
				: "bg-background text-alt-text",
		!disabled &&
			(variant === "error"
				? "hover:bg-error-bg-hover active:bg-error-bg/80"
				: "hover:bg-paper-2-bg active:bg-paper-2-active/20"),
		disabled ? "opacity-50 cursor-not-allowed" : "cursor-pointer",
		"rounded-md px-3 h-10 transition-all shrink-0",
		className,
	)

	export { className as class }
</script>

{#if href}
	<a
		data-not-standard
		class={twMerge("flex items-center justify-center", resolvedClassName)}
		{href}
		{...$$restProps}
	>
		<slot />
	</a>
{:else}
	<!-- svelte-ignore a11y-autofocus -->
	<button
		class={resolvedClassName}
		{type}
		{disabled}
		{autofocus}
		on:click={onClick}
		{...$$restProps}
	>
		<slot />
	</button>
{/if}
