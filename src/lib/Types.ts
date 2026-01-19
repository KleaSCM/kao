/**
 * Type Definition Module.
 *
 * Formalizes the data structures for kaomoji assets.
 * Ensures consistent typing for fuzzy search integration and UI rendering.
 *
 * DATA STRUCTURE:
 * ┌───────────────┐
 * │ KaomojiEntry  │
 * ├───────────────┤
 * │ - Character   │
 * │ - Tags        │
 * │ - Category    │
 * └───────────────┘
 *
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 */

export interface KaomojiEntry {
    Character: string;
    Tags: string[];
    Category: string;
}
