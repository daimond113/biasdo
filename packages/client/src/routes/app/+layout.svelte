<script lang="ts">
	import { afterNavigate, goto } from "$app/navigation"
	import {
		allFriendRequests,
		allMembers,
		channels,
		currentChannelData,
		currentServerData,
		currentServerId,
		isMobileUI,
		me,
		memberId,
		members,
		membersSidebarOpen,
		populateStores,
		servers,
		updateStore,
	} from "$lib/stores"
	import type { Channel } from "@biasdo/server-utils/src/Channel"
	import type { Server } from "@biasdo/server-utils/src/Server"
	import { createForm } from "felte"
	import { fetch } from "$lib/fetch"
	import { get } from "svelte/store"
	import { getImageUrl } from "$lib/images"
	import { onMount } from "svelte"
	import { page } from "$app/stores"
	import { twMerge } from "tailwind-merge"

	import Button from "$lib/Button.svelte"
	import Cog from "lucide-svelte/icons/cog"
	import Contact from "lucide-svelte/icons/contact"
	import DMButton from "./DMButton.svelte"
	import ErrorPage from "$lib/ErrorPage.svelte"
	import HardDrive from "lucide-svelte/icons/hard-drive"
	import Hash from "lucide-svelte/icons/hash"
	import LoadingSpinner from "$lib/LoadingSpinner.svelte"
	import MemberButton from "./MemberButton.svelte"
	import Menu from "lucide-svelte/icons/menu"
	import Modal from "$lib/Modal.svelte"
	import PencilLine from "lucide-svelte/icons/pencil-line"
	import Plus from "lucide-svelte/icons/plus"
	import Portal from "svelte-portal"
	import SidebarButton from "$lib/SidebarButton.svelte"
	import TextField from "$lib/TextField.svelte"
	import Users from "lucide-svelte/icons/users"
	import VirtualList from "svelte-virtual-scroll-list"
	import X from "lucide-svelte/icons/x"

	import logo from "../../../static/logo.svg?raw"

	let newServerModalOpen = false

	const {
		form: serverForm,
		errors: serverErrors,
		isValid: serverIsValid,
		isValidating: serverIsValidating,
		isSubmitting: serverIsSubmitting,
		reset: resetServerForm,
	} = createForm<{ name: string }>({
		validate: (values) => {
			const errors = {} as Record<string, string>

			const name = values.name?.trim()
			if (!name) {
				errors.name = "Server name is required"
			} else if (name.length < 2) {
				errors.name = "Server name must be at least 2 characters long"
			} else if (name.length > 32) {
				errors.name = "Server name must be at most 32 characters long"
			}

			return errors
		},
		onSubmit: async (values) => {
			return await fetch(`/servers`, {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
				},
				body: JSON.stringify(values),
			}).then((r) => r.json())
		},
		onSuccess: async (s) => {
			const {
				id,
				channels: [{ id: channelId }],
			} = s as Server & { channels: [Channel] }
			newServerModalOpen = false
			resetServerForm()
			goto(`/app/servers/${id}/channels/${channelId}`)
		},
	})

	let newChannelOrFriendsModalOpen = false

	const {
		form: channelForm,
		errors: channelErrors,
		isValid: channelIsValid,
		isValidating: channelIsValidating,
		isSubmitting: channelIsSubmitting,
		reset: resetChannelForm,
	} = createForm<{ name: string }>({
		validate: (values) => {
			const errors = {} as Record<string, string>

			const name = values.name?.trim()
			if (!name) {
				errors.name = "Channel name is required"
			} else if (name.length < 2) {
				errors.name = "Channel name must be at least 2 characters long"
			} else if (name.length > 32) {
				errors.name = "Channel name must be at most 32 characters long"
			}

			return errors
		},
		onSubmit: async (values) => {
			return await fetch(`/servers/${get(currentServerId)}/channels`, {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
				},
				body: JSON.stringify(values),
			}).then((res) => res.json())
		},
		onSuccess: async (s) => {
			const { id } = s as Channel
			newChannelOrFriendsModalOpen = false
			resetChannelForm()
			goto(`/app/servers/${get(currentServerId)}/channels/${id}`)
		},
	})

	const {
		form: addFriendForm,
		errors: addFriendErrors,
		isValid: addFriendIsValid,
		isValidating: addFriendIsValidating,
		isSubmitting: addFriendIsSubmitting,
		reset: resetAddFriendForm,
	} = createForm<{ username: string }>({
		validate: (values) => {
			const errors = {} as Record<string, string>

			const username = values.username?.trim()
			if (!username) {
				errors.username = "Username is required"
			} else if (username.length < 3) {
				errors.username = "Username must be at least 3 characters long"
			} else if (username.length > 32) {
				errors.username = "Username must be at most 32 characters long"
			} else if (!/^[a-zA-Z0-9_]+$/g.test(username)) {
				errors.username =
					"Username can only contain lowercase letters, numbers, and underscores"
			} else if (username.toLowerCase() === get(me)?.username.toLowerCase()) {
				errors.username = "You can't add yourself as a friend"
			} else if (
				get(allFriendRequests)
					.valuesArray()
					.some(
						({ receiver, sender }) =>
							receiver.username.toLowerCase() === username.toLowerCase() ||
							sender.username.toLowerCase() === username.toLowerCase(),
					)
			) {
				errors.username =
					"You already have a pending friend request with this user"
			}

			return errors
		},
		onSubmit: async (values) => {
			const { id } = await fetch(`/users/username/${values.username}`).then(
				(res) => res.json(),
			)

			return await fetch(`/users/${id}/friend-request`, {
				method: "POST",
			})
		},
		onSuccess: async () => {
			newChannelOrFriendsModalOpen = false
			resetAddFriendForm()
		},
	})

	let updateChannelModalOpen = false
	let updateChannelId: `${number}` | undefined = undefined

	const {
		form: updateChannelForm,
		errors: updateChannelErrors,
		isValid: updateChannelIsValid,
		isValidating: updateChannelIsValidating,
		isSubmitting: updateChannelIsSubmitting,
		resetField: resetUpdateChannelField,
		reset: resetUpdateChannelForm,
		setFields: setUpdateChannelFields,
	} = createForm<{ name: string }>({
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
					errors.name = "Channel name must be at least 2 characters long"
				} else if (name.length > 32) {
					errors.name = "Channel name must be at most 32 characters long"
				}
			}

			return errors
		},
		onSubmit: async (values) => {
			return await fetch(
				`/servers/${get(currentServerId)}/channels/${updateChannelId}`,
				{
					method: "PATCH",
					headers: {
						"Content-Type": "application/json",
					},
					body: JSON.stringify(values),
				},
			)
		},
		onSuccess: async () => {
			updateChannelModalOpen = false
			resetUpdateChannelForm()
		},
	})

	$: updateChannelId &&
		setUpdateChannelFields("name", $channels.get(updateChannelId)?.name ?? "")

	let vs: VirtualList

	let isFetching = false
	let abortController = new AbortController()
	let isFinished = false

	afterNavigate(() => {
		isFinished = false
		abortController.abort("Navigation interrupted")
		abortController = new AbortController()
	})

	let mobileSidebarsModalOpen = false
	let desktopNavigationElement: HTMLElement
	let mobileNavigationElement: HTMLElement
	let desktopMembersElement: HTMLElement
	let mobileMembersElement: HTMLElement

	let currentPage: "nav" | "members" = "nav"

	$: {
		if (!$isMobileUI && mobileSidebarsModalOpen) {
			mobileSidebarsModalOpen = false
		}
	}

	$: isHome = $page.url.pathname === "/app"

	let data: Promise<unknown> | undefined = undefined

	onMount(() => {
		const abortController = new AbortController()

		data = populateStores(() => ({
			servers: fetch(`/servers`, {
				signal: abortController.signal,
			}),
			friendRequests: fetch(`/friend-requests`, {
				signal: abortController.signal,
			}),
			friends: fetch(`/friends`, {
				signal: abortController.signal,
			}),
			me: fetch(`/users/@me`, {
				signal: abortController.signal,
			}),
			channels: fetch(`/direct-channels`, {
				signal: abortController.signal,
			}),
		}))

		return () => {
			abortController.abort("Navigation interrupted")
		}
	})

	$: ownsServer = $currentServerData?.owner_id === $me?.id
</script>

<svelte:head>
	<title>biasdo</title>
</svelte:head>

{#await data}
	<div class="flex h-screen w-full items-center justify-center">
		<LoadingSpinner />
	</div>
{:then}
	<Modal bind:showModal={newServerModalOpen}>
		<h1>Create Server</h1>
		<form use:serverForm class="mt-2">
			<TextField
				label="Server Name"
				class="mb-4"
				name="name"
				errors={$serverErrors}
				autocomplete="off"
			/>
			<div class="flex gap-2">
				<Button onClick={() => (newServerModalOpen = false)} variant="secondary"
					>Close</Button
				>
				<Button
					type="submit"
					disabled={$serverIsValidating ||
						$serverIsSubmitting ||
						!$serverIsValid}>Create</Button
				>
			</div>
		</form>
	</Modal>

	<Modal bind:showModal={newChannelOrFriendsModalOpen}>
		{#if $currentServerId}
			<h1>Create Channel</h1>
			<form use:channelForm class="mt-2">
				<TextField
					label="Channel Name"
					class="mb-4"
					name="name"
					errors={$channelErrors}
					autocomplete="off"
				/>
				<div class="flex gap-2">
					<Button
						onClick={() => (newChannelOrFriendsModalOpen = false)}
						variant="secondary">Close</Button
					>
					<Button
						type="submit"
						disabled={$channelIsValidating ||
							$channelIsSubmitting ||
							!$channelIsValid}>Create</Button
					>
				</div>
			</form>
		{:else}
			<h1>Add Friend</h1>
			<form use:addFriendForm class="mt-2">
				<TextField
					label="Username"
					class="mb-4"
					name="username"
					errors={$addFriendErrors}
					autocomplete="off"
				/>
				<div class="flex gap-2">
					<Button
						onClick={() => (newChannelOrFriendsModalOpen = false)}
						variant="secondary">Close</Button
					>
					<Button
						type="submit"
						disabled={$addFriendIsValidating ||
							$addFriendIsSubmitting ||
							!$addFriendIsValid}>Add</Button
					>
				</div>
			</form>
		{/if}
	</Modal>

	<Modal bind:showModal={updateChannelModalOpen}>
		<h1>Channel Settings</h1>
		<form use:updateChannelForm class="mt-2">
			<TextField
				label="Channel Name"
				class="mb-4"
				name="name"
				errors={$updateChannelErrors}
				autocomplete="off"
			>
				<Button
					class="ml-1 flex size-10 items-center justify-center p-2"
					title="Reset field"
					onClick={() => resetUpdateChannelField("name")}
				>
					<X />
				</Button>
			</TextField>
			<div class="flex gap-2">
				<Button
					onClick={() => (updateChannelModalOpen = false)}
					variant="secondary">Close</Button
				>
				<Button
					type="submit"
					disabled={$updateChannelIsValidating ||
						$updateChannelIsSubmitting ||
						!$updateChannelIsValid}>Update</Button
				>
				<Button
					type="button"
					onClick={() => {
						fetch(
							`/servers/${get(currentServerId)}/channels/${updateChannelId}`,
							{
								method: "DELETE",
							},
						).then(() => {
							updateChannelModalOpen = false
						})
					}}
					variant="error"
					class="ml-auto"
				>
					Delete
				</Button>
			</div>
		</form>
	</Modal>

	<Modal bind:showModal={mobileSidebarsModalOpen} class="p-8">
		<div class="mb-4 flex items-center px-2">
			<button
				type="button"
				on:click={() => (mobileSidebarsModalOpen = false)}
				title="Close mobile navigation menu"
			>
				<X class="size-6" />
			</button>
			<a
				class="mx-auto flex shrink-0 items-center gap-2"
				href="/app/settings"
				title="User Settings"
				data-not-standard
			>
				<div>
					{$me?.display_name ?? $me?.username}
				</div>
				<img
					src={getImageUrl("user", $me)}
					alt="My icon"
					class="inline-block size-[2.375rem] rounded-md"
				/>
			</a>
			<button
				type="button"
				on:click={() => {
					currentPage = currentPage === "nav" ? "members" : "nav"
				}}
				title="Toggle members panel"
			>
				{#if currentPage === "nav"}
					<Users class="size-6" />
				{:else}
					<HardDrive class="size-6" />
				{/if}
			</button>
		</div>
		<div
			bind:this={mobileNavigationElement}
			class={currentPage === "nav" ? "contents" : "hidden"}
		/>
		<div
			bind:this={mobileMembersElement}
			class={currentPage === "members" ? "contents" : "hidden"}
		/>
	</Modal>

	{#if $isMobileUI ? mobileNavigationElement : desktopNavigationElement}
		<Portal
			target={$isMobileUI ? mobileNavigationElement : desktopNavigationElement}
		>
			<div class="flex size-full flex-col items-center gap-2">
				<div class="my-2 flex w-full shrink-0 px-2 text-xl font-bold">
					<a
						class="overflow-text grow"
						href={$currentServerId
							? `/app/servers/${$currentServerId}`
							: `/app`}
						data-not-standard
					>
						{$currentServerData?.name ?? "Home"}
					</a>
					<a
						title={$currentServerData ? "Server Settings" : "User Settings"}
						href="/app{$currentServerId
							? `/servers/${$currentServerId}`
							: ''}/settings"
						class="ml-auto shrink-0"
						data-not-standard
					>
						<Cog class="size-6" />
					</a>
				</div>
				<SidebarButton
					onClick={() => {
						newChannelOrFriendsModalOpen = true
					}}
					disabled={$currentServerId && ($channels.length > 200 || !ownsServer)}
				>
					<Plus class="mr-2 size-6 shrink-0" /><span class="overflow-text"
						>{#if $page.params.serverId}
							Create Channel
						{:else}
							Add Friend
						{/if}</span
					>
				</SidebarButton>
				{#if !$currentServerId}
					<SidebarButton
						class={twMerge(
							$page.url.pathname === "/app/friends" && "bg-paper-2-active",
						)}
					>
						<a
							href="/app/friends"
							data-not-standard
							class="flex size-full items-center overflow-hidden"
						>
							<Contact class="mr-2 size-6 shrink-0" />
							<span class="overflow-text">Friends</span>
						</a>
					</SidebarButton>
				{/if}
				{#if $channels.length > 0}
					<div
						class="bg-paper-1-outline h-[2px] w-[calc(100%-2rem)] shrink-0 rounded-full"
					></div>
				{/if}
				<div class="flex size-full flex-col gap-2 overflow-auto">
					{#each $channels.values() as channel}
						<SidebarButton
							class={twMerge(
								$page.params.channelId === channel.id && "bg-paper-2-active",
								"group",
							)}
						>
							<a
								href={channel.server_id
									? `/app/servers/${channel.server_id}/channels/${channel.id}`
									: `/app/direct-messages/${channel.id}`}
								data-not-standard
								class="flex h-full flex-grow items-center overflow-hidden"
							>
								{#if $currentServerId}
									<Hash class="mr-2 size-6 shrink-0" />
									<span class="overflow-text">{channel.name}</span>
								{:else}
									<DMButton {channel} />
								{/if}
							</a>
							{#if $currentServerId && ownsServer}
								<button
									class="icons ml-auto shrink-0 opacity-0 transition-all group-hover:opacity-100"
									on:click={() => {
										updateChannelId = channel.id
										updateChannelModalOpen = true
									}}
								>
									<PencilLine class="size-5" />
								</button>
							{/if}
						</SidebarButton>
					{/each}
				</div>
			</div>
		</Portal>
	{/if}

	{#if $isMobileUI ? mobileMembersElement : desktopMembersElement}
		<Portal target={$isMobileUI ? mobileMembersElement : desktopMembersElement}>
			<div class="flex size-full flex-col items-center gap-2">
				<div class="overflow-text my-2 w-full shrink-0 px-2 text-xl font-bold">
					Members
				</div>
				<VirtualList
					data={$currentServerId
						? $members.valuesArray()
						: $currentChannelData?.recipients?.map((id) => ({ user_id: id })) ??
							[]}
					key="user_id"
					let:data
					bind:this={vs}
					let:index
					on:bottom={() => {
						if (isFetching || isFinished) return
						if (get(currentChannelData)?.kind === "DM") return

						const membersMap = get(members)
						const lastMember = membersMap.get(membersMap.maxKey())?.user_id

						if (!lastMember) {
							return
						}

						isFetching = true

						fetch(
							`/servers/${get(
								currentServerId,
							)}/members?last_id=${lastMember}&limit=100`,
							{
								signal: abortController.signal,
							},
						)
							.then((r) => r.json())
							.then((d) => {
								if (d.length === 0) {
									isFinished = true
									return
								}

								updateStore(allMembers, memberId(d))
								isFinished = d.length !== 100
							})
							.finally(() => {
								isFetching = false
							})
					}}
				>
					<MemberButton {data} {index} />
				</VirtualList>
			</div>
		</Portal>
	{/if}

	<div class="flex h-screen w-full flex-col gap-3 p-3">
		<div class="flex h-14 w-full shrink-0 gap-3">
			<a
				href="/app"
				class={twMerge(
					"bg-paper-1-bg outline-paper-1-outline hover:bg-paper-1-outline active:bg-paper-2-active rounded-paper-1 w-14 shrink-0 p-2 outline outline-1 -outline-offset-1 [&>svg]:h-full",
					isHome
						? "bg-paper-2-active outline-transparent [&>svg]:scale-110"
						: "hover:[&>svg]:scale-95",
					"transition-all [&>svg]:transition-all",
				)}
				title="Home"
			>
				<!-- eslint-disable-next-line svelte/no-at-html-tags -->
				{@html logo}
			</a>
			<div
				class="bg-paper-1-bg border-paper-1-outline rounded-paper-1 flex min-w-0 flex-grow items-center gap-2 border p-2"
			>
				<button
					class="bg-paper-2-bg hover:bg-paper-1-outline active:bg-paper-2-active flex aspect-square h-full shrink-0 items-center justify-center rounded-md transition-all"
					type="button"
					on:click={() => {
						newServerModalOpen = true
					}}
					title="Create Server"
				>
					<Plus />
				</button>
				{#if $servers.length > 0}
					<div
						class="bg-paper-1-outline h-[calc(100%-1rem)] w-[2px] shrink-0 rounded-full"
					></div>
				{/if}
				<div
					class="no-scrollbar flex size-full gap-2 overflow-auto"
					on:wheel={(e) => {
						e.preventDefault()
						e.currentTarget.scrollLeft += e.deltaY
					}}
				>
					{#each $servers.values() as server}
						<a
							href="/app/servers/{server.id}"
							class="inline-block aspect-square h-full"
							title={server.name}
						>
							<img
								src={getImageUrl("server", server)}
								alt="{server.name} Icon"
								class={twMerge(
									"h-full",
									$page.params.serverId === server.id
										? "rounded-[100%]"
										: "rounded-md",
									"transition-all",
								)}
							/>
						</a>
					{/each}
				</div>
				<a
					class="ml-auto hidden h-full shrink-0 items-center gap-2 lg:flex"
					href="/app/settings"
					title="User Settings"
					data-not-standard
				>
					<div>
						{$me?.display_name ?? $me?.username}
					</div>
					<img
						src={getImageUrl("user", $me)}
						alt="My icon"
						class="inline-block size-[2.375rem] rounded-md"
					/>
				</a>
				<button
					title="Toggle mobile sidebars"
					class="bg-paper-2-bg hover:bg-paper-1-outline h-full rounded-md px-[0.4375rem] transition-all lg:hidden"
					on:click={() => {
						mobileSidebarsModalOpen = true
					}}
				>
					<Menu />
				</button>
			</div>
		</div>

		<div class="flex min-h-0 grow gap-3">
			<div
				class="bg-paper-1-bg border-paper-1-outline rounded-paper-1 hidden h-full w-64 shrink-0 flex-col items-center gap-2 overflow-auto border p-2 lg:flex"
				bind:this={desktopNavigationElement}
			></div>
			<main class="grow overflow-auto">
				<slot />
			</main>
			<div
				class={twMerge(
					"bg-paper-1-bg border-paper-1-outline rounded-paper-1 hidden h-full w-64 shrink-0 flex-col items-center gap-2 overflow-auto border p-2",
					$membersSidebarOpen && "lg:flex",
					!$currentChannelData && "!hidden",
				)}
				bind:this={desktopMembersElement}
			></div>
		</div>
	</div>
{:catch error}
	<ErrorPage {error} />
{/await}
