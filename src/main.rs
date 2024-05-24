mod defer;
mod ffi;
mod geom;
mod math;
mod pathfinding;
mod prelude;

use crate::defer::Defer;
use crate::geom::{Geom, Line};
use crate::math::{Distance, Dot, Mat4, Normalize, Vec2, Vec3, Vec4};
use std::convert::TryInto;
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::fs::read_to_string;
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

const VIEW_DISTANCE: f32 = 600.0;
const VIEW_UP: Vec3<f32> = Vec3 { x: 0.0, y: 1.0, z: 0.0 };

const LINE_WIDTH: f32 = 4.0;

const FIRST_WAYPOINT_INDEX: usize = 5;
const WAYPOINT_LEN: usize = 22 - FIRST_WAYPOINT_INDEX;

const WAYPOINT_SCALE: f32 = 15.0;

const PLAYER_ACCEL: f32 = 2.125;
const PLAYER_DRAG: f32 = 0.725;

const PLAYER_QUAD_SCALE: f32 = 25.0;
const PLAYER_LINE_SCALE: f32 = 6.75;

const QUADS_LEN: usize = 23;
const PLAYER_QUAD_IDX: usize = QUADS_LEN - 1;

const LINES_LEN: usize = 12;
const PLAYER_LINE_IDX: usize = LINES_LEN - 1;
const CURSOR_LINE_IDX: usize = LINES_LEN - 2;

const PLAYER_QUAD_COLOR: Vec4<f32> = Vec4 { x: 1.0, y: 0.5, z: 0.75, w: 1.0 };
const PLAYER_LINE_COLOR: Vec4<f32> = Vec4 { w: 0.375, ..PLAYER_QUAD_COLOR };
const CURSOR_LINE_COLOR: Vec4<f32> = Vec4 { w: 0.15, ..PLAYER_QUAD_COLOR };
const BACKGROUND_COLOR: Vec4<f32> = Vec4 { x: 0.1, y: 0.09, z: 0.11, w: 1.0 };
const WALL_COLOR: Vec4<f32> = Vec4 { x: 1.0, y: 1.0, z: 1.0, w: 0.9 };
const PATH_COLOR: Vec4<f32> = Vec4 { x: 0.6, y: 0.85, z: 0.9, w: 0.0375 };
const WAYPOINT_COLOR: Vec4<f32> = Vec4 { x: 0.4, y: 0.875, z: 0.9, w: 0.2 };
const WAYPOINT_HIGHLIGHT_COLOR: Vec4<f32> = Vec4 { x: 1.0, ..WAYPOINT_COLOR };

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

fn create_program() -> ffi::GLuint {
    let program = unsafe { ffi::glCreateProgram() };

    let vert_shader = compile_shader(
        ffi::GL_VERTEX_SHADER,
        &read_to_string(Path::new("src").join("vert.glsl")).unwrap(),
    );
    defer!(unsafe {
        ffi::glDeleteShader(vert_shader);
    });

    let frag_shader = compile_shader(
        ffi::GL_FRAGMENT_SHADER,
        &read_to_string(Path::new("src").join("frag.glsl")).unwrap(),
    );
    defer!(unsafe {
        ffi::glDeleteShader(frag_shader);
    });

    unsafe {
        ffi::glAttachShader(program, vert_shader);
        ffi::glAttachShader(program, frag_shader);
        ffi::glLinkProgram(program);
    }

    program
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

fn update_camera(
    window: *mut ffi::GLFWwindow,
    camera: &mut Vec3<f32>,
    camera_speed: &mut Vec2<f32>,
) {
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
    *camera_speed += step.normalize() * CAMERA_ACCEL.into();
    *camera_speed *= CAMERA_DRAG.into();

    camera.x += camera_speed.x;
    camera.y += camera_speed.y;
}

fn update_cursor(
    window: *mut ffi::GLFWwindow,
    inverse_projection: &Mat4<f32>,
    world_cursor: &mut Vec2<f32>,
) {
    let mut screen_cursor: Vec2<f64> = Vec2::default();
    unsafe {
        ffi::glfwGetCursorPos(window, &mut screen_cursor.x, &mut screen_cursor.y);
    }
    screen_cursor.x /= f64::from(WINDOW_WIDTH);
    screen_cursor.y /= f64::from(WINDOW_HEIGHT);
    screen_cursor -= 0.5.into();
    screen_cursor *= 2.0.into();
    screen_cursor.y = -screen_cursor.y;

    #[allow(clippy::cast_possible_truncation)]
    let screen_cursor = Vec4 {
        x: screen_cursor.x as f32,
        y: screen_cursor.y as f32,
        z: 0.0,
        w: 1.0,
    };

    let unprojected_cursor = screen_cursor.dot(inverse_projection);

    *world_cursor = Vec2 {
        x: unprojected_cursor.x,
        y: unprojected_cursor.y,
    };
    *world_cursor *= VIEW_DISTANCE.into();
}

fn update_player<const N: usize>(
    quads: &mut [Geom<f32>],
    weights: &[[f32; N]; N],
    path: &mut [usize; N],
    player_speed: &mut Vec2<f32>,
    player_waypoint_idx: &mut usize,
    cursor_waypoint_idx: usize,
) {
    let path_len = pathfinding::dijkstra(
        weights,
        *player_waypoint_idx - FIRST_WAYPOINT_INDEX,
        cursor_waypoint_idx - FIRST_WAYPOINT_INDEX,
        path,
    );

    let distance = |i: usize, j: usize| quads[i].translate.0.distance(quads[j].translate.0);

    let mut gap = distance(*player_waypoint_idx, PLAYER_QUAD_IDX);
    if (1 < path_len) && (gap <= (PLAYER_QUAD_SCALE / 2.0)) {
        *player_waypoint_idx = FIRST_WAYPOINT_INDEX + path[1];
        gap = distance(*player_waypoint_idx, PLAYER_QUAD_IDX);
    }

    if (PLAYER_QUAD_SCALE / 2.0) < gap {
        let step = quads[*player_waypoint_idx].translate.0 - quads[PLAYER_QUAD_IDX].translate.0;
        *player_speed += step.normalize() * PLAYER_ACCEL.into();
    }
    *player_speed *= PLAYER_DRAG.into();
    quads[PLAYER_QUAD_IDX].translate.0 += *player_speed;
}

fn update_lines(
    quads: &[Geom<f32>],
    lines: &mut [Geom<f32>],
    player_speed: Vec2<f32>,
    world_cursor: Vec2<f32>,
) {
    let player_line = Line(
        quads[PLAYER_QUAD_IDX].translate.0,
        quads[PLAYER_QUAD_IDX].translate.0 + (player_speed * PLAYER_LINE_SCALE.into()),
    );
    lines[PLAYER_LINE_IDX].translate = player_line.into();
    lines[PLAYER_LINE_IDX].scale = player_line.into();

    let cursor_line = Line(quads[PLAYER_QUAD_IDX].translate.0, world_cursor);
    lines[CURSOR_LINE_IDX].translate = cursor_line.into();
    lines[CURSOR_LINE_IDX].scale = cursor_line.into();
}

fn main() {
    #[allow(clippy::cast_precision_loss)]
    let projection = math::perspective(
        45.0,
        (WINDOW_WIDTH as f32) / (WINDOW_HEIGHT as f32),
        VIEW_DISTANCE - 0.1,
        VIEW_DISTANCE + 0.1,
    );
    let inverse_projection: Mat4<f32> = math::inverse_perspective(&projection);

    let mut camera = Vec3 { x: 0.0, y: 0.0, z: VIEW_DISTANCE };

    let mut player_speed: Vec2<f32> = Vec2::default();
    let mut camera_speed: Vec2<f32> = Vec2::default();

    let mut player_waypoint_idx = 5;

    let mut world_cursor = Vec2::default();

    let mut quads: [Geom<f32>; QUADS_LEN] = [
        Geom {
            translate: Vec2::default().into(),
            scale: Vec2 { x: 600.0, y: 600.0 }.into(),
            color: Vec4 {
                x: 0.325,
                y: 0.375,
                z: 0.525,
                w: 0.25,
            }
            .into(),
        },
        Geom {
            translate: Vec2 { x: 50.0, y: 50.0 }.into(),
            scale: Vec2 { x: 310.0, y: 10.0 }.into(),
            color: WALL_COLOR.into(),
        },
        Geom {
            translate: Vec2 { x: -100.0, y: -75.0 }.into(),
            scale: Vec2 { x: 10.0, y: 260.0 }.into(),
            color: WALL_COLOR.into(),
        },
        Geom {
            translate: Vec2 { x: 200.0, y: 125.0 }.into(),
            scale: Vec2 { x: 10.0, y: 160.0 }.into(),
            color: WALL_COLOR.into(),
        },
        Geom {
            translate: Vec2 { x: -150.0, y: -200.0 }.into(),
            scale: Vec2 { x: 110.0, y: 10.0 }.into(),
            color: WALL_COLOR.into(),
        },
        Geom {
            translate: Vec2 { x: -50.0, y: 0.0 }.into(),
            scale: WAYPOINT_SCALE.into(),
            color: WAYPOINT_COLOR.into(),
        },
        Geom {
            translate: Vec2 { x: 250.0, y: 0.0 }.into(),
            scale: WAYPOINT_SCALE.into(),
            color: WAYPOINT_COLOR.into(),
        },
        Geom {
            translate: Vec2 { x: 250.0, y: 50.0 }.into(),
            scale: WAYPOINT_SCALE.into(),
            color: WAYPOINT_COLOR.into(),
        },
        Geom {
            translate: Vec2 { x: 250.0, y: 100.0 }.into(),
            scale: WAYPOINT_SCALE.into(),
            color: WAYPOINT_COLOR.into(),
        },
        Geom {
            translate: Vec2 { x: 250.0, y: 150.0 }.into(),
            scale: WAYPOINT_SCALE.into(),
            color: WAYPOINT_COLOR.into(),
        },
        Geom {
            translate: Vec2 { x: 250.0, y: 200.0 }.into(),
            scale: WAYPOINT_SCALE.into(),
            color: WAYPOINT_COLOR.into(),
        },
        Geom {
            translate: Vec2 { x: 250.0, y: 250.0 }.into(),
            scale: WAYPOINT_SCALE.into(),
            color: WAYPOINT_COLOR.into(),
        },
        Geom {
            translate: Vec2 { x: 200.0, y: 250.0 }.into(),
            scale: WAYPOINT_SCALE.into(),
            color: WAYPOINT_COLOR.into(),
        },
        Geom {
            translate: Vec2 { x: 150.0, y: 250.0 }.into(),
            scale: WAYPOINT_SCALE.into(),
            color: WAYPOINT_COLOR.into(),
        },
        Geom {
            translate: Vec2 { x: 150.0, y: 200.0 }.into(),
            scale: WAYPOINT_SCALE.into(),
            color: WAYPOINT_COLOR.into(),
        },
        Geom {
            translate: Vec2 { x: 150.0, y: 150.0 }.into(),
            scale: WAYPOINT_SCALE.into(),
            color: WAYPOINT_COLOR.into(),
        },
        Geom {
            translate: Vec2 { x: 150.0, y: 100.0 }.into(),
            scale: WAYPOINT_SCALE.into(),
            color: WAYPOINT_COLOR.into(),
        },
        Geom {
            translate: Vec2 { x: -150.0, y: 100.0 }.into(),
            scale: WAYPOINT_SCALE.into(),
            color: WAYPOINT_COLOR.into(),
        },
        Geom {
            translate: Vec2 { x: -150.0, y: -150.0 }.into(),
            scale: WAYPOINT_SCALE.into(),
            color: WAYPOINT_COLOR.into(),
        },
        Geom {
            translate: Vec2 { x: -250.0, y: -150.0 }.into(),
            scale: WAYPOINT_SCALE.into(),
            color: WAYPOINT_COLOR.into(),
        },
        Geom {
            translate: Vec2 { x: -250.0, y: -250.0 }.into(),
            scale: WAYPOINT_SCALE.into(),
            color: WAYPOINT_COLOR.into(),
        },
        Geom {
            translate: Vec2 { x: -50.0, y: -250.0 }.into(),
            scale: WAYPOINT_SCALE.into(),
            color: WAYPOINT_COLOR.into(),
        },
        Geom {
            translate: Vec2::default().into(),
            scale: PLAYER_QUAD_SCALE.into(),
            color: PLAYER_QUAD_COLOR.into(),
        },
    ];

    let nodes = {
        let mut nodes: [Vec2<f32>; WAYPOINT_LEN] = [Vec2::default(); WAYPOINT_LEN];
        for i in 0..WAYPOINT_LEN {
            nodes[i] = quads[FIRST_WAYPOINT_INDEX + i].translate.0;
        }
        nodes
    };

    let edges = {
        let mut edges: [(usize, usize); WAYPOINT_LEN] =
            [(WAYPOINT_LEN, WAYPOINT_LEN); WAYPOINT_LEN];
        for (i, edge) in edges.iter_mut().enumerate().take(WAYPOINT_LEN) {
            *edge = (i, (i + 1) % WAYPOINT_LEN);
        }
        edges
    };

    let mut weights = [[0.0; WAYPOINT_LEN]; WAYPOINT_LEN];
    pathfinding::init(&nodes, &edges, &mut weights);
    let mut path: [usize; WAYPOINT_LEN] = [WAYPOINT_LEN; WAYPOINT_LEN];

    let mut lines: [Geom<f32>; LINES_LEN] = [
        Geom {
            translate: Vec2 { x: 100.0, y: 0.0 }.into(),
            scale: Vec2 { x: 310.0, y: 0.0 }.into(),
            color: PATH_COLOR.into(),
        },
        Geom {
            translate: Vec2 { x: -50.0, y: -125.0 }.into(),
            scale: Vec2 { x: 0.0, y: 260.0 }.into(),
            color: PATH_COLOR.into(),
        },
        Geom {
            translate: Vec2 { x: 250.0, y: 125.0 }.into(),
            scale: Vec2 { x: 0.0, y: 260.0 }.into(),
            color: PATH_COLOR.into(),
        },
        Geom {
            translate: Vec2 { x: 200.0, y: 250.0 }.into(),
            scale: Vec2 { x: 110.0, y: 0.0 }.into(),
            color: PATH_COLOR.into(),
        },
        Geom {
            translate: Vec2 { x: 150.0, y: 175.0 }.into(),
            scale: Vec2 { x: 0.0, y: 160.0 }.into(),
            color: PATH_COLOR.into(),
        },
        Geom {
            translate: Vec2 { x: 0.0, y: 100.0 }.into(),
            scale: Vec2 { x: 310.0, y: 0.0 }.into(),
            color: PATH_COLOR.into(),
        },
        Geom {
            translate: Vec2 { x: -150.0, y: -25.0 }.into(),
            scale: Vec2 { x: 0.0, y: 260.0 }.into(),
            color: PATH_COLOR.into(),
        },
        Geom {
            translate: Vec2 { x: -200.0, y: -150.0 }.into(),
            scale: Vec2 { x: 110.0, y: 0.0 }.into(),
            color: PATH_COLOR.into(),
        },
        Geom {
            translate: Vec2 { x: -150.0, y: -250.0 }.into(),
            scale: Vec2 { x: 210.0, y: 0.0 }.into(),
            color: PATH_COLOR.into(),
        },
        Geom {
            translate: Vec2 { x: -250.0, y: -200.0 }.into(),
            scale: Vec2 { x: 0.0, y: 110.0 }.into(),
            color: PATH_COLOR.into(),
        },
        Geom {
            translate: Vec2::default().into(),
            scale: Vec2::default().into(),
            color: CURSOR_LINE_COLOR.into(),
        },
        Geom {
            translate: Vec2::default().into(),
            scale: Vec2::default().into(),
            color: PLAYER_LINE_COLOR.into(),
        },
    ];

    unsafe {
        println!("{}", CStr::from_ptr(ffi::glfwGetVersionString()).to_str().unwrap());

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

    let instance_vbo = unsafe {
        let mut instance_vbo: [ffi::GLuint; 2] = [0; 2];
        ffi::glGenBuffers(instance_vbo.len().try_into().unwrap(), instance_vbo.as_mut_ptr());
        instance_vbo
    };
    defer!(unsafe {
        ffi::glDeleteBuffers(instance_vbo.len().try_into().unwrap(), instance_vbo.as_ptr());
    });

    let program = create_program();
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

    println!("\n\n\n\n");
    while unsafe { ffi::glfwWindowShouldClose(window) } != 1 {
        let elapsed = now.elapsed();
        if 0 < elapsed.as_secs() {
            println!(
                "\x1B[5A\
                 {:12.2} elapsed ns\n\
                 {frames:12} frames\n\
                 {:12} ns / frame\n\
                 {:12.2} world_cursor.x\n\
                 {:12.2} world_cursor.y",
                elapsed.as_nanos(),
                elapsed.as_nanos() / frames,
                world_cursor.x,
                world_cursor.y,
            );
            now = time::Instant::now();
            frames = 0;
        }

        unsafe {
            ffi::glfwPollEvents();
        }

        update_camera(window, &mut camera, &mut camera_speed);
        update_cursor(window, &inverse_projection, &mut world_cursor);

        world_cursor.x += camera.x;
        world_cursor.y += camera.y;

        let cursor_waypoint_idx = geom::nearest(
            &quads[FIRST_WAYPOINT_INDEX..(FIRST_WAYPOINT_INDEX + WAYPOINT_LEN)],
            world_cursor,
        ) + FIRST_WAYPOINT_INDEX;

        update_player(
            &mut quads,
            &weights,
            &mut path,
            &mut player_speed,
            &mut player_waypoint_idx,
            cursor_waypoint_idx,
        );
        update_lines(&quads, &mut lines, player_speed, world_cursor);

        quads[cursor_waypoint_idx].color.0 = WAYPOINT_HIGHLIGHT_COLOR;

        let view = math::look_at(camera, Vec3 { z: 0.0, ..camera }, VIEW_UP);

        unsafe {
            uniform!(program, view);
            ffi::glClear(ffi::GL_COLOR_BUFFER_BIT);
        }

        bind_and_draw(vao[0], instance_vbo[0], &quads, &QUAD_VERTICES, ffi::GL_TRIANGLE_STRIP);
        bind_and_draw(vao[1], instance_vbo[1], &lines, &LINE_VERTICES, ffi::GL_LINES);

        unsafe {
            ffi::glfwSwapBuffers(window);
        }

        quads[cursor_waypoint_idx].color.0 = WAYPOINT_COLOR;

        frames += 1;
    }
}
