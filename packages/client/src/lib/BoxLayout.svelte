<script lang="ts">
	import logo from "../../static/logotype.svg?raw"
	import { twMerge } from "tailwind-merge"

	interface Props {
		class?: string | undefined
		children?: import("svelte").Snippet
	}

	let { class: className = undefined, children }: Props = $props()
</script>

<div class="h-screen p-4">
	<div class="flex h-full flex-col items-center justify-center gap-8">
		<div class="w-52 shrink-0">
			<!-- eslint-disable-next-line svelte/no-at-html-tags -->
			{@html logo}
		</div>
		<div
			class={twMerge(
				"border-paper-1-outline bg-paper-1-bg w-full max-w-[48rem] shrink-0 overflow-auto rounded-2xl border p-16",
				className,
			)}
		>
			{@render children?.()}
		</div>
	</div>
</div>
