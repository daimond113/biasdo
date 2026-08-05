<script lang="ts">
	import { run } from "svelte/legacy"

	import { currentServerData, me } from "$lib/stores"
	import type { Invite } from "@biasdo/server-utils/src/Invite"
	import { fetch } from "$lib/fetch"
	import { get } from "svelte/store"
	import { page } from "$app/stores"

	import Button from "$lib/Button.svelte"
	import Check from "@lucide/svelte/icons/check"
	import Copy from "@lucide/svelte/icons/copy"
	import TextField from "$lib/TextField.svelte"
	import X from "@lucide/svelte/icons/x"

	interface Props {
		invite: Invite
	}

	let { invite }: Props = $props()

	let url = $derived(new URL(`/app/invites/${invite.id}`, $page.url).toString())

	let copySuccessful: boolean | undefined = $state(undefined)
	let resetTimeout: number | undefined = $state(undefined)

	run(() => {
		copySuccessful

		if (resetTimeout) clearTimeout(resetTimeout)
		resetTimeout = setTimeout(() => {
			copySuccessful = undefined
		}, 1_250)
	})

	let ownsServer = $derived($currentServerData?.owner_id === $me?.id)

	let field: HTMLInputElement | undefined = $state()

	run(() => {
		if (field) {
			field.value = url
		}
	})
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
