# bevy_immediate: Immediate Mode UI for Bevy

[![bevy_version](https://img.shields.io/badge/bevy-0.17-blue)](https://github.com/bevyengine/bevy)
[![Latest version](https://img.shields.io/crates/v/bevy_immediate.svg)](https://crates.io/crates/bevy_immediate)
[![Documentation](https://docs.rs/bevy_immediate/badge.svg)](https://docs.rs/bevy_immediate)
[![License](https://img.shields.io/crates/l/bevy_immediate.svg)](https://crates.io/crates/bevy_immediate)

A **simple, fast, and modular UI library for [Bevy](https://bevyengine.org)**, combining immediate mode ergonomics with Bevy ECS-powered retained UI.  

* Develop complex UI as simple Rust code.  
* UI visuals, styling is fully customizable.
* Extend immediate mode with custom extensions / capabilities.

### 👉 [Web Demo](https://ppakalns.github.io/bevy_immediate/demo/) 👈 

![Demo screenshot](https://github.com/user-attachments/assets/a17b90e7-7763-44d3-a031-14e6fe84e5c7)

## Features

- **Immediate mode entity hierarchy management**  
  Build interactive entity hierarchies with a clean API.
- **Fully compatible with Bevy**  
  Heavy lifting is done by Bevy ECS and `bevy_ui` retained mode UI.
- **Custom extension support**  
  Add custom capabilities like `.clicked()`, `.selected(true)`, `.hovered()`. 
  Extension use integrated with rust type system for autocompletion and compile time check support.
- **Inbuilt support for UI use case**  
  Contains extensions that implement necessary logic for constructing UI.
- **Reusable widgets**  
  Implement widgets using functional or bevy native style.
- **Hot-patching support**  
  * Supported out of the box. See [hotpatching](#Hotpatching).
  * Alternative: [hot_lib_reloader](https://docs.rs/hot-lib-reloader/latest/hot_lib_reloader/).
- **Fast**  
  Only visits each entity once per tick and does minimal amount of changes. Heavy lifting is done by Bevy's retained UI.
- **Parallelizable**  
  Minimal data access requirements allow systems to run in parallel with other systems without exclusive world access.
- **Simple**  
  Define UI in straightforward functions, free from macro/observer/trigger boilerplate.
- **Modular**  
  Develop your UI by writing UI in small composable parts. Extend functionality with modular extensions.
- **Integration-friendly**  
  Works with other libraries (e.g., reloadable CSS style with [bevy_flair](https://github.com/eckz/bevy_flair)).


⚠️ **Note:** This library is under active development. Expect some breaking changes, but they will be minimized.

## Version compatibility

| bevy_immediate | bevy | MSRV           |
|------------|------------| ----------------|
| 0.3        | 0.17           | 1.88 |
| 0.2        | 0.17           | 1.88 |
| 0.1        | 0.16           | 1.85 |

To use add `bevy_immediate` to your project dependencies in `Cargo.toml` file.

See [CHANGELOG](./CHANGELOG.md) for changes between versions.

## Examples

Examples can be viewed: (`cargo run --example demo`).

- [Hello world](./examples/hello_world.rs) - Minimal usage example
- [Power user](./examples/power_user.rs) - Customized API for complex use cases
- [Plain UI](./examples/plain_ui.rs) - Create your UI as a single system
- **Bevy inbuilt widgets**:
  - [Widgets](./examples/bevy_widgets.rs) - Showcases how to use widgets from bevy
  - [Scrollarea](./examples/bevy_scrollbars.rs) - Showcases how to create reusable scrollareas 
- [Text edit](./examples/text_edit.rs) - Showcases text edit integration using bevy_ui_text_input crate.
- **Reusable widget implementation**
  - [Functional widget](./examples/widget_functional.rs) - Implement widgets as plain functions
  - [Native widget](./examples/widget_native.rs) - Implement native Bevy-like widgets
  - [Widget use](./examples/widget_use.rs) - Use functional and native widgets together
- [Main menu example](./examples/main_menu.rs) - Build a simple main menu with selectable buttons
- **Floating elements**
  - [Tooltips](./examples/tooltip.rs) - Add tooltips.
  - [Anchored UI](./examples/anchored.rs) - Build drop down menus, comboboxes, popups that utilize anchored floating UI.
  - [Floating windows](./examples/floating_window.rs) - Resizable, draggable floating windows.
- **Extensions**
  - [Extension implementation](./examples/extension.rs) - Write your own capabilities (e.g. `.clicked()` or `.selected(...)`)
  - [Using extensions](./examples/extension_use.rs) - Use a custom predefined set of extensions
- [Style](./examples/styles.rs) - Contains UI styling implementation for examples
- **[Hot-Patching example](./examples/hot_patching.rs) - Modify UI during program execution**:
  See [Hotpatching](#Hotpatching) section.

Examples are located in [`./examples/`](./examples/)

### Interactive UI example

Using `bevy_feathers` and `bevy_ui_widgets`.

```rust,ignore
// Checkbox
ui.ch()
    .on_spawn_insert(|| checkbox((), Text("Checkbox")))
    .checked(&mut checkbox_value);

// Toggle switch
ui.ch()
    .on_spawn_insert(|| toggle_switch(()))
    .interactions_disabled(state.disabled) // Control whether interactions are enabled
    .checked(&mut toggle_value);

// Button that counts clicks
let mut button = ui.ch().on_spawn_insert(|| controls::button(
        ButtonProps {
            variant: ButtonVariant::Normal,
            corners: RoundedCorners::All,
        },(),()
    ))
    .add(|ui| {
        ui.ch().text(format!("Clicked: {}", count));
    });

if button.activated() {
    count += 1;
}
```

### Power user example

Here's a more advanced example where user has added their own API.

```rust,ignore
pub struct PowerUserExamplePlugin;

impl bevy_app::Plugin for PowerUserExamplePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        // Initialize plugin with your widget root component
        app.add_plugins(BevyImmediateAttachPlugin::<CapsUi, PowerUserExampleRoot>::new());
        app.insert_resource(ShowHidden { show: false });
    }
}

#[derive(Resource)]
struct ShowHidden {
    show: bool,
}

#[derive(Component)]
pub struct PowerUserExampleRoot;

#[derive(SystemParam)]
pub struct Params<'w> {
    show_hidden: ResMut<'w, ShowHidden>,
}

impl ImmediateAttach<CapsUi> for PowerUserExampleRoot {
    type Params = Params<'static>;

    fn construct(ui: &mut Imm<CapsUi>, params: &mut Params) {
        ui.ch().my_title("Bevy power user example");

        ui.ch()
            .my_subtitle("Use helper functions to simplify and reuse code!");

        ui.ch().my_subtitle("Show collapsible element");

        ui.ch().my_row_container().add(|ui| {
            for (text, state) in [("No", false), ("Yes", true)] {
                let mut button = ui
                    .ch_id(("choice", state))
                    .my_button()
                    .selected(params.show_hidden.show == state)
                    .add(|ui| {
                        ui.ch().my_text(text);
                    });
                if button.clicked() {
                    params.show_hidden.show = state;
                }
            }
        });

        if params.show_hidden.show {
            ui.ch_id("yes_no").my_container_with_background().add(|ui| {
                ui.ch().my_text("Lorem Ipsum!");
            });
        }

        ui.ch().my_text("It is really simple!");
    }
}
```


### Extend functionality by implementing new capability

You can add new capabilities with just a few lines of code.
Here’s how `.selected(...)` is implemented.

```rust,ignore

/// Implements capability to mark entities as selectable.
pub struct CapabilityUiSelectable;

impl ImmCapability for CapabilityUiSelectable {
    fn build<Cap: CapSet>(app: &mut bevy_app::App, cap_req: &mut crate::ImmCapAccessRequests<Cap>) {
        cap_req.request_component_write::<Selectable>(app.world_mut());
    }
}

/// Marks component as being selectable
#[derive(bevy_ecs::component::Component)]
pub struct Selectable {
    /// Is selectable component selected
    pub selected: bool,
}

/// Implements methods to set entity selectable
pub trait ImmUiSelectable {
    /// Insert [`Selected`] component with given boolean value
    ///
    /// Useful for styling purposes
    fn selected(self, selected: bool) -> Self;
}

impl<Cap> ImmUiSelectable for ImmEntity<'_, '_, '_, Cap>
where
    Cap: ImplCap<CapabilityUiSelectable>,
{
    fn selected(mut self, selected: bool) -> Self {
        if let Ok(Some(mut comp)) = self.cap_get_component_mut::<Selectable>() {
            if comp.selected != selected {
                comp.selected = selected;
            }
            return self;
        }

        self.entity_commands().insert(Selectable { selected });
        self
    }
}
```

## New entity creation

New child entities can be created with `.ch`, `.ch_id`, `.ch_with_manual_id` family of functions.

For child entity creation that could appear, disappear, that are created inside loop: **unique id must be provided**.

Provided id is combined with parent id. **Id must be unique between siblings**.

Examples:
```rs
ui.ch_id("my_id");
ui.ch_id(lid!());
lch!(ui);

for idx in 0..count {
    ui.ch_id(("my_loop", idx));
    ui.ch_id(lid!(idx));
    lch!(ui, idx);
}

ui.ch(); // Has internal counter for id generation, but can not be used
         // for appearing, disappearing entities.
         // Because between frames entities may get misidentified.

for idx in 0..count {
    // In case of many items inside block, you can add additional id to auto id generation
    // In that case you have a new unique scope for which unique id requirements are restored.
    let mut ui = ui.with_local_auto_id_guard(("my_loop", idx));
    ui.ch();
    ui.ch();
    ui.ch();
}

```

`lid, lch` helper macros use current column, line numbers to generate auto id. But still inside loops you need to provide additional unique id.

## Hotpatching

Powered by [![Subsecond](https://img.shields.io/badge/Subsecond-Enabled-orange)](https://crates.io/crates/subsecond)

Follow: [Instructions & Limitations](https://bevy.org/news/bevy-0-17/#hot-patching-systems-in-a-running-app)

Launch examples with:
`BEVY_ASSET_ROOT="." dx serve --hot-patch --features "bevy_immediate/hotpatching" --features "bevy/hotpatching" --example demo`

Make sure that you enable `hotpatching` feature `bevy_immediate` and `bevy` crates so that UI is recreated upon hotpatch.

Try to modify and save `./examples/hot_patching.rs` or any other example and see changes in the live demo.

## FAQ

### UI nodes are changing order and not correctly laid out

Make sure that you assign unique id using `ch_id` for ui nodes that
can appear, disappear.

See [New entity creation](#new-entity-creation)

### How do I avoid collisions with resources or queries in my systems?

* Queries: Add `Without<ImmMarker<Caps>>` to your query filter.
* Resources: Avoid direct conflicts, or use .ctx() / .ctx_mut() APIs to access resources used by capabilities.

## Contributing

Contributions are welcome! 

* Add your improvements to examples
* Suggest or implement new capabilities useful for UI creation

Publish your own crate that is built using `bevy_immediate`!

## Inspiration

* Originally created for [Settletopia](https://settletopia.com/)
* Inspired by [egui_taffy](https://github.com/ppakalns/egui_taffy/).
* [Initial idea discussion](https://github.com/bevyengine/bevy/discussions/21030)


## Future work

- Easier definition of new capability sets
  - [x] Tried transitive capability implementation (works only inside one crate)
  - [x] Tried transitive trait implementation (works only inside one crate)
  - [x] Tried TupleList approach (conflicting trait implementations)
  - [ ] ???

- Create reusable logic for:
  - [x] Bevy ui widgets
  - [x] Bevy scroll areas
  - [x] Tooltips
  - [x] Popups
  - [x] Draggable, resizable windows (like `egui::Window`)

