use crate::windows::{intersection, Window};

#[test]
fn test_intersection() {
    let win1 = Window::new(0, 1, 256, 256);
    let win2 = Window::new(200, 201, 256, 256);
    let inter = intersection(&[win1, win2]);
    assert_eq!(inter, Some(Window::new(200, 201, 56, 56)));
}

#[test]
fn test_no_intersection() {
    let win1 = Window::new(0, 0, 256, 256);
    let win2 = Window::new(256, 256, 256, 256);
    let inter = intersection(&[win1, win2]);
    assert_eq!(inter, None);
}

#[test]
fn test_scale_window() {
    let win = Window::new(0, 0, 100, 100) * 1.02;
    assert_eq!(win, Window::new(-1, -1, 102, 102));
    assert_eq!(win * (1.0 / 1.02), Window::new(0, 0, 100, 100));
}
