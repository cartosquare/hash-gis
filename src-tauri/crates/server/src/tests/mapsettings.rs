use crate::{mapsettings::MapSettings, style::Style};
use map_engine::cmap::{ColourDefinition, Composite, HandleGet};

#[async_std::test]
async fn test_make_gradient_from_map_settings_inferno() {
    let style = Style {
        name: Some("inferno".to_string()),
        vmin: Some(0.0),
        vmax: Some(100.0),
        colours: None,
        bands: None,
    };
    let settings = MapSettings {
        style: Some(style),
        ..Default::default()
    };

    let gradient: Composite = settings
        .style
        .as_ref()
        .expect("Style not availble in MapSettings")
        .into();

    assert_eq!([0, 0, 3, 255], gradient.get(&[0.0], None));
    assert_eq!([252, 254, 164, 255], gradient.get(&[100.0], None));

    assert_eq!(gradient.get(&[0.0], None), [0, 0, 3, 255]);
    assert_eq!(gradient.get(&[100.0], None), [252, 254, 164, 255]);
}

#[async_std::test]
async fn test_make_gradient_from_map_settings_viridis() {
    let style = Style {
        name: Some("viridis".to_string()),
        vmin: Some(0.0),
        vmax: Some(100.0),
        colours: None,
        bands: None,
    };
    let settings = MapSettings {
        style: Some(style),
        ..Default::default()
    };

    let gradient: Composite = settings
        .style
        .as_ref()
        .expect("Style not availble in MapSettings")
        .into();

    assert_eq!(gradient.get(&[0.0], None), [68, 1, 84, 255]);
    assert_eq!(gradient.get(&[100.0], None), [253, 231, 36, 255]);
}

#[async_std::test]
async fn test_make_gradient_from_map_settings_using_default() {
    let style = Style {
        vmin: Some(0.0),
        vmax: Some(100.0),
        ..Default::default()
    };
    let mut settings = MapSettings {
        style: Some(style),
        ..Default::default()
    };

    let gradient: Composite = settings
        .style
        .as_ref()
        .expect("Style not availble in MapSettings")
        .into();

    assert_eq!(gradient.get(&[0.0], None), [0, 0, 0, 255]);
    assert_eq!(gradient.get(&[100.0], None), [255, 255, 255, 255]);

    let mut style = Style {
        vmin: Some(0.0),
        vmax: Some(100.0),
        ..Default::default()
    };
    style.name = Some("non-existent".to_string());
    settings.style = Some(style);
    let gradient: Composite = settings
        .style
        .as_ref()
        .expect("Style not availble in MapSettings")
        .into();

    assert_eq!(gradient.get(&[0.0], None), [68, 1, 84, 255]);
    assert_eq!(gradient.get(&[100.0], None), [253, 231, 36, 255]);
}

#[async_std::test]
async fn test_make_gradient_from_map_settings_colours() {
    let style = Style {
        vmin: Some(0.0),
        vmax: Some(100.0),
        colours: Some(ColourDefinition::Colours(vec![
            (0.0, 0.0, 0.0, 1.0).into(),
            (1.0, 1.0, 1.0, 1.0).into(),
        ])),
        ..Default::default()
    };
    let settings = MapSettings {
        style: Some(style),
        ..Default::default()
    };

    let gradient: Composite = settings
        .style
        .as_ref()
        .expect("Style not availble in MapSettings")
        .into();
    assert_eq!(gradient.get(&[0.0], None), [0, 0, 0, 255]);
    assert_eq!(gradient.get(&[100.0], None), [255, 255, 255, 255]);
}

#[async_std::test]
async fn test_make_gradient_from_map_settings_colours_and_breaks() {
    let mut style = Style {
        colours: Some(ColourDefinition::Colours(vec![
            (0.0, 0.0, 0.0, 1.0).into(),
            (1.0, 1.0, 1.0, 1.0).into(),
        ])),
        ..Default::default()
    };
    let mut settings = MapSettings {
        style: Some(style.clone()),
        ..Default::default()
    };

    let gradient: Composite = settings
        .style
        .as_ref()
        .expect("Style not availble in MapSettings")
        .into();
    assert_eq!(gradient.get(&[0.5], None), [127, 127, 127, 255]);

    let colours = ColourDefinition::ColoursAndBreaks(vec![
        (0.0, (0.0, 0.0, 0.0, 1.0).into()),
        (0.25, (0.5, 0.5, 0.5, 1.0).into()),
        (1.0, (1.0, 1.0, 1.0, 1.0).into()),
    ]);
    style.colours = Some(colours);
    settings.style = Some(style);
    let gradient: Composite = settings
        .style
        .as_ref()
        .expect("Style not availble in MapSettings")
        .into();
    assert_eq!(gradient.get(&[0.625], None), [191, 191, 191, 255]);
}

#[async_std::test]
async fn test_make_gradient_from_map_settings_rgb() {
    let style = Style {
        colours: Some(ColourDefinition::RGB(
            [0.0, 0.0, 0.0],
            [100.0, 100.0, 100.0],
        )),
        ..Default::default()
    };
    let settings = MapSettings {
        style: Some(style.clone()),
        ..Default::default()
    };

    let gradient: Composite = settings
        .style
        .as_ref()
        .expect("Style not availble in MapSettings")
        .into();
    assert_eq!(
        gradient.get(&[-50.0, 50.0, 150.0], None),
        [0, 127, 255, 255]
    );
}

#[async_std::test]
async fn test_style_hierarchy() {
    let style = Style {
        name: Some("viridis".to_string()),
        colours: Some(ColourDefinition::Colours(vec![
            (0.0, 0.0, 0.0, 1.0).into(),
            (1.0, 1.0, 1.0, 1.0).into(),
        ])),
        ..Default::default()
    };
    let settings = MapSettings {
        style: Some(style.clone()),
        ..Default::default()
    };

    let gradient: Composite = settings
        .style
        .as_ref()
        .expect("Style not availble in MapSettings")
        .into();
    assert_eq!(gradient.get(&[1.0], None), [253, 231, 36, 255]);
}
