//! Utilities for working with various strings from Insim.

mod escaping;
pub use escaping::*;

mod colours;
pub use colours::*;

mod code_page_string;
pub use code_page_string::ICodepageString;

mod i_string;
pub use i_string::IString;

mod vehicle_string;
pub use vehicle_string::IVehicleString;
