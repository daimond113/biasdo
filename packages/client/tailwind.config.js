import defaultTheme from "tailwindcss/defaultTheme"
import typography from "@tailwindcss/typography"

/** @type {import('tailwindcss').Config} */
export default {
	content: ["./src/**/*.{html,js,svelte,ts,rs}"],
	theme: {
		extend: {
			colors: {
				background: "#111311",
				text: "#CAD8C6",
				alt: {
					text: "#94A98E",
				},
				placeholder: {
					text: "#C6D8C899",
				},
				link: "#35e34c",
				paper: {
					1: {
						bg: "#181C17",
						outline: "#3E4A3A",
					},
					2: {
						bg: "#262E23",
						active: "#364931",
					},
				},
				error: {
					bg: "#450c0c",
					"bg-hover": "#5f0d0d",
					text: "#FFA6A6",
				},
				scrollbar: {
					track: "#3E4A3A",
					thumb: "#94A98E",
					"thumb-hover": "#364931",
				},
			},
			borderRadius: {
				"paper-1": "0.875rem",
			},
			fontFamily: {
				sans: ["Inter Variable", ...defaultTheme.fontFamily.sans],
			},
			typography: ({ theme }) => ({
				biasdo: {
					css: {
						"--tw-prose-body": theme("colors.text"),
						"--tw-prose-headings": theme("colors.text"),
						"--tw-prose-lead": theme("colors.green[100]"),
						"--tw-prose-links": theme("colors.link"),
						"--tw-prose-bold": theme("colors.green[400]"),
						"--tw-prose-counters": theme("colors.green[400]"),
						"--tw-prose-bullets": theme("colors.text"),
						"--tw-prose-hr": theme("colors.green[100]"),
						"--tw-prose-quotes": theme("colors.green[300]"),
						"--tw-prose-quote-borders": theme("colors.green[500]"),
						"--tw-prose-captions": theme("colors.green[300]"),
						"--tw-prose-th-borders": theme("colors.green[500]"),
						"--tw-prose-td-borders": theme("colors.text"),
						"--tw-prose-code": theme("colors.green[300]"),
					},
				},
			}),
		},
	},
	plugins: [typography()],
}
