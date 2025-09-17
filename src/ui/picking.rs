use crate::{ImmImplCap, impl_capabilities};

/// Defines capability that contains all Ui capabilities from this crate
pub struct ImmCapUiPickingAll;

impl_capabilities!(ImmCapUiPickingAll, (clicked::ImmCapUiClicked));

/// Module implments `.clicked()`
pub mod clicked;
