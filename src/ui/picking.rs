use crate::merge_capabilities;

/// Defines capability that contains all Ui capabilities from this crate
merge_capabilities!(CapUiPickingAll, (clicked::ImmCapUiClicked));

/// Module implments `.clicked()`
pub mod clicked;
