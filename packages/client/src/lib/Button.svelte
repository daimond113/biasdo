<script lang="ts">
	import { cn } from './cn'

	export let onClick: (() => void) | undefined = undefined
	export let className: string | undefined = undefined
	export let disabled: boolean = false
	export let type: 'button' | 'submit' | 'reset' = 'button'
	export let variant: 'primary' | 'secondary' | 'error' = 'primary'
	export let autofocus = false
	export let href: string | undefined = undefined

	$: resolvedClassName = cn(
		'rounded-lg p-2 px-3 border',
		variant === 'primary'
			? 'bg-[var(--primary-button)] border-[var(--primary-button-outline)] text-[var(--primary-button-text)]'
			: variant === 'secondary'
			  ? 'bg-[var(--secondary-button-active)] border-[var(--secondary-button-active-outline)]'
			  : 'bg-[var(--error-button)] border-[var(--error-button-outline)] text-[var(--error-paper-text)]',
		disabled
			? 'opacity-50 cursor-not-allowed'
			: 'cursor-pointer hover:brightness-110 active:brightness-90',
		'transition-all',
		className
	)

	export { className as class }
</script>

{#if href}
	<a data-not-standard class={resolvedClassName} {href} {...$$restProps}>
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
