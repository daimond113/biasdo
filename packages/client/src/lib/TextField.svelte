<script context="module">
	let counter = 0
</script>

<script lang="ts">
	import type { FormEventHandler } from 'svelte/elements'

	import { cn } from './cn'

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

	const handleInput: FormEventHandler<HTMLInputElement> = (e) => {
		const target = e.target as HTMLInputElement

		value = target.value
	}

	export { className as class }
</script>

<div class={cn('flex flex-col', className)}>
	{#if !withoutLabel} <label class="mb-1" for={eltId}>{label}</label> {/if}
	<div class={cn($$slots.default ? 'flex' : 'contents')}>
		<input
			class={cn(
				'w-full h-full p-3 rounded-lg outline outline-1 outline-offset-0 outline-[var(--text-field-outline)] bg-[var(--text-field)] placeholder:text-[var(--text-field-placeholder)] outline-none transition-all',
				readonly ? '' : 'focus:brightness-125'
			)}
			{type}
			id={eltId}
			placeholder={label}
			{readonly}
			{value}
			{name}
			on:input={handleInput}
			on:keydown|self={(e) => {
				if (e.key === 'Enter' && !e.shiftKey && formElement && canSubmit) {
					e.preventDefault()
					formElement.requestSubmit()
				}
			}}
		/>
		<slot />
	</div>
	{#if errors && errors[name]}
		<p class="text-[var(--error-paper-text)] text-sm mt-1">{errors[name]}</p>
	{/if}
</div>
