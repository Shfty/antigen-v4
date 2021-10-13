use legion::{
    query::EntityFilter,
    serialize::{CustomEntitySerializer, WorldSerializer},
    World,
};

use reflection::data::Data;

#[derive(Debug, Default, Clone)]
pub struct Archetypes(Option<Data>);

impl Archetypes {
    pub fn archetypes(&self) -> Option<&Data> {
        self.0.as_ref()
    }

    pub fn archetypes_mut(&mut self) -> Option<&mut Data> {
        self.0.as_mut()
    }

    pub fn parse_archetypes<F, W, E>(
        &mut self,
        world: &World,
        filter: &F,
        world_serializer: &W,
        entity_serializer: &E,
    ) 
    where
        F: EntityFilter + Clone,
        W: WorldSerializer,
        E: CustomEntitySerializer<SerializedID = uuid::Uuid> + 'static,
    {
        let serializable =
            world.as_serializable(filter.clone(), world_serializer, entity_serializer);
        let serialized = reflection::to_data(&serializable, false).unwrap();
        self.0 = Some(serialized);
    }
}

#[derive(Debug, Clone)]
pub struct ParseArchetypeError;

#[derive(Debug, Clone)]
pub enum ParseArchetypesError {
    UnexpectedWorldKey(Data),
    WorldIsNotASeq(Data),
    Archetype(ParseArchetypeError),
}

impl From<ParseArchetypeError> for ParseArchetypesError {
    fn from(e: ParseArchetypeError) -> Self {
        ParseArchetypesError::Archetype(e)
    }
}
