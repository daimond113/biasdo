<script lang="ts">
	import { currentServerId, populateStores } from "$lib/stores"
	import { fetch } from "$lib/fetch"

	import ErrorPage from "$lib/ErrorPage.svelte"
	import LoadingSpinner from "$lib/LoadingSpinner.svelte"

	let data: Promise<unknown> | undefined = undefined
	let abortController: AbortController | undefined = undefined

	$: {
		abortController?.abort("Navigation interrupted")
		abortController = new AbortController()

		if ($currentServerId) {
			data = populateStores(() => ({
				channels: fetch(`/servers/${$currentServerId}/channels`, {
					signal: abortController!.signal,
				}),
			}))
		}
	}
</script>

{#await data}
	<div class="flex size-full items-center justify-center">
		<LoadingSpinner />
	</div>
{:then}
	<slot />
{:catch error}
	<ErrorPage {error} />
{/await}
