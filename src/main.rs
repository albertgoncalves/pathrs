mod defer;
mod ffi;
mod geom;
mod math;
mod pathfinding;
mod prelude;

use crate::defer::Defer;
use crate::geom::{Geom, Line, Scale, Translate};
use crate::math::{Distance, Dot, Mat4, Normalize, Vec2, Vec3, Vec4};
use std::collections::HashMap;
use std::convert::TryInto;
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::fs;
use std::mem;
use std::path::Path;
use std::ptr;
use std::slice::from_raw_parts;
use std::str::from_utf8_unchecked;
use std::time;

const QUAD_VERTICES: [Vec2<f32>; 4] = [
    Vec2 { x: 0.5, y: 0.5 },
    Vec2 { x: 0.5, y: -0.5 },
    Vec2 { x: -0.5, y: 0.5 },
    Vec2 { x: -0.5, y: -0.5 },
];
const LINE_VERTICES: [Vec2<f32>; 2] = [Vec2 { x: -0.5, y: -0.5 }, Vec2 { x: 0.5, y: 0.5 }];

const WINDOW_WIDTH: i32 = 1400;
const WINDOW_HEIGHT: i32 = 900;

const CAMERA_ACCEL: f32 = 1.1125;
const CAMERA_DRAG: f32 = 0.8925;

const VIEW_DISTANCE: f32 = 500.0;
const VIEW_UP: Vec3<f32> = Vec3 { x: 0.0, y: 1.0, z: 0.0 };

const LINE_WIDTH: f32 = 4.0;

const PLAYER_ACCEL: f32 = 0.6975;
const PLAYER_DRAG: f32 = 0.825;

const PLAYER_QUAD_SCALE: f32 = 16.5;
const PLAYER_LINE_SCALE: f32 = 6.75;
const FLOOR_SCALE: f32 = 35.0;
const WAYPOINT_SCALE: f32 = 4.5;
const WAYPOINT_HIGHLIGHT_SCALE: f32 = 5.75;

const BACKGROUND_COLOR: Vec4<f32> = Vec4 { x: 0.1, y: 0.09, z: 0.11, w: 1.0 };
const FLOOR_COLOR: Vec4<f32> = Vec4 {
    x: 0.325,
    y: 0.375,
    z: 0.525,
    w: 0.25,
};
const WALL_COLOR: Vec4<f32> = Vec4 { x: 1.0, y: 1.0, z: 1.0, w: 0.9 };
const PLAYER_QUAD_COLOR: Vec4<f32> = Vec4 { x: 1.0, y: 0.5, z: 0.75, w: 1.0 };
const PLAYER_LINE_COLOR: Vec4<f32> = Vec4 { w: 0.375, ..PLAYER_QUAD_COLOR };
const CURSOR_LINE_COLOR: Vec4<f32> = Vec4 { w: 0.15, ..PLAYER_QUAD_COLOR };
const WAYPOINT_COLOR: Vec4<f32> = Vec4 { x: 0.4, y: 0.875, z: 0.9, w: 0.1 };
const WAYPOINT_HIGHLIGHT_COLOR: Vec4<f32> = Vec4 { y: 1.0, w: 0.675, ..WAYPOINT_COLOR };

extern "C" fn callback_glfw_error(error_code: c_int, description: *const c_char) {
    let mut message = error_code.to_string();
    if !description.is_null() {
        message
            .push_str(&format!(": {}", unsafe { CStr::from_ptr(description) }.to_str().unwrap()));
    }
    panic!("{}", message);
}

extern "C" fn callback_glfw_key(
    window: *mut ffi::GLFWwindow,
    key: c_int,
    _scancode: c_int,
    action: c_int,
    _mods: c_int,
) {
    if action != ffi::GLFW_PRESS {
        return;
    }
    if key == ffi::GLFW_KEY_ESCAPE {
        unsafe {
            ffi::glfwSetWindowShouldClose(window, 1);
        }
    }
}

extern "C" fn callback_gl_debug(
    _source: ffi::GLenum,
    _type: ffi::GLenum,
    _id: ffi::GLuint,
    _severity: ffi::GLenum,
    length: ffi::GLsizei,
    message: *const ffi::GLchar,
    #[allow(non_snake_case)] _userParam: *const c_void,
) {
    assert!(0 < length);
    let message: &str = unsafe {
        from_utf8_unchecked(from_raw_parts(message.cast::<u8>(), length.try_into().unwrap()))
    };
    panic!("{}", message);
}

fn compile_shader(r#type: ffi::GLenum, source: &str) -> ffi::GLuint {
    unsafe {
        let shader = ffi::glCreateShader(r#type);
        ffi::glShaderSource(
            shader,
            1,
            [source.as_bytes().as_ptr().cast::<ffi::GLchar>()].as_ptr(),
            [source.as_bytes().len().try_into().unwrap()].as_ptr(),
        );
        ffi::glCompileShader(shader);
        shader
    }
}

fn buffer<T>(target: ffi::GLuint, data: &[T], usage: ffi::GLenum) {
    unsafe {
        ffi::glBindBuffer(ffi::GL_ARRAY_BUFFER, target);
        ffi::glBufferData(
            ffi::GL_ARRAY_BUFFER,
            mem::size_of_val(data).try_into().unwrap(),
            data.as_ptr().cast::<c_void>(),
            usage,
        );
    }
}

macro_rules! index {
    ($program:expr, $ident:ident $(,)?) => {
        ffi::glGetAttribLocation(
            $program,
            CStr::from_bytes_with_nul(concat!(stringify!($ident), '\0').as_bytes())
                .unwrap()
                .as_ptr()
                .cast::<ffi::GLchar>(),
        )
        .try_into()
        .unwrap()
    };
}

macro_rules! attribute {
    ($program:expr, $ty:ty, $ident:ident $(,)?) => {{
        let index = index!($program, $ident);
        ffi::glEnableVertexAttribArray(index);
        ffi::glVertexAttribPointer(
            index,
            (mem::size_of::<$ty>() / mem::size_of::<ffi::GLfloat>()).try_into().unwrap(),
            ffi::GL_FLOAT,
            ffi::GL_FALSE,
            (mem::size_of::<$ty>()).try_into().unwrap(),
            ptr::null::<c_void>(),
        );
    }};
    ($program:expr, $ty:ty, $field:ident, $div:expr $(,)?) => {{
        let index = index!($program, $field);
        ffi::glEnableVertexAttribArray(index);
        ffi::glVertexAttribPointer(
            index,
            (size_of_field!($ty, $field) / mem::size_of::<ffi::GLfloat>())
                .try_into()
                .unwrap(),
            ffi::GL_FLOAT,
            ffi::GL_FALSE,
            (mem::size_of::<$ty>()).try_into().unwrap(),
            mem::offset_of!($ty, $field) as *const c_void,
        );
        ffi::glVertexAttribDivisor(index, $div);
    }};
}

macro_rules! uniform {
    ($program:expr, $ident:ident $(,)?) => {
        ffi::glUniformMatrix4fv(
            ffi::glGetUniformLocation(
                $program,
                CStr::from_bytes_with_nul(concat!(stringify!($ident), '\0').as_bytes())
                    .unwrap()
                    .as_ptr()
                    .cast::<ffi::GLchar>(),
            ),
            1,
            ffi::GL_FALSE,
            $ident.0.as_ptr().cast::<ffi::GLfloat>(),
        );
    };
}

fn buffers_and_attributes(
    program: ffi::GLuint,
    vao: ffi::GLuint,
    vbo: ffi::GLuint,
    instance_vbo: ffi::GLuint,
    geoms: &[Geom<ffi::GLfloat>],
    vertices: &[Vec2<ffi::GLfloat>],
) {
    unsafe {
        ffi::glBindVertexArray(vao);
    }

    buffer(vbo, vertices, ffi::GL_STATIC_DRAW);
    unsafe {
        attribute!(program, Vec2<ffi::GLfloat>, position);
    }

    buffer(instance_vbo, geoms, ffi::GL_DYNAMIC_DRAW);
    unsafe {
        attribute!(program, Geom<ffi::GLfloat>, translate, 1);
        attribute!(program, Geom<ffi::GLfloat>, scale, 1);
        attribute!(program, Geom<ffi::GLfloat>, color, 1);
    }
}

fn bind_and_draw(
    vao: ffi::GLuint,
    instance_vbo: ffi::GLuint,
    geoms: &[Geom<ffi::GLfloat>],
    vertices: &[Vec2<ffi::GLfloat>],
    mode: ffi::GLenum,
) {
    unsafe {
        ffi::glBindVertexArray(vao);
        ffi::glBindBuffer(ffi::GL_ARRAY_BUFFER, instance_vbo);
        ffi::glBufferSubData(
            ffi::GL_ARRAY_BUFFER,
            0,
            mem::size_of_val(geoms).try_into().unwrap(),
            geoms.as_ptr().cast::<c_void>(),
        );
        ffi::glDrawArraysInstanced(
            mode,
            0,
            vertices.len().try_into().unwrap(),
            geoms.len().try_into().unwrap(),
        );
    }
}

fn pressed(window: *mut ffi::GLFWwindow, key: c_int) -> bool {
    unsafe { ffi::glfwGetKey(window, key) == ffi::GLFW_PRESS }
}

#[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
fn main() {
    #[allow(clippy::cast_precision_loss)]
    let projection = math::perspective(
        45.0,
        (WINDOW_WIDTH as f32) / (WINDOW_HEIGHT as f32),
        VIEW_DISTANCE - 100.0,
        VIEW_DISTANCE + 100.0,
    );
    let inverse_projection: Mat4<f32> = math::inverse_perspective(&projection);

    let mut camera = Vec3 { x: 0.0, y: 0.0, z: VIEW_DISTANCE };

    let mut player_speed: Vec2<f32> = Vec2::default();
    let mut camera_speed: Vec2<f32> = Vec2::default();

    let mut world_cursor = Vec3::default();

    let mut quads = vec![Geom {
        translate: Vec2::default().into(),
        scale: Vec2::<f32>::from(PLAYER_QUAD_SCALE).into(),
        color: PLAYER_QUAD_COLOR.into(),
    }];
    let player_quad_idx = 0;

    let mut lines = vec![
        Geom {
            translate: Vec2::default().into(),
            scale: Vec2::default().into(),
            color: PLAYER_LINE_COLOR.into(),
        },
        Geom {
            translate: Vec2::default().into(),
            scale: Vec2::default().into(),
            color: CURSOR_LINE_COLOR.into(),
        },
    ];
    let player_line_idx = 0;
    let cursor_line_idx = 1;

    let (bounds, horizontals, verticals, waypoints) = {
        let floor_plan = fs::read(Path::new("assets").join("floor-plan.txt")).unwrap();

        let mut horizontals = vec![];
        let mut verticals = vec![];
        let mut waypoints = vec![];

        let mut x: u8 = 0;
        let mut y: u8 = 0;
        let mut w: u8 = 0;
        let mut h: u8 = 0;
        for byte in &floor_plan {
            match byte {
                b'\n' => {
                    x = 0;
                    y += 1;
                }
                _ => x += 1,
            }
            w = w.max(x);
            h = h.max(y);
        }

        x = 0;
        y = 0;
        for byte in floor_plan {
            match byte {
                b'\n' => {
                    assert!(x == w);
                    x = 0;
                    y += 1;
                }
                b'+' => {
                    horizontals.push(Vec2 { x, y });
                    verticals.push(Vec2 { x, y });
                    x += 1;
                }
                b'-' => {
                    horizontals.push(Vec2 { x, y });
                    x += 1;
                }
                b'|' => {
                    verticals.push(Vec2 { x, y });
                    x += 1;
                }
                b'.' => {
                    waypoints.push(Vec2 { x, y });
                    x += 1;
                }
                _ => panic!(),
            }
        }
        assert!(y == h);

        verticals.sort_unstable();
        (Vec2 { x: w, y: h }, horizontals, verticals, waypoints)
    };

    let walls = {
        let mut walls = vec![];

        walls.push((Line(horizontals[0], horizontals[0]), true));
        for horizontal in horizontals.into_iter().skip(1) {
            let n = walls.len() - 1;
            if (walls[n].0 .0.y != horizontal.y) || (walls[n].0 .1.x != (horizontal.x - 1)) {
                walls.push((Line(horizontal, horizontal), true));
                continue;
            }
            walls[n].0 .1.x = horizontal.x;
        }

        walls.push((Line(verticals[0], verticals[0]), false));
        for vertical in verticals.into_iter().skip(1) {
            let n = walls.len() - 1;
            if (walls[n].0 .0.x != vertical.x) || (walls[n].0 .1.y != (vertical.y - 1)) {
                walls.push((Line(vertical, vertical), false));
                continue;
            }
            walls[n].0 .1.y = vertical.y;
        }

        walls
    };

    let k = Vec2 { x: FLOOR_SCALE, y: -FLOOR_SCALE };
    let half_k = k * 0.5.into();
    let half_bounds = Vec2 {
        x: f32::from(bounds.x) * 0.5,
        y: f32::from(bounds.y) * 0.5,
    };

    quads.push(Geom {
        translate: Vec2::default().into(),
        scale: Vec2 {
            x: f32::from(bounds.x) * FLOOR_SCALE,
            y: f32::from(bounds.y) * FLOOR_SCALE,
        }
        .into(),
        color: FLOOR_COLOR.into(),
    });

    for (wall, horizontal) in walls {
        let wall = Line(
            Vec2 {
                x: f32::from(wall.0.x),
                y: f32::from(wall.0.y),
            },
            Vec2 {
                x: f32::from(wall.1.x),
                y: f32::from(wall.1.y),
            },
        );

        let mut translate: Translate<f32> = wall.into();
        translate.0 -= half_bounds;
        translate.0 *= k;
        translate.0 += half_k;

        let mut scale: Scale<f32> = wall.into();
        scale.0.x = scale.0.x.abs();
        scale.0.y = scale.0.y.abs();
        scale.0 += 1.0.into();

        if horizontal {
            scale.0.x *= k.x;
        } else {
            scale.0.y *= k.y;
        }

        quads.push(Geom {
            translate,
            scale,
            color: WALL_COLOR.into(),
        });
    }

    let first_waypoint_idx = quads.len();

    let (nodes, map) = {
        let mut nodes = Vec::with_capacity(waypoints.len());
        let mut map = HashMap::with_capacity(waypoints.len());

        for (i, waypoint) in waypoints.iter().enumerate() {
            let mut translate: Translate<f32> = Vec2 {
                x: f32::from(waypoint.x),
                y: f32::from(waypoint.y),
            }
            .into();
            translate.0 -= half_bounds;
            translate.0 *= k;
            translate.0 += half_k;

            quads.push(Geom {
                translate,
                scale: Vec2::<f32>::from(WAYPOINT_SCALE).into(),
                color: WAYPOINT_COLOR.into(),
            });
            nodes.push(translate.0);
            map.insert(waypoint, i);
        }
        (nodes, map)
    };

    let mut player_waypoint_idx = first_waypoint_idx;
    quads[player_quad_idx].translate = quads[first_waypoint_idx].translate;

    let edges = {
        let mut edges = Vec::with_capacity(waypoints.len());
        for (i, waypoint) in waypoints.iter().enumerate() {
            let min_x = waypoint.x.saturating_sub(1);
            let min_y = waypoint.y.saturating_sub(1);
            let max_x = (waypoint.x + 1).min(bounds.x - 1);
            let max_y = (waypoint.y + 1).min(bounds.y - 1);
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    if (x == waypoint.x) && (y == waypoint.y) {
                        continue;
                    }
                    let Some(j) = map.get(&Vec2 { x, y }) else {
                        continue;
                    };
                    assert!(i != *j);
                    edges.push((i, *j));
                }
            }
        }
        edges
    };

    let weights = {
        let mut weights = vec![f32::INFINITY; nodes.len() * nodes.len()];
        for (i, j) in edges {
            assert!(weights[(i * nodes.len()) + j].is_infinite());
            let weight = nodes[i].distance(nodes[j]);
            assert!(weight.is_sign_positive());
            weights[(i * nodes.len()) + j] = weight;
        }
        weights
    };

    println!("{}", unsafe { CStr::from_ptr(ffi::glfwGetVersionString()) }.to_str().unwrap());

    unsafe {
        ffi::glfwSetErrorCallback(callback_glfw_error);
        assert!(ffi::glfwInit() == 1);
    }
    defer!(unsafe {
        ffi::glfwTerminate();
    });

    unsafe {
        ffi::glfwWindowHint(ffi::GLFW_OPENGL_DEBUG_CONTEXT, 1);
        ffi::glfwWindowHint(ffi::GLFW_CONTEXT_VERSION_MAJOR, 3);
        ffi::glfwWindowHint(ffi::GLFW_CONTEXT_VERSION_MINOR, 3);
        ffi::glfwWindowHint(ffi::GLFW_OPENGL_PROFILE, ffi::GLFW_OPENGL_CORE_PROFILE);
        ffi::glfwWindowHint(ffi::GLFW_RESIZABLE, 0);
        ffi::glfwWindowHint(ffi::GLFW_SAMPLES, 16);
    }

    let window = unsafe {
        ffi::glfwCreateWindow(
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            CString::new(std::module_path!())
                .unwrap()
                .as_bytes_with_nul()
                .as_ptr()
                .cast::<c_char>(),
            ptr::null_mut::<ffi::GLFWmonitor>(),
            ptr::null_mut::<ffi::GLFWwindow>(),
        )
    };

    assert!(!window.is_null());

    defer!(unsafe {
        ffi::glfwDestroyWindow(window);
    });

    unsafe {
        ffi::glfwMakeContextCurrent(window);
        ffi::glfwSwapInterval(1);
        ffi::glfwSetKeyCallback(window, callback_glfw_key);

        ffi::glEnable(ffi::GL_DEBUG_OUTPUT);
        ffi::glEnable(ffi::GL_DEBUG_OUTPUT_SYNCHRONOUS);
        ffi::glDebugMessageCallback(callback_gl_debug, ptr::null::<c_void>());

        ffi::glEnable(ffi::GL_BLEND);
        ffi::glBlendFunc(ffi::GL_SRC_ALPHA, ffi::GL_ONE_MINUS_SRC_ALPHA);
        ffi::glClearColor(
            BACKGROUND_COLOR.x,
            BACKGROUND_COLOR.y,
            BACKGROUND_COLOR.z,
            BACKGROUND_COLOR.w,
        );
        ffi::glEnable(ffi::GL_MULTISAMPLE);
        ffi::glViewport(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT);
    }

    let vao = {
        let mut vao: [ffi::GLuint; 2] = [0; 2];
        unsafe {
            ffi::glGenVertexArrays(vao.len().try_into().unwrap(), vao.as_mut_ptr());
        }
        vao
    };
    defer!(unsafe {
        ffi::glDeleteVertexArrays(vao.len().try_into().unwrap(), vao.as_ptr());
    });

    let vbo = {
        let mut vbo: [ffi::GLuint; 2] = [0; 2];
        unsafe {
            ffi::glGenBuffers(vbo.len().try_into().unwrap(), vbo.as_mut_ptr());
        }
        vbo
    };
    defer!(unsafe {
        ffi::glDeleteBuffers(vbo.len().try_into().unwrap(), vbo.as_ptr());
    });

    let instance_vbo = {
        let mut instance_vbo: [ffi::GLuint; 2] = [0; 2];
        unsafe {
            ffi::glGenBuffers(instance_vbo.len().try_into().unwrap(), instance_vbo.as_mut_ptr());
        }
        instance_vbo
    };
    defer!(unsafe {
        ffi::glDeleteBuffers(instance_vbo.len().try_into().unwrap(), instance_vbo.as_ptr());
    });

    let program = unsafe { ffi::glCreateProgram() };
    {
        let vert_shader = compile_shader(
            ffi::GL_VERTEX_SHADER,
            &fs::read_to_string(Path::new("src").join("vert.glsl")).unwrap(),
        );
        defer!(unsafe {
            ffi::glDeleteShader(vert_shader);
        });

        let frag_shader = compile_shader(
            ffi::GL_FRAGMENT_SHADER,
            &fs::read_to_string(Path::new("src").join("frag.glsl")).unwrap(),
        );
        defer!(unsafe {
            ffi::glDeleteShader(frag_shader);
        });

        unsafe {
            ffi::glAttachShader(program, vert_shader);
            ffi::glAttachShader(program, frag_shader);
            ffi::glLinkProgram(program);
        }
    }

    defer!(unsafe {
        ffi::glDeleteProgram(program);
    });

    unsafe {
        ffi::glUseProgram(program);

        ffi::glLineWidth(LINE_WIDTH);
        ffi::glEnable(ffi::GL_LINE_SMOOTH);

        uniform!(program, projection);
    }

    buffers_and_attributes(program, vao[0], vbo[0], instance_vbo[0], &quads, &QUAD_VERTICES);
    buffers_and_attributes(program, vao[1], vbo[1], instance_vbo[1], &lines, &LINE_VERTICES);

    let mut now = time::Instant::now();
    let mut frames = 0;
    let mut path_counter = 0;

    println!("\n\n\n\n\n\n");
    while unsafe { ffi::glfwWindowShouldClose(window) } != 1 {
        let elapsed = now.elapsed();
        if 0 < elapsed.as_secs() {
            println!(
                "\x1B[7A\
                 {:12.2} elapsed ns\n\
                 {frames:12} frames\n\
                 {:12} ns / frame\n\
                 {:12.2} world_cursor.x\n\
                 {:12.2} world_cursor.y\n\
                 {:12.2} world_cursor.z\n\
                 {:12} path_counter",
                elapsed.as_nanos(),
                elapsed.as_nanos() / frames,
                world_cursor.x,
                world_cursor.y,
                world_cursor.z,
                path_counter,
            );
            now = time::Instant::now();
            frames = 0;
        }

        unsafe {
            ffi::glfwPollEvents();
        }

        {
            let mut step: Vec2<f32> = Vec2::default();

            if pressed(window, ffi::GLFW_KEY_W) {
                step.y += 1.0;
            }
            if pressed(window, ffi::GLFW_KEY_S) {
                step.y -= 1.0;
            }
            if pressed(window, ffi::GLFW_KEY_A) {
                step.x -= 1.0;
            }
            if pressed(window, ffi::GLFW_KEY_D) {
                step.x += 1.0;
            }

            camera_speed += step.normalize() * CAMERA_ACCEL.into();
            camera_speed *= CAMERA_DRAG.into();

            camera.x += camera_speed.x;
            camera.y += camera_speed.y;
        }

        let view_to = Vec3 {
            x: camera.x,
            y: camera.y + 100.0,
            z: 0.0,
        };
        let view = math::look_at(camera, view_to, VIEW_UP);

        {
            let mut screen_cursor: Vec2<f64> = Vec2::default();
            unsafe {
                ffi::glfwGetCursorPos(window, &mut screen_cursor.x, &mut screen_cursor.y);
            }

            screen_cursor.x /= f64::from(WINDOW_WIDTH);
            screen_cursor.y /= f64::from(WINDOW_HEIGHT);
            screen_cursor = (screen_cursor * 2.0.into()) - 1.0.into();

            #[allow(clippy::cast_possible_truncation)]
            let mut near = Vec4 {
                x: screen_cursor.x as f32,
                y: -screen_cursor.y as f32,
                z: -1.0,
                w: 1.0,
            };
            let mut far = Vec4 { z: 1.0, ..near };

            near = near.dot(&inverse_projection);
            near /= near.w.into();
            far = far.dot(&inverse_projection);
            far /= far.w.into();

            let inverse_view = math::invert(&view);

            let ray_origin = near.dot(&inverse_view);
            let ray_origin = Vec3 {
                x: ray_origin.x,
                y: ray_origin.y,
                z: ray_origin.z,
            };

            let ray_direction = (far - near).dot(&inverse_view);
            let ray_direction = Vec3 {
                x: ray_direction.x,
                y: ray_direction.y,
                z: ray_direction.z,
            }
            .normalize();

            let plane_origin = Vec3 { x: camera.x, y: camera.y, z: 0.0 };
            let plane_normal = Vec3 { x: 0.0, y: 0.0, z: 1.0 };
            let t = (plane_origin - ray_origin).dot(plane_normal) / plane_normal.dot(ray_direction);

            world_cursor = ray_origin + (ray_direction * t.into());
        };

        let cursor_waypoint_idx = {
            let mut min_gap = f32::INFINITY;
            let mut cursor_waypoint_idx = quads.len();

            for (i, neighbor) in quads[first_waypoint_idx..].iter().enumerate() {
                let gap = Vec2 {
                    x: world_cursor.x,
                    y: world_cursor.y,
                }
                .distance(neighbor.translate.0);
                assert!(gap.is_sign_positive());
                if gap < min_gap {
                    min_gap = gap;
                    cursor_waypoint_idx = i;
                }
            }

            first_waypoint_idx + cursor_waypoint_idx
        };

        let path = pathfinding::shortest_path(
            &nodes,
            &weights,
            player_waypoint_idx - first_waypoint_idx,
            cursor_waypoint_idx - first_waypoint_idx,
            &mut path_counter,
        );
        {
            let gap = {
                let mut gap = (quads[player_waypoint_idx].translate.0)
                    .distance(quads[player_quad_idx].translate.0);
                assert!(gap.is_sign_positive());

                if (1 < path.len()) && (gap <= (PLAYER_QUAD_SCALE / 2.0)) {
                    player_waypoint_idx = first_waypoint_idx + path[1];
                    gap = (quads[player_waypoint_idx].translate.0)
                        .distance(quads[player_quad_idx].translate.0);
                    assert!(gap.is_sign_positive());
                }

                gap
            };

            if (PLAYER_QUAD_SCALE / 2.0) < gap {
                let step =
                    quads[player_waypoint_idx].translate.0 - quads[player_quad_idx].translate.0;
                player_speed += step.normalize() * PLAYER_ACCEL.into();
            }
            player_speed *= PLAYER_DRAG.into();

            quads[player_quad_idx].translate.0 += player_speed;
        }

        {
            let player_line = Line(
                quads[player_quad_idx].translate.0,
                quads[player_quad_idx].translate.0 + (player_speed * PLAYER_LINE_SCALE.into()),
            );
            lines[player_line_idx].translate = player_line.into();
            lines[player_line_idx].scale = player_line.into();

            let cursor_line = Line(
                quads[player_quad_idx].translate.0,
                Vec2 {
                    x: world_cursor.x,
                    y: world_cursor.y,
                },
            );
            lines[cursor_line_idx].translate = cursor_line.into();
            lines[cursor_line_idx].scale = cursor_line.into();
        }

        for i in &path {
            quads[first_waypoint_idx + i].color.0 = WAYPOINT_HIGHLIGHT_COLOR;
            quads[first_waypoint_idx + i].scale.0 = WAYPOINT_HIGHLIGHT_SCALE.into();
        }

        unsafe {
            uniform!(program, view);
            ffi::glClear(ffi::GL_COLOR_BUFFER_BIT);
        }

        bind_and_draw(vao[0], instance_vbo[0], &quads, &QUAD_VERTICES, ffi::GL_TRIANGLE_STRIP);
        bind_and_draw(vao[1], instance_vbo[1], &lines, &LINE_VERTICES, ffi::GL_LINES);

        unsafe {
            ffi::glfwSwapBuffers(window);
        }

        for i in path {
            quads[first_waypoint_idx + i].color.0 = WAYPOINT_COLOR;
            quads[first_waypoint_idx + i].scale.0 = WAYPOINT_SCALE.into();
        }

        frames += 1;
    }
}
