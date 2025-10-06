# Changelog

# 0.3

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
