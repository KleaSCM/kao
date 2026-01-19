<script lang="ts">
	/**
	 * Kaomoji Picker Implementation.
	 *
	 * High-performance, accessible interface for kaomoji discovery and management.
	 * Merges static bundled assets with persistent user-defined entries.
	 *
	 * UI PIPELINE:
	 * ┌───────────────┐      ┌───────────────┐      ┌───────────────┐
	 * │  User Search  │ ───► │ Fuse.js Logic │ ───► │ Filtered Grid │
	 * └───────────────┘      └───────┬───────┘      └───────┬───────┘
	 *           ▲                    │                      │
	 *           │             ┌──────▼───────┐      ┌──────▼───────┐
	 *    ┌──────┴──────┐      │ Clipboard    │ ◄─── │ Selection    │
	 *    │ Add New Form│      └──────┬───────┘      └──────┬───────┘
	 *    └─────────────┘             │                      │
	 *                         ┌──────▼───────┐      ┌──────▼───────┐
	 *                         │ Recents Bar  │ ◄─── │ Copy Logic   │
	 *                         └──────────────┘      └──────────────┘
	 *
	 * STRATEGY:
	 * - `$state` used for dynamic collection and UI focus coordination.
	 * - `$derived` handles reactive search indexing via Fuse.js.
	 * - Keyboard events intercepted at window level for full navigation parity.
	 *
	 * Author: KleaSCM
	 * Email: KleaSCM@gmail.com
	 */

	import { onMount, tick } from "svelte";
	import Fuse from "fuse.js";
	import { invoke } from "@tauri-apps/api/core";
	import type { KaomojiEntry } from "$lib/Types";
	import KaomojiDataRaw from "$lib/Data/Kaomoji.json";

	let KaomojiList = $state(KaomojiDataRaw as KaomojiEntry[]);
	let RecentKaomojis = $state<KaomojiEntry[]>([]);

	let SearchQuery = $state("");
	let LastCopied = $state("");
	let LastCopiedIndex = $state(-1);
	let ShowToast = $state(false);
	let ShowAddModal = $state(false);
	let SelectedIndex = $state(0);

	let GridEl: HTMLDivElement | null = null;
	let GridCols = $state(3);

	let NewCharacter = $state("");
	let NewTags = $state("");
	let NewCategory = $state("Joy");

	const FuseOptions = {
		includeScore: true,
		threshold: 0.3,
		ignoreLocation: true,
		keys: [
			{ name: "Tags", weight: 0.55 },
			{ name: "Category", weight: 0.25 },
			{ name: "Character", weight: 0.2 },
		],
	};

	let FuseInstance = $derived.by(() => new Fuse(KaomojiList, FuseOptions));

	let FilteredResults = $derived.by(() => {
		let query = SearchQuery.trim();
		if (!query) {
			return KaomojiList;
		}

		// STRATEGY: Unicode-Aware Token Parsing
		const tokenRegex = /(?:^|\s)(cat|tag):([^\s]+)/g;
		let hasTokens = false;
		let filtered = [...KaomojiList];
		let remainingQuery = query;

		tokenRegex.lastIndex = 0;
		let match;
		while ((match = tokenRegex.exec(query)) !== null) {
			hasTokens = true;
			const [fullMatch, type, value] = match;
			// Sanitize punctuation for clean matching
			const sanitizedValue = value
				.replace(/[^\p{L}\p{N}_-]/gu, "")
				.toLowerCase();

			if (type === "cat") {
				filtered = filtered.filter(
					(k) => k.Category.toLowerCase() === sanitizedValue,
				);
			} else if (type === "tag") {
				filtered = filtered.filter((k) =>
					k.Tags.some((t) => t.toLowerCase() === sanitizedValue),
				);
			}
			remainingQuery = remainingQuery.replace(fullMatch, "").trim();
		}

		if (hasTokens) {
			if (remainingQuery.length > 0) {
				const fuseInner = new Fuse(filtered, FuseOptions);
				filtered = fuseInner.search(remainingQuery).map((r) => r.item);
			}
			return filtered;
		}

		const searchResults = FuseInstance.search(query);
		return searchResults.map((r) => r.item);
	});

	// RANGING CONTRACT: SelectedIndex must always point to a valid result
	$effect(() => {
		const count = FilteredResults.length;
		if (count === 0) {
			SelectedIndex = 0;
		} else if (SelectedIndex >= count) {
			SelectedIndex = count - 1;
		}
	});

	// BINDING CONTRACT: Ensure cols are recomputed when grid is ready
	$effect(() => {
		if (GridEl) {
			RecomputeCols();
		}
	});

	function RecomputeCols() {
		if (!GridEl) return;

		const styles = getComputedStyle(GridEl);
		const gap = parseFloat(styles.columnGap || styles.gap || "0") || 0;

		// Card min width is 100px per our CSS
		const minCard = 100;
		const width = GridEl.clientWidth;

		GridCols = Math.max(1, Math.floor((width + gap) / (minCard + gap)));
	}

	onMount(() => {
		// ASYNC INITIALIZATION STRATEGY
		(async () => {
			try {
				const userItems: KaomojiEntry[] =
					await invoke("LoadUserKaomojis");
				if (userItems && userItems.length > 0) {
					const existingChars = new Set(
						KaomojiList.map((k) => k.Character),
					);
					const uniqueUserItems = userItems.filter(
						(k) => !existingChars.has(k.Character),
					);
					KaomojiList = [...KaomojiList, ...uniqueUserItems];
				}
			} catch (err) {
				console.error("Failed to load user kaomojis", err);
			}

			RecomputeCols();
		})();

		const ro = new ResizeObserver(() => RecomputeCols());
		if (GridEl) ro.observe(GridEl);

		return () => ro.disconnect();
	});

	async function CopyToClipboard(character: string, index: number = -1) {
		let successState = false;

		try {
			await invoke("CopyToClipboard", { text: character });
			successState = true;
		} catch (err) {
			console.warn("Native copy failed, trying fallback...", err);
		}

		if (!successState) {
			try {
				await navigator.clipboard.writeText(character);
				successState = true;
			} catch (err) {
				console.warn("Navigator copy failed", err);
			}
		}

		if (successState) {
			const entry = KaomojiList.find((k) => k.Character === character);
			if (entry) {
				AddToRecents(entry);
			}
			HandleCopySuccess(character, index);
		}
	}

	function AddToRecents(entry: KaomojiEntry) {
		RecentKaomojis = [
			entry,
			...RecentKaomojis.filter((k) => k.Character !== entry.Character),
		].slice(0, 8);
	}

	function HandleCopySuccess(character: string, index: number) {
		LastCopied = character;
		LastCopiedIndex = index;
		ShowToast = false;
		tick().then(() => {
			ShowToast = true;
			setTimeout(() => {
				ShowToast = false;
				LastCopiedIndex = -1;
			}, 2000);
		});
	}

	async function SubmitNewKaomoji() {
		if (!NewCharacter.trim()) {
			return;
		}

		const entry: KaomojiEntry = {
			Character: NewCharacter.trim(),
			Tags: NewTags.split(",")
				.map((t) => t.trim())
				.filter((t) => t !== ""),
			Category: NewCategory,
		};

		try {
			await invoke("SaveKaomoji", { newEntry: entry });

			// IMMUTABLE UPSERT STRATEGY
			const index = KaomojiList.findIndex(
				(k) => k.Character === entry.Character,
			);
			if (index !== -1) {
				KaomojiList = [
					...KaomojiList.slice(0, index),
					entry,
					...KaomojiList.slice(index + 1),
				];
			} else {
				KaomojiList = [...KaomojiList, entry];
			}

			// Reset form
			NewCharacter = "";
			NewTags = "";
			ShowAddModal = false;
		} catch (err) {
			console.error("Failed to save kaomoji", err);
		}
	}

	function HandleKeyDown(event: KeyboardEvent) {
		if (ShowAddModal) {
			if (event.key === "Escape") {
				ShowAddModal = false;
			}
			return;
		}

		const target = event.target as HTMLElement | null;
		const isTyping =
			target &&
			(target.tagName === "INPUT" || target.tagName === "TEXTAREA");

		const navKeys = ["ArrowRight", "ArrowLeft", "ArrowDown", "ArrowUp"];
		const handledKeys = [...navKeys, "Enter", "Escape"];

		if (isTyping && navKeys.includes(event.key)) {
			return;
		}

		if (handledKeys.includes(event.key)) {
			event.preventDefault();
		}

		const itemsCount = FilteredResults.length;
		if (itemsCount === 0) return;

		switch (event.key) {
			case "ArrowRight":
				SelectedIndex = (SelectedIndex + 1) % itemsCount;
				ScrollToSelected();
				break;
			case "ArrowLeft":
				SelectedIndex = (SelectedIndex - 1 + itemsCount) % itemsCount;
				ScrollToSelected();
				break;
			case "ArrowDown":
				SelectedIndex = Math.min(
					SelectedIndex + GridCols,
					itemsCount - 1,
				);
				ScrollToSelected();
				break;
			case "ArrowUp":
				SelectedIndex = Math.max(SelectedIndex - GridCols, 0);
				ScrollToSelected();
				break;
			case "Enter":
				if (FilteredResults[SelectedIndex]) {
					CopyToClipboard(
						FilteredResults[SelectedIndex].Character,
						SelectedIndex,
					);
				}
				break;
			case "Escape":
				SearchQuery = "";
				break;
		}
	}

	function ScrollToSelected() {
		tick().then(() => {
			const selectedEl = GridEl?.querySelector(".KaomojiCard.IsSelected");
			if (selectedEl) {
				selectedEl.scrollIntoView({
					block: "nearest",
					behavior: "smooth",
				});
			}
		});
	}
</script>

<svelte:window onkeydown={HandleKeyDown} />

<div class="MainContainer">
	<div class="ActionHeader">
		<div class="SearchWrapper">
			<input
				type="text"
				placeholder="Search (try cat:Joy or tag:cute)..."
				bind:value={SearchQuery}
				class="SearchInput GlassCard"
			/>
		</div>
		<button
			class="AddButton GlassCard"
			onclick={() => (ShowAddModal = true)}
			aria-label="Add new kaomoji"
		>
			＋
		</button>
	</div>

	{#if RecentKaomojis.length > 0 && !SearchQuery}
		<div class="RecentsSection">
			<span class="Label">Recent ✨</span>
			<div class="RecentsGrid">
				{#each RecentKaomojis as recent}
					<button
						class="RecentItem GlassCard"
						onclick={() => CopyToClipboard(recent.Character)}
					>
						{recent.Character}
					</button>
				{/each}
			</div>
		</div>
	{/if}

	<div class="GridContainer" bind:this={GridEl}>
		{#each FilteredResults as item, i}
			<button
				class="KaomojiCard GlassCard"
				class:IsLarge={item.Category === "Large"}
				class:IsSelected={SelectedIndex === i}
				class:IsCopied={LastCopiedIndex === i}
				onclick={() => {
					SelectedIndex = i;
					CopyToClipboard(item.Character, i);
				}}
				title={item.Tags.join(", ")}
				tabindex={SelectedIndex === i ? 0 : -1}
			>
				{item.Character}
			</button>
		{/each}
	</div>

	{#if ShowAddModal}
		<div
			class="ModalOverlay"
			onclick={() => (ShowAddModal = false)}
			onkeydown={(e) => e.key === "Escape" && (ShowAddModal = false)}
			role="button"
			tabindex="-1"
			aria-label="Close modal overlay"
		>
			<div
				class="ModalContent GlassCard"
				onclick={(e) => e.stopPropagation()}
				onkeydown={(e) => e.key === "Enter" && SubmitNewKaomoji()}
				role="dialog"
				aria-modal="true"
				tabindex="0"
			>
				<h3>Add somehting</h3>

				<div class="FormGroup">
					<label for="char">Character</label>
					<textarea
						id="char"
						bind:value={NewCharacter}
						placeholder="(˶ˆᗜˆ˵)"
					></textarea>
				</div>

				<div class="FormGroup">
					<label for="tags">Tags (comma separated)</label>
					<input
						id="tags"
						type="text"
						bind:value={NewTags}
						placeholder="happy, blush, cute"
					/>
				</div>

				<div class="FormGroup">
					<label for="cat">Category</label>
					<select id="cat" bind:value={NewCategory}>
						<option value="Joy">Joy</option>
						<option value="Love">Love</option>
						<option value="Angry">Angry</option>
						<option value="Sadness">Sadness</option>
						<option value="Animals">Animals</option>
						<option value="Decoration">Decoration</option>
						<option value="Large">Large (ASCII)</option>
					</select>
				</div>

				<div class="ModalActions">
					<button
						class="CancelBtn"
						onclick={() => (ShowAddModal = false)}>Cancel</button
					>
					<button class="SaveBtn" onclick={SubmitNewKaomoji}
						>Save ✨</button
					>
				</div>
			</div>
		</div>
	{/if}

	{#if ShowToast}
		<div class="Toast GlassCard">
			<span
				>Copied {LastCopied.length > 20 ? "ASCII Art" : LastCopied}! 💕</span
			>
		</div>
	{/if}
</div>

<style>
	.MainContainer {
		overflow-x: hidden; /* Ensure no horizontal scroll */
		display: flex;
		flex-direction: column;
		height: 100vh;
	}

	.ActionHeader {
		display: flex;
		gap: var(--SpacingUnit);
		margin-bottom: var(--SpacingUnit);
		width: 100%;
	}

	.SearchWrapper {
		flex-grow: 1;
	}

	.SearchInput {
		width: 100%;
		padding: calc(var(--SpacingUnit) * 1.5);
		border: none;
		outline: none;
		font-size: 1rem;
		color: var(--ColorText);
		transition: var(--TransitionFast);
	}

	.SearchInput:focus {
		box-shadow: 0 0 15px var(--ColorPrimary);
		border: 1px solid var(--ColorPrimary);
		transform: scale(1.01);
	}

	.AddButton {
		border: none;
		width: 50px;
		font-size: 1.5rem;
		cursor: pointer;
		color: var(--ColorText);
		display: flex;
		align-items: center;
		justify-content: center;
		transition: var(--TransitionFast);
	}

	.AddButton:hover {
		background: var(--ColorPrimary);
		color: white;
		transform: rotate(90deg) scale(1.1);
	}

	.RecentsSection {
		margin-bottom: var(--SpacingUnit);
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.RecentsSection .Label {
		font-size: 0.7rem;
		font-weight: bold;
		opacity: 0.7;
		margin-left: 4px;
	}

	.RecentsGrid {
		display: flex;
		gap: 8px;
		overflow-x: auto;
		overflow-y: hidden;
		padding: 4px;
		scrollbar-width: none;
		flex-wrap: nowrap;
		-webkit-overflow-scrolling: touch;
	}

	.RecentsGrid::-webkit-scrollbar {
		display: none;
	}

	.RecentItem {
		border: none;
		padding: 6px 12px;
		font-size: 0.9rem;
		cursor: pointer;
		color: var(--ColorText);
		white-space: nowrap;
		transition: var(--TransitionFast);
		flex-shrink: 0;
	}

	.RecentItem:hover {
		background: var(--ColorPrimary);
		color: white;
		transform: translateY(-2px);
	}

	.GridContainer {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
		gap: var(--SpacingUnit);
		overflow-y: auto;
		overflow-x: hidden;
		flex-grow: 1;
		padding: 4px; /* Tiny space for glow effects */
		width: 100%;
	}

	.KaomojiCard {
		border: none;
		padding: var(--SpacingUnit);
		font-size: 1.1rem;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: var(--TransitionFast);
		color: var(--ColorText);
		min-height: 50px;
		white-space: pre;
		overflow: hidden;
		border: 2px solid transparent;
	}

	.KaomojiCard.IsSelected {
		border-color: var(--ColorPrimary);
		background: rgba(255, 133, 161, 0.1);
		box-shadow: 0 0 10px rgba(255, 133, 161, 0.3);
		transform: scale(1.02);
	}

	.KaomojiCard.IsCopied {
		animation: Pulse 0.4s ease-out;
		background: var(--ColorPrimary) !important;
		color: white !important;
	}

	@keyframes Pulse {
		0% {
			transform: scale(1.02);
		}
		50% {
			transform: scale(1.15);
			box-shadow: 0 0 20px var(--ColorPrimary);
		}
		100% {
			transform: scale(1.05);
		}
	}

	.KaomojiCard.IsLarge {
		grid-column: 1 / -1; /* Span full width */
		font-size: 6px;
		line-height: 1.1;
		padding: calc(var(--SpacingUnit) * 2);
		overflow: auto;
		background: rgba(var(--ColorPrimary), 0.05);
		font-family: "Courier New", Courier, monospace;
		justify-content: flex-start;
		align-items: flex-start;
		text-align: left;
		width: 100%;
	}

	.KaomojiCard:hover {
		transform: scale(1.05);
		background: var(--ColorPrimary);
		color: white;
		box-shadow: 0 5px 15px rgba(255, 133, 161, 0.4);
		z-index: 10;
	}

	.KaomojiCard:active {
		transform: scale(0.95);
	}

	/* Modal Styles */
	.ModalOverlay {
		position: fixed;
		top: 0;
		left: 0;
		width: 100%;
		height: 100%;
		background: rgba(0, 0, 0, 0.6);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 200;
		backdrop-filter: blur(6px);
	}

	.ModalContent {
		width: 85%;
		max-width: 400px;
		padding: calc(var(--SpacingUnit) * 3);
		display: flex;
		flex-direction: column;
		gap: var(--SpacingUnit);
		animation: SlideUp 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275);
		background: var(--ColorSurface);
		border: 1px solid var(--ColorPrimary);
		box-shadow: 0 15px 50px rgba(0, 0, 0, 0.3);
	}

	.FormGroup {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.FormGroup label {
		font-size: 0.85rem;
		font-weight: bold;
		color: var(--ColorText);
	}

	.FormGroup textarea,
	.FormGroup input,
	.FormGroup select {
		padding: calc(var(--SpacingUnit) * 1.2);
		border-radius: 8px;
		border: 1px solid rgba(var(--ColorPrimary), 0.3);
		background: #ffffff;
		color: var(--ColorText);
		outline: none;
		font-size: 0.95rem;
	}

	.FormGroup textarea:focus,
	.FormGroup input:focus,
	.FormGroup select:focus {
		border-color: var(--ColorPrimary);
		box-shadow: 0 0 5px var(--ColorPrimary);
	}

	.FormGroup textarea {
		height: 80px;
		font-family: monospace;
		resize: none;
	}

	.ModalActions {
		display: flex;
		gap: var(--SpacingUnit);
		margin-top: var(--SpacingUnit);
	}

	.ModalActions button {
		flex: 1;
		padding: var(--SpacingUnit);
		border: none;
		border-radius: 8px;
		cursor: pointer;
		font-weight: bold;
	}

	.CancelBtn {
		background: rgba(0, 0, 0, 0.05);
		color: var(--ColorText);
	}

	.CancelBtn:hover {
		background: rgba(0, 0, 0, 0.1);
	}

	.SaveBtn {
		background: var(--ColorPrimary);
		color: white;
	}

	.SaveBtn:hover {
		transform: translateY(-2px);
		box-shadow: 0 4px 12px var(--ColorPrimary);
	}

	@keyframes SlideUp {
		from {
			opacity: 0;
			transform: translateY(30px) scale(0.9);
		}
		to {
			opacity: 1;
			transform: translateY(0) scale(1);
		}
	}

	.Toast {
		position: fixed;
		bottom: 20px;
		left: 50%;
		transform: translateX(-50%);
		padding: var(--SpacingUnit) calc(var(--SpacingUnit) * 2);
		background: var(--ColorAccent);
		color: white;
		z-index: 100;
		animation: FadeIn 0.3s ease-out;
		pointer-events: none;
	}

	@keyframes FadeIn {
		from {
			opacity: 0;
			transform: translate(-50%, 20px);
		}
		to {
			opacity: 1;
			transform: translate(-50%, 0);
		}
	}
</style>
