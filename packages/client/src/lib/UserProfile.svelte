<script lang="ts">
	import {
		allFriendRequests,
		allFriends,
		allMembers,
		allUsers,
		currentServerId,
		me,
		servers,
	} from "./stores"
	import { flip, limitShift, offset, shift } from "svelte-floating-ui/dom"
	import type { ServerMember } from "@biasdo/server-utils/src/ServerMember"
	import type { User } from "@biasdo/server-utils/src/User"
	import { createFloatingActions } from "svelte-floating-ui"
	import { createForm } from "felte"
	import { fetch } from "./fetch"
	import { get } from "svelte/store"
	import { getImageUrl } from "./images"
	import { twMerge } from "tailwind-merge"

	import Button from "./Button.svelte"
	import Check from "lucide-svelte/icons/check"
	import PencilLine from "lucide-svelte/icons/pencil-line"
	import Plus from "lucide-svelte/icons/plus"
	import TextField from "./TextField.svelte"

	let givenUser: User
	let givenMember: ServerMember | undefined = undefined

	export { givenMember as member, givenUser as user }

	$: user =
		$allUsers.get(givenUser?.id ?? givenMember?.user_id ?? ("" as never)) ??
		givenUser
	$: member = givenMember
		? $allMembers.get(
				`${givenMember.server_id}-${givenMember.user_id}` as const,
			)
		: undefined

	$: username = user?.display_name ?? user?.username ?? "Deleted User"

	let [_floatingRef, floatingContent] = createFloatingActions({
		strategy: "absolute",
		placement: "right-end",
		middleware: [
			offset(12),
			flip(),
			shift({
				limiter: limitShift({
					offset: 0,
				}),
			}),
		],
	})

	let shown = false
	let contentRef: HTMLElement
	let boxRef: HTMLElement
	export let additionalRef: HTMLElement | undefined = undefined

	const floatingRef = (node: HTMLElement) => {
		_floatingRef(node)
		contentRef = node
	}

	export const show = (n: boolean) => {
		shown = n
	}

	let isEditing = false
	$: {
		if (!shown) isEditing = false
	}

	const { form, errors, setFields } = createForm<{
		nickname: string
	}>({
		initialValues: {
			nickname: member?.nickname ?? "",
		},
		validate: (values) => {
			const errors = {} as Record<string, string>

			if (values.nickname) {
				const nickname = values.nickname.trim()
				if (nickname.length < 2) {
					errors.nickname = "Nickname must be at least 2 characters long"
				} else if (nickname.length > 32) {
					errors.nickname = "Nickname must be at most 32 characters long"
				}
			}

			return errors
		},
		onSubmit: async (values) =>
			await fetch(`/servers/${get(currentServerId)}/members/${user.id}`, {
				method: "PATCH",
				headers: {
					"Content-Type": "application/json",
				},
				body: JSON.stringify({
					...values,
					nickname: values.nickname?.trim() || null,
				}),
			}),
		onSuccess: () => {
			isEditing = false
		},
	})

	$: friend = $allFriends
		.valuesArray()
		.find((f) => f.user.id === user?.id || f.friend.id === user?.id)

	$: setFields("nickname", member?.nickname ?? "")
</script>

<slot {show} {floatingRef} />

<svelte:window
	on:click={(e) => {
		if (!shown) return
		const path = e.composedPath()
		if (
			path.includes(contentRef) ||
			path.includes(boxRef) ||
			(additionalRef && path.includes(additionalRef))
		)
			return
		shown = false
	}}
/>

{#if shown}
	<div
		class="bg-paper-1-bg border-paper-1-outline rounded-paper-1 z-10 flex w-56 flex-col items-center gap-2 overflow-hidden border p-4 shadow-xl"
		use:floatingContent
		bind:this={boxRef}
	>
		<img
			src={getImageUrl("user", user)}
			class="h-20 w-20 rounded-lg"
			alt="{username}'s icon"
		/>
		<div class="flex w-full flex-col items-center">
			{#if member}
				<form class="flex max-w-full items-center" use:form>
					<TextField
						withoutLabel
						label="Nickname"
						class={twMerge("w-full", !isEditing && "hidden")}
						{errors}
					>
						<button class="ml-2 size-6" type="submit" title="Save nickname">
							<Check class="size-full" />
						</button></TextField
					>
					<div
						class={twMerge(
							"overflow-text w-full text-lg font-bold",
							member?.nickname ? "" : "italic",
							isEditing && "hidden",
						)}
					>
						{member?.nickname ?? "No nickname"}
					</div>
					{#if $me?.id === ($currentServerId ? $servers.get($currentServerId)?.owner_id : undefined) && !isEditing}
						<button
							class="ml-2 size-6"
							type="button"
							title="Edit nickname"
							on:click={() => {
								isEditing = true
							}}
						>
							<PencilLine class="size-full" />
						</button>
					{/if}
				</form>
			{/if}
			<div class="overflow-text max-w-full">{username}</div>
		</div>
		{#if user?.id !== $me?.id}
			{#if friend}
				<Button
					variant="secondary"
					href={`/app/direct-messages/${friend.channel.id}`}
				>
					Message
				</Button>
			{:else}
				<Button
					variant="secondary"
					disabled={$me?.id === user?.id ||
						$allFriendRequests
							.valuesArray()
							.some(
								(fr) =>
									fr.sender.id === user?.id || fr.receiver.id === user?.id,
							)}
					class="flex items-center justify-center"
					onClick={() => {
						fetch(`/users/${user.id}/friend-request`, {
							method: "POST",
						})
					}}
				>
					<Plus class="mr-1 size-6" /> <span>Add Friend</span></Button
				>
			{/if}
		{/if}
	</div>
{/if}
