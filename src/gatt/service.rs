use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use futures::channel::mpsc;
use uuid::Uuid;

use super::characteristic::Characteristic as OCharacteristic;
use crate::gatt::gatt_properties::{Properties, PropertyFlags};
use crate::gatt::event::{EventSender, Event};
use crate::common::{ConsumeBuilder, ChainedBuilder};

#[derive(Debug, Clone)]
pub struct _Service {
    pub(crate) uuid: Uuid,
    pub(crate) primary: bool,
    pub(crate) characteristics: HashSet<OCharacteristic>,
}

impl _Service {
    pub fn new(uuid: Uuid, primary: bool, characteristics: HashSet<OCharacteristic>) -> Self {
        _Service {
            uuid,
            primary,
            characteristics,
        }
    }
}

pub struct Service {
    pub(crate) uuid: Uuid,
    pub(crate) primary: bool,
}

pub struct ServiceBuilder {
    pub(crate) uuid: Uuid,
    pub(crate) primary: bool,
    pub(crate) characteristics: HashSet<Characteristic>,
}

impl ServiceBuilder {
    pub fn new(uuid: Uuid, primary: bool) -> Self {
        ServiceBuilder {
            uuid,
            primary,
            characteristics: HashSet::new(),
        }
    }

    pub fn new_characteristic(self, uuid: Uuid) -> ChainedBuilder<Characteristic, Self> {
        ChainedBuilder::new(Characteristic::new(uuid), self)
    }

    pub fn build(&self) -> Service {
        Service {
            uuid: self.uuid,
            primary: self.primary,
        }
    }
}

impl ConsumeBuilder<Characteristic> for ServiceBuilder {
    fn consume(mut self, target: Characteristic) -> Self {
        self.characteristics.insert(target);
        self
    }
}

pub struct Characteristic {
    uuid: Uuid,
    characteristic_properties: Properties<EventSender>,
    descriptor_properties: Properties<EventSender>,
}

impl_uuid_hash_eq!(Characteristic);

impl Characteristic {
    /// Sets the characteristics channel to be used for event handling.
    pub fn set_characteristic_read(&mut self, sender: mpsc::Sender<Event>) -> &mut Self {
        self.characteristic_properties.read = Some(sender);
        self
    }

    pub fn set_characteristic_write(&mut self, sender: mpsc::Sender<Event>) -> &mut Self {
        self.characteristic_properties.write = Some(sender);
        self
    }

    pub fn set_characteristic_flags(&mut self, flags: PropertyFlags) -> &mut Self {
        self.characteristic_properties.flags = flags;
        self
    }

    pub fn set_descriptor_read(&mut self, sender: mpsc::Sender<Event>) -> &mut Self {
        self.descriptor_properties.read = Some(sender);
        self
    }

    pub fn set_descriptor_write(&mut self, sender: mpsc::Sender<Event>) -> &mut Self {
        self.descriptor_properties.write = Some(sender);
        self
    }

    pub fn set_descriptor_flags(&mut self, flags: PropertyFlags) -> &mut Self {
        self.characteristic_properties.flags = flags;
        self
    }

    pub fn new(uuid: Uuid) -> Self {
        Self {
            uuid,
            characteristic_properties: Properties::default(),
            descriptor_properties: Properties::default(),
        }
    }
}

