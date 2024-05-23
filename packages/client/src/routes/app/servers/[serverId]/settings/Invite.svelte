<script lang="ts">
	import { currentServerData, me } from "$lib/stores"
	import type { Invite } from "@biasdo/server-utils/src/Invite"
	import { fetch } from "$lib/fetch"
	import { get } from "svelte/store"
	import { page } from "$app/stores"

	import Button from "$lib/Button.svelte"
	import Check from "lucide-svelte/icons/check"
	import Copy from "lucide-svelte/icons/copy"
	import TextField from "$lib/TextField.svelte"
	import X from "lucide-svelte/icons/x"

	export let invite: Invite

	$: url = new URL(`/app/invites/${invite.id}`, $page.url).toString()

	let copySuccessful: boolean | undefined = undefined
	let resetTimeout: number | undefined = undefined

	$: {
		copySuccessful

		if (resetTimeout) clearTimeout(resetTimeout)
		resetTimeout = setTimeout(() => {
			copySuccessful = undefined
		}, 1_250)
	}

	$: ownsServer = $currentServerData?.owner_id === $me?.id

	let field: HTMLInputElement | undefined

	$: if (field) {
		field.value = url
	}
</script>

<li class="contents">
	<TextField
		label="Invite URL"
		withoutLabel
		bind:self={field}
		readonly
		class="w-full"
	>
		<Button
			class="ml-2 flex size-10 items-center justify-center p-2"
			title="Copy invite URL"
			onClick={() => {
				try {
					navigator.clipboard.writeText(url)
					copySuccessful = true
				} catch {
					copySuccessful = false
				}
			}}
		>
			{#if copySuccessful}
				<Check />
			{:else if copySuccessful === false}
				<X />
			{:else}
				<Copy />
			{/if}
		</Button>
		<Button
			class="ml-2 flex size-10 items-center justify-center p-2"
			title="Revoke invite"
			disabled={!ownsServer}
			onClick={() => {
				fetch(`/servers/${get(currentServerData)?.id}/invites/${invite.id}`, {
					method: "DELETE",
				})
			}}
			variant="error"
		>
			<X />
		</Button>
	</TextField>
</li>
