export function getImageUrl(
	dataType: "user" | "server" | "app",
	data?: { id: string },
) {
	return `/${dataType}-icons/${(BigInt(data?.id ?? "1") >> BigInt(22)) % BigInt(4)}.svg`
}
