<script lang="ts">
	import Paper from '$lib/Paper.svelte'
	import SidebarButton from '$lib/SidebarButton.svelte'
	import TextField from '$lib/TextField.svelte'
	import Message from '$lib/Message.svelte'

	import { afterUpdate, beforeUpdate, onDestroy, tick } from 'svelte'
	import type { LayoutData } from './$types'
	import { currentServerId, currentChannelId, createPageStores, type APIMessage } from '$lib/stores'
	import { createForm } from 'felte'
	import { validator } from '@felte/validator-zod'
	import { z } from 'zod'
	import { credentialSubmitHandler } from '$lib'
	import VirtualList from 'svelte-virtual-scroll-list'
	import { afterNavigate } from '$app/navigation'

	let vs: VirtualList

	let autoscroll = false
	let isFetching = false
	let abortController = new AbortController()
	let isFinished = false
	let additionalMessages = [] as APIMessage[]

	afterNavigate(() => {
		additionalMessages = []
		isFinished = false
		abortController.abort('Navigation interrupted')
		abortController = new AbortController()
	})

	beforeUpdate(() => {
		if (vs) {
			const scrollableDistance = vs.getScrollSize() - vs.getOffsetDimension()
			autoscroll = vs.getOffset() > scrollableDistance - 20
		}
	})

	afterUpdate(() => {
		if (autoscroll) {
			vs?.scrollToBottom()
		}
	})

	onDestroy(
		currentChannelId.subscribe(async () => {
			await tick() // wait for new messages to be added to the DOM
			vs?.scrollToBottom()
		})
	)

	export let data: LayoutData

	const { wsServers, wsChannels, wsMessages, deletedServers } = createPageStores()

	$: servers = [...data.servers, ...$wsServers].filter(({ id }) => !$deletedServers.has(id))
	$: currentServerData = servers.find(({ id }) => id === $currentServerId)
	$: currentChannels = [
		...(currentServerData?.channels ?? []),
		...$wsChannels.filter(({ server_id }) => server_id === $currentServerId)
	]
	$: currentChannelData = currentChannels.find(({ id }) => id === $currentChannelId)
	$: messages = [
		...additionalMessages,
		...data.messages,
		...$wsMessages.filter(({ channel_id }) => channel_id === $currentChannelId)
	]

	let formElement: HTMLFormElement

	const { form, errors, isValid, isSubmitting, isValidating, reset } = createForm({
		extend: validator({
			schema: z.object({
				content: z.string().min(1).max(2000)
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
	<SidebarButton notButton class="w-max lg:max-w-[40%] inline-flex pr-3 mr-2"
		><span class="font-bold text-lg w-6 text-center mr-1">#</span
		>{currentChannelData?.name}</SidebarButton
	>
	<span class="min-w-0 whitespace-nowrap overflow-hidden text-ellipsis"
		>{currentChannelData?.topic ?? ''}</span
	>
</Paper>
<div class="flex-grow basis-0 overflow-hidden">
	<VirtualList
		data={messages}
		let:data
		bind:this={vs}
		on:top={() => {
			console.log('top')
			if (!isFetching && !isFinished) {
				console.log('fetching')
				isFetching = true

				const lastId = messages[0]?.id
				if (!lastId) {
					isFetching = false
					isFinished = true
					return
				}

				fetch(
					`${
						import.meta.env.VITE_API_URL
					}/v0/servers/${$currentServerId}/channels/${$currentChannelId}/messages?last_id=${lastId}`,
					{
						credentials: 'include',
						signal: abortController.signal
					}
				)
					.then((res) => res.json())
					.then((data) => {
						if (data.length === 0) {
							isFinished = true
						} else {
							additionalMessages = [...data, ...additionalMessages]
							isFinished = data.length !== 100
						}
					})
					.finally(() => {
						isFetching = false
					})
			}
		}}
	>
		<Message {data} />
	</VirtualList>
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

<style>
	:global(.virtual-scroll-root) {
		height: 100% !important;
	}
</style>
