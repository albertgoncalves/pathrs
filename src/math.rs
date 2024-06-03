use std::ops;

#[repr(C)]
#[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T: Copy> From<T> for Vec2<T> {
    fn from(value: T) -> Self {
        Self { x: value, y: value }
    }
}

impl<T: ops::Add<Output = T>> ops::Add for Vec2<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T: ops::AddAssign> ops::AddAssign for Vec2<T> {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl<T: ops::Sub<Output = T>> ops::Sub for Vec2<T> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl<T: ops::SubAssign> ops::SubAssign for Vec2<T> {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl<T: ops::Mul<Output = T>> ops::Mul for Vec2<T> {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }
}

impl<T: ops::MulAssign> ops::MulAssign for Vec2<T> {
    fn mul_assign(&mut self, other: Self) {
        self.x *= other.x;
        self.y *= other.y;
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: Copy> From<T> for Vec3<T> {
    fn from(value: T) -> Self {
        Self { x: value, y: value, z: value }
    }
}

impl<T: ops::Sub<Output = T>> ops::Sub for Vec3<T> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl<T: ops::AddAssign> ops::AddAssign for Vec3<T> {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl<T: ops::Mul<Output = T>> ops::Mul for Vec3<T> {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
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

impl<T: Copy> From<T> for Vec4<T> {
    fn from(value: T) -> Self {
        Self {
            x: value,
            y: value,
            z: value,
            w: value,
        }
    }
}

impl<T: ops::Sub<Output = T>> ops::Sub for Vec4<T> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
            w: self.w - other.w,
        }
    }
}

impl<T: ops::DivAssign> ops::DivAssign for Vec4<T> {
    fn div_assign(&mut self, other: Self) {
        self.x /= other.x;
        self.y /= other.y;
        self.z /= other.z;
        self.w /= other.w;
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Mat4<T>(pub [[T; 4]; 4]);

#[allow(clippy::suboptimal_flops)]
pub fn invert(mat: &Mat4<f32>) -> Mat4<f32> {
    let mut out = [[0.0; 4]; 4];
    let mut m = [0.0; 4 * 4];
    for i in 0..4 {
        for j in 0..4 {
            m[(j * 4) + i] = mat.0[i][j];
        }
    }

    out[0][0] = m[5] * m[10] * m[15] - m[5] * m[11] * m[14] - m[9] * m[6] * m[15]
        + m[9] * m[7] * m[14]
        + m[13] * m[6] * m[11]
        - m[13] * m[7] * m[10];

    out[1][0] = -m[1] * m[10] * m[15] + m[1] * m[11] * m[14] + m[9] * m[2] * m[15]
        - m[9] * m[3] * m[14]
        - m[13] * m[2] * m[11]
        + m[13] * m[3] * m[10];

    out[2][0] = m[1] * m[6] * m[15] - m[1] * m[7] * m[14] - m[5] * m[2] * m[15]
        + m[5] * m[3] * m[14]
        + m[13] * m[2] * m[7]
        - m[13] * m[3] * m[6];

    out[3][0] = -m[1] * m[6] * m[11] + m[1] * m[7] * m[10] + m[5] * m[2] * m[11]
        - m[5] * m[3] * m[10]
        - m[9] * m[2] * m[7]
        + m[9] * m[3] * m[6];

    out[0][1] = -m[4] * m[10] * m[15] + m[4] * m[11] * m[14] + m[8] * m[6] * m[15]
        - m[8] * m[7] * m[14]
        - m[12] * m[6] * m[11]
        + m[12] * m[7] * m[10];

    out[1][1] = m[0] * m[10] * m[15] - m[0] * m[11] * m[14] - m[8] * m[2] * m[15]
        + m[8] * m[3] * m[14]
        + m[12] * m[2] * m[11]
        - m[12] * m[3] * m[10];

    out[2][1] = -m[0] * m[6] * m[15] + m[0] * m[7] * m[14] + m[4] * m[2] * m[15]
        - m[4] * m[3] * m[14]
        - m[12] * m[2] * m[7]
        + m[12] * m[3] * m[6];

    out[3][1] = m[0] * m[6] * m[11] - m[0] * m[7] * m[10] - m[4] * m[2] * m[11]
        + m[4] * m[3] * m[10]
        + m[8] * m[2] * m[7]
        - m[8] * m[3] * m[6];

    out[0][2] = m[4] * m[9] * m[15] - m[4] * m[11] * m[13] - m[8] * m[5] * m[15]
        + m[8] * m[7] * m[13]
        + m[12] * m[5] * m[11]
        - m[12] * m[7] * m[9];

    out[1][2] = -m[0] * m[9] * m[15] + m[0] * m[11] * m[13] + m[8] * m[1] * m[15]
        - m[8] * m[3] * m[13]
        - m[12] * m[1] * m[11]
        + m[12] * m[3] * m[9];

    out[2][2] = m[0] * m[5] * m[15] - m[0] * m[7] * m[13] - m[4] * m[1] * m[15]
        + m[4] * m[3] * m[13]
        + m[12] * m[1] * m[7]
        - m[12] * m[3] * m[5];

    out[0][3] = -m[4] * m[9] * m[14] + m[4] * m[10] * m[13] + m[8] * m[5] * m[14]
        - m[8] * m[6] * m[13]
        - m[12] * m[5] * m[10]
        + m[12] * m[6] * m[9];

    out[3][2] = -m[0] * m[5] * m[11] + m[0] * m[7] * m[9] + m[4] * m[1] * m[11]
        - m[4] * m[3] * m[9]
        - m[8] * m[1] * m[7]
        + m[8] * m[3] * m[5];

    out[1][3] = m[0] * m[9] * m[14] - m[0] * m[10] * m[13] - m[8] * m[1] * m[14]
        + m[8] * m[2] * m[13]
        + m[12] * m[1] * m[10]
        - m[12] * m[2] * m[9];

    out[2][3] = -m[0] * m[5] * m[14] + m[0] * m[6] * m[13] + m[4] * m[1] * m[14]
        - m[4] * m[2] * m[13]
        - m[12] * m[1] * m[6]
        + m[12] * m[2] * m[5];

    out[3][3] = m[0] * m[5] * m[10] - m[0] * m[6] * m[9] - m[4] * m[1] * m[10]
        + m[4] * m[2] * m[9]
        + m[8] * m[1] * m[6]
        - m[8] * m[2] * m[5];

    let det = m[0] * out[0][0] + m[1] * out[0][1] + m[2] * out[0][2] + m[3] * out[0][3];

    assert!(det != 0.0);
    let inv_det = 1.0 / det;

    #[allow(clippy::needless_range_loop)]
    for j in 0..4 {
        for i in 0..4 {
            out[i][j] *= inv_det;
        }
    }

    Mat4(out)
}

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

pub fn inverse_perspective(mat: &Mat4<f32>) -> Mat4<f32> {
    let mut inv = Mat4::default();

    inv.0[0][0] = 1.0 / mat.0[0][0];
    inv.0[1][1] = 1.0 / mat.0[1][1];
    inv.0[2][3] = 1.0 / mat.0[3][2];
    inv.0[3][3] = mat.0[2][2] * inv.0[2][3];
    inv.0[3][2] = mat.0[2][3];

    inv
}

#[test]
fn test_inverse_perspective() {
    let projection = perspective(0.45, 800.0 / 600.0, 1.0, 1000.0);
    let inverse_projection = inverse_perspective(&projection);
    let identity = projection.dot(&inverse_projection);
    for i in 0..4 {
        for j in 0..4 {
            if i == j {
                assert!((identity.0[i][j] - 1.0).abs() < f32::EPSILON);
            } else {
                assert!(identity.0[i][j].abs() < f32::EPSILON);
            }
        }
    }
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

#[test]
fn test_inverse_look_at() {
    let view = look_at(
        Vec3 { x: 0.0, y: 0.0, z: 100.0 },
        Vec3 { x: 0.0, y: 1.0, z: 0.0 },
        Vec3 { x: 0.0, y: 1.0, z: 0.0 },
    );
    let inverse_view = invert(&view);
    let identity = view.dot(&inverse_view);
    for i in 0..4 {
        for j in 0..4 {
            if i == j {
                assert!((identity.0[i][j] - 1.0).abs() < f32::EPSILON);
            } else {
                assert!(identity.0[i][j].abs() < f32::EPSILON);
            }
        }
    }
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

pub trait Dot<A, B> {
    fn dot(self, other: A) -> B;
}

impl Dot<Self, f32> for Vec2<f32> {
    fn dot(self, other: Self) -> f32 {
        self.y.mul_add(other.y, self.x * other.x)
    }
}

impl Dot<Self, f32> for Vec3<f32> {
    fn dot(self, other: Self) -> f32 {
        self.z.mul_add(other.z, self.y.mul_add(other.y, self.x * other.x))
    }
}

impl Dot<[f32; 4], f32> for Vec4<f32> {
    fn dot(self, other: [f32; 4]) -> f32 {
        self.w.mul_add(
            other[3],
            self.z.mul_add(other[2], self.y.mul_add(other[1], self.x * other[0])),
        )
    }
}

impl Dot<[f32; 4], f32> for [f32; 4] {
    fn dot(self, other: [f32; 4]) -> f32 {
        self[3].mul_add(
            other[3],
            self[2].mul_add(other[2], self[1].mul_add(other[1], self[0] * other[0])),
        )
    }
}

impl<T: Copy> Dot<&Mat4<T>, Self> for Vec4<T>
where
    Self: Dot<[T; 4], T>,
{
    fn dot(self, other: &Mat4<T>) -> Self {
        Self {
            x: self.dot([other.0[0][0], other.0[1][0], other.0[2][0], other.0[3][0]]),
            y: self.dot([other.0[0][1], other.0[1][1], other.0[2][1], other.0[3][1]]),
            z: self.dot([other.0[0][2], other.0[1][2], other.0[2][2], other.0[3][2]]),
            w: self.dot([other.0[0][3], other.0[1][3], other.0[2][3], other.0[3][3]]),
        }
    }
}

impl<T: Copy + Default> Dot<&Self, Self> for Mat4<T>
where
    [T; 4]: Dot<[T; 4], T>,
{
    fn dot(self, other: &Self) -> Self {
        let mut mat = Self::default();

        for i in 0..4 {
            for j in 0..4 {
                mat.0[i][j] =
                    self.0[i].dot([other.0[0][j], other.0[1][j], other.0[2][j], other.0[3][j]]);
            }
        }

        mat
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

impl<T: Dot<T, f32> + ops::Mul<Output = T> + From<f32> + Copy> Normalize for T {
    fn normalize(self) -> Self {
        self * (1.0 / (self.dot(self) + f32::EPSILON).sqrt()).into()
    }
}

pub trait Distance<T> {
    fn distance(self, other: Self) -> T;
}

impl<T: Dot<T, f32> + ops::Sub<Output = T> + Copy> Distance<f32> for T {
    fn distance(self, other: Self) -> f32 {
        let delta: Self = self - other;
        (delta.dot(delta) + f32::EPSILON).sqrt()
    }
}
