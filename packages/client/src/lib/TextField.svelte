<script context="module">
	let counter = 0
</script>

<script lang="ts">
	import type { FormEventHandler } from 'svelte/elements'

	import { cn } from './cn'
	import { afterUpdate } from 'svelte'

	export let label: string
	export let name = label.toLowerCase()
	export let type: string = 'text'
	export let className: string | undefined = undefined
	export let withoutLabel = false
	export const eltId = 'input_' + counter++
	export let readonly = false
	export let value: string = ''
	export let errors: Record<string, any> | undefined = undefined
	export let formElement: HTMLFormElement | undefined = undefined
	export let canSubmit = true
	export let autocomplete: string = 'off'
	export let withBorder = false
	export let asTextarea = false
	export let self: HTMLInputElement | HTMLTextAreaElement = undefined as never

	const updateAreaSize = (area: HTMLTextAreaElement) => {
		area.style.height = 'auto'
		area.style.height = `min(${area.scrollHeight}px, 12rem)`
		area.scrollTo({
			top: area.offsetTop + area.offsetHeight
		})
	}

	afterUpdate(() => {
		if (asTextarea) {
			updateAreaSize(self as HTMLTextAreaElement)
		}
	})

	const handleInput: FormEventHandler<HTMLInputElement | HTMLTextAreaElement> = (e) => {
		const target = e.target as HTMLInputElement | HTMLTextAreaElement

		value = target.value
	}

	export { className as class }

	$: resolvedClassName = cn(
		'w-full h-full p-3 rounded-lg bg-[var(--text-field)] placeholder:text-[var(--text-field-placeholder)] transition-all resize-none',
		withBorder
			? 'outline-0 border border-[var(--text-field-outline)]'
			: 'outline outline-1 outline-offset-0 outline-[var(--text-field-outline)]',
		!readonly && 'focus:brightness-125'
	)
</script>

<div class={cn('flex flex-col', className)}>
	{#if !withoutLabel}
		<label class="mb-1" for={eltId}>{label}</label>
	{/if}
	<div class={cn($$slots.default ? 'flex' : 'contents')}>
		{#if asTextarea}
			<textarea
				class={resolvedClassName}
				id={eltId}
				placeholder={label}
				{...readonly ? { readonly: true } : {}}
				{autocomplete}
				{value}
				{name}
				bind:this={self}
				on:input={handleInput}
				on:change={(e) => {
					updateAreaSize(e.target)
				}}
				on:keydown|self={(e) => {
					if (e.key === 'Enter' && !e.shiftKey && canSubmit) {
						e.preventDefault()
						formElement?.requestSubmit()
					}
				}}
				rows={1}
			/>
		{:else}
			<input
				class={resolvedClassName}
				{type}
				id={eltId}
				placeholder={label}
				{...readonly ? { readonly: true } : {}}
				{autocomplete}
				{value}
				{name}
				bind:this={self}
				on:input={handleInput}
			/>
		{/if}
		<slot />
	</div>
	{#if errors?.[name]}
		<p class="text-[var(--error-paper-text)] text-sm mt-1">{errors[name]}</p>
	{/if}
</div>
