<script module>
	let counter = 0
</script>

<script lang="ts">
	import { twMerge } from "tailwind-merge"

	export const eltId = "input_" + counter++

	interface Props {
		label: string
		name?: any
		type?: string
		class?: string | undefined
		withoutLabel?: boolean
		readonly?: boolean
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		errors?: Record<string, any> | undefined
		autocomplete?: string
		self?: HTMLInputElement
		children?: import("svelte").Snippet
	}

	let {
		label,
		name = label.toLowerCase(),
		type = "text",
		class: className = undefined,
		withoutLabel = false,
		readonly = false,
		errors = undefined,
		autocomplete = "off",
		self = $bindable(undefined as never),
		children,
	}: Props = $props()
</script>

<div class={twMerge("flex flex-col", className)}>
	{#if !withoutLabel}
		<label for={eltId}>{label}</label>
	{/if}
	<div class={children ? "flex items-center" : "contents"}>
		<input
			class={twMerge(
				"bg-paper-2-bg placeholder:text-placeholder-text h-10 w-full resize-none rounded-md px-3 py-2 outline-0 transition-all",
				!readonly && "focus:bg-paper-2-active",
			)}
			{...{ type }}
			id={eltId}
			placeholder={label}
			{...readonly ? { readonly: true } : {}}
			{autocomplete}
			{name}
			bind:this={self}
		/>
		{@render children?.()}
	</div>
	{#if errors?.[name]}
		<p class="text-error-text mt-1 text-sm">{errors[name]}</p>
	{/if}
</div>
