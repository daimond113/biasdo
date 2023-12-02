<script lang="ts">
	import type { Message } from '@biasdo/server-utils/src/Message'
	import type { Member } from '@biasdo/server-utils/src/Member'
	import type { User } from '@biasdo/server-utils/src/User'
	import UserIcon from './UserIcon.svelte'

	export let data: Message & { member?: Member; user?: User }

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
</script>

<div
	class="w-full border border-transparent hover:border-[var(--paper-level-1-outline)] hover:bg-[var(--paper-level-1)] p-2 rounded-lg transition-all flex gap-2 min-h-0 relative"
>
	<UserIcon class="mr-1" user={data.user} member={data.member} />
	<div class="flex-grow overflow-hidden -mt-[0.375rem]">
		<span class="mr-1 font-bold"
			>{data.member?.nickname ?? data.user?.username ?? 'Deleted User'}</span
		>
		<time class="text-xs" datetime={data.created_at}>{dateToText(new Date(data.created_at))}</time>
		<div class="break-words">{data.content}</div>
	</div>
</div>
