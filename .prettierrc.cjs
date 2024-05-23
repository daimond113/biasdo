/** @type {import('prettier').Config} */
module.exports = {
	plugins: [
		require.resolve("prettier-plugin-svelte"),
		require.resolve("prettier-plugin-tailwindcss"),
	],
	overrides: [
		{
			files: "*.svelte",
			options: {
				parser: "svelte",
			},
		},
	],
	semi: false,
	useTabs: true,
	endOfLine: "auto",
}
