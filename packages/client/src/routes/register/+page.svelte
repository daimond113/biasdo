<script lang="ts">
	import { createForm } from "felte"
	import { fetch } from "$lib/fetch"
	import { goto } from "$app/navigation"
	import { invalidateAll } from "$lib/stores"

	import BoxLayout from "$lib/BoxLayout.svelte"
	import Button from "$lib/Button.svelte"
	import TextField from "$lib/TextField.svelte"

	let error: string | undefined

	const { form, errors, isSubmitting, isValidating, isValid } = createForm<{
		email: string
		username: string
		password: string
	}>({
		validate: (values) => {
			const errors = {} as Record<string, string>

			if (!values.email) {
				errors.email = "Email is required"
			} else if (
				!/^(([^<>()\[\]\\.,;:\s@"]+(\.[^<>()\[\]\\.,;:\s@"]+)*)|(".+"))@((\[[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}])|(([a-zA-Z\-0-9]+\.)+[a-zA-Z]{2,}))$/.test(
					values.email,
				)
			) {
				errors.email = "Email is invalid"
			} else if (values.email.length > 255) {
				errors.email = "Email must be at most 255 characters long"
			}

			const username = values.username?.trim()
			if (!username) {
				errors.username = "Username is required"
			} else if (username.length < 3) {
				errors.username = "Username must be at least 3 characters long"
			} else if (username.length > 32) {
				errors.username = "Username must be at most 32 characters long"
			}
			if (values.username) {
				if (!/^[a-zA-Z0-9_]+$/g.test(username)) {
					errors.username =
						"Username can only contain lowercase letters, numbers, and underscores"
				}
			}

			if (!values.password) {
				errors.password = "Password is required"
			} else if (values.password.length < 8) {
				errors.password = "Password must be at least 8 characters long"
			} else if (values.password.length > 128) {
				errors.password = "Password must be at most 128 characters long"
			}

			return errors
		},
		onSubmit: async (values) => {
			error = undefined

			const req = await fetch(`/register`, {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
				},
				body: JSON.stringify(values),
			})

			const body = await req.json()

			if (!req.ok) {
				error = body.error ?? body.errors
				throw new Error(error)
			}

			const { token } = body as { token: string }

			localStorage.setItem("session", token)
		},
		onSuccess: () => {
			goto("/app").then(invalidateAll)
		},
	})
</script>

<svelte:head>
	<title>Register - biasdo</title>
</svelte:head>

<BoxLayout>
	{#if error}
		<div class="bg-error-bg text-error-text rounded p-4">
			{error}
		</div>
	{/if}
	<form class="flex flex-col gap-2" use:form>
		<TextField
			type="email"
			label="Email"
			errors={$errors}
			autocomplete="email"
		/>
		<TextField
			type="text"
			label="Username"
			errors={$errors}
			autocomplete="username"
		/>
		<TextField
			type="password"
			label="Password"
			errors={$errors}
			autocomplete="new-password"
		/>
		<p>Already have an account? <a href="/login">Login!</a></p>
		<Button
			type="submit"
			class="mt-4 w-full"
			disabled={$isSubmitting || $isValidating || !$isValid}>Register</Button
		>
	</form>
</BoxLayout>
