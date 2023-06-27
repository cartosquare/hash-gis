use crate::mercator::GlobalMercator;

#[test]
fn test_mercator() {
    let mercator: GlobalMercator = Default::default();
    assert_eq!(mercator.tile_size, 256);
    assert_eq!(mercator.zoom_for_pixel_size(&150.0), 10);
}
