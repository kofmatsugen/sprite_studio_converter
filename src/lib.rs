pub mod prefab;
pub mod timeline;

pub fn test(data: &sprite_studio::SpriteStudioData) {
    for pack in data.packs() {
        println!("{}", pack.name());
        for anim in pack.animations() {
            println!("{}", anim.name());
            let count = anim.setting().count() as usize;
            for pa in anim.part_animes() {
                let _tl = timeline::part_anime_to_timeline::<()>(count, pa);
                println!("{}", pa.name());
            }
        }
    }
}
