<script lang="ts">
	import BoxLayout from "$lib/BoxLayout.svelte"
	import Button from "$lib/Button.svelte"
</script>

<BoxLayout class="h-0 flex-grow overflow-auto">
	<h1>Terms of Service</h1>
	<h2>Reporting content</h2>
	<p>
		If you find any content on this platform that violates these terms, please
		report it by <a href="mailto:biasdo@daimond113.com">emailing us</a>.
	</p>
	<h2>Content on the platform</h2>
	<p>
		We are not responsible for any content posted on this platform, whether it
		be text, images, or any other form of content. We do not endorse any content
		posted on this platform. If you find anything in need of removal, please
		report it.
	</p>
	<br />
	<p>
		By using this platform, you agree to not post any content that is illegal,
		or can be considered harmful, threatening, abusive, harassing or defamatory,
		vulgar, obscene, libelous, invasive of another's privacy, hateful, or
		racially, ethnically, or otherwise objectionable. You also agree to report
		any content that violates these terms.
	</p>
	<br />
	<p>
		Any content posted on this platform is the responsibility of the user who
		posted it. We are not responsible for any content posted on this platform.
		We reserve the right to remove any content for any reason, without notice.
	</p>
	<br />
	<p>
		Using this platform as a hosting service, that is, uploading files without
		the intention of sharing them with other users, is not allowed. This
		includes but is not limited to uploading files for the purpose of embedding
		them on other websites, or using the platform as a CDN.
	</p>
	<h2>Abuse</h2>
	<p>
		You agree to not harm the user experience of other users, including but not
		limited to spamming, flooding, or otherwise disrupting the platform.
		Repeatedly sending requests after the 429 status code is returned is
		considered abuse and will result in a ban, unless proven to be a bug in the
		platform or just human error without ill intent. If you believe you have
		been banned in error, please contact us by email.
	</p>
	<h2>Privacy</h2>
	<p>
		The only data we store is the data you provide us with when you register an
		account. We do not store any data that is not necessary for the operation of
		the platform. We do not share your data with any third parties (unless
		necessary, for example your email address with the email provider to send
		you an email).
	</p>
	<Button href="/" variant="primary" class="mt-4">Back</Button>
</BoxLayout>

<style>
	h2 {
		@apply my-2;
	}
</style>
