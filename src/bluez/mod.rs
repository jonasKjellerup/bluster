use std::collections::HashMap;
use std::time::Duration;
use std::sync::Arc;
use std::fmt;

use dbus::{nonblock::{SyncConnection, Proxy}, Path};
use dbus::arg::{messageitem::MessageItem, RefArg, Variant};
use crate::Error;
use crate::peripheral::Peripheral;
use crate::peripheral::properties::PeripheralProperty;

use async_trait::async_trait;


const DBUS_PROPERTIES_IFACE: &str = "org.freedesktop.DBus.Properties";
const DBUS_OBJECTMANAGER_IFACE: &str = "org.freedesktop.DBus.ObjectManager";

const BLUEZ_SERVICE_NAME: &str = "org.bluez";

const ADAPTER_IFACE: &str = "org.bluez.Adapter1";

const LE_ADVERTISING_MANAGER_IFACE: &str = "org.bluez.LEAdvertisingManager1";
const LE_ADVERTISEMENT_IFACE: &str = "org.bluez.LEAdvertisement1";

const GATT_SERVICE_IFACE: &str = "org.bluez.GattService1";
const GATT_CHARACTERISTIC_IFACE: &str = "org.bluez.GattCharacteristic1";
const GATT_DESCRIPTOR_IFACE: &str = "org.bluez.GattDescriptor1";
const GATT_GATT_MANAGER_IFACE: &str = "org.bluez.GattManager1";

const BLUEZ_ERROR_FAILED: &str = "org.bluez.Error.Failed";
// pub const BLUEZ_ERROR_INPROGRESS: &str = "org.bluez.Error.InProgress";
// pub const BLUEZ_ERROR_NOTPERMITTED: &str = "org.bluez.Error.NotPermitted";
// pub const BLUEZ_ERROR_NOTAUTHORIZED: &str = "org.bluez.Error.NotAuthorized";
// pub const BLUEZ_ERROR_INVALIDOFFSET: &str = "org.bluez.Error.InvalidOffset";
const BLUEZ_ERROR_NOTSUPPORTED: &str = "org.bluez.Error.NotSupported";

const PATH_BASE: &str = "/org/bluez/example";

const BLUEZ_DBUS_TIMEOUT: Duration = Duration::from_secs(30);



type ManagedObjectsProps = HashMap<Path<'static>, HashMap<String, HashMap<String, Variant<Box<dyn RefArg>>>>>;

pub struct DBusConnection(Arc<SyncConnection>);

impl fmt::Debug for DBusConnection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DBusConnection")
    }
}

impl<'a> DBusConnection {

    /// Initializes a new dbus system connection.
    fn new() -> Result<Self, Error> {
        let (resource, default) = dbus_tokio::connection::new_system_sync()?;
        tokio::spawn(async {
            let err = resource.await;
            panic!("Lost connection to D-Bus: {}", err);
        });

        Ok(DBusConnection(default))
    }   

    /// Creates a proxy object for a given D-Bus connection and path.
    fn get_bluez_proxy(&'a self, path: &'a Path) -> dbus::nonblock::Proxy<&'a SyncConnection> {
        dbus::nonblock::Proxy::new(BLUEZ_SERVICE_NAME, path, BLUEZ_DBUS_TIMEOUT, &self.0)
    }
}

/// An implementor of the `Peripheral` trait for systems using Bluez.
/// This interface wraps the Bluez D-Bus API.
#[derive(Debug)]
pub struct BluezPeripheral {
    pub object_path: Path<'static>,
    connection: DBusConnection,
}

impl BluezPeripheral {
    async fn find_adapter(connection: &DBusConnection) -> Result<Path<'static>, Error> {
        let path = "/".into();
        let proxy = connection.get_bluez_proxy(&path);

        let (props, ): (ManagedObjectsProps, ) = proxy
            .method_call(DBUS_OBJECTMANAGER_IFACE, "GetManagedObjects", ())
            .await?;
        Ok(props
            .into_iter()
            .find(|(_path, props)| props.contains_key(LE_ADVERTISING_MANAGER_IFACE))
            .map(|(path, _props)| path)
            .expect("LEAdvertisingManager1 interface not found"))
    }
}

#[async_trait]
impl Peripheral for BluezPeripheral {
    async fn new() -> Result<Self, Error> {
        let connection = DBusConnection::new()?;
        BluezPeripheral::find_adapter(&connection)
            .await
            .map(|object_path| BluezPeripheral {
                object_path,
                connection,
            })
    }

    async fn get<P>(&self) -> Result<P::Type, Error> 
        where P: PeripheralProperty
    {
        P::Type::dbus_get(&self.connection, &self.object_path, P::DBUS_KEY).await
    }

    async fn set<P>(&self, value: P::Type) -> Result<(), Error> where P: PeripheralProperty {
        let proxy = self.connection.get_bluez_proxy(&self.object_path);
        proxy.method_call(
            DBUS_PROPERTIES_IFACE,
            "Set",
            (
                ADAPTER_IFACE,
                P::DBUS_KEY,
                MessageItem::Variant(Box::new(value.into()))
            ),
        ).await?;
        Ok(())
    }
}


/// An abstraction intended to be used to create a bound
/// for `PeripheralProperty::Type` without directly creating
/// a bound to `dbus::arg::Get<'_>` which would complicate
/// the public api greatly across all targets.
#[async_trait]
pub trait DBusGet: Sized {
    async fn dbus_get(connection: &DBusConnection, object_path: &Path<'static>, key: &str) -> Result<Self, Error>;
}

// Blanket implementations for most types that
// implement `dbus::arg::Get<'a>`.
#[async_trait]
impl<T, 'b> DBusGet for T
where T: for<'a> dbus::arg::Get<'a> + 'static
{
    async fn dbus_get(connection: &DBusConnection, object_path: &Path<'static>, key: &str) -> Result<Self, Error>
    {
        let proxy = connection.get_bluez_proxy(object_path);
        let (value, ): (Variant<Self>, ) =  proxy.method_call(DBUS_PROPERTIES_IFACE, "Get", (ADAPTER_IFACE, key)).await?;
        Ok(value.0)
    }
}

// Error conversion specific to the Bluez implementation (D-Bus errors, etc.).

use crate::ErrorType;
use dbus::{arg::TypeMismatchError as DbusTypeMismatchError, Error as DbusError};
use std::io::Error as IoError;

impl From<DbusError> for Error {
    fn from(dbus_error: DbusError) -> Error {
        Error::new(
            dbus_error.name().unwrap_or(""),
            dbus_error.message().unwrap_or(""),
            ErrorType::Bluez,
        )
    }
}

impl From<DbusTypeMismatchError> for Error {
    fn from(dbus_type_mismatch_error: DbusTypeMismatchError) -> Error {
        Error::from(DbusError::from(dbus_type_mismatch_error))
    }
}

impl From<IoError> for Error {
    fn from(io_error: IoError) -> Error {
        Error::new(
            format!("std::io::Error: {:?}", io_error.kind()),
            format!("{:?}", io_error),
            ErrorType::Bluez,
        )
    }
}

impl From<()> for Error {
    fn from(_: ()) -> Error {
        Error::new("no name", "no description", ErrorType::Bluez)
    }
}
