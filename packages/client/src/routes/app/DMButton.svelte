<script lang="ts">
	import { type APIChannel, allUsers, me } from "$lib/stores"
	import { getImageUrl } from "$lib/images"

	export let channel: APIChannel

	$: recipientId = channel.recipients?.find((r) => r !== $me?.id)
	$: recipient = recipientId && $allUsers.get(recipientId)
	$: name = recipient?.display_name ?? recipient?.username ?? "Unknown"
</script>

<img
	src={getImageUrl("user", recipient)}
	alt="{name}'s icon"
	class="mr-2 size-6 shrink-0 rounded-sm"
/>
<span class="overflow-text">
	{name}
</span>
