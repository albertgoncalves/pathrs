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

const WINDOW_WIDTH: i32 = 1400;
const WINDOW_HEIGHT: i32 = 900;

const CAMERA_ACCEL: f32 = 1.1125;
const CAMERA_DRAG: f32 = 0.8925;

const VIEW_DISTANCE: f32 = 600.0;
const VIEW_UP: Vec3<f32> = Vec3 { x: 0.0, y: 1.0, z: 0.0 };

const FIRST_WAYPOINT_INDEX: usize = 5;
const WAYPOINT_LEN: usize = 22 - FIRST_WAYPOINT_INDEX;

const PLAYER_ACCEL: f32 = 2.125;
const PLAYER_DRAG: f32 = 0.725;

const PLAYER_QUAD_SCALE: f32 = 25.0;
const PLAYER_LINE_SCALE: f32 = 6.75;

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
    unsafe {
        let program = ffi::glCreateProgram();

        let vert_shader = compile_shader(
            ffi::GL_VERTEX_SHADER,
            &read_to_string(Path::new("src").join("vert.glsl")).unwrap(),
        );
        defer!(ffi::glDeleteShader(vert_shader));

        let frag_shader = compile_shader(
            ffi::GL_FRAGMENT_SHADER,
            &read_to_string(Path::new("src").join("frag.glsl")).unwrap(),
        );
        defer!(ffi::glDeleteShader(frag_shader));

        ffi::glAttachShader(program, vert_shader);
        ffi::glAttachShader(program, frag_shader);
        ffi::glLinkProgram(program);

        program
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

        buffer(vbo, vertices, ffi::GL_STATIC_DRAW);
        attribute!(program, Vec2<ffi::GLfloat>, position);

        buffer(instance_vbo, geoms, ffi::GL_DYNAMIC_DRAW);
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
    camera: Vec3<f32>,
    world_cursor: &mut Vec2<f32>,
    quads: &[Geom<f32>],
    inverse_projection: &Mat4<f32>,
) -> usize {
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
    world_cursor.x += camera.x;
    world_cursor.y += camera.y;

    let mut min_distance = f32::INFINITY;
    let mut cursor_waypoint_idx = quads.len();

    #[allow(clippy::needless_range_loop)]
    for i in FIRST_WAYPOINT_INDEX..(FIRST_WAYPOINT_INDEX + WAYPOINT_LEN) {
        let candidate = world_cursor.distance(quads[i].translate.0);
        if candidate < min_distance {
            min_distance = candidate;
            cursor_waypoint_idx = i;
        }
    }

    cursor_waypoint_idx
}

#[allow(clippy::too_many_arguments)]
fn update_player<const N: usize>(
    quads: &mut [Geom<f32>],
    weights: &mut [[f32; N]; N],
    nodes: &[Vec2<f32>],
    edges: &[(usize, usize)],
    player_speed: &mut Vec2<f32>,
    player_quad_idx: usize,
    player_waypoint_idx: &mut usize,
    cursor_waypoint_idx: usize,
) {
    let path = pathfinding::dijkstra(
        weights,
        nodes,
        edges,
        *player_waypoint_idx - FIRST_WAYPOINT_INDEX,
        cursor_waypoint_idx - FIRST_WAYPOINT_INDEX,
    );

    let distance = |i: usize, j: usize| quads[i].translate.0.distance(quads[j].translate.0);

    let mut gap = distance(*player_waypoint_idx, player_quad_idx);
    if (1 < path.len()) && (gap <= (PLAYER_QUAD_SCALE / 2.0)) {
        *player_waypoint_idx = FIRST_WAYPOINT_INDEX + path[1];
        gap = distance(*player_waypoint_idx, player_quad_idx);
    }

    if (PLAYER_QUAD_SCALE / 2.0) < gap {
        let step = quads[*player_waypoint_idx].translate.0 - quads[player_quad_idx].translate.0;
        *player_speed += step.normalize() * PLAYER_ACCEL.into();
    }
    *player_speed *= PLAYER_DRAG.into();
    quads[player_quad_idx].translate.0 += *player_speed;
}

fn update_lines(
    quads: &[Geom<f32>],
    lines: &mut [Geom<f32>],
    player_speed: Vec2<f32>,
    player_quad_idx: usize,
    player_line_idx: usize,
    world_cursor: Vec2<f32>,
    cursor_line_idx: usize,
) {
    let player_line = Line(
        quads[player_quad_idx].translate.0,
        quads[player_quad_idx].translate.0 + (player_speed * PLAYER_LINE_SCALE.into()),
    );

    lines[player_line_idx].translate = player_line.into();
    lines[player_line_idx].scale = player_line.into();

    let cursor_line = Line(quads[player_quad_idx].translate.0, world_cursor);
    lines[cursor_line_idx].translate = cursor_line.into();
    lines[cursor_line_idx].scale = cursor_line.into();
}

fn main() {
    let mut camera = Vec3 {
        x: 0.0,
        y: 0.0,
        z: VIEW_DISTANCE,
    };

    #[allow(clippy::cast_precision_loss)]
    let projection = math::perspective(
        45.0,
        (WINDOW_WIDTH as f32) / (WINDOW_HEIGHT as f32),
        VIEW_DISTANCE - 0.1,
        VIEW_DISTANCE + 0.1,
    );
    let inverse_projection: Mat4<f32> = math::inverse_perspective(&projection);

    let mut player_speed: Vec2<f32> = Vec2::default();
    let mut camera_speed: Vec2<f32> = Vec2::default();

    let waypoint_scale = 15.0;

    let player_quad_color = Vec4 {
        x: 1.0,
        y: 0.5,
        z: 0.75,
        w: 1.0,
    };
    let player_line_color = Vec4 {
        w: 0.375,
        ..player_quad_color
    };
    let cursor_line_color = Vec4 {
        w: 0.15,
        ..player_quad_color
    };
    let background_color = Vec4 {
        x: 0.1,
        y: 0.09,
        z: 0.11,
        w: 1.0,
    };
    let wall_color = Vec4 {
        x: 1.0,
        y: 1.0,
        z: 1.0,
        w: 0.9,
    };
    let path_color = Vec4 {
        x: 0.6,
        y: 0.85,
        z: 0.9,
        w: 0.0375,
    };
    let waypoint_color = Vec4 {
        x: 0.4,
        y: 0.875,
        z: 0.9,
        w: 0.2,
    };

    let mut quads = [
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
            color: wall_color.into(),
        },
        Geom {
            translate: Vec2 { x: -100.0, y: -75.0 }.into(),
            scale: Vec2 { x: 10.0, y: 260.0 }.into(),
            color: wall_color.into(),
        },
        Geom {
            translate: Vec2 { x: 200.0, y: 125.0 }.into(),
            scale: Vec2 { x: 10.0, y: 160.0 }.into(),
            color: wall_color.into(),
        },
        Geom {
            translate: Vec2 { x: -150.0, y: -200.0 }.into(),
            scale: Vec2 { x: 110.0, y: 10.0 }.into(),
            color: wall_color.into(),
        },
        Geom {
            translate: Vec2 { x: -50.0, y: 0.0 }.into(),
            scale: waypoint_scale.into(),
            color: waypoint_color.into(),
        },
        Geom {
            translate: Vec2 { x: 250.0, y: 0.0 }.into(),
            scale: waypoint_scale.into(),
            color: waypoint_color.into(),
        },
        Geom {
            translate: Vec2 { x: 250.0, y: 50.0 }.into(),
            scale: waypoint_scale.into(),
            color: waypoint_color.into(),
        },
        Geom {
            translate: Vec2 { x: 250.0, y: 100.0 }.into(),
            scale: waypoint_scale.into(),
            color: waypoint_color.into(),
        },
        Geom {
            translate: Vec2 { x: 250.0, y: 150.0 }.into(),
            scale: waypoint_scale.into(),
            color: waypoint_color.into(),
        },
        Geom {
            translate: Vec2 { x: 250.0, y: 200.0 }.into(),
            scale: waypoint_scale.into(),
            color: waypoint_color.into(),
        },
        Geom {
            translate: Vec2 { x: 250.0, y: 250.0 }.into(),
            scale: waypoint_scale.into(),
            color: waypoint_color.into(),
        },
        Geom {
            translate: Vec2 { x: 200.0, y: 250.0 }.into(),
            scale: waypoint_scale.into(),
            color: waypoint_color.into(),
        },
        Geom {
            translate: Vec2 { x: 150.0, y: 250.0 }.into(),
            scale: waypoint_scale.into(),
            color: waypoint_color.into(),
        },
        Geom {
            translate: Vec2 { x: 150.0, y: 200.0 }.into(),
            scale: waypoint_scale.into(),
            color: waypoint_color.into(),
        },
        Geom {
            translate: Vec2 { x: 150.0, y: 150.0 }.into(),
            scale: waypoint_scale.into(),
            color: waypoint_color.into(),
        },
        Geom {
            translate: Vec2 { x: 150.0, y: 100.0 }.into(),
            scale: waypoint_scale.into(),
            color: waypoint_color.into(),
        },
        Geom {
            translate: Vec2 { x: -150.0, y: 100.0 }.into(),
            scale: waypoint_scale.into(),
            color: waypoint_color.into(),
        },
        Geom {
            translate: Vec2 { x: -150.0, y: -150.0 }.into(),
            scale: waypoint_scale.into(),
            color: waypoint_color.into(),
        },
        Geom {
            translate: Vec2 { x: -250.0, y: -150.0 }.into(),
            scale: waypoint_scale.into(),
            color: waypoint_color.into(),
        },
        Geom {
            translate: Vec2 { x: -250.0, y: -250.0 }.into(),
            scale: waypoint_scale.into(),
            color: waypoint_color.into(),
        },
        Geom {
            translate: Vec2 { x: -50.0, y: -250.0 }.into(),
            scale: waypoint_scale.into(),
            color: waypoint_color.into(),
        },
        Geom {
            translate: Vec2::default().into(),
            scale: PLAYER_QUAD_SCALE.into(),
            color: player_quad_color.into(),
        },
    ];
    let player_quad_idx = quads.len() - 1;
    let mut player_waypoint_idx = 5;

    let mut weights = [[0.0; WAYPOINT_LEN]; WAYPOINT_LEN];
    let mut nodes: [Vec2<f32>; WAYPOINT_LEN] = [Vec2::default(); WAYPOINT_LEN];
    for i in 0..WAYPOINT_LEN {
        nodes[i] = quads[FIRST_WAYPOINT_INDEX + i].translate.0;
    }
    let mut edges: [(usize, usize); WAYPOINT_LEN] = [(WAYPOINT_LEN, WAYPOINT_LEN); WAYPOINT_LEN];

    #[allow(clippy::needless_range_loop)]
    for i in 0..WAYPOINT_LEN {
        edges[i] = (i, (i + 1) % WAYPOINT_LEN);
    }

    let mut lines = [
        Geom {
            translate: Vec2 { x: 100.0, y: 0.0 }.into(),
            scale: Vec2 { x: 310.0, y: 0.0 }.into(),
            color: path_color.into(),
        },
        Geom {
            translate: Vec2 { x: -50.0, y: -125.0 }.into(),
            scale: Vec2 { x: 0.0, y: 260.0 }.into(),
            color: path_color.into(),
        },
        Geom {
            translate: Vec2 { x: 250.0, y: 125.0 }.into(),
            scale: Vec2 { x: 0.0, y: 260.0 }.into(),
            color: path_color.into(),
        },
        Geom {
            translate: Vec2 { x: 200.0, y: 250.0 }.into(),
            scale: Vec2 { x: 110.0, y: 0.0 }.into(),
            color: path_color.into(),
        },
        Geom {
            translate: Vec2 { x: 150.0, y: 175.0 }.into(),
            scale: Vec2 { x: 0.0, y: 160.0 }.into(),
            color: path_color.into(),
        },
        Geom {
            translate: Vec2 { x: 0.0, y: 100.0 }.into(),
            scale: Vec2 { x: 310.0, y: 0.0 }.into(),
            color: path_color.into(),
        },
        Geom {
            translate: Vec2 { x: -150.0, y: -25.0 }.into(),
            scale: Vec2 { x: 0.0, y: 260.0 }.into(),
            color: path_color.into(),
        },
        Geom {
            translate: Vec2 { x: -200.0, y: -150.0 }.into(),
            scale: Vec2 { x: 110.0, y: 0.0 }.into(),
            color: path_color.into(),
        },
        Geom {
            translate: Vec2 { x: -150.0, y: -250.0 }.into(),
            scale: Vec2 { x: 210.0, y: 0.0 }.into(),
            color: path_color.into(),
        },
        Geom {
            translate: Vec2 { x: -250.0, y: -200.0 }.into(),
            scale: Vec2 { x: 0.0, y: 110.0 }.into(),
            color: path_color.into(),
        },
        Geom {
            translate: Vec2::default().into(),
            scale: Vec2::default().into(),
            color: cursor_line_color.into(),
        },
        Geom {
            translate: Vec2::default().into(),
            scale: Vec2::default().into(),
            color: player_line_color.into(),
        },
    ];
    let cursor_line_idx = lines.len() - 2;
    let player_line_idx = lines.len() - 1;

    let quad_vertices = [
        Vec2 { x: 0.5, y: 0.5 },
        Vec2 { x: 0.5, y: -0.5 },
        Vec2 { x: -0.5, y: 0.5 },
        Vec2 { x: -0.5, y: -0.5 },
    ];
    let line_vertices = [Vec2 { x: -0.5, y: -0.5 }, Vec2 { x: 0.5, y: 0.5 }];

    let line_width = 4.0;

    unsafe {
        println!("{}", CStr::from_ptr(ffi::glfwGetVersionString()).to_str().unwrap());

        ffi::glfwSetErrorCallback(callback_glfw_error);

        assert!(ffi::glfwInit() == 1);
        defer!(ffi::glfwTerminate());

        ffi::glfwWindowHint(ffi::GLFW_OPENGL_DEBUG_CONTEXT, 1);
        ffi::glfwWindowHint(ffi::GLFW_CONTEXT_VERSION_MAJOR, 3);
        ffi::glfwWindowHint(ffi::GLFW_CONTEXT_VERSION_MINOR, 3);
        ffi::glfwWindowHint(ffi::GLFW_OPENGL_PROFILE, ffi::GLFW_OPENGL_CORE_PROFILE);
        ffi::glfwWindowHint(ffi::GLFW_RESIZABLE, 0);
        ffi::glfwWindowHint(ffi::GLFW_SAMPLES, 16);

        let window = ffi::glfwCreateWindow(
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            CString::new(std::module_path!())
                .unwrap()
                .as_bytes_with_nul()
                .as_ptr()
                .cast::<c_char>(),
            ptr::null_mut::<ffi::GLFWmonitor>(),
            ptr::null_mut::<ffi::GLFWwindow>(),
        );

        assert!(!window.is_null());
        defer!(ffi::glfwDestroyWindow(window));

        ffi::glfwMakeContextCurrent(window);
        ffi::glfwSwapInterval(1);
        ffi::glfwSetKeyCallback(window, callback_glfw_key);

        ffi::glEnable(ffi::GL_DEBUG_OUTPUT);
        ffi::glEnable(ffi::GL_DEBUG_OUTPUT_SYNCHRONOUS);
        ffi::glDebugMessageCallback(callback_gl_debug, ptr::null::<c_void>());

        ffi::glEnable(ffi::GL_BLEND);
        ffi::glBlendFunc(ffi::GL_SRC_ALPHA, ffi::GL_ONE_MINUS_SRC_ALPHA);
        ffi::glClearColor(
            background_color.x,
            background_color.y,
            background_color.z,
            background_color.w,
        );
        ffi::glEnable(ffi::GL_MULTISAMPLE);
        ffi::glViewport(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT);

        let mut vao: [ffi::GLuint; 2] = [0; 2];
        ffi::glGenVertexArrays(vao.len().try_into().unwrap(), vao.as_mut_ptr());
        defer!(ffi::glDeleteVertexArrays(vao.len().try_into().unwrap(), vao.as_ptr()));

        let mut vbo: [ffi::GLuint; 2] = [0; 2];
        ffi::glGenBuffers(vbo.len().try_into().unwrap(), vbo.as_mut_ptr());
        defer!(ffi::glDeleteBuffers(vbo.len().try_into().unwrap(), vbo.as_ptr()));

        let mut instance_vbo: [ffi::GLuint; 2] = [0; 2];
        ffi::glGenBuffers(instance_vbo.len().try_into().unwrap(), instance_vbo.as_mut_ptr());
        defer!(ffi::glDeleteBuffers(
            instance_vbo.len().try_into().unwrap(),
            instance_vbo.as_ptr()
        ));

        let program = create_program();
        defer!(ffi::glDeleteProgram(program));
        ffi::glUseProgram(program);

        ffi::glLineWidth(line_width);
        ffi::glEnable(ffi::GL_LINE_SMOOTH);

        uniform!(program, projection);

        buffers_and_attributes(program, vao[0], vbo[0], instance_vbo[0], &quads, &quad_vertices);
        buffers_and_attributes(program, vao[1], vbo[1], instance_vbo[1], &lines, &line_vertices);

        let mut world_cursor = Vec2::default();
        let mut now = time::Instant::now();
        let mut frames = 0;

        println!("\n\n\n\n");
        while ffi::glfwWindowShouldClose(window) != 1 {
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

            ffi::glfwPollEvents();

            update_camera(window, &mut camera, &mut camera_speed);
            let cursor_waypoint_idx =
                update_cursor(window, camera, &mut world_cursor, &quads, &inverse_projection);
            update_player(
                &mut quads,
                &mut weights,
                &nodes,
                &edges,
                &mut player_speed,
                player_quad_idx,
                &mut player_waypoint_idx,
                cursor_waypoint_idx,
            );
            update_lines(
                &quads,
                &mut lines,
                player_speed,
                player_quad_idx,
                player_line_idx,
                world_cursor,
                cursor_line_idx,
            );

            quads[cursor_waypoint_idx].color.0.x = 1.0;

            let view = math::look_at(camera, Vec3 { z: 0.0, ..camera }, VIEW_UP);
            uniform!(program, view);

            ffi::glClear(ffi::GL_COLOR_BUFFER_BIT);

            bind_and_draw(vao[0], instance_vbo[0], &quads, &quad_vertices, ffi::GL_TRIANGLE_STRIP);
            bind_and_draw(vao[1], instance_vbo[1], &lines, &line_vertices, ffi::GL_LINES);

            ffi::glfwSwapBuffers(window);

            quads[cursor_waypoint_idx].color.0.x = waypoint_color.x;

            frames += 1;
        }
    }
}
