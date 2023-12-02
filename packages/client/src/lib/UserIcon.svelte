<script lang="ts">
	import type { User } from '@biasdo/server-utils/src/User'
	import { cn } from './cn'
	import type { Member } from '@biasdo/server-utils/src/Member'
	import { createFloatingActions } from 'svelte-floating-ui'
	import { flip, offset } from 'svelte-floating-ui/dom'
	import { credentialSubmitHandler, getPaperStyles } from '$lib'
	import { limitShift, shift } from 'svelte-floating-ui/core'
	import Button from './Button.svelte'
	import type { Id } from '@biasdo/server-utils/src/Id'
	import { page } from '$app/stores'
	import { createForm } from 'felte'
	import type { Channel } from '@biasdo/server-utils/src/Channel'
	import { goto } from '$app/navigation'

	let className: string | undefined

	export let user: User | undefined
	export let member: (Member & { user?: User }) | undefined

	$: meId = $page.data.me?.id as Id

	const [floatingRef, floatingContent] = createFloatingActions({
		strategy: 'absolute',
		placement: 'right-end',
		middleware: [
			offset(8),
			flip(),
			shift({
				limiter: limitShift({
					offset: 0
				})
			})
		]
	})

	let showTooltip: boolean = false

	export { className as class }

	let div: HTMLDivElement
	let button: HTMLButtonElement

	$: username = member?.nickname ?? member?.user?.username ?? user?.username ?? 'Deleted User'
	$: userId = member?.user_id ?? user?.id

	let channelFormEl: HTMLFormElement

	const { form: channelForm, isSubmitting: channelIsSubmitting } = createForm({
		onSubmit: () => credentialSubmitHandler(channelFormEl),
		onSuccess: async (s) => {
			const succ = s as Response
			const { id } = (await succ.json()) as Channel
			goto(`/app/direct-messages/${id}`, {
				invalidateAll: true
			})
		}
	})
</script>

<svelte:window
	on:click={(e) => {
		if (!showTooltip) return
		const path = e.composedPath()
		if (path.includes(div) || path.includes(button)) return
		showTooltip = false
	}}
/>

<button
	type="button"
	class={cn(
		'w-10 h-10 inline rounded-lg flex-shrink-0 appearance-none focus:outline-none overflow-hidden',
		className
	)}
	on:click={() => (showTooltip = true)}
	use:floatingRef
	bind:this={button}
>
	<img
		src="/user-icons/{BigInt(userId ?? 1) % BigInt(4)}.svg"
		class="w-full h-full"
		alt={username}
		loading="lazy"
	/>
</button>

{#if showTooltip}
	<div
		class={getPaperStyles(
			1,
			'bg-[var(--modal-background)] p-3 min-w-[10rem] max-w-[15rem] overflow-hidden text-center z-10 flex flex-col items-center gap-2'
		)}
		use:floatingContent
		bind:this={div}
	>
		<img
			src="/user-icons/{BigInt(userId ?? 1) % BigInt(4)}.svg"
			class="w-20 h-20 rounded-lg"
			alt={username}
			loading="lazy"
		/>
		<span
			class="text-lg font-bold whitespace-nowrap overflow-hidden text-ellipsis min-w-0 max-w-full"
			>{username}</span
		>
		<form
			use:channelForm
			bind:this={channelFormEl}
			action="{import.meta.env.VITE_API_URL}/v0/direct-messages/channel/{userId}"
			method="post"
			class="contents"
		>
			<Button
				class="w-full"
				variant="secondary"
				disabled={meId === userId || $channelIsSubmitting}
				type="submit">Message</Button
			>
		</form>
	</div>
{/if}
