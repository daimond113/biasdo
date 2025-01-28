<script lang="ts">
	import { invalidateAll, me } from "$lib/stores"
	import { createForm } from "felte"
	import { fetch } from "$lib/fetch"
	import { get } from "svelte/store"
	import { goto } from "$app/navigation"

	import Button from "$lib/Button.svelte"
	import TextField from "$lib/TextField.svelte"
	import X from "lucide-svelte/icons/x"

	const { form, errors, isValid, isValidating, isSubmitting, setFields } =
		createForm<{
			username?: string
			display_name?: string
			email?: string
			password?: string
		}>({
			initialValues: {
				username: get(me)?.username,
				display_name: get(me)?.display_name ?? undefined,
				email: get(me)?.email,
				password: "",
			},
			validate: (values) => {
				if (
					Object.keys(values).filter(
						(name) => values[name as keyof typeof values],
					).length === 0
				)
					return { username: "At least one field must be filled in" }

				const errors = {} as Record<string, string>

				if (values.email) {
					if (
						!/^(([^<>()\[\]\\.,;:\s@"]+(\.[^<>()\[\]\\.,;:\s@"]+)*)|(".+"))@((\[[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}])|(([a-zA-Z\-0-9]+\.)+[a-zA-Z]{2,}))$/.test(
							values.email,
						)
					) {
						errors.email = "Email is invalid"
					} else if (values.email.length > 255) {
						errors.email = "Email must be at most 255 characters long"
					}
				}

				if (values.username) {
					const username = values.username.trim()
					if (username.length < 3) {
						errors.username = "Username must be at least 3 characters long"
					} else if (username.length > 32) {
						errors.username = "Username must be at most 32 characters long"
					} else if (!/^[a-zA-Z0-9_]+$/g.test(values.username)) {
						errors.username =
							"Username can only contain lowercase letters, numbers, and underscores"
					}
				}

				if (values.display_name) {
					const display_name = values.display_name.trim()
					if (display_name.length < 2) {
						errors.display_name =
							"Display name must be at least 2 characters long"
					} else if (display_name.length > 32) {
						errors.display_name =
							"Display name must be at most 32 characters long"
					}
				}

				if (values.password) {
					if (values.password.length < 8) {
						errors.password = "Password must be at least 8 characters long"
					} else if (values.password.length > 128) {
						errors.password = "Password must be at most 128 characters long"
					}
				}

				return errors
			},
			onSubmit: async (values) => {
				return await fetch(`/users/@me`, {
					method: "PATCH",
					headers: {
						"Content-Type": "application/json",
					},
					body: JSON.stringify({
						...Object.fromEntries(
							Object.entries(values).filter(
								([k, v]) => v && v !== (get(me) as never)[k],
							),
						),
						display_name: values.display_name || null,
					}),
				})
			},
			onSuccess: () => {
				setFields("password", "")
			},
		})

	$: setFields({
		username: $me?.username,
		display_name: $me?.display_name ?? undefined,
		email: $me?.email,
	})

	let passkeyError: string | undefined
	const addPasskey = async () => {
		const req = await fetch(`/webauthn/register-start`, {
			method: "POST",
			credentials: "include",
		})

		const resp = await req.json()

		let cred: Credential | null = null
		try {
			cred = await navigator.credentials.create({
				...resp,
				publicKey: PublicKeyCredential.parseCreationOptionsFromJSON(
					resp.publicKey,
				),
			})
		} catch (e) {
			console.error(e)
			passkeyError = e.message
			return
		}

		if (!cred) {
			passkeyError =
				"No credential returned. Your browser may not support passkeys."
			return
		}

		try {
			await fetch(`/webauthn/register-finish`, {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
				},
				body: JSON.stringify(cred),
				credentials: "include",
			})
		} catch (e) {
			console.error(e)
			passkeyError = e.message
		}
	}
</script>

<svelte:head>
	<title>User Settings - biasdo</title>
</svelte:head>

<div
	class="border-paper-1-outline bg-paper-1-bg flex size-full shrink-0 flex-col gap-4 overflow-auto rounded-2xl border pt-16"
>
	<div class="max-w-[48rem] px-16">
		<h1>User Settings</h1>
		<form use:form class="mt-2">
			<TextField
				label="Username"
				class="mb-2 w-full"
				name="username"
				errors={$errors}
				autocomplete="username"
			>
				<Button
					class="ml-2 flex size-10 items-center justify-center p-2"
					title="Reset field"
					onClick={() => setFields("username", get(me)?.username)}
					variant="error"
				>
					<X />
				</Button>
			</TextField>
			<TextField
				label="Display Name"
				class="mb-2 w-full"
				name="display_name"
				errors={$errors}
				autocomplete="name"
			>
				<Button
					class="ml-2 flex size-10 items-center justify-center p-2"
					title="Reset field"
					onClick={() =>
						setFields("display_name", get(me)?.display_name ?? undefined)}
					variant="error"
				>
					<X />
				</Button>
			</TextField>
			<TextField
				label="Email"
				class="mb-2 w-full"
				name="email"
				errors={$errors}
				autocomplete="email"
			>
				<Button
					class="ml-2 flex size-10 items-center justify-center p-2"
					title="Reset field"
					onClick={() => setFields("email", get(me)?.email)}
					variant="error"
				>
					<X />
				</Button>
			</TextField>
			<TextField
				label="Password"
				type="password"
				class="mb-2 w-full"
				name="password"
				errors={$errors}
				autocomplete="new-password"
			/>
			<Button
				type="submit"
				disabled={$isValidating || $isSubmitting || !$isValid}>Update</Button
			>
			<div class="border-paper-1-outline mt-2 rounded-md border-2 p-4">
				<Button onClick={addPasskey} variant="secondary">Add Passkey</Button>
				{#if passkeyError}
					<p class="text-error-text">{passkeyError}</p>
				{/if}
			</div>
		</form>
	</div>
	<div class="bg-error-bg/40 text-error-text mt-auto w-full px-16 py-8">
		<h2>Danger Zone</h2>
		<p>These actions cannot be undone. Be careful!</p>
		<div class="mt-4 flex gap-4 overflow-x-auto">
			<Button
				onClick={() => {
					fetch(`/users/@me`, {
						method: "DELETE",
					})
				}}
				variant="error"
			>
				Delete Account
			</Button>
			<Button
				onClick={async () => {
					await fetch(`/logout`, {
						method: "POST",
					})

					localStorage.removeItem("session")
					invalidateAll()
					goto("/")
				}}
				variant="error">Logout</Button
			>
			<Button
				onClick={async () => {
					await fetch(`/logout?all=true`, {
						method: "POST",
					})

					localStorage.removeItem("session")
					invalidateAll()
					goto("/")
				}}
				variant="error">Logout All Sessions</Button
			>
		</div>
	</div>
</div>
