use crate::{Error, Path};
use cbor_data::{Cbor, CborBuilder, CborOwned, TaggedItem, Visitor, Writer};
use std::borrow::Cow;

pub struct WriteVisitor<'a, F>
where
    F: FnMut(&'a Cbor) -> Cow<'a, Cbor>
{
    paths: Vec<Path>,
    map_function: F,
    pending_items: Vec<Vec<Cow<'a, Cbor>>>,
    current_path: Path,
    skip_end: bool,
}

impl<'a, F> WriteVisitor<'a, F>
where
    F: FnMut(&'a Cbor) -> Cow<'a, Cbor>
{
    pub fn new(paths: Vec<Path>, map_function: F) -> Self {
        Self {
            paths,
            map_function,
            pending_items: vec![Vec::new()],
            current_path: Path::default(),
            skip_end: false,
        }
    }

    pub fn get_cbor(&mut self) -> Option<CborOwned> {
        match self.pending_items.pop() {
            Some(mut v) => v.pop().map(|c| c.into_owned()),
            None => None,
        }
    }
}

impl<'a, F> Visitor<'a, Error> for WriteVisitor<'a, F>
where
    F: FnMut(&'a Cbor) -> Cow<'a, Cbor>
{
    fn visit_simple(&mut self, item: TaggedItem<'a>) -> Result<(), Error> {
        if let Some(pending_items) = self.pending_items.last_mut() {
            if self.paths.iter().any(|p| self.current_path == *p) {
                pending_items.push((self.map_function)(item.cbor()));
            } else {
                pending_items.push(Cow::Borrowed(item.cbor()));
            }
        }
        Ok(())
    }

    fn visit_array_begin(
        &mut self,
        array: TaggedItem<'a>,
        size: Option<u64>,
    ) -> Result<bool, Error> {
        if self.paths.iter().any(|p| self.current_path.is_parent(p)) {
            let items = if let Some(size) = size {
                Vec::with_capacity(size as usize)
            } else {
                Vec::new()
            };
            self.pending_items.push(items);
            Ok(true)
        } else {
            if let Some(pending_items) = self.pending_items.last_mut() {
                let item = if self.paths.iter().any(|p| self.current_path == *p) {
                    (self.map_function)(array.cbor())
                } else {
                    Cow::Borrowed(array.cbor())
                };
                pending_items.push(item);
            }
            self.skip_end = true;
            Ok(false)
        }
    }

    fn visit_array_index(&mut self, _array: TaggedItem<'a>, index: u64) -> Result<bool, Error> {
        if index > 0 {
            self.current_path.pop();
        }
        self.current_path.append_idx(index as usize);
        Ok(true)
    }

    fn visit_array_end(&mut self, _array: TaggedItem<'a>) -> Result<(), Error> {
        if self.skip_end {
            self.skip_end = false;
            return Ok(());
        }

        if let Some(pending_items) = self.pending_items.pop() {
            self.current_path.pop();
            let item = CborBuilder::new().write_array(None, |builder| {
                for item in pending_items.into_iter() {
                    builder.write_item(item.as_ref());
                }
            });
            if let Some(pending_items) = self.pending_items.last_mut() {
                pending_items.push(Cow::Owned(item));
            }
        }
        Ok(())
    }

    fn visit_dict_begin(&mut self, dict: TaggedItem<'a>, size: Option<u64>) -> Result<bool, Error> {
        if self.paths.iter().any(|p| self.current_path.is_parent(p)) {
            let items = if let Some(size) = size {
                Vec::with_capacity((size * 2) as usize)
            } else {
                Vec::new()
            };
            self.pending_items.push(items);
            Ok(true)
        } else {
            let item = if self.paths.iter().any(|p| self.current_path == *p) {
                (self.map_function)(dict.cbor())
            } else {
                Cow::Borrowed(dict.cbor())
            };

            if let Some(pending_items) = self.pending_items.last_mut() {
                pending_items.push(item);
            }
            self.skip_end = true;
            Ok(false)
        }
    }

    fn visit_dict_key(
        &mut self,
        _dict: TaggedItem<'a>,
        key: TaggedItem<'a>,
        is_first: bool,
    ) -> Result<bool, Error> {
        if !is_first {
            self.current_path.pop();
        }
        let key = key.cbor();
        if let Some(pending_items) = self.pending_items.last_mut() {
            pending_items.push(Cow::Borrowed(key));
        }
        self.current_path.append_key(key);
        Ok(true)
    }

    fn visit_dict_end(&mut self, _dict: TaggedItem<'a>) -> Result<(), Error> {
        if self.skip_end {
            self.skip_end = false;
            return Ok(());
        }

        if let Some(pending_items) = self.pending_items.pop() {
            self.current_path.pop();
            let item = CborBuilder::new().write_dict(None, |builder| {
                let mut iter = pending_items.into_iter();
                while let Some(key) = iter.next() {
                    if let Some(value) = iter.next() {
                        builder.with_cbor_key(
                            |b| b.write_item(key.as_ref()),
                            |b| b.write_item(value.as_ref()),
                        );
                    }
                }
            });
            if let Some(pending_items) = self.pending_items.last_mut() {
                pending_items.push(Cow::Owned(item));
            }
        }
        Ok(())
    }
}
