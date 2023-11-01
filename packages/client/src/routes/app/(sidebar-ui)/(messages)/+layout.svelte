<script lang="ts">
	import Paper from '$lib/Paper.svelte'
	import SidebarButton from '$lib/SidebarButton.svelte'
	import TextField from '$lib/TextField.svelte'

	import { afterUpdate, beforeUpdate, onMount } from 'svelte'
	import type { LayoutData } from './$types'
	import {
		currentServerId,
		currentChannelId,
		wsServers,
		wsChannels,
		wsMessages,
		deletedServers
	} from '$lib/stores'
	import { createForm } from 'felte'
	import { validator } from '@felte/validator-zod'
	import { z } from 'zod'
	import { credentialSubmitHandler } from '$lib'

	let div: HTMLDivElement | undefined
	let autoscroll = false

	beforeUpdate(() => {
		if (div) {
			const scrollableDistance = div.scrollHeight - div.offsetHeight
			autoscroll = div.scrollTop > scrollableDistance - 20
		}
	})

	afterUpdate(() => {
		if (autoscroll && div) {
			div.scrollTo(0, div.scrollHeight)
		}
	})

	onMount(() => {
		if (div) {
			div.scrollTo(0, div.scrollHeight)
		}
	})

	function dateToText(date: Date) {
		const isToday = new Date().toDateString() === date.toDateString()
		// 60 * 60 * 24 * 1000(ms) = 86400000(ms) = 1 day
		const isYesterday = date.toDateString() === new Date(Date.now() - 86400000).toDateString()

		if (isToday || isYesterday) {
			return `${isToday ? 'Today' : 'Yesterday'} at ${date.toLocaleTimeString([], {
				hour: '2-digit',
				minute: '2-digit'
			})}`
		}

		return date.toLocaleString([], { timeStyle: 'short', dateStyle: 'short' })
	}

	export let data: LayoutData

	$: servers = [...data.servers, ...$wsServers].filter(({ id }) => !$deletedServers.has(id))
	$: currentServerData = servers.find(({ id }) => id === $currentServerId)
	$: channels = [
		...(currentServerData?.channels.filter(({ server_id }) => server_id === $currentServerId) ??
			[]),
		...$wsChannels.filter(({ server_id }) => server_id === $currentServerId)
	]
	$: currentChannelData = channels.find(({ id }) => id === $currentChannelId)
	$: messages = [...data.messages, ...$wsMessages]
	$: currentChannelMessages = messages.filter(({ channel_id }) => channel_id === $currentChannelId)

	let formElement: HTMLFormElement

	const { form, errors, isValid, isSubmitting, isValidating, reset } = createForm({
		extend: validator({
			schema: z.object({
				content: z.string().min(1).max(4500)
			})
		}),
		onSubmit: () => credentialSubmitHandler(formElement),
		onSuccess: () => {
			reset()
		}
	})
</script>

<svelte:head>
	<title
		>#{currentChannelData?.name} | {currentServerData?.name ?? import.meta.env.VITE_APP_NAME}</title
	>
</svelte:head>

<Paper class="w-full flex-shrink-0 p-[0.375rem] h-[3.625rem] flex items-center">
	<SidebarButton notButton class="w-max max-w-[40%] inline-flex pr-3 mr-2"
		><span class="font-bold text-lg w-6 text-center mr-1">#</span
		>{currentChannelData?.name}</SidebarButton
	>
	<span class="min-w-0 whitespace-nowrap overflow-hidden text-ellipsis"
		>{currentChannelData?.topic ?? ''}</span
	>
</Paper>
<div class="flex-grow overflow-auto flex flex-col gap-2 justify-end" bind:this={div}>
	{#each currentChannelMessages ?? [] as { id, content, created_at, member: { nickname, user } } (id)}
		<div
			class="w-full flex-shrink-0 border border-transparent hover:border-[var(--paper-level-1-outline)] hover:bg-[var(--paper-level-1)] p-2 rounded-lg transition-all flex gap-2"
		>
			<img
				src="https://www.daimond113.com/logo.svg"
				class="w-10 h-10 inline rounded-lg mr-1 flex-shrink-0"
				alt={nickname ?? user?.username ?? 'Deleted User'}
			/>
			<div class="flex-grow">
				<span class="mr-1 font-bold">{nickname ?? user?.username ?? 'Deleted User'}</span>
				<span class="text-xs">{dateToText(new Date(created_at))}</span>
				<div>{content}</div>
			</div>
		</div>
	{/each}
</div>
<Paper class="w-full flex-shrink-0 p-[0.375rem] min-h-[3.625rem]">
	<form
		use:form
		bind:this={formElement}
		action="{import.meta.env
			.VITE_API_URL}/v0/servers/{$currentServerId}/channels/{$currentChannelId}/messages"
		method="post"
		class="contents"
	>
		<TextField
			label="Message"
			withoutLabel
			class="w-full h-full"
			{errors}
			name="content"
			{formElement}
			canSubmit={!($isValidating || $isSubmitting || !$isValid)}
		/>
	</form>
</Paper>