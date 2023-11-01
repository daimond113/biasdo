<script lang="ts">
	import { getPaperStyles } from '$lib'
	import { cn } from './cn'

	export let showModal: boolean
	let className: string | undefined = undefined
	let containerClassName: string | undefined = undefined
	export { className as class, containerClassName as containerClass }

	export let dialog: HTMLDialogElement = undefined as never

	$: if (dialog && showModal) dialog.showModal()
</script>

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-noninteractive-element-interactions -->
<dialog
	bind:this={dialog}
	on:close={() => (showModal = false)}
	on:click|self={() => dialog.close()}
	class={cn('backdrop:bg-[var(--modal-backdrop)] bg-transparent min-w-[24rem]', className)}
>
	<!-- svelte-ignore a11y-no-static-element-interactions -->
	<div
		on:click|stopPropagation
		class={getPaperStyles(
			1,
			cn('bg-[var(--modal-background)] p-6 w-full h-full', containerClassName)
		)}
	>
		<slot />
	</div>
</dialog>

<style>
	@keyframes zoom {
		from {
			transform: scale(0.95);
		}
		to {
			transform: scale(1);
		}
	}

	dialog[open] {
		animation: zoom 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
	}

	@keyframes fade {
		from {
			opacity: 0;
		}
		to {
			opacity: 1;
		}
	}

	dialog[open]::backdrop {
		animation: fade 0.2s ease-out;
	}
</style>
