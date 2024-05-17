mod defer;
mod ffi;
mod math;
mod prelude;

use crate::defer::Defer;
use std::convert::TryInto;
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::fs::read_to_string;
use std::mem;
use std::path::Path;
use std::ptr;
use std::slice::from_raw_parts;
use std::str::from_utf8_unchecked;
use std::time;

#[repr(C)]
struct Geom<T> {
    translate: math::Vec2<T>,
    scale: math::Vec2<T>,
    color: math::Vec3<T>,
}

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
            (mem::size_of::<$ty>() / mem::size_of::<ffi::GLfloat>())
                .try_into()
                .unwrap(),
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
            $ident.as_ptr().cast::<ffi::GLfloat>(),
        );
    };
}

fn buffers_and_attributes(
    program: ffi::GLuint,
    vao: ffi::GLuint,
    vbo: ffi::GLuint,
    instance_vbo: ffi::GLuint,
    geoms: &[Geom<ffi::GLfloat>],
    vertices: &[math::Vec2<ffi::GLfloat>],
) {
    unsafe {
        ffi::glBindVertexArray(vao);

        buffer(vbo, vertices, ffi::GL_STATIC_DRAW);
        attribute!(program, math::Vec2<ffi::GLfloat>, position);

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
    vertices: &[math::Vec2<ffi::GLfloat>],
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

fn main() {
    let window_width = 1400;
    let window_height = 900;

    let view_zoom = 1.5;

    #[allow(clippy::cast_precision_loss)]
    let view_translate = math::Vec2 {
        x: ((window_width / 2) as f32) / view_zoom,
        y: ((window_height / 2) as f32) / view_zoom,
    };
    let mut view_rotate = std::f32::consts::PI;

    let acceleration = 3.125;
    let drag = 0.81125;
    let spin = 0.005;

    let background_color = math::Vec3 {
        x: 0.1,
        y: 0.09,
        z: 0.11,
    };

    let mut quads = [Geom {
        translate: math::Vec2::default(),
        scale: 25.0.into(),
        color: math::Vec3 {
            x: 1.0,
            y: 0.5,
            z: 0.75,
        },
    }];
    let mut lines = [Geom {
        translate: math::Vec2::default(),
        scale: 1000.0.into(),
        color: math::Vec3 {
            x: 0.75,
            y: 0.1,
            z: 0.25,
        },
    }];

    let quad_vertices = [
        math::Vec2 { x: 0.5, y: 0.5 },
        math::Vec2 { x: 0.5, y: -0.5 },
        math::Vec2 { x: -0.5, y: 0.5 },
        math::Vec2 { x: -0.5, y: -0.5 },
    ];
    let line_vertices = [
        math::Vec2 { x: -0.5, y: -0.5 },
        math::Vec2 { x: 0.5, y: 0.5 },
    ];

    let line_width = 5.0;

    #[allow(clippy::cast_precision_loss)]
    let projection = math::orthographic(
        0.0,
        (window_width as f32) / view_zoom,
        (window_height as f32) / view_zoom,
        0.0,
        -1.0,
        1.0,
    );

    unsafe {
        println!(
            "{}",
            CStr::from_ptr(ffi::glfwGetVersionString())
                .to_str()
                .unwrap(),
        );

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
            window_width,
            window_height,
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
        ffi::glClearColor(background_color.x, background_color.y, background_color.z, 1.0);
        ffi::glEnable(ffi::GL_MULTISAMPLE);
        ffi::glViewport(0, 0, window_width, window_height);

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

        let mut now = time::Instant::now();
        let mut frames = 0;

        let mut speed: math::Vec2<f32> = math::Vec2::default();

        println!("\n");
        while ffi::glfwWindowShouldClose(window) != 1 {
            if 0 < now.elapsed().as_secs() {
                println!(
                    "\x1B[2A\
                     {:10} ns/f\n\
                     {frames:10} frames",
                    now.elapsed().as_nanos() / frames,
                );
                now = time::Instant::now();
                frames = 0;
            }

            ffi::glfwPollEvents();

            let mut r#move: math::Vec2<f32> = math::Vec2::default();
            if pressed(window, ffi::GLFW_KEY_W) {
                r#move.y -= 1.0;
            }
            if pressed(window, ffi::GLFW_KEY_S) {
                r#move.y += 1.0;
            }
            if pressed(window, ffi::GLFW_KEY_A) {
                r#move.x -= 1.0;
            }
            if pressed(window, ffi::GLFW_KEY_D) {
                r#move.x += 1.0;
            }
            math::turn(&mut r#move, math::Vec2::default(), view_rotate);
            math::normalize(&mut r#move);

            speed += r#move * acceleration.into();
            speed *= drag.into();
            quads[0].translate += speed;
            lines[0].translate += speed;

            view_rotate += spin;
            // TODO: This doesn't spin correctly.
            let view = math::translate_and_rotate(view_translate, view_rotate);
            uniform!(program, view);

            ffi::glClear(ffi::GL_COLOR_BUFFER_BIT);

            bind_and_draw(vao[0], instance_vbo[0], &quads, &quad_vertices, ffi::GL_TRIANGLE_STRIP);
            bind_and_draw(vao[1], instance_vbo[1], &lines, &line_vertices, ffi::GL_LINES);

            ffi::glfwSwapBuffers(window);

            frames += 1;
        }
    }
}
