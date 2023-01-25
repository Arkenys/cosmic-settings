// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::page::{self, section, Content, Meta, Page, Section};
use regex::Regex;
use slotmap::{SecondaryMap, SlotMap, SparseSecondaryMap};

pub struct Model {
    pub pages: SlotMap<page::Entity, Meta>,
    pub resource: HashMap<TypeId, Box<dyn Any>>,
    pub storage: HashMap<TypeId, SecondaryMap<page::Entity, Box<dyn Any>>>,
    pub sub_pages: SparseSecondaryMap<page::Entity, Vec<page::Entity>>,
    pub sections: SlotMap<section::Entity, Section>,
    pub content: SparseSecondaryMap<page::Entity, Content>,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            content: SparseSecondaryMap::new(),
            pages: SlotMap::with_key(),
            resource: HashMap::new(),
            sections: SlotMap::with_key(),
            storage: HashMap::new(),
            sub_pages: SparseSecondaryMap::new(),
        }
    }
}

impl Model {
    /// Check if a page exists in the model.
    #[must_use]
    pub fn contains_item(&self, id: page::Entity) -> bool {
        self.pages.contains_key(id)
    }

    /// Returns the content of a page, if it has any.
    #[must_use]
    pub fn content(&self, page: page::Entity) -> Option<&[section::Entity]> {
        self.content.get(page).map(Vec::as_slice)
    }

    /// Get an immutable reference to data associated with a page.
    #[must_use]
    pub fn data<Data: 'static>(&self, id: page::Entity) -> Option<&Data> {
        self.storage
            .get(&TypeId::of::<Data>())
            .and_then(|storage| storage.get(id))
            .and_then(|data| data.downcast_ref())
    }

    /// Get a mutable reference to data associated with a page.
    pub fn data_mut<Data: 'static>(&mut self, id: page::Entity) -> Option<&mut Data> {
        self.storage
            .get_mut(&TypeId::of::<Data>())
            .and_then(|storage| storage.get_mut(id))
            .and_then(|data| data.downcast_mut())
    }

    /// Associates data with the item.
    pub fn data_set<Data: 'static>(&mut self, id: page::Entity, data: Data) {
        if self.contains_item(id) {
            self.storage
                .entry(TypeId::of::<Data>())
                .or_insert_with(SecondaryMap::new)
                .insert(id, Box::new(data));
        }
    }

    /// Removes a specific data type from the item.
    pub fn data_remove<Data: 'static>(&mut self, id: page::Entity) {
        self.storage
            .get_mut(&TypeId::of::<Data>())
            .and_then(|storage| storage.remove(id));
    }

    // Registers a new page in the settings panel.
    pub fn register<P: Page>(&mut self) -> Insert {
        let id = self.pages.insert(P::page());

        if let Some(content) = P::content(&mut self.sections) {
            self.content.insert(id, content);
        }

        self.resource_register::<P::Model>();

        P::sub_pages(Insert { id, model: self })
    }

    #[must_use]
    pub fn resource<Resource: 'static>(&self) -> Option<&Resource> {
        self.resource
            .get(&TypeId::of::<Resource>())
            .and_then(|resource| resource.downcast_ref())
    }

    #[must_use]
    pub fn resource_mut<Resource: 'static>(&mut self) -> Option<&mut Resource> {
        self.resource
            .get_mut(&TypeId::of::<Resource>())
            .and_then(|resource| resource.downcast_mut())
    }

    #[allow(unused_must_use)]
    pub fn resource_register<Resource: Default + 'static>(&mut self) {
        self.resource
            .entry(TypeId::of::<Resource>())
            .or_insert_with(|| Box::new(Resource::default()));
    }

    /// Finds content of panels that match the search.
    pub fn search<'a>(
        &'a self,
        rule: &'a Regex,
    ) -> impl Iterator<Item = (page::Entity, section::Entity)> + 'a {
        SearchIter {
            content: self.content.iter(),
            model: self,
            sections: None,
            rule,
            page: page::Entity::default(),
        }
    }

    /// Returns the sub-pages of a page, if it has any.
    pub fn sub_pages(&self, page: page::Entity) -> Option<&[page::Entity]> {
        self.sub_pages.get(page).map(AsRef::as_ref)
    }
}

pub struct Insert<'a> {
    pub model: &'a mut Model,
    pub id: page::Entity,
}

impl<'a> Insert<'a> {
    #[must_use]
    pub fn id(self) -> page::Entity {
        self.id
    }

    #[must_use]
    pub fn content(self, content: Content) -> Self {
        self.model.content.insert(self.id, content);
        self
    }

    /// Adds a page and associates it with its parent page.
    #[allow(clippy::return_self_not_must_use)]
    #[allow(clippy::must_use_candidate)]
    pub fn sub_page<P: Page>(self) -> Self {
        let page = self.model.pages.insert(Meta {
            parent: Some(self.id),
            ..P::page()
        });

        if let Some(content) = P::content(&mut self.model.sections) {
            self.model.content.insert(page, content);
        }

        self.model.resource_register::<P::Model>();

        self.model
            .sub_pages
            .entry(self.id)
            .expect("parent page missing")
            .and_modify(|v| v.push(page))
            .or_insert_with(|| vec![page]);

        self
    }
}

pub struct SearchIter<'a> {
    model: &'a Model,
    content: slotmap::sparse_secondary::Iter<'a, page::Entity, Content>,
    sections: Option<std::slice::Iter<'a, section::Entity>>,
    page: page::Entity,
    rule: &'a Regex,
}

impl<'a> Iterator for SearchIter<'a> {
    type Item = (page::Entity, section::Entity);

    fn next(&mut self) -> Option<Self::Item> {
        'outer: loop {
            if let Some(sections) = self.sections.as_mut() {
                for id in sections {
                    if self.model.sections[*id].matches_search(self.rule) {
                        return Some((self.page, *id));
                    }
                }

                self.sections = None;
            }

            if let Some((page, content)) = self.content.next() {
                self.page = page;
                self.sections = Some(content.iter());
                continue 'outer;
            }

            return None;
        }
    }
}