<script lang="ts">
	import Paper from '$lib/Paper.svelte'
	import SidebarButton from '$lib/SidebarButton.svelte'
	import TextField from '$lib/TextField.svelte'
	import Message from '$lib/Message.svelte'

	import { afterUpdate, beforeUpdate, onDestroy, tick } from 'svelte'
	import type { LayoutData } from './$types'
	import { currentServerId, currentChannelId, type APIMessage, makeStores } from '$lib/stores'
	import { createForm } from 'felte'
	import { validator } from '@felte/validator-zod'
	import { z } from 'zod'
	import { credentialSubmitHandler } from '$lib'
	import VirtualList from 'svelte-virtual-scroll-list'
	import { afterNavigate } from '$app/navigation'
	import type { User } from '@biasdo/server-utils/src/User'
	import { get } from 'svelte/store'

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

	$: ({ servers, messages, channels } = makeStores({
		...data,
		messages: [...additionalMessages, ...data.messages],
		channels: [...data.servers.flatMap(({ channels }) => channels), ...data.channels]
	}))

	$: currentServerData = $servers.find(({ id }) => id === $currentServerId)
	$: currentChannelData = $channels.find(({ id }) => id === $currentChannelId)
	$: otherRecipient = currentChannelData?.recipients?.find(({ id }) => id !== data.me.id) as
		| User
		| undefined

	$: otherRecipientUsername = otherRecipient?.username ?? 'Deleted User'

	$: title = $currentServerId ? `#${currentChannelData?.name}` : otherRecipientUsername

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
	<title>{title} | {currentServerData?.name ?? import.meta.env.VITE_APP_NAME}</title>
</svelte:head>

<Paper class="w-full flex-shrink-0 p-[0.375rem] h-[3.625rem] flex items-center">
	<SidebarButton notButton class="w-max lg:max-w-[40%] inline-flex pr-3 mr-2"
		>{#if $currentServerId}
			<span class="font-bold text-lg w-6 text-center mr-1">#</span>
			{currentChannelData?.name}
		{:else}
			<img
				src="/user-icons/{BigInt(otherRecipient?.id ?? 0) % BigInt(4)}.svg"
				class="w-6 mr-1 rounded-md"
				alt={otherRecipientUsername}
				loading="lazy"
			/>
			{otherRecipientUsername}
		{/if}</SidebarButton
	>
	<span class="min-w-0 whitespace-nowrap overflow-hidden text-ellipsis"
		>{currentChannelData?.topic ?? ''}</span
	>
</Paper>
<div class="flex-grow basis-0 overflow-hidden">
	<VirtualList
		data={$messages}
		let:data
		bind:this={vs}
		on:top={() => {
			if (isFetching || isFinished) return

			isFetching = true

			const lastId = get(messages)[0]?.id
			if (!lastId) {
				isFetching = false
				isFinished = true
				return
			}

			fetch(
				`${import.meta.env.VITE_API_URL}/v0/channels/${get(
					currentChannelId
				)}/messages?last_id=${lastId}`,
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
		}}
	>
		<Message {data} />
	</VirtualList>
</div>
<Paper class="w-full flex-shrink-0 p-[0.375rem] min-h-[3.625rem]">
	<form
		use:form
		bind:this={formElement}
		action="{import.meta.env.VITE_API_URL}/v0/channels/{$currentChannelId}/messages"
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
			asTextarea
		/>
	</form>
</Paper>

<style>
	:global(.virtual-scroll-root) {
		height: 100% !important;
	}
</style>
