use amethyst::renderer::sprite::{SpriteList, SpritePosition};

use sprite_studio::AnimationCells;

pub(crate) fn make_sprite_sheet(cell_map: &AnimationCells) -> SpriteList {
    let mut sprites = vec![];
    for cell in cell_map.cells() {
        let (x, y) = cell.position();
        let (x, y) = (x as u32, y as u32);
        let (width, height) = cell.size();
        let (width, height) = (width as u32, height as u32);
        let (pivot_x, pivot_y) = cell.pivot();
        let flip_horizontal = false;
        let flip_vertical = false;
        let sprite = SpritePosition {
            x,
            y,
            width,
            height,
            flip_horizontal,
            flip_vertical,
            offsets: Some([pivot_x, pivot_y]),
        };
        sprites.push(sprite);
    }

    let (texture_width, texture_height) = cell_map.pixel_size();
    let (texture_width, texture_height) = (texture_width as u32, texture_height as u32);
    let sprite = SpriteList {
        texture_width,
        texture_height,
        sprites,
    };

    sprite
}
