mod aseprite_data;
mod rect;

use std::collections::HashMap;
use bevy_app::prelude::*;
use bevy_asset::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;
use bevy_ecs::prelude::*;
use bevy_render::prelude::*;
use bevy_render::render_resource::Extent3d;
use bevy_render::texture::TextureFormatPixelInfo;
use crate::aseprite_data::AsepriteData;
use crate::rect::Rect;

/// Plugin which will create sprite sheets from loaded aseprite json assets with their matching image
/// assets. The sheets will be loaded when entering CreateState and afterwards, the plugin will switch to NextState.
///
/// Important: The aseprite json assets and associated image assets must be loaded in before.
pub struct SpriteSheetPlugin<CreateState: States, NextState: States> {
    /// The state the plugin will start creating all sprite sheets.
    loading_state: CreateState,
    /// The state the plugin will switch to when all sprite sheets were created
    next_state: NextState
}

impl <CreateState: States, NextState: States> SpriteSheetPlugin<CreateState, NextState> {
    pub fn new(loading_state: CreateState, next_state: NextState) -> Self {
        Self { loading_state, next_state }
    }
}

impl <CreateState: States, NextState: States> Plugin for SpriteSheetPlugin<CreateState, NextState> {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(JsonAssetPlugin::<AsepriteData>::new(&["aseprite.json"]))
            .add_systems(
                OnEnter(self.loading_state.clone()),
                create_sprite_sheets(self.next_state.clone())
            )
        ;
    }
}

fn create_sprite_sheets<S: States>(followup_state: S) -> impl Fn(Commands, Res<AssetServer>, ResMut<Assets<Image>>, Res<Assets<AsepriteData>>, ResMut<NextState<S>>) {
    move |mut commands, asset_server, mut images, aseprite_data, mut next_state| {
        commands.insert_resource(create_sprite_sheets_from_aseprite_data(
            &asset_server,
            &mut images,
            &aseprite_data,
        ));
        next_state.set(followup_state.clone())
    }
}

fn create_sprite_sheets_from_aseprite_data(
    asset_server: &AssetServer,
    images: &mut Assets<Image>,
    aseprite_data: &Assets<AsepriteData>,
) -> SpriteSheets {
    let paths_and_data = aseprite_data
        .iter()
        .map(|(id, ad)| (
            asset_server
                .get_path(id)
                .expect("aseprite data should be loaded")
                .path()
                .to_str()
                .unwrap()
                .replace(".aseprite.json", ""),
            ad
        ))
        .filter_map(|(path, ad)| images
            .iter()
            .find(|(id, _)| asset_server
                .get_path(*id)
                .unwrap()
                .path()
                .to_str()
                .unwrap()
                .contains(&path)
            )
            .map(|(_, image)| (path, ad, image.clone()))
        )
        .collect::<Vec<_>>();

    SpriteSheets::new(
        paths_and_data
            .into_iter()
            .map(|(path, aseprite_data, image)| (
                path,
                SpriteSheet::new(
                    split_image_by_rectangles(&image, aseprite_data.rect_iter())
                        .into_iter()
                        .map(|image| images.add(image))
                )
            ))
    )
}

/// Split a given image by the given iterator of rectangles and create sub images from it.
pub fn split_image_by_rectangles<'a>(image: &'a Image, rectangles: impl IntoIterator<Item=Rect> + 'a) -> impl IntoIterator<Item=Image> + 'a {
    let dimension = image.texture_descriptor.dimension;
    let format = image.texture_descriptor.format;
    let sheet_width = image.texture_descriptor.size.width as usize * format.pixel_size();

    rectangles
        .into_iter()
        .map(move |rect| {
            let size = Extent3d {
                width: rect.width as u32,
                height: rect.height as u32,
                depth_or_array_layers: image.texture_descriptor.size.depth_or_array_layers,
            };

            let data = extract_rectangle(image.data.as_slice(), rect, sheet_width, format.pixel_size());
            Image::new(
                size,
                dimension,
                data,
                format,
            )
        })
}

fn extract_rectangle(data: &[u8], rect: Rect, data_width: usize, pixel_width: usize) -> Vec<u8> {
    let mut extracted = Vec::with_capacity(rect.width * rect.height);
    let start_index = data_width * rect.position.y as usize;

    for y in 0..rect.height {
        let start = start_index + y * data_width + rect.position.x as usize * pixel_width;
        let end = start + rect.width * pixel_width;
        data[start..end].into_iter().for_each(|val| extracted.push(*val))
    }

    extracted
}

/// Collection of all existing sprite sheets.
/// As these sprite sheets aren't assets themself, they are stored in here instead of Assets.
#[derive(Resource)]
pub struct SpriteSheets {
    path_sheet_map: HashMap<String, SpriteSheet>,
}

impl SpriteSheets {
    pub(crate) fn new(paths_and_sheets: impl IntoIterator<Item=(String, SpriteSheet)>) -> Self {
        SpriteSheets {
            path_sheet_map: paths_and_sheets.into_iter().collect()
        }
    }

    /// Return the sheet specified by the given path.
    ///
    /// The path should have no file ending, so if you have an asset "animation/my_animation.png" as a sheet
    /// and a "animation/my_animation.aseprite.json" aseprite file, you need to provide
    /// "animation/my_animation" as parameter
    pub fn get_sheet(&self, path: &str) -> &SpriteSheet {
        self.path_sheet_map.get(path).expect(&format!("sprite sheet {path} was not loaded!"))
    }
}

/// Stores handles to image parts from a bigger sprite sheet image.
pub struct SpriteSheet {
    pub textures: Vec<Handle<Image>>,
}

impl SpriteSheet {
    pub(crate) fn new(handles: impl IntoIterator<Item=Handle<Image>>) -> Self {
        Self {
            textures: handles.into_iter().collect()
        }
    }

    pub fn image_at(&self, index: usize) -> Handle<Image> {
        self.textures[index].clone()
    }

    pub fn images_at(&self, indexes: impl IntoIterator<Item=usize>) -> Vec<Handle<Image>> {
        indexes.into_iter().map(|i| self.textures[i].clone()).collect()
    }
}