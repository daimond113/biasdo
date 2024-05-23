<script context="module">
	let counter = 0
</script>

<script lang="ts">
	import { twMerge } from "tailwind-merge"

	export let label: string
	export let name = label.toLowerCase()
	export let type: string = "text"
	export let className: string | undefined = undefined
	export let withoutLabel = false
	export const eltId = "input_" + counter++
	export let readonly = false
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	export let errors: Record<string, any> | undefined = undefined
	export let autocomplete: string = "off"
	export let self: HTMLInputElement = undefined as never

	export { className as class }
</script>

<div class={twMerge("flex flex-col", className)}>
	{#if !withoutLabel}
		<label for={eltId}>{label}</label>
	{/if}
	<div class={$$slots.default ? "flex items-center" : "contents"}>
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
		<slot />
	</div>
	{#if errors?.[name]}
		<p class="text-error-text mt-1 text-sm">{errors[name]}</p>
	{/if}
</div>
