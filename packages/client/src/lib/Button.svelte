<script lang="ts">
	import { twMerge } from "tailwind-merge"

	interface Props {
		onClick?: (() => void) | undefined
		class?: string | undefined
		disabled?: boolean
		type?: "button" | "submit" | "reset"
		variant?: "primary" | "secondary" | "error"
		autofocus?: boolean
		href?: string | undefined
		children?: import("svelte").Snippet
		[key: string]: any
	}

	let {
		onClick = undefined,
		class: className = undefined,
		disabled = false,
		type = "button",
		variant = "primary",
		autofocus = false,
		href = undefined,
		children,
		...rest
	}: Props = $props()

	let resolvedClassName = $derived(
		twMerge(
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
		),
	)
</script>

{#if href}
	<a
		data-not-standard
		class={twMerge("flex items-center justify-center", resolvedClassName)}
		{href}
		{...rest}
	>
		{@render children?.()}
	</a>
{:else}
	<!-- svelte-ignore a11y_autofocus -->
	<button
		class={resolvedClassName}
		{type}
		{disabled}
		{autofocus}
		onclick={onClick}
		{...rest}
	>
		{@render children?.()}
	</button>
{/if}
