use std::{collections::BTreeMap, ops::Deref};

use legion::{
    query::EntityFilter,
    serialize::{CustomEntitySerializer, WorldSerializer},
    Entity, World,
};

use reflection::data::Data;

#[derive(Debug, Default, Clone)]
pub struct Entities(Option<Data>);

impl Entities {
    pub fn entities(&self) -> Option<&Data> {
        self.0.as_ref()
    }

    pub fn entities_mut(&mut self) -> Option<&mut Data> {
        self.0.as_mut()
    }

    pub fn parse_entities<F, W, E>(
        &mut self,
        world: &World,
        filter: &F,
        world_serializer: &W,
        entity_serializer: &E,
    ) where
        F: EntityFilter + Clone,
        W: WorldSerializer,
        E: CustomEntitySerializer<SerializedID = uuid::Uuid> + 'static,
    {
        let serializable =
            world.as_serializable(filter.clone(), world_serializer, entity_serializer);
        let serialized = reflection::to_data(&serializable, true).unwrap();
        self.0 = Some(serialized)
    }
}

#[derive(Debug, Clone)]
pub enum ParseEntitiesError {
    UnexpectedWorldKey(Data),
    WorldIsNotAMap(Data),
    KeyIsNotAString(Data),
    InvalidEntityUuid(uuid::Error),
    ParseComponents(ParseComponentsError),
}

impl From<ParseComponentsError> for ParseEntitiesError {
    fn from(e: ParseComponentsError) -> Self {
        ParseEntitiesError::ParseComponents(e)
    }
}

impl From<uuid::Error> for ParseEntitiesError {
    fn from(e: uuid::Error) -> Self {
        ParseEntitiesError::InvalidEntityUuid(e)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Components(BTreeMap<String, Data>);

impl Deref for Components {
    type Target = BTreeMap<String, Data>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub enum ParseComponentsError {
    ComponentsIsNotAMap(Data),
    KeyIsNotAString(Data),
}

pub fn parse_components(data: Data) -> Result<Components, ParseComponentsError> {
    let map = if let Data::Map(map) = data {
        map
    } else {
        return Err(ParseComponentsError::ComponentsIsNotAMap(data));
    };

    let mut components = BTreeMap::default();

    for (key, value) in map {
        let key = if let Data::String(key) = key {
            key
        } else {
            return Err(ParseComponentsError::KeyIsNotAString(key));
        };

        components.insert(key, value);
    }

    Ok(Components(components))
}
