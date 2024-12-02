use std::cmp::Ordering::*;
use std::collections::HashMap;

use bevy_asset::prelude::*;
use bevy_reflect_derive::Reflect;
use pad::p;
use serde::Deserialize;

use crate::rect::Rect;

/// Represents the json data for a sprite sheet that can be generated when exporting a sheet.
/// Used to load sheets from images using the data from the json file.
#[derive(Asset, Reflect, Deserialize, Clone)]
#[reflect(opaque)]
pub struct AsepriteData {
    frames: HashMap<String, FrameValue>,
}

impl AsepriteData {
    /// Returns an rectangle iterator for this aseprite data.
    /// All sub sprite names have a name like "<sheet name> <n>.aseprite", where
    /// <sheet name> is the name of the sprite sheet and <n> is the index.
    /// To return an ordered iterator, the entries must be sorted by key regarding length
    /// and name.
    pub fn rect_iter(&self) -> impl IntoIterator<Item=Rect> + '_ {
        let mut frames_vec = self.frames.iter().collect::<Vec<_>>();
        frames_vec.sort_by(|(ka, _), (kb, _)| match ka.len().cmp(&kb.len()) {
            Less => Less,
            Greater => Greater,
            Equal => ka.cmp(&kb)
        });

        frames_vec
            .into_iter()
            .map(|(_, fv)| fv.frame)
            .map(|f| Rect::new(p!(f.x, f.y), f.w, f.h))
    }
}

#[derive(Deserialize, Clone)]
struct FrameValue {
    frame: Frame,
}

#[derive(Copy, Clone, Deserialize, Eq, PartialEq)]
struct Frame {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}