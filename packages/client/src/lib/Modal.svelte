<script lang="ts">
	import { twMerge } from "tailwind-merge"

	export let showModal: boolean | null
	let className: string | undefined = undefined
	export { className as class }

	export let dialog: HTMLDialogElement = undefined as never

	$: if (dialog) {
		if (showModal) {
			dialog.showModal()
		} else {
			dialog.close()
		}
	}
</script>

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-noninteractive-element-interactions -->
<dialog
	bind:this={dialog}
	on:close={() => (showModal = false)}
	on:click|self={() => dialog.close()}
	class="w-full max-w-96 bg-transparent md:max-w-[32rem] lg:max-w-[48rem]"
>
	<!-- svelte-ignore a11y-no-static-element-interactions -->
	<div
		class={twMerge(
			"border-paper-1-outline bg-paper-1-bg overflow-auto rounded-2xl border p-16",
			className,
		)}
		on:click|stopPropagation
	>
		<slot />
	</div>
</dialog>
