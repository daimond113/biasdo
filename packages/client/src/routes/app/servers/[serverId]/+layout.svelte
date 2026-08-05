<script lang="ts">
	import { run } from "svelte/legacy"

	import { currentServerId, populateStores } from "$lib/stores"
	import { fetch } from "$lib/fetch"

	import ErrorPage from "$lib/ErrorPage.svelte"
	import LoadingSpinner from "$lib/LoadingSpinner.svelte"
	interface Props {
		children?: import("svelte").Snippet
	}

	let { children }: Props = $props()

	let data: Promise<unknown> | undefined = $state(undefined)
	let abortController: AbortController | undefined = $state(undefined)

	run(() => {
		abortController?.abort("Navigation interrupted")
		abortController = new AbortController()

		if ($currentServerId) {
			data = populateStores(() => ({
				channels: fetch(`/servers/${$currentServerId}/channels`, {
					signal: abortController!.signal,
				}),
			}))
		}
	})
</script>

{#await data}
	<div class="flex size-full items-center justify-center">
		<LoadingSpinner />
	</div>
{:then}
	{@render children?.()}
{:catch error}
	<ErrorPage {error} />
{/await}
