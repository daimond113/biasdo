import MarkdownIt from 'markdown-it'
import Shikiji from 'markdown-it-shikiji'
import KaTeX from 'markdown-it-katex'
import 'katex/dist/katex.min.css'
import { writable } from 'svelte/store'

// nasty hack to get around the fact that the markdown-it-shikiji is async
export const md = writable<MarkdownIt | undefined>(undefined)

const it = MarkdownIt({
	linkify: false
}).use(KaTeX)

Promise.all([Shikiji({ theme: 'one-dark-pro' })]).then((plugins) => {
	for (const plugin of plugins) {
		it.use(plugin)
	}

	md.set(it)
})
