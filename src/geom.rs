use crate::math;
use crate::math::Rotate;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Translate<T>(pub math::Vec2<T>);

impl<T> From<math::Vec2<T>> for Translate<T> {
    fn from(vec: math::Vec2<T>) -> Self {
        Self(vec)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Scale<T>(pub math::Vec2<T>);

impl<T> From<math::Vec2<T>> for Scale<T> {
    fn from(vec: math::Vec2<T>) -> Self {
        Self(vec)
    }
}

impl<T: Copy> From<T> for Scale<T> {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Color<T>(pub math::Vec4<T>);

impl<T> From<math::Vec4<T>> for Color<T> {
    fn from(vec: math::Vec4<T>) -> Self {
        Self(vec)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Geom<T> {
    pub translate: Translate<T>,
    pub scale: Scale<T>,
    pub color: Color<T>,
}

#[derive(Clone, Copy)]
pub struct Line<T>(pub math::Vec2<T>, pub math::Vec2<T>);

impl From<Line<f32>> for Translate<f32> {
    fn from(line: Line<f32>) -> Self {
        Self((line.0 * 0.5) + (line.1 * 0.5))
    }
}

impl From<Line<f32>> for Scale<f32> {
    fn from(line: Line<f32>) -> Self {
        Self(line.0 - line.1)
    }
}

impl Line<f32> {
    #[allow(dead_code)]
    pub fn new(point: math::Vec2<f32>, length: f32, radians: f32) -> Self {
        let mut line = Self(point, point + math::Vec2 { x: length, y: 0.0 });
        line.1.rotate(point, radians);
        line
    }
}
