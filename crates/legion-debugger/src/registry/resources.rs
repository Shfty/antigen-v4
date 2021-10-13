use reflection::{data::Data, serializer::Error};

pub struct ResourceRegistrar(
    pub fn(&legion::Resources) -> Result<Data, Error>,
);

plugin_registry::init!(ResourceRegistrar);

pub fn serialize_resources(
    resources: &legion::Resources,
) -> Result<Data, Error> {
    let mut serialized = vec![];

    for ResourceRegistrar(registry_func) in plugin_registry::iter!(ResourceRegistrar) {
        serialized.push(registry_func(resources)?);
    }

    Ok(Data::Seq(serialized))
}
