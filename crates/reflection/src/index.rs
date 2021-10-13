use crate::data::Data;
use std::borrow::Cow;

/// Type for indexing [`Data`]
#[derive(Debug, Clone)]
pub enum Index {
    Integer(usize),
    String(Cow<'static, str>),
    Data(Data),
}

impl Index {
    pub fn index<'a>(&self, data: &'a Data) -> &'a Data {
        match &self {
            Index::Integer(index) => match data {
                Data::NewtypeStruct { data, .. } => {
                    if *index == 0 {
                        &data
                    } else {
                        panic!("No such field")
                    }
                }
                Data::NewtypeVariant { data, .. } => {
                    if *index == 0 {
                        &data
                    } else {
                        panic!("No such field")
                    }
                }
                Data::Seq(seq) => &seq[*index],
                Data::Tuple(tuple) => &tuple[*index],
                Data::TupleStruct { data, .. } => &data[*index],
                Data::TupleVariant { data, .. } => &data[*index],
                _ => panic!("Can't index this type with an Integer"),
            },
            Index::String(index) => match data {
                Data::Struct { fields, .. } => {
                    &fields.iter().find(|(key, _)| key == index).unwrap().1
                }
                Data::StructVariant { fields, .. } => {
                    &fields.iter().find(|(key, _)| key == index).unwrap().1
                }
                _ => panic!("Can't index this type with a String"),
            },
            Index::Data(index) => match data {
                Data::Map(map) => {
                    &map.iter()
                        .find(|(key, _)| key == index)
                        .expect("No such index")
                        .0
                }
                _ => panic!("Can't index this type with Data"),
            },
        }
    }

    pub fn index_mut<'a>(&self, data: &'a mut Data) -> &'a mut Data {
        match &self {
            Index::Integer(index) => match data {
                Data::NewtypeStruct { data, .. } => {
                    if *index == 0 {
                        data
                    } else {
                        panic!("No such field")
                    }
                }
                Data::NewtypeVariant { data, .. } => {
                    if *index == 0 {
                        data
                    } else {
                        panic!("No such field")
                    }
                }
                Data::Seq(seq) => &mut seq[*index],
                Data::Tuple(tuple) => &mut tuple[*index],
                Data::TupleStruct { data, .. } => &mut data[*index],
                Data::TupleVariant { data, .. } => &mut data[*index],
                _ => panic!("Can't index this type with an Integer"),
            },
            Index::String(index) => match data {
                Data::Struct { fields, .. } => {
                    &mut fields.iter_mut().find(|(key, _)| key == index).expect("No such field").1
                }
                Data::StructVariant { fields, .. } => {
                    &mut fields.iter_mut().find(|(key, _)| key == index).expect("No such field").1
                }
                _ => panic!("Can't index this type with a String"),
            },
            Index::Data(index) => match data {
                Data::Map(map) => {
                    &mut map
                        .iter_mut()
                        .find(|(key, _)| key == index)
                        .expect("No such index")
                        .0
                }
                _ => panic!("Can't index this type with Data"),
            },
        }
    }

    pub fn into_index<'a>(&self, data: Data) -> Data {
        match &self {
            Index::Integer(index) => match data {
                Data::NewtypeStruct { data, .. } => {
                    if *index == 0 {
                        *data
                    } else {
                        panic!("No such field")
                    }
                }
                Data::NewtypeVariant { data, .. } => {
                    if *index == 0 {
                        *data
                    } else {
                        panic!("No such field")
                    }
                }
                Data::Seq(mut seq) => seq.remove(*index),
                Data::Tuple(mut tuple) => tuple.remove(*index),
                Data::TupleStruct { mut data, .. } => data.remove(*index),
                Data::TupleVariant { mut data, .. } => data.remove(*index),
                _ => panic!("Can't index this type with an Integer"),
            },
            Index::String(index) => match data {
                Data::Struct { mut fields, .. } => {
                    let index = fields.iter().enumerate().find(|(_, (key, _))| key == index).expect("No such field").0;
                    fields.remove(index).1
                }
                Data::StructVariant { mut fields, .. } => {
                    let index = fields.iter().enumerate().find(|(_, (key, _))| key == index).expect("No such field").0;
                    fields.remove(index).1
                }
                _ => panic!("Can't index this type with a String"),
            },
            Index::Data(index) => match data {
                Data::Map(map) => {
                    map.into_iter()
                        .find(|(key, _)| key == index)
                        .expect("No such index")
                        .0
                }
                _ => panic!("Can't index this type with Data"),
            },
        }
    }
}
