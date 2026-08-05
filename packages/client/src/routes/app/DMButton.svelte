<script lang="ts">
	import { type APIChannel, allUsers, me } from "$lib/stores"
	import { getImageUrl } from "$lib/images"

	interface Props {
		channel: APIChannel
	}

	let { channel }: Props = $props()

	let recipientId = $derived(channel.recipients?.find((r) => r !== $me?.id))
	let recipient = $derived(recipientId && $allUsers.get(recipientId))
	let name = $derived(
		recipient?.display_name ?? recipient?.username ?? "Unknown",
	)
</script>

<img
	src={getImageUrl("user", recipient)}
	alt="{name}'s icon"
	class="mr-2 size-6 shrink-0 rounded-sm"
/>
<span class="overflow-text">
	{name}
</span>
