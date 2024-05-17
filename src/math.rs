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

pub trait Normalize {
    fn normalize(&mut self);
}

impl Normalize for Vec2<f32> {
    fn normalize(&mut self) {
        let len = self.x.hypot(self.y) + f32::EPSILON;
        self.x /= len;
        self.y /= len;
    }
}

impl From<f32> for Vec2<f32> {
    fn from(value: f32) -> Self {
        Self { x: value, y: value }
    }
}

impl std::ops::AddAssign for Vec2<f32> {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
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
        self.x *= other.x;
        self.y *= other.y;
    }
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

pub fn translate_rotate(translate: Vec2<f32>, rotate_radians: f32) -> Mat4<f32> {
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
