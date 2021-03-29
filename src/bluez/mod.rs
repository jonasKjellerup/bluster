use std::collections::HashMap;
use std::time::Duration;
use std::sync::Arc;

use dbus::{nonblock::SyncConnection, Path};
use dbus::arg::{messageitem::MessageItem, RefArg, Variant};
use crate::error::Error;
use crate::peripheral::Peripheral;
use crate::peripheral::properties::PeripheralProperty;
use crate::error::ErrorType::Bluez;



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
type DBusConnection = Arc<SyncConnection>;

/// Initializes a new dbus system connection.
fn create_dbus_connection() -> Result<DBusConnection, Error> {
    let (resource, default) = dbus_tokio::connection::new_system_sync()?;
    tokio::spawn(async {
        let err = resource.await;
        panic!("Lost connection to D-Bus: {}", err);
    });

    Ok(default)
}

/// Creates a proxy object for a given D-Bus connection and path.
fn get_bluez_proxy(conn: &DBusConnection, path: &Path) -> dbus::nonblock::Proxy<&SyncConnection> {
    dbus::nonblock::Proxy::new(BLUEZ_SERVICE_NAME, path, BLUEZ_DBUS_TIMEOUT, conn)
}

#[derive(Debug, Clone)]
pub struct BluezPeripheral {
    pub object_path: Path<'static>,
    connection: DBusConnection,
}

impl BluezPeripheral {
    async fn find_adapter(connection: &DBusConnection) -> Result<Path<'static>, Error> {
        let path = "/".into();
        let proxy = get_bluez_proxy(connection, &path);

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

impl Peripheral for BluezPeripheral {
    async fn new() -> Result<Self, Error> {
        let connection = create_dbus_connection()?;
        BluezPeripheral::find_adapter(&connection)
            .await
            .map(|object_path| Peripheral {
                object_path,
                connection,
            })
    }

    async fn get<P>(&self) -> Result<P::Type, Error> where P: PeripheralProperty {
        let proxy = get_bluez_proxy(&self.connection, &self.object_path);
        let (value, ): (Variant<P::Type>, ) = proxy
            .method_call(DBUS_PROPERTIES_IFACE, "Get", (ADAPTER_IFACE, P::DBUS_KEY))
            .await?;
        Ok(value.0)
    }

    async fn set<P>(&self, value: P::Type) -> Result<(), Error> where P: PeripheralProperty {
        let proxy = get_bluez_proxy(&self.connection, &self.object_path);
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