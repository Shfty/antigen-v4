use std::borrow::Cow;

use crate::{data::Data, index::Index};

/// Type for traversing nested [`Data`] structures.
#[derive(Debug, Clone)]
pub struct Path {
    index: Index,
    next: Option<Box<Path>>,
}

impl Path {
    pub fn integer(index: usize) -> Path {
        Path {
            index: Index::Integer(index),
            next: None,
        }
    }

    pub fn string<T>(index: T) -> Path
    where
        T: Into<Cow<'static, str>>,
    {
        Path {
            index: Index::String(index.into()),
            next: None,
        }
    }

    pub fn data(index: Data) -> Path {
        Path {
            index: Index::Data(index),
            next: None,
        }
    }

    /// Append a new [`Path`]` to this [`Path`].
    pub fn push(mut self, new: Path) -> Path {
        if let Some(next) = self.next {
            self.next = Some(Box::new(next.push(new)));
        } else {
            self.next = Some(Box::new(new));
        }
        self
    }

    /// Remove the last [`Path`] from this [`Path`]
    pub fn pop(mut self) -> Path {
        if let Some(child) = &mut self.next {
            if let Some(_) = child.next {
                child.next = None;
            } else {
                self.next = None;
            }
        } else {
            panic!("Can't pop from a single-part Path")
        }

        self
    }

    /// Retrieve the [`Data`] pointed to by this [`Path`].
    pub fn walk<'a>(&self, data: &'a Data) -> &'a Data {
        let data = self.index.index(data);
        if let Some(path) = &self.next {
            path.walk(data)
        } else {
            data
        }
    }

    /// Retrieve the [`Data`] pointed to by this [`Path`].
    pub fn walk_mut<'a>(&self, data: &'a mut Data) -> &'a mut Data {
        let data = self.index.index_mut(data);
        if let Some(path) = &self.next {
            path.walk_mut(data)
        } else {
            data
        }
    }

    /// Retrieve the [`Data`] pointed to by this [`Path`].
    pub fn into_data<'a>(&self, data: Data) -> Data {
        let data = self.index.into_index(data);
        if let Some(path) = &self.next {
            path.into_data(data)
        } else {
            data
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path() {
        let data = Data::Struct {
            name: "MyStruct",
            fields: vec![
                (
                    "position",
                    Data::Tuple(vec![Data::F32(0.0), Data::F32(0.0)]),
                ),
                ("rotation", Data::F32(0.0)),
                ("scale", Data::Tuple(vec![Data::F32(1.0), Data::F32(1.0)])),
            ]
            .into_iter()
            .collect(),
        };

        println!("Data: {:#?}", &data);

        let path = Path::string("position").push(Path::integer(0));

        println!("Path: {:#?}", path);

        let result = path.walk(&data);
        println!("Result: {:#?}", result);
    }
}
