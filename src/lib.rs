//! brb (Better Recipe Book) for Horizon: the mod API wiring of the slice-1 core.
//!
//! Upstream (mrshmllow/BetterRecipeBook, MIT (c) 2021 marshamallow) has one
//! config object; the mod API wires what the pinned SDK (api 0.2, f00125f61f)
//! reaches:
//! - the twelve config toggles register as settings on the Options > Mods
//!   page, same defaults, section-prefixed keys mirroring the upstream
//!   autoconfig categories;
//! - `client_setup` runs the mod entry 1:1: load `config/brb.json`
//!   (defaults when missing or broken, like autoconfig's Gson holder), sync
//!   the settings into the live config cell, construct the
//!   `InstantCraftingManager` mirroring `instantCraft.enabled`, and load
//!   `brb.pins` into the live `PinnedRecipeManager` (an absent or broken file
//!   leaves an empty set, like upstream's Gson `read`).
//!
//! The GUI half (recipe book components, group buttons, recipe pages,
//! brewing-stand and smithing components) and the mixins are the remaining
//! 1:1 parity work, tracked in the README; the config load/save listeners
//! that re-sync the instant-craft toggle button are the config-holder seam
//! the game side owns.
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

use mod_api::{ClientSetupEvent, ClientHost, RegistrationContext, Registrar, SettingSpec};

mod runtime;

/// The game directory (upstream: FabricLoader's game dir). The pins file
/// lives at its root: `brb.pins`, no extension (upstream
/// `new File(gameDirectory, "brb.pins")`).
pub fn game_dir() -> PathBuf {
    std::env::var_os("BRB_GAME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// `config/brb.json` (upstream autoconfig `@Config(name = "brb")`).
pub fn config_path() -> PathBuf {
    game_dir().join("config").join(runtime::CONFIG_FILE)
}

/// `<game dir>/brb.pins` (upstream: the game directory root, not config/).
pub fn pins_path() -> PathBuf {
    game_dir().join(runtime::PINS_FILE)
}

/// The live config the GUI seam reads (upstream: the config instance).
static LIVE_CONFIG: OnceLock<RwLock<runtime::Config>> = OnceLock::new();

pub fn set_live_config(config: runtime::Config) {
    let cell = LIVE_CONFIG.get_or_init(|| RwLock::new(runtime::Config::default()));
    *cell.write().expect("the config cell is not poisoned") = config;
}

pub fn live_config() -> runtime::Config {
    LIVE_CONFIG
        .get()
        .and_then(|cell| cell.read().ok())
        .map(|guard| guard.clone())
        .unwrap_or_default()
}

/// The live pinned-recipe manager (upstream: the `PinnedRecipeManager`
/// instance, loaded once at init).
static LIVE_PINS: OnceLock<RwLock<runtime::PinnedRecipeManager>> = OnceLock::new();

pub fn set_live_pins(pins: runtime::PinnedRecipeManager) {
    let cell = LIVE_PINS
        .get_or_init(|| RwLock::new(runtime::PinnedRecipeManager::default()));
    *cell.write().expect("the pins cell is not poisoned") = pins;
}

/// The live instant-craft manager (upstream: the `InstantCraftingManager`
/// instance constructed at init with `config.instantCraft.enabled`).
static LIVE_MANAGER: OnceLock<RwLock<runtime::InstantCraftingManager>> = OnceLock::new();

pub fn set_live_manager(manager: runtime::InstantCraftingManager) {
    let cell = LIVE_MANAGER
        .get_or_init(|| RwLock::new(runtime::InstantCraftingManager::new(false)));
    *cell.write().expect("the manager cell is not poisoned") = manager;
}

#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl mod_api::Hooks for Mod {
    /// The upstream config toggles as settings, same defaults, section
    /// prefixes mirroring the autoconfig `ConfigEntry` categories.
    fn register(&self, registrar: &mut dyn Registrar, _context: &RegistrationContext) {
        let _ = registrar.register_setting(SettingSpec::toggle(
            "enable_pinning",
            "setting.brb.enablePinning",
            true,
        ));
        let _ = registrar.register_setting(SettingSpec::toggle(
            "keep_centered",
            "setting.brb.keepCentered",
            false,
        ));
        let _ = registrar.register_setting(SettingSpec::toggle(
            "new_recipes_unlock_all",
            "setting.brb.newRecipes.unlockAll",
            true,
        ));
        let _ = registrar.register_setting(SettingSpec::toggle(
            "new_recipes_enable_bounce",
            "setting.brb.newRecipes.enableBounce",
            false,
        ));
        let _ = registrar.register_setting(SettingSpec::toggle(
            "instant_craft_show_button",
            "setting.brb.instantCraft.showButton",
            true,
        ));
        let _ = registrar.register_setting(SettingSpec::toggle(
            "instant_craft_enabled",
            "setting.brb.instantCraft.enabled",
            false,
        ));
        let _ = registrar.register_setting(SettingSpec::toggle(
            "alternative_recipes_on_hover",
            "setting.brb.alternativeRecipes.onHover",
            true,
        ));
        let _ = registrar.register_setting(SettingSpec::toggle(
            "alternative_recipes_no_grouped",
            "setting.brb.alternativeRecipes.noGrouped",
            false,
        ));
        let _ = registrar.register_setting(SettingSpec::toggle(
            "scrolling_enable_scrolling",
            "setting.brb.scrolling.enableScrolling",
            true,
        ));
        let _ = registrar.register_setting(SettingSpec::toggle(
            "scrolling_scroll_around",
            "setting.brb.scrolling.scrollAround",
            false,
        ));
        let _ = registrar.register_setting(SettingSpec::toggle(
            "settings_button",
            "setting.brb.settingsButton",
            true,
        ));
        let _ = registrar.register_setting(SettingSpec::toggle(
            "enable_book",
            "setting.brb.enableBook",
            true,
        ));
    }

    /// 1:1 of the original's mod init: load `config/brb.json` (defaults when
    /// missing or broken), sync the settings into the live config cell,
    /// construct the instant-craft manager mirroring `instantCraft.enabled`,
    /// and load `brb.pins` into the live manager (an absent or broken file
    /// leaves an empty set).
    fn client_setup(&self, host: &mut dyn ClientHost, _event: &mut ClientSetupEvent) {
        let mut config = std::fs::read_to_string(config_path())
            .ok()
            .map(|text| runtime::Config::from_json(&text))
            .unwrap_or_default();
        if let Some(value) = host.setting_bool("enable_pinning") {
            config.enable_pinning = value;
        }
        if let Some(value) = host.setting_bool("keep_centered") {
            config.keep_centered = value;
        }
        if let Some(value) = host.setting_bool("new_recipes_unlock_all") {
            config.new_recipes.unlock_all = value;
        }
        if let Some(value) = host.setting_bool("new_recipes_enable_bounce") {
            config.new_recipes.enable_bounce = value;
        }
        if let Some(value) = host.setting_bool("instant_craft_show_button") {
            config.instant_craft.show_button = value;
        }
        if let Some(value) = host.setting_bool("instant_craft_enabled") {
            config.instant_craft.enabled = value;
        }
        if let Some(value) = host.setting_bool("alternative_recipes_on_hover") {
            config.alternative_recipes.on_hover = value;
        }
        if let Some(value) = host.setting_bool("alternative_recipes_no_grouped") {
            config.alternative_recipes.no_grouped = value;
        }
        if let Some(value) = host.setting_bool("scrolling_enable_scrolling") {
            config.scrolling.enable_scrolling = value;
        }
        if let Some(value) = host.setting_bool("scrolling_scroll_around") {
            config.scrolling.scroll_around = value;
        }
        if let Some(value) = host.setting_bool("settings_button") {
            config.settings_button = value;
        }
        if let Some(value) = host.setting_bool("enable_book") {
            config.enable_book = value;
        }
        set_live_manager(runtime::InstantCraftingManager::new(
            config.instant_craft.enabled,
        ));
        set_live_config(config);
        let mut pins = runtime::PinnedRecipeManager::default();
        if let Ok(text) = std::fs::read_to_string(pins_path()) {
            let _ = pins.read_from_str(&text);
        }
        set_live_pins(pins);
    }
}
