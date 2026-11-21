//! Better Recipe Book (brb) for Horizon: the mod API wiring of the 1:1 core.
//!
//! The 1:1 logic core lives in `runtime.rs`: the original's config model
//! (Config with its NewRecipes / InstantCraft / AlternativeRecipes / Scrolling
//! sections, camelCase keys matching the upstream autoconfig serializer), the
//! instant-craft bookkeeping (InstantCraftingManager: pending-craft state, the
//! container-id guard, the QUICK_MOVE click trigger) and the pinned-recipe
//! manager (brb.pins, a Gson-shaped JSON array).
//!
//! The mod API wires the original's config as settings so Horizon's
//! Options > Mods page holds it, same defaults as the annotated config file,
//! and runs the mod entry 1:1 with `BetterRecipeBook.init()`: load
//! `config/brb.toml` (the upstream Toml4jConfigSerializer's file), read
//! brb.pins, initialize the managers, `queuedScroll = 0` and
//! `isFilteringNone = true`, then sync the settings into the live config
//! cells.
//!
//! Gaps, tracked: the recipe-book GUI half (the generic components and group
//! buttons, the brewing-stand and smithing books, toast suppression,
//! scrollable pages, keep-centered) and the pin keybind need screen, menu and
//! keybind surfaces the pinned mod API has not got; the brewing and smithing
//! books also cannot be declared on the vanilla menus while menu book
//! declarations are limited to the registering mod's own menus.
use mod_api::{ClientSetupEvent, ClientHost, RegistrationContext, Registrar, SettingSpec};

mod runtime;


/// The mod's id: its settings' namespace.
const MOD_ID: &str = "brb";

/// The original's config fields registered as settings, same defaults as the
/// annotated config file: (key, label, default).
const SETTINGS: &[(&str, &str, bool)] = &[
    ("enable_book", "setting.brb.enable_book", true),
    ("enable_pinning", "setting.brb.enable_pinning", true),
    ("keep_centered", "setting.brb.keep_centered", false),
    ("settings_button", "setting.brb.settings_button", true),
    ("new_recipes_unlock_all", "setting.brb.new_recipes_unlock_all", true),
    ("new_recipes_enable_bounce", "setting.brb.new_recipes_enable_bounce", false),
    ("instant_craft_show_button", "setting.brb.instant_craft_show_button", true),
    ("instant_craft_enabled", "setting.brb.instant_craft_enabled", false),
    ("alternative_recipes_on_hover", "setting.brb.alternative_recipes_on_hover", true),
    ("alternative_recipes_no_grouped", "setting.brb.alternative_recipes_no_grouped", false),
    ("scrolling_enable_scrolling", "setting.brb.scrolling_enable_scrolling", true),
    ("scrolling_scroll_around", "setting.brb.scrolling_scroll_around", false),
];

#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl mod_api::Hooks for Mod {
    /// The original's config fields registered as settings, same defaults.
    fn register(&self, registrar: &mut dyn Registrar, _context: &RegistrationContext) {
        for &(key, label, default) in SETTINGS {
            let _ = registrar.register_setting(SettingSpec::toggle(key, label, default));
        }
    }

    /// 1:1 of the original's mod init (`BetterRecipeBook.init()`): load
    /// `config/brb.toml` (defaults when missing or broken, as the autoconfig
    /// holder keeps), sync the settings into the live config cells, then
    /// read brb.pins and initialize the managers.
    fn client_setup(&self, host: &mut dyn ClientHost, _event: &mut ClientSetupEvent) {
        let config = runtime::load_config(&runtime::game_config_dir()).unwrap_or_default();
        runtime::set_active_config(config);
        for (key, _, default) in SETTINGS {
            let value = host.setting_bool(key).unwrap_or(*default);
            runtime::set_flag_active(key, value);
        }
        runtime::init_managers(&runtime::active_config());
    }
}
