import { sveltekit } from '@sveltejs/kit/vite'
import inspect from 'vite-plugin-inspect'
import { defineConfig } from 'vite'
import tailwindcss from "@tailwindcss/vite"

export default defineConfig({
	plugins: [tailwindcss(), sveltekit(), inspect()],
	ssr: {
		noExternal: ['felte']
	}
})
