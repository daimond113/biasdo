<script lang="ts">
	import Button from '$lib/Button.svelte'
	import Paper from '$lib/Paper.svelte'
	import TextField from '$lib/TextField.svelte'
	import { page } from '$app/stores'
	import { FelteSubmitError, createForm } from 'felte'
	import { validator } from '@felte/validator-zod'
	import { z } from 'zod'
	import { goto } from '$app/navigation'
	import { credentialSubmitHandler } from '$lib'

	$: isRegister = $page.url.searchParams.get('register') === 'true'

	let serverErrors: string | undefined = undefined

	let authForm: HTMLFormElement

	const { form, errors, isSubmitting, isValidating, isValid } = createForm({
		extend: [
			validator({
				schema: z.object({
					username: z.string().min(1).max(16),
					password: z.string().min(8).max(70)
				})
			})
		],
		onSubmit: () => credentialSubmitHandler(authForm),
		onSuccess: () => {
			goto('/app')
		},
		onError: async (e) => {
			const { response } = e as FelteSubmitError
			const data = await response.json()
			serverErrors = 'errors' in data ? data.errors : 'Unknown error'
		}
	})
</script>

<div class="flex justify-center items-center h-screen p-2">
	<form use:form bind:this={authForm} action="{import.meta.env.VITE_API_URL}/auth" method="post">
		<Paper class="max-w-[32rem] p-6 sm:p-12">
			<h1 class="text-center mb-2">
				{#if isRegister}
					Register
				{:else}
					Log in
				{/if}
			</h1>
			<p class="text-center mb-8">
				Welcome to {import.meta.env.VITE_APP_NAME}. Please {#if isRegister}
					register
				{:else}
					login
				{/if} to continue
			</p>
			{#if serverErrors}
				<Paper isError class="px-3 py-2 mb-4">{serverErrors}</Paper>
			{/if}
			<input type="hidden" name="kind" value={isRegister ? 'register' : 'login'} />
			<TextField label="Username" class="mb-4" errors={$errors} autocomplete="username" />
			<TextField
				label="Password"
				type="password"
				class="mb-6"
				errors={$errors}
				autocomplete="current-password"
			/>
			<p class="text-center mb-6">
				{#if isRegister}
					Already have an account? <a href="?register=false">Login</a> right now!
				{:else}
					Don't have an account yet? <a href="?register=true">Register one</a> right now!
				{/if}
			</p>
			<Button
				onClick={() => {}}
				class="w-full"
				type="submit"
				disabled={$isSubmitting || $isValidating || !$isValid}
			>
				{#if isRegister}
					Register
				{:else}
					Login
				{/if}
			</Button>
		</Paper>
	</form>
</div>
