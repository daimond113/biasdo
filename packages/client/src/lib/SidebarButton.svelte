<script lang="ts">
	import { cn } from './cn'

	export let onClick: (() => void) | undefined = undefined
	export let href: string | undefined = undefined
	export let className: string | undefined = undefined
	export let isActive = false
	export let notButton = false
	export let disabled = false

	$: resolvedClassName = cn(
		'rounded-lg p-2 h-10 border flex items-center w-full flex-shrink-0',
		isActive
			? 'border-[var(--secondary-button-active-outline)] bg-[var(--secondary-button-active)]'
			: 'bg-[var(--paper-level-1)] border-[var(--paper-level-1-outline)]',
		!notButton && !disabled && 'hover:brightness-110 active:brightness-90',
		disabled && 'opacity-50 cursor-not-allowed',
		'transition-all',
		className
	)

	export { className as class }
</script>

{#if notButton}
	<div class={resolvedClassName}>
		<slot />
	</div>
{:else if href}
	<a class={resolvedClassName} {href} data-not-standard>
		<slot />
	</a>
{:else}
	<button class={resolvedClassName} type="button" on:click={onClick} {disabled}>
		<slot />
	</button>
{/if}
