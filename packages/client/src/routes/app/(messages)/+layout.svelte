<script lang="ts">
	import { afterUpdate, beforeUpdate, tick } from "svelte"
	import {
		allMessages,
		allUsers,
		currentChannelData,
		currentChannelId,
		currentServerData,
		currentServerId,
		me,
		membersSidebarOpen,
		messages,
		populateStores,
		updateStore,
	} from "$lib/stores"
	import { MarkdownEditor } from "carta-md"
	import { afterNavigate } from "$app/navigation"
	import { carta } from "$lib/markdown"
	import { createForm } from "felte"
	import { fetch } from "$lib/fetch"
	import { get } from "svelte/store"

	import ErrorPage from "$lib/ErrorPage.svelte"
	import Hash from "lucide-svelte/icons/hash"
	import LoadingSpinner from "$lib/LoadingSpinner.svelte"
	import Message from "./Message.svelte"
	import Send from "lucide-svelte/icons/send"
	import Users from "lucide-svelte/icons/users"
	import VirtualList from "svelte-virtual-scroll-list"

	let vs: VirtualList

	let autoscroll = false
	let isFetching = false
	let abortController = new AbortController()
	let isFinished = false

	afterNavigate(() => {
		isFinished = false
		abortController.abort("Navigation interrupted")
		abortController = new AbortController()
	})

	beforeUpdate(() => {
		if (vs && vs.getScrollSize && vs.getOffsetDimension && vs.getOffset) {
			const scrollableDistance = vs.getScrollSize() - vs.getOffsetDimension()
			autoscroll = vs.getOffset() > scrollableDistance - 20
		}
	})

	afterUpdate(() => {
		if (autoscroll) {
			vs?.scrollToBottom()
		}
	})

	$: otherRecipientId = $currentChannelData?.recipients?.find(
		(r) => r !== $me?.id,
	)
	$: otherRecipient = otherRecipientId && $allUsers.get(otherRecipientId)
	$: otherRecipientName =
		otherRecipient?.display_name ?? otherRecipient?.username

	$: title = $currentServerId
		? `#${$currentChannelData?.name}`
		: `DM - ${otherRecipientName}`

	let formElement: HTMLFormElement
	let editingMessage: string | undefined

	const { form, isValid, isSubmitting, isValidating } = createForm<{
		content: string
	}>({
		validate: (values) => {
			const errors = {} as Record<string, string>

			const content = values.content?.trim()
			if (!content) {
				errors.content = "Message is required"
			} else if (content.length > 3500) {
				errors.content = "Message must be at most 3500 characters long"
			} else if (content.length < 1) {
				errors.content = "Message must be at least 1 character long"
			}

			return errors
		},
		onSubmit: async (values) => {
			const editingMessageId = editingMessage
			editingMessage = undefined
			textareaValue = ""

			return await fetch(
				`/channels/${get(currentChannelId)}/messages${editingMessageId ? `/${editingMessageId}` : ""}`,
				{
					method: editingMessageId ? "PATCH" : "POST",
					headers: {
						"Content-Type": "application/json",
					},
					body: JSON.stringify(values),
				},
			)
		},
	})

	let textareaValue: string = ""

	let data: Promise<unknown> | undefined = undefined
	let dataAbortController: AbortController | undefined = undefined

	const bottomScroll = () => {
		tick().then(() => vs?.scrollToBottom())
	}

	$: {
		dataAbortController?.abort("Navigation interrupted")
		dataAbortController = new AbortController()

		const promise = populateStores(() => {
			const promises = {
				messages: fetch(`/channels/${$currentChannelId}/messages?limit=100`, {
					signal: dataAbortController!.signal,
				}),
			} as Record<string, Promise<Response>>

			if ($currentServerId && $currentChannelId) {
				promises.channels = fetch(`/servers/${$currentServerId}/channels`, {
					signal: dataAbortController!.signal,
				})
				promises.members = fetch(
					`/servers/${$currentServerId}/members?limit=100`,
					{
						signal: dataAbortController!.signal,
					},
				)
			}

			return promises
		})

		promise.then(bottomScroll)

		data = promise
	}
</script>

<svelte:head>
	<title>{title} | {$currentServerData?.name ?? "biasdo"}</title>
</svelte:head>

{#await data}
	<div class="flex size-full items-center justify-center">
		<LoadingSpinner />
	</div>
{:then}
	<div class="flex h-full grow flex-col gap-2">
		<div
			class="bg-paper-1-bg border-paper-1-outline rounded-paper-1 flex h-[3.34375rem] w-full shrink-0 items-center border p-2"
		>
			<div class="flex items-center">
				{#if $currentServerId}
					<Hash class="mr-1 size-6" />
					<span class="overflow-text">{$currentChannelData?.name}</span>
				{:else}
					<span class="overflow-text"
						>Direct Message - {otherRecipientName}</span
					>
				{/if}
			</div>
			{#if $currentChannelData?.topic}
				<div
					class="bg-paper-1-outline mx-2 h-[calc(100%-1rem)] w-[2px] shrink-0 rounded-full"
				></div>
				<span class="overflow-text">{$currentChannelData.topic}</span>
			{/if}
			<button
				class="ml-auto hidden lg:block"
				type="button"
				on:click={() => membersSidebarOpen.update((prev) => !prev)}
				title="Toggle members panel"
			>
				<Users class="size-6" />
			</button>
		</div>

		<div class="flex-grow basis-0 overflow-hidden">
			<VirtualList
				data={$messages.valuesArray()}
				let:data
				bind:this={vs}
				on:top={() => {
					if (isFetching || isFinished) return

					isFetching = true

					const lastId = get(messages).minKey()
					if (!lastId) {
						isFetching = false
						isFinished = true
						return
					}

					fetch(
						`/channels/${get(
							currentChannelId,
						)}/messages?last_id=${lastId}&limit=100`,
						{
							signal: abortController.signal,
						},
					)
						.then((res) => res.json())
						.then((data) => {
							if (data.length === 0) {
								isFinished = true
								return
							}

							updateStore(allMessages, data)
							isFinished = data.length !== 100
						})
						.finally(() => {
							isFetching = false
						})
				}}
			>
				<Message {data} bind:textareaValue bind:editingMessage />
			</VirtualList>
		</div>

		<form
			use:form
			bind:this={formElement}
			class="border-paper-1-outline bg-paper-1-bg rounded-paper-1 flex shrink-0 items-start gap-2 border p-2"
		>
			<div class="flex w-full flex-col gap-1">
				{#if editingMessage}
					<div class="text-sm">You're currently editing a message</div>
				{/if}
				<MarkdownEditor
					{carta}
					mode="tabs"
					placeholder="Message"
					theme="biasdo"
					bind:value={textareaValue}
					textarea={{
						name: "content",
					}}
					onKeydown={(e) => {
						if (e.key === "Enter" && !e.shiftKey) {
							e.preventDefault()
							formElement.requestSubmit()
						} else if (e.key === "Escape") {
							editingMessage = undefined
							textareaValue = ""
						}
					}}
				/>
			</div>
			<button
				type="submit"
				title="Send message"
				class="cursor-pointer disabled:cursor-not-allowed"
				{...$isSubmitting || $isValidating || !$isValid
					? { disabled: true }
					: {}}
			>
				<Send class="min-h-[2.21875rem] w-6" />
			</button>
		</form>

		<slot />
	</div>
{:catch error}
	<ErrorPage {error} />
{/await}
