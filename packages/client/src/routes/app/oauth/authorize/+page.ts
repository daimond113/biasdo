import type { PageLoad } from "./$types"
import { error } from "@sveltejs/kit"

const SCOPE_TO_DESCRIPTION = {
	profile: {
		read: "Read your account information",
		write: "Update your account information",
	},
	servers: { read: "Read the servers you're in", write: "Update your servers" },
	messages: {
		read: "Read the messages you can see",
		write: "Write messages as you",
	},
	friends: {
		read: "See who you're friends with",
		write: "Manage your friends",
	},
}

const SCOPE_ORDER: (keyof typeof SCOPE_TO_DESCRIPTION)[] = [
	"profile",
	"servers",
	"messages",
	"friends",
] as const

const PERM_ORDER: ["read", "write"] = ["read", "write"]

export const load: PageLoad = async ({ fetch, url }) => {
	const session = localStorage.getItem("session")
	if (!session) {
		error(401, "You must be logged in to authorize a client")
	}

	const uriAuthorize = `${import.meta.env.VITE_API_URL}/clients/authorize?${url.searchParams}`

	const clientDataRequest = await fetch(uriAuthorize, {
		headers: {
			Authorization: session,
		},
	})

	if (!clientDataRequest.ok) {
		error(clientDataRequest.status, await clientDataRequest.text())
	}

	const redirectUri = url.searchParams.get("redirect_uri")!

	const redirectUriDecline = new URL(redirectUri)
	redirectUriDecline.searchParams.set("error", "access_denied")
	redirectUriDecline.searchParams.set(
		"error_description",
		"The user denied the request",
	)

	// if the user has specified a write scope, the read permission is also assumed to be present
	return {
		client: {
			...((await clientDataRequest.json()) as {
				client_name: string
				client_uri: string
				tos_uri: string
				policy_uri: string
				redirect_uris: string[]
			}),
			id: url.searchParams.get("client_id")!,
		},
		redirectUri,
		uriDecline: redirectUriDecline.toString(),
		uriAuthorize,
		scopes: url.searchParams
			.get("scope")!
			.split(" ")
			.filter((s) => s.trim() !== "")
			.map(
				(s) =>
					s.split(".") as [keyof typeof SCOPE_TO_DESCRIPTION, "read" | "write"],
			)
			.flatMap(([scope, permission]) =>
				permission === "write"
					? ([
							[scope, "read"],
							[scope, "write"],
						] as const)
					: ([[scope, "read"]] as const),
			)
			.map(([s, perm]) => [s, perm, SCOPE_TO_DESCRIPTION[s][perm]] as const)
			.toSorted(([a, aPerm], [b, bPerm]) => {
				const aIndex = SCOPE_ORDER.indexOf(a)
				const bIndex = SCOPE_ORDER.indexOf(b)

				if (aIndex === bIndex) {
					const aPermIndex = PERM_ORDER.indexOf(aPerm)
					const bPermIndex = PERM_ORDER.indexOf(bPerm)

					return aPermIndex - bPermIndex
				}

				return aIndex - bIndex
			})
			.map((a) => a[2]),
	}
}
