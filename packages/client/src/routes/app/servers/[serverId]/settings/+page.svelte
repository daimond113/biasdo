<script lang="ts">
	import {
		currentServerData,
		currentServerId,
		invites,
		me,
		populateStores,
	} from "$lib/stores"
	import { createForm } from "felte"
	import { fetch } from "$lib/fetch"
	import { get } from "svelte/store"

	import Button from "$lib/Button.svelte"
	import ErrorPage from "$lib/ErrorPage.svelte"
	import Invite from "./Invite.svelte"
	import LoadingSpinner from "$lib/LoadingSpinner.svelte"
	import TextField from "$lib/TextField.svelte"
	import X from "lucide-svelte/icons/x"

	let data: Promise<unknown> | undefined = undefined
	let abortController: AbortController | undefined = undefined

	$: {
		abortController?.abort("Navigation interrupted")
		abortController = new AbortController()

		if ($currentServerId) {
			data = populateStores(() => ({
				invites: fetch(`/servers/${$currentServerId}/invites`, {
					signal: abortController!.signal,
				}),
			}))
		}
	}

	const { form, errors, isValid, isValidating, isSubmitting, setFields } =
		createForm<{ name: string }>({
			initialValues: { name: get(currentServerData)?.name ?? "" },
			validate: (values) => {
				if (
					Object.keys(values).filter(
						(name) => values[name as keyof typeof values],
					).length === 0
				)
					return { name: "At least one field must be filled in" }

				const errors = {} as Record<string, string>

				if (values.name) {
					const name = values.name.trim()
					if (name.length < 2) {
						errors.name = "Server name must be at least 2 characters long"
					} else if (name.length > 32) {
						errors.name = "Server name must be at most 32 characters long"
					}
				}

				return errors
			},
			onSubmit: async (values) => {
				return await fetch(`/servers/${get(currentServerId)}`, {
					method: "PATCH",
					headers: {
						"Content-Type": "application/json",
					},
					body: JSON.stringify(values),
				})
			},
		})

	$: setFields("name", $currentServerData?.name ?? "")

	$: ownsServer = $currentServerData?.owner_id === $me?.id
</script>

<svelte:head>
	<title>{$currentServerData?.name ?? ""} Settings - biasdo</title>
</svelte:head>

{#await data}
	<div class="flex size-full items-center justify-center">
		<LoadingSpinner />
	</div>
{:then}
	<div
		class="border-paper-1-outline bg-paper-1-bg flex size-full shrink-0 flex-col gap-4 overflow-auto rounded-2xl border pt-16"
	>
		<div class="max-w-[48rem] px-16">
			<h1>Server Settings</h1>
			<form use:form class="mt-2">
				<TextField
					label="Server Name"
					class="mb-2 w-full"
					name="name"
					errors={$errors}
					autocomplete="off"
					readonly={!ownsServer}
				>
					<Button
						class="ml-2 flex size-10 items-center justify-center p-2"
						title="Reset field"
						onClick={() =>
							setFields("name", get(currentServerData)?.name ?? "")}
						disabled={!ownsServer}
						variant="error"
					>
						<X />
					</Button>
				</TextField>
				<Button
					type="submit"
					disabled={$isValidating || $isSubmitting || !$isValid || !ownsServer}
					>Update</Button
				>
			</form>
			<div class="mt-4">
				<h2>Invites</h2>
				{#if $invites.length > 0}
					<ul class="flex max-h-72 flex-col gap-2 overflow-auto">
						{#each $invites.values() as invite}
							<Invite {invite} />
						{/each}
					</ul>
				{:else}
					<p>No invites have been generated yet. Generate one below!</p>
				{/if}
				<Button
					class="mt-2"
					disabled={$invites.length > 30 || !ownsServer}
					onClick={() => {
						fetch(`/servers/${get(currentServerId)}/invites`, {
							method: "POST",
						})
					}}>Generate Invite</Button
				>
			</div>
		</div>
		<div class="bg-error-bg/40 text-error-text mt-auto w-full px-16 py-8">
			<h2>Danger Zone</h2>
			<p>These actions cannot be undone. Be careful!</p>
			<div class="mt-4 flex gap-4">
				<Button
					onClick={() => {
						fetch(`/servers/${get(currentServerId)}`, {
							method: "DELETE",
						})
					}}
					variant="error"
					disabled={!ownsServer}
				>
					Delete Server
				</Button>
				<Button
					onClick={() => {
						fetch(`/servers/${get(currentServerId)}/leave`, {
							method: "POST",
						})
					}}
					variant="error"
					disabled={ownsServer}>Leave Server</Button
				>
			</div>
		</div>
	</div>
{:catch error}
	<ErrorPage {error} />
{/await}
