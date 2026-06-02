// gb/metasprites.h bindings — metasprite support

#[repr(C)]
pub struct Metasprite {
    pub dy: i8,
    pub dx: i8,
    pub dtile: u8,
    pub props: u8,
}

pub const METASPRITE_END: i8 = -128;

unsafe extern "sdcccall-0" {
    pub fn hide_sprites_range(from: u8, to: u8);
    pub fn move_metasprite_ex(
        metasprite: *const Metasprite, base_tile: u8, base_prop: u8,
        base_sprite: u8, x: u8, y: u8,
    ) -> u8;
    pub fn move_metasprite(
        metasprite: *const Metasprite, base_tile: u8,
        base_sprite: u8, x: u8, y: u8,
    ) -> u8;
    pub fn move_metasprite_flipx(
        metasprite: *const Metasprite, base_tile: u8, base_prop: u8,
        base_sprite: u8, x: u8, y: u8,
    ) -> u8;
    pub fn move_metasprite_flipy(
        metasprite: *const Metasprite, base_tile: u8, base_prop: u8,
        base_sprite: u8, x: u8, y: u8,
    ) -> u8;
    pub fn move_metasprite_flipxy(
        metasprite: *const Metasprite, base_tile: u8, base_prop: u8,
        base_sprite: u8, x: u8, y: u8,
    ) -> u8;
    #[link_name = "move_metasprite_flipx"]
    pub fn move_metasprite_vflip(
        metasprite: *const Metasprite, base_tile: u8, base_prop: u8,
        base_sprite: u8, x: u8, y: u8,
    ) -> u8;
    #[link_name = "move_metasprite_flipy"]
    pub fn move_metasprite_hflip(
        metasprite: *const Metasprite, base_tile: u8, base_prop: u8,
        base_sprite: u8, x: u8, y: u8,
    ) -> u8;
    #[link_name = "move_metasprite_flipxy"]
    pub fn move_metasprite_hvflip(
        metasprite: *const Metasprite, base_tile: u8, base_prop: u8,
        base_sprite: u8, x: u8, y: u8,
    ) -> u8;
    pub fn hide_metasprite(metasprite: *const Metasprite, base_sprite: u8);
}
