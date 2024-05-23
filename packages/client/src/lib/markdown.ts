import "katex/dist/katex.min.css"
import { Carta } from "carta-md"
import DOMPurify from "dompurify"
import { code } from "@cartamd/plugin-code"
import { math } from "@cartamd/plugin-math"

DOMPurify.addHook("afterSanitizeAttributes", function (node) {
	if (node.tagName === "A") {
		node.setAttribute("target", "_blank")
		node.setAttribute("rel", "noopener noreferrer")
	}
})

export const carta = new Carta({
	sanitizer: (html) =>
		DOMPurify.sanitize(html, {
			FORBID_ATTR: ["src"],
			// currently don't allow images, videos, audio etc. because they can be used to leak information like IP addresses. bring back when we have a proxy
			FORBID_TAGS: [
				"img",
				"video",
				"audio",
				"iframe",
				"object",
				"embed",
				"canvas",
				"source",
			],
		}),
	disableIcons: true,
	theme: "monokai",
	extensions: [code(), math()],
})
