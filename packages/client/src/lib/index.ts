import { FelteSubmitError } from 'felte'
import { cn } from './cn'

const variantToColors = {
	[1]: {
		background: 'bg-[var(--paper-level-1)]',
		outline: 'border-[var(--paper-level-1-outline)]'
	},
	error: {
		background: 'bg-[var(--error-paper)]',
		outline: 'border-[var(--error-paper-outline)]',
		text: 'text-[var(--error-paper-text)]'
	}
}

export function getPaperStyles(variant: keyof typeof variantToColors, className?: string) {
	return cn(Object.values(variantToColors[variant]), 'border rounded-lg', className)
}

export async function credentialSubmitHandler(form: HTMLFormElement): Promise<unknown> {
	let body: FormData | URLSearchParams = new FormData(form)
	const action = new URL(form.action)
	const method =
		form.method.toLowerCase() === 'get' ? 'get' : action.searchParams.get('_method') || form.method
	let enctype = form.enctype

	if (form.querySelector('input[type="file"]')) {
		enctype = 'multipart/form-data'
	}
	if (method === 'get' || enctype === 'application/x-www-form-urlencoded') {
		body = new URLSearchParams(body as never)
	}

	let fetchOptions: RequestInit

	if (method === 'get') {
		;(body as URLSearchParams).forEach((value, key) => {
			action.searchParams.append(key, value)
		})
		fetchOptions = { method, headers: { Accept: 'application/json' } }
	} else {
		fetchOptions = {
			method,
			body,
			headers: {
				// If `Content-Type` is set on multipart/form-data, boundary will be missing
				// See: https://github.com/pablo-abc/felte/issues/165
				...(enctype !== 'multipart/form-data' && {
					'Content-Type': enctype
				}),
				Accept: 'application/json'
			},
			credentials: 'include'
		}
	}

	const response = await window.fetch(action.toString(), fetchOptions)

	if (response.ok) return response
	throw new FelteSubmitError('An error occurred while submitting the form', response as never)
}

// can this be made quicker and more performant? set approach loses order
export const dedupe = <T extends { id: string }>(arr: T[]) =>
	arr.filter(({ id }, index) => arr.findIndex(({ id: idB }) => id === idB) === index)
