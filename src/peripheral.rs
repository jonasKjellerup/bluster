//! Platform independent BLE peripheral interace. The generic interface
//! is described in the `Peripheral` trait, which each native implementation
//! is expected to implement. The relevant native implementation for the given
//! target os is exposed through the `NativePeripheral` type alias.
//!
//! The supported platforms and their corresponding implementing types are listed below:
//!     - Linux: `BluezPeripheral`
//!
//! # Example
//! ```rust
//! # use std::error::Error;
//! #
//! # fn main() -> Result<(), Box<dyn Error>> {
//! use bluster::peripheral::{NativePeripheral, properties};
//! let peripheral = NativePeripheral::new();
//! peripheral.set::<properties::Powered>(true);
//! # Ok(())
//! # }
//! ```

use crate::Error;

/// A type alias for the corresponding `Peripheral` implementation
/// for the used target os. For unsupported platforms this is set to
/// `()`.
#[cfg(any(not(target_os = "linux"), doc))]
pub type NativePeripheral = ();

#[cfg(target_os = "linux")]
pub type NativePeripheral = BluezPeripheral;

/// Definitions for managing the various properties
/// of the bluetooth peripheral/interface.
pub mod properties {
    /// Defines the a property for a bluetooth peripheral.
    /// The trait is used in conjunction with the `Peripheral::get<P>` and
    /// `Peripheral::set<P>` functions as the type parameter `P` to access
    /// of modify individual properties of the peripheral.
    pub trait PeripheralProperty {
        /// The name used by the Bluez D-Bus api to refer to a property.
        /// This is only relevant for when targeting linux.
        #[cfg(any(target_os = "linux", doc))]
        const DBUS_KEY: &str;

        /// The type used when representing the value in rust code.
        /// E.g. for the `Powered` property the `bool` type is used
        /// to represent whether the peripheral is powered.
        type Type;
    }

    macro_rules! define_property_type {
        (@implement dbus_key = $key:expr ; $($tail:tt)*) => {
            #[cfg(any(target_os = "linux"))]
            const DBUS_KEY: &'static str = $key;

            define_property_type!(@implement $($tail)*);
        };

        (@implement type = $T:ty ; $($tail:tt)*) => {
            type Type = $T;

            define_property_type!(@implement $($tail)*);
        };

        ($name:ident { $($tail:tt)+ }) => {
            pub struct $name;
            impl PeripheralProperty for $name {
                define_property_type!(@implement $($tail)+);
            }
        };

        (@implement) => {}
    }

    define_property_type!(Powered {
        type = bool;
        dbus_key = "Powered";
    });

    define_property_type!(Discoverable {
        type = bool;
        dbus_key = "Discoverable";
    });

    define_property_type!(Alias {
        type = String;
        dbus_key = "Alias";
    });
}

pub trait Peripheral {
    /// Instantiates a new peripheral instance.
    ///
    /// To instantiate a new peripheral for a generic (supported)
    /// target os use `NativePeripheral::new`.
    async fn new() -> Result<Self, Error>;
    async fn get<P>(&self) -> Result<P::Type, Error> where P: properties::PeripheralProperty;
    async fn set<P>(&self, value: P::Type) -> Result<(), Error> where P: properties::PeripheralProperty;
}