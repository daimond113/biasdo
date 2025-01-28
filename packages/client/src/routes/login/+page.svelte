<script lang="ts">
	import { createForm } from "felte"
	import { fetch } from "$lib/fetch"
	import { goto } from "$app/navigation"
	import { invalidateAll } from "$lib/stores"

	import BoxLayout from "$lib/BoxLayout.svelte"
	import Button from "$lib/Button.svelte"
	import Modal from "$lib/Modal.svelte"
	import TextField from "$lib/TextField.svelte"

	let error: string | undefined

	const gotoApp = () => goto("/app").then(invalidateAll)

	const {
		form: loginForm,
		errors: loginErrors,
		isSubmitting: loginIsSubmitting,
		isValidating: loginIsValidating,
		isValid: loginIsValid,
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
			gotoApp()
		},
	})

	let passkeyError: string | undefined
	let passkeyModalOpen = false

	const {
		form: passkeyForm,
		errors: passkeyErrors,
		isSubmitting: passkeyIsSubmitting,
		isValidating: passkeyIsValidating,
		isValid: passkeyIsValid,
	} = createForm<{
		username: string
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
			return errors
		},
		onSubmit: async (values) => {
			const req = await fetch(`/webauthn/auth-start`, {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
				},
				body: JSON.stringify(values),
				credentials: "include",
			})

			if (req.status === 404) {
				throw new Error("No account or passkey found for this username")
			}

			const resp = await req.json()

			const cred = await navigator.credentials.get({
				...resp,
				publicKey: PublicKeyCredential.parseRequestOptionsFromJSON(
					resp.publicKey,
				),
			})

			if (!cred) {
				throw new Error(
					"No credentials provided. Your browser may not support passkeys.",
				)
			}

			const res = await fetch(`/webauthn/auth-finish`, {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
				},
				body: JSON.stringify(cred),
				credentials: "include",
			})
			if (!res.ok) {
				throw new Error((await res.json()).error)
			}

			localStorage.setItem("session", (await res.json()).token)
		},
		onSuccess: () => {
			gotoApp()
		},
		onError: (err) => {
			passkeyError = err.message
		},
	})

	const conditionalLoginWithPasskey = async () => {
		try {
			const req = await fetch(`/webauthn/cond/auth-start`, {
				method: "POST",
				credentials: "include",
			})

			if (req.status === 404) {
				throw new Error("No account or passkey found for this username")
			}

			const resp = await req.json()

			const cred = await navigator.credentials.get({
				...resp,
				publicKey: PublicKeyCredential.parseRequestOptionsFromJSON(
					resp.publicKey,
				),
			})

			if (!cred) {
				throw new Error(
					"No credentials provided. Your browser may not support passkeys.",
				)
			}

			const res = await fetch(`/webauthn/cond/auth-finish`, {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
				},
				body: JSON.stringify(cred),
				credentials: "include",
			})
			if (!res.ok) {
				throw new Error((await res.json()).error)
			}

			localStorage.setItem("session", (await res.json()).token)

			gotoApp()
		} catch (err) {
			console.error(err)
			passkeyError = err.message
		}
	}

	let supportsConditional =
		PublicKeyCredential.isConditionalMediationAvailable().then((s) => {
			if (s) conditionalLoginWithPasskey()
			return s
		})
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
	<form class="flex flex-col gap-2" use:loginForm>
		<TextField
			type="text"
			label="Username"
			errors={$loginErrors}
			autocomplete="username webauthn"
		/>
		<TextField
			type="password"
			label="Password"
			errors={$loginErrors}
			autocomplete="current-password webauthn"
		/>
		<p>Don't have an account yet? <a href="/register">Register one!</a></p>
		<Button
			type="submit"
			class="mt-4 w-full"
			disabled={$loginIsSubmitting || $loginIsValidating || !$loginIsValid}
			>Login</Button
		>
		{#await supportsConditional then supports}
			{#if supports}
				{#if passkeyError}
					<div class="text-error-text mt-2">{passkeyError}</div>
				{/if}
			{:else}
				<Modal bind:showModal={passkeyModalOpen}>
					<h1>Login with Passkey</h1>
					<p class="my-2">
						Your browser doesn't support Conditional UI, so please input the
						username of your account.
					</p>

					<form use:passkeyForm>
						<TextField
							type="text"
							label="Username"
							errors={$passkeyErrors}
							autocomplete="username"
						/>
						<Button
							type="submit"
							class="mt-4 w-full"
							disabled={$passkeyIsSubmitting ||
								$passkeyIsValidating ||
								!$passkeyIsValid}
							on:click={conditionalLoginWithPasskey}>Login</Button
						>
					</form>
					{#if passkeyError}
						<div class="text-error-text mt-2">{passkeyError}</div>
					{/if}
				</Modal>
			{/if}
		{/await}
	</form>
</BoxLayout>
