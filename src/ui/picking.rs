use crate::impl_capabilities;

/// Defines capability that contains all Ui capabilities from this crate
pub struct CapUiPickingAll;

impl_capabilities!(
    CapUiPickingAll,
    ((clicked::ImmCapUiClicked, clicked::ImplImmCapUiClicked))
);

/// Module implments `.clicked()`
pub mod clicked;
