#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T: Copy> From<T> for Vec2<T> {
    fn from(value: T) -> Self {
        Self { x: value, y: value }
    }
}

impl<T: std::ops::Add<Output = T>> std::ops::Add for Vec2<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T: std::ops::Add<Output = T> + Copy> std::ops::Add<T> for Vec2<T> {
    type Output = Self;

    fn add(self, other: T) -> Self {
        Self {
            x: self.x + other,
            y: self.y + other,
        }
    }
}

impl<T: std::ops::AddAssign> std::ops::AddAssign for Vec2<T> {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl<T: std::ops::Sub<Output = T>> std::ops::Sub for Vec2<T> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl<T: std::ops::Mul<Output = T>> std::ops::Mul for Vec2<T> {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }
}

impl<T: std::ops::Mul<Output = T> + Copy> std::ops::Mul<T> for Vec2<T> {
    type Output = Self;

    fn mul(self, other: T) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl<T: std::ops::MulAssign> std::ops::MulAssign for Vec2<T> {
    fn mul_assign(&mut self, other: Self) {
        self.x *= other.x;
        self.y *= other.y;
    }
}

impl<T: std::ops::MulAssign + Copy> std::ops::MulAssign<T> for Vec2<T> {
    fn mul_assign(&mut self, other: T) {
        self.x *= other;
        self.y *= other;
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Vec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: std::ops::Sub<Output = T>> std::ops::Sub for Vec3<T> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl<T: std::ops::Mul<Output = T> + Copy> std::ops::Mul<T> for Vec3<T> {
    type Output = Self;

    fn mul(self, other: T) -> Self {
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
    let cotangent = 1.0 / (fov / 2.0).tan();

    let mut mat = Mat4::default();

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

pub trait Rotate<T> {
    fn rotate(&mut self, center: Self, radians: T);
}

impl Rotate<f32> for Vec2<f32> {
    fn rotate(&mut self, center: Self, radians: f32) {
        let delta = *self - center;
        let sin = radians.sin();
        let cos = radians.cos();
        self.x = delta.y.mul_add(-sin, delta.x.mul_add(cos, center.x));
        self.y = delta.y.mul_add(cos, delta.x.mul_add(sin, center.y));
    }
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
