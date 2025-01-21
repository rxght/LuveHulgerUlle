use std::{
    fs::OpenOptions,
    path::Path,
    sync::{Arc, OnceLock},
};

use vulkano::image::sampler::Filter;

use crate::graphics::{bindable::Texture, Graphics};

#[derive(Default)]
pub struct ItemId(usize);

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
enum ItemRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl Default for ItemRarity {
    fn default() -> Self {
        Self::Common
    }
}

static ITEM_DATA: OnceLock<Vec<Item>> = OnceLock::new();

#[derive(Default)]
pub struct Item {
    name: String,
    texture: Option<Arc<Texture>>,
    rarity: ItemRarity,
    stack_size: u32,
}

impl Item {
    pub fn get(gfx: &Graphics, id: ItemId) -> Option<&Self> {
        let item_list = ITEM_DATA.get_or_init(|| load_items(gfx, "assets/resources/items.xml"));
        item_list.get(id.0)
    }
}

fn load_items(gfx: &Graphics, resource_file: &str) -> Vec<Item> {
    let mut loaded_items: Vec<Item> = Vec::new();

    use xml::reader::XmlEvent;

    let file = OpenOptions::new().read(true).open(resource_file).unwrap();
    let base_path = Path::new(resource_file).parent().unwrap();

    let mut reader = xml::EventReader::new(file);

    match reader.next().unwrap() {
        XmlEvent::StartDocument { .. } => (),
        _ => panic!("no xml header"),
    }

    let current_element;
    match reader.next().unwrap() {
        XmlEvent::StartElement { name, .. } => current_element = name.local_name,
        _ => panic!("expected an element!"),
    }
    assert!(current_element == "Items");

    loop {
        match reader.next().unwrap() {
            XmlEvent::StartElement {
                name, attributes, ..
            } => {
                if name.local_name == "Item" {
                    let mut name = None;
                    let mut texture = None;
                    let mut rarity = ItemRarity::Common as u32;
                    let mut stack_size = 1;

                    for attribute in attributes {
                        match attribute.name.local_name.as_str() {
                            "name" => name = Some(attribute.value),
                            "source" => {
                                texture = Some(Texture::new(
                                    gfx,
                                    base_path.join(attribute.value).to_str().unwrap(),
                                    Filter::Nearest,
                                ))
                            }
                            "rarity" => {
                                rarity = u32::from_str_radix(attribute.value.as_str(), 10).unwrap()
                            }
                            "stack_size" => {
                                stack_size =
                                    u32::from_str_radix(attribute.value.as_str(), 10).unwrap()
                            }
                            _ => (),
                        }
                    }

                    let rarity = match rarity {
                        0 => ItemRarity::Common,
                        1 => ItemRarity::Uncommon,
                        2 => ItemRarity::Rare,
                        3 => ItemRarity::Epic,
                        4 => ItemRarity::Legendary,
                        _ => ItemRarity::Common,
                    };

                    if stack_size == 0 {
                        stack_size = 1;
                    }

                    loaded_items.push(Item {
                        name: name.unwrap_or_else(|| "Unnamed Item".to_string()),
                        texture,
                        rarity,
                        stack_size,
                    });
                }
            }
            XmlEvent::EndElement { name } => {
                if name.local_name == "Items" {
                    break;
                }
            }
            XmlEvent::EndDocument => panic!("unexpected eof"),
            _ => (),
        }
    }

    return loaded_items;
}
