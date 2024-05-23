<script lang="ts">
	import BoxLayout from "$lib/BoxLayout.svelte"
	import { goto } from "$app/navigation"

	const RATE_LIMIT_UNIT_PER_SECOND = 5

	setTimeout(
		() => {
			goto("/app")
		},
		(20 * 1000) / RATE_LIMIT_UNIT_PER_SECOND,
	)
</script>

<BoxLayout>
	<h1>Rate Limit</h1>
	<p>
		You've hit the rate limit. Please wait a few seconds, we will redirect you
		in a moment.
	</p>
</BoxLayout>
