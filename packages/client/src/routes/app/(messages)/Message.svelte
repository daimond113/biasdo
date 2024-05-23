<script lang="ts">
	import { allUsers, currentChannelId, me, members } from "$lib/stores"
	import { Markdown } from "carta-md"
	import type { Message } from "@biasdo/server-utils/src/Message"
	import { carta } from "$lib/markdown"
	import { fetch } from "$lib/fetch"
	import { get } from "svelte/store"
	import { getImageUrl } from "$lib/images"
	import { twMerge } from "tailwind-merge"

	import Check from "lucide-svelte/icons/check"
	import Copy from "lucide-svelte/icons/copy"
	import PencilLine from "lucide-svelte/icons/pencil-line"
	import Trash from "lucide-svelte/icons/trash-2"
	import UserProfile from "$lib/UserProfile.svelte"
	import X from "lucide-svelte/icons/x"

	export let data: Message
	export let textareaValue: string
	export let editingMessage: string | undefined

	function dateToText(date: Date, upper = true) {
		const isToday = new Date().toDateString() === date.toDateString()
		// 60 * 60 * 24 * 1000(ms) = 86400000(ms) = 1 day
		const isYesterday =
			date.toDateString() === new Date(Date.now() - 86400000).toDateString()

		if (isToday || isYesterday) {
			const prefix = isToday ? "Today" : "Yesterday"

			return `${upper ? prefix : prefix.toLowerCase()} at ${date.toLocaleTimeString(
				[],
				{
					hour: "2-digit",
					minute: "2-digit",
				},
			)}`
		}

		return date.toLocaleString([], { timeStyle: "short", dateStyle: "short" })
	}

	$: date = new Date(Number(BigInt(data.id) >> 22n) + 1716501600000)
	$: updated_at = data.updated_at ? new Date(data.updated_at) : undefined

	let copySuccessful: boolean | undefined = undefined
	let resetTimeout: number | undefined = undefined

	$: {
		copySuccessful

		if (resetTimeout) clearTimeout(resetTimeout)
		resetTimeout = setTimeout(() => {
			copySuccessful = undefined
		}, 1_250)
	}

	$: member = data.member?.user_id
		? $members.get(`${data.member.server_id}-${data.member.user_id}` as const)
		: undefined
	$: user = data.user?.id ? $allUsers.get(data.user.id) : undefined

	$: name =
		member?.nickname ?? user?.display_name ?? user?.username ?? "Deleted User"
</script>

<div
	class={twMerge(
		"rounded-paper-1 group relative flex min-h-0 w-full gap-2 border border-transparent p-2 transition-all",
		editingMessage === data.id
			? "bg-paper-2-active"
			: "hover:border-paper-1-outline hover:bg-paper-1-bg",
	)}
>
	<UserProfile let:floatingRef let:show user={data.user} member={data.member}>
		<button
			type="button"
			title="Open {name}'s profile"
			on:click={() => show(true)}
			class="mr-1 size-10 rounded-md"
			use:floatingRef
		>
			<img
				class="size-full rounded-md"
				src={getImageUrl("user", data.user)}
				alt="{name}'s icon"
			/>
		</button>
	</UserProfile>
	<div class="-mt-[0.375rem] flex-grow overflow-hidden">
		<span class="mr-1 font-bold">{name}</span>
		<time class="text-xs" datetime={date.toISOString()}>{dateToText(date)}</time
		>
		{#if updated_at}
			<time class="text-xs" datetime={updated_at.toISOString()}
				>Edited {dateToText(updated_at, false)}</time
			>
		{/if}
		<div class="break-words">
			{#key data.content}
				<Markdown {carta} value={data.content} />
			{/key}
		</div>
	</div>
	<div
		class="absolute -right-px -top-px flex overflow-hidden rounded-[0.8125rem] opacity-0 shadow-lg transition-all group-hover:opacity-100"
	>
		<button
			class="bg-paper-2-active hover:bg-paper-1-outline p-2 transition-all"
			type="button"
			title="Copy message content"
			on:click={() => {
				copySuccessful = undefined
				try {
					navigator.clipboard.writeText(data.content)
					copySuccessful = true
				} catch {
					copySuccessful = false
				}
			}}
		>
			{#if copySuccessful}
				<Check class="size-5" />
			{:else if copySuccessful === false}
				<X class="size-5" />
			{:else}
				<Copy class="size-5" />
			{/if}
		</button>
		{#if data.user?.id === $me?.id}
			<button
				class="bg-paper-2-active hover:bg-paper-1-outline p-2 transition-all"
				type="button"
				title="Edit message"
				on:click={() => {
					editingMessage = data.id
					textareaValue = data.content
				}}
			>
				<PencilLine class="size-5" />
			</button>
			<button
				class="bg-error-bg text-error-text hover:bg-error-bg-hover p-2 transition-all"
				type="button"
				title="Delete message"
				on:click={() => {
					fetch(`/channels/${get(currentChannelId)}/messages/${data.id}`, {
						method: "DELETE",
					})
				}}
			>
				<Trash class="size-5" />
			</button>
		{/if}
	</div>
</div>
