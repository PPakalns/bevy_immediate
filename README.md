# bevy_immediate: Immediate Mode UI for Bevy

[![bevy_version](https://img.shields.io/badge/bevy-0.17-blue)](https://github.com/bevy/bevy)
[![Latest version](https://img.shields.io/crates/v/bevy_immediate.svg)](https://crates.io/crates/bevy_immediate)
[![Documentation](https://docs.rs/bevy_immediate/badge.svg)](https://docs.rs/bevy_immediate)
[![License](https://img.shields.io/crates/l/bevy_immediate.svg)](https://crates.io/crates/bevy_immediate)

A **simple, fast, and modular UI library for [Bevy](https://bevyengine.org)**, combining immediate mode ergonomics with Bevy ECS-powered retained UI.  

* Write complex UI logic as simple Rust code.  
* No macros, observers, triggers, events, signals.  
* Focus on your UI, not the boilerplate.

## Features

- **Immediate mode entity hierarchy management**  
  Build interactive entity hierarchies with a clean API.
- **Fully compatible with Bevy**  
  Heavy lifting is done by Bevy ECS and `bevy_ui` retained mode UI.
- **Custom extension support**  
  Add custom capabilities like `.clicked()`, `.selected(true)`, `.hovered()`. 
  Extension use integrated with rust type system for IDE and compile check support.
- **Inbuilt support for UI use case**  
  Contains extensions that implement necessary logic for constructing UI.
- **Reusable widgets**  
  Implement widgets using functional or bevy native style.
- **Fast**  
  Only visits each entity once per tick and does minimal amount of changes. Heavy lifting is done by Bevy's retained UI.
- **Parallelizable**  
  Minimal data access requirements allow systems to run in parallel with other systems without exclusive world access.
- **Simple**  
  Define UI in straightforward functions, free from macro/observer/trigger boilerplate.
- **Modular**  
  Extend the API with your own small capabilities and traits that encapsulate complex logic.
- **Integration-friendly**  
  Works with other libraries (e.g., reloadable CSS style with [bevy_flair](https://github.com/eckz/bevy_flair)).
- **Hot reloading support**  
  Supported out of the box using bevy hotpatching approach.
  See Hot-Patching example description in [Examples](#Examples) section.
  [hot_lib_reloader](https://docs.rs/hot-lib-reloader/latest/hot_lib_reloader/) works too as alternative.


⚠️ **Note:** This library is under active development. Expect some breaking changes, but they will be minimized.

## Version compatibility

| bevy_immediate | bevy | MSRV           |
|------------|------| ----------------|
| 0.2        | 0.17 | 1.88 |
| 0.1        | 0.16 | 1.85 |

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
- **Reusable widget implementation**
  - [Functional widget](./examples/widget_functional.rs) - Implement widgets as plain functions
  - [Native widget](./examples/widget_native.rs) - Implement native Bevy-like widgets
  - [Widget use](./examples/widget_use.rs) - Use functional and native widgets together
- [Menu example](./examples/menu.rs) - Build a simple menu with selectable buttons
- **Extensions**
  - [Extension implementation](./examples/extension.rs) - Write your own capabilities (e.g. `.clicked()` or `.selected(...)`)
  - [Using extensions](./examples/extension_use.rs) - Use a custom predefined set of extensions
- [Style](./examples/styles.rs) - Contains UI styling implementation for examples
- **[Hot-Patching example](./examples/hot_patching.rs) - Modify UI during program execution**:
  See [Hot-Patching instructions][#Hotpatching]

Examples are located in [`./examples/`](./examples/)

<img width="768" height="447" alt="image" src="https://github.com/user-attachments/assets/a6aa921b-d87b-4779-91e7-0106773183d1" />

### Menu example

Example with code reuse and **interactive elements**:

```rust,ignore

pub struct MenuExamplePlugin;

impl bevy_app::Plugin for MenuExamplePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.insert_resource(CurrentExample::WidgetUse);

        app.add_plugins(BevyImmediateAttachPlugin::<CapsUi, MenuUiRoot>::new());
    }
}

#[derive(Component)]
pub struct MenuUiRoot;

#[derive(SystemParam)]
pub struct Params<'w> {
    current_example: ResMut<'w, CurrentExample>,
    debug_options: ResMut<'w, UiDebugOptions>,
}

impl ImmediateAttach<CapsUi> for MenuUiRoot {
    type Params = Params<'static>;

    fn construct(ui: &mut Imm<CapsUi>, params: &mut Params) {
        ui.ch()
            .on_spawn_insert(|| Node {
                flex_direction: FlexDirection::Column,
                align_items: bevy_ui::AlignItems::Stretch,
                ..fill_parent_node()
            })
            .add(|ui| {
                ui.ch()
                    .on_spawn_insert(styles::title_text_style)
                    .on_spawn_text("Demo");
                ui.ch()
                    .on_spawn_insert(styles::text_style)
                    .on_spawn_text("bevy_immediate");

                ui.ch().on_spawn_insert(|| Node {
                    height: Val::Px(10.),
                    ..default()
                });

                for (example, title) in MENU_VARIANTS {
                    let mut button = ui
                        .ch()
                        .on_spawn_insert(styles::button_bundle)
                        .selected(example == *params.current_example)
                        .add(|ui| {
                            ui.ch()
                                .on_spawn_insert(styles::text_style)
                                .on_spawn_text(title);
                        });

                    if button.clicked() {
                        *params.current_example = example;
                    }
                }

                ui.ch().on_spawn_insert(|| Node {
                    flex_grow: 1.,
                    ..default()
                });

                let mut button = ui
                    .ch()
                    .on_spawn_insert(button_bundle)
                    .selected(params.debug_options.enabled)
                    .add(|ui| {
                        ui.ch().on_spawn_insert(text_style).text("Debug");
                    });
                if button.clicked() {
                    params.debug_options.enabled = !params.debug_options.enabled;
                }
            });
    }
}

pub const MENU_VARIANTS: [(CurrentExample, &str); 4] = [
    (CurrentExample::HelloWorld, "Hello World"),
    (CurrentExample::WidgetUse, "Widget usage"),
    (CurrentExample::ExtensionUse, "Extension usage"),
    (CurrentExample::PowerUser, "Power user"),
];

#[derive(Resource, Hash, Clone, Copy, PartialEq, Eq)]
pub enum CurrentExample {
    WidgetUse,
    HelloWorld,
    ExtensionUse,
    PowerUser,
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
            if comp.selected == selected {
                return self;
            }
            comp.selected = selected;
            return self;
        }

        self.entity_commands().insert(Selectable { selected });
        self
    }
}
```

## New element creation

For child entity creation that could appear, disappear: **unique id must be provided**.

Examples:
```rs
ui.ch_id(lid!());
ui.ch_id("my_id");

for idx in 0..20 {
    ui.ch_id((lid!(), idx));
    ui.ch_id(("my_loop", idx));
}
```

## Hotpatching

Powered by [![Subsecond](https://img.shields.io/badge/Subsecond-Enabled-orange)](https://crates.io/crates/subsecond)

Follow: [Instructions & Limitations](https://bevy.org/news/bevy-0-17/#hot-patching-systems-in-a-running-app)

Launch examples with:
`BEVY_ASSET_ROOT="." dx serve --hot-patch --features "bevy_immediate/hotpatching" --features "bevy/hotpatching" --example demo`

Make sure that you enable `hotpatching` feature `bevy_immediate` and `bevy` crates so that UI is recreated upon hotpatch.

Try to modify and save `./examples/hot_patching.rs` or any other example.

## FAQ

### UI nodes are changing order and not correctly laid out

Make sure that you assign unique id using `ch_id` for ui nodes that
can appear, disappear.

If you do not want to think about unique ids. 
You can use helper macro `lid!()`. 
But this still requires adding unique id counter inside loops: `ui.ch_id((lid!(), idx))`.

See [New element creation](#New element creation)

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

- Create reusable logic for:
  - [x] Bevy ui widgets
  - [x] Bevy scroll areas
  - [ ] Tooltips
  - [ ] Popups
  - [ ] Draggable windows (like `egui::Window`)

