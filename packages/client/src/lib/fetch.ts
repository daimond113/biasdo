import { goto } from "$app/navigation"

export const fetch = async (path: `/${string}`, options: RequestInit = {}) => {
	const response = await window.fetch(
		`${import.meta.env.VITE_API_URL}${path}`,
		{
			...options,
			headers: {
				...options.headers,
				Authorization: localStorage.getItem("session")!,
			},
		},
	)

	if (!response.ok) {
		if (response.status === 401) {
			goto("/login")
		}
		if (response.status === 429) {
			goto("/rate-limit")
		}
	}

	return response
}
