<script lang="ts">
	import { run, self, createBubbler, stopPropagation } from "svelte/legacy"
	import { twMerge } from "tailwind-merge"

	const bubble = createBubbler()

	interface Props {
		showModal: boolean | null
		class?: string | undefined
		dialog?: HTMLDialogElement
		children?: import("svelte").Snippet
	}

	let {
		showModal = $bindable(),
		class: className = undefined,
		dialog = $bindable(undefined as never),
		children,
	}: Props = $props()

	run(() => {
		if (dialog) {
			if (showModal) {
				dialog.showModal()
			} else {
				dialog.close()
			}
		}
	})
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
<dialog
	bind:this={dialog}
	onclose={() => (showModal = false)}
	onclick={self(() => dialog.close())}
	class="w-full max-w-96 bg-transparent md:max-w-[32rem] lg:max-w-[48rem]"
>
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class={twMerge(
			"border-paper-1-outline bg-paper-1-bg overflow-auto rounded-2xl border p-16",
			className,
		)}
		onclick={stopPropagation(bubble("click"))}
	>
		{@render children?.()}
	</div>
</dialog>
