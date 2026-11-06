# brb (Better Recipe Book) for Horizon

Native Rust port of [Better Recipe Book](https://github.com/mrshmllow/BetterRecipeBook)
(Modrinth slug `brb`) for the [Horizon](https://github.com/horizon-rust/horizon) client.

Upstream commit `ea7c787` (2024-07-07), MIT (c) 2021 marshamallow. `LICENSE` carried verbatim.

## Slice 1: pure-logic core (this repo)

Ported 1:1 from `common/src/main/java/marsh/town/brb`:

- `config/Config.java` + `NewRecipes` / `InstantCraft` / `AlternativeRecipes` / `Scrolling`
  (defaults and JSON shape match upstream autoconfig, camelCase keys, `config/brb.json`)
- `InstantCraftingManager.java` (pending-craft bookkeeping, container-id guard,
  QUICK_MOVE click trigger, toggle)
- `PinnedRecipeManager.java` (`brb.pins` JSON array, add/remove favourite, `has`)

`cargo test` 18/18 green.

## Remaining for 1:1 parity

- GUI half: recipe book components, group buttons, recipe pages, brewing-stand and
  smithing components (upstream `generic/`, `brewingstand/`, `smithingtable/`)
- Mixins: unlock-recipes flow, toast suppression, disable-book, ungrouping, scrollable pages
- Textures/assets

## Seams (game-side)

- Config holder load/save listeners re-sync the instant-craft toggle button (GUI seam)
- `ItemStack.isSameItemSameComponents` is modeled as value equality on
  `CraftResult { item, components }` (components = serialized component map)
- `onResultSlotUpdated`'s click is sent by game glue as QUICK_MOVE, slot 0, button 0
- `ResourceLocation` is modeled as its `"namespace:path"` string
