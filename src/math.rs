#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl From<f32> for Vec2<f32> {
    fn from(value: f32) -> Self {
        Self { x: value, y: value }
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

impl std::ops::Add<f32> for Vec2<f32> {
    type Output = Self;

    fn add(self, other: f32) -> Self {
        Self {
            x: self.x + other,
            y: self.y + other,
        }
    }
}

impl std::ops::AddAssign for Vec2<f32> {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl std::ops::Sub for Vec2<f32> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
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

impl std::ops::Mul<f32> for Vec2<f32> {
    type Output = Self;

    fn mul(self, other: f32) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl std::ops::MulAssign for Vec2<f32> {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

impl std::ops::MulAssign<f32> for Vec2<f32> {
    fn mul_assign(&mut self, other: f32) {
        *self = *self * other;
    }
}

pub fn rotate(to: Vec2<f32>, from: Vec2<f32>, radians: f32) -> Vec2<f32> {
    let delta = to - from;
    let sin = radians.sin();
    let cos = radians.cos();
    Vec2 {
        x: delta.y.mul_add(sin, delta.x.mul_add(cos, from.x)),
        y: delta.y.mul_add(cos, delta.x.mul_add(-sin, from.y)),
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Vec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl std::ops::Sub for Vec3<f32> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl std::ops::Mul<f32> for Vec3<f32> {
    type Output = Self;

    fn mul(self, other: f32) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Vec4<T> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Mat4<T>(pub [[T; 4]; 4]);

// NOTE: See `https://www.khronos.org/registry/OpenGL-Refpages/gl2.1/xhtml/gluPerspective.xml`.
pub fn perspective(fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Mat4<f32> {
    let mut mat = Mat4::default();

    let cotangent = 1.0 / (fov / 2.0).tan();

    mat.0[0][0] = cotangent / aspect_ratio;
    mat.0[1][1] = cotangent;
    mat.0[2][3] = -1.0;
    mat.0[2][2] = (near + far) / (near - far);
    mat.0[3][2] = (2.0 * near * far) / (near - far);

    mat
}

pub fn look_at(from: Vec3<f32>, to: Vec3<f32>, up: Vec3<f32>) -> Mat4<f32> {
    let forward: Vec3<f32> = (to - from).normalize();
    let right: Vec3<f32> = forward.cross(up).normalize();
    let up: Vec3<f32> = right.cross(forward);

    let mut mat = Mat4::default();

    mat.0[0][0] = right.x;
    mat.0[0][1] = up.x;
    mat.0[0][2] = -forward.x;

    mat.0[1][0] = right.y;
    mat.0[1][1] = up.y;
    mat.0[1][2] = -forward.y;

    mat.0[2][0] = right.z;
    mat.0[2][1] = up.z;
    mat.0[2][2] = -forward.z;

    mat.0[3][0] = -right.dot(from);
    mat.0[3][1] = -up.dot(from);
    mat.0[3][2] = forward.dot(from);
    mat.0[3][3] = 1.0;

    mat
}

trait Dot<T> {
    fn dot(self, other: Self) -> T;
}

impl Dot<f32> for Vec2<f32> {
    fn dot(self, other: Self) -> f32 {
        self.x.mul_add(other.x, self.y * other.y)
    }
}

impl Dot<f32> for Vec3<f32> {
    fn dot(self, other: Self) -> f32 {
        self.z.mul_add(other.z, self.x.mul_add(other.x, self.y * other.y))
    }
}

trait Cross {
    fn cross(self, other: Self) -> Self;
}

impl Cross for Vec3<f32> {
    fn cross(self, other: Self) -> Self {
        Self {
            x: self.y.mul_add(other.z, -(self.z * other.y)),
            y: self.z.mul_add(other.x, -(self.x * other.z)),
            z: self.x.mul_add(other.y, -(self.y * other.x)),
        }
    }
}

pub trait Normalize {
    fn normalize(self) -> Self;
}

impl<T: Dot<f32> + std::ops::Mul<f32, Output = T> + Copy> Normalize for T {
    fn normalize(self) -> Self {
        self * (1.0 / (self.dot(self) + f32::EPSILON).sqrt())
    }
}

pub trait Distance<T> {
    fn distance(self, other: Self) -> T;
}

impl<T: Dot<f32> + std::ops::Sub<Output = T> + Copy> Distance<f32> for T {
    fn distance(self, other: Self) -> f32 {
        let delta: Self = self - other;
        (delta.dot(delta) + f32::EPSILON).sqrt()
    }
}
