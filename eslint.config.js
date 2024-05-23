import eslintConfigPrettier from "eslint-config-prettier"
import eslintPluginPrettier from "eslint-plugin-prettier/recommended"
import eslintPluginSvelte from "eslint-plugin-svelte"
import eslintPluginTailwindcss from "eslint-plugin-tailwindcss"
import svelteParser from "svelte-eslint-parser"
import tsEslint from "typescript-eslint"

export default [
	...tsEslint.configs.recommended,
	...eslintPluginSvelte.configs["flat/recommended"],
	...eslintPluginSvelte.configs["flat/prettier"],
	eslintPluginPrettier,
	eslintConfigPrettier,
	{
		name: "tailwindcss:base",
		plugins: {
			get tailwindcss() {
				return eslintPluginTailwindcss
			},
		},
		languageOptions: {
			parserOptions: {
				ecmaFeatures: {
					jsx: true,
				},
			},
		},
	},
	{
		rules: {
			"sort-imports": [
				"error",
				{
					allowSeparatedGroups: true,
				},
			],
		},
	},
	{
		name: "tailwindcss:rules",
		rules: {
			"tailwindcss/classnames-order": "warn",
			"tailwindcss/enforces-negative-arbitrary-values": "warn",
			"tailwindcss/enforces-shorthand": "warn",
			"tailwindcss/migration-from-tailwind-2": "warn",
			"tailwindcss/no-arbitrary-value": "off",
			"tailwindcss/no-custom-classname": [
				"warn",
				{
					whitelist: [],
				},
			],
			"tailwindcss/no-contradicting-classname": "error",
			"tailwindcss/no-unnecessary-arbitrary-value": "warn",
		},
	},
	{
		files: ["**/*.svelte"],
		languageOptions: {
			parser: svelteParser,
			parserOptions: {
				parser: tsEslint.parser,
			},
		},
		rules: {
			"svelte/no-target-blank": "error",
			"svelte/no-at-debug-tags": "error",
			"svelte/no-reactive-functions": "error",
			"svelte/no-reactive-literals": "error",
		},
	},
	{
		files: ["**/*.ts", "**/*.tsx"],
		languageOptions: {
			parser: tsEslint.parser,
		},
	},
	{
		plugins: {
			"@typescript-eslint": tsEslint.plugin,
		},
	},
	{
		ignores: [
			".DS_Store",
			"node_modules",
			".svelte-kit",
			".env",
			".env.*",
			"!.env.example",
			"pnpm-lock.yaml",
		],
	},
]
