# brb (Better Recipe Book) for Horizon

Native Rust port of [Better Recipe Book](https://github.com/mrshmllow/BetterRecipeBook)
(Modrinth slug `brb`) for the [Horizon](https://github.com/horizon-rust/horizon) client.

Upstream commit `ea7c787` (2024-07-07), MIT (c) 2021 marshamallow. `LICENSE` carried verbatim.

## Layout

`src/lib.rs` is the mod API wiring: the original's config fields are
registered as settings so Horizon's Options > Mods page holds them, same
defaults as the annotated config file, and `client_setup` runs the mod entry
1:1 with `BetterRecipeBook.init()` (load `config/brb.toml`, read `brb.pins`,
initialize the pinned-recipe and instant-craft managers, `queuedScroll = 0`,
`isFilteringNone = true`, then sync the settings into the live config cells).

`src/runtime.rs` is the 1:1 core, ported from `common/src/main/java/marsh/town/brb`:

- `config/Config.java` + `NewRecipes` / `InstantCraft` / `AlternativeRecipes` /
  `Scrolling` (defaults and keys match upstream autoconfig, camelCase, and the
  file is `config/brb.toml`: upstream registers the config with the
  Toml4jConfigSerializer, so this is a correction of the earlier slice-1
  modeling that read `config/brb.json`)
- `InstantCraftingManager.java` (pending-craft bookkeeping, container-id guard,
  QUICK_MOVE click trigger, toggle)
- `PinnedRecipeManager.java` (`brb.pins` JSON array, add/remove favourite, `has`)

## Seams (game-side)

- The settings page's writes land like the config holder's; the config
  holder's load/save listeners that re-sync the instant-craft toggle button
  are the GUI seam.
- `ItemStack.isSameItemSameComponents` is modeled as value equality on
  `CraftResult { item, components }` (components = serialized component map)
- `onResultSlotUpdated`'s click is sent by game glue as QUICK_MOVE, slot 0,
  button 0, on the screen's containerId, exactly like upstream
  gameMode.handleInventoryMouseClick.
- `ResourceLocation` is modeled as its `"namespace:path"` string.
- The keybind (`key.brb.pin`, default R) and the PotionLoader level-load stay
  game-side seams.

## Gaps (mod API)

The recipe-book GUI half (the generic components and group buttons, the
brewing-stand and smithing books, toast suppression, scrollable pages,
keep-centered) needs screen, menu and keybind surfaces the pinned mod API has
not got; the brewing and smithing books also cannot be declared on the vanilla
menus while menu book declarations are limited to the registering mod's own
menus. Tracked in a `mod API:` issue on horizon-rust/horizon.
