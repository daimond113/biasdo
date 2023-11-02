<script lang="ts">
	import Paper from '$lib/Paper.svelte'
	import SidebarButton from '$lib/SidebarButton.svelte'
	import SidebarDivider from '$lib/SidebarDivider.svelte'
	import Button from '$lib/Button.svelte'
	import Modal from '$lib/Modal.svelte'
	import TextField from '$lib/TextField.svelte'
	import { createForm } from 'felte'
	import { validator } from '@felte/validator-zod'
	import { z } from 'zod'
	import { credentialSubmitHandler } from '$lib'
	import { goto } from '$app/navigation'
	import type { Server } from '@biasdo/server-utils/src/Server'
	import type { Channel } from '@biasdo/server-utils/src/Channel'
	import type { LayoutData } from './$types'
	import {
		currentChannelId,
		currentServerId,
		deletedServers,
		isMobileUI,
		wsChannels,
		wsMessages,
		wsServers
	} from '$lib/stores'
	import { page } from '$app/stores'
	import Portal from 'svelte-portal/src/Portal.svelte'
	import { cn } from '$lib/cn'
	import { onDestroy } from 'svelte'

	let newServerModalOpen = false
	let newServerModal: HTMLDialogElement
	let newServerModalForm: HTMLFormElement

	const {
		form: serverForm,
		errors: serverErrors,
		isValid: serverIsValid,
		isValidating: serverIsValidating,
		isSubmitting: serverIsSubmitting
	} = createForm({
		extend: validator({
			schema: z.object({
				name: z.string().min(2).max(32)
			})
		}),
		onSubmit: () => credentialSubmitHandler(newServerModalForm),
		onSuccess: async (s) => {
			const succ = s as Response
			const {
				id,
				channels: [{ id: channelId }]
			} = (await succ.json()) as Server & { channels: [Channel] }
			newServerModal.close()
			goto(`/app/servers/${id}/channels/${channelId}`)
		}
	})

	let newChannelModalOpen = false
	let newChannelModal: HTMLDialogElement
	let newChannelModalForm: HTMLFormElement

	const {
		form: channelForm,
		errors: channelErrors,
		isValid: channelIsValid,
		isValidating: channelIsValidating,
		isSubmitting: channelIsSubmitting
	} = createForm({
		extend: validator({
			schema: z.object({
				name: z.string().min(2).max(32)
			})
		}),
		onSubmit: () => credentialSubmitHandler(newChannelModalForm),
		onSuccess: async (s) => {
			const succ = s as Response
			const { id } = (await succ.json()) as Channel
			newChannelModal.close()
			goto(`/app/servers/${$currentServerId}/channels/${id}`)
		}
	})

	export let data: LayoutData

	$: servers = [...data.servers, ...$wsServers].filter(({ id }) => !$deletedServers.has(id))
	$: allChannels = [...servers.flatMap(({ channels }) => channels), ...$wsChannels]
	$: channels = allChannels.filter(({ server_id }) => server_id === $currentServerId)

	let mobileSidebarsModal: HTMLDialogElement
	let mobileSidebarsModalOpen = false

	$: {
		if (!$isMobileUI && mobileSidebarsModalOpen) {
			mobileSidebarsModal.close()
		}
	}

	let mobileServerListContainer: HTMLDivElement
	let mobileChannelListContainer: HTMLDivElement
	let desktopServerListContainer: HTMLDivElement
	let desktopChannelListContainer: HTMLDivElement

	let currentView: 'server' | 'channel' = 'server'

	if (Notification.permission === 'default') {
		Notification.requestPermission()
	}

	onDestroy(
		wsMessages.subscribe((m) => {
			const newest = m[m.length - 1]
			if (!newest) return
			if (newest.member.user_id === data.me.id) return
			if (newest.channel_id === $currentChannelId && document.visibilityState === 'visible') return

			if (Notification.permission === 'granted') {
				const name = newest.member.nickname ?? newest.member.user?.username ?? 'Deleted User'
				const channel = allChannels.find(({ id }) => id === newest.channel_id)

				new Notification(`${name}${channel ? ` | #${channel.name}` : ''}`, {
					body: newest.content,
					timestamp: new Date(newest.created_at).getTime()
				}).addEventListener('click', () => {
					if (!channel) return
					goto(`/app/servers/${channel.server_id}/channels/${newest.channel_id}`)
				})
			}
		})
	)
</script>

<Modal
	bind:showModal={mobileSidebarsModalOpen}
	class="h-full min-w-0 w-full sm:w-1/2"
	bind:dialog={mobileSidebarsModal}
>
	<div class="w-full flex justify-between mb-5">
		<button
			type="button"
			title={currentView === 'server' ? 'Close modal' : 'Server view'}
			on:click={() => {
				if (currentView === 'server') {
					mobileSidebarsModal.close()
				} else {
					currentView = 'server'
				}
			}}
		>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				fill="none"
				viewBox="0 0 24 24"
				stroke-width="1.5"
				stroke="currentColor"
				class="w-8 h-8"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					d="M19.5 12h-15m0 0l6.75 6.75M4.5 12l6.75-6.75"
				/>
			</svg>
		</button>
		<button
			type="button"
			title="Channel view"
			disabled={currentView === 'channel'}
			class="disabled:opacity-30"
			on:click={() => {
				currentView = 'channel'
			}}
		>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				fill="none"
				viewBox="0 0 24 24"
				stroke-width="1.5"
				stroke="currentColor"
				class="w-8 h-8 rotate-180"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					d="M19.5 12h-15m0 0l6.75 6.75M4.5 12l6.75-6.75"
				/>
			</svg>
		</button>
	</div>
	<div
		bind:this={mobileServerListContainer}
		class={cn('flex-col gap-3 items-center', currentView === 'server' ? 'flex' : 'hidden')}
	/>
	<div
		bind:this={mobileChannelListContainer}
		class={cn('flex-col gap-3 items-center', currentView === 'channel' ? 'flex' : 'hidden')}
	/>
</Modal>

<Modal bind:showModal={newServerModalOpen} bind:dialog={newServerModal}>
	<h1 class="font-bold text-2xl">Create Server</h1>
	<form
		use:serverForm
		bind:this={newServerModalForm}
		class="mt-2"
		action="{import.meta.env.VITE_API_URL}/v0/servers"
		method="post"
	>
		<TextField label="Server Name" class="mb-4" name="name" errors={$serverErrors} />
		<div class="flex gap-2">
			<Button onClick={() => newServerModal.close()} variant="secondary">Close</Button>
			<Button type="submit" disabled={$serverIsValidating || $serverIsSubmitting || !$serverIsValid}
				>Create</Button
			>
		</div>
	</form>
</Modal>

<Modal bind:showModal={newChannelModalOpen} bind:dialog={newChannelModal}>
	<h1 class="font-bold text-2xl">Create Channel</h1>
	<form
		use:channelForm
		bind:this={newChannelModalForm}
		class="mt-2"
		action="{import.meta.env.VITE_API_URL}/v0/servers/{$currentServerId}/channels"
		method="post"
	>
		<TextField label="Channel Name" class="mb-4" name="name" errors={$channelErrors} />
		<div class="flex gap-2">
			<Button onClick={() => newChannelModal.close()} variant="secondary">Close</Button>
			<Button
				type="submit"
				disabled={$channelIsValidating || $channelIsSubmitting || !$channelIsValid}>Create</Button
			>
		</div>
	</form>
</Modal>

<Portal target={$isMobileUI ? mobileServerListContainer : desktopServerListContainer}>
	<SidebarButton href="/app" isActive={$currentServerId === null}
		><img src="/logo.svg" class="w-6 h-6 mr-1 rounded-md" alt={import.meta.env.VITE_APP_NAME} />
		<span
			class="min-w-0 whitespace-nowrap overflow-hidden text-ellipsis"
			style="font-family: var(--logo-font);">{import.meta.env.VITE_APP_NAME}</span
		></SidebarButton
	>
	<SidebarButton
		onClick={() => {
			newServerModalOpen = true
		}}
	>
		<span class="font-bold text-lg w-6 text-center mr-1">+</span>
		<span class="min-w-0 whitespace-nowrap overflow-hidden text-ellipsis">Create server</span>
	</SidebarButton>
	<SidebarDivider />
	<div class="w-full flex-grow flex flex-col gap-3 items-center overflow-auto">
		{#if servers.length > 0}
			{#each servers as { id, name, channels } (id)}
				<SidebarButton
					href="/app/servers/{id}/channels/{channels[0].id}"
					isActive={$currentServerId === id}
				>
					<img
						src="/server-icons/{BigInt(id) % 4n}.svg"
						class="w-6 mr-1 rounded-md"
						alt={name}
						loading="lazy"
					/>
					<span class="min-w-0 whitespace-nowrap overflow-hidden text-ellipsis">{name}</span>
				</SidebarButton>
			{/each}
		{:else}
			<span class="w-full h-full flex items-center align-center text-center">
				There are no servers. Create one and invite your friends to get started!
			</span>
		{/if}
	</div>
</Portal>

<Portal target={$isMobileUI ? mobileChannelListContainer : desktopChannelListContainer}>
	<SidebarButton
		href={$currentServerId ? `/app/servers/${$currentServerId}/settings` : '/app/settings'}
		isActive={$currentServerId === null
			? $page.url.pathname === '/app/settings'
			: $currentChannelId === null}
		><span class="min-w-0 whitespace-nowrap overflow-hidden text-ellipsis"
			>{#if $currentServerId === null} User {:else} Server {/if} Settings</span
		></SidebarButton
	>
	{#if $currentServerId}
		<SidebarButton
			onClick={() => {
				newChannelModalOpen = true
			}}
		>
			<span class="font-bold text-lg w-6 text-center mr-1">+</span>
			<span class="min-w-0 whitespace-nowrap overflow-hidden text-ellipsis">Create channel</span>
		</SidebarButton>
	{/if}
	{#if channels.length > 0}
		<SidebarDivider />
	{/if}
	<div class="w-full flex-grow flex flex-col gap-3 items-center overflow-auto">
		{#each channels as { id, name } (id)}
			<SidebarButton
				href={$currentServerId
					? `/app/servers/${$currentServerId}/channels/${id}`
					: `/app/direct-messages/${id}`}
				isActive={$currentChannelId === id}
				><span class="font-bold text-lg w-6 text-center mr-1">#</span><span
					class="min-w-0 whitespace-nowrap overflow-hidden text-ellipsis">{name}</span
				>
			</SidebarButton>
		{/each}
	</div>
</Portal>

<div class={cn('flex gap-3 p-3 h-screen max-w-full', $isMobileUI ? 'flex-col' : 'items-center')}>
	<button
		title="Open menu"
		class={$isMobileUI ? 'block' : 'hidden'}
		type="button"
		on:click={() => {
			mobileSidebarsModalOpen = true
		}}
	>
		<svg
			xmlns="http://www.w3.org/2000/svg"
			fill="none"
			viewBox="0 0 24 24"
			stroke-width="1.5"
			stroke="currentColor"
			class="w-6 h-6"
		>
			<path
				stroke-linecap="round"
				stroke-linejoin="round"
				d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5"
			/>
		</svg>
	</button>

	{#if import.meta.env.DEV}
		<div class="fixed top-4 right-4 text-[var(--error-paper-text)] opacity-50">
			{import.meta.env.VITE_APP_NAME} client beta. This is not a finished product.
		</div>
	{/if}

	<Paper
		class="h-full w-56 flex-shrink-0 p-3 flex-col gap-3 items-center hidden lg:flex"
		bind:div={desktopServerListContainer}
	/>

	<Paper
		class="h-[calc(100%-7.25rem)] w-56 flex-shrink-0 p-3 flex-col gap-3 items-center hidden lg:flex"
		bind:div={desktopChannelListContainer}
	/>

	<main
		class="h-full flex-grow flex flex-col gap-2 min-w-0 w-full max-h-[calc(100%-1.5rem-0.75rem)] lg:max-h-[unset]"
	>
		<slot />
	</main>
</div>
