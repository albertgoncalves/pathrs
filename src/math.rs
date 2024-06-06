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
#[derive(Clone, Copy, Default)]
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

impl<T: ops::Add<Output = T>> ops::Add for Vec3<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
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

impl<T: ops::MulAssign> ops::MulAssign for Vec3<T> {
    fn mul_assign(&mut self, other: Self) {
        self.x *= other.x;
        self.y *= other.y;
        self.z *= other.z;
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
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

pub type Mat4<T> = [[T; 4]; 4];

const fn column<T: Copy>(mat: &Mat4<T>, j: usize) -> [T; 4] {
    [mat[0][j], mat[1][j], mat[2][j], mat[3][j]]
}

#[allow(clippy::many_single_char_names)]
pub fn invert(mat: &Mat4<f32>) -> Mat4<f32> {
    let a = Vec3 {
        x: mat[0][0],
        y: mat[0][1],
        z: mat[0][2],
    };
    let b = Vec3 {
        x: mat[1][0],
        y: mat[1][1],
        z: mat[1][2],
    };
    let c = Vec3 {
        x: mat[2][0],
        y: mat[2][1],
        z: mat[2][2],
    };
    let d = Vec3 {
        x: mat[3][0],
        y: mat[3][1],
        z: mat[3][2],
    };

    let mut e = a.cross(b);
    let mut f = c.cross(d);
    let mut g = (a * mat[1][3].into()) - (b * mat[0][3].into());
    let mut h = (c * mat[3][3].into()) - (d * mat[2][3].into());

    let inv_determinant = (1.0 / (e.dot(h) + f.dot(g))).into();

    e *= inv_determinant;
    f *= inv_determinant;
    g *= inv_determinant;
    h *= inv_determinant;

    let i = b.cross(h) + (f * mat[1][3].into());
    let j = h.cross(a) - (f * mat[0][3].into());
    let k = d.cross(g) + (e * mat[3][3].into());
    let l = g.cross(c) - (e * mat[2][3].into());

    [
        [i.x, j.x, k.x, l.x],
        [i.y, j.y, k.y, l.y],
        [i.z, j.z, k.z, l.z],
        [-b.dot(f), a.dot(f), -d.dot(e), c.dot(e)],
    ]
}

// NOTE: See `https://www.khronos.org/registry/OpenGL-Refpages/gl2.1/xhtml/gluPerspective.xml`.
pub fn perspective(fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Mat4<f32> {
    let cotangent = 1.0 / (fov / 2.0).tan();

    let mut mat = Mat4::default();

    mat[0][0] = cotangent / aspect_ratio;
    mat[1][1] = cotangent;
    mat[2][3] = -1.0;
    mat[2][2] = (near + far) / (near - far);
    mat[3][2] = (2.0 * near * far) / (near - far);

    mat
}

pub fn look_at(from: Vec3<f32>, to: Vec3<f32>, up: Vec3<f32>) -> Mat4<f32> {
    let forward: Vec3<f32> = (to - from).normalize();
    let right: Vec3<f32> = forward.cross(up).normalize();
    let up: Vec3<f32> = right.cross(forward);

    let mut mat = Mat4::default();

    mat[0][0] = right.x;
    mat[0][1] = up.x;
    mat[0][2] = -forward.x;

    mat[1][0] = right.y;
    mat[1][1] = up.y;
    mat[1][2] = -forward.y;

    mat[2][0] = right.z;
    mat[2][1] = up.z;
    mat[2][2] = -forward.z;

    mat[3][0] = -right.dot(from);
    mat[3][1] = -up.dot(from);
    mat[3][2] = forward.dot(from);
    mat[3][3] = 1.0;

    mat
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

#[rustfmt::skip]
impl Dot<[f32; 4], f32> for Vec4<f32> {
    fn dot(self, other: [f32; 4]) -> f32 {
        self.w.mul_add(other[3], self.z.mul_add(other[2], self.y.mul_add(other[1], self.x * other[0])))
    }
}

#[rustfmt::skip]
impl Dot<Self, f32> for [f32; 4] {
    fn dot(self, other: Self) -> f32 {
        self[3].mul_add(other[3], self[2].mul_add(other[2], self[1].mul_add(other[1], self[0] * other[0])))
    }
}

impl<T: Copy> Dot<&Mat4<T>, Self> for Vec4<T>
where
    Self: Dot<[T; 4], T>,
{
    fn dot(self, other: &Mat4<T>) -> Self {
        Self {
            x: self.dot(column(other, 0)),
            y: self.dot(column(other, 1)),
            z: self.dot(column(other, 2)),
            w: self.dot(column(other, 3)),
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
                mat[i][j] = self[i].dot(column(other, j));
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

#[cfg(test)]
mod tests {
    use super::*;

    const IDENTITY: Mat4<f32> = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];

    fn compare(a: &Mat4<f32>, b: &Mat4<f32>, epsilon: f32) -> bool {
        for (i, row) in a.iter().enumerate() {
            for (j, cell) in row.iter().enumerate() {
                if epsilon < (cell - b[i][j]).abs() {
                    return false;
                }
            }
        }
        true
    }

    #[test]
    fn test_inverse_perspective() {
        let projection = perspective(0.45, 1400.0 / 900.0, 1.0, 1000.0);
        assert!(compare(&projection.dot(&invert(&projection)), &IDENTITY, f32::EPSILON));
    }

    #[test]
    fn test_inverse_look_at() {
        let view = look_at(
            Vec3 { x: 40.0, y: 13.5, z: 5.0 },
            Vec3 { x: -11.0, y: 1.0, z: -1.5 },
            Vec3 { x: 0.25, y: 1.0, z: 0.8 }.normalize(),
        );
        assert!(compare(&view.dot(&invert(&view)), &IDENTITY, f32::EPSILON * 8.0));
    }
}
