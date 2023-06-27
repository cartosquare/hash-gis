use crate::tiles::Tile;

#[test]
fn test_tile_from_lat_lng() {
    assert_eq!(Tile::from_lat_lng(0.0, 0.0, 0), Tile::new(0, 0, 0));
    assert_eq!(Tile::from_lat_lng(0.0, 0.0, 10), Tile::new(512, 512, 10));
}

#[test]
fn test_tile_ul() {
    let tile = Tile::new(1, 2, 3);
    assert_eq!(tile.ul(), (-135.0, 66.51326044311186));
}

#[test]
fn test_tile_zoom_out() {
    let tile = Tile::new(0, 0, 2);
    assert_eq!(tile.zoom_out(None).unwrap(), Tile::new(0, 0, 1));
    assert_eq!(tile.zoom_out(Some(0)).unwrap(), Tile::new(0, 0, 0));
    assert_eq!(tile.zoom_out(Some(3)).unwrap(), tile);
    let tile = Tile::new(0, 0, 0);
    assert_eq!(tile.zoom_out(None), None);
}

#[test]
fn test_tile_zoom_in() {
    let tile = Tile::new(0, 0, 0);
    assert_eq!(
        tile.zoom_in(None),
        Some(vec![
            Tile::new(0, 0, 1),
            Tile::new(1, 0, 1),
            Tile::new(1, 1, 1),
            Tile::new(0, 1, 1),
        ])
    );
    let children2 = tile.zoom_in(Some(2));
    assert_eq!(children2.as_ref().unwrap().len(), 16);
    assert!(children2.unwrap().iter().all(|t| t.z == 2));
    let tile = Tile::new(0, 0, 32);
    let children_max = tile.zoom_in(None);
    assert_eq!(children_max, None);
}
