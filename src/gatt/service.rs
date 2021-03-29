use super::characteristic::Characteristic as OCharacteristic;
use std::collections::HashSet;
use futures::{channel::mpsc::{self, channel}, prelude::*};
use uuid::Uuid;
use crate::gatt::gatt_properties::{Properties, PropertyFlags};
use crate::gatt::event::{EventSender, Event};

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
}

impl ServiceBuilder {
    pub fn new(uuid: Uuid, primary: bool) -> Self {
        ServiceBuilder {
            uuid,
            primary,
        }
    }

    pub fn build(&self) -> _Service {
        Service
    }
}

pub struct Characteristic {
}

pub struct CharacteristicBuilder<ChainResult> {
    uuid: Uuid,
    characteristic_properties: Properties<EventSender>,
    characteristic_channel: Option<(EventSender, mspc::Receiver<Event>)>,
    descriptor_properties: Properties<EventSender>,
    descriptor_channel: Option<(EventSender, mspc::Receiver<Event>)>,
    chain_result: ChainResult,
}

impl<ChainResult> CharacteristicBuilder<ChainResult> {
    pub fn with_characteristic_channel(&mut self, sender: mspc::Sender<Event>, receiver: mspc::Receiver<Event>) -> &mut self {
        self.characteristic_channel = Some((sender, receiver));
        self
    }

    pub fn with_descriptor_channel(&mut self, sender: mspc::Sender<Event>, receiver: mspc::Receiver<Event>) -> &mut self {
        self.descriptor_channel_channel = Some((sender, receiver));
        self
    }

    pub fn set_characteristic_flags(&mut self, flags: PropertyFlags) -> &mut self {
        self.characteristic_properties.flags = flags;
        self
    }
}

impl CharacteristicBuilder<()> {
    pub fn new(uuid: Uuid) -> Self {
        Self {
            uuid,
            characteristic_properties: Properties::default(),
            characteristic_channel: None,
            descriptor_properties: Properties::default(),
            descriptor_channel: None,
            chain_result: (),
        }
    }
}