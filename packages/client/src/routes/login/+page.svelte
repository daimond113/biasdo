<script lang="ts">
	import { createForm } from "felte"
	import { fetch } from "$lib/fetch"
	import { goto } from "$app/navigation"
	import { invalidateAll } from "$lib/stores"

	import BoxLayout from "$lib/BoxLayout.svelte"
	import Button from "$lib/Button.svelte"
	import TextField from "$lib/TextField.svelte"
	import { get } from "svelte/store"

	let error: string | undefined

	const {
		form,
		errors,
		isSubmitting,
		isValidating,
		isValid,
		data: formData,
	} = createForm<{
		username: string
		password: string
	}>({
		validate: (values) => {
			const errors = {} as Record<string, string>

			const username = values.username?.trim()
			if (!username) {
				errors.username = "Username is required"
			} else if (username.length < 2) {
				errors.username = "Username must be at least 2 characters long"
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

			const req = await fetch(`/login`, {
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

	let passkeyError: string | undefined

	const loginWithPasskey = async () => {
		const req = await fetch(`/webauthn/auth-start`, {
			method: "POST",
			headers: {
				"Content-Type": "application/json",
			},
			body: JSON.stringify({
				username: get(formData).username,
			}),
			credentials: "include",
		})

		if (req.status === 404) {
			passkeyError = "No account or passkey found for this username"
			return
		}

		let cred: Credential | null = null
		try {
			cred = await navigator.credentials.get({
				publicKey: PublicKeyCredential.parseRequestOptionsFromJSON(
					(await req.json()).publicKey,
				),
			})
		} catch (e) {
			console.error(e)
			passkeyError = e.message
			return
		}

		if (!cred) {
			passkeyError =
				"No credentials provided. Your browser may not support passkeys."
			return
		}

		try {
			const res = await fetch(`/webauthn/auth-finish`, {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
				},
				body: JSON.stringify(cred),
				credentials: "include",
			})

			if (!res.ok) {
				passkeyError = (await res.json()).error
			} else {
				localStorage.setItem("session", (await res.json()).token)
				goto("/app").then(invalidateAll)
			}
		} catch (e) {
			console.error(e)
			passkeyError = e.message
		}
	}
</script>

<svelte:head>
	<title>Login - biasdo</title>
</svelte:head>

<BoxLayout>
	{#if error}
		<div class="bg-error-bg text-error-text rounded p-4">
			{error}
		</div>
	{/if}
	<form class="flex flex-col gap-2" use:form>
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
			autocomplete="current-password"
		/>
		<p>Don't have an account yet? <a href="/register">Register one!</a></p>
		<Button
			type="submit"
			class="mt-4 w-full"
			disabled={$isSubmitting || $isValidating || !$isValid}>Login</Button
		>
		<p class="mt-2 w-full text-center text-sm">
			<button
				type="button"
				class="text-link appearance-none border-none bg-transparent"
				on:click={loginWithPasskey}
			>
				Login with passkey
			</button>
			{#if passkeyError}
				<div class="text-error-text mt-2">{passkeyError}</div>
			{/if}
		</p>
	</form>
</BoxLayout>
