use crate::math;

#[repr(C)]
pub struct Translate<T>(pub math::Vec2<T>);

#[repr(C)]
pub struct Scale<T>(pub math::Vec2<T>);

#[repr(C)]
pub struct Color<T>(pub math::Vec4<T>);

#[repr(C)]
pub struct Geom<T> {
    pub translate: Translate<T>,
    pub scale: Scale<T>,
    pub color: Color<T>,
}

#[derive(Clone, Copy)]
pub struct Line<T>(pub math::Vec2<T>, pub math::Vec2<T>);

impl<T> From<math::Vec2<T>> for Translate<T> {
    fn from(vec: math::Vec2<T>) -> Self {
        Self(vec)
    }
}

impl From<Line<f32>> for Translate<f32> {
    fn from(line: Line<f32>) -> Self {
        Self((line.0 * 0.5) + (line.1 * 0.5))
    }
}

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

impl From<Line<f32>> for Scale<f32> {
    fn from(line: Line<f32>) -> Self {
        Self(line.0 - line.1)
    }
}

impl<T> From<math::Vec4<T>> for Color<T> {
    fn from(vec: math::Vec4<T>) -> Self {
        Self(vec)
    }
}
