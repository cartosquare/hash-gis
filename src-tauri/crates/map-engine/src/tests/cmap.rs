use crate::cmap::{viridis, Composite, HandleGet};

#[test]
fn test_new_rgb() {
    let comp = Composite::new_rgb(vec![0.0, 0.0, 0.0], vec![1.0, 1.0, 1.0]);
    assert_eq!(comp.get(&[0.5, 0.5, 0.5], None), [127, 127, 127, 255])
}

#[test]
fn test_new_gradient() {
    let comp = Composite::new_gradient(0.0, 1., &viridis);
    assert_eq!(comp.get(&[0., 0., 0.], None), [68, 1, 84, 255])
}

#[test]
fn test_new_gradient_with_breaks() {
    let comp = Composite::new_gradient_with_breaks(vec![
        (0., (0., 0., 0., 1.).into()),
        (0.75, (0.5, 0.5, 0.5, 1.).into()),
        (1., (1., 1., 1., 1.).into()),
    ]);
    assert_eq!(comp.get(&[0.75], None), [127, 127, 127, 255])
}

#[test]
fn test_new_custom_gradient() {
    let comp = Composite::new_custom_gradient(
        0.,
        1.,
        vec![
            (0., 0., 0., 1.).into(),
            (0.5, 0.5, 0.5, 1.).into(),
            (1., 1., 1., 1.).into(),
        ],
    );
    assert_eq!(comp.get(&[0.5], None), [127, 127, 127, 255])
}

#[test]
fn test_new_discrete_palette() {
    let comp = Composite::new_discrete_palette(vec![
        (1, (0., 0., 0., 1.).into()),
        (2, (1., 1., 1., 1.).into()),
    ]);

    assert_eq!(comp.get(&[1.0], None), [0, 0, 0, 255]);
    assert_eq!(comp.get(&[2.0], None), [255, 255, 255, 255]);
    assert_eq!(comp.get(&[3.0], None), [0, 0, 0, 0]);
}
