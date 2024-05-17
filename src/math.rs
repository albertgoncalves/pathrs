#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

#[repr(C)]
pub struct Vec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

type Mat4<T> = [[T; 4]; 4];

impl From<f32> for Vec2<f32> {
    fn from(value: f32) -> Self {
        Self { x: value, y: value }
    }
}

impl From<f32> for Vec3<f32> {
    fn from(value: f32) -> Self {
        Self {
            x: value,
            y: value,
            z: value,
        }
    }
}

impl std::ops::Add for Vec2<f32> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl std::ops::AddAssign for Vec2<f32> {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl std::ops::Mul for Vec2<f32> {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }
}

impl std::ops::MulAssign for Vec2<f32> {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

pub fn normalize(vec: &mut Vec2<f32>) {
    let len = vec.x.hypot(vec.y) + f32::EPSILON;
    vec.x /= len;
    vec.y /= len;
}

pub fn turn(to: &mut Vec2<f32>, from: Vec2<f32>, radians: f32) {
    let x = to.x - from.x;
    let y = to.y - from.y;
    let s = radians.sin();
    let c = radians.cos();
    to.x = y.mul_add(s, x.mul_add(c, from.x));
    to.y = y.mul_add(c, x.mul_add(-s, from.y));
}

pub fn orthographic(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> Mat4<f32> {
    let mut column_row = [[0.0; 4]; 4];
    column_row[0][0] = 2.0 / (right - left);
    column_row[1][1] = 2.0 / (top - bottom);
    column_row[2][2] = 2.0 / (near - far);
    column_row[3][3] = 1.0;
    column_row[3][0] = (left + right) / (left - right);
    column_row[3][1] = (bottom + top) / (bottom - top);
    column_row[3][2] = (near + far) / (near - far);
    column_row
}

pub fn translate_and_rotate(translate: Vec2<f32>, rotate_radians: f32) -> Mat4<f32> {
    let s = rotate_radians.sin();
    let c = rotate_radians.cos();
    let mut column_row = [[0.0; 4]; 4];
    column_row[0][0] = c;
    column_row[0][1] = s;
    column_row[1][0] = -s;
    column_row[1][1] = c;
    column_row[2][2] = 1.0;
    column_row[3][0] = translate.x;
    column_row[3][1] = translate.y;
    column_row[3][3] = 1.0;
    column_row
}
