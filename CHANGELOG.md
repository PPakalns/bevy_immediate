# Changelog

# 0.5.1

* No changes to crate
* Support bevy_ui_text_input, bevy_framepace in demo example

# 0.5.0

* Upgraded to bevy 0.18
* Currently bevy_ui_text_input and related example has been disabled until it has been ported to bevy 0.18.

# 0.4.0
* bevy_immediate implementation has been split into multiple crates. Migration shouldn't require any changes
  because `bevy_immediate` reexports all necessary types.

* FloatingWindowStoreLocationId(pub u64) now stores u64 value.
  You can call `id.raw()` method on `ImmId` `id` value to retrieve u64 representation.

* bevy_immediate floating ui logic can now be used without requiring users to use rest of bevy_immediate.
  Use `bevy_immediate_floating_ui` crate.

* Added `checked_get_set` method to control for `Checkable` entity if `Checked` component is attached.
* Added `is_interaction_enabled` method to check if `InteractionDisabled` component is added.

* Tooltip improvements to avoid flickering.
* Improve slider behaviour when get_set is called.
* Improve behaviour when dropdown is closed.

# 0.3.3
* Improved `with_local_auto_id_guard` id prefix concatenation logic to correctly work in recursive situations.
* Renamed `with_local_auto_id_guard` to `with_add_id_pref`. Old function name deprecated until 0.4 release.
* Introduced `ImmScopeGuard` to avoid additional nesting levels when
  creating child entities.
* Introduced `ImmEntity::add_scoped` method to create child entities
  without using closure.
* Added function `text_color` to update text color in each frame.
* Added immutable `.ctx()` function to retrieve `ImmCtx`.

# 0.3.2

* Added `with_local_auto_id_guard` to create scope with different id generation. See `ImmCustomAutoIdScopeGuard`.
* Added `on_hash_change_text_fn` to update text only when some data has changed.
* Added `slider_get_set` function to update slider value. Useful for converting to/from integer values.
* Added utility trait `Mutable<T>` to abstract over `T`, `Mut<T>` types.
* Added `interaction_enabled` function that does the opposite of `interaction_disabled` (Manages `InteractionDisabled` component)
* Added fallback locations for anchored elements when outside view.
* Added `on_hash_change_insert`, `on_hash_change_typ_insert` functions to insert bundle into entity when provided value changes for given key.

# 0.3.1

Added text edit example

* Improved documentation
* Fixed bug where small floating windows were not correctly moved back inside main window.

# 0.3.0

* `.hovered()`, `.pressed()` is now calculated using `bevy_picking`.
* Added support for tooltips, dropdowns, menus, floating windows, (draggable, resizable).
* Added immediate mode entity local hash store to simplify storing local state 
  or to implement change detection based on hash value change.
* Added ergonomic API to spawn unrooted entities. Useful for dropdowns, tooltips, popups, floating windows.
* Added plugins for entity anchoring and window focus management.

# 0.2.1

* Documentation fixes

# 0.2.0

* Upgraded to bevy 0.17
* Added support for bevy_ui_widgets, bevy_feathers.
* Added support for hotpatching.
* Added examples to showcase bevy_feathers, bevy_ui_widgets.
* Added example to showcase scrollareas.
* Added example for hotpatching.
* A lot of breaking changes. See examples to find out how to use this library correctly. 

# 0.1.1

Minor documentation improvements.

# 0.1.0

Initial version.
